//! Hole making: Drill (drilling, peck drilling, chip breaking and tapping cycles) and Bore
//! (helical milling of holes larger than the end mill).
//!
//! Holes are the model's vertical cylindrical walls the material surrounds
//! ([`crate::model::Model::holes`]), with their exact diameters and depths from the kernel.

use crate::geom::{P2, dist2};
use crate::model::Hole;
use crate::tool::ToolKind;
use crate::toolpath::{CycleKind, DrillCycle, FeedKind, Step, Toolpath};

use super::link::Builder;
use super::{HoleSelect, Input};

/// The holes a toolpath takes, nearest first from the model's corner.
fn pick_holes(input: &Input, bore: bool) -> (Vec<Hole>, Vec<String>) {
    let p = input.params;
    let t = input.tool;
    let all = input.model.holes();
    let mut warnings = Vec::new();
    let near_picked = |h: &Hole| {
        p.drive
            .holes
            .iter()
            .any(|c| dist2(*c, h.center) < h.diameter / 2.0 + 1e-3)
    };
    // The diameter a drill or tap is for.
    let fits = |h: &Hole| -> bool {
        if bore {
            h.diameter > t.diameter + 2.0 * p.wall_stock + 1e-3
        } else if t.kind == ToolKind::Tap {
            // A tapped hole is modelled at the tap's size or its tap-drill size.
            (h.diameter - t.diameter).abs() < 0.05
                || (h.diameter - (t.diameter - t.pitch)).abs() < 0.15
        } else {
            (h.diameter - t.diameter).abs() < 0.05
        }
    };
    let picked_d: Vec<f64> = all
        .iter()
        .filter(|h| near_picked(h))
        .map(|h| h.diameter)
        .collect();
    let mut holes: Vec<Hole> = all
        .iter()
        .filter(|h| match p.cycle.select {
            HoleSelect::MatchingTool => fits(h),
            HoleSelect::All => true,
            HoleSelect::Blind => !h.through,
            HoleSelect::SameDiameter => picked_d.iter().any(|d| (d - h.diameter).abs() < 1e-3),
        })
        .filter(|h| {
            p.drive.holes.is_empty() || p.cycle.select == HoleSelect::SameDiameter || near_picked(h)
        })
        .copied()
        .collect();
    if !bore {
        let small: Vec<&Hole> = holes
            .iter()
            .filter(|h| h.diameter < t.diameter - 1e-3)
            .collect();
        if !small.is_empty() {
            warnings.push(format!(
                "{} hole(s) are smaller than the tool and were left out",
                small.len()
            ));
        }
        holes.retain(|h| h.diameter >= t.diameter - 1e-3);
    }
    // Nearest neighbour from the model's lower-left corner.
    let b = input.model.bounds();
    let mut from: P2 = [b.min[0], b.min[1]];
    let mut ordered = Vec::new();
    while !holes.is_empty() {
        let i = holes
            .iter()
            .enumerate()
            .min_by(|a, b| dist2(a.1.center, from).total_cmp(&dist2(b.1.center, from)))
            .map(|(i, _)| i)
            .unwrap();
        let h = holes.swap_remove(i);
        from = h.center;
        ordered.push(h);
    }
    (ordered, warnings)
}

/// Whether the straight move between two holes at height `z` clears the model.
fn clear_between(input: &Input, a: P2, b: P2, z: f64) -> bool {
    let n = ((dist2(a, b) / 1.0).ceil() as usize).clamp(2, 400);
    let r = input.tool.radius();
    (0..=n).all(|i| {
        let t = i as f64 / n as f64;
        let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        [[0.0, 0.0], [r, 0.0], [-r, 0.0], [0.0, r], [0.0, -r]]
            .iter()
            .all(|o| {
                input
                    .model
                    .top_at([p[0] + o[0], p[1] + o[1]])
                    .is_none_or(|top| top < z - 0.5)
            })
    })
}

