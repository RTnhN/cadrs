//! 3-axis finishing: Parallel (raster passes over the surfaces, each point dropped onto the
//! model by the cutter's shape) and Constant Z (waterlines round the model at levels).

use crate::dropcutter::{TriIndex, drop_line};
use crate::geom::{P2, P3, dist2};
use crate::region::{self, Region};
use crate::toolpath::{FeedKind, Toolpath};

use super::Input;
use super::link::{Builder, Cut};
use super::mill::material_above;
use super::path::{Contour, hatch, nearest_order, region_contours};
use super::{CuttingMethod, Direction};

/// Drops 3D points within tolerance of each other in a line (fewer, longer moves).
fn simplify3(pts: &[P3], tol: f64) -> Vec<P3> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![false; pts.len()];
    keep[0] = true;
    keep[pts.len() - 1] = true;
    let mut stack = vec![(0usize, pts.len() - 1)];
    while let Some((a, b)) = stack.pop() {
        let (pa, pb) = (pts[a], pts[b]);
        let d = crate::geom::sub(pb, pa);
        let l2 = crate::geom::dot(d, d).max(1e-18);
        let mut worst = (0.0, a);
        for (i, p) in pts.iter().enumerate().take(b).skip(a + 1) {
            let t = (crate::geom::dot(crate::geom::sub(*p, pa), d) / l2).clamp(0.0, 1.0);
            let q = crate::geom::add(pa, crate::geom::scale(d, t));
            let e = crate::geom::dist(*p, q);
            if e > worst.0 {
                worst = (e, i);
            }
        }
        if worst.0 > tol {
            keep[worst.1] = true;
            stack.push((a, worst.1));
            stack.push((worst.1, b));
        }
    }
    pts.iter()
        .zip(keep)
        .filter(|(_, k)| *k)
        .map(|(p, _)| *p)
        .collect()
}

/// The finishing area: the picked faces' footprint, else the model's, within the stock.
fn finish_area(input: &Input) -> Region {
    let r = input.tool.radius();
    let d = &input.params.drive;
    let base = if d.faces.is_empty() {
        input.model.silhouette()
    } else {
        let tris = d
            .faces
            .iter()
            .filter_map(|n| input.model.face(n))
            .flat_map(|f| f.triangles.iter())
            .map(|&t| {
                input.model.triangles[t as usize]
                    .iter()
                    .map(|p| [p[0], p[1]])
                    .collect::<Vec<P2>>()
            });
        region::union_all(tris)
    };
    region::intersection(
        &region::offset(&base, r * 0.5),
        &region::offset(&input.stock.footprint(), r),
    )
}

/// Parallel: raster lines over the area at the angle, each sampled and dropped onto the model.
pub fn parallel(input: &Input) -> Result<Toolpath, String> {
    let p = input.params;
    let area = finish_area(input);
    if area.is_empty() {
        return Err("Nothing to finish inside the stock".into());
    }
    let idx = TriIndex::new(&input.model.triangles);
    let bottom = input.bottom();
    let top = input.top();
    let zig = p.cutting_method == CuttingMethod::Zigzag;
    let lines = hatch(&area, p.angle, p.step_over, zig);
    // Samples every half radius at most, split where the surface needs it (exact drops).
    let step = (input.tool.radius() / 2.0).clamp(0.05, 1.0);
    let min_step = (p.tolerance * 2.0).max(0.01);
    let mut b = Builder::new(input);
    let drop = |pts: &[P2]| -> Vec<P3> {
        drop_line(
            input.tool,
            &idx,
            pts,
            bottom - p.floor_stock,
            step,
            p.tolerance,
            min_step,
        )
        .into_iter()
        .map(|q| [q[0], q[1], (q[2] + p.floor_stock).max(bottom)])
        .collect()
    };
    let mut prev_end: Option<P3> = None;
    let mut skipped_high = false;
    for l in lines {
        let pts = drop(&l);
        if pts.iter().all(|q| q[2] > top + 1e-6) {
            skipped_high = true;
            continue;
        }
        let pts = simplify3(&pts, p.tolerance);
        let start = pts[0];
        // A link that follows the surface from the last line's end, when it's short.
        let linked = match prev_end {
            Some(e)
                if zig
                    && dist2([e[0], e[1]], [start[0], start[1]])
                        <= (p.step_over * 3.0).max(p.links.small_gap) =>
            {
                for q in drop(&[[e[0], e[1]], [start[0], start[1]]]).iter().skip(1) {
                    b.feed(*q, FeedKind::Link);
                }
                true
            }
            _ => false,
        };
        if !linked {
            let material_top = input.stock.bounds.max[2].min(start[2] + p.step_down.max(1.0));
            let cut = Cut {
                material_top,
                direct_ok: false,
                free_side: None,
                tabs: None,
                ramp: false,
            };
            b.travel_to([start[0], start[1]], start[2], &cut);
            b.feed(start, FeedKind::Plunge);
        }
        for q in pts.iter().skip(1) {
            b.feed(*q, FeedKind::Cut);
        }
        prev_end = pts.last().copied();
    }
    if skipped_high {
        b.warnings
            .push("Passes entirely above the top height were left out".into());
    }
    Ok(b.finish())
}

