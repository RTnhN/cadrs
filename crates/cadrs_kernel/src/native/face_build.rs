//! A sheet face from a bare surface (SMLib's `SmBrep::CreateFaceFromSurface`): the surface over
//! its natural uv domain, or a given rectangle of it, bounded by its four iso-curves.
//!
//! The rectangle's sides (counter-clockwise in uv: v = v0, u = u1, v = v1, u = u0) become the
//! loop. A side that collapses to a point (a pole: a sphere's, or an approximate one of a
//! B-spline whose last row of control points nearly meets) is a degenerate edge at one vertex;
//! where the rectangle spans a closed direction of the surface (a whole period, or a B-spline
//! whose ends meet) the two opposite sides are one seam edge, used once each way. Corners that
//! meet are one vertex. Edge curves are the exact iso-curves (lines, circles, rotated meridians,
//! a B-spline's rows of control points blended by its basis functions); the face's normal is
//! the surface's (Su × Sv).
//!
//! So a sphere gives 1 face, 1 edge (its seam), 2 vertices (its poles) and 2 degenerate edges; a
//! torus 1 face, 2 seam edges and 1 vertex; a plane rectangle 4 edges and 4 vertices. The one
//! shell is closed when every edge is a seam.

use super::curve::Curve;
use super::math::*;
use super::nurbs::{basis_ders, find_span, NurbsCurve, NurbsSurface, H};
use super::surface::Surface;
use super::topo::{Body, Coedge, Face, Shell};
use crate::{KernelError, Result};

/// A uv rectangle: ((u0, u1), (v0, v1)).
pub type UvBox = ((f64, f64), (f64, f64));

/// The surface's natural domain: a B-spline's knot range, one period of a periodic direction,
/// a bounded meridian's or extruded curve's range; `None` when unbounded (planes, the length of
/// a cylinder or cone, an extrusion's direction).
pub fn natural_domain(s: &Surface) -> Option<UvBox> {
    let ((u0, u1), (v0, v1)) = s.param_box();
    let bounded = |a: f64, b: f64| a > -1e4 + 1.0 && b < 1e4 - 1.0;
    (bounded(u0, u1) && bounded(v0, v1)).then_some(((u0, u1), (v0, v1)))
}

