//! Sketch patterns (Onshape's sketch *Linear pattern* and *Circular pattern*): copies of seed
//! curves, linked to the seeds as Onshape links them.
//!
//! A [`SketchPattern`] is kept in [`Sketch::patterns`]: its seeds, its [`Layout`] and its
//! copies, each with the seed point and curve it copies.
//! - **Linear:** one or two directions, each a distance and a count along a sketch line or axis
//!   ([`PatternDir::along`], followed as the line turns) or at a fixed angle.
//! - **Circular:** a count over a total angle about a center (a sketch point or the origin,
//!   followed as it moves); a full turn spaces the instances evenly round it.
//!
//! The solver ([`crate::solve`]) holds every copied point where the copy's [`Xform`] puts its
//! seed point (and a copied circle's or ellipse's radius at its seed's), so editing or dragging
//! a seed moves its copies, and dragging a copy moves the seed. [`edit_pattern`] changes the
//! layout afterwards, adding or removing copies to match.
//!
//! The copies carry no constraints of their own: the pattern places them. Deleting a copy
//! drops it from its pattern; a pattern with no seeds or no copies left is dropped, and so is a
//! circular one whose center point is deleted. A direction whose line is deleted keeps the
//! line's last angle ([`Sketch::prune_patterns`]).

use std::collections::HashMap;
use std::f64::consts::TAU;

use serde::{Deserialize, Serialize};

use crate::constraint::{CurveRef, PointRef};
use crate::{Curve, CurveId, CurveKind, PatternId, PointId, Sketch, Vec2};

/// The most instances a pattern makes (the seed included), so a mistyped count can't stall
/// the sketch.
pub const MAX_INSTANCES: u32 = 2500;

/// One direction of a linear pattern.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PatternDir {
    /// Its direction, in radians anticlockwise from the sketch's x axis, when it is along no
    /// line (and the line's last direction when it is).
    pub angle: f64,
    /// How far apart neighbouring instances are (mm; negative: the other way).
    pub spacing: f64,
    /// How many instances there are along it, the seed included (at least 1).
    pub count: u32,
    /// The sketch line or axis it runs along (from its first end to its second), if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub along: Option<CurveRef>,
}

impl PatternDir {
    /// Along the x axis (`angle_deg` 0) or at an angle, `count` instances `spacing` apart.
    pub fn at_angle(angle_deg: f64, spacing: f64, count: u32) -> Self {
        Self {
            angle: angle_deg.to_radians(),
            spacing,
            count,
            along: None,
        }
    }

    /// Its unit direction: its line's, or its angle's.
    pub fn unit(&self, s: &Sketch) -> Vec2 {
        if let Some(u) = self.along.and_then(|l| line_dir(s, l)) {
            return u;
        }
        Vec2::new(self.angle.cos(), self.angle.sin())
    }

    /// The step from one instance to the next.
    pub fn step(&self, s: &Sketch) -> Vec2 {
        self.unit(s) * self.spacing
    }
}

/// The unit direction of a sketch line or axis (first end to second), if it is one with length.
pub fn line_dir(s: &Sketch, l: CurveRef) -> Option<Vec2> {
    let (a, b) = crate::dimension::line_ends(s, l)?;
    let d = b - a;
    (d.length() > 1e-9).then(|| d * (1.0 / d.length()))
}

/// A circular pattern: `count` instances (the seed included) over `angle` about `center`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circular {
    pub center: PointRef,
    pub count: u32,
    /// The angle the instances span (radians, anticlockwise; negative: clockwise). A full turn
    /// (360°) spaces them evenly round the center; less puts the last one at the angle.
    pub angle: f64,
}

impl Circular {
    /// The turn from one instance to the next.
    pub fn step(&self) -> f64 {
        if (self.angle.abs() - TAU).abs() < 1e-9 {
            self.angle / self.count.max(1) as f64
        } else {
            self.angle / (self.count.max(2) - 1) as f64
        }
    }

    /// Where its center is now.
    pub fn center_pos(&self, s: &Sketch) -> Option<Vec2> {
        match self.center {
            PointRef::Origin => Some(Vec2::ZERO),
            PointRef::Point(p) => s.points.get(p).map(|x| x.pos),
        }
    }
}

/// How a pattern lays out its copies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Layout {
    /// One or two directions.
    Linear(Vec<PatternDir>),
    Circular(Circular),
}

