//! Laser, plasma and waterjet: Cut (through-cutting, kerf compensated), Etch (marking along
//! edges) and Fill (laser hatching of faces).
//!
//! A cut is ordered inside out: contours nested deeper are cut first (a hole before the part
//! round it, a part in a hole before the hole), so nothing falls before it's cut. Each contour
//! is pierced in the scrap (beside the cut, through a lead-in) at the pierce height, the beam
//! dwells for the pierce delay, drops to its cut height and runs round with the part on its
//! right (the good side of a clockwise-swirl plasma torch), past the start by the overlap.
//! Microjoints leave short uncut bridges that hold the parts in the sheet.

use crate::geom::{P2, P3, dist2, lerp2, ring_area2, ring_contains};
use crate::machine::MachineKind;
use crate::region::{self, Region};
use crate::toolpath::{FeedKind, Step, Toolpath};

use super::link::Builder;
use super::path::{
    Contour, chain, depth_in, hatch, inner_point, lead_arc, nearest_order, region_contours,
};
use super::{CuttingMethod, Direction, Input, LeadKind, Side};

/// Heights of a beam cut: the top of the sheet, travel, pierce and cut heights.
struct Z {
    travel: f64,
    pierce: f64,
    cut: f64,
}

fn heights(input: &Input) -> Z {
    let top = input.top();
    let b = &input.tool.beam;
    let (pierce, cut) = match input.machine {
        MachineKind::Plasma => (top + b.pierce_height, top + b.cut_height),
        MachineKind::Waterjet => (top + b.cut_height, top + b.cut_height),
        _ => (top + b.cut_height, top + b.cut_height),
    };
    let travel = (top + input.params.links.clearance).max(pierce);
    Z {
        travel,
        pierce,
        cut,
    }
}

/// The contours a cut follows, kerf compensated, each with whether it's a hole (scrap inside).
fn cut_contours(input: &Input) -> Result<Vec<(Contour, bool)>, String> {
    let p = input.params;
    let k = input.tool.radius() + p.wall_stock;
    let d = &p.drive;
    let mut out = Vec::new();
    if !d.edges.is_empty() {
        let polys: Vec<Vec<P3>> = d
            .edges
            .iter()
            .filter_map(|n| input.model.edge(n))
            .map(|e| e.points.clone())
            .collect();
        for c in chain(&polys, 1e-3) {
            if !c.closed || p.side == Side::On {
                out.push((c, false));
                continue;
            }
            let z = input.selection_z().unwrap_or(input.model.bounds().max[2]);
            let part_inside = input.part_inside(&c, z);
            let mut ring = c.pts.clone();
            if ring_area2(&ring) < 0.0 {
                ring.reverse();
            }
            let outside = match p.side {
                Side::Outside => true,
                Side::Inside => false,
                _ => part_inside,
            };
            let off = region::offset(&[ring], if outside { k } else { -k });
            for cc in region_contours(&off) {
                // Part inside an outside cut; scrap inside an inside cut.
                out.push((cc, !outside));
            }
        }
        if out.is_empty() {
            return Err("The picked edges make no contour".into());
        }
        return Ok(out);
    }
    let sil = input.model.silhouette();
    if sil.is_empty() {
        return Err("The job has no bodies to cut".into());
    }
    let off = match p.side {
        Side::On => sil,
        Side::Inside => region::offset(&sil, -k),
        _ => region::offset(&sil, k),
    };
    for c in region_contours(&off) {
        let hole = !c.ccw();
        out.push((c, hole));
    }
    Ok(out)
}

/// The radius of the largest circle in a ring, roughly (twice the area over the perimeter).
fn inscribed(ring: &[P2]) -> f64 {
    let a = ring_area2(ring).abs() / 2.0;
    let per = crate::geom::ring_len(ring).max(1e-9);
    2.0 * a / per
}

