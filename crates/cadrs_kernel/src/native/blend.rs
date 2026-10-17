//! Fillets and chamfers as exact tools combined by booleans.
//!
//! For each edge, the cross-section of what the blend removes (a convex edge) or adds (a
//! concave one) is built in the plane square to the edge: the corner, the two contact points
//! on the faces and the section curve between them (a circular arc for a fillet, a straight
//! line for a chamfer, a conic or a quintic). The section is swept along a straight edge
//! between two planes (exact planes and a cylinder), or turned about the axis of a circular
//! edge between a plane and a coaxial surface of revolution (a torus), and extended past the
//! edge's ends, where the end faces trim it. Where three filleted edges between planes meet,
//! the corner is the ball rolling on all three faces: a corner tool (the block between the
//! faces and the ball's centre, less the ball) leaves the exact sphere octant.

use super::boolean::{boolean_bodies, BoolOut};
use super::curve::Curve;
use super::kernel::NativeKernel;
use super::math::*;
use super::nurbs::NurbsCurve;
use super::profile::{MergedFace, Piece, Src};
use super::surface::Surface;
use super::sweep::{self, Prism, Revolve};
use super::topo::{co_range, Body};
use crate::*;

/// What each face of a body continues: an input face, or a new face with an origin.
#[derive(Debug, Clone, PartialEq)]
pub enum FaceSrc {
    Input(usize),
    New(Origin),
}

/// A body with the origin of every face.
#[derive(Debug, Clone)]
pub struct Tracked {
    pub body: Body,
    pub src: Vec<Vec<FaceSrc>>,
}

impl Tracked {
    pub fn of(b: Body) -> Self {
        let n = b.faces.len();
        Self { body: b, src: (0..n).map(|i| vec![FaceSrc::Input(i)]).collect() }
    }

    /// Combines with a tool whose faces carry `tool_src`.
    pub fn apply(&self, tool: &Body, tool_src: &[Vec<FaceSrc>], op: BoolOp) -> Result<Tracked> {
        let r: BoolOut = boolean_bodies(&self.body, tool, op)?;
        let src = r
            .from
            .iter()
            .map(|srcs| {
                let mut out: Vec<FaceSrc> = Vec::new();
                for &(o, f) in srcs {
                    let from = if o == 0 { &self.src[f] } else { &tool_src[f] };
                    for x in from {
                        if !out.contains(x) {
                            out.push(x.clone());
                        }
                    }
                }
                out
            })
            .collect();
        Ok(Tracked { body: r.body, src })
    }

    pub fn into_result(self, k: &mut NativeKernel, input: BodyId) -> OpResult {
        let mut history = History::default();
        let n = k.get(input).map(|b| b.faces.len()).unwrap_or(0);
        let mut seen = vec![false; n];
        for (f, srcs) in self.src.iter().enumerate() {
            for s in srcs {
                match s {
                    FaceSrc::Input(i) => {
                        history.modified.push((FaceId(f as u64), InputFace { body: input, face: FaceId(*i as u64) }));
                        if *i < n {
                            seen[*i] = true;
                        }
                    }
                    FaceSrc::New(o) => history.generated.push((FaceId(f as u64), *o)),
                }
            }
        }
        for (i, s) in seen.iter().enumerate() {
            if !s {
                history.deleted.push(InputFace { body: input, face: FaceId(i as u64) });
            }
        }
        let id = k.insert(self.body);
        OpResult { bodies: vec![id], history }
    }
}

/// The section curve of a blend between the contact points.
#[derive(Debug, Clone, Copy)]
pub enum SectionKind {
    /// A circular fillet of radius r (contact distances from the faces' angle).
    Circle { r: f64 },
    /// Straight, with the contact distances on the first and second face.
    Chamfer { d1: f64, d2: f64 },
    /// A conic with the circle's contact points for radius r and shape `rho`.
    Conic { r: f64, rho: f64 },
    /// A curvature-continuous quintic with the circle's contact points.
    Curvature { r: f64, m: f64 },
    /// Contact points of the circle for r1 (first face) and r2 (second), the circle's weight.
    Asymmetric { r1: f64, r2: f64 },
}

/// The geometry of an edge's blend in the plane square to it, in that plane's 2D coordinates:
/// the corner at the origin, the unit directions into the faces and whether the edge is
/// convex.
#[derive(Debug, Clone, Copy)]
pub struct Corner2 {
    pub u1: V2,
    pub u2: V2,
    pub convex: bool,
}

