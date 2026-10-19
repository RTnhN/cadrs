//! Reading OCCT's BRep text format (`.brep` files as `BRepTools::Write` and DRAW's `save` write
//! them; versions 1 to 3, with or without DRAW's `DBRep_DrawableShape` header).
//!
//! **The file.** After the version line come tables, each a keyword and a count: `Locations`
//! (elementary 3×4 matrices, `1 …`, and products of earlier ones raised to powers, `2 l p … 0`),
//! `Curve2ds`, `Curves`, `Polygon3D`, `PolygonOnTriangulations`, `Surfaces`, `Triangulations`,
//! then `TShapes`: each a type (`Ve Ed Wi Fa Sh So CS Co`), its geometry, a flags word and its
//! sub-shapes (`+n l`, `-n l`, `in l`, `en l`: an orientation, a shape counted back from the end
//! of the table, a location; `*` ends the list). The last reference is the shape itself.
//!
//! **Geometry.** Curves: line, circle, ellipse, parabola (an exact quadratic on the edge's range),
//! hyperbola (interpolated), Bézier, B-spline (periodic ones unrolled), trimmed (its basis) and
//! offset (exact for lines and circles in their plane, else interpolated). Surfaces: plane,
//! cylinder, cone, sphere, torus, linear extrusion, revolution, Bézier, B-spline, rectangular
//! trimmed (its basis) and offset (exact for the elementary surfaces, else interpolated).
//! A spline edge curve is cut to the edge's range.
//!
//! **Topology.** A sub-shape's place is its parents' locations composed with its own, and its
//! orientation theirs composed with its own (as `TopAbs::Compose`). Two uses of one TShape with
//! the same composed location are one vertex or edge. The body is rebuilt by
//! [`super::xbuild::XBuilder`] from the 3D geometry (the file's p-curves aren't needed): our own
//! p-curves, pole edges and seams; the file's degenerate edges are left out (they are put back),
//! as are internal and external edges. Solids keep their shells; shells and faces outside a
//! solid become sheets.

use std::collections::HashMap;
use std::sync::Arc;

use super::curve::Curve;
use super::math::*;
use super::nurbs::{hom, NurbsCurve, NurbsSurface, H};
use super::surface::Surface;
use super::topo::{co_range, Body, Coedge, Face, Shell, Solid, TOL_MIN};
use super::xbuild::{XBuilder, XFace};

type R<T> = Result<T, String>;

// ---------------------------------------------------------------------------------------------
// Tokens

struct Tok<'a> {
    it: std::iter::Peekable<std::str::SplitAsciiWhitespace<'a>>,
}