impl Layout {
    /// "Linear pattern" or "Circular pattern".
    pub fn name(&self) -> &'static str {
        match self {
            Layout::Linear(_) => "Linear pattern",
            Layout::Circular(_) => "Circular pattern",
        }
    }
}

/// Where a copy is: its seed turned `angle` (radians) about `center`, then moved by `shift`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Xform {
    pub center: Vec2,
    pub angle: f64,
    pub shift: Vec2,
}

impl Xform {
    /// The image of a vector (a spline's end derivative): turned, not moved.
    pub fn turn(&self, v: Vec2) -> Vec2 {
        let (sin, cos) = self.angle.sin_cos();
        Vec2::new(cos * v.x - sin * v.y, sin * v.x + cos * v.y)
    }

    /// The image of a point.
    pub fn apply(&self, p: Vec2) -> Vec2 {
        self.center + self.turn(p - self.center) + self.shift
    }
}

/// One copy of a pattern's seeds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternCopy {
    /// Where it is in the pattern: the step along each direction (the seed is `[0, 0]`; a
    /// circular pattern's copies are `[k, 0]`).
    pub index: [u32; 2],
    /// Each seed curve and its copy.
    pub curves: Vec<(CurveId, CurveId)>,
    /// Each seed point and its copy.
    pub points: Vec<(PointId, PointId)>,
}

/// A sketch pattern (see the module docs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SketchPattern {
    pub seeds: Vec<CurveId>,
    pub layout: Layout,
    pub copies: Vec<PatternCopy>,
}

impl SketchPattern {
    /// Where the copy at `index` is, as the sketch is now.
    pub fn xform(&self, s: &Sketch, index: [u32; 2]) -> Xform {
        xform(s, &self.layout, index)
    }

    /// True if `c` is one of the pattern's seeds or copies.
    pub fn has_curve(&self, c: CurveId) -> bool {
        self.seeds.contains(&c)
            || self
                .copies
                .iter()
                .any(|k| k.curves.iter().any(|x| x.1 == c))
    }
}

/// Where the copy at `index` of `layout` is, as the sketch is now.
pub fn xform(s: &Sketch, layout: &Layout, index: [u32; 2]) -> Xform {
    match layout {
        Layout::Linear(dirs) => Xform {
            center: Vec2::ZERO,
            angle: 0.0,
            shift: dirs
                .iter()
                .zip(index)
                .fold(Vec2::ZERO, |acc, (d, i)| acc + d.step(s) * i as f64),
        },
        Layout::Circular(c) => Xform {
            center: c.center_pos(s).unwrap_or_default(),
            angle: c.step() * index[0] as f64,
            shift: Vec2::ZERO,
        },
    }
}

/// Every copy's index for `layout`, the seed's `[0, 0]` left out.
pub fn indices(layout: &Layout) -> Vec<[u32; 2]> {
    let (n0, n1) = match layout {
        Layout::Linear(dirs) => {
            let n = |k: usize| dirs.get(k).map_or(1, |d| d.count.max(1));
            (n(0), n(1))
        }
        Layout::Circular(c) => (c.count.max(1), 1),
    };
    let mut out = Vec::new();
    for j in 0..n1 {
        for i in 0..n0 {
            if (i, j) != (0, 0) {
                out.push([i, j]);
            }
        }
    }
    out
}

/// Where every copy `layout` makes is (for a preview).
pub fn xforms(s: &Sketch, layout: &Layout) -> Vec<Xform> {
    indices(layout)
        .into_iter()
        .map(|i| xform(s, layout, i))
        .collect()
}

