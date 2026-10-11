//! A native board as parts of a Part Studio (`docs/PCB_IN_PART_STUDIO.md` §5): what the
//! Derived feature's rebuild makes from a board's layout, in the **board frame** of
//! [`crate::pcb::model`] (layout x and y in mm, the bottom face at z = 0).
//!
//! - **The board**: one kernel solid, the outline (with its cut-outs) extruded by the board's
//!   thickness, with every drilled hole of 0.5 mm or more through it ([`board_profile`],
//!   [`board_body`]). Its top and bottom faces are named as an extrude's caps, so a sketch or a
//!   mate on them survives outline edits; its sides by their place.
//! - **Components**: one mesh part per footprint with a visible model ([`components`]): the
//!   model's meshes made a [`Solid`] once per model ([`crate::pcb::mesh_solid`], cached for the
//!   process) and placed by [`crate::pcb::model::footprint_motion`]. Their faces are named in the
//!   model's order, so moving or turning a footprint keeps them.
//! - **Skins**: the board's top and bottom as made ([`cadrs_eda::skin`]) as pictures on its faces
//!   ([`skins`]); the PNGs go into the in-memory blob cache only (they are made again from the
//!   layout, never saved).

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use cadrs_eda::board::Board;
use cadrs_eda::layer::Side;
use cadrs_eda::units::to_mm;
use cadrs_kernel::naming::{EdgeName, FaceName, FaceOrigin, OpId, VertexName, stable_hash};
use cadrs_kernel::{BodyId, Curve2, Extent, Kernel, Loop, Motion, Plane, Profile, Region};
use nalgebra::{Point2, Point3, Vector3};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::solid::{Solid, SolidImage};

/// Holes smaller than this (mm across) aren't cut through the board (vias, fine pins): they
/// would only add faces.
pub const MIN_HOLE: f64 = 0.5;

/// Which components a derived board brings in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentFilter {
    #[default]
    All,
    Top,
    Bottom,
    None,
}

impl ComponentFilter {
    pub const ALL: [ComponentFilter; 4] = [ComponentFilter::All, ComponentFilter::Top, ComponentFilter::Bottom, ComponentFilter::None];

    pub fn label(self) -> &'static str {
        match self {
            ComponentFilter::All => "All components",
            ComponentFilter::Top => "Top side only",
            ComponentFilter::Bottom => "Bottom side only",
            ComponentFilter::None => "No components",
        }
    }

    pub fn takes(self, side: Side) -> bool {
        match self {
            ComponentFilter::All => true,
            ComponentFilter::Top => side == Side::Top,
            ComponentFilter::Bottom => side == Side::Bottom,
            ComponentFilter::None => false,
        }
    }
}

/// What a derived board shows: its 3D model, or its flat layout on its top face.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoardView {
    #[default]
    ThreeD,
    Layout,
}

/// Which face of the board goes onto the Derived feature's location.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoardSeat {
    /// The bottom face: the board grows along the location's +z (as an extrude from it).
    #[default]
    Bottom,
    /// The top (component) face: the board lies behind the location.
    Top,
}

/// Which point of the board goes onto the location (with Base origin).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoardAnchor {
    /// The middle of the outline's box.
    #[default]
    Center,
    /// The layout's origin (0, 0): a board made from a sketch has the sketch's coordinates.
    Origin,
}

/// A derived board's options.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardOptions {
    #[serde(default)]
    pub view: BoardView,
    #[serde(default)]
    pub components: ComponentFilter,
    /// Footprints left out.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<Uuid>,
    /// The board's faces show its mask, copper and silkscreen.
    #[serde(default = "yes")]
    pub skins: bool,
    #[serde(default)]
    pub seat: BoardSeat,
    #[serde(default)]
    pub anchor: BoardAnchor,
}

fn yes() -> bool {
    true
}