impl<'a> Tok<'a> {
    fn next(&mut self) -> R<&'a str> {
        self.it.next().ok_or_else(|| "unexpected end of file".to_string())
    }
    fn peek(&mut self) -> Option<&'a str> {
        self.it.peek().copied()
    }
    fn f(&mut self) -> R<f64> {
        let t = self.next()?;
        t.parse::<f64>().map_err(|_| format!("not a number: {t}"))
    }
    fn i(&mut self) -> R<i64> {
        let t = self.next()?;
        t.parse::<i64>().map_err(|_| format!("not an integer: {t}"))
    }
    fn u(&mut self) -> R<usize> {
        let v = self.i()?;
        usize::try_from(v).map_err(|_| format!("not a count: {v}"))
    }
    fn p3(&mut self) -> R<P3> {
        Ok(P3::new(self.f()?, self.f()?, self.f()?))
    }
    fn v3(&mut self) -> R<V3> {
        Ok(V3::new(self.f()?, self.f()?, self.f()?))
    }
    fn expect(&mut self, kw: &str) -> R<()> {
        let t = self.next()?;
        if t == kw { Ok(()) } else { Err(format!("expected {kw}, found {t}")) }
    }
    /// A table's count when the next keyword is `kw` (tables a writer may leave out).
    fn table(&mut self, kw: &str) -> R<usize> {
        if self.peek() == Some(kw) {
            self.next()?;
            self.u()
        } else {
            Ok(0)
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Geometry as the file has it

/// A coordinate system: origin, main direction, x and y (y may make it left-handed).
#[derive(Debug, Clone, Copy)]
struct Ax {
    o: P3,
    z: V3,
    x: V3,
    y: V3,
}

impl Ax {
    fn read(t: &mut Tok) -> R<Self> {
        Ok(Self { o: t.p3()?, z: t.v3()?, x: t.v3()?, y: t.v3()? })
    }
    fn frame(&self) -> Frame {
        Frame::from_xz(self.o, self.x, self.z)
    }
    /// The file's own axes (left-handed ones too), so (u, v) are OCCT's.
    fn frame_exact(&self) -> Frame {
        Frame { o: self.o, x: self.x.normalize(), y: self.y.normalize(), z: self.z.normalize() }
    }
    /// +1 for a right-handed system, −1 for a left-handed one (whose surfaces' natural normals
    /// point the other way from ours).
    fn hand(&self) -> f64 {
        if self.x.cross(&self.y).dot(&self.z) >= 0.0 { 1.0 } else { -1.0 }
    }
}

/// A B-spline as OCCT holds it: poles, weights, distinct knots with multiplicities.
#[derive(Debug, Clone)]
pub(crate) struct Spline {
    pub degree: usize,
    pub poles: Vec<H>,
    pub knots: Vec<f64>,
    pub mults: Vec<usize>,
    pub periodic: bool,
}

#[derive(Debug, Clone)]
enum GCurve {
    Line(P3, V3),
    Circle(Ax, f64),
    Ellipse(Ax, f64, f64),
    Parabola(Ax, f64),
    Hyperbola(Ax, f64, f64),
    Spline(Spline),
    Trimmed(Box<GCurve>),
    Offset(f64, V3, Box<GCurve>),
}

#[derive(Debug, Clone)]
enum GSurf {
    Plane(Ax),
    Cylinder(Ax, f64),
    Cone(Ax, f64, f64),
    Sphere(Ax, f64),
    Torus(Ax, f64, f64),
    Extrusion(V3, GCurve),
    Revolution(P3, V3, GCurve),
    /// Poles along u (outer) and v (inner), and the two directions' knots.
    Spline { du: usize, dv: usize, poles: Vec<Vec<H>>, u: (Vec<f64>, Vec<usize>, bool), v: (Vec<f64>, Vec<usize>, bool) },
    Trimmed(Box<GSurf>),
    Offset(f64, Box<GSurf>),
}

fn read_poles(t: &mut Tok, n: usize, rational: bool) -> R<Vec<H>> {
    (0..n)
        .map(|_| {
            let p = t.p3()?;
            let w = if rational { t.f()? } else { 1.0 };
            Ok(hom(&p, w))
        })
        .collect()
}

fn read_knots(t: &mut Tok, n: usize) -> R<(Vec<f64>, Vec<usize>)> {
    let mut k = Vec::with_capacity(n);
    let mut m = Vec::with_capacity(n);
    for _ in 0..n {
        k.push(t.f()?);
        m.push(t.u()?);
    }
    Ok((k, m))
}

fn read_curve(t: &mut Tok) -> R<GCurve> {
    Ok(match t.i()? {
        1 => GCurve::Line(t.p3()?, t.v3()?),
        2 => GCurve::Circle(Ax::read(t)?, t.f()?),
        3 => GCurve::Ellipse(Ax::read(t)?, t.f()?, t.f()?),
        4 => GCurve::Parabola(Ax::read(t)?, t.f()?),
        5 => GCurve::Hyperbola(Ax::read(t)?, t.f()?, t.f()?),
        6 => {
            let rational = t.i()? != 0;
            let degree = t.u()?;
            let poles = read_poles(t, degree + 1, rational)?;
            GCurve::Spline(Spline { degree, poles, knots: vec![0.0, 1.0], mults: vec![degree + 1; 2], periodic: false })
        }
        7 => {
            let rational = t.i()? != 0;
            let periodic = t.i()? != 0;
            let (degree, np, nk) = (t.u()?, t.u()?, t.u()?);
            let poles = read_poles(t, np, rational)?;
            let (knots, mults) = read_knots(t, nk)?;
            GCurve::Spline(Spline { degree, poles, knots, mults, periodic })
        }
        8 => {
            let (_a, _b) = (t.f()?, t.f()?);
            GCurve::Trimmed(Box::new(read_curve(t)?))
        }
        9 => {
            let d = t.f()?;
            let v = t.v3()?;
            GCurve::Offset(d, v, Box::new(read_curve(t)?))
        }
        k => return Err(format!("unknown curve type {k}")),
    })
}

fn p2(t: &mut Tok) -> R<P3> {
    Ok(P3::new(t.f()?, t.f()?, 0.0))
}

fn d2(t: &mut Tok) -> R<V3> {
    Ok(V3::new(t.f()?, t.f()?, 0.0))
}

fn ax2(t: &mut Tok) -> R<Ax> {
    let (o, x, y) = (p2(t)?, d2(t)?, d2(t)?);
    Ok(Ax { o, z: x.cross(&y), x, y })
}

fn poles2(t: &mut Tok, n: usize, rational: bool) -> R<Vec<H>> {
    (0..n)
        .map(|_| {
            let p = p2(t)?;
            let w = if rational { t.f()? } else { 1.0 };
            Ok(hom(&p, w))
        })
        .collect()
}

/// A 2D curve (a p-curve), as a curve in the z = 0 plane.
fn read_curve2d(t: &mut Tok) -> R<GCurve> {
    Ok(match t.i()? {
        1 => GCurve::Line(p2(t)?, d2(t)?),
        2 => GCurve::Circle(ax2(t)?, t.f()?),
        3 => GCurve::Ellipse(ax2(t)?, t.f()?, t.f()?),
        4 => GCurve::Parabola(ax2(t)?, t.f()?),
        5 => GCurve::Hyperbola(ax2(t)?, t.f()?, t.f()?),
        6 => {
            let rational = t.i()? != 0;
            let degree = t.u()?;
            let poles = poles2(t, degree + 1, rational)?;
            GCurve::Spline(Spline { degree, poles, knots: vec![0.0, 1.0], mults: vec![degree + 1; 2], periodic: false })
        }
        7 => {
            let rational = t.i()? != 0;
            let periodic = t.i()? != 0;
            let (degree, np, nk) = (t.u()?, t.u()?, t.u()?);
            let poles = poles2(t, np, rational)?;
            let (knots, mults) = read_knots(t, nk)?;
            GCurve::Spline(Spline { degree, poles, knots, mults, periodic })
        }
        8 => {
            skip(t, 2)?;
            GCurve::Trimmed(Box::new(read_curve2d(t)?))
        }
        9 => {
            // Offset to the right of the direction of travel: T × z.
            let d = t.f()?;
            GCurve::Offset(d, V3::z(), Box::new(read_curve2d(t)?))
        }
        k => return Err(format!("unknown 2D curve type {k}")),
    })
}

/// Reads past a 2D curve.
#[allow(dead_code)]
fn skip_curve2d(t: &mut Tok) -> R<()> {
    match t.i()? {
        1 => skip(t, 4),
        2 | 4 => skip(t, 7),
        3 | 5 => skip(t, 8),
        6 => {
            let rational = t.i()? != 0;
            let degree = t.u()?;
            skip(t, (degree + 1) * (2 + rational as usize))
        }
        7 => {
            let rational = t.i()? != 0;
            let _periodic = t.i()?;
            let (_degree, np, nk) = (t.u()?, t.u()?, t.u()?);
            skip(t, np * (2 + rational as usize) + nk * 2)
        }
        8 => {
            skip(t, 2)?;
            skip_curve2d(t)
        }
        9 => {
            skip(t, 1)?;
            skip_curve2d(t)
        }
        k => Err(format!("unknown 2D curve type {k}")),
    }
}

fn skip(t: &mut Tok, n: usize) -> R<()> {
    for _ in 0..n {
        t.next()?;
    }
    Ok(())
}

fn read_surface(t: &mut Tok) -> R<GSurf> {
    Ok(match t.i()? {
        1 => GSurf::Plane(Ax::read(t)?),
        2 => GSurf::Cylinder(Ax::read(t)?, t.f()?),
        3 => GSurf::Cone(Ax::read(t)?, t.f()?, t.f()?),
        4 => GSurf::Sphere(Ax::read(t)?, t.f()?),
        5 => GSurf::Torus(Ax::read(t)?, t.f()?, t.f()?),
        6 => {
            let d = t.v3()?;
            GSurf::Extrusion(d, read_curve(t)?)
        }
        7 => {
            let (o, d) = (t.p3()?, t.v3()?);
            GSurf::Revolution(o, d, read_curve(t)?)
        }
        8 => {
            let (ur, vr) = (t.i()? != 0, t.i()? != 0);
            let (du, dv) = (t.u()?, t.u()?);
            let poles = (0..=du).map(|_| read_poles(t, dv + 1, ur || vr)).collect::<R<Vec<_>>>()?;
            GSurf::Spline { du, dv, poles, u: (vec![0.0, 1.0], vec![du + 1; 2], false), v: (vec![0.0, 1.0], vec![dv + 1; 2], false) }
        }
        9 => {
            let (ur, vr) = (t.i()? != 0, t.i()? != 0);
            let (up, vp) = (t.i()? != 0, t.i()? != 0);
            let (du, dv) = (t.u()?, t.u()?);
            let (nu, nv) = (t.u()?, t.u()?);
            let (nku, nkv) = (t.u()?, t.u()?);
            let poles = (0..nu).map(|_| read_poles(t, nv, ur || vr)).collect::<R<Vec<_>>>()?;
            let (ku, mu) = read_knots(t, nku)?;
            let (kv, mv) = read_knots(t, nkv)?;
            GSurf::Spline { du, dv, poles, u: (ku, mu, up), v: (kv, mv, vp) }
        }
        10 => {
            skip(t, 4)?;
            GSurf::Trimmed(Box::new(read_surface(t)?))
        }
        11 => {
            let d = t.f()?;
            GSurf::Offset(d, Box::new(read_surface(t)?))
        }
        k => return Err(format!("unknown surface type {k}")),
    })
}

// ---------------------------------------------------------------------------------------------
// Splines

/// The knot vector and poles of a spline whose domain covers [t0, t1] (a periodic one unrolled
/// over enough periods), cut to [t0, t1] when that lies inside.
pub(crate) fn spline_curve(s: &Spline, t0: f64, t1: f64) -> R<NurbsCurve> {
    let p = s.degree;
    if p == 0 || s.knots.len() < 2 || s.knots.len() != s.mults.len() {
        return Err("a bad B-spline".into());
    }
    let (a, b) = (s.knots[0], *s.knots.last().unwrap());
    if !s.periodic {
        let knots: Vec<f64> = s.knots.iter().zip(&s.mults).flat_map(|(k, m)| std::iter::repeat_n(*k, *m)).collect();
        if knots.len() != s.poles.len() + p + 1 {
            return Err(format!("{} knots for {} poles of degree {p}", knots.len(), s.poles.len()));
        }
        let c = NurbsCurve { degree: p, knots, cps: s.poles.clone() };
        let (lo, hi) = (t0.max(a), t1.min(b));
        return Ok(if hi > lo && (lo > a || hi < b) { clamp(c, lo, hi) } else { c });
    }
    // One period of knots (the last knot is the first one a period on).
    let per: Vec<f64> = s.knots[..s.knots.len() - 1].iter().zip(&s.mults).flat_map(|(k, m)| std::iter::repeat_n(*k, *m)).collect();
    let n = per.len();
    if n != s.poles.len() || n <= p {
        return Err(format!("a periodic B-spline with {} poles for {n} knots", s.poles.len()));
    }
    let period = b - a;
    let knot = |j: i64| -> f64 { per[j.rem_euclid(n as i64) as usize] + period * j.div_euclid(n as i64) as f64 };
    // OCCT's flat knots start p + 1 − m₁ knots before the first one, with the first basis
    // function on the first pole.
    let m1 = (p + 1).saturating_sub(s.mults[0]) as i64;
    let (lo, hi) = if t1 > t0 { (t0, t1) } else { (a, b) };
    let plo = ((lo - a) / period).floor() as i64 - 1;
    let phi = ((hi - a) / period).ceil() as i64 + 1;
    let (i0, i1) = (plo * n as i64, phi * n as i64);
    let knots: Vec<f64> = (i0..=i1 + p as i64 + 1).map(|i| knot(i - m1)).collect();
    let cps: Vec<H> = (i0..=i1).map(|i| s.poles[i.rem_euclid(n as i64) as usize]).collect();
    Ok(clamp(NurbsCurve { degree: p, knots, cps }, lo, hi))
}

/// The curve on [a, b] (inside its domain) with clamped ends.
fn clamp(c: NurbsCurve, a: f64, b: f64) -> NurbsCurve {
    let eps = 1e-13 * (1.0 + a.abs().max(b.abs()));
    if !(b > a + eps) {
        // An empty range (a degenerate edge's): nothing to cut.
        return c;
    }
    clamp_on(c.clone(), a, b, eps).unwrap_or(c)
}

fn clamp_on(mut c: NurbsCurve, a: f64, b: f64, eps: f64) -> Option<NurbsCurve> {
    let p = c.degree;
    for u in [a, b] {
        let m = c.knots.iter().filter(|k| (**k - u).abs() <= eps).count();
        for _ in m..p {
            c = c.insert_knot(u);
        }
    }
    let ia = c.knots.iter().position(|k| (*k - a).abs() <= eps)?;
    let ma = c.knots[ia..].iter().take_while(|k| (**k - a).abs() <= eps).count();
    let ib = c.knots.iter().position(|k| (*k - b).abs() <= eps)?;
    if ib < ia + ma || ia + ma < p + 1 {
        return None;
    }
    let mut knots = vec![a; p + 1];
    knots.extend_from_slice(&c.knots[ia + ma..ib]);
    knots.extend(vec![b; p + 1]);
    let first = ia + ma - 1 - p;
    let ncp = knots.len() - p - 1;
    Some(NurbsCurve { degree: p, knots, cps: c.cps.get(first..first + ncp)?.to_vec() })
}

fn spline_surface(du: usize, dv: usize, poles: &[Vec<H>], u: &(Vec<f64>, Vec<usize>, bool), v: &(Vec<f64>, Vec<usize>, bool)) -> R<NurbsSurface> {
    if poles.is_empty() || poles.iter().any(|r| r.len() != poles[0].len()) {
        return Err("a bad B-spline surface".into());
    }
    let full = |k: &(Vec<f64>, Vec<usize>, bool)| (k.0[0], *k.0.last().unwrap());
    // Along u, column by column.
    let nv = poles[0].len();
    let mut cols: Vec<NurbsCurve> = Vec::with_capacity(nv);
    for j in 0..nv {
        let s = Spline { degree: du, poles: poles.iter().map(|r| r[j]).collect(), knots: u.0.clone(), mults: u.1.clone(), periodic: u.2 };
        let (a, b) = full(u);
        cols.push(spline_curve(&s, a, b)?);
    }
    let nu = cols[0].cps.len();
    let ku = cols[0].knots.clone();
    // Then along v, row by row.
    let mut rows: Vec<NurbsCurve> = Vec::with_capacity(nu);
    for i in 0..nu {
        let s = Spline { degree: dv, poles: cols.iter().map(|c| c.cps[i]).collect(), knots: v.0.clone(), mults: v.1.clone(), periodic: v.2 };
        let (a, b) = full(v);
        rows.push(spline_curve(&s, a, b)?);
    }
    let kv = rows[0].knots.clone();
    Ok(NurbsSurface { du, dv, ku, kv, cps: rows.into_iter().map(|r| r.cps).collect() })
}

/// A curve through points of `f` on [t0, t1] (for the curves we have no exact form of).
fn sampled(f: impl Fn(f64) -> P3, t0: f64, t1: f64) -> Curve {
    let n = 64;
    let ts: Vec<f64> = (0..=n).map(|i| t0 + (t1 - t0) * i as f64 / n as f64).collect();
    let pts: Vec<P3> = ts.iter().map(|&t| f(t)).collect();
    Curve::Nurbs(Arc::new(NurbsCurve::interpolate(&pts, &ts, 3, None, None)))
}

// ---------------------------------------------------------------------------------------------
// Geometry to ours

/// The curve on [t0, t1] (the edge's range: unbounded curves and spline pieces need one).
fn to_curve(g: &GCurve, t0: f64, t1: f64) -> R<Curve> {
    Ok(match g {
        GCurve::Line(o, d) => Curve::Line { o: *o, d: d.normalize() },
        GCurve::Circle(ax, r) => Curve::Circle { c: ax.o, x: ax.x.normalize(), y: ax.y.normalize(), r: *r },
        GCurve::Ellipse(ax, a, b) => Curve::Ellipse { c: ax.o, x: ax.x.normalize(), y: ax.y.normalize(), a: *a, b: *b },
        GCurve::Parabola(ax, f) => {
            // o + t²/(4f)·x + t·y: a quadratic, exactly a Bézier curve on [t0, t1].
            let pt = |t: f64| ax.o + ax.x * (t * t / (4.0 * f)) + ax.y * t;
            let d0 = ax.x * (t0 / (2.0 * f)) + ax.y;
            let mid = pt(t0) + d0 * ((t1 - t0) / 2.0);
            let mut c = NurbsCurve::bezier(&[pt(t0), mid, pt(t1)], &[1.0; 3]);
            c = c.reparam(t0, t1);
            Curve::Nurbs(Arc::new(c))
        }
        GCurve::Hyperbola(ax, a, b) => {
            let (ax, a, b) = (*ax, *a, *b);
            sampled(move |t| ax.o + ax.x * (a * t.cosh()) + ax.y * (b * t.sinh()), t0, t1)
        }
        GCurve::Spline(s) => Curve::Nurbs(Arc::new(spline_curve(s, t0, t1)?)),
        GCurve::Trimmed(c) => to_curve(c, t0, t1)?,
        GCurve::Offset(d, v, basis) => {
            let c = to_curve(basis, t0, t1)?;
            let v = v.normalize();
            match &c {
                // The offset direction is T × V.
                Curve::Line { o, d: dir } if dir.cross(&v).norm() > 1e-12 => Curve::Line { o: o + dir.cross(&v).normalize() * *d, d: *dir },
                Curve::Circle { c: o, x, y, r } if x.cross(y).normalize().cross(&v).norm() < 1e-12 => {
                    let r2 = r + d * x.cross(y).normalize().dot(&v);
                    if r2 <= 0.0 {
                        return Err("an offset circle of no radius".into());
                    }
                    Curve::Circle { c: *o, x: *x, y: *y, r: r2 }
                }
                _ => {
                    let d = *d;
                    sampled(move |t| c.point(t) + c.tangent(t).cross(&v).normalize() * d, t0, t1)
                }
            }
        }
    })
}

/// The basis curve's own range (a spline's knots; lines and conics have none).
fn curve_range(g: &GCurve) -> Option<(f64, f64)> {
    match g {
        GCurve::Spline(s) => Some((s.knots[0], *s.knots.last()?)),
        GCurve::Trimmed(c) | GCurve::Offset(_, _, c) => curve_range(c),
        GCurve::Circle(..) | GCurve::Ellipse(..) => Some((0.0, TAU)),
        _ => None,
    }
}

/// Our surface and whether its natural normal is OCCT's.
fn to_surface(g: &GSurf, exact: bool) -> R<(Surface, bool)> {
    let fr = |ax: &Ax| if exact { ax.frame_exact() } else { ax.frame() };
    let hand = |ax: &Ax| exact || ax.hand() > 0.0;
    Ok(match g {
        GSurf::Plane(ax) => (Surface::Plane { f: fr(ax) }, hand(ax)),
        GSurf::Cylinder(ax, r) => (Surface::Cylinder { f: fr(ax), r: *r }, hand(ax)),
        GSurf::Cone(ax, r, a) => (Surface::Cone { f: fr(ax), r: *r, a: *a }, hand(ax)),
        GSurf::Sphere(ax, r) => (Surface::Sphere { f: fr(ax), r: *r }, hand(ax)),
        GSurf::Torus(ax, rmaj, rmin) => (Surface::Torus { f: fr(ax), rmaj: *rmaj, rmin: *rmin }, hand(ax)),
        GSurf::Extrusion(d, c) => {
            let (a, b) = curve_range(c).ok_or("an extrusion of an unbounded curve")?;
            let c = to_curve(c, a, b)?;
            // OCCT's normal is C' × d, ours Su × Sv: the same.
            (Surface::Extrusion { c, d: d.normalize() }, true)
        }
        GSurf::Revolution(o, z, c) => {
            let (a, b) = curve_range(c).unwrap_or((-1.0, 1.0));
            let m = to_curve(c, a, b)?;
            let z = z.normalize();
            let mid = m.point((a + b) / 2.0);
            let w = mid - o;
            let xh = w - z * w.dot(&z);
            // OCCT's revolution runs u about the axis and v along the meridian, as ours.
            (Surface::Revolution { f: Frame::from_xz(*o, xh, z), m }, true)
        }
        GSurf::Spline { du, dv, poles, u, v } => (Surface::Nurbs(Arc::new(spline_surface(*du, *dv, poles, u, v)?)), true),
        GSurf::Trimmed(s) => to_surface(s, exact)?,
        GSurf::Offset(d, s) => offset_surface(*d, s, exact)?,
    })
}

fn offset_surface(d: f64, g: &GSurf, exact: bool) -> R<(Surface, bool)> {
    let (base, same) = to_surface(g, exact)?;
    // Along OCCT's normal: ours, or ours turned round.
    let d = if same { d } else { -d };
    let pos = |r: f64| if r > 1e-12 { Ok(r) } else { Err("an offset surface of no radius".to_string()) };
    Ok((
        match &base {
            Surface::Plane { f } => Surface::Plane { f: Frame { o: f.o + f.z * d, ..*f } },
            Surface::Cylinder { f, r } => Surface::Cylinder { f: *f, r: pos(r + d)? },
            Surface::Sphere { f, r } => Surface::Sphere { f: *f, r: pos(r + d)? },
            Surface::Torus { f, rmaj, rmin } => Surface::Torus { f: *f, rmaj: *rmaj, rmin: pos(rmin + d)? },
            Surface::Cone { f, r, a } => Surface::Cone { f: *f, r: r + d / a.cos(), a: *a },
            s => {
                // Interpolated through offset points on a grid.
                let ((u0, u1), (v0, v1)) = s.param_box();
                let n = 32;
                let us: Vec<f64> = (0..=n).map(|i| u0 + (u1 - u0) * i as f64 / n as f64).collect();
                let vs: Vec<f64> = (0..=n).map(|i| v0 + (v1 - v0) * i as f64 / n as f64).collect();
                let rows: Vec<NurbsCurve> = vs
                    .iter()
                    .map(|&v| {
                        let pts: Vec<P3> = us.iter().map(|&u| s.eval(u, v) + s.normal(u, v) * d).collect();
                        NurbsCurve::interpolate(&pts, &us, 3, None, None)
                    })
                    .collect();
                Surface::Nurbs(Arc::new(NurbsSurface::through_curves(&rows, &vs, 3, None, None)))
            }
        },
        same,
    ))
}

// ---------------------------------------------------------------------------------------------
// Locations

/// A location: a product of elementary ones raised to powers (as TopLoc compares them) and its
/// map.
#[derive(Debug, Clone)]
struct Loc {
    chain: Vec<(usize, i64)>,
    m: Affine,
}

fn compose(a: &Affine, b: &Affine) -> Affine {
    Affine { m: a.m * b.m, t: a.m * b.t + a.t }
}

fn inverse(a: &Affine) -> Affine {
    let mi = a.m.try_inverse().unwrap_or_else(nalgebra::Matrix3::identity);
    Affine { m: mi, t: -(mi * a.t) }
}

fn normalize(chain: Vec<(usize, i64)>) -> Vec<(usize, i64)> {
    let mut out: Vec<(usize, i64)> = Vec::with_capacity(chain.len());
    for (e, p) in chain {
        match out.last_mut() {
            Some(l) if l.0 == e => {
                l.1 += p;
                if l.1 == 0 {
                    out.pop();
                }
            }
            _ if p != 0 => out.push((e, p)),
            _ => {}
        }
    }
    out
}

impl Loc {
    fn identity() -> Self {
        Self { chain: Vec::new(), m: Affine::identity() }
    }
    fn then(&self, o: &Loc) -> Loc {
        let mut chain = self.chain.clone();
        chain.extend_from_slice(&o.chain);
        Loc { chain: normalize(chain), m: compose(&self.m, &o.m) }
    }
    fn inverse(&self) -> Loc {
        Loc { chain: self.chain.iter().rev().map(|&(e, p)| (e, -p)).collect(), m: inverse(&self.m) }
    }
    fn powered(&self, p: i64) -> Loc {
        let base = if p < 0 { self.inverse() } else { self.clone() };
        let mut out = Loc::identity();
        for _ in 0..p.unsigned_abs() {
            out = out.then(&base);
        }
        out
    }
}

// ---------------------------------------------------------------------------------------------
// Shapes

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ori {
    Fwd,
    Rev,
    Int,
    Ext,
}

impl Ori {
    fn compose(self, child: Ori) -> Ori {
        match (self, child) {
            (Ori::Fwd, c) => c,
            (Ori::Rev, Ori::Fwd) => Ori::Rev,
            (Ori::Rev, Ori::Rev) => Ori::Fwd,
            (Ori::Rev, c) => c,
            (o, _) => o,
        }
    }
}

#[derive(Debug, Clone)]
enum Kind {
    Vertex(P3),
    /// The 3D curve (index, location, range), when there is one, whether it is degenerate, and
    /// its p-curves.
    Edge(Option<(usize, usize, f64, f64)>, bool, Vec<PcRep>),
    Wire,
    /// The surface (index, location).
    Face(Option<(usize, usize)>),
    Shell,
    Solid,
    Compound,
}

/// An edge's p-curve on a surface (two on a closed surface: the edge's forward use first).
#[derive(Debug, Clone)]
struct PcRep {
    pc: usize,
    pc2: Option<usize>,
    surf: usize,
    loc: usize,
    range: (f64, f64),
}

#[derive(Debug, Clone)]
struct TShape {
    kind: Kind,
    /// A vertex's or edge's tolerance from the file (0 for other shapes).
    tol: f64,
    closed: bool,
    subs: Vec<(Ori, usize, usize)>,
}

/// A parsed file.
struct File {
    locs: Vec<Loc>,
    curves2d: Vec<GCurve>,
    curves: Vec<GCurve>,
    surfaces: Vec<GSurf>,
    shapes: Vec<TShape>,
    root: (Ori, usize, usize),
}

fn shape_ref(t: &mut Tok, n: usize) -> R<Option<(Ori, usize, usize)>> {
    let w = t.next()?;
    if w == "*" {
        return Ok(None);
    }
    let ori = match w.as_bytes()[0] {
        b'+' => Ori::Fwd,
        b'-' => Ori::Rev,
        b'i' => Ori::Int,
        b'e' => Ori::Ext,
        _ => return Err(format!("bad shape reference {w}")),
    };
    let k: usize = w[1..].parse().map_err(|_| format!("bad shape reference {w}"))?;
    if k == 0 || k > n {
        return Err(format!("shape reference {w} out of range"));
    }
    let l = t.u()?;
    Ok(Some((ori, n - k, l)))
}

fn parse(text: &str) -> R<File> {
    // The version line (after DRAW's header, if any).
    let start = text.find("CASCADE Topology V").ok_or("not an OCCT BRep file")?;
    let rest = &text[start..];
    let version: u32 = rest["CASCADE Topology V".len()..].chars().next().and_then(|c| c.to_digit(10)).ok_or("no format version")?;
    let body = rest.find('\n').map_or("", |i| &rest[i..]);
    let mut t = Tok { it: body.split_ascii_whitespace().peekable() };

    // Locations (index 0: none).
    t.expect("Locations")?;
    let nl = t.u()?;
    let mut locs = vec![Loc::identity()];
    for _ in 0..nl {
        match t.i()? {
            1 => {
                let mut v = [0.0; 12];
                for x in v.iter_mut() {
                    *x = t.f()?;
                }
                let m = nalgebra::Matrix3::new(v[0], v[1], v[2], v[4], v[5], v[6], v[8], v[9], v[10]);
                let id = locs.len();
                locs.push(Loc { chain: vec![(id, 1)], m: Affine { m, t: V3::new(v[3], v[7], v[11]) } });
            }
            2 => {
                let mut l = Loc::identity();
                loop {
                    let l1 = t.u()?;
                    if l1 == 0 {
                        break;
                    }
                    let p = t.i()?;
                    let base = locs.get(l1).ok_or("location out of range")?.clone();
                    l = base.powered(p).then(&l);
                }
                locs.push(l);
            }
            k => return Err(format!("unknown location type {k}")),
        }
    }

    let n2 = t.table("Curve2ds")?;
    let curves2d = (0..n2).map(|_| read_curve2d(&mut t)).collect::<R<Vec<_>>>()?;
    let nc = t.table("Curves")?;
    let curves = (0..nc).map(|_| read_curve(&mut t)).collect::<R<Vec<_>>>()?;
    let np = t.table("Polygon3D")?;
    for _ in 0..np {
        let nn = t.u()?;
        let has = t.i()? != 0;
        skip(&mut t, 1 + 3 * nn + if has { nn } else { 0 })?;
    }
    let npt = t.table("PolygonOnTriangulations")?;
    for _ in 0..npt {
        let nn = t.u()?;
        skip(&mut t, nn)?;
        // "p", the deflection, whether there are parameters.
        t.next()?;
        t.next()?;
        let has = t.i()? != 0;
        if has {
            skip(&mut t, nn)?;
        }
    }
    let ns = t.table("Surfaces")?;
    let surfaces = (0..ns).map(|_| read_surface(&mut t)).collect::<R<Vec<_>>>()?;
    let ntr = t.table("Triangulations")?;
    for _ in 0..ntr {
        let (nn, ntri) = (t.u()?, t.u()?);
        let uv = t.i()? != 0;
        let normals = if version >= 3 { t.i()? != 0 } else { false };
        skip(&mut t, 1 + 3 * nn + if uv { 2 * nn } else { 0 } + 3 * ntri + if normals { 3 * nn } else { 0 })?;
    }

    t.expect("TShapes")?;
    let n = t.u()?;
    let mut shapes: Vec<TShape> = Vec::with_capacity(n);
    for si in 0..n {
        let ty = t.next()?;
        let mut tol = 0.0;
        let kind = (|| -> R<Kind> { Ok(match ty {
            "Ve" => {
                tol = t.f()?;
                let p = t.p3()?;
                // Point representations, to a "0 0".
                loop {
                    let _p1 = t.f()?;
                    match t.i()? {
                        0 => break,
                        1 => skip(&mut t, 1)?,
                        2 => skip(&mut t, 2)?,
                        3 => skip(&mut t, 2)?,
                        k => return Err(format!("unknown point representation {k}")),
                    }
                    let _l = t.u()?;
                }
                Kind::Vertex(p)
            }
            "Ed" => {
                tol = t.f()?;
                let (_sp, _sr) = (t.i()?, t.i()?);
                let degenerate = t.i()? == 1;
                let mut c3 = None;
                let mut pcs: Vec<PcRep> = Vec::new();
                loop {
                    match t.i()? {
                        0 => break,
                        1 => {
                            let (c, l) = (t.u()?, t.u()?);
                            let (a, b) = (t.f()?, t.f()?);
                            if c >= 1 && c <= curves.len() {
                                c3 = Some((c - 1, l, a, b));
                            }
                        }
                        k @ (2 | 3) => {
                            let pc = t.u()?;
                            let pc2 = if k == 3 {
                                // pc2 and the regularity, written "24C0" or "24 C0".
                                let w = t.next()?;
                                let digits: String = w.chars().take_while(|c| c.is_ascii_digit()).collect();
                                if digits.len() == w.len() {
                                    skip(&mut t, 1)?;
                                }
                                Some(digits.parse::<usize>().map_err(|_| format!("bad p-curve index {w}"))?)
                            } else {
                                None
                            };
                            let (sf, l) = (t.u()?, t.u()?);
                            let (a, b) = (t.f()?, t.f()?);
                            if version == 2 {
                                skip(&mut t, 4)?;
                            }
                            let ok = |i: usize| i >= 1 && i <= curves2d.len();
                            if ok(pc) && pc2.is_none_or(ok) && sf >= 1 {
                                pcs.push(PcRep { pc: pc - 1, pc2: pc2.map(|x| x - 1), surf: sf - 1, loc: l, range: (a, b) });
                            }
                        }
                        4 => skip(&mut t, 5)?,
                        5 => skip(&mut t, 2)?,
                        6 => skip(&mut t, 3)?,
                        7 => skip(&mut t, 4)?,
                        k => return Err(format!("unknown edge representation {k}")),
                    }
                }
                Kind::Edge(c3, degenerate, pcs)
            }
            "Wi" => Kind::Wire,
            "Fa" => {
                let val = t.i()?;
                let mut surf = None;
                if val == 0 || val == 1 {
                    let _tol = t.f()?;
                    let (s, l) = (t.u()?, t.u()?);
                    if s >= 1 && s <= surfaces.len() {
                        surf = Some((s - 1, l));
                    }
                    if t.peek() == Some("2") {
                        skip(&mut t, 2)?;
                    }
                } else {
                    skip(&mut t, 1)?;
                }
                Kind::Face(surf)
            }
            "Sh" => Kind::Shell,
            "So" => Kind::Solid,
            "CS" | "Co" => Kind::Compound,
            k => return Err(format!("unknown shape type {k}")),
        }) })()
        .map_err(|e| format!("shape {} ({ty}): {e}", n - si))?;
        let flags = t.next()?.as_bytes().to_vec();
        let closed = flags.get(4) == Some(&b'1');
        let mut subs = Vec::new();
        // References count back from the end of the whole table; only earlier shapes exist.
        while let Some(r) = shape_ref(&mut t, n)? {
            if r.1 >= shapes.len() || r.2 >= locs.len() {
                return Err(format!("shape {} ({ty}): a sub-shape reference out of range", n - si));
            }
            subs.push(r);
        }
        shapes.push(TShape { kind, tol, closed, subs });
    }
    let root = match shape_ref(&mut t, n) {
        Ok(Some(r)) if r.1 < shapes.len() && r.2 < locs.len() => r,
        _ => (Ori::Fwd, shapes.len().checked_sub(1).ok_or("no shapes")?, 0),
    };
    Ok(File { locs, curves2d, curves, surfaces, shapes, root })
}

// ---------------------------------------------------------------------------------------------
// Building

/// A location's map rounded (to 1e-9), so two uses of a shape placed alike are one shape
/// whichever products of locations placed them.
fn loc_key(l: &Loc) -> [i64; 12] {
    let m = &l.m;
    let v = [m.m[(0, 0)], m.m[(0, 1)], m.m[(0, 2)], m.m[(1, 0)], m.m[(1, 1)], m.m[(1, 2)], m.m[(2, 0)], m.m[(2, 1)], m.m[(2, 2)], m.t.x, m.t.y, m.t.z];
    v.map(|x| (x * 1e9).round() as i64)
}

struct Builder<'a> {
    f: &'a File,
    xb: XBuilder,
    verts: HashMap<(usize, [i64; 12]), usize>,
    edges: HashMap<(usize, [i64; 12]), Option<usize>>,
    free_faces: Vec<usize>,
    /// Edges outside any face (wires and edges of the shape), in order.
    free_edges: Vec<usize>,
    /// Whether any free edge is in a wire.
    wires: bool,
    warnings: Vec<String>,
}

impl Builder<'_> {
    fn vertex(&mut self, s: usize, loc: &Loc) -> Option<usize> {
        let key = (s, loc_key(loc));
        if let Some(&v) = self.verts.get(&key) {
            return Some(v);
        }
        let Kind::Vertex(p) = self.f.shapes[s].kind else { return None };
        let v = self.xb.vertex(loc.m.point(&p));
        self.xb.body.verts[v].tol = self.f.shapes[s].tol.max(TOL_MIN);
        self.verts.insert(key, v);
        Some(v)
    }

    fn edge(&mut self, s: usize, loc: &Loc) -> Option<usize> {
        let key = (s, loc_key(loc));
        if let Some(e) = self.edges.get(&key) {
            return *e;
        }
        let e = self.make_edge(s, loc);
        self.edges.insert(key, e);
        e
    }

    fn make_edge(&mut self, s: usize, loc: &Loc) -> Option<usize> {
        let sh = &self.f.shapes[s];
        let Kind::Edge(c3, degenerate, _) = sh.kind else { return None };
        if degenerate {
            return None;
        }
        let (mut v0, mut v1) = (None, None);
        for &(o, vs, l) in &sh.subs.clone() {
            let vl = loc.then(&self.f.locs[l]);
            match o {
                Ori::Fwd => v0 = self.vertex(vs, &vl),
                Ori::Rev => v1 = self.vertex(vs, &vl),
                _ => {}
            }
        }
        let (Some((ci, cl, a, b)), Some(v0), Some(v1)) = (c3, v0, v1) else {
            self.warnings.push(format!("edge {s}: no curve or vertices"));
            return None;
        };
        let at = compose(&loc.m, &self.f.locs[cl].m);
        let curve = match to_curve(&self.f.curves[ci], a, b) {
            Ok(c) => c.transformed(&at),
            Err(e) => {
                self.warnings.push(format!("edge {s}: {e}"));
                return None;
            }
        };
        let e = self.xb.edge(curve, v0, v1);
        if let Some(e) = e {
            self.xb.body.edges[e].tol = self.f.shapes[s].tol.max(TOL_MIN);
        }
        if e.is_none() {
            self.warnings.push(format!("edge {s}: empty"));
        }
        e
    }

    fn face(&mut self, s: usize, ori: Ori, loc: &Loc) -> Option<usize> {
        let sh = self.f.shapes[s].clone();
        let Kind::Face(Some((si, sl))) = sh.kind else {
            self.warnings.push(format!("face {s}: no surface"));
            return None;
        };
        let (surf, same) = match to_surface(&self.f.surfaces[si], false) {
            Ok(x) => x,
            Err(e) => {
                self.warnings.push(format!("face {s}: {e}"));
                return None;
            }
        };
        let at = compose(&loc.m, &self.f.locs[sl].m);
        let surf = surf.transformed(&at);
        let mut loops = Vec::new();
        for &(wo, ws, wl) in &sh.subs {
            let wloc = loc.then(&self.f.locs[wl]);
            let wori = ori.compose(wo);
            let mut lp = Vec::new();
            for &(eo, es, el) in &self.f.shapes[ws].subs.clone() {
                let o = wori.compose(eo);
                if !matches!(o, Ori::Fwd | Ori::Rev) {
                    continue;
                }
                if let Some(e) = self.edge(es, &wloc.then(&self.f.locs[el])) {
                    lp.push((e, o == Ori::Fwd));
                }
            }
            if !lp.is_empty() {
                loops.push(lp);
            }
        }
        // The outward normal is OCCT's natural normal for a forward face.
        let sense = (ori == Ori::Fwd) == same;
        Some(self.xb.face(XFace { surf, sense, loops }))
    }

    fn shell(&mut self, s: usize, ori: Ori, loc: &Loc) -> Vec<usize> {
        let mut faces = Vec::new();
        for &(o, fs, l) in &self.f.shapes[s].subs.clone() {
            let o = ori.compose(o);
            if let Some(fi) = self.face(fs, o, &loc.then(&self.f.locs[l])) {
                faces.push(fi);
            }
        }
        faces
    }

    fn visit(&mut self, s: usize, ori: Ori, loc: &Loc) {
        let sh = self.f.shapes[s].clone();
        match sh.kind {
            Kind::Compound => {
                for &(o, c, l) in &sh.subs {
                    self.visit(c, ori.compose(o), &loc.then(&self.f.locs[l]));
                }
            }
            Kind::Solid => {
                let mut shells = Vec::new();
                for &(o, c, l) in &sh.subs {
                    if !matches!(self.f.shapes[c].kind, Kind::Shell) {
                        continue;
                    }
                    let faces = self.shell(c, ori.compose(o), &loc.then(&self.f.locs[l]));
                    if !faces.is_empty() {
                        shells.push(self.xb.shell(faces, true));
                    }
                }
                if !shells.is_empty() {
                    self.xb.solid(shells);
                }
            }
            Kind::Shell => {
                let faces = self.shell(s, ori, loc);
                if !faces.is_empty() {
                    self.xb.shell(faces, sh.closed);
                }
            }
            Kind::Face(_) => {
                if let Some(fi) = self.face(s, ori, loc) {
                    self.free_faces.push(fi);
                }
            }
            Kind::Wire => {
                self.wires = true;
                for &(o, c, l) in &sh.subs {
                    self.visit(c, ori.compose(o), &loc.then(&self.f.locs[l]));
                }
            }
            Kind::Edge(..) => {
                if matches!(ori, Ori::Fwd | Ori::Rev) {
                    if let Some(e) = self.edge(s, loc) {
                        if !self.free_edges.contains(&e) {
                            self.free_edges.push(e);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}


// ---------------------------------------------------------------------------------------------
// Building straight from the file

/// Builds the body from the file as it is: its vertices, its edges' 3D curves and ranges, its
/// faces on surfaces with OCCT's (u, v) (the file's own axes) and their loops with the file's
/// p-curves (a seam's two). Shared shapes stay shared. Anything the file leaves out (an edge
/// without a p-curve on a face, a missing curve) makes it give up (the caller rebuilds through
/// [`XBuilder`] instead).
struct Direct<'a> {
    f: &'a File,
    body: Body,
    verts: HashMap<(usize, [i64; 12]), usize>,
    /// Vertices by position (1e-6 cells), to merge coincident vertex records.
    grid: HashMap<[i64; 3], Vec<usize>>,
    edges: HashMap<(usize, [i64; 12]), usize>,
    shells: Vec<(Vec<usize>, bool)>,
    solids: Vec<Vec<usize>>,
    free_faces: Vec<usize>,
}

impl Direct<'_> {
    fn vertex(&mut self, s: usize, loc: &Loc) -> R<usize> {
        let key = (s, loc_key(loc));
        if let Some(&v) = self.verts.get(&key) {
            return Ok(v);
        }
        let Kind::Vertex(p) = self.f.shapes[s].kind else { return Err("not a vertex".into()) };
        let p = loc.m.point(&p);
        // Two vertex records at one point (files from other systems): one vertex.
        let cell = |q: &P3| [(q.x * 1e6).round() as i64, (q.y * 1e6).round() as i64, (q.z * 1e6).round() as i64];
        let c = cell(&p);
        let mut found = None;
        'look: for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(vs) = self.grid.get(&[c[0] + dx, c[1] + dy, c[2] + dz]) {
                        if let Some(&v) = vs.iter().find(|&&v| (self.body.verts[v].p - p).norm() < 1e-7) {
                            found = Some(v);
                            break 'look;
                        }
                    }
                }
            }
        }
        let v = match found {
            Some(v) => v,
            None => {
                let v = self.body.add_vertex(p);
                self.grid.entry(c).or_default().push(v);
                v
            }
        };
        let tol = self.f.shapes[s].tol;
        if tol > self.body.verts[v].tol {
            self.body.verts[v].tol = tol;
        }
        self.verts.insert(key, v);
        Ok(v)
    }

    fn edge(&mut self, s: usize, loc: &Loc) -> R<usize> {
        let key = (s, loc_key(loc));
        if let Some(&e) = self.edges.get(&key) {
            return Ok(e);
        }
        let sh = &self.f.shapes[s];
        let Kind::Edge(c3, degenerate, ref pcs) = sh.kind else { return Err("not an edge".into()) };
        let (mut v0, mut v1) = (None, None);
        for &(o, vs, l) in &sh.subs.clone() {
            let vl = loc.then(&self.f.locs[l]);
            match o {
                Ori::Fwd => v0 = Some(self.vertex(vs, &vl)?),
                Ori::Rev => v1 = Some(self.vertex(vs, &vl)?),
                _ => {}
            }
        }
        let (v0, v1) = match (v0, v1) {
            (Some(a), Some(b)) => (a, b),
            (Some(a), None) | (None, Some(a)) => (a, a),
            _ => return Err(format!("edge {s} has no vertices")),
        };
        let e = if degenerate {
            let (a, b) = pcs.first().map_or((0.0, 1.0), |r| r.range);
            self.body.add_degenerate(self.body.verts[v0].p, a, b, v0)
        } else {
            let (ci, cl, a, b) = c3.ok_or_else(|| format!("edge {s} has no 3D curve"))?;
            let at = compose(&loc.m, &self.f.locs[cl].m);
            let c = to_curve(&self.f.curves[ci], a, b)?.transformed(&at);
            self.body.add_edge(c, a, b, v0, v1)
        };
        self.body.edges[e].tol = self.f.shapes[s].tol.max(TOL_MIN);
        self.edges.insert(key, e);
        Ok(e)
    }

    fn face(&mut self, s: usize, ori: Ori, loc: &Loc) -> R<usize> {
        let sh = self.f.shapes[s].clone();
        let Kind::Face(Some((si, sl))) = sh.kind else { return Err(format!("face {s} has no surface")) };
        let (surf, _) = to_surface(&self.f.surfaces[si], true)?;
        let sloc = loc.then(&self.f.locs[sl]);
        let surf = surf.transformed(&sloc.m);
        let skey = loc_key(&sloc);
        let mut loops: Vec<Vec<Coedge>> = Vec::new();
        for &(wo, ws, wl) in &sh.subs {
            let wloc = loc.then(&self.f.locs[wl]);
            let mut lp = Vec::new();
            for &(eo, es, el) in &self.f.shapes[ws].subs.clone() {
                // Relative to the face (its own orientation aside), then as the face uses it.
                let rel = wo.compose(eo);
                if !matches!(rel, Ori::Fwd | Ori::Rev) {
                    continue;
                }
                let eloc = wloc.then(&self.f.locs[el]);
                let e = self.edge(es, &eloc)?;
                let Kind::Edge(_, _, ref pcs) = self.f.shapes[es].kind else { unreachable!() };
                let rep = pcs.iter().find(|r| r.surf == si && loc_key(&eloc.then(&self.f.locs[r.loc])) == skey);
                let ed = &self.body.edges[e];
                let pc = match rep {
                    Some(rep) => {
                        let idx = match rep.pc2 {
                            Some(p2) if rel == Ori::Rev => p2,
                            _ => rep.pc,
                        };
                        to_curve(&self.f.curves2d[idx], ed.t[0], ed.t[1])?
                    }
                    // OCCT keeps no p-curves on planes (it projects the edge when asked).
                    None if matches!(surf, Surface::Plane { .. }) && !ed.degenerate => super::pcurve::pcurve(&surf, &ed.curve, ed.t[0], ed.t[1], None),
                    None => return Err(format!("edge {es} has no p-curve on face {s}")),
                };
                lp.push(Coedge { edge: e, fwd: ori.compose(rel) == Ori::Fwd, pc });
            }
            if !lp.is_empty() {
                loops.push(lp);
            }
        }
        let sense = ori == Ori::Fwd;
        for l in &mut loops {
            *l = chain(std::mem::take(l), &self.body).ok_or_else(|| format!("a loop of face {s} doesn't chain"))?;
            align_periods(&surf, l, &self.body);
        }
        // The outer loop first: the one enclosing the most (counter-clockwise about the outward
        // normal: positive in (u, v) for a forward face).
        let area = |l: &Vec<Coedge>| uv_area(l, &self.body) * if sense { 1.0 } else { -1.0 };
        if let Some(k) = (0..loops.len()).max_by(|&a, &b| area(&loops[a]).total_cmp(&area(&loops[b]))) {
            loops.swap(0, k);
        }
        self.body.faces.push(Face { surf, sense, loops });
        Ok(self.body.faces.len() - 1)
    }

    fn shell(&mut self, s: usize, ori: Ori, loc: &Loc) -> R<Vec<usize>> {
        let mut faces = Vec::new();
        for &(o, fs, l) in &self.f.shapes[s].subs.clone() {
            if matches!(self.f.shapes[fs].kind, Kind::Face(_)) {
                faces.push(self.face(fs, ori.compose(o), &loc.then(&self.f.locs[l]))?);
            }
        }
        Ok(faces)
    }

    fn visit(&mut self, s: usize, ori: Ori, loc: &Loc) -> R<()> {
        let sh = self.f.shapes[s].clone();
        match sh.kind {
            Kind::Compound => {
                for &(o, c, l) in &sh.subs {
                    self.visit(c, ori.compose(o), &loc.then(&self.f.locs[l]))?;
                }
            }
            Kind::Solid => {
                let mut shells = Vec::new();
                for &(o, c, l) in &sh.subs {
                    if matches!(self.f.shapes[c].kind, Kind::Shell) {
                        let faces = self.shell(c, ori.compose(o), &loc.then(&self.f.locs[l]))?;
                        self.shells.push((faces, true));
                        shells.push(self.shells.len() - 1);
                    }
                }
                self.solids.push(shells);
            }
            Kind::Shell => {
                let faces = self.shell(s, ori, loc)?;
                self.shells.push((faces, sh.closed));
            }
            Kind::Face(_) => {
                let f = self.face(s, ori, loc)?;
                self.free_faces.push(f);
            }
            _ => {}
        }
        Ok(())
    }
}

