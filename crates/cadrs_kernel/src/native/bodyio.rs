//! A body as bytes and back, exactly ([`crate::Kernel::write_body`] / `read_body`): `cadrs_core`
//! caches built bodies this way and peers exchange them, and names index faces, edges and
//! vertices, so everything keeps its order and every number its bits.
//!
//! Layout (little-endian): the magic `CADRSNB1`, then the counts of vertices, edges, faces,
//! shells and solids (u32), then each list in order:
//! - vertex: x, y, z (f64);
//! - edge: curve, t0, t1, v0, v1 (u32), degenerate (u8);
//! - face: surface, sense (u8), loop count, per loop its coedge count and per coedge edge (u32),
//!   fwd (u8) and p-curve;
//! - shell: face count, faces (u32), closed (u8); solid: shell count, shells (u32).
//!
//! A curve is a tag (0 line, 1 circle, 2 ellipse, 3 NURBS) and its fields; a surface a tag
//! (0 plane, 1 cylinder, 2 cone, 3 sphere, 4 torus, 5 revolution, 6 extrusion, 7 NURBS), a frame
//! (origin, x, y, z) where it has one, and its fields. NURBS store degree(s), knots and
//! homogeneous control points.

use std::sync::Arc;

use super::curve::Curve;
use super::math::*;
use super::nurbs::{NurbsCurve, NurbsSurface, H};
use super::surface::Surface;
use super::topo::*;

const MAGIC: &[u8; 8] = b"CADRSNB1";

struct W(Vec<u8>);

impl W {
    fn u8(&mut self, x: u8) {
        self.0.push(x);
    }
    fn u32(&mut self, x: usize) {
        self.0.extend_from_slice(&(x as u32).to_le_bytes());
    }
    fn f(&mut self, x: f64) {
        self.0.extend_from_slice(&x.to_le_bytes());
    }
    fn p(&mut self, p: &P3) {
        self.f(p.x);
        self.f(p.y);
        self.f(p.z);
    }
    fn v(&mut self, v: &V3) {
        self.f(v.x);
        self.f(v.y);
        self.f(v.z);
    }
    fn h(&mut self, h: &H) {
        for k in 0..4 {
            self.f(h[k]);
        }
    }
    fn frame(&mut self, f: &Frame) {
        self.p(&f.o);
        self.v(&f.x);
        self.v(&f.y);
        self.v(&f.z);
    }
    fn knots(&mut self, k: &[f64]) {
        self.u32(k.len());
        for x in k {
            self.f(*x);
        }
    }
    fn curve(&mut self, c: &Curve) {
        match c {
            Curve::Line { o, d } => {
                self.u8(0);
                self.p(o);
                self.v(d);
            }
            Curve::Circle { c, x, y, r } => {
                self.u8(1);
                self.p(c);
                self.v(x);
                self.v(y);
                self.f(*r);
            }
            Curve::Ellipse { c, x, y, a, b } => {
                self.u8(2);
                self.p(c);
                self.v(x);
                self.v(y);
                self.f(*a);
                self.f(*b);
            }
            Curve::Nurbs(n) => {
                self.u8(3);
                self.u32(n.degree);
                self.knots(&n.knots);
                self.u32(n.cps.len());
                for h in &n.cps {
                    self.h(h);
                }
            }
        }
    }
    fn surface(&mut self, s: &Surface) {
        match s {
            Surface::Plane { f } => {
                self.u8(0);
                self.frame(f);
            }
            Surface::Cylinder { f, r } => {
                self.u8(1);
                self.frame(f);
                self.f(*r);
            }
            Surface::Cone { f, r, a } => {
                self.u8(2);
                self.frame(f);
                self.f(*r);
                self.f(*a);
            }
            Surface::Sphere { f, r } => {
                self.u8(3);
                self.frame(f);
                self.f(*r);
            }
            Surface::Torus { f, rmaj, rmin } => {
                self.u8(4);
                self.frame(f);
                self.f(*rmaj);
                self.f(*rmin);
            }
            Surface::Revolution { f, m } => {
                self.u8(5);
                self.frame(f);
                self.curve(m);
            }
            Surface::Extrusion { c, d } => {
                self.u8(6);
                self.curve(c);
                self.v(d);
            }
            Surface::Nurbs(n) => {
                self.u8(7);
                self.u32(n.du);
                self.u32(n.dv);
                self.knots(&n.ku);
                self.knots(&n.kv);
                self.u32(n.cps.len());
                self.u32(n.cps.first().map_or(0, |r| r.len()));
                for row in &n.cps {
                    for h in row {
                        self.h(h);
                    }
                }
            }
        }
    }
}