/// The part of a walk between arc lengths `s0` and `s1`.
fn sub_path(walk: &[P2], s0: f64, s1: f64) -> Vec<P2> {
    let mut out = Vec::new();
    let mut s = 0.0;
    for w in walk.windows(2) {
        let l = dist2(w[0], w[1]);
        let (a, b) = (s, s + l);
        s = b;
        if l <= 0.0 || b < s0 || a > s1 {
            continue;
        }
        if out.is_empty() {
            out.push(lerp2(w[0], w[1], ((s0 - a) / l).clamp(0.0, 1.0)));
        }
        if b <= s1 {
            out.push(w[1]);
        } else {
            out.push(lerp2(w[0], w[1], ((s1 - a) / l).clamp(0.0, 1.0)));
            break;
        }
    }
    out
}

/// Splits a closed walk into the pieces between `count` gaps of `width`, evenly spaced along
/// it (the first half a spacing from the start).
fn microjoint_pieces(walk: &[P2], count: usize, width: f64) -> Vec<Vec<P2>> {
    let total: f64 = walk.windows(2).map(|w| dist2(w[0], w[1])).sum();
    if count == 0 || width * count as f64 >= total * 0.5 {
        return vec![walk.to_vec()];
    }
    let spacing = total / count as f64;
    let mut pieces = Vec::new();
    let mut s = 0.0;
    for i in 0..count {
        let c = spacing * (i as f64 + 0.5);
        pieces.push(sub_path(walk, s, c - width / 2.0));
        s = c + width / 2.0;
    }
    pieces.push(sub_path(walk, s, total));
    pieces.retain(|p| p.len() >= 2);
    pieces
}

/// Pierce, beam on, down to the cut height.
fn pierce(b: &mut Builder, xy: P2, z: &Z, power: f64, delay: f64) {
    b.up_to(z.travel);
    b.rapid([xy[0], xy[1], z.travel]);
    b.rapid([xy[0], xy[1], z.pierce]);
    b.steps.push(Step::BeamOn { power });
    if delay > 0.0 {
        b.steps.push(Step::Dwell(delay));
    }
    if (z.pierce - z.cut).abs() > 1e-9 {
        b.feed([xy[0], xy[1], z.cut], FeedKind::Plunge);
    }
}

fn beam_off(b: &mut Builder, z: &Z) {
    b.steps.push(Step::BeamOff);
    b.up_to(z.travel);
}

/// Cut: through-cutting with kerf compensation, inside first.
pub fn cut(input: &Input) -> Result<Toolpath, String> {
    let p = input.params;
    let t = input.tool;
    let contours = cut_contours(input)?;
    let rings: Vec<Vec<P2>> = contours
        .iter()
        .filter(|(c, _)| c.closed)
        .map(|(c, _)| c.pts.clone())
        .collect();
    // Deepest nesting first, then nearest.
    let mut by_depth: Vec<(usize, Contour, bool)> = contours
        .into_iter()
        .map(|(c, hole)| {
            // How many other contours it lies in (its own boundary says nothing).
            let d = if c.closed {
                rings
                    .iter()
                    .filter(|r| **r != c.pts)
                    .filter(|r| ring_contains(r, c.pts[0]))
                    .count()
            } else {
                usize::MAX
            };
            (d, c, hole)
        })
        .collect();
    by_depth.sort_by_key(|x| std::cmp::Reverse(x.0));
    let z = heights(input);
    let mut b = Builder::new(input);
    let power = (p.beam.power / 100.0).clamp(0.0, 1.0);
    let delay = t.beam.pierce_delay;
    let passes = p.beam.passes.max(1);
    let mut from: P2 = {
        let mb = input.model.bounds();
        [mb.min[0], mb.min[1]]
    };
    let mut i = 0;
    while i < by_depth.len() {
        let d = by_depth[i].0;
        let group: Vec<(Contour, bool)> = by_depth
            .iter()
            .skip(i)
            .take_while(|x| x.0 == d)
            .map(|x| (x.1.clone(), x.2))
            .collect();
        i += group.len();
        let holes: Vec<bool> = group.iter().map(|g| g.1).collect();
        let cs: Vec<Contour> = group.into_iter().map(|g| g.0).collect();
        for c in nearest_order(cs.clone(), from) {
            let hole = cs
                .iter()
                .position(|x| x.pts.len() == c.pts.len() && x.pts.iter().all(|q| c.pts.contains(q)))
                .map(|k| holes[k])
                .unwrap_or(false);
            // The region's rings have the part (and its kerf) on their left; the part on the
            // right (climb) is every ring reversed.
            let c = if c.closed && p.direction == Direction::Climb {
                c.reversed().starting_near(from)
            } else if c.closed {
                c.starting_near(from)
            } else {
                c
            };
            let feed =
                if hole && p.beam.small_hole > 0.0 && inscribed(&c.pts) * 2.0 < p.beam.small_hole {
                    p.speeds.feed * p.beam.small_hole_feed
                } else {
                    p.speeds.feed
                };
            for pass in 0..passes {
                let zc = z.cut - pass as f64 * p.beam.pass_depth;
                let zz = Z {
                    travel: z.travel,
                    pierce: z.pierce - pass as f64 * p.beam.pass_depth,
                    cut: zc,
                };
                cut_one(&mut b, input, &c, hole, &zz, power, delay, feed);
            }
            from = b.at.map_or(from, |a| [a[0], a[1]]);
        }
    }
    let mut tp = b.finish();
    tp.steps.retain(|s| !matches!(s, Step::Spindle { .. }));
    Ok(tp)
}