/// A loop's coedges in order, each starting where the one before ends (files list a wire's
/// edges in any order); at a fork, another edge before going straight back along the same one
/// (a seam's two uses). None when they don't close up.
fn chain(mut rest: Vec<Coedge>, b: &Body) -> Option<Vec<Coedge>> {
    let mut out = vec![rest.remove(0)];
    while !rest.is_empty() {
        let last = out.last().unwrap();
        let end = b.co_end(last);
        let i = rest
            .iter()
            .enumerate()
            .filter(|(_, c)| b.co_start(c) == end)
            .min_by_key(|(i, c)| ((c.edge == last.edge) as usize, *i))
            .map(|(i, _)| i)?;
        out.push(rest.remove(i));
    }
    (b.co_end(out.last().unwrap()) == b.co_start(&out[0])).then_some(out)
}

/// Shifts each p-curve of a loop by whole periods to start where the one before ended (loops
/// are continuous in (u, v); the file's p-curves may sit in any period).
fn align_periods(s: &Surface, l: &mut [Coedge], b: &Body) {
    let (pu, pv) = (if s.u_periodic() { Some(TAU) } else { None }, if s.v_periodic() { Some(TAU) } else { None });
    if pu.is_none() && pv.is_none() {
        return;
    }
    let ends = |c: &Coedge| {
        let (a, z) = co_range(&b.edges[c.edge], c.fwd);
        (c.pc.point(a), c.pc.point(z))
    };
    for k in 1..l.len() {
        let prev_end = ends(&l[k - 1]).1;
        let start = ends(&l[k]).0;
        let shift = |d: f64, per: Option<f64>| per.map_or(0.0, |p| (d / p).round() * p);
        let (du, dv) = (shift(prev_end.x - start.x, pu), shift(prev_end.y - start.y, pv));
        if du != 0.0 || dv != 0.0 {
            l[k].pc = l[k].pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)));
        }
    }
}

