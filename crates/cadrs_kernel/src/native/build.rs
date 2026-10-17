//! Building bodies from profiles: planar faces, prisms (extrudes).

use super::curve::Curve;
use super::math::*;
use super::profile::{MergedFace, Piece};
use super::surface::Surface;
use super::topo::*;
use crate::{KernelError, Origin, Result};

/// A body and the origins of its faces (a face may have several: a merged cap).
#[derive(Debug, Clone, Default)]
pub struct Built {
    pub body: Body,
    pub origins: Vec<(usize, Origin)>,
}

/// The extrude depth below which the kernel refuses to build (mm).
pub const MIN_DEPTH: f64 = 1e-6;

/// The map from sketch coordinates (z = 0) to model space at `s` along `dir`.
pub fn plane_map(plane: &crate::Plane, dir: &V3, s: f64) -> Affine {
    let (x, y, n) = (plane.x_dir.into_inner(), plane.y_dir().into_inner(), plane.normal.into_inner());
    Affine { m: nalgebra::Matrix3::from_columns(&[x, y, n]), t: plane.origin.coords + dir * s }
}

pub fn plane_frame(plane: &crate::Plane, offset: V3) -> Frame {
    Frame { o: plane.origin + offset, x: plane.x_dir.into_inner(), y: plane.y_dir().into_inner(), z: plane.normal.into_inner() }
}

/// Signed area in uv of a face's first loop, by sampling its p-curves.
pub fn loop_uv_area(body: &Body, l: &[Coedge]) -> f64 {
    let mut pts: Vec<P2> = Vec::new();
    for c in l {
        let e = &body.edges[c.edge];
        let (a, b) = co_range(e, c.fwd);
        let n = 32;
        for i in 0..n {
            let t = a + (b - a) * i as f64 / n as f64;
            let p = c.pc.point(t);
            pts.push(P2::new(p.x, p.y));
        }
    }
    polygon_area(&pts)
}

/// Sets each face's sense from the uv orientation of its outer loop (loops run CCW about the
/// outward normal, so an outer loop that is CCW in uv means outward = Su × Sv).
pub fn fix_senses(body: &mut Body, faces: impl IntoIterator<Item = usize>) {
    for fi in faces {
        let a = loop_uv_area(body, &body.faces[fi].loops[0]);
        body.faces[fi].sense = a > 0.0;
    }
}

/// A side surface swept along `dir` by a piece's 3D curve, with the affine map (t, s) → (u, v).
struct Side {
    surf: Surface,
    /// uv(t, s) = o + a·t + b·s
    o: P2,
    a: V2,
    b: V2,
}

impl Side {
    fn uv(&self, t: f64, s: f64) -> P2 {
        self.o + self.a * t + self.b * s
    }
    /// The p-curve of the curve at height s (param t).
    fn along(&self, s: f64) -> Curve {
        let o = self.uv(0.0, s);
        Curve::Line { o: P3::new(o.x, o.y, 0.0), d: V3::new(self.a.x, self.a.y, 0.0) }
    }
    /// The p-curve of the straight line at t (param s).
    fn across(&self, t: f64) -> Curve {
        let o = self.uv(t, 0.0);
        Curve::Line { o: P3::new(o.x, o.y, 0.0), d: V3::new(self.b.x, self.b.y, 0.0) }
    }
}

fn side_surface(c3: &Curve, dir: &V3) -> Side {
    match c3 {
        Curve::Line { o, d } => {
            let x = d.normalize();
            let mut y = dir - x * x.dot(dir);
            y.normalize_mut();
            let z = x.cross(&y);
            let f = Frame { o: *o, x, y, z };
            // C(t) + s·dir = o + x·t·|d| + s(x(x·dir) + y(y·dir)).
            Side { surf: Surface::Plane { f }, o: P2::origin(), a: V2::new(d.norm(), 0.0), b: V2::new(x.dot(dir), y.dot(dir)) }
        }
        Curve::Circle { c, x, y, r } if x.cross(y).cross(dir).norm() < 1e-12 => {
            let z = x.cross(y);
            let f = Frame { o: *c, x: *x, y: *y, z };
            Side { surf: Surface::Cylinder { f, r: *r }, o: P2::origin(), a: V2::new(1.0, 0.0), b: V2::new(0.0, z.dot(dir)) }
        }
        _ => Side { surf: Surface::Extrusion { c: c3.clone(), d: *dir }, o: P2::origin(), a: V2::new(1.0, 0.0), b: V2::new(0.0, 1.0) },
    }
}

/// Lifts a 2D curve (z = 0, sketch coordinates) by an affine map.
pub fn lift(c: &Curve, m: &Affine) -> Curve {
    c.transformed(m)
}