#[allow(clippy::too_many_arguments)]
fn cut_one(
    b: &mut Builder,
    input: &Input,
    c: &Contour,
    hole: bool,
    z: &Z,
    power: f64,
    delay: f64,
    feed: f64,
) {
    let p = input.params;
    let l = &p.links;
    let walk = c.walk();
    let start = walk[0];
    // The lead: in the scrap, which is on the left with the part on the right.
    let scrap_side = if p.direction == Direction::Climb {
        1.0
    } else {
        -1.0
    };
    let mut lead: Option<(P2, Option<(P2, bool)>)> = None;
    if c.closed
        && l.lead_in != LeadKind::None
        && l.lead_radius > 0.0
        && let Some((pt, t)) = c.start_tangent()
    {
        let room = if hole {
            inscribed(&c.pts)
        } else {
            f64::INFINITY
        };
        if l.lead_radius < room * 0.8 {
            lead = Some(match l.lead_in {
                LeadKind::Arc => {
                    let (s, ctr, cw) = lead_arc(pt, t, scrap_side, l.lead_radius, 90.0);
                    (s, Some((ctr, cw)))
                }
                _ => {
                    let n = [-t[1] * scrap_side, t[0] * scrap_side];
                    (
                        [pt[0] + n[0] * l.lead_radius, pt[1] + n[1] * l.lead_radius],
                        None,
                    )
                }
            });
        } else if hole && room > input.tool.radius() * 2.0 {
            // A small hole: pierce in its middle and go straight to the edge.
            let mid = inner_point(&c.pts);
            if dist2(mid, start) > input.tool.radius() {
                lead = Some((mid, None));
            }
        }
    }
    let pierce_at = lead.map_or(start, |(s, _)| s);
    pierce(b, pierce_at, z, power, delay);
    if let Some((_, arc)) = lead {
        match arc {
            Some((ctr, cw)) => b.arc([start[0], start[1], z.cut], ctr, cw, FeedKind::LeadIn),
            None => b.feed([start[0], start[1], z.cut], FeedKind::LeadIn),
        }
    }
    // Microjoints hold parts, not the scrap in their holes.
    let pieces = if c.closed && !hole && p.beam.microjoints > 0 {
        microjoint_pieces(
            &walk,
            p.beam.microjoints,
            p.beam.microjoint_width + input.tool.diameter,
        )
    } else {
        vec![walk.clone()]
    };
    for (k, piece) in pieces.iter().enumerate() {
        if k > 0 {
            // Over the bridge with the beam off, then on again.
            b.steps.push(Step::BeamOff);
            b.rapid([piece[0][0], piece[0][1], z.cut]);
            b.steps.push(Step::BeamOn { power });
            if delay > 0.0 {
                b.steps.push(Step::Dwell(delay));
            }
        }
        for q in piece.iter().skip(1) {
            b.feed_at([q[0], q[1], z.cut], FeedKind::Cut, feed);
        }
    }
    // Past the start by the overlap (unless microjoints break the loop there).
    if c.closed && l.overlap > 0.0 && p.beam.microjoints == 0 {
        let mut left = l.overlap;
        for w in walk.windows(2) {
            let d = dist2(w[0], w[1]);
            if d >= left {
                let q = lerp2(w[0], w[1], left / d);
                b.feed_at([q[0], q[1], z.cut], FeedKind::Cut, feed);
                break;
            }
            b.feed_at([w[1][0], w[1][1], z.cut], FeedKind::Cut, feed);
            left -= d;
        }
    }
    beam_off(b, z);
}

