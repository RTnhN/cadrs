//! The Collision tab: the tool assembly above the flutes (the shank, the holder, the arbor
//! stacked on the holder) checked against the part along every cutting move, as Simulate
//! checks it against the stock. Each piece is a disc at a height above the tip: the shank the
//! cutter's radius from the end of the flutes, the holder's and arbor's profile segments their
//! larger radius from where they start, each grown by its clearance. A move whose discs would
//! meet the part is reported (Warn) or left out (Retract: the tool lifts to the clearance height
//! before it and comes back down over where the cut goes on).

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

/// Checks a toolpath's cutting moves: with Retract the ones that would hit are left out; either
/// way a warning counts them and says where the first is.
pub fn apply(input: &Input, tp: Toolpath) -> Toolpath {
    let c = input.params.collision;
    if !c.check || input.tool.is_beam() || input.model.triangles.is_empty() {
        return tp;
    }
    let discs = assembly_discs(input.tool, &c);
    let Some(lowest) = discs.iter().map(|d| d.1).reduce(f64::min) else {
        return tp;
    };
    let top = input.model.bounds().max[2];
    let idx = TriIndex::new(&input.model.triangles);
    let probes: Vec<(Tool, f64)> = discs
        .iter()
        .map(|(r, h)| (Tool::new(ToolKind::EndMill, 2.0 * (r - 1e-3).max(1e-3)), *h))
        .collect();
    let step = (input.tool.radius() / 2.0).clamp(0.05, 1.0);
    let hits_at = |q: P3, scratch: &mut Vec<u32>| -> bool {
        // Nothing of the part reaches the lowest disc.
        if q[2] + lowest >= top {
            return false;
        }
        probes.iter().any(|(t, h)| {
            q[2] + h < top
                && drop_point(t, &idx, [q[0], q[1]], f64::NEG_INFINITY, scratch) > q[2] + h + 1e-6
        })
    };
    let mut scratch = Vec::new();
    let mut hits_move = |from: P3, s: &Step| -> bool {
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
            _ => return false,
        };
        pts.into_iter().any(|q| hits_at(q, &mut scratch))
    };
    let clearance = input.clearance();
    let fd = input.params.links.feed_distance;
    let plunge = input.params.speeds.plunge;
    let retract = c.action == CollisionAction::Retract;
    let mut out: Vec<Step> = Vec::with_capacity(tp.steps.len());
    let mut at: Option<P3> = None;
    let mut skipping = false;
    let mut hits = 0usize;
    let mut first: Option<P3> = None;
    for s in tp.steps {
        match &s {
            Step::Feed { to, .. } | Step::Arc { to, .. } => {
                let to = *to;
                if let Some(from) = at
                    && hits_move(from, &s)
                {
                    hits += 1;
                    first.get_or_insert(from);
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
                    // Back over where the cut goes on, and down to it.
                    out.push(Step::Rapid([from[0], from[1], clearance.max(from[2])]));
                    if from[2] + fd < clearance {
                        out.push(Step::Rapid([from[0], from[1], from[2] + fd]));
                    }
                    out.push(Step::Feed {
                        to: from,
                        feed: plunge,
                        kind: FeedKind::Plunge,
                    });
                    skipping = false;
                }
                at = Some(to);
                out.push(s);
            }
            Step::Rapid(p) => {
                if skipping {
                    if p[2] < clearance {
                        out.push(Step::Rapid([p[0], p[1], clearance]));
                    }
                    skipping = false;
                }
                at = Some(*p);
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
    if let Some(p) = first {
        warnings.push(format!(
            "{hits} move{} would bring the {} into the part (first near {:.1}, {:.1}, {:.1}){}",
            if hits == 1 { "" } else { "s" },
            if input.tool.holder.is_some() {
                "shank or holder"
            } else {
                "shank"
            },
            p[0],
            p[1],
            p[2],
            if retract {
                ": left out, the tool lifting clear round them"
            } else {
                ""
            }
        ));
    }
    Toolpath {
        steps: out,
        warnings,
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
}
