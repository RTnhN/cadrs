//! Writing OCCT's BRep text format (what `BRepTools::Read` and DRAW's `restore` read, and
//! [`super::brep_occt`] reads back): bodies as one compound of their solids, sheets, wires and
//! points, with exact geometry.
//!
//! **The file.** Version 1 (`CASCADE Topology V1`), no locations (the geometry is where the
//! bodies are), then the tables: `Curve2ds` (each coedge's p-curve), `Curves` (each edge's 3D
//! curve), `Surfaces` (each face's) and `TShapes`, each shape listed after the shapes it holds.
//!
//! **Geometry.** Our curves and surfaces are OCCT's, parametrized the same way: lines, circles
//! and ellipses, B-splines (poles in Cartesian coordinates with weights); planes, cylinders,
//! cones, spheres, tori, surfaces of revolution and of linear extrusion, B-spline surfaces.
//! Where OCCT's form is narrower than ours the curve is rewritten without changing a point of
//! it: a line whose direction isn't a unit vector becomes a degree-1 B-spline over its range,
//! an ellipse whose second axis is the longer turns its axes a quarter turn (its edge's range
//! and p-curves shift with it), and an extrusion along a vector that isn't a unit vector
//! scales its p-curves' v. A p-curve with no exact form after that (an ellipse whose second axis
//! is the longer, a circle stretched along v) becomes a C1 cubic B-spline through 64 Hermite
//! segments with the edge's parameter, within about 1e-9 of its size.
//!
//! **Topology.** Edges carry their tolerance, their 3D curve and a p-curve per face (two on a
//! seam: the first for the edge's forward use in the face), all on the edge's range (same
//! parameter, same range); degenerate edges carry only p-curves. Vertices take the largest
//! tolerance of their edges. A face whose outward normal is its surface's natural normal is used
//! forward by its shell; another is written with its loops turned round and used reversed (as
//! OCCT builds such faces). A solid's void shells are written as they are, facing into the void.

use std::fmt::Write as _;

use super::curve::Curve;
use super::math::*;
use super::nurbs::{dehom, NurbsCurve};
use super::surface::Surface;
use super::topo::{Body, Coedge, TOL_MIN};
use super::xbuild::knot_mults;

/// A real as the file holds it: the shortest digits that read back to the same value.
fn num(x: f64) -> String {
    if !x.is_finite() {
        return "0".into();
    }
    let a = x.abs();
    if a == 0.0 || (1e-5..1e15).contains(&a) { format!("{x}") } else { format!("{x:e}") }
}

fn p3(p: &P3) -> String {
    format!("{} {} {}", num(p.x), num(p.y), num(p.z))
}

fn v3(v: &V3) -> String {
    format!("{} {} {}", num(v.x), num(v.y), num(v.z))
}

fn unit(v: &V3) -> bool {
    (v.norm() - 1.0).abs() <= 1e-12
}

/// A B-spline curve's record (type 7): 2D (x, y) or 3D poles, weights when rational.
fn bspline(n: &NurbsCurve, dim2: bool) -> String {
    let rational = n.cps.iter().any(|h| (h.w - 1.0).abs() > 1e-15);
    let (k, m) = knot_mults(&n.knots);
    let mut s = format!("7 {} 0 {} {} {}", u8::from(rational), n.degree, n.cps.len(), k.len());
    for h in &n.cps {
        let p = dehom(h);
        let _ = if dim2 { write!(s, " {} {}", num(p.x), num(p.y)) } else { write!(s, " {}", p3(&p)) };
        if rational {
            let _ = write!(s, " {}", num(h.w));
        }
    }
    for (x, m) in k.iter().zip(&m) {
        let _ = write!(s, " {} {m}", num(*x));
    }
    s
}

/// The degree-1 B-spline that is `c` on [t0, t1] (a line).
fn linear(c: &Curve, t0: f64, t1: f64) -> NurbsCurve {
    let h = |p: P3| super::nurbs::hom(&p, 1.0);
    NurbsCurve { degree: 1, knots: vec![t0, t0, t1, t1], cps: vec![h(c.point(t0)), h(c.point(t1))] }
}

