//! Sweeping planar faces into solids: prisms (a straight sweep) and revolutions (a sweep about
//! an axis), one generic builder.
//!
//! A profile point q traces a *rail* as the sweep parameter s runs from s0 to s1 (a straight
//! line, or a circle about the axis); each profile piece sweeps a side face; the profile itself
//! is the start cap at s0 and the end cap at s1 (none for a whole turn).

use super::build::{fix_senses, Built, MIN_DEPTH};
use super::curve::Curve;
use super::math::*;
use super::profile::{MergedFace, Piece};
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
    let mut solid_shells = Vec::new();
    for mf in faces {
        let face_start = b.faces.len();
        let mut bottom_loops: Vec<Vec<Coedge>> = Vec::new();
        let mut top_loops: Vec<Vec<Coedge>> = Vec::new();
        let mut sides: Vec<(usize, (u64, u64))> = Vec::new();
        let loops: Vec<&Vec<Piece>> = std::iter::once(&mf.outer).chain(mf.holes.iter()).collect();
        for lp in loops {
            let k = lp.len();
            let rails: Vec<Option<Curve>> = lp.iter().map(|p| kind.rail(p.start())).collect();
            let vb: Vec<usize> = lp.iter().map(|p| {
                let q = p.start();
                b.add_vertex(m0.point(&P3::new(q.x, q.y, 0.0)))
            }).collect();
            let vt: Vec<usize> = (0..k)
                .map(|j| {
                    if whole || rails[j].is_none() {
                        vb[j]
                    } else {
                        let q = lp[j].start();
                        b.add_vertex(m1.point(&P3::new(q.x, q.y, 0.0)))
                    }
                })
                .collect();
            // Rails at the joints: an edge, or a degenerate edge on the axis.
            let rail_e: Vec<(usize, bool)> = (0..k)
                .map(|j| match &rails[j] {
                    Some(c) => (b.add_edge(c.clone(), s0, s1, vb[j], vt[j]), false),
                    None => (b.add_degenerate(b.verts[vb[j]].p, s0, s1, vb[j]), true),
                })
                .collect();
            let mut bl = Vec::new();
            let mut tl = Vec::new();
            for (i, p) in lp.iter().enumerate() {
                let j = (i + 1) % k;
                let c0 = p.c.transformed(&m0);
                let (v0b, v1b) = if p.rev { (vb[j], vb[i]) } else { (vb[i], vb[j]) };
                let (v0t, v1t) = if p.rev { (vt[j], vt[i]) } else { (vt[i], vt[j]) };
                let Some(side) = kind.side(p, &p.c.transformed(&kind.place(0.0))) else {
                    // On the axis: the start and end profile edges coincide.
                    let e = b.add_edge(c0.clone(), p.t0, p.t1, v0b, v1b);
                    bl.push(Coedge { edge: e, fwd: !p.rev, pc: p.c.clone() });
                    tl.push(Coedge { edge: e, fwd: !p.rev, pc: p.c.clone() });
                    continue;
                };
                let eb = b.add_edge(c0.clone(), p.t0, p.t1, v0b, v1b);
                let et = if whole { eb } else { b.add_edge(p.c.transformed(&m1), p.t0, p.t1, v0t, v1t) };
                bl.push(Coedge { edge: eb, fwd: !p.rev, pc: p.c.clone() });
                tl.push(Coedge { edge: et, fwd: !p.rev, pc: p.c.clone() });
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
                sides.push((b.faces.len() - 1, p.src));
            }
            bottom_loops.push(bl);
            top_loops.push(tl);
        }
        let n_sides = b.faces.len() - face_start;
        let rev_loop = |l: &mut Vec<Coedge>| {
            l.reverse();
            for c in l.iter_mut() {
                c.fwd = !c.fwd;
            }
        };
        if !whole {
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
        for (fi, (region, curve)) in sides {
            out.origins.push((fi, Origin::ProfileCurve { region, curve }));
        }
        fix_senses(b, face_start..face_start + n_sides);
        let fs: Vec<usize> = (face_start..b.faces.len()).collect();
        b.shells.push(Shell { faces: fs, closed: true });
        solid_shells.push(b.shells.len() - 1);
    }
    for s in solid_shells {
        b.solids.push(Solid { shells: vec![s] });
    }
    out.origins.sort_by_key(|(f, _)| *f);
    Ok(out)
}