impl Default for BoardOptions {
    fn default() -> Self {
        BoardOptions { view: BoardView::ThreeD, components: ComponentFilter::All, exclude: vec![], skins: true, seat: BoardSeat::Bottom, anchor: BoardAnchor::Center }
    }
}

/// The board's thickness in mm.
pub fn thickness(b: &Board) -> f64 {
    to_mm(b.thickness)
}

/// The point (board frame) that goes onto the location: the anchor, on the seat face.
pub fn anchor_point(b: &Board, o: &BoardOptions) -> [f64; 3] {
    let (x, y) = match o.anchor {
        BoardAnchor::Origin => (0.0, 0.0),
        BoardAnchor::Center => {
            let r = cadrs_eda::outline::board_region(b);
            match r.iter().flatten().fold(None, |acc, p| Some(cadrs_eda::units::Bounds::union(acc, cadrs_eda::units::Bounds::of(*p)))) {
                Some(bb) => (to_mm((bb.min.x + bb.max.x) / 2), to_mm((bb.min.y + bb.max.y) / 2)),
                None => (0.0, 0.0),
            }
        }
    };
    let z = match o.seat {
        BoardSeat::Bottom => 0.0,
        BoardSeat::Top => thickness(b),
    };
    [x, y, z]
}

/// A converted IDF loop as exact kernel curves (lines, arcs from the included angle, circles).
/// Zero-length segments are dropped.
pub fn idf_loop_to_kernel(l: &cadrs_idf::Loop) -> Loop {
    use cadrs_idf::Segment;
    let p = |q: [f64; 2]| Point2::new(q[0], q[1]);
    let curves = l
        .segments()
        .filter_map(|s| match s {
            Segment::Line { start, end } => ((start[0] - end[0]).hypot(start[1] - end[1]) > 1e-12).then(|| Curve2::Line { a: p(start), b: p(end), source: None }),
            Segment::Arc { start, center, radius, sweep, .. } => (radius > 1e-12).then(|| Curve2::Arc {
                center: p(center),
                radius,
                start_angle: (start[1] - center[1]).atan2(start[0] - center[0]),
                sweep: sweep.to_radians(),
                source: None,
            }),
            Segment::Circle { center, radius } => (radius > 1e-12).then(|| Curve2::Circle { center: p(center), radius, source: None }),
        })
        .collect();
    Loop { curves }
}

/// An oval (slot) hole as a loop: two half circles joined by lines, `w` × `h` turned by
/// `angle` degrees about `c`.
fn slot(c: [f64; 2], w: f64, h: f64, angle: f64) -> Loop {
    let (long, short, extra) = if w >= h { (w, h, 0.0) } else { (h, w, 90.0) };
    let r = short / 2.0;
    let half = long / 2.0 - r;
    let a = (angle + extra).to_radians();
    let (dx, dy) = (a.cos() * half, a.sin() * half);
    let (nx, ny) = (-a.sin() * r, a.cos() * r);
    let p = |x: f64, y: f64| Point2::new(x, y);
    let (e1, e2) = (p(c[0] + dx, c[1] + dy), p(c[0] - dx, c[1] - dy));
    Loop {
        curves: vec![
            Curve2::Line { a: p(e2.x - nx, e2.y - ny), b: p(e1.x - nx, e1.y - ny), source: None },
            Curve2::Arc { center: e1, radius: r, start_angle: a - std::f64::consts::FRAC_PI_2, sweep: std::f64::consts::PI, source: None },
            Curve2::Line { a: p(e1.x + nx, e1.y + ny), b: p(e2.x + nx, e2.y + ny), source: None },
            Curve2::Arc { center: e2, radius: r, start_angle: a + std::f64::consts::FRAC_PI_2, sweep: std::f64::consts::PI, source: None },
        ],
    }
}