/// A sheet body of one face: `s` over `domain` (its natural domain when `None`). See the module
/// docs. Errors: an unbounded surface without a domain, an empty or non-finite rectangle, one
/// more than a period wide, or outside a B-spline's range in a direction where it has several
/// spans or its ends meet (a Bézier direction goes on as its polynomial).
pub fn face_from_surface(s: &Surface, domain: Option<UvBox>) -> Result<Body> {
    let natural = natural_domain(s);
    let ((u0, u1), (v0, v1)) = match domain.or(natural) {
        Some(d) => d,
        None => return Err(KernelError::InvalidParameter("the surface is unbounded: give the uv domain of the face to make".into())),
    };
    if ![u0, u1, v0, v1].iter().all(|x| x.is_finite()) || u1 <= u0 || v1 <= v0 {
        return Err(KernelError::InvalidParameter(format!("an empty uv domain [{u0}, {u1}] × [{v0}, {v1}]")));
    }
    // A Bézier patch (one span in a direction, ends apart) goes on as its polynomial past its
    // range, as OCCT evaluates it: the face is made on the same polynomial re-expressed over
    // the domain.
    let extended;
    let s = match s {
        Surface::Nurbs(n) => match bezier_extended(n, (u0, u1), (v0, v1)) {
            Some(e) => {
                extended = Surface::Nurbs(std::sync::Arc::new(e));
                &extended
            }
            None => s,
        },
        Surface::Extrusion { c: Curve::Nurbs(c), d } => match bezier_curve_extended(c, (u0, u1)) {
            Some(e) => {
                extended = Surface::Extrusion { c: Curve::nurbs(e), d: *d };
                &extended
            }
            None => s,
        },
        _ => s,
    };
    let ((pu0, pu1), (pv0, pv1)) = s.param_box();
    // Angular directions: a periodic one, or the v of a revolution of a circle or ellipse.
    let (per_u, per_v) = (s.u_periodic(), s.v_periodic() || matches!(s, Surface::Revolution { m, .. } if m.is_periodic()));
    let slack = |a: f64, b: f64| 1e-9 * (1.0 + a.abs().max(b.abs()));
    if !per_u && (u0 < pu0 - slack(pu0, pu1) || u1 > pu1 + slack(pu0, pu1)) {
        return Err(KernelError::InvalidParameter(format!("u [{u0}, {u1}] is outside the surface's [{pu0}, {pu1}]")));
    }
    if !per_v && (v0 < pv0 - slack(pv0, pv1) || v1 > pv1 + slack(pv0, pv1)) {
        return Err(KernelError::InvalidParameter(format!("v [{v0}, {v1}] is outside the surface's [{pv0}, {pv1}]")));
    }
    if (per_u && u1 - u0 > TAU + 1e-9) || (per_v && v1 - v0 > TAU + 1e-9) {
        return Err(KernelError::InvalidParameter("the uv domain is wider than the surface's period".into()));
    }
    let (u0, u1) = if per_u { (u0, u1) } else { (u0.max(pu0), u1.min(pu1)) };
    let (v0, v1) = if per_v { (v0, v1) } else { (v0.max(pv0), v1.min(pv1)) };

    let size = s.patch_bbox((u0, u1), (v0, v1)).diagonal().max(1e-9);
    // Sides and poles meet within this: approximate poles and seams of B-splines read from
    // other systems are off by more than exact geometry (1e-6 of the face's size).
    let tol = (1e-6 * size).max(super::tol::CONFUSION);
    // Longest distance between samples of two curves of the surface, or of one curve from its
    // first point.
    let spread = |f: &dyn Fn(f64) -> (P3, P3)| (0..=16).map(|i| f(i as f64 / 16.0)).map(|(p, q)| (p - q).norm()).fold(0.0, f64::max);
    let closed_u = spread(&|f| {
        let v = v0 + (v1 - v0) * f;
        (s.eval(u0, v), s.eval(u1, v))
    }) <= tol;
    let closed_v = spread(&|f| {
        let u = u0 + (u1 - u0) * f;
        (s.eval(u, v0), s.eval(u, v1))
    }) <= tol;
    // Sides: 0 v = v0 (u0 → u1), 1 u = u1 (v0 → v1), 2 v = v1 (u1 → u0), 3 u = u0 (v1 → v0).
    let side_pole = |k: usize| -> f64 {
        let (fixed_u, at) = match k {
            0 => (false, v0),
            1 => (true, u1),
            2 => (false, v1),
            _ => (true, u0),
        };
        let p0 = if fixed_u { s.eval(at, v0) } else { s.eval(u0, at) };
        spread(&|f| if fixed_u { (s.eval(at, v0 + (v1 - v0) * f), p0) } else { (s.eval(u0 + (u1 - u0) * f, at), p0) })
    };
    let pole: Vec<f64> = (0..4).map(side_pole).collect();
    let is_pole = |k: usize| pole[k] <= tol;

    let mut b = Body::default();
    // Corners (u0, v0), (u1, v0), (u1, v1), (u0, v1), merged where they meet.
    let corners = [(u0, v0), (u1, v0), (u1, v1), (u0, v1)];
    let mut cv = [0usize; 4];
    for (k, &(u, v)) in corners.iter().enumerate() {
        let p = s.eval(u, v);
        cv[k] = match (0..k).find(|&j| (b.verts[cv[j]].p - p).norm() <= tol) {
            Some(j) => cv[j],
            None => b.add_vertex(p),
        };
    }
    // Vertex tolerances cover the poles' and seams' spread.
    for k in 0..4 {
        if is_pole(k) {
            let v = cv[k];
            b.verts[v].tol = b.verts[v].tol.max(pole[k] * 1.01);
        }
    }
    for k in 0..4 {
        let p = s.eval(corners[k].0, corners[k].1);
        let v = cv[k];
        let d = (b.verts[v].p - p).norm();
        if d > b.verts[v].tol {
            b.verts[v].tol = d * 1.01;
        }
    }

    let line_uv = |o: (f64, f64), d: (f64, f64)| Curve::Line { o: P3::new(o.0, o.1, 0.0), d: V3::new(d.0, d.1, 0.0) };
    // The edge of each side: (edge, used forward).
    let mut side_edge: [(usize, bool); 4] = [(0, true); 4];
    let mut pcs: [Option<Curve>; 4] = [None, None, None, None];
    for k in 0..4 {
        // Along u (sides 0 and 2: p-curve (t, v)), or along v (sides 1 and 3: (u, t)).
        let along_u = k % 2 == 0;
        let fixed = [v0, u1, v1, u0][k];
        let pc = if along_u { line_uv((0.0, fixed), (1.0, 0.0)) } else { line_uv((fixed, 0.0), (0.0, 1.0)) };
        let (t0, t1) = if along_u { (u0, u1) } else { (v0, v1) };
        // Forward sides run with t (0 and 1); 2 and 3 run against it.
        let fwd = k < 2;
        // Edge ends at t0 and t1.
        let (va, vb) = match k {
            0 => (cv[0], cv[1]),
            1 => (cv[1], cv[2]),
            2 => (cv[3], cv[2]),
            _ => (cv[0], cv[3]),
        };
        pcs[k] = Some(pc);
        // The second side of a seam reuses the first's edge.
        if (k == 2 && closed_v && !is_pole(0)) || (k == 3 && closed_u && !is_pole(1)) {
            side_edge[k] = (side_edge[k - 2].0, false);
            continue;
        }
        let e = if is_pole(k) {
            b.add_degenerate(b.verts[va].p, t0, t1, va)
        } else {
            let c = iso_curve(s, !along_u, fixed).ok_or_else(|| KernelError::OperationFailed("no iso-curve of the surface for the face's side".into()))?;
            let e = b.add_edge(c, t0, t1, va, vb);
            // The iso-curve against the surface at the other end of a seam, and the corners.
            let mut dev: f64 = 0.0;
            for i in 0..=8 {
                let t = t0 + (t1 - t0) * i as f64 / 8.0;
                let q = if along_u { s.eval(t, fixed) } else { s.eval(fixed, t) };
                dev = dev.max((b.edges[e].point(t) - q).norm());
            }
            if dev > b.edges[e].tol {
                b.edges[e].tol = dev * 1.01;
            }
            e
        };
        side_edge[k] = (e, fwd);
    }
    let lp: Vec<Coedge> = (0..4)
        .map(|k| {
            let (edge, fwd) = side_edge[k];
            let pc = pcs[k].clone().unwrap_or_else(|| line_uv((0.0, 0.0), (1.0, 0.0)));
            Coedge { edge, fwd, pc }
        })
        .collect();
    // A degenerate side's edge is its own (never a seam): sides 2 and 3 reusing a pole's edge
    // would be wrong, which the `!is_pole` guards above exclude.
    b.faces.push(Face { surf: s.clone(), sense: true, loops: vec![lp] });
    // Seam tolerance: the two sides of a seam lie within `tol` of each other.
    let closed = b.edges.iter().enumerate().all(|(ei, e)| e.degenerate || b.faces[0].loops[0].iter().filter(|c| c.edge == ei).count() == 2);
    b.shells.push(Shell { faces: vec![0], closed });
    b.fit_tolerances(tol.max(1e-3 * size));
    Ok(b)
}