/// A C1 cubic B-spline through `segs` Hermite pieces of `f` (point and derivative) on [t0, t1],
/// parametrized as `f`.
fn hermite(f: impl Fn(f64) -> (P3, V3), t0: f64, t1: f64, segs: usize) -> NurbsCurve {
    let h = |p: P3| super::nurbs::hom(&p, 1.0);
    let mut knots = vec![t0; 4];
    let mut cps = Vec::with_capacity(3 * segs + 1);
    let ts: Vec<f64> = (0..=segs).map(|i| t0 + (t1 - t0) * i as f64 / segs as f64).collect();
    for (i, w) in ts.windows(2).enumerate() {
        let (a, b) = (w[0], w[1]);
        let ((pa, da), (pb, db)) = (f(a), f(b));
        let k = (b - a) / 3.0;
        if i == 0 {
            cps.push(h(pa));
        }
        cps.extend([h(pa + da * k), h(pb - db * k), h(pb)]);
        knots.extend(std::iter::repeat_n(b, if i + 1 == segs { 4 } else { 3 }));
    }
    NurbsCurve { degree: 3, knots, cps }
}

/// `c` as `c'(t) = c(t + h)` (h a quarter turn for conics; any shift for lines and splines).
fn shifted(c: &Curve, h: f64) -> Curve {
    match c {
        Curve::Line { o, d } => Curve::Line { o: o + d * h, d: *d },
        Curve::Circle { c, x, y, r } => {
            let (s, k) = h.sin_cos();
            Curve::Circle { c: *c, x: x * k + y * s, y: y * k - x * s, r: *r }
        }
        // h = π/2: a cos(t + π/2) x + b sin(t + π/2) y = b cos t y + a sin t (−x).
        Curve::Ellipse { c, x, y, a, b } => Curve::Ellipse { c: *c, x: *y, y: -x, a: *b, b: *a },
        Curve::Nurbs(n) => {
            let mut n = NurbsCurve::clone(n);
            n.knots.iter_mut().for_each(|k| *k -= h);
            Curve::nurbs(n)
        }
    }
}

/// How our (u, v) of a surface become OCCT's: u' = uk·u + uo, v' = vk·v + vo.
#[derive(Debug, Clone, Copy)]
struct UvMap {
    uk: f64,
    uo: f64,
    vk: f64,
    vo: f64,
}

impl UvMap {
    const SAME: UvMap = UvMap { uk: 1.0, uo: 0.0, vk: 1.0, vo: 0.0 };
    fn p(&self, p: &P3) -> P3 {
        P3::new(self.uk * p.x + self.uo, self.vk * p.y + self.vo, 0.0)
    }
    fn v(&self, d: &V3) -> V3 {
        V3::new(self.uk * d.x, self.vk * d.y, 0.0)
    }
    /// Whether it only moves (u, v) (lengths and angles kept).
    fn rigid(&self) -> bool {
        self.uk == 1.0 && self.vk == 1.0
    }
}

/// A p-curve's record on [t0, t1], in OCCT's (u, v) of its surface.
fn curve2d(pc: &Curve, t0: f64, t1: f64, m: &UvMap) -> String {
    match pc {
        Curve::Line { o, d } => {
            let (o, d) = (m.p(o), m.v(d));
            if unit(&d) {
                format!("1 {} {} {} {}", num(o.x), num(o.y), num(d.x), num(d.y))
            } else {
                bspline(&linear(&Curve::Line { o, d }, t0, t1), true)
            }
        }
        Curve::Circle { c, x, y, r } if m.rigid() => {
            let c = m.p(c);
            format!("2 {} {} {} {} {} {} {}", num(c.x), num(c.y), num(x.x), num(x.y), num(y.x), num(y.y), num(*r))
        }
        Curve::Ellipse { c, x, y, a, b } if m.rigid() && a >= b => {
            let c = m.p(c);
            format!("3 {} {} {} {} {} {} {} {}", num(c.x), num(c.y), num(x.x), num(x.y), num(y.x), num(y.y), num(*a), num(*b))
        }
        Curve::Nurbs(n) => {
            let mut n = NurbsCurve::clone(n);
            for h in &mut n.cps {
                (h.x, h.y) = (m.uk * h.x + m.uo * h.w, m.vk * h.y + m.vo * h.w);
            }
            bspline(&n, true)
        }
        _ => {
            let f = |t: f64| {
                let (p, d, _) = pc.d2(t);
                (m.p(&p), m.v(&d))
            };
            bspline(&hermite(f, t0.min(t1), t0.max(t1), 64), true)
        }
    }
}