/// The body as bytes.
pub fn write(b: &Body) -> Vec<u8> {
    let mut w = W(MAGIC.to_vec());
    for n in [b.verts.len(), b.edges.len(), b.faces.len(), b.shells.len(), b.solids.len()] {
        w.u32(n);
    }
    for v in &b.verts {
        w.p(&v.p);
    }
    for e in &b.edges {
        w.curve(&e.curve);
        w.f(e.t[0]);
        w.f(e.t[1]);
        w.u32(e.v[0]);
        w.u32(e.v[1]);
        w.u8(e.degenerate as u8);
    }
    for f in &b.faces {
        w.surface(&f.surf);
        w.u8(f.sense as u8);
        w.u32(f.loops.len());
        for l in &f.loops {
            w.u32(l.len());
            for c in l {
                w.u32(c.edge);
                w.u8(c.fwd as u8);
                w.curve(&c.pc);
            }
        }
    }
    for s in &b.shells {
        w.u32(s.faces.len());
        for &f in &s.faces {
            w.u32(f);
        }
        w.u8(s.closed as u8);
    }
    for s in &b.solids {
        w.u32(s.shells.len());
        for &x in &s.shells {
            w.u32(x);
        }
    }
    w.0
}

struct Rd<'a> {
    b: &'a [u8],
    i: usize,
}

type Res<T> = Result<T, String>;

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> Res<&'a [u8]> {
        let s = self.b.get(self.i..self.i + n).ok_or("the body data is cut short")?;
        self.i += n;
        Ok(s)
    }
    fn u8(&mut self) -> Res<u8> {
        Ok(self.take(1)?[0])
    }
    fn bool(&mut self) -> Res<bool> {
        Ok(self.u8()? != 0)
    }
    fn u32(&mut self) -> Res<usize> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()) as usize)
    }
    /// A count, checked against the bytes left (each item at least `min` bytes).
    fn count(&mut self, min: usize) -> Res<usize> {
        let n = self.u32()?;
        if n.saturating_mul(min) > self.b.len() - self.i {
            return Err("a count runs past the end of the body data".into());
        }
        Ok(n)
    }
    fn f(&mut self) -> Res<f64> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn p(&mut self) -> Res<P3> {
        Ok(P3::new(self.f()?, self.f()?, self.f()?))
    }
    fn v(&mut self) -> Res<V3> {
        Ok(V3::new(self.f()?, self.f()?, self.f()?))
    }
    fn h(&mut self) -> Res<H> {
        Ok(H::new(self.f()?, self.f()?, self.f()?, self.f()?))
    }
    fn frame(&mut self) -> Res<Frame> {
        Ok(Frame { o: self.p()?, x: self.v()?, y: self.v()?, z: self.v()? })
    }
    fn knots(&mut self) -> Res<Vec<f64>> {
        let n = self.count(8)?;
        (0..n).map(|_| self.f()).collect()
    }
    fn curve(&mut self) -> Res<Curve> {
        Ok(match self.u8()? {
            0 => Curve::Line { o: self.p()?, d: self.v()? },
            1 => Curve::Circle { c: self.p()?, x: self.v()?, y: self.v()?, r: self.f()? },
            2 => Curve::Ellipse { c: self.p()?, x: self.v()?, y: self.v()?, a: self.f()?, b: self.f()? },
            3 => {
                let degree = self.u32()?;
                let knots = self.knots()?;
                let n = self.count(32)?;
                let cps = (0..n).map(|_| self.h()).collect::<Res<Vec<_>>>()?;
                if n < degree + 1 || knots.len() != n + degree + 1 {
                    return Err("a NURBS curve with bad knots".into());
                }
                Curve::Nurbs(Arc::new(NurbsCurve { degree, knots, cps }))
            }
            t => return Err(format!("unknown curve tag {t}")),
        })
    }
    fn surface(&mut self) -> Res<Surface> {
        Ok(match self.u8()? {
            0 => Surface::Plane { f: self.frame()? },
            1 => Surface::Cylinder { f: self.frame()?, r: self.f()? },
            2 => Surface::Cone { f: self.frame()?, r: self.f()?, a: self.f()? },
            3 => Surface::Sphere { f: self.frame()?, r: self.f()? },
            4 => Surface::Torus { f: self.frame()?, rmaj: self.f()?, rmin: self.f()? },
            5 => Surface::Revolution { f: self.frame()?, m: self.curve()? },
            6 => Surface::Extrusion { c: self.curve()?, d: self.v()? },
            7 => {
                let (du, dv) = (self.u32()?, self.u32()?);
                let (ku, kv) = (self.knots()?, self.knots()?);
                let (nu, nv) = (self.count(1)?, self.count(1)?);
                if nu.saturating_mul(nv).saturating_mul(32) > self.b.len() - self.i {
                    return Err("a NURBS surface runs past the end of the body data".into());
                }
                let mut cps = Vec::with_capacity(nu);
                for _ in 0..nu {
                    cps.push((0..nv).map(|_| self.h()).collect::<Res<Vec<_>>>()?);
                }
                if nu < du + 1 || nv < dv + 1 || ku.len() != nu + du + 1 || kv.len() != nv + dv + 1 {
                    return Err("a NURBS surface with bad knots".into());
                }
                Surface::Nurbs(Arc::new(NurbsSurface { du, dv, ku, kv, cps }))
            }
            t => return Err(format!("unknown surface tag {t}")),
        })
    }
}

