//! Sweeping planar faces into solids: prisms (a straight sweep) and revolutions (a sweep about
//! an axis), one generic builder.
//!
//! A profile point q traces a *rail* as the sweep parameter s runs from s0 to s1 (a straight
//! line, or a circle about the axis); each profile piece sweeps a side face; the profile itself
//! is the start cap at s0 and the end cap at s1 (none for a whole turn).

use super::build::{fix_senses, Built, MIN_DEPTH};
use super::curve::Curve;
use super::math::*;
use super::profile::{MergedFace, Piece, Src};
use super::surface::Surface;
use super::topo::*;
use crate::{KernelError, Origin, Result};

/// The geometry of a side face: its surface and the p-curves of the profile piece at a sweep
/// parameter (parametrized by the piece's t) and of the rail of a joint (parametrized by s).
pub struct SideGeo {
    pub surf: Surface,
    pub along: Box<dyn Fn(f64) -> Curve>,
    pub across: Box<dyn Fn(f64) -> Curve>,
}

pub trait SweepKind {
    /// Sketch coordinates (z = 0) to model space at sweep parameter s.
    fn place(&self, s: f64) -> Affine;
    /// The rail traced by the sketch point q (parametrized by s), or None when it stays put
    /// (a point on the axis).
    fn rail(&self, q: P2) -> Option<Curve>;
    /// The side face swept by a piece (`c3` is the piece's curve at s = 0), or None when it
    /// sweeps no area (a piece lying on the axis).
    fn side(&self, piece: &Piece, c3: &Curve) -> Option<SideGeo>;
    /// Whether the sweep leaves the profile plane along its normal.
    fn up(&self) -> bool;
}

/// A straight sweep along the unit `dir`.
pub struct Prism {
    pub plane: crate::Plane,
    pub dir: V3,
}

impl SweepKind for Prism {
    fn place(&self, s: f64) -> Affine {
        super::build::plane_map(&self.plane, &self.dir, s)
    }
    fn rail(&self, q: P2) -> Option<Curve> {
        let p = self.place(0.0).point(&P3::new(q.x, q.y, 0.0));
        Some(Curve::Line { o: p, d: self.dir })
    }
    fn side(&self, _piece: &Piece, c3: &Curve) -> Option<SideGeo> {
        let dir = self.dir;
        Some(match c3 {
            Curve::Line { o, d } => {
                let x = d.normalize();
                let mut y = dir - x * x.dot(&dir);
                y.normalize_mut();
                let z = x.cross(&y);
                let (a, b) = (V2::new(d.norm(), 0.0), V2::new(x.dot(&dir), y.dot(&dir)));
                SideGeo {
                    surf: Surface::Plane { f: Frame { o: *o, x, y, z } },
                    along: Box::new(move |s| line2(P2::origin() + b * s, a)),
                    across: Box::new(move |t| line2(P2::origin() + a * t, b)),
                }
            }
            Curve::Circle { c, x, y, r } if x.cross(y).cross(&dir).norm() < 1e-12 => {
                let z = x.cross(y);
                let k = z.dot(&dir);
                SideGeo {
                    surf: Surface::Cylinder { f: Frame { o: *c, x: *x, y: *y, z }, r: *r },
                    along: Box::new(move |s| line2(P2::new(0.0, k * s), V2::new(1.0, 0.0))),
                    across: Box::new(move |t| line2(P2::new(t, 0.0), V2::new(0.0, k))),
                }
            }
            _ => SideGeo {
                surf: Surface::Extrusion { c: c3.clone(), d: dir },
                along: Box::new(|s| line2(P2::new(0.0, s), V2::new(1.0, 0.0))),
                across: Box::new(|t| line2(P2::new(t, 0.0), V2::new(0.0, 1.0))),
            },
        })
    }
    fn up(&self) -> bool {
        self.dir.dot(&self.plane.normal) > 0.0
    }
}

/// A turn about an axis lying in the profile's plane; s is the angle (radians, counter-clockwise
/// about `f.z`). `f.x` points from the axis towards the profile, so the profile lies in the
/// frame's xz half plane x ≥ 0.
pub struct Revolve {
    pub plane: crate::Plane,
    pub f: Frame,
}

pub fn line2(o: P2, d: V2) -> Curve {
    Curve::Line { o: P3::new(o.x, o.y, 0.0), d: V3::new(d.x, d.y, 0.0) }
}