/// The board's plan as kernel regions (board frame, z = 0): the outline with its cut-outs and
/// the holes through it, and warnings for holes left out (outside the board, or overlapping
/// another hole or a cut-out's edge).
pub fn board_profile(b: &Board) -> Result<(Vec<Region>, Vec<String>), String> {
    let loops = crate::pcb::design::board_outline_loops(b);
    let Some((outer, cuts)) = loops.split_first() else { return Err("The board has no closed outline".into()) };
    let mut warnings = vec![];
    let region = cadrs_eda::outline::board_region(b);
    let inside = |x: f64, y: f64, r: f64| {
        // The hole and 0.05 mm round it within the board area.
        let disc = cadrs_eda::poly::circle(cadrs_eda::units::Pt::mm(x, y), cadrs_eda::units::mm(r + 0.05));
        cadrs_eda::poly::area(&cadrs_eda::poly::difference(&disc, &region)) < 1e6
    };
    let mut holes: Vec<Loop> = cuts.iter().map(idf_loop_to_kernel).collect();
    let mut placed: Vec<([f64; 2], f64)> = vec![];
    let mut dropped = 0;
    for h in cadrs_eda::fab::holes(b) {
        let (w, hh) = (to_mm(h.size.w), to_mm(h.size.h));
        if w.min(hh) < MIN_HOLE {
            continue;
        }
        let c = [to_mm(h.at.x), to_mm(h.at.y)];
        let r = w.max(hh) / 2.0;
        // A footprint waiting off the board: its holes aren't the board's.
        let disc = cadrs_eda::poly::circle(h.at, cadrs_eda::units::mm(r));
        if cadrs_eda::poly::area(&cadrs_eda::poly::intersection(&disc, &region)) < 1e6 {
            continue;
        }
        if !inside(c[0], c[1], r) || placed.iter().any(|(q, s)| (q[0] - c[0]).hypot(q[1] - c[1]) < r + s + 0.01) {
            dropped += 1;
            continue;
        }
        placed.push((c, r));
        holes.push(if (w - hh).abs() < 1e-6 {
            Loop { curves: vec![Curve2::Circle { center: Point2::new(c[0], c[1]), radius: w / 2.0, source: None }] }
        } else {
            slot(c, w, hh, h.angle)
        });
    }
    if dropped > 0 {
        warnings.push(format!("{dropped} hole{} not cut (outside the board or overlapping)", if dropped == 1 { "" } else { "s" }));
    }
    Ok((vec![Region { outer: idf_loop_to_kernel(outer), holes, source: None }], warnings))
}

/// The board as a kernel solid in the board frame, and warnings.
pub fn board_body(k: &mut dyn Kernel, b: &Board) -> Result<(BodyId, Vec<String>), String> {
    let (regions, warnings) = board_profile(b)?;
    let t = thickness(b).max(1e-3);
    let r = k.extrude(&Profile::new(Plane::top(), regions.clone()), Extent::Blind(t));
    match r {
        Ok(r) if !r.bodies.is_empty() => Ok((r.bodies[0], warnings)),
        // A hole the kernel refuses: try the outline alone, and say so.
        _ => {
            let bare = Region { outer: regions[0].outer.clone(), holes: vec![], source: None };
            let r = k.extrude(&Profile::new(Plane::top(), vec![bare]), Extent::Blind(t)).map_err(|e| format!("The board outline doesn't make a solid: {e}"))?;
            let body = r.bodies.into_iter().next().ok_or("The board outline doesn't make a solid")?;
            let mut w = warnings;
            w.push("The board's holes and cut-outs couldn't be cut".into());
            Ok((body, w))
        }
    }
}