/// The loop (in 2D) of the region a blend removes (convex) or adds (concave), and the index of
/// the section piece within it.
pub fn section_loop(c: &Corner2, kind: SectionKind, src: Src) -> Result<(Vec<Piece>, P2, P2)> {
    let cos = c.u1.dot(&c.u2).clamp(-1.0, 1.0);
    let theta = cos.acos();
    if theta < 1e-6 || theta > PI - 1e-6 {
        return Err(KernelError::OperationFailed("the faces at the edge are tangent: nothing to blend".into()));
    }
    let bis = (c.u1 + c.u2).normalize();
    let tan_half = (theta / 2.0).tan();
    let (a1, a2) = match kind {
        SectionKind::Circle { r } | SectionKind::Conic { r, .. } | SectionKind::Curvature { r, .. } => (r / tan_half, r / tan_half),
        SectionKind::Chamfer { d1, d2 } => (d1, d2),
        SectionKind::Asymmetric { r1, r2 } => (r1 / tan_half, r2 / tan_half),
    };
    let p = P2::origin();
    let c1 = p + c.u1 * a1;
    let c2 = p + c.u2 * a2;
    let pt = |q: P2| P3::new(q.x, q.y, 0.0);
    let section: Curve = match kind {
        SectionKind::Circle { r } => {
            let center = p + bis * (r / (theta / 2.0).sin());
            let x = (c1 - center).normalize();
            let to2 = (c2 - center).normalize();
            // The short arc from c1 to c2.
            let y0 = V2::new(-x.y, x.x);
            let y = if y0.dot(&to2) >= 0.0 { y0 } else { -y0 };
            let sweep = to2.dot(&y).atan2(to2.dot(&x));
            let circle = Curve::Circle { c: pt(center), x: V3::new(x.x, x.y, 0.0), y: V3::new(y.x, y.y, 0.0), r };
            return finish(circle, 0.0, sweep, c1, c2, c, src);
        }
        SectionKind::Chamfer { .. } => {
            let d = c2 - c1;
            let l = d.norm();
            return finish(Curve::Line { o: pt(c1), d: V3::new(d.x / l, d.y / l, 0.0) }, 0.0, l, c1, c2, c, src);
        }
        SectionKind::Conic { rho, .. } => {
            if !(rho > 0.0 && rho < 1.0) {
                return Err(KernelError::InvalidParameter("The conic's rho must lie between 0 and 1".into()));
            }
            Curve::Nurbs(std::sync::Arc::new(NurbsCurve::bezier(&[pt(c1), pt(p), pt(c2)], &[1.0, rho / (1.0 - rho), 1.0])))
        }
        SectionKind::Asymmetric { .. } => {
            let w = (theta / 2.0).sin();
            Curve::Nurbs(std::sync::Arc::new(NurbsCurve::bezier(&[pt(c1), pt(p), pt(c2)], &[1.0, w, 1.0])))
        }
        SectionKind::Curvature { m, .. } => {
            if !(m > 0.0 && m <= 1.0) {
                return Err(KernelError::InvalidParameter("The curvature magnitude must lie in (0, 1]".into()));
            }
            let a = |q: P2, f: f64| q + (p - q) * f;
            let poles = [c1, a(c1, m / 2.0), a(c1, m), a(c2, m), a(c2, m / 2.0), c2].map(pt);
            Curve::Nurbs(std::sync::Arc::new(NurbsCurve::bezier(&poles, &[1.0; 6])))
        }
    };
    finish(section, 0.0, 1.0, c1, c2, c, src)
}

fn finish(section: Curve, t0: f64, t1: f64, c1: P2, c2: P2, c: &Corner2, src: Src) -> Result<(Vec<Piece>, P2, P2)> {
    let (t0, t1, rev) = if t1 >= t0 { (t0, t1, false) } else { (t1, t0, true) };
    let pt = |q: P2| P3::new(q.x, q.y, 0.0);
    let bis = (c.u1 + c.u2).normalize();
    let size = (c1.coords.norm()).max(c2.coords.norm());
    // The corner pushed out of the wedge, so no tool face lies on a body face.
    let pc = P2::origin() - bis * (size * 0.5 + 1e-3);
    let line = |a: P2, b: P2| {
        let d = b - a;
        let l = d.norm();
        Piece { c: Curve::Line { o: pt(a), d: V3::new(d.x / l, d.y / l, 0.0) }, t0: 0.0, t1: l, rev: false, src: Src::Curve { region: u64::MAX, curve: u64::MAX }, more: vec![] }
    };
    let sec = Piece { c: section, t0, t1, rev, src, more: vec![] };
    let mut lp = vec![line(pc, c1), sec, line(c2, pc)];
    if super::profile::loop_area(&lp) < 0.0 {
        lp = super::profile::reverse_loop(&lp);
    }
    Ok((lp, c1, c2))
}

/// The faces on either side of an edge, with the coedges.
fn edge_sides(b: &Body, e: usize) -> Result<[(usize, bool); 2]> {
    let mut out = Vec::new();
    for (fi, f) in b.faces.iter().enumerate() {
        for l in &f.loops {
            for c in l {
                if c.edge == e {
                    out.push((fi, c.fwd));
                }
            }
        }
    }
    if out.len() != 2 || out[0].0 == out[1].0 {
        return Err(KernelError::InvalidParameter("Only edges between two faces can be blended".into()));
    }
    Ok([out[0], out[1]])
}

/// The outward normal and the unit direction into the face, square to the edge, at the edge's
/// parameter t (the coedge's p-curve gives the face's uv there).
fn face_dirs(b: &Body, fi: usize, e: usize, fwd: bool, t: f64) -> (V3, V3) {
    let f = &b.faces[fi];
    let c = f.loops.iter().flatten().find(|c| c.edge == e && c.fwd == fwd).unwrap();
    let uv = c.pc.point(t);
    let n = f.surf.normal(uv.x, uv.y) * if f.sense { 1.0 } else { -1.0 };
    let d = b.edges[e].curve.tangent(t).normalize() * if fwd { 1.0 } else { -1.0 };
    (n, n.cross(&d))
}

/// An edge's blend tool and whether it is removed (convex) or added (concave).
pub struct Tool {
    pub body: Body,
    pub src: Vec<Vec<FaceSrc>>,
    pub convex: bool,
    /// (corner point, faces' outward normals, ball centre) at each end, for corners.
    pub ends: [(P3, V3, V3); 2],
}