/// A 3D curve's record (the curve as OCCT can hold it: see [`edge_curve`]).
fn curve3d(c: &Curve) -> String {
    match c {
        Curve::Line { o, d } => format!("1 {} {}", p3(o), v3(d)),
        Curve::Circle { c, x, y, r } => format!("2 {} {} {} {} {}", p3(c), v3(&x.cross(y).normalize()), v3(x), v3(y), num(*r)),
        Curve::Ellipse { c, x, y, a, b } => format!("3 {} {} {} {} {} {}", p3(c), v3(&x.cross(y).normalize()), v3(x), v3(y), num(*a), num(*b)),
        Curve::Nurbs(n) => bspline(n, false),
    }
}

/// An edge's curve as OCCT can hold it, and the shift of its parameter (t' = t − shift, which
/// its p-curves follow): a line's direction a unit vector, an ellipse's first axis the longer.
fn edge_curve(c: &Curve, t0: f64, t1: f64) -> (Curve, f64) {
    match c {
        Curve::Line { d, .. } if !unit(d) => (Curve::nurbs(linear(c, t0, t1)), 0.0),
        Curve::Ellipse { a, b, .. } if b > a => (shifted(c, std::f64::consts::FRAC_PI_2), std::f64::consts::FRAC_PI_2),
        _ => (c.clone(), 0.0),
    }
}

/// A surface's basis curve as OCCT can hold it, and its parameter's map t' = k·t + o: a line
/// along a unit vector, an ellipse's first axis the longer.
fn basis_curve(c: &Curve) -> (Curve, f64, f64) {
    match c {
        Curve::Line { o, d } if !unit(d) => (Curve::Line { o: *o, d: d.normalize() }, d.norm(), 0.0),
        Curve::Ellipse { a, b, .. } if b > a => (shifted(c, std::f64::consts::FRAC_PI_2), 1.0, -std::f64::consts::FRAC_PI_2),
        _ => (c.clone(), 1.0, 0.0),
    }
}

/// A surface's record, and how our (u, v) on it become OCCT's.
fn surface(s: &Surface) -> (String, UvMap) {
    let ax = |f: &Frame| format!("{} {} {} {}", p3(&f.o), v3(&f.z), v3(&f.x), v3(&f.y));
    let same = UvMap::SAME;
    match s {
        Surface::Plane { f } => (format!("1 {}", ax(f)), same),
        Surface::Cylinder { f, r } => (format!("2 {} {}", ax(f), num(*r)), same),
        Surface::Cone { f, r, a } => (format!("3 {} {} {}", ax(f), num(*r), num(*a)), same),
        Surface::Sphere { f, r } => (format!("4 {} {}", ax(f), num(*r)), same),
        Surface::Torus { f, rmaj, rmin } => (format!("5 {} {} {}", ax(f), num(*rmaj), num(*rmin)), same),
        // Turned about x × y (the way u turns the meridian).
        Surface::Revolution { f, m } => {
            let (m, k, o) = basis_curve(m);
            (format!("7 {} {} {}", p3(&f.o), v3(&f.x.cross(&f.y).normalize()), curve3d(&m)), UvMap { vk: k, vo: o, ..same })
        }
        Surface::Extrusion { c, d } => {
            let (c, k, o) = basis_curve(c);
            let len = d.norm();
            (format!("6 {} {}", v3(&(d / len)), curve3d(&c)), UvMap { uk: k, uo: o, vk: len, vo: 0.0 })
        }
        Surface::Nurbs(n) => {
            let rational = n.cps.iter().flatten().any(|h| (h.w - 1.0).abs() > 1e-15);
            let (ku, mu) = knot_mults(&n.ku);
            let (kv, mv) = knot_mults(&n.kv);
            let r = u8::from(rational);
            let nv = n.cps.first().map_or(0, Vec::len);
            let mut s = format!("9 {r} {r} 0 0 {} {} {} {} {} {}", n.du, n.dv, n.cps.len(), nv, ku.len(), kv.len());
            for h in n.cps.iter().flatten() {
                let _ = write!(s, " {}", p3(&dehom(h)));
                if rational {
                    let _ = write!(s, " {}", num(h.w));
                }
            }
            for (x, m) in ku.iter().zip(&mu).chain(kv.iter().zip(&mv)) {
                let _ = write!(s, " {} {m}", num(*x));
            }
            (s, same)
        }
    }
}