impl Revolve {
    fn rot(&self, s: f64) -> Affine {
        let r = nalgebra::Rotation3::from_axis_angle(&nalgebra::Unit::new_normalize(self.f.z), s).into_inner();
        let o = self.f.o.coords;
        Affine { m: r, t: o - r * o }
    }
    /// Local (ρ, h) of a model point.
    fn rh(&self, p: &P3) -> (f64, f64) {
        let l = self.f.local(p);
        ((l.x * l.x + l.y * l.y).sqrt(), l.z)
    }
}

impl SweepKind for Revolve {
    fn place(&self, s: f64) -> Affine {
        let base = super::build::plane_map(&self.plane, &V3::zeros(), 0.0);
        let r = self.rot(s);
        Affine { m: r.m * base.m, t: r.m * base.t + r.t }
    }
    fn rail(&self, q: P2) -> Option<Curve> {
        let p = self.place(0.0).point(&P3::new(q.x, q.y, 0.0));
        let (rho, h) = self.rh(&p);
        if rho < TOL {
            return None;
        }
        Some(Curve::Circle { c: self.f.o + self.f.z * h, x: self.f.x, y: self.f.y, r: rho })
    }
    fn side(&self, piece: &Piece, c3: &Curve) -> Option<SideGeo> {
        let f = self.f;
        match c3 {
            Curve::Line { o, d } => {
                let (r0, h0) = self.rh(o);
                let dl = f.local(&(o + d)) - f.local(o);
                let (dr, dh) = (dl.x, dl.z); // the line lies in the xz plane: dl.y = 0
                let len = d.norm();
                let (dr, dh) = (dr / len, dh / len);
                // Lies on the axis: sweeps nothing.
                let (ra, rb) = (r0 + dr * piece.t0 * len, r0 + dr * piece.t1 * len);
                if ra.abs() < TOL && rb.abs() < TOL {
                    return None;
                }
                if dr.abs() < 1e-12 {
                    // Cylinder: u = s, v = h0 + dh·len·t.
                    let k = dh * len;
                    Some(SideGeo {
                        surf: Surface::Cylinder { f, r: r0 },
                        along: Box::new(move |s| line2(P2::new(s, h0), V2::new(0.0, k))),
                        across: Box::new(move |t| line2(P2::new(0.0, h0 + k * t), V2::new(1.0, 0.0))),
                    })
                } else if dh.abs() < 1e-12 {
                    // Plane square to the axis at h0: uv = ρ(t)(cos s, sin s).
                    let fr = Frame { o: f.o + f.z * h0, x: f.x, y: f.y, z: f.z };
                    let k = dr * len;
                    Some(SideGeo {
                        surf: Surface::Plane { f: fr },
                        along: Box::new(move |s| {
                            let c = V2::new(s.cos(), s.sin());
                            line2(P2::origin() + c * r0, c * k)
                        }),
                        across: Box::new(move |t| {
                            let r = r0 + k * t;
                            Curve::Circle { c: P3::origin(), x: V3::x(), y: V3::y(), r: r.abs().max(1e-300) }
                        }),
                    })
                } else {
                    // Cone through C(0): r = r0, sin a = dr, cos a = dh, v = len·t.
                    let a = dr.atan2(dh);
                    let fr = Frame { o: f.o + f.z * h0, x: f.x, y: f.y, z: f.z };
                    Some(SideGeo {
                        surf: Surface::Cone { f: fr, r: r0, a },
                        along: Box::new(move |s| line2(P2::new(s, 0.0), V2::new(0.0, len))),
                        across: Box::new(move |t| line2(P2::new(0.0, len * t), V2::new(1.0, 0.0))),
                    })
                }
            }
            Curve::Circle { c, x, y, r } => {
                let (rc, hc) = self.rh(c);
                // Angle φ of the point from the centre in the (ρ, h) plane: φ(t) = σ t + φ0.
                let lx = f.local(&(c + x)) - f.local(c);
                let ly = f.local(&(c + y)) - f.local(c);
                let ax = lx.z.atan2(lx.x);
                let sig = if lx.x * ly.z - lx.z * ly.x > 0.0 { 1.0 } else { -1.0 };
                // Unwrap φ0 so φ at the piece's middle lies in (−π, π].
                let tm = (piece.t0 + piece.t1) * 0.5;
                let phim = wrap(sig * tm + ax, -PI, TAU);
                let phi0 = phim - sig * tm;
                let fr = Frame { o: f.o + f.z * hc, x: f.x, y: f.y, z: f.z };
                let surf = if rc < TOL { Surface::Sphere { f: fr, r: *r } } else { Surface::Torus { f: fr, rmaj: rc, rmin: *r } };
                Some(SideGeo {
                    surf,
                    along: Box::new(move |s| line2(P2::new(s, phi0), V2::new(0.0, sig))),
                    across: Box::new(move |t| line2(P2::new(0.0, sig * t + phi0), V2::new(1.0, 0.0))),
                })
            }
            _ => Some(SideGeo {
                surf: Surface::Revolution { f, m: c3.clone() },
                along: Box::new(|s| line2(P2::new(s, 0.0), V2::new(0.0, 1.0))),
                across: Box::new(|t| line2(P2::new(0.0, t), V2::new(1.0, 0.0))),
            }),
        }
    }
    fn up(&self) -> bool {
        // The sweep leaves along f.y (z × x) at s = 0.
        self.f.y.dot(&self.plane.normal) > 0.0
    }
}