/// Checks a layout: a linear one has one or two directions, each with a count, a direction (a
/// line with length, or an angle) and (with more than one instance) a distance, not parallel;
/// a circular one a center, a count and an angle of at most a turn. Either makes at least one
/// copy, at most [`MAX_INSTANCES`] instances.
pub fn check(s: &Sketch, layout: &Layout) -> Result<(), String> {
    let total: u64 = match layout {
        Layout::Linear(dirs) => {
            if dirs.is_empty() || dirs.len() > 2 {
                return Err("a linear pattern has one or two directions".into());
            }
            let mut total: u64 = 1;
            for d in dirs {
                if d.count == 0 {
                    return Err("a direction needs at least one instance".into());
                }
                if !d.angle.is_finite() || !d.spacing.is_finite() {
                    return Err("a direction needs an angle and a distance".into());
                }
                if let Some(l) = d.along
                    && line_dir(s, l).is_none()
                {
                    return Err("a direction runs along a line".into());
                }
                if d.count > 1 && d.spacing.abs() < 1e-9 {
                    return Err("the instances need a distance between them".into());
                }
                total *= d.count as u64;
            }
            if let [a, b] = &dirs[..]
                && a.count > 1
                && b.count > 1
                && a.unit(s).cross(b.unit(s)).abs() < 1e-9
            {
                return Err("the two directions are parallel".into());
            }
            total
        }
        Layout::Circular(c) => {
            if c.center_pos(s).is_none() {
                return Err("the center is gone".into());
            }
            if !c.angle.is_finite() || c.angle.abs() < 1e-9 || c.angle.abs() > TAU + 1e-9 {
                return Err("a circular pattern turns through up to 360°".into());
            }
            c.count as u64
        }
    };
    if total < 2 {
        return Err("a pattern needs more than one instance".into());
    }
    if total > MAX_INSTANCES as u64 {
        return Err(format!("a pattern has at most {MAX_INSTANCES} instances"));
    }
    Ok(())
}

/// The curves a pattern can copy: those of `curves` that are in the sketch, once each, and not
/// part of a text (whose box and outline go together).
fn seeds_of(s: &Sketch, curves: &[CurveId]) -> Vec<CurveId> {
    let mut out: Vec<CurveId> = Vec::new();
    for &c in curves {
        if s.curves.contains_key(c)
            && !out.contains(&c)
            && crate::text::text_of_curve(s, c).is_none()
        {
            out.push(c);
        }
    }
    out
}

/// Patterns `curves` as `layout` lays them out (the Linear and Circular pattern tools).
/// Returns the new pattern.
pub fn add_pattern(
    s: &mut Sketch,
    curves: &[CurveId],
    layout: Layout,
) -> Result<PatternId, String> {
    check(s, &layout)?;
    let seeds = seeds_of(s, curves);
    if seeds.is_empty() {
        return Err("nothing to pattern".into());
    }
    let id = s.patterns.insert(SketchPattern {
        seeds,
        layout,
        copies: Vec::new(),
    });
    regenerate(s, id);
    Ok(id)
}

/// Gives a pattern a new layout (of the same kind): copies no longer wanted are deleted, new
/// ones are made, and every copy is moved to its place.
pub fn edit_pattern(s: &mut Sketch, id: PatternId, layout: Layout) -> Result<(), String> {
    check(s, &layout)?;
    let p = s.patterns.get_mut(id).ok_or("the pattern is gone")?;
    if std::mem::discriminant(&p.layout) != std::mem::discriminant(&layout) {
        return Err("a linear pattern stays linear, a circular one circular".into());
    }
    p.layout = layout;
    regenerate(s, id);
    Ok(())
}

/// Each direction along a line takes the line's angle (kept if the line goes).
fn sync_angles(s: &mut Sketch, id: PatternId) {
    let Some(Layout::Linear(dirs)) = s.patterns.get(id).map(|p| p.layout.clone()) else {
        return;
    };
    let dirs: Vec<PatternDir> = dirs
        .into_iter()
        .map(|mut d| {
            if let Some(u) = d.along.and_then(|l| line_dir(s, l)) {
                d.angle = u.y.atan2(u.x);
            }
            d
        })
        .collect();
    s.patterns[id].layout = Layout::Linear(dirs);
}

/// Makes the pattern's copies match its layout and moves them into place.
fn regenerate(s: &mut Sketch, id: PatternId) {
    sync_angles(s, id);
    let Some(p) = s.patterns.get(id) else { return };
    let want = indices(&p.layout);
    let seeds = p.seeds.clone();
    // Copies past the counts go.
    let (keep, drop): (Vec<PatternCopy>, Vec<PatternCopy>) = p
        .copies
        .iter()
        .cloned()
        .partition(|c| want.contains(&c.index));
    s.patterns[id].copies = keep;
    for c in drop {
        for (_, k) in c.curves {
            s.remove_curve(k);
        }
    }
    // New ones are made where there are none.
    let have: Vec<[u32; 2]> = s.patterns[id].copies.iter().map(|c| c.index).collect();
    for index in want {
        if !have.contains(&index) {
            let x = s.patterns[id].xform(s, index);
            let copy = make_copy(s, &seeds, index, x);
            s.patterns[id].copies.push(copy);
        }
    }
    s.patterns[id]
        .copies
        .sort_by_key(|c| (c.index[1], c.index[0]));
    place_copies(s, id);
}