/// Builds the tool of one edge. `kind_at` gives the section for the edge's faces' angle
/// (normals φ apart) and which face is first.
pub fn edge_tool(b: &Body, input: BodyId, e: usize, kind: &dyn Fn(f64) -> SectionKind, swap: bool) -> Result<Tool> {
    let ed = &b.edges[e];
    if ed.degenerate {
        return Err(KernelError::InvalidParameter("A degenerate edge can't be blended".into()));
    }
    let [mut s1, mut s2] = edge_sides(b, e)?;
    if swap {
        std::mem::swap(&mut s1, &mut s2);
    }
    let tm = ed.mid_t();
    let (n1, w1) = face_dirs(b, s1.0, e, s1.1, tm);
    let (n2, w2) = face_dirs(b, s2.0, e, s2.1, tm);
    let convex = w2.dot(&n1) < 0.0;
    let phi = n1.dot(&n2).clamp(-1.0, 1.0).acos();
    let section = kind(phi);
    let src = Src::Edge { body: input.0, edge: e as u64 };
    let (f1, f2) = (&b.faces[s1.0].surf, &b.faces[s2.0].surf);
    match (&ed.curve, f1, f2) {
        (Curve::Line { d, .. }, Surface::Plane { .. }, Surface::Plane { .. }) => {
            let t = d.normalize();
            // Section plane square to the edge at its start point.
            let p0 = ed.point(ed.t[0]);
            let x = w1;
            let y = t.cross(&x);
            let plane = Plane { origin: p0, x_dir: nalgebra::Unit::new_normalize(x), normal: nalgebra::Unit::new_normalize(t) };
            let to2 = |v: &V3| V2::new(v.dot(&x), v.dot(&y));
            let corner = Corner2 { u1: to2(&w1).normalize(), u2: to2(&w2).normalize(), convex };
            let (lp, _, _) = section_loop(&corner, section, src)?;
            let len = (ed.t[1] - ed.t[0]) * d.norm();
            let size = lp.iter().map(|p| p.start().coords.norm()).fold(0.0, f64::max);
            let ext = size * 2.0 + 1.0;
            let mf = MergedFace { outer: lp, holes: vec![], regions: vec![u64::MAX] };
            let kind_ = Prism { plane, dir: t };
            let built = sweep::sweep_full(&[mf], &[], &kind_, -ext, len + ext, false, true)?;
            let src = tool_src(&built);
            let ends = corner_info(b, ed, &n1, &n2, &w1, &w2, section);
            Ok(Tool { body: built.body, src, convex, ends })
        }
        (Curve::Circle { c, x: cx, y: cy, .. }, _, _) => {
            // A rim: one face a plane square to the axis, the other a coaxial surface of
            // revolution; the section is the same all round, so the tool is a revolve.
            let axis_dir = cx.cross(cy).normalize();
            let coaxial = |s: &Surface| match s {
                Surface::Plane { f } => f.x.cross(&f.y).cross(&axis_dir).norm() < 1e-9,
                _ => s.axis().is_some_and(|(o, z)| z.cross(&axis_dir).norm() < 1e-9 && (o - c).cross(&axis_dir).norm() < 1e-7),
            };
            if !coaxial(f1) || !coaxial(f2) {
                return Err(KernelError::Unsupported("blends of this edge (only straight edges between planes and circular rims)"));
            }
            let q = ed.point(ed.t[0]);
            let t = ed.curve.tangent(ed.t[0]).normalize();
            let (n1, w1) = face_dirs(b, s1.0, e, s1.1, ed.t[0]);
            let (n2, w2) = face_dirs(b, s2.0, e, s2.1, ed.t[0]);
            let _ = (n1, n2);
            // Meridian plane through q: x radial, normal along the tangent.
            let radial = (q - c).normalize();
            let plane = Plane { origin: q, x_dir: nalgebra::Unit::new_normalize(radial), normal: nalgebra::Unit::new_normalize(t) };
            let y = t.cross(&radial);
            let to2 = |v: &V3| V2::new(v.dot(&radial), v.dot(&y));
            let corner = Corner2 { u1: to2(&w1).normalize(), u2: to2(&w2).normalize(), convex };
            let (lp, _, _) = section_loop(&corner, section, src)?;
            let mf = MergedFace { outer: lp, holes: vec![], regions: vec![u64::MAX] };
            // Revolve frame: z the axis, x towards the section.
            let f = Frame { o: *c, x: radial, y: axis_dir.cross(&radial), z: axis_dir };
            let kind_ = Revolve { plane, f };
            let closed = ed.closed();
            let span = (ed.t[1] - ed.t[0]).abs();
            let (s0, s1, whole) = if closed { (0.0, TAU, true) } else {
                // The angle the edge turns about the axis, from q.
                let sgn = if t.dot(&f.y) > 0.0 { 1.0 } else { -1.0 };
                let ext = 0.2f64.min((TAU - span) / 2.0 - 1e-3).max(0.0);
                if sgn > 0.0 { (-ext, span + ext, false) } else { (-span - ext, ext, false) }
            };
            let built = sweep::sweep_full(&[mf], &[], &kind_, s0, s1, whole, true)?;
            let src = tool_src(&built);
            let z = (q, V3::zeros(), V3::zeros());
            Ok(Tool { body: built.body, src, convex, ends: [z, z] })
        }
        _ => Err(KernelError::Unsupported("blends of this edge (only straight edges between planes and circular rims)")),
    }
}

fn tool_src(built: &super::build::Built) -> Vec<Vec<FaceSrc>> {
    let mut src = vec![Vec::new(); built.body.faces.len()];
    for (f, o) in &built.origins {
        if let Origin::FromEdge { .. } = o {
            src[*f].push(FaceSrc::New(*o));
        }
    }
    src
}

fn corner_info(b: &Body, ed: &super::topo::Edge, _n1: &V3, _n2: &V3, _w1: &V3, _w2: &V3, _s: SectionKind) -> [(P3, V3, V3); 2] {
    let p0 = b.verts[ed.v[0]].p;
    let p1 = b.verts[ed.v[1]].p;
    [(p0, V3::zeros(), V3::zeros()), (p1, V3::zeros(), V3::zeros())]
}