/// Etch: along the picked edges (or the picked faces' outlines) at marking power, no
/// compensation.
pub fn etch(input: &Input) -> Result<Toolpath, String> {
    let p = input.params;
    let d = &p.drive;
    let mut cs: Vec<Contour> = Vec::new();
    if !d.edges.is_empty() {
        let polys: Vec<Vec<P3>> = d
            .edges
            .iter()
            .filter_map(|n| input.model.edge(n))
            .map(|e| e.points.clone())
            .collect();
        cs = chain(&polys, 1e-3);
    }
    for f in d.faces.iter().filter_map(|n| input.model.face(n)) {
        for l in &f.loops {
            cs.push(Contour::closed(l.iter().map(|q| [q[0], q[1]]).collect()));
        }
    }
    if cs.is_empty() {
        return Err("Pick the edges or faces to etch".into());
    }
    let z = heights(input);
    let power = (p.beam.power / 100.0).clamp(0.0, 1.0);
    let mut b = Builder::new(input);
    let mb = input.model.bounds();
    for c in nearest_order(cs, [mb.min[0], mb.min[1]]) {
        let walk = c.walk();
        // Marking doesn't need the full pierce delay.
        pierce(&mut b, walk[0], &z, power, 0.0);
        for q in walk.iter().skip(1) {
            b.feed([q[0], q[1], z.cut], FeedKind::Cut);
        }
        beam_off(&mut b, &z);
    }
    let mut tp = b.finish();
    tp.steps.retain(|s| !matches!(s, Step::Spindle { .. }));
    Ok(tp)
}