/// Sweeps the faces from s0 to s1 (s0 < s1). `whole`: a closed sweep (a whole turn) whose end
/// is its start: no caps, the start edges are seams.
pub fn sweep(_plane: &crate::Plane, faces: &[MergedFace], kind: &dyn SweepKind, s0: f64, s1: f64, whole: bool) -> Result<Built> {
    sweep_full(faces, &[], kind, s0, s1, whole, true)
}

/// The side faces swept by one strand of pieces (a closed loop or an open chain): returns the
/// face indices with their origins and the start and end profile coedges (cap boundaries).
struct Strand {
    sides: Vec<(usize, Vec<Src>)>,
    bottom: Vec<Coedge>,
    top: Vec<Coedge>,
}

#[allow(clippy::too_many_arguments)]
fn sweep_strand(b: &mut Body, lp: &[Piece], closed: bool, kind: &dyn SweepKind, s0: f64, s1: f64, whole: bool, up: bool, m0: &Affine, m1: &Affine) -> Strand {
    let k = lp.len();
    let nj = if closed { k } else { k + 1 };
    let joint = |j: usize| -> P2 { if j < k { lp[j].start() } else { lp[k - 1].end() } };
    let rails: Vec<Option<Curve>> = (0..nj).map(|j| kind.rail(joint(j))).collect();
    let vb: Vec<usize> = (0..nj)
        .map(|j| {
            let q = joint(j);
            b.add_vertex(m0.point(&P3::new(q.x, q.y, 0.0)))
        })
        .collect();
    let vt: Vec<usize> = (0..nj)
        .map(|j| {
            if whole || rails[j].is_none() {
                vb[j]
            } else {
                let q = joint(j);
                b.add_vertex(m1.point(&P3::new(q.x, q.y, 0.0)))
            }
        })
        .collect();
    let rail_e: Vec<(usize, bool)> = (0..nj)
        .map(|j| match &rails[j] {
            Some(c) => (b.add_edge(c.clone(), s0, s1, vb[j], vt[j]), false),
            None => (b.add_degenerate(b.verts[vb[j]].p, s0, s1, vb[j]), true),
        })
        .collect();
    let mut out = Strand { sides: Vec::new(), bottom: Vec::new(), top: Vec::new() };
    for (i, p) in lp.iter().enumerate() {
        let j = if closed { (i + 1) % k } else { i + 1 };
        let c0 = p.c.transformed(m0);
        let (v0b, v1b) = if p.rev { (vb[j], vb[i]) } else { (vb[i], vb[j]) };
        let (v0t, v1t) = if p.rev { (vt[j], vt[i]) } else { (vt[i], vt[j]) };
        let Some(side) = kind.side(p, &p.c.transformed(&kind.place(0.0))) else {
            let e = b.add_edge(c0.clone(), p.t0, p.t1, v0b, v1b);
            out.bottom.push(Coedge { edge: e, fwd: !p.rev, pc: p.c.clone() });
            out.top.push(Coedge { edge: e, fwd: !p.rev, pc: p.c.clone() });
            continue;
        };
        let eb = b.add_edge(c0.clone(), p.t0, p.t1, v0b, v1b);
        let et = if whole { eb } else { b.add_edge(p.c.transformed(m1), p.t0, p.t1, v0t, v1t) };
        out.bottom.push(Coedge { edge: eb, fwd: !p.rev, pc: p.c.clone() });
        out.top.push(Coedge { edge: et, fwd: !p.rev, pc: p.c.clone() });
        let (ts, te) = (p.ts(), p.te());
        let plane_side = matches!(side.surf, Surface::Plane { .. });
        let mut sl = Vec::new();
        sl.push(Coedge { edge: eb, fwd: !p.rev, pc: (side.along)(s0) });
        if !(rail_e[j].1 && plane_side) {
            sl.push(Coedge { edge: rail_e[j].0, fwd: true, pc: (side.across)(te) });
        }
        sl.push(Coedge { edge: et, fwd: p.rev, pc: (side.along)(s1) });
        if !(rail_e[i].1 && plane_side) {
            sl.push(Coedge { edge: rail_e[i].0, fwd: false, pc: (side.across)(ts) });
        }
        if !up {
            sl.reverse();
            for c in &mut sl {
                c.fwd = !c.fwd;
            }
        }
        b.faces.push(Face { surf: side.surf, sense: true, loops: vec![sl] });
        out.sides.push((b.faces.len() - 1, std::iter::once(p.src).chain(p.more.iter().copied()).collect()));
    }
    out
}