/// The signed area a loop encloses in (u, v) (sampled).
fn uv_area(l: &[Coedge], b: &Body) -> f64 {
    let mut pts: Vec<P3> = Vec::new();
    for c in l {
        let (a, z) = co_range(&b.edges[c.edge], c.fwd);
        for i in 0..16 {
            pts.push(c.pc.point(a + (z - a) * i as f64 / 16.0));
        }
    }
    let n = pts.len();
    (0..n).map(|i| { let (p, q) = (pts[i], pts[(i + 1) % n]); p.x * q.y - q.x * p.y }).sum::<f64>() / 2.0
}

fn build_direct(f: &File) -> R<Body> {
    let mut d = Direct { f, body: Body::default(), verts: HashMap::new(), grid: HashMap::new(), edges: HashMap::new(), shells: Vec::new(), solids: Vec::new(), free_faces: Vec::new() };
    let (o, s, l) = f.root;
    d.visit(s, o, &f.locs[l])?;
    if !d.free_faces.is_empty() {
        let ff = std::mem::take(&mut d.free_faces);
        d.shells.push((ff, false));
    }
    if d.body.faces.is_empty() {
        return Err("no faces".into());
    }
    let mut body = d.body;
    for (faces, closed) in d.shells {
        body.shells.push(Shell { faces, closed });
    }
    for shells in d.solids {
        body.solids.push(Solid { shells });
    }
    Ok(body)
}

