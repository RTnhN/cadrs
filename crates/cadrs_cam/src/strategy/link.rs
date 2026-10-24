//! Putting cuts together: the tool's travel between them (the Link tab's actions), entries
//! (plunge or ramp), leads and tabs. Strategies hand the [`Builder`] contours at a depth; it
//! adds every move needed to get there safely and away again.

use crate::geom::{P2, P3, dist2, lerp2};
use crate::toolpath::{FeedKind, Step, Toolpath};

use super::path::{Contour, lead_arc, lead_out_arc, ramp_along, with_tabs};
use super::{Entry, Input, LeadKind, LinkAction};

/// Points along an XY arc from `a` to `b` about `c` (for checking what a lead sweeps).
fn arc_path(a: P2, b: P2, c: P2, cw: bool) -> Vec<P2> {
    let mut v = vec![a];
    v.extend(
        crate::toolpath::arc_points([a[0], a[1], 0.0], [b[0], b[1], 0.0], c, cw, 0.01)
            .into_iter()
            .map(|p| [p[0], p[1]]),
    );
    v
}

/// How a contour is cut.
#[derive(Debug, Clone, Copy)]
pub struct Cut<'a> {
    /// The top of the material over the cut: feeding starts the feed distance above it.
    pub material_top: f64,
    /// The tool may stay down and feed straight from where it is (the way there is cleared).
    pub direct_ok: bool,
    /// The free side for leads (+1 left of the direction of travel, −1 right); `None`: no
    /// leads.
    pub free_side: Option<f64>,
    /// Tabs: (top of the tabs, count, width).
    pub tabs: Option<(f64, usize, f64)>,
    /// Ramp in along the contour (else plunge, or lead from the free side).
    pub ramp: bool,
    /// Where the tool may come down beside the cut (a lead-in's start must be in it): `None`
    /// anywhere.
    pub clear: Option<&'a crate::region::Region>,
}

pub struct Builder<'a> {
    pub input: &'a Input<'a>,
    pub steps: Vec<Step>,
    pub at: Option<P3>,
    /// The direction the tool last moved in across XY (unit).
    pub heading: Option<P2>,
    pub warnings: Vec<String>,
}

