//! The Collision tab: the tool assembly above the flutes (the shank, the holder, the arbor
//! stacked on the holder) checked against the part along every cutting move, as Simulate
//! checks it against the stock. Each piece is a disc at a height above the tip: the shank the
//! cutter's radius from the end of the flutes, the holder's and arbor's profile segments their
//! larger radius from where they start, each grown by its clearance. A move whose discs would
//! meet the part is reported (Warn) or left out (Retract: the tool lifts to the clearance height
//! before it and comes back in over where the cut goes on, ramping by the Link tab's entry).
//!
//! The workholding (vises, clamps: the job's workholding bodies) is checked whatever the tab
//! says, against the whole tool, its cutter too: nothing may cut or touch it.

use serde::{Deserialize, Serialize};

use crate::dropcutter::{TriIndex, drop_point};
use crate::geom::{P3, dist, lerp};
use crate::tool::{Tool, ToolKind};
use crate::toolpath::{FeedKind, Step, Toolpath, arc_points};

use super::Input;

/// What to do where the tool assembly would hit the part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CollisionAction {
    /// Keep the moves; list them in the toolpath's warnings.
    Warn,
    /// Leave the moves out, lifting clear round them.
    #[default]
    Retract,
}

impl CollisionAction {
    pub const ALL: [CollisionAction; 2] = [CollisionAction::Retract, CollisionAction::Warn];

    pub fn label(self) -> &'static str {
        match self {
            CollisionAction::Warn => "Warn",
            CollisionAction::Retract => "Retract",
        }
    }
}

/// The Collision tab.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Collision {
    /// Check the tool assembly at all.
    pub check: bool,
    /// How far the shank must stay from the part (mm).
    pub shank_clearance: f64,
    /// How far the holder and arbor must stay from the part (mm).
    pub holder_clearance: f64,
    pub action: CollisionAction,
}

impl Default for Collision {
    fn default() -> Self {
        Collision {
            check: true,
            shank_clearance: 0.0,
            holder_clearance: 1.0,
            action: CollisionAction::Retract,
        }
    }
}

/// The tool assembly above the flutes as discs: (radius, height above the tip it starts at).
pub fn assembly_discs(tool: &Tool, c: &Collision) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    if tool.cutting_length < tool.overall_length {
        out.push((tool.radius() + c.shank_clearance, tool.cutting_length));
    }
    let mut base = tool.overall_length;
    for profile in [&tool.holder, &tool.arbor].into_iter().flatten() {
        for w in profile.points.windows(2) {
            let r = w[0][0].max(w[1][0]);
            let h = w[0][1].min(w[1][1]);
            if r > 0.0 {
                out.push((r + c.holder_clearance, base + h));
            }
        }
        base += profile.length();
    }
    out
}

/// What a move would hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Hit {
    /// The shank, holder or arbor the part.
    Part,
    /// Any of the tool (its cutter too) the workholding.
    Fixture,
}