/// What a BRep file holds.
pub struct BrepRead {
    /// The faces' body (solids, then sheets); None when the file has no faces.
    pub body: Option<Body>,
    /// The middle points of the body's edges in OCCT's exploration order (the order of first
    /// use, depth first, as `TopExp_Explorer` meets them; degenerate edges left out). The faces
    /// keep the file's order.
    pub edge_order: Vec<P3>,
    /// Edges outside faces (a wire's or a compound's): curve and parameter range.
    pub free_edges: Vec<(Curve, f64, f64)>,
    /// Whether the free edges are in wires (else loose edges).
    pub wires: bool,
    /// The vertices of a file with no faces or edges.
    pub points: Vec<P3>,
    pub warnings: Vec<String>,
}

/// [`read`], with free edges and OCCT's edge order.
pub fn read_ordered(bytes: &[u8]) -> R<BrepRead> {
    let text = String::from_utf8_lossy(bytes);
    let f = parse(&text)?;
    let mut xb = XBuilder::new();
    xb.trust_orientation = true;
    let mut b = Builder { f: &f, xb, verts: HashMap::new(), edges: HashMap::new(), free_faces: Vec::new(), free_edges: Vec::new(), wires: false, warnings: Vec::new() };
    let (o, s, l) = f.root;
    b.visit(s, o, &f.locs[l]);
    let free = std::mem::take(&mut b.free_faces);
    if !free.is_empty() {
        b.xb.shell(free, false);
    }
    let mut warnings = std::mem::take(&mut b.warnings);
    let free_edges: Vec<(Curve, f64, f64)> = b.free_edges.iter().map(|&e| {
        let ed = &b.xb.body.edges[e];
        (ed.curve.clone(), ed.t[0], ed.t[1])
    }).collect();
    let mids: Vec<P3> = b.xb.body.edges.iter().map(|e| e.point(e.mid_t())).collect();
    let has_faces = !b.xb.faces.is_empty();
    // Straight from the file when that gives a valid body; else rebuilt.
    let direct = if has_faces && std::env::var_os("CADRS_OCCT_REBUILD").is_none() {
        match build_direct(&f) {
            Ok(x) => {
                let issues = x.check();
                if issues.is_empty() {
                    Some(x)
                } else {
                    warnings.push(format!("rebuilt (the file's body: {})", issues[0]));
                    None
                }
            }
            Err(e) => {
                warnings.push(format!("rebuilt ({e})"));
                None
            }
        }
    } else {
        None
    };
    let body = if let Some(x) = direct {
        Some(x)
    } else if has_faces {
        let (body, w) = b.xb.finish();
        warnings.extend(w);
        Some(body).filter(|x| !x.faces.is_empty())
    } else {
        None
    };
    // A file of only vertices (or of edges without curves, a wire collapsed to a point).
    let mut points: Vec<P3> = Vec::new();
    if body.is_none() && free_edges.is_empty() {
        fn walk(f: &File, s: usize, loc: &Loc, out: &mut Vec<P3>) {
            if let Kind::Vertex(p) = f.shapes[s].kind {
                let q = loc.m.point(&p);
                if !out.iter().any(|x| (x - q).norm() < 1e-7) {
                    out.push(q);
                }
            }
            for &(_, c, l) in &f.shapes[s].subs {
                walk(f, c, &loc.then(&f.locs[l]), out);
            }
        }
        walk(&f, s, &f.locs[l], &mut points);
        if points.is_empty() {
            return Err(format!("no faces or edges read ({})", warnings.join("; ")));
        }
    }
    Ok(BrepRead { body, edge_order: mids, free_edges, wires: b.wires, points, warnings })
}