/// Moves every copy of a pattern to where its xform puts its seed (radii too), as the solver
/// holds them.
fn place_copies(s: &mut Sketch, id: PatternId) {
    let Some(p) = s.patterns.get(id).cloned() else {
        return;
    };
    for c in &p.copies {
        let x = p.xform(s, c.index);
        for &(a, b) in &c.points {
            if s.points.contains_key(a) && s.points.contains_key(b) {
                let pos = x.apply(s.pos(a));
                s.points[b].pos = pos;
            }
        }
        for &(a, b) in &c.curves {
            if let (Some(v), Some(_)) = (
                s.curves.get(a).and_then(|x| x.kind.scalar()),
                s.curves.get(b),
            ) {
                s.curves[b].kind.set_scalar(v);
            }
        }
    }
}

/// A copy of `seeds` placed by `x`, on points of its own.
fn make_copy(s: &mut Sketch, seeds: &[CurveId], index: [u32; 2], x: Xform) -> PatternCopy {
    let mut map: HashMap<PointId, PointId> = HashMap::new();
    let mut points: Vec<(PointId, PointId)> = Vec::new();
    let mut curves = Vec::new();
    for &c in seeds {
        let Some(curve) = s.curves.get(c).copied() else {
            continue;
        };
        let mut image = |s: &mut Sketch, p: PointId| -> PointId {
            if let Some(q) = map.get(&p) {
                return *q;
            }
            let q = s.add_point(x.apply(s.pos(p)));
            map.insert(p, q);
            points.push((p, q));
            q
        };
        let kind = match curve.kind {
            CurveKind::Line { a, b } => CurveKind::Line {
                a: image(s, a),
                b: image(s, b),
            },
            CurveKind::Circle { center, radius } => CurveKind::Circle {
                center: image(s, center),
                radius,
            },
            CurveKind::Arc { center, start, end } => CurveKind::Arc {
                center: image(s, center),
                start: image(s, start),
                end: image(s, end),
            },
            CurveKind::Ellipse {
                center,
                major,
                minor,
            } => CurveKind::Ellipse {
                center: image(s, center),
                major: image(s, major),
                minor,
            },
            CurveKind::EllipseOffset {
                center,
                major,
                minor,
                distance,
            } => CurveKind::EllipseOffset {
                center: image(s, center),
                major: image(s, major),
                minor,
                distance,
            },
            CurveKind::EllipseArc {
                center,
                major,
                minor,
                start,
                end,
            } => CurveKind::EllipseArc {
                center: image(s, center),
                major: image(s, major),
                minor,
                start: image(s, start),
                end: image(s, end),
            },
            CurveKind::Bezier { a, c1, c2, b } => CurveKind::Bezier {
                a: image(s, a),
                c1: image(s, c1),
                c2: image(s, c2),
                b: image(s, b),
            },
            CurveKind::Spline { start, end } => {
                let Some(mut d) = s.splines.get(c).cloned() else {
                    continue;
                };
                d.points = d.points.iter().map(|p| image(s, *p)).collect();
                d.start_tangent = d.start_tangent.map(|v| x.turn(v));
                d.end_tangent = d.end_tangent.map(|v| x.turn(v));
                let kind = CurveKind::Spline {
                    start: image(s, start),
                    end: image(s, end),
                };
                let copy = s.curves.insert(Curve {
                    kind,
                    construction: curve.construction,
                });
                s.splines.insert(copy, d);
                curves.push((c, copy));
                continue;
            }
        };
        let copy = s.curves.insert(Curve {
            kind,
            construction: curve.construction,
        });
        curves.push((c, copy));
    }
    PatternCopy {
        index,
        curves,
        points,
    }
}

/// The pattern a curve is a seed or copy of.
pub fn pattern_of_curve(s: &Sketch, c: CurveId) -> Option<PatternId> {
    s.patterns
        .iter()
        .find(|(_, p)| p.has_curve(c))
        .map(|(k, _)| k)
}