impl<'a> Builder<'a> {
    /// A builder that starts the spindle (spindle machines).
    pub fn new(input: &'a Input<'a>) -> Builder<'a> {
        let mut b = Builder {
            input,
            steps: Vec::new(),
            at: None,
            heading: None,
            warnings: Vec::new(),
        };
        let p = input.params;
        if !p.posting.comment.is_empty() {
            b.steps.push(Step::Comment(p.posting.comment.clone()));
        }
        if !input.tool.is_beam() {
            b.steps.push(Step::Spindle {
                rpm: p.speeds.spindle_rpm,
                cw: true,
            });
        }
        b
    }

    pub fn rapid(&mut self, p: P3) {
        if self.at.is_some_and(|a| a == p) {
            return;
        }
        self.steps.push(Step::Rapid(p));
        self.at = Some(p);
    }

    pub fn feed(&mut self, p: P3, kind: FeedKind) {
        if self.at.is_some_and(|a| a == p) {
            return;
        }
        let s = &self.input.params.speeds;
        let feed = match kind {
            FeedKind::Cut | FeedKind::Link => s.feed,
            FeedKind::Plunge => s.plunge,
            FeedKind::Ramp => s.ramp,
            FeedKind::LeadIn | FeedKind::LeadOut => s.lead,
            FeedKind::Retract => s.plunge.max(s.feed),
        };
        self.feed_at(p, kind, feed);
    }

    pub fn feed_at(&mut self, p: P3, kind: FeedKind, feed: f64) {
        if self.at.is_some_and(|a| a == p) {
            return;
        }
        self.steps.push(Step::Feed { to: p, feed, kind });
        self.turn_to(p);
    }

    /// Moves to `p`, keeping the heading.
    fn turn_to(&mut self, p: P3) {
        if let Some(a) = self.at {
            let d = [p[0] - a[0], p[1] - a[1]];
            if d[0].hypot(d[1]) > 1e-9 {
                self.heading = Some(super::path::unit(d));
            }
        }
        self.at = Some(p);
    }

    pub fn arc(&mut self, to: P3, center: P2, cw: bool, kind: FeedKind) {
        let s = &self.input.params.speeds;
        let feed = match kind {
            FeedKind::LeadIn | FeedKind::LeadOut => s.lead,
            FeedKind::Ramp => s.ramp,
            _ => s.feed,
        };
        self.steps.push(Step::Arc {
            to,
            center,
            cw,
            feed,
            kind,
        });
        // Leaving an arc along its tangent.
        let r = [to[0] - center[0], to[1] - center[1]];
        if r[0].hypot(r[1]) > 1e-9 {
            let t = if cw { [r[1], -r[0]] } else { [-r[1], r[0]] };
            self.heading = Some(super::path::unit(t));
        }
        self.at = Some(to);
    }

    /// Straight up (rapid) to `z` if lower.
    pub fn up_to(&mut self, z: f64) {
        if let Some(a) = self.at
            && a[2] < z - 1e-9
        {
            self.rapid([a[0], a[1], z]);
        }
    }

    /// The height a link action retracts to over material topped at `material_top`.
    fn retract_z(&self, action: LinkAction, material_top: f64) -> f64 {
        let l = &self.input.params.links;
        match action {
            LinkAction::Direct | LinkAction::RetractToFeed => material_top + l.feed_distance,
            LinkAction::RetractToRapid => self.input.rapid_plane(),
            LinkAction::RetractToClearance => self.input.clearance(),
        }
    }

    /// Gets the tool over `xy` ready to cut at depth `z`: straight there at feed when the link
    /// may stay down (returns true: it's at depth), else up, across and down to the feed
    /// distance above the material (returns false: an entry is still needed).
    pub fn travel_to(&mut self, xy: P2, z: f64, c: &Cut<'_>) -> bool {
        let l = &self.input.params.links;
        let Some(at) = self.at else {
            let cl = self.input.clearance();
            self.rapid([xy[0], xy[1], cl]);
            self.rapid([xy[0], xy[1], (c.material_top + l.feed_distance).min(cl)]);
            return false;
        };
        let d = dist2([at[0], at[1]], xy);
        let action = if d <= l.small_gap {
            l.small_gaps
        } else {
            l.large_gaps
        };
        let same_level = (at[2] - z).abs() < 1e-6;
        if action == LinkAction::Direct && c.direct_ok && same_level {
            self.feed([xy[0], xy[1], z], FeedKind::Link);
            return true;
        }
        let action = if action == LinkAction::Direct {
            LinkAction::RetractToFeed
        } else {
            action
        };
        // Retracts to the feed distance only stay down inside cleared areas; elsewhere go to the
        // rapid plane.
        let action = if action == LinkAction::RetractToFeed && !c.direct_ok {
            LinkAction::RetractToRapid
        } else {
            action
        };
        let rz = self.retract_z(action, c.material_top).max(at[2]);
        self.up_to(rz);
        self.rapid([xy[0], xy[1], rz]);
        let start = c.material_top + l.feed_distance;
        if start < rz {
            self.rapid([xy[0], xy[1], start]);
        }
        false
    }

    /// Cuts a contour at depth `z`, with its entry, leads, overlap and tabs.
    pub fn cut(&mut self, c: &Contour, z: f64, cut: &Cut<'_>) {
        if c.pts.is_empty() {
            return;
        }
        let p = self.input.params;
        let l = &p.links;
        let walk = c.walk();
        let tabs_here = cut.tabs.filter(|(top, _, _)| *top > z + 1e-9 && c.closed);
        // Leads (in the free side only).
        let lead_in = cut
            .free_side
            .filter(|_| l.lead_in != LeadKind::None && l.lead_radius > 0.0)
            .and_then(|side| {
                let (pt, t) = c.start_tangent()?;
                let lead = match l.lead_in {
                    LeadKind::Arc => {
                        let (s, ctr, cw) = lead_arc(pt, t, side, l.lead_radius, 90.0);
                        (s, Some((ctr, cw)))
                    }
                    _ => {
                        let n = [-t[1] * side, t[0] * side];
                        (
                            [pt[0] + n[0] * l.lead_radius, pt[1] + n[1] * l.lead_radius],
                            None,
                        )
                    }
                };
                // Only where the tool can come down and run to the contour: what it sweeps along
                // the lead clear (else it ramps in along the contour).
                let path = match lead.1 {
                    Some((ctr, cw)) => arc_path(lead.0, pt, ctr, cw),
                    None => vec![lead.0, pt],
                };
                self.swept_clear(&path, cut.clear).then_some(lead)
            });
        let start = walk[0];
        let entry_xy = lead_in.map_or(start, |(s, _)| s);
        let at_depth = self.travel_to(entry_xy, z, cut);
        if !at_depth {
            let use_ramp =
                cut.ramp && lead_in.is_none() && tabs_here.is_none() && l.entry == Entry::Ramp;
            if use_ramp {
                let from = self.at.map_or(cut.material_top, |a| a[2]);
                let pts = ramp_along(c, from, z, l.ramp_angle);
                let mut deep = false;
                for q in pts.iter().skip(1) {
                    if !deep && (q[2] - z).abs() < 1e-9 {
                        deep = true;
                        self.feed(*q, FeedKind::Ramp);
                        continue;
                    }
                    self.feed(*q, if deep { FeedKind::Cut } else { FeedKind::Ramp });
                }
                self.finish_contour(c, z, cut, true);
                return;
            }
            self.feed([entry_xy[0], entry_xy[1], z], FeedKind::Plunge);
        }
        if let Some((_, arc)) = lead_in {
            match arc {
                Some((ctr, cw)) => self.arc([start[0], start[1], z], ctr, cw, FeedKind::LeadIn),
                None => self.feed([start[0], start[1], z], FeedKind::LeadIn),
            }
        }
        match tabs_here {
            Some((top, count, width)) => {
                for q in with_tabs(&walk, z, top, count, width).iter().skip(1) {
                    self.feed(*q, FeedKind::Cut);
                }
            }
            None => {
                for q in walk.iter().skip(1) {
                    self.feed([q[0], q[1], z], FeedKind::Cut);
                }
            }
        }
        self.finish_contour(c, z, cut, false);
    }

    /// The overlap past the start of a closed contour and the lead-out.
    fn finish_contour(&mut self, c: &Contour, z: f64, cut: &Cut<'_>, ramped: bool) {
        let l = &self.input.params.links;
        let mut end_contour = c.clone();
        if c.closed && l.overlap > 0.0 && !ramped {
            // Along the start again.
            let mut left = l.overlap;
            let walk = c.walk();
            for w in walk.windows(2) {
                let d = dist2(w[0], w[1]);
                if d >= left {
                    let q = lerp2(w[0], w[1], left / d);
                    self.feed([q[0], q[1], z], FeedKind::Cut);
                    end_contour = Contour::open(vec![w[0], q]);
                    break;
                }
                self.feed([w[1][0], w[1][1], z], FeedKind::Cut);
                left -= d;
            }
        }
        let Some(side) = cut.free_side else {
            return;
        };
        if l.lead_out == LeadKind::None || l.lead_radius <= 0.0 {
            return;
        }
        let Some((pt, t)) = end_contour.end_tangent() else {
            return;
        };
        // Leave from where the tool is, the way it's going (a ramp ends where it reached depth).
        let pt = self.at.map_or(pt, |a| [a[0], a[1]]);
        let t = self.heading.unwrap_or(t);
        // Off the wall only where what the tool sweeps is clear (else it retracts straight up).
        match l.lead_out {
            LeadKind::Arc => {
                let (e, ctr, cw) = lead_out_arc(pt, t, side, l.lead_radius, 90.0);
                if self.swept_clear(&arc_path(pt, e, ctr, cw), cut.clear) {
                    self.arc([e[0], e[1], z], ctr, cw, FeedKind::LeadOut);
                }
            }
            _ => {
                let n = [-t[1] * side, t[0] * side];
                let e = [pt[0] + n[0] * l.lead_radius, pt[1] + n[1] * l.lead_radius];
                if self.swept_clear(&[pt, e], cut.clear) {
                    self.feed([e[0], e[1], z], FeedKind::LeadOut);
                }
            }
        }
    }

    /// Whether the tool moving along `path` stays inside `clear` (`None`: anywhere is).
    fn swept_clear(&self, path: &[P2], clear: Option<&crate::region::Region>) -> bool {
        let Some(clear) = clear else {
            return true;
        };
        let r = self.input.tool.radius();
        let swept = crate::region::stroke(path, 2.0 * r * 0.98);
        crate::region::area(&crate::region::difference(&swept, clear)) < 1e-3
    }

    /// Ends at the clearance height.
    pub fn finish(mut self) -> Toolpath {
        let cl = self.input.clearance();
        self.up_to(cl);
        Toolpath {
            steps: self.steps,
            warnings: self.warnings,
        }
    }

    /// Whether anything was cut.
    pub fn moved(&self) -> bool {
        self.at.is_some()
    }
}