/// Fill: hatches the picked faces (laser engraving an area).
pub fn fill(input: &Input) -> Result<Toolpath, String> {
    let p = input.params;
    let areas: Region = p
        .drive
        .faces
        .iter()
        .filter_map(|n| input.model.face(n))
        .flat_map(|f| {
            f.loops
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    let mut r: Vec<P2> = l.iter().map(|q| [q[0], q[1]]).collect();
                    let ccw = ring_area2(&r) > 0.0;
                    if (i == 0) != ccw {
                        r.reverse();
                    }
                    r
                })
                .collect::<Vec<_>>()
        })
        .collect();
    if areas.is_empty() {
        return Err("Pick the faces to fill".into());
    }
    let area = region::normalize(&areas);
    let zig = p.cutting_method == CuttingMethod::Zigzag;
    let lines = hatch(
        &region::offset(&area, -input.tool.radius()),
        p.angle,
        p.step_over,
        zig,
    );
    let z = heights(input);
    let power = (p.beam.power / 100.0).clamp(0.0, 1.0);
    let mut b = Builder::new(input);
    for l in lines {
        let (a, e) = (l[0], l[l.len() - 1]);
        // Each line on its own: beam on at its start, off at its end.
        b.rapid([a[0], a[1], z.cut]);
        b.steps.push(Step::BeamOn { power });
        b.feed([e[0], e[1], z.cut], FeedKind::Cut);
        b.steps.push(Step::BeamOff);
    }
    let mut tp = b.finish();
    tp.steps.retain(|s| !matches!(s, Step::Spindle { .. }));
    Ok(tp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Aabb;
    use crate::model::{Face, Model, Surface, box_triangles};
    use crate::strategy::{Params, Pattern, Stock, generate};
    use crate::tool::{Tool, ToolKind};

    /// A 100 × 60 × 3 plate with a 20 × 20 square hole (triangles made from three boxes and
    /// two strips round the hole).
    fn plate() -> Model {
        let mut t = Vec::new();
        t.extend(box_triangles(&Aabb::new(
            [0.0, 0.0, 0.0],
            [40.0, 60.0, 3.0],
        )));
        t.extend(box_triangles(&Aabb::new(
            [60.0, 0.0, 0.0],
            [100.0, 60.0, 3.0],
        )));
        t.extend(box_triangles(&Aabb::new(
            [40.0, 0.0, 0.0],
            [60.0, 20.0, 3.0],
        )));
        t.extend(box_triangles(&Aabb::new(
            [40.0, 40.0, 0.0],
            [60.0, 60.0, 3.0],
        )));
        let mut m = Model::from_triangles("plate", t);
        m.faces.push(Face {
            name: "top".into(),
            body: 0,
            surface: Surface::Plane {
                normal: [0.0, 0.0, 1.0],
            },
            loops: vec![vec![
                [10.0, 10.0, 3.0],
                [30.0, 10.0, 3.0],
                [30.0, 30.0, 3.0],
                [10.0, 30.0, 3.0],
            ]],
            triangles: vec![],
        });
        m
    }

    fn run(
        pattern: Pattern,
        machine: MachineKind,
        tool: Tool,
        f: impl FnOnce(&mut Params),
    ) -> Result<Toolpath, String> {
        let m = plate();
        let stock = Stock::block(Aabb::new([-5.0, -5.0, 0.0], [105.0, 65.0, 3.0]));
        let mut p = Params::new(pattern, &tool, machine);
        f(&mut p);
        let input = Input {
            model: &m,
            stock: &stock,
            tool: &tool,
            machine,
            params: &p,
            rest: None,
        };
        generate(&input)
    }

    #[test]
    fn plasma_cuts_the_hole_first_with_pierces() {
        let tp = run(
            Pattern::Cut,
            MachineKind::Plasma,
            Tool::new(ToolKind::Plasma, 1.5),
            |_| {},
        )
        .unwrap();
        let ons: Vec<usize> = tp
            .steps
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s, Step::BeamOn { .. }))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(ons.len(), 2);
        // The first cut is the hole, counter-clockwise (the part on the right), pierced in the
        // scrap: its feeds stay inside 40..60.
        let first_cut: Vec<P3> = tp.steps[ons[0]..ons[1]]
            .iter()
            .filter_map(|s| match s {
                Step::Feed {
                    to,
                    kind: FeedKind::Cut,
                    ..
                } => Some(*to),
                _ => None,
            })
            .collect();
        assert!(
            first_cut.iter().all(|p| p[0] > 40.0 && p[0] < 60.0),
            "{first_cut:?}"
        );
        // Kerf: 0.75 inside the hole.
        assert!(first_cut.iter().any(|p| (p[0] - 40.75).abs() < 1e-3));
        // A pierce dwell and the torch at its pierce height.
        assert!(
            tp.steps
                .iter()
                .any(|s| matches!(s, Step::Dwell(d) if (*d - 0.5).abs() < 1e-9))
        );
        assert!(
            tp.steps
                .iter()
                .any(|s| matches!(s, Step::Rapid(p) if (p[2] - 6.8).abs() < 1e-9))
        );
    }

    /// A plate with a round hole (the hole's triangles a fan, as a kernel tessellates it): the
    /// hole is cut before the outline whatever the outline's distance.
    #[test]
    fn round_hole_first() {
        let mut t = Vec::new();
        let n = 48;
        let ring = |r: f64, z: f64| -> Vec<P3> {
            (0..n)
                .map(|i| {
                    let a = std::f64::consts::TAU * i as f64 / n as f64;
                    [30.0 + r * a.cos(), 20.0 + r * a.sin(), z]
                })
                .collect()
        };
        // The plate's top as triangles between the outline (a square) and the hole.
        let outer = |i: usize| -> P3 {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            let (c, s) = (a.cos(), a.sin());
            let k = 1.0 / c.abs().max(s.abs());
            [30.0 + 30.0 * c * k, 20.0 + 20.0 * s * k, 3.0]
        };
        let hole = ring(8.0, 3.0);
        for i in 0..n {
            let j = (i + 1) % n;
            t.push([hole[i], outer(i), outer(j)]);
            t.push([hole[i], outer(j), hole[j]]);
        }
        let m = Model::from_triangles("plate", t);
        let stock = Stock::block(Aabb::new([-5.0, -5.0, 0.0], [65.0, 45.0, 3.0]));
        let tool = Tool::new(ToolKind::Plasma, 1.5);
        let p = Params::new(Pattern::Cut, &tool, MachineKind::Plasma);
        let input = Input {
            model: &m,
            stock: &stock,
            tool: &tool,
            machine: MachineKind::Plasma,
            params: &p,
            rest: None,
        };
        let tp = generate(&input).unwrap();
        let first = tp
            .steps
            .iter()
            .find_map(|s| match s {
                Step::Feed {
                    to,
                    kind: FeedKind::Cut,
                    ..
                } => Some(*to),
                _ => None,
            })
            .unwrap();
        // On the hole: 8 − 0.75 from its centre.
        let r = (first[0] - 30.0).hypot(first[1] - 20.0);
        assert!((r - 7.25).abs() < 0.05, "{first:?} r={r}");
    }

    #[test]
    fn part_on_the_right() {
        let tp = run(
            Pattern::Cut,
            MachineKind::Laser,
            Tool::new(ToolKind::Laser, 0.2),
            |p| {
                p.links.lead_in = LeadKind::None;
                p.links.overlap = 0.0;
            },
        )
        .unwrap();
        // The outline (second cut) runs clockwise.
        let ons: Vec<usize> = tp
            .steps
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s, Step::BeamOn { .. }))
            .map(|(i, _)| i)
            .collect();
        let outline: Vec<P2> = tp.steps[ons[1]..]
            .iter()
            .filter_map(|s| match s {
                Step::Feed {
                    to,
                    kind: FeedKind::Cut,
                    ..
                } => Some([to[0], to[1]]),
                _ => None,
            })
            .collect();
        assert!(ring_area2(&outline) < 0.0);
    }

    #[test]
    fn microjoints_break_the_outline() {
        let tp = run(
            Pattern::Cut,
            MachineKind::Laser,
            Tool::new(ToolKind::Laser, 0.2),
            |p| {
                p.beam.microjoints = 4;
            },
        )
        .unwrap();
        let ons = tp
            .steps
            .iter()
            .filter(|s| matches!(s, Step::BeamOn { .. }))
            .count();
        // The hole in one go, the outline in five pieces.
        assert_eq!(ons, 1 + 5);
    }

    #[test]
    fn fill_hatches_the_face() {
        let tp = run(
            Pattern::Fill,
            MachineKind::Laser,
            Tool::new(ToolKind::Laser, 0.2),
            |p| {
                p.drive.faces = vec!["top".into()];
                p.step_over = 1.0;
            },
        )
        .unwrap();
        let lines = tp
            .steps
            .iter()
            .filter(|s| matches!(s, Step::BeamOn { .. }))
            .count();
        assert_eq!(lines, 20);
    }

    #[test]
    fn mill_tools_cant_cut() {
        assert!(
            run(
                Pattern::Cut,
                MachineKind::Mill,
                Tool::new(ToolKind::EndMill, 6.0),
                |_| {}
            )
            .is_err()
        );
    }

    #[test]
    fn microjoint_split() {
        let walk = vec![
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 10.0],
            [0.0, 10.0],
            [0.0, 0.0],
        ];
        let pieces = microjoint_pieces(&walk, 4, 1.0);
        let cut: f64 = pieces
            .iter()
            .map(|p| p.windows(2).map(|w| dist2(w[0], w[1])).sum::<f64>())
            .sum();
        assert!((cut - 36.0).abs() < 1e-6, "{cut} {pieces:?}");
    }
}