/// A shape of the `TShapes` table: its record (type, geometry, flags) and sub-shapes
/// (orientation, index into the table).
struct TShape {
    head: String,
    subs: Vec<(bool, usize)>,
}

#[derive(Default)]
struct Writer {
    curves2d: Vec<String>,
    curves: Vec<String>,
    surfaces: Vec<String>,
    shapes: Vec<TShape>,
}

impl Writer {
    fn shape(&mut self, head: String, subs: Vec<(bool, usize)>) -> usize {
        self.shapes.push(TShape { head, subs });
        self.shapes.len() - 1
    }

    /// Adds a body's shapes; returns its top-level pieces (solids, free shells, wires, points).
    fn body(&mut self, b: &Body) -> Vec<usize> {
        // Each edge's uses: (face, coedge).
        let mut uses: Vec<Vec<(usize, &Coedge)>> = vec![Vec::new(); b.edges.len()];
        for (fi, f) in b.faces.iter().enumerate() {
            for c in f.loops.iter().flatten() {
                uses[c.edge].push((fi, c));
            }
        }
        let mut vtol: Vec<f64> = b.verts.iter().map(|v| v.tol.max(TOL_MIN)).collect();
        for e in &b.edges {
            for v in e.v {
                vtol[v] = vtol[v].max(e.tol);
            }
        }
        let mut surf_of = Vec::with_capacity(b.faces.len());
        for f in &b.faces {
            let (rec, map) = surface(&f.surf);
            self.surfaces.push(rec);
            surf_of.push((self.surfaces.len(), map));
        }
        let vid: Vec<usize> = b
            .verts
            .iter()
            .zip(&vtol)
            .map(|(v, t)| self.shape(format!("Ve\n{}\n{}\n0 0\n\n0101101", num(*t), p3(&v.p)), Vec::new()))
            .collect();
        let mut eid = Vec::with_capacity(b.edges.len());
        for (ei, e) in b.edges.iter().enumerate() {
            let (t0, t1) = (e.t[0], e.t[1]);
            let (curve, h) = edge_curve(&e.curve, t0, t1);
            let (a, z) = (t0 - h, t1 - h);
            let mut head = format!("Ed\n {} 1 1 {}\n", num(e.tol.max(TOL_MIN)), u8::from(e.degenerate));
            if !e.degenerate {
                self.curves.push(curve3d(&curve));
                let _ = writeln!(head, "1 {} 0 {} {}", self.curves.len(), num(a), num(z));
            }
            // P-curves by face; on a seam the first is the edge's forward use in the face as
            // written (a face written reversed turns its coedges round).
            let mut by_face: Vec<(usize, Vec<&Coedge>)> = Vec::new();
            for &(fi, c) in &uses[ei] {
                match by_face.iter_mut().find(|(f, _)| *f == fi) {
                    Some((_, cs)) => cs.push(c),
                    None => by_face.push((fi, vec![c])),
                }
            }
            for (fi, mut cs) in by_face {
                let (sf, map) = surf_of[fi];
                let sense = b.faces[fi].sense;
                cs.sort_by_key(|c| c.fwd != sense);
                let mut pc = |c: &Coedge| {
                    let pc = if h != 0.0 { shifted(&c.pc, h) } else { c.pc.clone() };
                    self.curves2d.push(curve2d(&pc, a, z, &map));
                    self.curves2d.len()
                };
                if cs.len() >= 2 {
                    let (p1, p2) = (pc(cs[0]), pc(cs[1]));
                    let _ = writeln!(head, "3 {p1} {p2} C0 {sf} 0 {} {}", num(a), num(z));
                } else {
                    let p1 = pc(cs[0]);
                    let _ = writeln!(head, "2 {p1} {sf} 0 {} {}", num(a), num(z));
                }
            }
            head.push_str("0\n\n0101000");
            eid.push(self.shape(head, vec![(true, vid[e.v[0]]), (false, vid[e.v[1]])]));
        }
        let mut fid = Vec::with_capacity(b.faces.len());
        for (fi, f) in b.faces.iter().enumerate() {
            let mut wires = Vec::new();
            for l in &f.loops {
                // As OCCT's face holds it: about the surface's natural normal.
                let subs: Vec<(bool, usize)> = if f.sense { l.iter().map(|c| (c.fwd, eid[c.edge])).collect() } else { l.iter().rev().map(|c| (!c.fwd, eid[c.edge])).collect() };
                wires.push((true, self.shape("Wi\n\n0101100".into(), subs)));
            }
            fid.push(self.shape(format!("Fa\n0 {} {} 0\n\n0101000", num(TOL_MIN), surf_of[fi].0), wires));
        }
        let in_solid: std::collections::HashSet<usize> = b.solids.iter().flat_map(|s| s.shells.iter().copied()).collect();
        let sid: Vec<usize> = b
            .shells
            .iter()
            .map(|s| {
                let subs = s.faces.iter().map(|&fi| (b.faces[fi].sense, fid[fi])).collect();
                self.shape(format!("Sh\n\n0101{}00", u8::from(s.closed)), subs)
            })
            .collect();
        let mut out = Vec::new();
        for s in &b.solids {
            out.push(self.shape("So\n\n0100000".into(), s.shells.iter().map(|&si| (true, sid[si])).collect()));
        }
        out.extend((0..b.shells.len()).filter(|s| !in_solid.contains(s)).map(|s| sid[s]));
        for w in &b.wires {
            out.push(self.shape("Wi\n\n0101000".into(), w.edges.iter().map(|&(e, fwd)| (fwd, eid[e])).collect()));
        }
        out.extend(b.points.iter().map(|&v| vid[v]));
        out
    }
}