/// `n` re-expressed over a uv domain reaching past its range in a direction where it is a
/// single polynomial span whose ends don't meet (a Bézier direction); None when the domain is
/// inside the range, or reaches past it in a direction that isn't such a span.
fn bezier_extended(n: &NurbsSurface, u: (f64, f64), v: (f64, f64)) -> Option<NurbsSurface> {
    let ((ru0, ru1), (rv0, rv1)) = n.range();
    let past = |(a, b): (f64, f64), (r0, r1): (f64, f64)| a < r0 - 1e-9 * (1.0 + r0.abs()) || b > r1 + 1e-9 * (1.0 + r1.abs());
    let (pu, pv) = (past(u, (ru0, ru1)), past(v, (rv0, rv1)));
    if !pu && !pv {
        return None;
    }
    let single = |deg: usize, cnt: usize| cnt == deg + 1;
    let size = n.cps.iter().flatten().map(|h| super::nurbs::dehom(h).coords.norm()).fold(1.0, f64::max);
    let meets = |along_u: bool| {
        (0..=4).all(|k| {
            let f = k as f64 / 4.0;
            let (p, q) = if along_u {
                let w = rv0 + (rv1 - rv0) * f;
                (n.point(ru0, w), n.point(ru1, w))
            } else {
                let w = ru0 + (ru1 - ru0) * f;
                (n.point(w, rv0), n.point(w, rv1))
            };
            (p - q).norm() <= 1e-9 * size
        })
    };
    if (pu && (!single(n.du, n.cps.len()) || meets(true))) || (pv && (!single(n.dv, n.cps[0].len()) || meets(false))) {
        return None;
    }
    let mut cps = n.cps.clone();
    let (mut ku, mut kv) = (n.ku.clone(), n.kv.clone());
    if pu {
        let cols: Vec<Vec<H>> = (0..cps[0].len()).map(|j| bezier_over(&cps.iter().map(|row| row[j]).collect::<Vec<_>>(), (ru0, ru1), u)).collect();
        cps = (0..cols[0].len()).map(|i| cols.iter().map(|c| c[i]).collect()).collect();
        ku = [vec![u.0; n.du + 1], vec![u.1; n.du + 1]].concat();
    }
    if pv {
        cps = cps.iter().map(|row| bezier_over(row, (rv0, rv1), v)).collect();
        kv = [vec![v.0; n.dv + 1], vec![v.1; n.dv + 1]].concat();
    }
    NurbsSurface::new(n.du, n.dv, ku, kv, cps).ok()
}