/// Checks a toolpath's cutting moves against the part (the tool assembly above the flutes, when
/// the tab's check is on) and the workholding (the whole tool and its holder, always). With
/// Retract the moves that would hit are left out, the tool lifting clear round them and coming
/// back in by the Link tab's entry; either way a warning counts them and says where the first is.
pub fn apply(input: &Input, tp: Toolpath) -> Toolpath {
    let c = input.params.collision;
    if input.tool.is_beam() {
        return tp;
    }
    let part_check = c.check && !input.model.triangles.is_empty();
    if !part_check && input.fixtures.is_empty() {
        return tp;
    }
    let discs = assembly_discs(input.tool, &c);
    let lowest = discs.iter().map(|d| d.1).fold(f64::INFINITY, f64::min);
    let top = input.model.bounds().max[2];
    let idx = TriIndex::new(&input.model.triangles);
    let probes: Vec<(Tool, f64)> = discs
        .iter()
        .map(|(r, h)| (Tool::new(ToolKind::EndMill, 2.0 * (r - 1e-3).max(1e-3)), *h))
        .collect();
    // The workholding: the cutter itself (a hair smaller, so running along it isn't a hit) and
    // the holder's discs.
    let fixtures = (!input.fixtures.is_empty()).then(|| TriIndex::new(input.fixtures));
    let fixture_top = input
        .fixtures
        .iter()
        .flatten()
        .map(|p| p[2])
        .fold(f64::NEG_INFINITY, f64::max);
    let mut cutter = input.tool.clone();
    cutter.diameter = (cutter.diameter - 0.002).max(1e-3);
    if cutter.kind == ToolKind::BallMill {
        cutter.corner_radius = cutter.diameter / 2.0;
    }
    let step = (input.tool.radius() / 2.0).clamp(0.05, 1.0);
    let hits_at = |q: P3, scratch: &mut Vec<u32>| -> Option<Hit> {
        if let Some(f) = &fixtures
            && q[2] < fixture_top
        {
            let low = drop_point(&cutter, f, [q[0], q[1]], f64::NEG_INFINITY, scratch);
            let holder = probes
                .iter()
                .filter(|(_, h)| *h >= input.tool.overall_length - 1e-9);
            if low > q[2] + 1e-3
                || holder.into_iter().any(|(t, h)| {
                    q[2] + h < fixture_top
                        && drop_point(t, f, [q[0], q[1]], f64::NEG_INFINITY, scratch)
                            > q[2] + h + 1e-6
                })
            {
                return Some(Hit::Fixture);
            }
        }
        // Nothing of the part reaches the lowest disc.
        if !part_check || q[2] + lowest >= top {
            return None;
        }
        probes
            .iter()
            .any(|(t, h)| {
                q[2] + h < top
                    && drop_point(t, &idx, [q[0], q[1]], f64::NEG_INFINITY, scratch)
                        > q[2] + h + 1e-6
            })
            .then_some(Hit::Part)
    };
    let mut scratch = Vec::new();
    let mut hits_move = |from: P3, s: &Step| -> Option<Hit> {
        let pts: Vec<P3> = match s {
            Step::Feed { to, .. } => {
                let n = (dist(from, *to) / step).ceil().max(1.0) as usize;
                (0..=n)
                    .map(|i| lerp(from, *to, i as f64 / n as f64))
                    .collect()
            }
            Step::Arc { to, center, cw, .. } => {
                let mut v = vec![from];
                v.extend(arc_points(from, *to, *center, *cw, step.min(0.05)));
                v
            }
            _ => return None,
        };
        let mut worst = None;
        for q in pts {
            match hits_at(q, &mut scratch) {
                Some(Hit::Fixture) => return Some(Hit::Fixture),
                Some(h) => worst = Some(h),
                None => {}
            }
        }
        worst
    };
    let p = input.params;
    let clearance = input.clearance();
    let fd = p.links.feed_distance;
    let retract = c.action == CollisionAction::Retract;
    let mut out: Vec<Step> = Vec::with_capacity(tp.steps.len());
    let mut at: Option<P3> = None;
    let mut skipping = false;
    let mut counts = [0usize; 2];
    let mut first: [Option<P3>; 2] = [None; 2];
    for s in tp.steps {
        match &s {
            Step::Feed { to, .. } | Step::Arc { to, .. } => {
                let to = *to;
                if let Some(from) = at
                    && let Some(hit) = hits_move(from, &s)
                {
                    let k = (hit == Hit::Fixture) as usize;
                    counts[k] += 1;
                    first[k].get_or_insert(from);
                    if retract {
                        if !skipping {
                            out.push(Step::Rapid([from[0], from[1], clearance.max(from[2])]));
                            skipping = true;
                        }
                        at = Some(to);
                        continue;
                    }
                }
                if skipping && let Some(from) = at {
                    reenter(&mut out, input, from, &s, clearance, fd);
                    skipping = false;
                }
                at = Some(to);
                out.push(s);
            }
            Step::Rapid(q) => {
                if skipping {
                    if q[2] < clearance {
                        out.push(Step::Rapid([q[0], q[1], clearance]));
                    }
                    skipping = false;
                }
                at = Some(*q);
                out.push(s);
            }
            Step::Drill(d) => {
                at = Some([d.at[0], d.at[1], d.retract]);
                out.push(s);
            }
            _ => out.push(s),
        }
    }
    let mut warnings = tp.warnings;
    let what = [
        if input.tool.holder.is_some() {
            "the shank or holder into the part"
        } else {
            "the shank into the part"
        },
        "the tool or its holder into the workholding",
    ];
    for k in 0..2 {
        if let Some(q) = first[k] {
            let n = counts[k];
            warnings.push(format!(
                "{n} move{} would bring {} (first near {:.1}, {:.1}, {:.1}){}",
                if n == 1 { "" } else { "s" },
                what[k],
                q[0],
                q[1],
                q[2],
                if retract {
                    ": left out, the tool lifting clear round them"
                } else {
                    ""
                }
            ));
        }
    }
    Toolpath {
        steps: out,
        warnings,
    }
}