/// Drill: a cycle at each hole.
pub fn drill(input: &Input) -> Result<Toolpath, String> {
    let p = input.params;
    let t = input.tool;
    let (holes, warnings) = pick_holes(input, false);
    if holes.is_empty() {
        return Err(if input.model.holes().is_empty() {
            "The model has no holes".into()
        } else {
            format!(
                "No hole fits the {} (Ø{})",
                t.name,
                crate::tool::fmt_mm(t.diameter)
            )
        });
    }
    let mut b = Builder::new(input);
    b.warnings.extend(warnings);
    let clearance = input.clearance();
    let kind = if t.kind == ToolKind::Tap {
        CycleKind::Tap
    } else if p.cycle.kind == CycleKind::Tap {
        CycleKind::Drill
    } else {
        p.cycle.kind
    };
    let tip = if t.kind == ToolKind::Drill {
        t.radius() / (t.angle.clamp(60.0, 179.0) / 2.0).to_radians().tan()
    } else {
        0.0
    };
    let mut prev: Option<(P2, f64)> = None;
    for h in holes {
        let r_plane = h.top + p.links.rapid_distance;
        let bottom = if h.through {
            h.bottom - p.cycle.break_through - tip
        } else {
            h.bottom + p.floor_stock
        };
        let retract = if p.cycle.retract_full {
            clearance
        } else {
            r_plane
        };
        // Up to clearance first when the way at the R plane isn't clear.
        match prev {
            None => b.rapid([h.center[0], h.center[1], clearance]),
            Some((c, z)) => {
                let travel = if clear_between(input, c, h.center, z) {
                    z
                } else {
                    clearance
                };
                b.up_to(travel);
                b.rapid([h.center[0], h.center[1], travel.max(r_plane)]);
            }
        }
        let feed = if kind == CycleKind::Tap {
            t.pitch * p.speeds.spindle_rpm
        } else {
            p.speeds.feed
        };
        b.steps.push(Step::Drill(DrillCycle {
            at: h.center,
            r_plane,
            bottom,
            retract,
            feed,
            kind,
            peck: p.cycle.peck,
            dwell: p.cycle.dwell,
            pitch: t.pitch,
        }));
        b.at = Some([h.center[0], h.center[1], retract]);
        prev = Some((h.center, retract));
    }
    Ok(b.finish())
}