/// The corner tool at a vertex where three filleted edges between planes meet with the same
/// radius: the block between the faces and the ball centre, less the ball.
pub fn corner_tool(b: &Body, v: usize, r: f64, vertex_origin: Origin) -> Result<Option<(Body, Vec<Vec<FaceSrc>>)>> {
    // The planar faces around the vertex.
    let p = b.verts[v].p;
    let mut normals: Vec<V3> = Vec::new();
    for f in &b.faces {
        let touches = f.loops.iter().flatten().any(|c| b.edges[c.edge].v.contains(&v));
        if !touches {
            continue;
        }
        let Surface::Plane { f: fr } = &f.surf else { return Ok(None) };
        let n = fr.x.cross(&fr.y) * if f.sense { 1.0 } else { -1.0 };
        if !normals.iter().any(|m| m.dot(&n) > 1.0 - 1e-9) {
            normals.push(n);
        }
    }
    if normals.len() != 3 {
        return Ok(None);
    }
    // Ball centre: n_i·(c − p) = −r.
    let m = nalgebra::Matrix3::from_rows(&[normals[0].transpose(), normals[1].transpose(), normals[2].transpose()]);
    let Some(inv) = m.try_inverse() else { return Ok(None) };
    let c = p + inv * V3::new(-r, -r, -r);
    // The block: between the offset planes through c and planes pushed out past p.
    let push = r * 0.5 + 1e-3;
    let corner = |s: [bool; 3]| -> P3 {
        // s[i]: on the outer plane (n·(x − p) = push) or the inner one (n·(x − c) = 0).
        let rhs = V3::new(
            if s[0] { normals[0].dot(&p.coords) + push } else { normals[0].dot(&c.coords) },
            if s[1] { normals[1].dot(&p.coords) + push } else { normals[1].dot(&c.coords) },
            if s[2] { normals[2].dot(&p.coords) + push } else { normals[2].dot(&c.coords) },
        );
        P3::from(inv * rhs)
    };
    let block = hexahedron(&[
        corner([false, false, false]),
        corner([true, false, false]),
        corner([true, true, false]),
        corner([false, true, false]),
        corner([false, false, true]),
        corner([true, false, true]),
        corner([true, true, true]),
        corner([false, true, true]),
    ])?;
    // The ball's axis along one face normal and its seam away from the corner, so the octant's
    // boundary arcs are its equator and two meridians (exact p-curves).
    let d = p - c;
    let z = normals[0];
    let side = d - z * d.dot(&z);
    let ball = if side.norm() > 1e-9 { ball_frame(c, r, z, -side.normalize())? } else { ball_towards(c, r, d)? };
    let r = boolean_bodies(&block, &ball, BoolOp::Subtract)?;
    let src = r
        .from
        .iter()
        .map(|s| if s.iter().any(|(o, _)| *o == 1) { vec![FaceSrc::New(vertex_origin)] } else { vec![] })
        .collect();
    Ok(Some((r.body, src)))
}

/// A solid ball; its seam and poles face away from the direction `away` (so a tool that keeps
/// the part of the sphere towards `away` has no seam or pole in it).
pub fn ball_body(c: P3, r: f64) -> Result<Body> {
    ball_towards(c, r, V3::z())
}

pub fn ball_towards(c: P3, r: f64, away: V3) -> Result<Body> {
    let d = away.normalize();
    ball_frame(c, r, any_perpendicular(&d), -d)
}

/// A ball with its axis along `z` and its seam (u = 0) towards `x` (square to z).
pub fn ball_frame(c: P3, r: f64, z: V3, x: V3) -> Result<Body> {
    let y = z.cross(&x);
    let plane = Plane { origin: c, x_dir: nalgebra::Unit::new_normalize(x), normal: nalgebra::Unit::new_normalize(-y) };
    let half = Region {
        outer: Loop {
            curves: vec![
                Curve2::Arc { center: nalgebra::Point2::new(0.0, 0.0), radius: r, start_angle: -PI / 2.0, sweep: PI, source: None },
                Curve2::Line { a: nalgebra::Point2::new(0.0, r), b: nalgebra::Point2::new(0.0, -r), source: None },
            ],
        },
        holes: vec![],
        source: None,
    };
    let prof = Profile::new(plane, vec![half]);
    let regions = super::profile::regions(&prof)?;
    let mf = super::profile::merge_regions(&regions)?;
    let kind = Revolve { plane, f: Frame { o: c, x, y, z } };
    Ok(sweep::sweep_full(&mf, &[], &kind, 0.0, TAU, true, true)?.body)
}

/// A hexahedron with planar faces from its 8 corners: 0–3 the bottom quad, 4–7 the top one
/// above them in order.
pub fn hexahedron(c: &[P3; 8]) -> Result<Body> {
    let mut b = Body::default();
    for p in c {
        b.add_vertex(*p);
    }
    // Orient so the bottom is counter-clockwise seen from below (outward) when the top is above.
    let quads: [[usize; 4]; 6] = [[0, 3, 2, 1], [4, 5, 6, 7], [0, 1, 5, 4], [1, 2, 6, 5], [2, 3, 7, 6], [3, 0, 4, 7]];
    let mut edges: std::collections::HashMap<(usize, usize), usize> = Default::default();
    let mut faces = Vec::new();
    for q in quads {
        let mut lp = Vec::new();
        let n = (c[q[1]] - c[q[0]]).cross(&(c[q[2]] - c[q[0]]));
        let f = Frame::from_xz(c[q[0]], c[q[1]] - c[q[0]], n);
        let surf = Surface::Plane { f };
        for i in 0..4 {
            let (a, bb) = (q[i], q[(i + 1) % 4]);
            let key = (a.min(bb), a.max(bb));
            let e = *edges.entry(key).or_insert_with(|| {
                let (line, len) = Curve::line(c[key.0], c[key.1]);
                b.add_edge(line, 0.0, len, key.0, key.1)
            });
            let fwd = a == key.0;
            let ed = &b.edges[e];
            let pc = super::pcurve::pcurve(&surf, &ed.curve, ed.t[0], ed.t[1], None);
            lp.push(super::topo::Coedge { edge: e, fwd, pc });
        }
        faces.push(super::topo::Face { surf, sense: true, loops: vec![lp] });
    }
    b.faces = faces;
    b.shells.push(super::topo::Shell { faces: (0..6).collect(), closed: true });
    b.solids.push(super::topo::Solid { shells: vec![0] });
    // Outward: flip everything if the volume came out negative.
    if super::mass::body_mass(&b).volume < 0.0 {
        for f in &mut b.faces {
            Body::reverse_face(f);
        }
    }
    let _ = co_range;
    Ok(b)
}