impl Sketch {
    /// Drops what deleted curves and points leave of the patterns: a copied pair whose seed or
    /// copy is gone, a copy with no curves, a pattern with no seeds or no copies, and a
    /// circular one whose center point is gone. A direction whose line is gone keeps its angle.
    pub fn prune_patterns(&mut self) {
        if self.patterns.is_empty() {
            return;
        }
        let (curves, points) = (&self.curves, &self.points);
        let line_gone = |l: CurveRef| matches!(l, CurveRef::Curve(k) if !curves.contains_key(k));
        for p in self.patterns.values_mut() {
            p.seeds.retain(|c| curves.contains_key(*c));
            for c in &mut p.copies {
                c.curves
                    .retain(|(a, b)| curves.contains_key(*a) && curves.contains_key(*b));
                c.points
                    .retain(|(a, b)| points.contains_key(*a) && points.contains_key(*b));
            }
            p.copies.retain(|c| !c.curves.is_empty());
            if let Layout::Linear(dirs) = &mut p.layout {
                for d in dirs {
                    if d.along.is_some_and(line_gone) {
                        d.along = None;
                    }
                }
            }
        }
        self.patterns.retain(|_, p| {
            !p.seeds.is_empty()
                && !p.copies.is_empty()
                && match p.layout {
                    Layout::Circular(Circular {
                        center: PointRef::Point(c),
                        ..
                    }) => points.contains_key(c),
                    _ => true,
                }
        });
    }