/// The prism swept by planar faces (in `plane`'s coordinates) along the unit `dir` from `s0`
/// to `s1` (s0 < s1). The caps at s0 are `StartCap`s, at s1 `EndCap`s.
pub fn prism(plane: &crate::Plane, faces: &[MergedFace], dir: V3, s0: f64, s1: f64) -> Result<Built> {
    let h = s1 - s0;
    if h.abs() < MIN_DEPTH {
        return Err(KernelError::InvalidParameter(format!(
            "The depth {h:e} mm is below the modelling tolerance ({MIN_DEPTH:e} mm)"
        )));
    }
    let n = plane.normal.into_inner();
    let dn = dir.dot(&n);
    if dn.abs() < 1e-9 {
        return Err(KernelError::InvalidParameter("The extrude direction lies in the sketch plane".into()));
    }
    let up = dn > 0.0;
    let m0 = plane_map(plane, &dir, s0);
    let m1 = plane_map(plane, &dir, s1);
    let mut out = Built::default();
    let b = &mut out.body;
    let mut shell_faces = Vec::new();
    let mut solids_shells = Vec::new();
    for mf in faces {
        let face_start = b.faces.len();
        let mut bottom_loops: Vec<Vec<Coedge>> = Vec::new();
        let mut top_loops: Vec<Vec<Coedge>> = Vec::new();
        let mut side_faces: Vec<(usize, (u64, u64))> = Vec::new();
        let loops: Vec<&Vec<Piece>> = std::iter::once(&mf.outer).chain(mf.holes.iter()).collect();
        for lp in loops {
            let k = lp.len();
            // Joint vertices.
            let vb: Vec<usize> = lp.iter().map(|p| {
                let q = p.start();
                b.add_vertex(m0.point(&P3::new(q.x, q.y, 0.0)))
            }).collect();
            let vt: Vec<usize> = lp.iter().map(|p| {
                let q = p.start();
                b.add_vertex(m1.point(&P3::new(q.x, q.y, 0.0)))
            }).collect();
            // Vertical edges at the joints.
            let ve: Vec<usize> = (0..k)
                .map(|j| {
                    let o = b.verts[vb[j]].p;
                    b.add_edge(Curve::Line { o, d: dir }, 0.0, h, vb[j], vt[j])
                })
                .collect();
            let mut bl = Vec::new();
            let mut tl = Vec::new();
            for (i, p) in lp.iter().enumerate() {
                let j = (i + 1) % k;
                let c0 = lift(&p.c, &m0);
                let c1 = lift(&p.c, &m1);
                // Edge natural direction is increasing t.
                let (v0b, v1b) = if p.rev { (vb[j], vb[i]) } else { (vb[i], vb[j]) };
                let (v0t, v1t) = if p.rev { (vt[j], vt[i]) } else { (vt[i], vt[j]) };
                let eb = b.add_edge(c0.clone(), p.t0, p.t1, v0b, v1b);
                let et = b.add_edge(c1, p.t0, p.t1, v0t, v1t);
                // Cap coedges (in the piece's direction; reversed later for the bottom/top).
                bl.push(Coedge { edge: eb, fwd: !p.rev, pc: p.c.clone() });
                tl.push(Coedge { edge: et, fwd: !p.rev, pc: p.c.clone() });
                // Side face.
                let side = side_surface(&c0, &dir);
                let (ts, te) = (p.ts(), p.te());
                let bottom = Coedge { edge: eb, fwd: !p.rev, pc: side.along(0.0) };
                let top = Coedge { edge: et, fwd: p.rev, pc: side.along(h) };
                let v_end = Coedge { edge: ve[j], fwd: true, pc: side.across(te) };
                let v_start = Coedge { edge: ve[i], fwd: false, pc: side.across(ts) };
                let mut sl = vec![bottom, v_end, top, v_start];
                if !up {
                    sl.reverse();
                    for c in &mut sl {
                        c.fwd = !c.fwd;
                    }
                }
                b.faces.push(Face { surf: side.surf, sense: true, loops: vec![sl] });
                side_faces.push((b.faces.len() - 1, p.src));
            }
            bottom_loops.push(bl);
            top_loops.push(tl);
        }
        let rev_loop = |l: &mut Vec<Coedge>| {
            l.reverse();
            for c in l.iter_mut() {
                c.fwd = !c.fwd;
            }
        };
        if up {
            bottom_loops.iter_mut().for_each(rev_loop);
        } else {
            top_loops.iter_mut().for_each(rev_loop);
        }
        let fb = Face { surf: Surface::Plane { f: plane_frame(plane, dir * s0) }, sense: !up, loops: bottom_loops };
        let ft = Face { surf: Surface::Plane { f: plane_frame(plane, dir * s1) }, sense: up, loops: top_loops };
        b.faces.push(fb);
        let ib = b.faces.len() - 1;
        b.faces.push(ft);
        let it = b.faces.len() - 1;
        for r in &mf.regions {
            out.origins.push((ib, Origin::StartCap { region: *r }));
            out.origins.push((it, Origin::EndCap { region: *r }));
        }
        for (fi, (region, curve)) in side_faces {
            out.origins.push((fi, Origin::ProfileCurve { region, curve }));
        }
        let fs: Vec<usize> = (face_start..b.faces.len()).collect();
        fix_senses(b, fs[..fs.len() - 2].iter().copied());
        b.shells.push(Shell { faces: fs.clone(), closed: true });
        solids_shells.push(b.shells.len() - 1);
        shell_faces.extend(fs);
    }
    for s in solids_shells {
        b.solids.push(Solid { shells: vec![s] });
    }
    out.origins.sort_by_key(|(f, _)| *f);
    Ok(out)
}