/// Keeps the part of a tool on the side of the plane (through `o`, normal `n`) that `n`
/// points away from.
fn trim_tool(tool: &Body, src: &[Vec<FaceSrc>], o: P3, n: V3) -> Result<(Body, Vec<Vec<FaceSrc>>)> {
    let bb = tool.bbox();
    let size = bb.diagonal() * 2.0 + (bb.center() - o).norm() * 2.0 + 10.0;
    let f = Frame::from_z(o, n);
    let plane = Plane { origin: o, x_dir: nalgebra::Unit::new_normalize(f.x), normal: nalgebra::Unit::new_normalize(n) };
    let p2 = |x: f64, y: f64| nalgebra::Point2::new(x, y);
    let l = |a, b| Curve2::Line { a, b, source: None };
    let sq = Region {
        outer: Loop { curves: vec![l(p2(-size, -size), p2(size, -size)), l(p2(size, -size), p2(size, size)), l(p2(size, size), p2(-size, size)), l(p2(-size, size), p2(-size, -size))] },
        holes: vec![],
        source: None,
    };
    let prof = Profile::new(plane, vec![sq]);
    let regions = super::profile::regions(&prof)?;
    let mf = super::profile::merge_regions(&regions)?;
    let kind = Prism { plane, dir: n };
    let hs = sweep::sweep(&plane, &mf, &kind, 0.0, size, false)?.body;
    let t = Tracked { body: tool.clone(), src: src.to_vec() };
    let r = t.apply(&hs, &vec![Vec::new(); hs.faces.len()], BoolOp::Subtract)?;
    Ok((r.body, r.src))
}

/// The section kind for a fillet spec on an edge whose faces' normals are `phi` apart.
fn fillet_section(spec: &FilletSpec, phi: f64) -> SectionKind {
    let r = match spec.size {
        FilletSize::Radius(r) => r,
        FilletSize::Width(w) => w / (2.0 * (phi / 2.0).sin()),
    };
    match spec.profile {
        FilletProfile::Circular => SectionKind::Circle { r },
        FilletProfile::Conic { rho } => SectionKind::Conic { r, rho },
        FilletProfile::Curvature { magnitude } => SectionKind::Curvature { r, m: magnitude },
        FilletProfile::Asymmetric { second, flip } => {
            if flip {
                SectionKind::Asymmetric { r1: second, r2: r }
            } else {
                SectionKind::Asymmetric { r1: r, r2: second }
            }
        }
    }
}

/// Blends the edges (each with its section) and the corners where three circular fillets of
/// one radius meet between planes.
pub fn blend(k: &mut NativeKernel, body: BodyId, edges: &[(EdgeId, Box<dyn Fn(f64) -> SectionKind>, bool)], corners: bool) -> Result<OpResult> {
    let b = (**k.get(body)?).clone();
    let picked: Vec<usize> = edges.iter().map(|(e, _, _)| e.0 as usize).collect();
    for &e in &picked {
        if e >= b.edges.len() {
            return Err(KernelError::InvalidParameter(format!("unknown edge {e}")));
        }
    }
    // Picked edges at each vertex.
    let mut at_vertex: std::collections::HashMap<usize, Vec<usize>> = Default::default();
    for &e in &picked {
        let ed = &b.edges[e];
        for v in ed.v {
            at_vertex.entry(v).or_default().push(e);
        }
    }
    let mut cur = Tracked::of(b.clone());
    let mut radius_at: std::collections::HashMap<usize, f64> = Default::default();
    let mut corner_vertices: Vec<usize> = Vec::new();
    for (e, kind, swap) in edges {
        let e = e.0 as usize;
        let ed = b.edges[e].clone();
        let tool = edge_tool(&b, body, e, kind.as_ref(), *swap)?;
        // The section's contact points must lie on the faces (the blend fits).
        check_fits(&b, e, kind.as_ref())?;
        let (mut tb, mut ts) = (tool.body, tool.src);
        if let Curve::Line { d, .. } = &ed.curve {
            let t = d.normalize();
            for (end, v) in ed.v.iter().enumerate() {
                let others: Vec<usize> = at_vertex.get(v).map(|x| x.iter().copied().filter(|&o| o != e).collect()).unwrap_or_default();
                let p = b.verts[*v].p;
                let out = if end == 0 { -t } else { t }; // away from the edge at this end
                match others.len() {
                    0 => {}
                    1 => {
                        // Miter with the other edge: the bisector plane of the two directions.
                        let o = &b.edges[others[0]];
                        let od = match &o.curve {
                            Curve::Line { d, .. } => {
                                let d = d.normalize();
                                if o.v[0] == *v { d } else { -d }
                            }
                            _ => continue,
                        };
                        // The plane through p bisecting the angle between −out (along this edge)
                        // and od (along the other): normal (out + od) normalized... the half
                        // space beyond keeps the other's side.
                        // The plane through p bisecting the two edges; its normal points to the
                        // other edge's side, which is cut away.
                        let n = (out + od).normalize();
                        let (nb, ns) = trim_tool(&tb, &ts, p, n)?;
                        tb = nb;
                        ts = ns;
                    }
                    2 if corners => {
                        // Three fillets: stop square to the edge at the ball centre.
                        if let SectionKind::Circle { r } = kind(PI / 2.0) {
                            radius_at.insert(*v, r);
                            if !corner_vertices.contains(v) {
                                corner_vertices.push(*v);
                            }
                            let c = ball_centre(&b, *v, r).unwrap_or(p);
                            let along = (c - p).dot(&out);
                            let (nb, ns) = trim_tool(&tb, &ts, p + out * along, out)?;
                            tb = nb;
                            ts = ns;
                        }
                    }
                    _ => {}
                }
            }
        }
        // A concave tool adds material: cut its extensions off at the end faces (or square to
        // the edge at its ends).
        if !tool.convex {
            for (end, v) in ed.v.iter().enumerate() {
                let p = b.verts[*v].p;
                let tv = ed.curve.tangent(ed.t[end]).normalize();
                let out = if end == 0 { -tv } else { tv };
                let [s1, s2] = edge_sides(&b, e)?;
                let mut plane: Option<(P3, V3)> = None;
                for (fi, f) in b.faces.iter().enumerate() {
                    if fi == s1.0 || fi == s2.0 || !f.loops.iter().flatten().any(|c| b.edges[c.edge].v.contains(v)) {
                        continue;
                    }
                    if let Surface::Plane { f: fr } = &f.surf {
                        let n = fr.x.cross(&fr.y) * if f.sense { 1.0 } else { -1.0 };
                        if n.dot(&out) > 1e-6 {
                            plane = Some((p, n));
                        }
                    }
                }
                let (o, n) = plane.unwrap_or((p, out));
                let (nb, ns) = trim_tool(&tb, &ts, o, n)?;
                tb = nb;
                ts = ns;
            }
        }
        let op = if tool.convex { BoolOp::Subtract } else { BoolOp::Union };
        cur = cur.apply(&tb, &ts, op)?;
    }
    for v in corner_vertices {
        let r = radius_at[&v];
        if let Some((tool, src)) = corner_tool(&b, v, r, Origin::FromVertex { body, vertex: VertexId(vertex_index(&b, v) as u64) })? {
            cur = cur.apply(&tool, &src, BoolOp::Subtract)?;
        }
    }
    Ok(cur.into_result(k, body))
}