/// Names for the board solid's faces (in the kernel's order) under `op`: the planar faces
/// facing +z and −z in the board frame `m` maps to the world are the top and bottom caps, the
/// rest are named by their place.
pub fn board_face_names(k: &dyn Kernel, body: BodyId, op: OpId, up: Vector3<f64>) -> Result<Vec<FaceName>, String> {
    let faces = k.faces(body).map_err(|e| e.to_string())?;
    let mut side = 0u32;
    Ok(faces
        .iter()
        .map(|f| {
            let n = f.plane.as_ref().map(|pl| pl.normal.dot(&up));
            match n {
                Some(d) if d > 0.999 => FaceName::new(op, FaceOrigin::Cap { region: 0, end: true }),
                Some(d) if d < -0.999 => FaceName::new(op, FaceOrigin::Cap { region: 0, end: false }),
                _ => {
                    side += 1;
                    FaceName::new(op, FaceOrigin::Imported { body: 0, face: side })
                }
            }
        })
        .collect())
}

/// A component of a derived board, in the board frame.
#[derive(Clone, Debug)]
pub struct Component {
    pub footprint: Uuid,
    /// "J1 AC inlet".
    pub name: String,
    pub reference: String,
    /// Its mesh part in its footprint frame (faces named under the nil op).
    pub solid: Arc<Solid>,
    /// Footprint frame → board frame.
    pub motion: Motion,
}

type SolidKey = u64;

fn solid_cache() -> &'static RwLock<HashMap<SolidKey, Arc<Solid>>> {
    static CACHE: OnceLock<RwLock<HashMap<SolidKey, Arc<Solid>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// The mesh part of a footprint's visible model, made once per model (by its file, generated
/// body, placement in the footprint and opacity).
fn footprint_solid(fp: &cadrs_eda::footprint::Footprint) -> Result<Option<Arc<Solid>>, String> {
    let Some(m) = crate::pcb::model::shown_model(fp) else { return Ok(None) };
    let key = stable_hash(format!("{:?}{:?}{:?}{:?}{}{:?}", m.blob, m.offset, m.rotation, m.scale, m.opacity, m.body).as_bytes());
    if let Some(s) = solid_cache().read().ok().and_then(|c| c.get(&key).cloned()) {
        return Ok(Some(s));
    }
    let Some((meshes, opacity)) = crate::pcb::model::footprint_meshes(fp)? else { return Ok(None) };
    let s = Arc::new(crate::pcb::mesh_solid::mesh_solid(&meshes, Uuid::nil(), opacity));
    if let Ok(mut c) = solid_cache().write() {
        c.insert(key, s.clone());
    }
    Ok(Some(s))
}

/// The value field of a footprint ("10k"), empty when it has none.
fn value_of(fp: &cadrs_eda::footprint::Footprint) -> String {
    fp.field(cadrs_eda::symbol::fields::VALUE).map(|v| v.text.text.text.clone()).unwrap_or_default()
}

/// The components `o` takes from `b`, with warnings for models that can't be shown.
pub fn components(b: &Board, o: &BoardOptions) -> (Vec<Component>, Vec<String>) {
    let t = thickness(b);
    let mut out = vec![];
    let mut warnings = vec![];
    for f in &b.footprints {
        if !o.components.takes(f.placement.side) || o.exclude.contains(&f.id) {
            continue;
        }
        let reference = f.reference().to_string();
        match footprint_solid(&f.footprint) {
            Ok(Some(solid)) => {
                let value = value_of(&f.footprint);
                let name = if value.is_empty() { reference.clone() } else { format!("{reference} {value}") };
                out.push(Component { footprint: f.id, name, reference, solid, motion: crate::pcb::model::footprint_motion(&f.placement, t) });
            }
            Ok(None) => {}
            Err(e) => warnings.push(format!("{reference}: {e}")),
        }
    }
    (out, warnings)
}

/// `s` with every face, edge and vertex name moved to the operation `op` (a component's mesh
/// part is made under the nil op; the Derived feature names it after its footprint, then as a
/// copy, [`crate::derived::derived_face`]).
pub fn renamed(s: &Solid, op: OpId) -> Solid {
    renamed_by(s, |n| FaceName { op, ..n })
}