/// A Bézier curve (one span, ends apart) re-expressed over [a, b] reaching past its range;
/// None when [a, b] is inside it or the curve isn't such a span.
fn bezier_curve_extended(c: &NurbsCurve, (a, b): (f64, f64)) -> Option<NurbsCurve> {
    let (r0, r1) = c.range();
    let past = a < r0 - 1e-9 * (1.0 + r0.abs()) || b > r1 + 1e-9 * (1.0 + r1.abs());
    let size = c.cps.iter().map(|h| super::nurbs::dehom(h).coords.norm()).fold(1.0, f64::max);
    if !past || c.cps.len() != c.degree + 1 || (c.point(r0) - c.point(r1)).norm() <= 1e-9 * size {
        return None;
    }
    NurbsCurve::new(c.degree, [vec![a; c.degree + 1], vec![b; c.degree + 1]].concat(), bezier_over(&c.cps, (r0, r1), (a, b))).ok()
}

/// The control points (homogeneous) over [a, b] of the Bézier polynomial with control points
/// `cps` over `range`: its blossom at (a, …, a, b, …, b) (de Casteljau, which extrapolates).
fn bezier_over(cps: &[H], range: (f64, f64), (a, b): (f64, f64)) -> Vec<H> {
    let p = cps.len() - 1;
    let (sa, sb) = ((a - range.0) / (range.1 - range.0), (b - range.0) / (range.1 - range.0));
    (0..=p)
        .map(|i| {
            let mut c = cps.to_vec();
            for r in 0..p {
                let x = if r < p - i { sa } else { sb };
                for j in 0..p - r {
                    c[j] = c[j] * (1.0 - x) + c[j + 1] * x;
                }
            }
            c[0]
        })
        .collect()
}

/// The iso-curve of `s` at u = `at` (parametrized by v) when `fixed_u`, else at v = `at`
/// (parametrized by u): exact for every kind of surface.
pub fn iso_curve(s: &Surface, fixed_u: bool, at: f64) -> Option<Curve> {
    match s {
        Surface::Nurbs(n) => Some(Curve::nurbs(nurbs_iso(n, fixed_u, at))),
        Surface::Revolution { f, m } => {
            if fixed_u {
                // The meridian turned by u about the axis.
                let (sn, cs) = at.sin_cos();
                let fm = nalgebra::Matrix3::from_columns(&[f.x, f.y, f.z]);
                let rz = nalgebra::Matrix3::new(cs, -sn, 0.0, sn, cs, 0.0, 0.0, 0.0, 1.0);
                let lin = fm * rz * fm.transpose();
                Some(m.transformed(&Affine { m: lin, t: f.o.coords - lin * f.o.coords }))
            } else {
                let l = f.local(&m.point(at));
                let rho = l.x.hypot(l.y);
                if rho < 1e-300 {
                    return None;
                }
                let x = (f.x * l.x + f.y * l.y) / rho;
                let y = (f.y * l.x - f.x * l.y) / rho;
                Some(Curve::Circle { c: f.o + f.z * l.z, x, y, r: rho })
            }
        }
        Surface::Extrusion { c, d } if fixed_u => Some(Curve::Line { o: c.point(at), d: *d }),
        _ if fixed_u => s.iso_u(at),
        _ => s.iso_v(at),
    }
}