/// A body from bytes written by [`write`].
pub fn read(bytes: &[u8]) -> Result<Body, String> {
    if bytes.len() < 28 || &bytes[..8] != MAGIC {
        return Err("not a native kernel body".into());
    }
    let mut r = Rd { b: bytes, i: 8 };
    let (nv, ne, nf, ns, nso) = (r.count(24)?, r.count(30)?, r.count(6)?, r.count(5)?, r.count(4)?);
    let mut b = Body::default();
    for _ in 0..nv {
        b.verts.push(Vertex { p: r.p()? });
    }
    for _ in 0..ne {
        let curve = r.curve()?;
        let t = [r.f()?, r.f()?];
        let v = [r.u32()?, r.u32()?];
        if v[0] >= nv || v[1] >= nv {
            return Err("an edge's vertex is out of range".into());
        }
        b.edges.push(Edge { curve, t, v, degenerate: r.bool()? });
    }
    for _ in 0..nf {
        let surf = r.surface()?;
        let sense = r.bool()?;
        let nl = r.count(4)?;
        let mut loops = Vec::with_capacity(nl);
        for _ in 0..nl {
            let nc = r.count(6)?;
            let mut l = Vec::with_capacity(nc);
            for _ in 0..nc {
                let edge = r.u32()?;
                if edge >= ne {
                    return Err("a coedge's edge is out of range".into());
                }
                l.push(Coedge { edge, fwd: r.bool()?, pc: r.curve()? });
            }
            loops.push(l);
        }
        b.faces.push(Face { surf, sense, loops });
    }
    for _ in 0..ns {
        let n = r.count(4)?;
        let faces = (0..n).map(|_| r.u32()).collect::<Res<Vec<_>>>()?;
        if faces.iter().any(|&f| f >= nf) {
            return Err("a shell's face is out of range".into());
        }
        b.shells.push(Shell { faces, closed: r.bool()? });
    }
    for _ in 0..nso {
        let n = r.count(4)?;
        let shells = (0..n).map(|_| r.u32()).collect::<Res<Vec<_>>>()?;
        if shells.iter().any(|&s| s >= ns) {
            return Err("a solid's shell is out of range".into());
        }
        b.solids.push(Solid { shells });
    }
    if r.i != bytes.len() {
        return Err("trailing bytes after the body".into());
    }
    Ok(b)
}