/// The region the tool's centre can't enter with its tip at `z`: the material above, offset by
/// the cutter's radius at each height up its flank (exact for a flat end mill; sampled up a
/// ball or bull nose).
fn keep_out(input: &Input, z: f64) -> Region {
    let t = input.tool;
    let r = t.radius();
    let w = input.params.wall_stock;
    let samples: Vec<f64> = match t.kind {
        crate::tool::ToolKind::EndMill | crate::tool::ToolKind::FaceMill => vec![0.0],
        _ => (0..=6).map(|i| r * i as f64 / 6.0).collect(),
    };
    let mut out: Region = Vec::new();
    for h in samples {
        let solid = material_above(input, z + h);
        if solid.is_empty() {
            continue;
        }
        let o = region::offset(&solid, t.radius_at_height(h) + w);
        out = region::union(&out, &o);
    }
    out
}

/// Constant Z: waterlines round the model at each level, climb or conventional, with leads.
pub fn constant_z(input: &Input) -> Result<Toolpath, String> {
    let p = input.params;
    let mb = input.model.bounds();
    let top = input.top().min(mb.max[2]);
    let bottom = input.bottom().max(mb.min[2]);
    let levels = input.levels(top, bottom, true);
    let mut b = Builder::new(input);
    let side = if p.direction == Direction::Climb {
        1.0
    } else {
        -1.0
    };
    let mut above = top;
    for z in levels {
        let ko = keep_out(input, z);
        if ko.is_empty() {
            above = z;
            continue;
        }
        let rings = region_contours(&ko)
            .into_iter()
            // The material is inside the keep-out region: climb keeps it on the right.
            .map(|c| {
                if p.direction == Direction::Climb {
                    c.reversed()
                } else {
                    c
                }
            })
            .collect::<Vec<Contour>>();
        let start = b.at.map_or([mb.min[0], mb.min[1]], |a| [a[0], a[1]]);
        for c in nearest_order(rings, start) {
            let cut = Cut {
                material_top: above,
                direct_ok: false,
                free_side: Some(side),
                tabs: None,
                ramp: false,
            };
            b.cut(&c, z, &cut);
        }
        above = z;
    }
    if !b.moved() {
        return Err("No walls between the heights".into());
    }
    Ok(b.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Aabb;
    use crate::machine::MachineKind;
    use crate::model::{Model, box_triangles};
    use crate::strategy::{Height, HeightRef, Params, Pattern, Stock, generate};
    use crate::tool::{Tool, ToolKind};
    use crate::toolpath::Step;

    /// A dome-ish stepped part: a 40 × 40 × 10 base with a 20 × 20 block up to 20.
    fn stepped() -> Model {
        let mut t = box_triangles(&Aabb::new([0.0, 0.0, 0.0], [40.0, 40.0, 10.0]));
        t.extend(box_triangles(&Aabb::new(
            [10.0, 10.0, 10.0],
            [30.0, 30.0, 20.0],
        )));
        Model::from_triangles("stepped", t)
    }

    fn run(pattern: Pattern, tool: Tool, f: impl FnOnce(&mut Params)) -> Toolpath {
        let m = stepped();
        let stock = Stock::block(Aabb::new([-2.0, -2.0, 0.0], [42.0, 42.0, 21.0]));
        let mut p = Params::new(pattern, &tool, MachineKind::Mill);
        f(&mut p);
        let input = Input {
            model: &m,
            stock: &stock,
            tool: &tool,
            machine: MachineKind::Mill,
            params: &p,
            rest: None,
        };
        generate(&input).unwrap()
    }

    #[test]
    fn parallel_ball_never_goes_under_the_surface() {
        let tool = Tool::new(ToolKind::BallMill, 6.0);
        let tp = run(Pattern::Parallel, tool.clone(), |p| {
            p.step_over = 2.0;
            p.heights.bottom = Height::new(HeightRef::ModelBottom, 0.0);
        });
        let m = stepped();
        let idx = TriIndex::new(&m.triangles);
        let mut scratch = Vec::new();
        let mut n = 0;
        for s in &tp.steps {
            if let Step::Feed {
                to,
                kind: FeedKind::Cut,
                ..
            } = s
            {
                let lowest =
                    crate::dropcutter::drop_point(&tool, &idx, [to[0], to[1]], 0.0, &mut scratch);
                assert!(to[2] >= lowest - 1e-9, "{to:?} below {lowest}");
                n += 1;
            }
        }
        assert!(n > 50);
        // It goes over the top of the boss.
        assert!(
            tp.steps
                .iter()
                .any(|s| matches!(s, Step::Feed { to, .. } if (to[2] - 20.0).abs() < 1e-6))
        );
    }

    #[test]
    fn constant_z_rings_round_the_boss() {
        let tp = run(Pattern::ConstantZ, Tool::new(ToolKind::EndMill, 6.0), |p| {
            p.step_down = 2.5;
            p.links.lead_in = crate::strategy::LeadKind::None;
            p.links.lead_out = crate::strategy::LeadKind::None;
        });
        let at15: Vec<[f64; 3]> = tp
            .steps
            .iter()
            .filter_map(|s| match s {
                Step::Feed {
                    to,
                    kind: FeedKind::Cut,
                    ..
                } if (to[2] - 15.0).abs() < 1e-6 => Some(*to),
                _ => None,
            })
            .collect();
        assert!(!at15.is_empty());
        // 3 mm round the 20 × 20 boss.
        let maxx = at15.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        assert!((maxx - 33.0).abs() < 0.01, "{maxx}");
    }

    #[test]
    fn simplify_keeps_corners() {
        let pts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
        ];
        assert_eq!(simplify3(&pts, 0.01).len(), 3);
    }
}