/// The index of a vertex among the vertices the kernel reports (those with edges).
fn vertex_index(b: &Body, v: usize) -> usize {
    let mut used = vec![false; b.verts.len()];
    for e in &b.edges {
        used[e.v[0]] = true;
        used[e.v[1]] = true;
    }
    used[..v].iter().filter(|x| **x).count()
}

fn ball_centre(b: &Body, v: usize, r: f64) -> Option<P3> {
    let p = b.verts[v].p;
    let mut normals: Vec<V3> = Vec::new();
    for f in &b.faces {
        if !f.loops.iter().flatten().any(|c| b.edges[c.edge].v.contains(&v)) {
            continue;
        }
        let Surface::Plane { f: fr } = &f.surf else { return None };
        let n = fr.x.cross(&fr.y) * if f.sense { 1.0 } else { -1.0 };
        if !normals.iter().any(|m| m.dot(&n) > 1.0 - 1e-9) {
            normals.push(n);
        }
    }
    if normals.len() != 3 {
        return None;
    }
    let m = nalgebra::Matrix3::from_rows(&[normals[0].transpose(), normals[1].transpose(), normals[2].transpose()]);
    Some(p + m.try_inverse()? * V3::new(-r, -r, -r))
}

/// Refuses a blend whose contact lines would leave the faces.
fn check_fits(b: &Body, e: usize, kind: &dyn Fn(f64) -> SectionKind) -> Result<()> {
    let ed = &b.edges[e];
    let [s1, s2] = edge_sides(b, e)?;
    let tm = ed.mid_t();
    let (n1, w1) = face_dirs(b, s1.0, e, s1.1, tm);
    let (n2, w2) = face_dirs(b, s2.0, e, s2.1, tm);
    let phi = n1.dot(&n2).clamp(-1.0, 1.0).acos();
    let theta = w1.dot(&w2).clamp(-1.0, 1.0).acos();
    let (a1, a2) = match kind(phi) {
        SectionKind::Circle { r } | SectionKind::Conic { r, .. } | SectionKind::Curvature { r, .. } => (r / (theta / 2.0).tan(), r / (theta / 2.0).tan()),
        SectionKind::Chamfer { d1, d2 } => (d1, d2),
        SectionKind::Asymmetric { r1, r2 } => (r1 / (theta / 2.0).tan(), r2 / (theta / 2.0).tan()),
    };
    // The contact lines must reach the faces somewhere along the edge (a fillet running into a
    // hole beside the edge is the boolean's business, not a misfit).
    let mut fits = [false, false];
    for frac in [0.1, 0.3, 0.5, 0.7, 0.9] {
    let tm = ed.t[0] + (ed.t[1] - ed.t[0]) * frac;
    let p = ed.point(tm);
    for (side, (s, w, a)) in [(s1, w1, a1), (s2, w2, a2)].into_iter().enumerate() {
        if !(a > 0.0) || !a.is_finite() {
            return Err(KernelError::InvalidParameter("The blend's size must be positive".into()));
        }
        let q = p + w * a;
        let f = &b.faces[s.0];
        let polys = super::classify::face_polygons(b, f);
        let (u, v) = f.surf.project(&q, None);
        if (f.surf.eval(u, v) - q).norm() > 1e-6 * (1.0 + a) {
            // Curved across the edge: the contact point isn't on the face's surface; leave the
            // check to the boolean.
            fits[side] = true;
            continue;
        }
        let inside = super::classify::periodic_candidates(&f.surf, u, v, &polys)
            .into_iter()
            .any(|uv| super::classify::in_face(&polys, &f.surf, uv, 1e-7) != super::classify::InFace::Outside);
        if inside {
            fits[side] = true;
        }
    }
    }
    if !(fits[0] && fits[1]) {
        return Err(KernelError::OperationFailed("the blend is too large for the faces at the edge".into()));
    }
    Ok(())
}