/// Sweeps closed faces (solids, or their boundaries as sheets when `solid` is false) and open
/// chains (sheets) from s0 to s1.
pub fn sweep_full(faces: &[MergedFace], chains: &[Vec<Piece>], kind: &dyn SweepKind, s0: f64, s1: f64, whole: bool, solid: bool) -> Result<Built> {
    if s1 - s0 < MIN_DEPTH {
        return Err(KernelError::InvalidParameter(format!(
            "The depth {:e} is below the modelling tolerance ({MIN_DEPTH:e} mm)",
            s1 - s0
        )));
    }
    let up = kind.up();
    let m0 = kind.place(s0);
    let m1 = kind.place(s1);
    let mut out = Built::default();
    let b = &mut out.body;
    let rev_loop = |l: &mut Vec<Coedge>| {
        l.reverse();
        for c in l.iter_mut() {
            c.fwd = !c.fwd;
        }
    };
    for mf in faces {
        let face_start = b.faces.len();
        let mut bottom_loops: Vec<Vec<Coedge>> = Vec::new();
        let mut top_loops: Vec<Vec<Coedge>> = Vec::new();
        let mut sides = Vec::new();
        for lp in std::iter::once(&mf.outer).chain(mf.holes.iter()) {
            let st = sweep_strand(b, lp, true, kind, s0, s1, whole, up, &m0, &m1);
            sides.extend(st.sides);
            bottom_loops.push(st.bottom);
            top_loops.push(st.top);
            if !solid {
                // Each loop's sheet is its own shell.
            }
        }
        let n_sides = b.faces.len() - face_start;
        if solid && !whole {
            if up {
                bottom_loops.iter_mut().for_each(rev_loop);
            } else {
                top_loops.iter_mut().for_each(rev_loop);
            }
            let frame = |m: &Affine| Frame { o: m.point(&P3::origin()), x: m.vector(&V3::x()), y: m.vector(&V3::y()), z: m.vector(&V3::z()) };
            b.faces.push(Face { surf: Surface::Plane { f: frame(&m0) }, sense: !up, loops: bottom_loops });
            let ib = b.faces.len() - 1;
            b.faces.push(Face { surf: Surface::Plane { f: frame(&m1) }, sense: up, loops: top_loops });
            let it = b.faces.len() - 1;
            for r in &mf.regions {
                out.origins.push((ib, Origin::StartCap { region: *r }));
                out.origins.push((it, Origin::EndCap { region: *r }));
            }
        }
        for (fi, srcs) in sides {
            for s in srcs {
                out.origins.push((fi, s.origin()));
            }
        }
        fix_senses(b, face_start..face_start + n_sides);
        let fs: Vec<usize> = (face_start..b.faces.len()).collect();
        b.shells.push(Shell { faces: fs, closed: solid });
        if solid {
            let s = b.shells.len() - 1;
            b.solids.push(Solid { shells: vec![s] });
        }
    }
    for ch in chains {
        let face_start = b.faces.len();
        let st = sweep_strand(b, ch, false, kind, s0, s1, whole, true, &m0, &m1);
        for (fi, srcs) in st.sides {
            for s in srcs {
                out.origins.push((fi, s.origin()));
            }
        }
        fix_senses(b, face_start..b.faces.len());
        b.shells.push(Shell { faces: (face_start..b.faces.len()).collect(), closed: false });
    }
    out.origins.sort_by_key(|(f, _)| *f);
    Ok(out)
}