/// Back over `from` (where the cut goes on with `next`) from the clearance height and in: by a
/// ramp along `next`'s way when the Link tab ramps (the material there may be a step down
/// thick), else a plunge at the plunge feed.
fn reenter(out: &mut Vec<Step>, input: &Input, from: P3, next: &Step, clearance: f64, fd: f64) {
    let p = input.params;
    out.push(Step::Rapid([from[0], from[1], clearance.max(from[2])]));
    let way = match next {
        Step::Feed { to, .. } | Step::Arc { to, .. } => {
            let d = [to[0] - from[0], to[1] - from[1]];
            let l = d[0].hypot(d[1]);
            (l > 1e-6).then(|| ([d[0] / l, d[1] / l], l))
        }
        _ => None,
    };
    let ramp = p.links.entry == super::Entry::Ramp && way.is_some();
    // The material over it: as much as a step down (a level above cut down to it).
    let top = (from[2] + p.step_down.max(0.0)).min(clearance - fd);
    let start = top + fd;
    if start < clearance {
        out.push(Step::Rapid([from[0], from[1], start]));
    }
    if let (true, Some((u, l))) = (ramp, way) {
        // Back and forth along the next move's first stretch, down at the ramp angle.
        let run = l.min(input.tool.diameter * 2.0).max(1e-3);
        let drop = run * p.links.ramp_angle.clamp(0.5, 45.0).to_radians().tan();
        let mut z = start;
        let mut out_leg = true;
        while z > from[2] + 1e-9 {
            z = (z - drop).max(from[2]);
            let q = if out_leg {
                [from[0] + u[0] * run, from[1] + u[1] * run, z]
            } else {
                [from[0], from[1], z]
            };
            out.push(Step::Feed {
                to: q,
                feed: p.speeds.ramp,
                kind: FeedKind::Ramp,
            });
            out_leg = !out_leg;
        }
        // Back to where the cut goes on.
        if !out_leg {
            out.push(Step::Feed {
                to: from,
                feed: p.speeds.feed,
                kind: FeedKind::Cut,
            });
        }
    } else {
        out.push(Step::Feed {
            to: from,
            feed: p.speeds.plunge,
            kind: FeedKind::Plunge,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Aabb;
    use crate::machine::MachineKind;
    use crate::model::{Model, box_triangles};
    use crate::strategy::{Params, Pattern, Stock, generate};
    use crate::tool::Profile;

    /// A 40 mm tall wall beside a 10 mm deep slot: the profile round the 30 mm deep part runs
    /// the tool's 20 mm flutes 30 deep beside the wall.
    fn run(action: CollisionAction, holder: bool) -> Toolpath {
        let m = Model::from_triangles(
            "block",
            box_triangles(&Aabb::new([0.0, 0.0, 0.0], [40.0, 30.0, 30.0])),
        );
        let stock = Stock::block(Aabb::new([-10.0, -10.0, 0.0], [50.0, 40.0, 30.0]));
        let mut tool = Tool::new(ToolKind::EndMill, 6.0);
        tool.cutting_length = 20.0;
        tool.overall_length = 25.0;
        if holder {
            tool.holder = Some(Profile::cylinder("ER20", 30.0, 40.0));
        }
        let mut p = Params::new(Pattern::Profile2D, &tool, MachineKind::Mill);
        p.step_down = 10.0;
        p.collision.action = action;
        let input = Input {
            model: &m,
            stock: &stock,
            tool: &tool,
            machine: MachineKind::Mill,
            params: &p,
            rest: None,
            fixtures: &[],
        };
        generate(&input).unwrap()
    }

    fn lowest_cut(tp: &Toolpath) -> f64 {
        tp.steps
            .iter()
            .filter_map(|s| match s {
                Step::Feed {
                    to,
                    kind: FeedKind::Cut,
                    ..
                } => Some(to[2]),
                _ => None,
            })
            .fold(f64::INFINITY, f64::min)
    }

    #[test]
    fn shank_beside_its_wall_is_fine_holder_is_not() {
        // The shank runs beside the wall, never into it: nothing to report.
        let tp = run(CollisionAction::Retract, false);
        assert!(tp.warnings.is_empty(), "{:?}", tp.warnings);
        assert!(lowest_cut(&tp) < 0.01);
        // A Ø30 holder 25 above the tip reaches over the part below 5 mm.
        let warned = run(CollisionAction::Warn, true);
        assert_eq!(warned.warnings.len(), 1, "{:?}", warned.warnings);
        assert!(lowest_cut(&warned) < 0.01);
        let kept = run(CollisionAction::Retract, true);
        assert!(kept.warnings[0].contains("left out"));
        // The cuts the holder would hit are gone: none deeper than 30 − 25 − 1 clearance.
        assert!(lowest_cut(&kept) > 3.9, "{}", lowest_cut(&kept));
    }

    #[test]
    fn discs_stack_the_holder_and_arbor() {
        let mut t = Tool::new(ToolKind::EndMill, 10.0);
        t.cutting_length = 22.0;
        t.overall_length = 40.0;
        t.holder = Some(Profile::cylinder("h", 32.0, 50.0));
        t.arbor = Some(Profile::cylinder("a", 63.0, 20.0));
        let d = assembly_discs(&t, &Collision::default());
        assert_eq!(d, vec![(5.0, 22.0), (17.0, 40.0), (32.5, 90.0)]);
    }

    /// A vise jaw 10 high, 2 mm off the block's +X side: the profile's deepest pass would cut
    /// it. Retract leaves those moves out (lifting clear, ramping back in); nothing cuts the jaw.
    #[test]
    fn the_workholding_is_never_cut() {
        let m = Model::from_triangles(
            "block",
            box_triangles(&Aabb::new([0.0, 0.0, 0.0], [40.0, 30.0, 30.0])),
        );
        let jaw = box_triangles(&Aabb::new([42.0, -10.0, 0.0], [60.0, 40.0, 10.0]));
        let stock = Stock::block(Aabb::new([-5.0, -5.0, 0.0], [45.0, 35.0, 30.0]));
        let tool = Tool::new(ToolKind::EndMill, 6.0);
        for action in [CollisionAction::Retract, CollisionAction::Warn] {
            let mut p = Params::new(Pattern::Profile2D, &tool, MachineKind::Mill);
            p.step_down = 10.0;
            p.collision.action = action;
            let input = Input {
                model: &m,
                stock: &stock,
                tool: &tool,
                machine: MachineKind::Mill,
                params: &p,
                rest: None,
                fixtures: &jaw,
            };
            let tp = generate(&input).unwrap();
            assert!(
                tp.warnings
                    .iter()
                    .any(|w| w.contains("into the workholding")),
                "{:?}",
                tp.warnings
            );
            let jaw_model = Model::from_triangles("jaw", jaw.clone());
            let (g, at) = crate::strategy::testing::max_gouge(&jaw_model, &tool, &tp);
            if action == CollisionAction::Retract {
                assert!(g < 1e-3, "{g} into the jaw at {at:?}");
                // The cut goes on elsewhere at the bottom, re-entered by a ramp.
                assert!(lowest_cut(&tp) < 0.01);
                assert!(tp.steps.iter().any(|s| matches!(
                    s,
                    Step::Feed {
                        kind: FeedKind::Ramp,
                        ..
                    }
                )));
                // The clearance height is over the jaw (and everything else).
                assert!(input.clearance() >= 40.0);
            } else {
                assert!(g > 1.0, "Warn keeps the moves");
            }
        }
    }
}