    /// Points `gone` merged into `keep`: the patterns' copied pairs and centers follow.
    pub(crate) fn merge_pattern_points(&mut self, keep: PointId, gone: PointId) {
        for p in self.patterns.values_mut() {
            if let Layout::Circular(c) = &mut p.layout
                && c.center == PointRef::Point(gone)
            {
                c.center = PointRef::Point(keep);
            }
            for c in &mut p.copies {
                for (a, b) in &mut c.points {
                    if *a == gone {
                        *a = keep;
                    }
                    if *b == gone {
                        *b = keep;
                    }
                }
                // A point merged into its own copy holds nothing.
                c.points.retain(|(a, b)| a != b);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SketchOp;

    fn dir(angle_deg: f64, spacing: f64, count: u32) -> PatternDir {
        PatternDir::at_angle(angle_deg, spacing, count)
    }

    fn rectangle(s: &mut Sketch) -> Vec<CurveId> {
        SketchOp::AddPolyline {
            points: vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(4.0, 0.0),
                Vec2::new(4.0, 2.0),
                Vec2::new(0.0, 2.0),
            ],
            closed: true,
            construction: false,
            label: "Add rectangle",
        }
        .apply(s)
        .unwrap();
        s.curves.keys().collect()
    }

    fn pattern(s: &Sketch) -> (PatternId, SketchPattern) {
        let (k, p) = s.patterns.iter().next().unwrap();
        (k, p.clone())
    }

    #[test]
    fn copies_a_rectangle_along_one_direction() {
        let mut s = Sketch::default();
        let seeds = rectangle(&mut s);
        SketchOp::LinearPattern {
            curves: seeds.clone(),
            dirs: vec![dir(0.0, 10.0, 3)],
        }
        .apply(&mut s)
        .unwrap();
        assert_eq!(s.curves.len(), 12);
        let (_, p) = pattern(&s);
        assert_eq!(p.copies.len(), 2);
        let last = &p.copies[1];
        assert_eq!(last.index, [2, 0]);
        for &(a, b) in &last.points {
            assert!((s.pos(b) - s.pos(a) - Vec2::new(20.0, 0.0)).length() < 1e-9);
        }
        // The copies have points of their own.
        assert_eq!(s.points.len(), 12);
    }

    #[test]
    fn two_directions_make_a_grid() {
        let mut s = Sketch::default();
        SketchOp::AddCircle {
            center: Vec2::new(0.0, 0.0),
            radius: 1.0,
            construction: false,
        }
        .apply(&mut s)
        .unwrap();
        let seeds: Vec<CurveId> = s.curves.keys().collect();
        SketchOp::LinearPattern {
            curves: seeds,
            dirs: vec![dir(0.0, 5.0, 3), dir(90.0, 4.0, 2)],
        }
        .apply(&mut s)
        .unwrap();
        assert_eq!(s.curves.len(), 6);
        let mut centers: Vec<(i64, i64)> = s
            .curves
            .values()
            .map(|c| match c.kind {
                CurveKind::Circle { center, .. } => {
                    let p = s.pos(center);
                    (p.x.round() as i64, p.y.round() as i64)
                }
                _ => unreachable!(),
            })
            .collect();
        centers.sort();
        assert_eq!(centers, [(0, 0), (0, 4), (5, 0), (5, 4), (10, 0), (10, 4)]);
    }

    #[test]
    fn copies_follow_their_seed() {
        let mut s = Sketch::default();
        SketchOp::AddCircle {
            center: Vec2::new(0.0, 0.0),
            radius: 1.0,
            construction: false,
        }
        .apply(&mut s)
        .unwrap();
        let seed = s.curves.keys().next().unwrap();
        let CurveKind::Circle { center, .. } = s.curves[seed].kind else {
            unreachable!()
        };
        SketchOp::LinearPattern {
            curves: vec![seed],
            dirs: vec![dir(0.0, 5.0, 2)],
        }
        .apply(&mut s)
        .unwrap();
        // Dragging the seed's center and changing its radius moves and resizes the copy.
        assert!(crate::solve::drag(
            &mut s,
            &crate::solve::Drag::Points(vec![(center, Vec2::new(1.0, 2.0))]),
            &Default::default(),
        ));
        SketchOp::SetDimension {
            dimension: crate::Dimension::new(
                crate::DimensionKind::Radius { curve: seed },
                3.0,
                0.0,
            ),
            moves: vec![],
            radii: vec![],
        }
        .apply(&mut s)
        .unwrap();
        let (_, p) = pattern(&s);
        let (_, copy) = p.copies[0].curves[0];
        let CurveKind::Circle {
            center: cc,
            radius: r,
        } = s.curves[copy].kind
        else {
            unreachable!()
        };
        assert!((s.pos(cc) - Vec2::new(6.0, 2.0)).length() < 1e-6);
        assert!((r - 3.0).abs() < 1e-6);
    }

    #[test]
    fn a_copy_of_a_fixed_seed_is_fully_constrained() {
        let mut s = Sketch::default();
        SketchOp::AddCircle {
            center: Vec2::new(0.0, 0.0),
            radius: 1.0,
            construction: false,
        }
        .apply(&mut s)
        .unwrap();
        let seed = s.curves.keys().next().unwrap();
        SketchOp::LinearPattern {
            curves: vec![seed],
            dirs: vec![dir(0.0, 5.0, 2)],
        }
        .apply(&mut s)
        .unwrap();
        let copy = pattern(&s).1.copies[0].curves[0].1;
        use crate::solve::{Status, analyze};
        assert_eq!(analyze(&s).curve(copy), Status::Under);
        SketchOp::AddConstraint {
            constraints: vec![crate::ConstraintOf::FixCurve(crate::CurveRef::Curve(seed))],
            label: "Add fix",
        }
        .apply(&mut s)
        .unwrap();
        assert_eq!(analyze(&s).curve(copy), Status::Full);
    }

    #[test]
    fn editing_the_count_adds_and_removes_copies() {
        let mut s = Sketch::default();
        let seeds = rectangle(&mut s);
        SketchOp::LinearPattern {
            curves: seeds,
            dirs: vec![dir(0.0, 10.0, 2)],
        }
        .apply(&mut s)
        .unwrap();
        let (id, _) = pattern(&s);
        SketchOp::EditPattern {
            id,
            layout: Layout::Linear(vec![dir(0.0, 6.0, 4)]),
        }
        .apply(&mut s)
        .unwrap();
        assert_eq!(s.curves.len(), 16);
        let (_, p) = pattern(&s);
        let far = p.copies.iter().find(|c| c.index == [3, 0]).unwrap();
        for &(a, b) in &far.points {
            assert!((s.pos(b) - s.pos(a) - Vec2::new(18.0, 0.0)).length() < 1e-9);
        }
        SketchOp::EditPattern {
            id,
            layout: Layout::Linear(vec![dir(0.0, 6.0, 2)]),
        }
        .apply(&mut s)
        .unwrap();
        assert_eq!(s.curves.len(), 8);
        assert_eq!(s.points.len(), 8);
    }

    #[test]
    fn deleting_every_copy_drops_the_pattern() {
        let mut s = Sketch::default();
        let seeds = rectangle(&mut s);
        SketchOp::LinearPattern {
            curves: seeds.clone(),
            dirs: vec![dir(0.0, 10.0, 2)],
        }
        .apply(&mut s)
        .unwrap();
        let copies: Vec<CurveId> = s.curves.keys().filter(|c| !seeds.contains(c)).collect();
        SketchOp::Delete {
            curves: copies[..1].to_vec(),
            points: vec![],
            dimensions: vec![],
            constraints: vec![],
        }
        .apply(&mut s)
        .unwrap();
        assert_eq!(pattern(&s).1.copies[0].curves.len(), 3);
        SketchOp::Delete {
            curves: copies[1..].to_vec(),
            points: vec![],
            dimensions: vec![],
            constraints: vec![],
        }
        .apply(&mut s)
        .unwrap();
        assert!(s.patterns.is_empty());
    }

    #[test]
    fn rejects_bad_directions() {
        let mut s = Sketch::default();
        let seeds = rectangle(&mut s);
        for dirs in [
            vec![dir(0.0, 10.0, 1)],
            vec![dir(0.0, 0.0, 3)],
            vec![dir(0.0, 5.0, 2), dir(180.0, 5.0, 2)],
            vec![dir(0.0, 5.0, 100), dir(90.0, 5.0, 100)],
        ] {
            assert!(
                SketchOp::LinearPattern {
                    curves: seeds.clone(),
                    dirs
                }
                .apply(&mut s)
                .is_err()
            );
        }
        assert!(s.patterns.is_empty());
        assert_eq!(s.curves.len(), 4);
    }

    #[test]
    fn round_trips_through_ron() {
        let mut s = Sketch::default();
        let seeds = rectangle(&mut s);
        SketchOp::LinearPattern {
            curves: seeds,
            dirs: vec![dir(30.0, 7.0, 3)],
        }
        .apply(&mut s)
        .unwrap();
        let back: Sketch = ron::from_str(&ron::to_string(&s).unwrap()).unwrap();
        assert_eq!(back.patterns.len(), 1);
        assert_eq!(pattern(&back).1, pattern(&s).1);
    }

    fn circle(s: &mut Sketch, x: f64, y: f64, r: f64) -> CurveId {
        let before: Vec<CurveId> = s.curves.keys().collect();
        SketchOp::AddCircle {
            center: Vec2::new(x, y),
            radius: r,
            construction: false,
        }
        .apply(s)
        .unwrap();
        s.curves.keys().find(|k| !before.contains(k)).unwrap()
    }

    fn center_of(s: &Sketch, c: CurveId) -> Vec2 {
        match s.curves[c].kind {
            CurveKind::Circle { center, .. } => s.pos(center),
            _ => unreachable!(),
        }
    }

    fn line(s: &mut Sketch, a: Vec2, b: Vec2) -> CurveId {
        let before: Vec<CurveId> = s.curves.keys().collect();
        SketchOp::AddPolyline {
            points: vec![a, b],
            closed: false,
            construction: true,
            label: "Add line",
        }
        .apply(s)
        .unwrap();
        s.curves.keys().find(|k| !before.contains(k)).unwrap()
    }

    #[test]
    fn a_direction_along_a_line_follows_the_line() {
        let mut s = Sketch::default();
        let seed = circle(&mut s, 0.0, 0.0, 1.0);
        let l = line(&mut s, Vec2::new(0.0, -10.0), Vec2::new(10.0, -10.0));
        SketchOp::LinearPattern {
            curves: vec![seed],
            dirs: vec![PatternDir {
                along: Some(CurveRef::Curve(l)),
                ..dir(0.0, 5.0, 2)
            }],
        }
        .apply(&mut s)
        .unwrap();
        let copy = pattern(&s).1.copies[0].curves[0].1;
        assert!((center_of(&s, copy) - Vec2::new(5.0, 0.0)).length() < 1e-9);
        // Turning the line a quarter turn (its far end dragged straight up) turns the pattern.
        let CurveKind::Line { b, .. } = s.curves[l].kind else {
            unreachable!()
        };
        assert!(crate::solve::drag(
            &mut s,
            &crate::solve::Drag::Points(vec![(b, Vec2::new(0.0, 0.0))]),
            &Default::default(),
        ));
        let d = center_of(&s, copy) - center_of(&s, seed);
        let (a, bb) = crate::dimension::line_ends(&s, CurveRef::Curve(l)).unwrap();
        // The line turns about its other end, and the seed stays: the copies, which the
        // pattern drives, are what move.
        assert!((a - Vec2::new(0.0, -10.0)).length() < 1e-4, "{a:?}");
        assert!(center_of(&s, seed).length() < 1e-4);
        let u = (bb - a) * (1.0 / (bb - a).length());
        assert!((d - u * 5.0).length() < 1e-6, "{d:?} {u:?}");
        // Deleting the line keeps the pattern at the line's last angle.
        SketchOp::Delete {
            curves: vec![l],
            points: vec![],
            dimensions: vec![],
            constraints: vec![],
        }
        .apply(&mut s)
        .unwrap();
        let Layout::Linear(dirs) = &pattern(&s).1.layout else {
            unreachable!()
        };
        assert_eq!(dirs[0].along, None);
    }

    #[test]
    fn the_sketch_axes_are_directions_too() {
        let mut s = Sketch::default();
        let seed = circle(&mut s, 0.0, 0.0, 1.0);
        SketchOp::LinearPattern {
            curves: vec![seed],
            dirs: vec![PatternDir {
                along: Some(CurveRef::YAxis),
                ..dir(0.0, 3.0, 2)
            }],
        }
        .apply(&mut s)
        .unwrap();
        let copy = pattern(&s).1.copies[0].curves[0].1;
        assert!((center_of(&s, copy) - Vec2::new(0.0, 3.0)).length() < 1e-9);
    }

    #[test]
    fn a_full_circle_spaces_the_instances_evenly() {
        let mut s = Sketch::default();
        let seed = circle(&mut s, 10.0, 0.0, 1.0);
        SketchOp::CircularPattern {
            curves: vec![seed],
            circular: Circular {
                center: PointRef::Origin,
                count: 4,
                angle: TAU,
            },
        }
        .apply(&mut s)
        .unwrap();
        let p = pattern(&s).1;
        let got: Vec<Vec2> = p
            .copies
            .iter()
            .map(|c| center_of(&s, c.curves[0].1))
            .collect();
        let want = [
            Vec2::new(0.0, 10.0),
            Vec2::new(-10.0, 0.0),
            Vec2::new(0.0, -10.0),
        ];
        for (g, w) in got.iter().zip(want) {
            assert!((*g - w).length() < 1e-9, "{got:?}");
        }
    }

    #[test]
    fn a_partial_angle_ends_on_it() {
        let mut s = Sketch::default();
        let seeds = rectangle(&mut s);
        SketchOp::CircularPattern {
            curves: seeds,
            circular: Circular {
                center: PointRef::Origin,
                count: 3,
                angle: 90f64.to_radians(),
            },
        }
        .apply(&mut s)
        .unwrap();
        // The corner at (4, 0) goes to 45° and 90°.
        let p = pattern(&s).1;
        let corner = s.point_at(Vec2::new(4.0, 0.0), 1e-9).unwrap();
        let last = &p.copies[1];
        let q = last.points.iter().find(|(a, _)| *a == corner).unwrap().1;
        assert!((s.pos(q) - Vec2::new(0.0, 4.0)).length() < 1e-9);
        assert_eq!(s.curves.len(), 12);
    }

    #[test]
    fn copies_turn_about_a_center_point_that_moves() {
        let mut s = Sketch::default();
        let seed = circle(&mut s, 10.0, 0.0, 1.0);
        SketchOp::AddPoint {
            pos: Vec2::new(0.0, 0.0),
        }
        .apply(&mut s)
        .unwrap();
        let c = s.point_at(Vec2::ZERO, 1e-9).unwrap();
        SketchOp::CircularPattern {
            curves: vec![seed],
            circular: Circular {
                center: PointRef::Point(c),
                count: 4,
                angle: TAU,
            },
        }
        .apply(&mut s)
        .unwrap();
        assert!(crate::solve::drag(
            &mut s,
            &crate::solve::Drag::Points(vec![(c, Vec2::new(2.0, 0.0))]),
            &Default::default(),
        ));
        let o = s.pos(c);
        let sc = center_of(&s, seed);
        let first = center_of(&s, pattern(&s).1.copies[0].curves[0].1);
        let v = sc - o;
        let want = o + Vec2::new(-v.y, v.x);
        assert!((first - want).length() < 1e-6, "{first:?} {want:?}");
        // Deleting the center leaves the copies as they are, unlinked.
        SketchOp::Delete {
            curves: vec![],
            points: vec![c],
            dimensions: vec![],
            constraints: vec![],
        }
        .apply(&mut s)
        .unwrap();
        assert!(s.patterns.is_empty());
        assert_eq!(s.curves.len(), 4);
    }

    #[test]
    fn rejects_bad_circular_layouts() {
        let mut s = Sketch::default();
        let seed = circle(&mut s, 10.0, 0.0, 1.0);
        for (count, angle) in [(1, TAU), (4, 0.0), (4, 7.0), (3000, TAU)] {
            assert!(
                SketchOp::CircularPattern {
                    curves: vec![seed],
                    circular: Circular {
                        center: PointRef::Origin,
                        count,
                        angle,
                    },
                }
                .apply(&mut s)
                .is_err()
            );
        }
        assert!(s.patterns.is_empty());
    }
}