/// A B-spline surface's iso-curve: its rows of control points blended by the basis functions
/// at the fixed parameter (homogeneous, so exact for rational surfaces).
fn nurbs_iso(n: &NurbsSurface, fixed_u: bool, at: f64) -> NurbsCurve {
    let (nu, nv) = (n.cps.len(), n.cps[0].len());
    if fixed_u {
        let span = find_span(nu - 1, n.du, at, &n.ku);
        let basis = &basis_ders(span, at, n.du, 0, &n.ku)[0];
        let cps: Vec<H> = (0..nv).map(|j| (0..=n.du).map(|r| n.cps[span - n.du + r][j] * basis[r]).sum()).collect();
        NurbsCurve { degree: n.dv, knots: n.kv.clone(), cps }
    } else {
        let span = find_span(nv - 1, n.dv, at, &n.kv);
        let basis = &basis_ders(span, at, n.dv, 0, &n.kv)[0];
        let cps: Vec<H> = (0..nu).map(|i| (0..=n.dv).map(|r| n.cps[i][span - n.dv + r] * basis[r]).sum()).collect();
        NurbsCurve { degree: n.du, knots: n.ku.clone(), cps }
    }
}

/// [`crate::Kernel::face_from_surface`] for the native kernel.
pub fn kernel_face_from_surface(k: &mut super::NativeKernel, body: crate::BodyId, face: crate::FaceId, domain: Option<crate::UvRect>) -> Result<crate::OpResult> {
    let s = {
        let b = k.get(body)?;
        let Some(f) = b.faces.get(face.0 as usize) else {
            return Err(KernelError::InvalidParameter(format!("{body:?} has no face {}", face.0)));
        };
        f.surf.clone()
    };
    let b = face_from_surface(&s, domain.map(|d| ((d[0][0], d[0][1]), (d[1][0], d[1][1]))))?;
    Ok(crate::OpResult { bodies: vec![k.insert(b)], history: crate::History::default() })
}