/// Reads an OCCT BRep text file into a body (its solids, then sheets) and the problems met.
pub fn read(bytes: &[u8]) -> R<(Body, Vec<String>)> {
    let r = read_ordered(bytes)?;
    let warnings = r.warnings;
    r.body.map(|b| (b, warnings)).ok_or_else(|| "no faces read".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unit box as OCCT 7.8 writes it (`box b 1 2 3; save b`), trimmed of its p-curves.
    const BOX: &str = include_str!("../../tests/data/occt_box.brep");

    #[test]
    fn reads_a_box() {
        let (b, w) = read(BOX.as_bytes()).unwrap();
        assert_eq!((b.verts.len(), b.edges.len(), b.faces.len(), b.solids.len()), (8, 12, 6, 1), "{w:?}");
        let v = super::super::mass::body_mass(&b).volume;
        assert!((v - 6.0).abs() < 1e-9, "{v}");
    }

    #[test]
    fn periodic_splines_unroll() {
        // A periodic quadratic through 4 poles (a closed curve): unrolled, it closes and runs
        // through the same points as its rotated copy.
        let poles: Vec<H> = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)].iter().map(|&(x, y)| hom(&P3::new(x, y, 0.0), 1.0)).collect();
        let s = Spline { degree: 2, poles, knots: vec![0.0, 1.0, 2.0, 3.0, 4.0], mults: vec![1; 5], periodic: true };
        let c = spline_curve(&s, 0.0, 4.0).unwrap();
        assert!((c.point(0.0) - c.point(4.0)).norm() < 1e-12);
        let shifted = spline_curve(&s, 1.5, 5.5).unwrap();
        for k in 0..=8 {
            let t = 1.5 + 0.25 * k as f64;
            let q = if t > 4.0 { t - 4.0 } else { t };
            assert!((shifted.point(t) - c.point(q)).norm() < 1e-12, "{t}");
        }
        // Quadratic B-spline with knots i: at knot 1 the point is the mean of poles 0 and 1.
        let m = (c.point(1.0) - P3::new(0.5, 0.5, 0.0)).norm();
        let m2 = (c.point(1.0) - P3::new(-0.5, 0.5, 0.0)).norm();
        assert!(m < 1e-12 || m2 < 1e-12, "{:?}", c.point(1.0));
    }
}