/// A turn about any axis, also one off the profile's plane (SMLib's rotational sweep,
/// `SmRotationalSweepGeometry`, turns whole solids about skew axes); s is the angle (radians,
/// counter-clockwise about `f.z`, the frame orthonormal with `f.o` on the axis).
///
/// A piece lying in a half plane through the axis sweeps the analytic face [`Revolve`] makes,
/// in the frame turned to that half plane; a line square to the axis sweeps a plane (an
/// annulus); any other piece sweeps a surface of revolution of the piece itself (u = s, v = the
/// piece's t: a line skew to the axis makes a hyperboloid). `up`: whether the profile leaves its
/// plane along its normal; the caller checks that every point of it leaves the same way (it
/// lies on one side of the plane through the axis along the normal), which also keeps a whole
/// turn from meeting itself.
pub struct Turn {
    pub plane: crate::Plane,
    pub f: Frame,
    pub up: bool,
}

impl Turn {
    /// The turn by s about the axis.
    pub fn rot(&self, s: f64) -> Affine {
        Revolve { plane: self.plane, f: self.f }.rot(s)
    }

    /// The circle a model point travels on (parametrized by s), or None on the axis.
    pub fn rail_of(&self, p: &P3) -> Option<Curve> {
        let f = &self.f;
        let l = f.local(p);
        let rho = l.x.hypot(l.y);
        if rho < TOL {
            return None;
        }
        let x = (f.x * l.x + f.y * l.y) / rho;
        Some(Curve::Circle { c: f.o + f.z * l.z, x, y: f.z.cross(&x), r: rho })
    }

    /// The side face a model-space curve (a piece's curve at s = 0) sweeps, or None for a piece
    /// on the axis.
    pub fn side_of(&self, piece: &Piece, c3: &Curve) -> Option<SideGeo> {
        let f = self.f;
        let pts: Vec<V3> = (0..=8).map(|i| f.local(&c3.point(piece.t0 + (piece.t1 - piece.t0) * i as f64 / 8.0))).collect();
        let scale = pts.iter().map(|l| l.norm()).fold(1.0, f64::max);
        let far = pts.iter().max_by(|a, b| a.x.hypot(a.y).total_cmp(&b.x.hypot(b.y)))?;
        if far.x.hypot(far.y) < TOL {
            return None;
        }
        // In the half plane at the azimuth of its point farthest from the axis?
        let phi = far.y.atan2(far.x);
        let (c, s) = (phi.cos(), phi.sin());
        let half = Frame { o: f.o, x: f.x * c + f.y * s, y: f.y * c - f.x * s, z: f.z };
        let in_half = pts.iter().all(|l| {
            let (x, y) = (l.x * c + l.y * s, l.y * c - l.x * s);
            y.abs() <= 1e-9 * scale && x >= -1e-9 * scale
        });
        let in_half = in_half
            && match c3 {
                Curve::Line { d, .. } => half.y.dot(d).abs() <= 1e-12 * d.norm(),
                Curve::Circle { x, y, .. } | Curve::Ellipse { x, y, .. } => half.y.dot(x).abs() <= 1e-12 * x.norm() && half.y.dot(y).abs() <= 1e-12 * y.norm(),
                Curve::Nurbs(_) => true,
            };
        if in_half {
            return Revolve { plane: self.plane, f: half }.side(piece, c3);
        }
        if let Curve::Line { o, d } = c3
            && f.z.dot(d).abs() <= 1e-12 * d.norm()
        {
            // A line square to the axis, off every half plane through it: it turns in the plane
            // at its height; uv = its point's local (x, y) turned by s.
            let l0 = f.local(o);
            let dl = V2::new(f.x.dot(d), f.y.dot(d));
            let q0 = V2::new(l0.x, l0.y);
            let rot = |s: f64, v: V2| V2::new(v.x * s.cos() - v.y * s.sin(), v.x * s.sin() + v.y * s.cos());
            return Some(SideGeo {
                surf: Surface::Plane { f: Frame { o: f.o + f.z * l0.z, x: f.x, y: f.y, z: f.z } },
                along: Box::new(move |s| line2(P2::origin() + rot(s, q0), rot(s, dl))),
                across: Box::new(move |t| {
                    let q = q0 + dl * t;
                    let r = q.norm();
                    let (x, y) = if r > 1e-300 { (V3::new(q.x / r, q.y / r, 0.0), V3::new(-q.y / r, q.x / r, 0.0)) } else { (V3::x(), V3::y()) };
                    Curve::Circle { c: P3::origin(), x, y, r: r.max(1e-300) }
                }),
            });
        }
        Some(SideGeo {
            surf: Surface::Revolution { f, m: c3.clone() },
            along: Box::new(|s| line2(P2::new(s, 0.0), V2::new(0.0, 1.0))),
            across: Box::new(|t| line2(P2::new(0.0, t), V2::new(1.0, 0.0))),
        })
    }
}