/// `s` with every face name passed through `f` (edge and vertex names follow their faces).
pub fn renamed_by(s: &Solid, f: impl Fn(FaceName) -> FaceName) -> Solid {
    let mut out = s.clone();
    for face in &mut out.faces {
        face.name = f(face.name);
    }
    for e in &mut out.edges {
        e.name = EdgeName::new(f(e.name.faces[0]), f(e.name.faces[1]), e.name.index);
    }
    for v in &mut out.vertices {
        v.name = VertexName { faces: v.name.faces.map(&f), ..v.name };
    }
    for l in &mut out.looks {
        l.0 = f(l.0);
    }
    out
}

/// `s` moved by the rigid motion `m`.
pub fn moved(s: &Solid, m: &Motion) -> Solid {
    let r = m.linear;
    let rows = [[r[(0, 0)], r[(0, 1)], r[(0, 2)]], [r[(1, 0)], r[(1, 1)], r[(1, 2)]], [r[(2, 0)], r[(2, 1)], r[(2, 2)]]];
    let pose = crate::assembly::Pose { rotation: rows, translation: [m.translation.x, m.translation.y, m.translation.z] };
    crate::assembly::transform_solid(s, &pose)
}

fn skin_cache() -> &'static RwLock<HashMap<(u64, bool), Option<(String, [f64; 4])>>> {
    static CACHE: OnceLock<RwLock<HashMap<(u64, bool), Option<(String, [f64; 4])>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// Pixels per mm of a board's skin.
pub const SKIN_PX_PER_MM: f32 = 24.0;

/// The skin of one face of `b` as a blob (made once per layout `hash` and side): its hash and
/// the box it covers (x0, y0, w, h mm).
fn skin_blob(b: &Board, hash: u64, side: Side) -> Option<(String, [f64; 4])> {
    let key = (hash, side == Side::Top);
    if let Some(s) = skin_cache().read().ok().and_then(|c| c.get(&key).cloned()) {
        return s;
    }
    let made = cadrs_eda::skin::board_skin(b, side, SKIN_PX_PER_MM, 4096).and_then(|s| {
        let png = s.png()?;
        let bb = s.bounds;
        Some((crate::blobs::insert(png), [to_mm(bb.min.x), to_mm(bb.min.y), to_mm(bb.size().w), to_mm(bb.size().h)]))
    });
    if let Ok(mut c) = skin_cache().write() {
        c.insert(key, made.clone());
    }
    made
}

/// The board's top and bottom skins as pictures in the board frame, a hair off each face (the
/// layout `hash` keys the cache).
pub fn skins(b: &Board, hash: u64) -> Vec<SolidImage> {
    let t = thickness(b);
    let mut out = vec![];
    for (side, z) in [(Side::Top, t + 0.01), (Side::Bottom, -0.01)] {
        let Some((blob, [x0, y0, w, h])) = skin_blob(b, hash, side) else { continue };
        out.push(SolidImage { blob, corner: [x0, y0, z], u: [w, 0.0, 0.0], v: [0.0, h, 0.0] });
    }
    out
}

/// A stable hash of a layout (the Derived feature's cache key sees it instead of the layout).
pub fn layout_hash(b: &Board) -> u64 {
    // Debug prints every field (floats in full); the FNV hash doesn't change between runs.
    stable_hash(format!("{b:?}").as_bytes())
}