/// Bore: a helix down each hole bigger than the end mill, a full circle at the bottom, out
/// through the middle.
pub fn bore(input: &Input) -> Result<Toolpath, String> {
    let p = input.params;
    let (holes, warnings) = pick_holes(input, true);
    if holes.is_empty() {
        return Err("No hole is bigger than the tool".into());
    }
    let mut b = Builder::new(input);
    b.warnings.extend(warnings);
    let clearance = input.clearance();
    // Climb: the wall (material outside the circle) on the right, so counter-clockwise.
    let cw = p.direction != super::Direction::Climb;
    for h in holes {
        let r = h.diameter / 2.0 - input.tool.radius() - p.wall_stock;
        if r <= 1e-3 {
            continue;
        }
        let c = h.center;
        let top = h.top + p.links.feed_distance;
        let bottom = if h.through {
            h.bottom - p.cycle.break_through
        } else {
            h.bottom + p.floor_stock
        };
        let start = [c[0] + r, c[1]];
        b.up_to(clearance);
        b.rapid([c[0], c[1], clearance]);
        b.rapid([c[0], c[1], top]);
        b.feed([start[0], start[1], top], FeedKind::LeadIn);
        // The helix, a turn per step down.
        let pitch = p.step_down.max(0.05);
        let mut z = top;
        while z > bottom + 1e-9 {
            let next = (z - pitch).max(bottom);
            b.arc([start[0], start[1], next], c, cw, FeedKind::Ramp);
            z = next;
        }
        // A level circle at the bottom, then off the wall and out.
        b.arc([start[0], start[1], bottom], c, cw, FeedKind::Cut);
        b.feed([c[0], c[1], bottom], FeedKind::LeadOut);
        b.rapid([c[0], c[1], clearance]);
    }
    if !b.moved() {
        return Err("No hole is bigger than the tool".into());
    }
    Ok(b.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::{Aabb, P3};
    use crate::machine::MachineKind;
    use crate::model::{Face, Model, Surface, box_triangles};
    use crate::strategy::{Params, Pattern, Stock, generate};
    use crate::tool::Tool;

    fn plate() -> Model {
        let mut m = Model::from_triangles(
            "plate",
            box_triangles(&Aabb::new([0.0, 0.0, 0.0], [80.0, 40.0, 10.0])),
        );
        let ring = |cx: f64, cy: f64, r: f64, z: f64| -> Vec<P3> {
            (0..24)
                .map(|i| {
                    let a = std::f64::consts::TAU * i as f64 / 24.0;
                    [cx + r * a.cos(), cy + r * a.sin(), z]
                })
                .collect()
        };
        for (cx, d, bottom) in [(10.0, 6.0, 0.0), (40.0, 6.0, 4.0), (70.0, 20.0, 0.0)] {
            m.faces.push(Face {
                name: format!("hole{cx}"),
                body: 0,
                surface: Surface::Cylinder {
                    point: [cx, 20.0, 0.0],
                    axis: [0.0, 0.0, 1.0],
                    radius: d / 2.0,
                    inner: true,
                },
                loops: vec![
                    ring(cx, 20.0, d / 2.0, 10.0),
                    ring(cx, 20.0, d / 2.0, bottom),
                ],
                triangles: vec![],
            });
        }
        m
    }

    fn run(pattern: Pattern, tool: Tool) -> Result<Toolpath, String> {
        let m = plate();
        let stock = Stock::block(Aabb::new([0.0, 0.0, 0.0], [80.0, 40.0, 10.0]));
        let p = Params::new(pattern, &tool, MachineKind::Mill);
        let input = Input {
            model: &m,
            stock: &stock,
            tool: &tool,
            machine: MachineKind::Mill,
            params: &p,
            rest: None,
        };
        generate(&input)
    }

    #[test]
    fn drills_matching_holes_through_and_blind() {
        let tp = run(Pattern::Drill, Tool::new(ToolKind::Drill, 6.0)).unwrap();
        let cycles: Vec<DrillCycle> = tp
            .steps
            .iter()
            .filter_map(|s| match s {
                Step::Drill(c) => Some(*c),
                _ => None,
            })
            .collect();
        assert_eq!(cycles.len(), 2);
        // Through: past the bottom by the break-through and the tip.
        assert!(cycles[0].bottom < -1.0);
        // Blind: to its bottom.
        assert!((cycles[1].bottom - 4.0).abs() < 1e-9);
        assert!((cycles[0].r_plane - 13.0).abs() < 1e-9);
    }

    #[test]
    fn no_fitting_hole_is_an_error() {
        assert!(run(Pattern::Drill, Tool::new(ToolKind::Drill, 3.0)).is_err());
    }

    #[test]
    fn bore_helixes_the_big_hole() {
        let tp = run(Pattern::Bore, Tool::new(ToolKind::EndMill, 10.0)).unwrap();
        let arcs: Vec<_> = tp
            .steps
            .iter()
            .filter_map(|s| match s {
                Step::Arc { to, center, cw, .. } => Some((*to, *center, *cw)),
                _ => None,
            })
            .collect();
        assert!(arcs.len() > 3);
        for (to, c, cw) in &arcs {
            assert_eq!(*c, [70.0, 20.0]);
            assert!((to[0] - 75.0).abs() < 1e-9);
            assert!(!cw);
        }
        // Down through the bottom.
        assert!(arcs.last().unwrap().0[2] < 0.0);
    }
}