impl SweepKind for Turn {
    fn place(&self, s: f64) -> Affine {
        Revolve { plane: self.plane, f: self.f }.place(s)
    }
    fn rail(&self, q: P2) -> Option<Curve> {
        let p = self.place(0.0).point(&P3::new(q.x, q.y, 0.0));
        self.rail_of(&p)
    }
    fn side(&self, piece: &Piece, c3: &Curve) -> Option<SideGeo> {
        self.side_of(piece, c3)
    }
    fn up(&self) -> bool {
        self.up
    }
}

/// How [`sweep_body`] moves a body: s runs from 0 along `Straight`'s unit direction (mm) or
/// about a [`Turn`]'s axis (radians).
pub enum BodyMove<'a> {
    Straight(V3),
    Turn(&'a Turn),
}

impl BodyMove<'_> {
    fn place(&self, s: f64) -> Affine {
        match self {
            BodyMove::Straight(d) => Affine { m: nalgebra::Matrix3::identity(), t: d * s },
            BodyMove::Turn(t) => t.rot(s),
        }
    }
    fn rail(&self, p: &P3) -> Option<Curve> {
        match self {
            BodyMove::Straight(d) => Some(Curve::Line { o: *p, d: *d }),
            BodyMove::Turn(t) => t.rail_of(p),
        }
    }
    fn side(&self, piece: &Piece, c3: &Curve) -> Option<SideGeo> {
        match self {
            BodyMove::Straight(d) => {
                // A line along the sweep sweeps no area.
                if let Curve::Line { d: ld, .. } = c3
                    && ld.cross(d).norm() <= 1e-12 * ld.norm()
                {
                    return None;
                }
                Prism { plane: crate::Plane::top(), dir: *d }.side(piece, c3)
            }
            BodyMove::Turn(t) => t.side_of(piece, c3),
        }
    }
    /// The direction a point starts moving in.
    fn velocity(&self, p: &P3) -> V3 {
        match self {
            BodyMove::Straight(d) => *d,
            BodyMove::Turn(t) => t.f.z.cross(&(p - t.f.o)),
        }
    }
}