/// The point `p` (board frame) moved by `m`.
pub fn apply(m: &Motion, p: [f64; 3]) -> [f64; 3] {
    let q = m.linear * Point3::new(p[0], p[1], p[2]).coords + m.translation;
    [q.x, q.y, q.z]
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadrs_eda::units::mm;

    fn course_board() -> Board {
        let lib = cadrs_eda::library::LibraryTable::builtin();
        cadrs_eda::getting_started::gs18(&lib).board
    }

    #[test]
    fn the_profile_has_the_outline_and_the_holes() {
        let b = course_board();
        let (regions, warnings) = board_profile(&b).unwrap();
        assert_eq!(regions.len(), 1);
        // The course board's through-hole parts: BT1's two pins, D1's two, R1's two.
        let holes = cadrs_eda::fab::holes(&b).iter().filter(|h| to_mm(h.size.w.min(h.size.h)) >= MIN_HOLE).count();
        let all: Vec<f64> = cadrs_eda::fab::holes(&b).iter().map(|h| to_mm(h.size.w)).collect();
        assert_eq!(regions[0].holes.len(), holes, "{warnings:?} {all:?}");
        assert!(holes >= 4, "{all:?}");
    }

    #[test]
    fn components_are_placed_where_their_footprints_are() {
        let b = course_board();
        let (cs, warnings) = components(&b, &BoardOptions::default());
        assert!(warnings.is_empty(), "{warnings:?}");
        let refs: Vec<&str> = cs.iter().map(|c| c.reference.as_str()).collect();
        assert_eq!(refs.len(), 3, "{refs:?}");
        // BT1 is on the bottom: its model hangs below the board.
        let bt = cs.iter().find(|c| c.reference == "BT1").unwrap();
        let s = moved(&bt.solid, &bt.motion);
        assert!(s.positions.iter().all(|p| p[2] <= 1e-6), "BT1 below the board");
        let d1 = cs.iter().find(|c| c.reference == "D1").unwrap();
        let s = moved(&d1.solid, &d1.motion);
        // A through-hole LED: its body stands on the top face, its leads go down through it.
        let zmax = s.positions.iter().map(|p| p[2]).fold(f64::MIN, f64::max);
        assert!(zmax > 1.6 + 3.0, "D1 on top: {zmax}");
        // Filters.
        let top = BoardOptions { components: ComponentFilter::Top, ..Default::default() };
        assert_eq!(components(&b, &top).0.len(), 2);
        let ex = BoardOptions { exclude: vec![d1.footprint], ..Default::default() };
        assert!(components(&b, &ex).0.iter().all(|c| c.reference != "D1"));
    }

    #[test]
    fn anchors_and_seats() {
        let b = Board::with_rect_outline(mm(40.0), mm(20.0));
        assert_eq!(anchor_point(&b, &BoardOptions::default()), [20.0, 10.0, 0.0]);
        let o = BoardOptions { anchor: BoardAnchor::Origin, seat: BoardSeat::Top, ..Default::default() };
        assert_eq!(anchor_point(&b, &o), [0.0, 0.0, 1.6]);
    }

    #[test]
    fn renaming_moves_every_name() {
        let mut body = cadrs_eda::model3d::Body::default();
        body.cuboid([0.0; 3], [1.0; 3], [1, 1, 1]);
        let s = crate::pcb::mesh_solid::mesh_solid(&cadrs_eda::model3d::mesh(&body), Uuid::nil(), 1.0);
        let op = Uuid::from_u128(7);
        let r = renamed(&s, op);
        assert!(r.faces.iter().all(|f| f.name.op == op));
        assert!(r.edges.iter().all(|e| e.name.faces.iter().all(|f| f.op == op)));
        assert!(r.looks.iter().all(|l| l.0.op == op));
    }

    #[test]
    fn slots_are_closed() {
        let l = slot([1.0, 2.0], 3.0, 1.0, 30.0);
        let ends: Vec<(Point2<f64>, Point2<f64>)> = l
            .curves
            .iter()
            .map(|c| match c {
                Curve2::Line { a, b, .. } => (*a, *b),
                Curve2::Arc { center, radius, start_angle, sweep, .. } => {
                    let at = |t: f64| Point2::new(center.x + radius * t.cos(), center.y + radius * t.sin());
                    (at(*start_angle), at(start_angle + sweep))
                }
                _ => unreachable!(),
            })
            .collect();
        for i in 0..4 {
            let (a, b) = (ends[i].1, ends[(i + 1) % 4].0);
            assert!((a - b).norm() < 1e-9, "{i}: {a:?} {b:?}");
        }
    }
}