impl super::NativeKernel {
    /// A sheet body of one face of a bare surface (one read from a file, say) over its natural
    /// domain or `domain` ([`face_from_surface`]).
    pub fn face_from_bare_surface(&mut self, s: &Surface, domain: Option<UvBox>) -> Result<crate::BodyId> {
        let b = face_from_surface(s, domain)?;
        Ok(self.insert(b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn counts(b: &Body) -> (usize, usize, usize, usize) {
        (b.faces.len(), b.edges.iter().filter(|e| !e.degenerate).count(), b.edges.iter().filter(|e| e.degenerate).count(), b.verts.len())
    }

    #[test]
    fn sphere_torus_cylinder_and_plane() {
        let sphere = face_from_surface(&Surface::Sphere { f: Frame::world(), r: 2.0 }, None).unwrap();
        assert!(sphere.check().is_empty(), "{:?}", sphere.check());
        assert_eq!(counts(&sphere), (1, 1, 2, 2));
        assert!(sphere.shells[0].closed);
        let a = super::super::mass::face_area(&sphere, 0).0;
        assert!((a - 16.0 * PI).abs() < 1e-6, "{a}");

        let torus = face_from_surface(&Surface::Torus { f: Frame::world(), rmaj: 5.0, rmin: 1.0 }, None).unwrap();
        assert!(torus.check().is_empty(), "{:?}", torus.check());
        assert_eq!(counts(&torus), (1, 2, 0, 1));
        let a = super::super::mass::face_area(&torus, 0).0;
        assert!((a - 4.0 * PI * PI * 5.0).abs() < 1e-6, "{a}");

        // Unbounded without a domain; a half cylinder with one.
        let cyl = Surface::Cylinder { f: Frame::world(), r: 1.0 };
        assert!(face_from_surface(&cyl, None).is_err());
        let half = face_from_surface(&cyl, Some(((0.0, PI), (0.0, 3.0)))).unwrap();
        assert!(half.check().is_empty(), "{:?}", half.check());
        assert_eq!(counts(&half), (1, 4, 0, 4));
        assert!(!half.shells[0].closed);
        let a = super::super::mass::face_area(&half, 0).0;
        assert!((a - 3.0 * PI).abs() < 1e-9, "{a}");
        let full = face_from_surface(&cyl, Some(((0.0, TAU), (0.0, 3.0)))).unwrap();
        assert!(full.check().is_empty(), "{:?}", full.check());
        assert_eq!(counts(&full), (1, 3, 0, 2));

        let plane = face_from_surface(&Surface::Plane { f: Frame::world() }, Some(((0.0, 2.0), (0.0, 3.0)))).unwrap();
        assert_eq!(counts(&plane), (1, 4, 0, 4));
        assert!((super::super::mass::face_area(&plane, 0).0 - 6.0).abs() < 1e-12);
        assert!(face_from_surface(&Surface::Plane { f: Frame::world() }, Some(((1.0, 1.0), (0.0, 3.0)))).is_err());
    }

    /// A Bézier patch made a face past its [0, 1] range (OCCT evaluates it there): the face
    /// is the same polynomial, its corners where the patch's extrapolation puts them.
    #[test]
    fn bezier_past_its_range() {
        // z = x·y over [0, 1]², bilinear.
        let h = |x: f64, y: f64| H::new(x, y, x * y, 1.0);
        let n = NurbsSurface::new(1, 1, vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0], vec![vec![h(0.0, 0.0), h(0.0, 1.0)], vec![h(1.0, 0.0), h(1.0, 1.0)]]).unwrap();
        let s = Surface::Nurbs(Arc::new(n));
        let b = face_from_surface(&s, Some(((-1.0, 2.0), (-0.5, 1.5)))).unwrap();
        assert!(b.check().is_empty(), "{:?}", b.check());
        assert_eq!(counts(&b), (1, 4, 0, 4));
        let Surface::Nurbs(e) = &b.faces[0].surf else { panic!() };
        let p = e.point(2.0, 1.5);
        assert!((p - P3::new(2.0, 1.5, 3.0)).norm() < 1e-12, "{p:?}");
        // A periodic surface still refuses more than a period; a closed B-spline past its range too.
        assert!(face_from_surface(&Surface::Sphere { f: Frame::world(), r: 1.0 }, Some(((0.0, 7.0), (-1.0, 1.0)))).is_err());
        let sph = Surface::Sphere { f: Frame::world(), r: 2.0 }.to_nurbs((0.0, TAU), (-PI / 2.0, PI / 2.0));
        let ((a, b2), (c, d)) = sph.range();
        assert!(face_from_surface(&Surface::Nurbs(Arc::new(sph)), Some(((a - 1.0, b2), (c, d)))).is_err());
    }

    /// OCCT's mkface on an extrusion of a Bézier curve past the curve's [0, 1] (mkface c6–c9).
    #[test]
    fn extruded_bezier_past_its_range() {
        // y = x² over [0, 1], extruded along z.
        let c = NurbsCurve::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0], vec![H::new(0.0, 0.0, 0.0, 1.0), H::new(0.5, 0.0, 0.0, 1.0), H::new(1.0, 1.0, 0.0, 1.0)]).unwrap();
        let s = Surface::Extrusion { c: Curve::nurbs(c), d: V3::z() };
        let b = face_from_surface(&s, Some(((0.0, 3.0), (0.0, 2.0)))).unwrap();
        assert!(b.check().is_empty(), "{:?}", b.check());
        assert_eq!(counts(&b), (1, 4, 0, 4));
        assert!(b.verts.iter().any(|v| (v.p - P3::new(3.0, 9.0, 2.0)).norm() < 1e-9), "{:?}", b.verts);
    }

    /// A sphere as a B-spline (its rational quadratic net, poles as rows of equal points) and as
    /// a revolution of a half circle.
    #[test]
    fn nurbs_and_revolved_sphere() {
        let s = Surface::Sphere { f: Frame::world(), r: 2.0 }.to_nurbs((0.0, TAU), (-PI / 2.0, PI / 2.0));
        let b = face_from_surface(&Surface::Nurbs(Arc::new(s)), None).unwrap();
        assert!(b.check().is_empty(), "{:?}", b.check());
        assert_eq!(counts(&b), (1, 1, 2, 2));
        let a = super::super::mass::face_area(&b, 0).0;
        assert!((a - 16.0 * PI).abs() < 1e-6, "{a}");
        let m = Curve::Circle { c: P3::origin(), x: V3::x(), y: V3::z(), r: 2.0 };
        let rev = face_from_surface(&Surface::Revolution { f: Frame::world(), m }, Some(((0.0, TAU), (-PI / 2.0, PI / 2.0)))).unwrap();
        assert!(rev.check().is_empty(), "{:?}", rev.check());
        assert_eq!(counts(&rev), (1, 1, 2, 2));
    }
}