/// Sweeps a body one kind higher (Phase 1b): its points into edges (a wire body), its wires
/// into faces (a sheet), or its sheet into a solid (the sheet at both ends and the faces its
/// boundary edges sweep), from s = 0 to `s1`; `whole`: a whole turn (no end copies; the start
/// edges are seams). The input's vertices and edges keep their indices in the result (the
/// sweep's start); side faces are generated `FromEdge` of the edge of body `source` that swept
/// them, a sheet's caps `StartCap`/`EndCap` (region 0).
///
/// A sheet must cross the sweep the same way everywhere (every point of it leaves the same side
/// of it), else the solid would fold through itself: an `InvalidParameter` error.
pub fn sweep_body(b: &Body, mv: &BodyMove, s1: f64, whole: bool, source: u64) -> Result<Built> {
    if !whole && s1 < MIN_DEPTH {
        return Err(KernelError::InvalidParameter(format!("The sweep's length {s1:e} is below the modelling tolerance ({MIN_DEPTH:e} mm)")));
    }
    let class = b.class();
    let m1 = mv.place(s1);
    let mut out = Built::default();
    match class {
        BodyClass::Point => {
            let o = &mut out.body;
            for &v in &b.points {
                let p = b.verts[v].p;
                let rail = mv.rail(&p).ok_or_else(|| KernelError::InvalidParameter("A point on the axis sweeps nothing".into()))?;
                let v0 = o.add_vertex(p);
                let v1 = if whole { v0 } else { o.add_vertex(m1.point(&p)) };
                let e = o.add_edge(rail, 0.0, s1, v0, v1);
                o.wires.push(Wire { edges: vec![(e, true)] });
            }
            return Ok(out);
        }
        BodyClass::Wire | BodyClass::Sheet => {}
        BodyClass::Solid => return Err(KernelError::InvalidParameter("A solid can't be swept one kind higher (sweep its faces, or unite its copies)".into())),
        BodyClass::Empty => return Err(KernelError::InvalidParameter("The body to sweep is empty".into())),
        BodyClass::Mixed => return Err(KernelError::InvalidParameter("A sweep takes one kind of body: points, wires or a sheet".into())),
    }
    let sheet = class == BodyClass::Sheet;
    // The edges swept, each with the way its wire (or its sheet's face) runs along it.
    let swept: Vec<(usize, bool)> = if sheet {
        let mut uses: Vec<Vec<bool>> = vec![Vec::new(); b.edges.len()];
        for c in b.faces.iter().flat_map(|f| f.loops.iter().flatten()) {
            uses[c.edge].push(c.fwd);
        }
        (0..b.edges.len()).filter(|&e| uses[e].len() == 1 && !b.edges[e].degenerate).map(|e| (e, uses[e][0])).collect()
    } else {
        b.wires.iter().flat_map(|w| w.edges.iter().copied()).collect()
    };
    if sheet {
        check_transversal(b, mv)?;
    }
    let o = &mut out.body;
    // The start: the body's vertices and edges as they are.
    o.verts = b.verts.clone();
    o.edges = b.edges.clone();
    let nv = b.verts.len();
    let mut on_edge = vec![false; nv];
    for e in &b.edges {
        on_edge[e.v[0]] = true;
        on_edge[e.v[1]] = true;
    }
    let mut swept_v = vec![false; nv];
    for &(e, _) in &swept {
        swept_v[b.edges[e].v[0]] = true;
        swept_v[b.edges[e].v[1]] = true;
    }
    // The end vertices (the same on the axis or for a whole turn) and the rails.
    let mut top_v: Vec<usize> = (0..nv).collect();
    let mut rail_e: Vec<Option<(usize, bool)>> = vec![None; nv];
    for v in 0..nv {
        if !on_edge[v] {
            continue;
        }
        let p = b.verts[v].p;
        let rail = mv.rail(&p);
        if !whole && rail.is_some() {
            top_v[v] = o.add_vertex(m1.point(&p));
            o.verts[top_v[v]].tol = b.verts[v].tol;
        }
        if swept_v[v] {
            rail_e[v] = Some(match rail {
                Some(c) => (o.add_edge(c, 0.0, s1, v, top_v[v]), false),
                None => (o.add_degenerate(p, 0.0, s1, v), true),
            });
        }
    }
    // The end edges: every edge of a sheet (its end copy), the swept edges of wires.
    let ne = b.edges.len();
    let mut top_e: Vec<usize> = (0..ne).collect();
    if !whole {
        let mut need = vec![sheet; ne];
        for &(e, _) in &swept {
            need[e] = true;
        }
        for e in (0..ne).filter(|&e| need[e]) {
            let mut t = b.edges[e].clone();
            t.curve = t.curve.transformed(&m1);
            t.v = [top_v[t.v[0]], top_v[t.v[1]]];
            o.edges.push(t);
            top_e[e] = o.edges.len() - 1;
        }
    }
    let mut faces = Vec::new();
    for &(e, fwd) in &swept {
        let ed = &b.edges[e];
        let piece = Piece { c: ed.curve.clone(), t0: ed.t[0], t1: ed.t[1], rev: false, src: Src::Edge { body: source, edge: e as u64 }, more: Vec::new() };
        let Some(side) = mv.side(&piece, &ed.curve) else { continue };
        let (vs, ve) = if fwd { (ed.v[0], ed.v[1]) } else { (ed.v[1], ed.v[0]) };
        let (ts, te) = if fwd { (ed.t[0], ed.t[1]) } else { (ed.t[1], ed.t[0]) };
        let (Some(rs), Some(re)) = (rail_e[vs], rail_e[ve]) else { continue };
        let plane_side = matches!(side.surf, Surface::Plane { .. });
        let mut sl = vec![Coedge { edge: e, fwd, pc: (side.along)(0.0) }];
        if !(re.1 && plane_side) {
            sl.push(Coedge { edge: re.0, fwd: true, pc: (side.across)(te) });
        }
        sl.push(Coedge { edge: top_e[e], fwd: !fwd, pc: (side.along)(s1) });
        if !(rs.1 && plane_side) {
            sl.push(Coedge { edge: rs.0, fwd: false, pc: (side.across)(ts) });
        }
        o.faces.push(Face { surf: side.surf, sense: true, loops: vec![sl] });
        let fi = o.faces.len() - 1;
        faces.push(fi);
        out.origins.push((fi, Origin::FromEdge { body: crate::BodyId(source), edge: crate::EdgeId(e as u64) }));
    }
    fix_senses(o, faces.clone());
    if faces.is_empty() {
        return Err(KernelError::InvalidParameter("The sweep makes no faces (every edge lies along it)".into()));
    }
    if !sheet {
        // One open shell per piece of edge-connected faces (disjoint wires sweep apart).
        for piece in o.face_components(&faces) {
            o.shells.push(Shell { faces: piece, closed: false });
        }
        out.origins.sort_by_key(|(f, _)| *f);
        return Ok(out);
    }
    if !whole {
        for f in &b.faces {
            let mut bottom = f.clone();
            Body::reverse_face(&mut bottom);
            o.faces.push(bottom);
            faces.push(o.faces.len() - 1);
            out.origins.push((o.faces.len() - 1, Origin::StartCap { region: 0 }));
            let mut top = f.clone();
            top.surf = f.surf.transformed(&m1);
            for c in top.loops.iter_mut().flatten() {
                c.edge = top_e[c.edge];
            }
            o.faces.push(top);
            faces.push(o.faces.len() - 1);
            out.origins.push((o.faces.len() - 1, Origin::EndCap { region: 0 }));
        }
    }
    // Built facing the way the sheet faces at the start: turn it out.
    let whole_body = super::kernel::subset(o, &faces, true);
    if super::mass::body_mass(&whole_body).volume < 0.0 {
        for f in &mut o.faces {
            Body::reverse_face(f);
        }
    }
    assemble_solids(o, &faces)?;
    out.origins.sort_by_key(|(f, _)| *f);
    Ok(out)
}