/// Writes bodies as one compound in OCCT's BRep text format.
pub fn write(bodies: &[&Body]) -> String {
    let mut w = Writer::default();
    let mut top = Vec::new();
    for b in bodies {
        top.extend(w.body(b).into_iter().map(|s| (true, s)));
    }
    let root = w.shape("Co\n\n1100000".into(), top);
    let mut s = String::from("CASCADE Topology V1, (c) Matra-Datavision\nLocations 0\n");
    for (name, table) in [("Curve2ds", &w.curves2d), ("Curves", &w.curves), ("Polygon3D", &Vec::new()), ("PolygonOnTriangulations", &Vec::new()), ("Surfaces", &w.surfaces), ("Triangulations", &Vec::new())] {
        let _ = writeln!(s, "{name} {}", table.len());
        for r in table {
            s.push_str(r);
            s.push('\n');
        }
    }
    // References count back from the end of the table: the last shape is 1.
    let n = w.shapes.len();
    let _ = writeln!(s, "\nTShapes {n}");
    for sh in &w.shapes {
        s.push_str(&sh.head);
        s.push('\n');
        for (fwd, i) in &sh.subs {
            let _ = write!(s, "{}{} 0 ", if *fwd { '+' } else { '-' }, n - i);
        }
        s.push_str("*\n");
    }
    let _ = writeln!(s, "\n+{} 0", n - root);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_read_back_exactly() {
        for x in [0.0, -0.0, 1.0, -2.5, 1e-7, 0.1 + 0.2, 1e300, -1e-300, std::f64::consts::PI, 123456789.123] {
            assert_eq!(num(x).parse::<f64>().unwrap(), x, "{x}");
        }
    }

    #[test]
    fn hermite_pieces_follow_the_curve() {
        let e = Curve::Ellipse { c: P3::new(1.0, 2.0, 0.0), x: V3::x(), y: V3::y(), a: 1.0, b: 3.0 };
        let n = hermite(|t| { let (p, d, _) = e.d2(t); (p, d) }, 0.5, 4.0, 64);
        for i in 0..=100 {
            let t = 0.5 + 3.5 * i as f64 / 100.0;
            assert!((n.point(t) - e.point(t)).norm() < 1e-7, "{t}");
        }
    }

    #[test]
    fn shifted_curves_are_the_same_points() {
        let h = std::f64::consts::FRAC_PI_2;
        let cs = [
            Curve::Ellipse { c: P3::new(1.0, 2.0, 3.0), x: V3::x(), y: V3::z(), a: 1.0, b: 3.0 },
            Curve::Circle { c: P3::origin(), x: V3::x(), y: V3::y(), r: 2.0 },
            Curve::Line { o: P3::new(1.0, 0.0, 0.0), d: V3::new(0.6, 0.8, 0.0) },
        ];
        for c in cs {
            let s = shifted(&c, h);
            for t in [-1.0, 0.0, 0.3, 2.0] {
                assert!((s.point(t) - c.point(t + h)).norm() < 1e-12, "{c:?} {t}");
            }
        }
    }
}