/// The kernel's fillet: every edge with its tangent chain.
pub fn fillet(k: &mut NativeKernel, body: BodyId, edges: &[EdgeId], spec: &FilletSpec) -> Result<OpResult> {
    if let FilletSize::Radius(r) | FilletSize::Width(r) = spec.size {
        if !(r > 0.0) {
            return Err(KernelError::InvalidParameter("The fillet's size must be positive".into()));
        }
    }
    let mut all: Vec<EdgeId> = Vec::new();
    for &e in edges {
        for c in k.tangent_chain(body, e)? {
            if !all.contains(&c) {
                all.push(c);
            }
        }
    }
    // Section profiles other than circular: straight edges between flat faces only.
    if !matches!(spec.profile, FilletProfile::Circular) {
        let b = k.get(body)?;
        for e in &all {
            let ed = &b.edges[e.0 as usize];
            let [s1, s2] = edge_sides(b, e.0 as usize)?;
            let flat = matches!(b.faces[s1.0].surf, Surface::Plane { .. }) && matches!(b.faces[s2.0].surf, Surface::Plane { .. });
            if !matches!(ed.curve, Curve::Line { .. }) || !flat {
                return Err(KernelError::InvalidParameter("Conic, curvature and asymmetric fillets work on straight edges between flat faces".into()));
            }
        }
    }
    let spec = *spec;
    let list: Vec<(EdgeId, Box<dyn Fn(f64) -> SectionKind>, bool)> = all
        .iter()
        .map(|&e| {
            let f: Box<dyn Fn(f64) -> SectionKind> = Box::new(move |phi| fillet_section(&spec, phi));
            (e, f, false)
        })
        .collect();
    let corners = matches!(spec.profile, FilletProfile::Circular) && matches!(spec.size, FilletSize::Radius(_));
    let r = blend(k, body, &list, corners)?;
    if !spec.allow_overflow {
        check_overflow(k, body, &all, &r)?;
    }
    Ok(r)
}

/// With Allow edge overflow off: a fillet face may only touch faces continuing the faces at its
/// edge or around the edge's end vertices.
fn check_overflow(k: &NativeKernel, body: BodyId, edges: &[EdgeId], r: &OpResult) -> Result<()> {
    let b = k.get(body)?;
    let out = k.get(r.bodies[0])?;
    let ef = out.edge_faces();
    for (f, o) in &r.history.generated {
        let Origin::FromEdge { edge, .. } = o else { continue };
        let e = edge.0 as usize;
        // Allowed input faces: those at the edge and at its end vertices.
        let ed = &b.edges[e];
        let mut allowed: Vec<usize> = Vec::new();
        for (fi, face) in b.faces.iter().enumerate() {
            if face.loops.iter().flatten().any(|c| c.edge == e || b.edges[c.edge].v.contains(&ed.v[0]) || b.edges[c.edge].v.contains(&ed.v[1])) {
                allowed.push(fi);
            }
        }
        let fi = f.0 as usize;
        for l in &out.faces[fi].loops {
            for c in l {
                for &g in &ef[c.edge] {
                    if g == fi {
                        continue;
                    }
                    let from: Vec<usize> = r.history.modified.iter().filter(|(x, _)| x.0 as usize == g).map(|(_, i)| i.face.0 as usize).collect();
                    let fresh = r.history.generated.iter().any(|(x, _)| x.0 as usize == g);
                    if !fresh && !from.is_empty() && !from.iter().any(|x| allowed.contains(x)) {
                        return Err(KernelError::OperationFailed(
                            "The fillet runs onto another face; turn on Allow edge overflow to let it".into(),
                        ));
                    }
                }
            }
        }
        let _ = edges;
    }
    Ok(())
}

/// The kernel's chamfer.
pub fn chamfer(k: &mut NativeKernel, body: BodyId, edges: &[EdgeId], opts: &ChamferOpts) -> Result<OpResult> {
    let spec = opts.spec;
    let b = (**k.get(body)?).clone();
    let mut list: Vec<(EdgeId, Box<dyn Fn(f64) -> SectionKind>, bool)> = Vec::new();
    for &e in edges {
        let flip = opts.flip != opts.flipped.contains(&e);
        let ei = e.0 as usize;
        // Tangent measurement: a face curving across the edge (a cylinder along a straight edge,
        // a sphere) takes the chord of the tangent distance on that side.
        let chord = |side: usize, d: f64| -> f64 {
            if opts.measurement != ChamferMeasure::Tangent {
                return d;
            }
            let Ok(sides) = edge_sides(&b, ei) else { return d };
            let f = &b.faces[sides[side].0];
            let ed = &b.edges[ei];
            let across = match (&f.surf, &ed.curve) {
                (Surface::Cylinder { f: fr, r }, Curve::Line { d: dl, .. }) if dl.normalize().cross(&fr.z).norm() < 1e-9 => Some(*r),
                (Surface::Sphere { r, .. }, _) => Some(*r),
                _ => None,
            };
            match across {
                Some(r) => 2.0 * r * ((d / r).atan() / 2.0).sin(),
                None => d,
            }
        };
        let (s0, s1) = if flip { (1usize, 0usize) } else { (0, 1) };
        let (d1, d2) = match spec {
            ChamferSpec::EqualDistance(d) => (chord(s0, d), chord(s1, d)),
            ChamferSpec::TwoDistances(a, c) => (chord(s0, a), chord(s1, c)),
            ChamferSpec::DistanceAngle { distance, angle } => (distance, f64::NAN + angle),
        };
        let kind: Box<dyn Fn(f64) -> SectionKind> = match spec {
            ChamferSpec::DistanceAngle { distance, angle } => Box::new(move |phi| {
                // The second distance from the angle at the first face: in the section triangle
                // with the faces' interior angle θ = π − φ, d2 = d sin(a) / sin(π − a − θ).
                let theta = PI - phi;
                let d2 = distance * angle.sin() / (PI - angle - theta).sin();
                SectionKind::Chamfer { d1: distance, d2 }
            }),
            _ => Box::new(move |_| SectionKind::Chamfer { d1, d2 }),
        };
        list.push((e, kind, flip));
    }
    blend(k, body, &list, false)
}