/// Makes the closed shells and solids of `faces` (outward-facing): one closed shell per piece of
/// edge-connected faces; a piece enclosing a positive volume bounds a solid, one enclosing a
/// negative volume is a void of the smallest such solid around it (a profile's hole turned a
/// whole turn about an axis outside it sweeps a tunnel, not a hole).
fn assemble_solids(o: &mut Body, faces: &[usize]) -> Result<()> {
    let pieces = o.face_components(faces);
    let vols: Vec<f64> = pieces.iter().map(|p| super::mass::body_mass(&super::kernel::subset(o, p, true)).volume).collect();
    let mut outers: Vec<usize> = (0..pieces.len()).filter(|&k| vols[k] > 0.0).collect();
    outers.sort_by(|&a, &b| vols[a].total_cmp(&vols[b]));
    let mut solids: Vec<Vec<usize>> = outers.iter().map(|&k| vec![k]).collect();
    for k in (0..pieces.len()).filter(|&k| !(vols[k] > 0.0)) {
        let Some(p) = pieces[k].iter().flat_map(|&f| o.faces[f].loops.iter().flatten()).next().map(|c| o.verts[o.co_start(c)].p) else { continue };
        // The smallest solid holding it.
        let home = outers.iter().position(|&s| {
            let sub = super::kernel::subset(o, &pieces[s], true);
            let polys: Vec<_> = sub.faces.iter().map(|f| super::classify::face_polygons(&sub, f)).collect();
            super::classify::parity_inside(&sub, &polys, &p)
        });
        let Some(h) = home else {
            return Err(KernelError::OperationFailed("The sweep makes a cavity outside its solid".into()));
        };
        solids[h].push(k);
    }
    for sol in solids {
        let first = o.shells.len();
        for &k in &sol {
            o.shells.push(Shell { faces: pieces[k].clone(), closed: true });
        }
        o.solids.push(Solid { shells: (first..first + sol.len()).collect() });
    }
    Ok(())
}

/// Refuses a sheet that doesn't cross the sweep the same way everywhere (sampled on its
/// tessellation): its solid would fold through itself, or (a sheet along the sweep) be flat.
fn check_transversal(b: &Body, mv: &BodyMove) -> Result<()> {
    let size = b.bbox().diagonal().max(1e-9);
    let (meshes, _) = super::tess::mesh_body(b, size * 1e-2, 0.5);
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for m in &meshes {
        for (p, n) in m.positions.iter().zip(&m.normals) {
            let v = mv.velocity(p);
            if v.norm() < 1e-12 * size {
                continue;
            }
            let d = v.normalize().dot(&n.normalize());
            lo = lo.min(d);
            hi = hi.max(d);
        }
    }
    const EPS: f64 = 1e-6;
    if hi > EPS && lo < -EPS {
        return Err(KernelError::InvalidParameter("The sheet folds back across the sweep: its solid would cross itself".into()));
    }
    if !(hi > EPS || lo < -EPS) {
        return Err(KernelError::InvalidParameter("The sheet lies along the sweep: it sweeps no volume".into()));
    }
    Ok(())
}