/// Onshape's Full round: the middle face replaced by a half cylinder tangent to two parallel
/// flat side faces (r = half their distance).
pub fn full_round(k: &mut NativeKernel, body: BodyId, side1: FaceId, center: FaceId, side2: FaceId) -> Result<OpResult> {
    let b = (**k.get(body)?).clone();
    let plane_of = |f: FaceId| -> Result<(P3, V3)> {
        let face = b.faces.get(f.0 as usize).ok_or_else(|| KernelError::InvalidParameter(format!("unknown {f:?}")))?;
        match &face.surf {
            Surface::Plane { f: fr } => Ok((fr.o, fr.x.cross(&fr.y) * if face.sense { 1.0 } else { -1.0 })),
            _ => Err(KernelError::InvalidParameter("A full round needs flat side and middle faces".into())),
        }
    };
    let (o1, n1) = plane_of(side1)?;
    let (o2, n2) = plane_of(side2)?;
    let (om, nm) = plane_of(center)?;
    if n1.dot(&n2) > -1.0 + 1e-9 {
        return Err(KernelError::InvalidParameter("A full round needs two parallel side faces facing away from each other".into()));
    }
    if n1.dot(&nm).abs() > 1e-9 {
        return Err(KernelError::InvalidParameter("The middle face must be square to the parallel side faces".into()));
    }
    let width = (o2 - o1).dot(&n1).abs();
    let r = width / 2.0;
    // The round's axis: in the middle face's plane moved r into the material, midway between
    // the sides, along the sides' and middle face's common direction.
    let along = n1.cross(&nm).normalize();
    let mid_side = o1 + (o2 - o1).dot(&n1) * n1 * 0.5 + n1 * 0.0;
    let _ = mid_side;
    // A point on the axis: on the middle plane, halfway between the side planes, then r down.
    let base = om + n1 * ((o1 - om).dot(&n1) + (o2 - om).dot(&n1)) * 0.5 - nm * r;
    // Section plane square to the axis.
    let x = n1;
    let y = nm;
    let plane = Plane { origin: base, x_dir: nalgebra::Unit::new_normalize(x), normal: nalgebra::Unit::new_normalize(x.cross(&y)) };
    let ydir = plane.y_dir().into_inner();
    let up = if ydir.dot(&nm) > 0.0 { 1.0 } else { -1.0 };
    // Rectangle above the axis less the half disc: corners (±r, 0), (±r, r + δ) in (x, y·up).
    let delta = r * 0.5 + 1e-3;
    let p2 = |a: f64, c: f64| nalgebra::Point2::new(a, c * up);
    let l = |a, c| Curve2::Line { a, b: c, source: None };
    let (start, sweep) = if up > 0.0 { (0.0, PI) } else { (0.0, -PI) };
    let region = Region {
        outer: Loop {
            curves: vec![
                Curve2::Arc { center: p2(0.0, 0.0), radius: r, start_angle: start, sweep, source: Some(1) },
                l(p2(-r, 0.0), p2(-r, r + delta)),
                l(p2(-r, r + delta), p2(r, r + delta)),
                l(p2(r, r + delta), p2(r, 0.0)),
            ],
        },
        holes: vec![],
        source: None,
    };
    let prof = Profile::new(plane, vec![region]);
    let regions = super::profile::regions(&prof)?;
    let mf = super::profile::merge_regions(&regions)?;
    // Long enough to cover the body along the axis.
    let (lo, hi) = super::kernel::extent_along(&b, &base, &along);
    let dir = plane.normal.into_inner();
    let s = if dir.dot(&along) > 0.0 { 1.0 } else { -1.0 };
    let (lo, hi) = if s > 0.0 { (lo, hi) } else { (-hi, -lo) };
    let kind = Prism { plane, dir };
    let built = sweep::sweep_full(&mf, &[], &kind, lo - 1.0, hi + 1.0, false, true)?;
    let mut src = vec![Vec::new(); built.body.faces.len()];
    for (f, o) in &built.origins {
        if let Origin::ProfileCurve { curve: 1, .. } = o {
            src[*f].push(FaceSrc::Input(center.0 as usize));
        }
    }
    let cur = Tracked::of(b.clone());
    let out = cur.apply(&built.body, &src, BoolOp::Subtract)?;
    // The removed volume must be the two corner slivers: (2 − π/2) r² per unit length.
    let v0 = super::mass::body_mass(&b).volume;
    let v1 = super::mass::body_mass(&out.body).volume;
    let len = hi - lo;
    if v0 - v1 > (2.0 - PI / 2.0) * r * r * len * 1.5 + 1e-6 {
        return Err(KernelError::InvalidParameter("The side faces are too shallow for the full round".into()));
    }
    Ok(out.into_result(k, body))
}
