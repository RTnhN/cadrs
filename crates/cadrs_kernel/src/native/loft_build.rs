//! Lofts (P3.7, PS20).
//!
//! - **Exact skinning** (no end conditions, sections with the same number of curves): each
//!   section curve becomes a rational B-spline (lines and conics exactly), made compatible by
//!   knot insertion; two sections give the ruled surface between them (planes, cones and
//!   prisms exactly), more are interpolated through the sections' poles. Faces of the loft:
//!   one per section curve, the caps the first and last sections (a face section is its own
//!   cap), a point section an apex.
//! - **Sampled interpolation** (end conditions, or sections that don't match): every section is
//!   sampled with [`crate::loft::sample_sections`], interpolated by a cubic B-spline (all with
//!   the same knots), and the columns of poles by cubics through the sections with the end
//!   derivatives of the conditions (as the OCCT backend's interpolated loft).

use std::sync::Arc;

use super::build::{fix_senses, Built};
use super::curve::Curve;
use super::features::Tagged;
use super::kernel::NativeKernel;
use super::math::*;
use super::nurbs::{dehom, hom, NurbsCurve, NurbsSurface, H};
use super::surface::Surface;
use super::topo::{Body, Coedge, Face, Shell, Solid};
use crate::loft::{sample_sections, section_loop, SectionCurves};
use crate::*;

/// One curve of a section in 3D, in the loop's direction, with its source.
#[derive(Debug, Clone)]
struct Part {
    c: Curve,
    t0: f64,
    t1: f64,
    rev: bool,
    src: u64,
}

impl Part {
    fn start(&self) -> P3 {
        self.c.point(if self.rev { self.t1 } else { self.t0 })
    }
    /// An exact B-spline of the part in its direction of travel, on [0, 1].
    fn nurbs(&self) -> NurbsCurve {
        let n = self.c.to_nurbs(self.t0, self.t1);
        let n = if self.rev { n.reversed() } else { n };
        let n = n.reparam(0.0, 1.0);
        // Lines as quadratics so they match conics.
        if n.degree == 1 && n.cps.len() == 2 {
            let (a, b) = (dehom(&n.cps[0]), dehom(&n.cps[1]));
            return NurbsCurve::bezier(&[a, nalgebra::center(&a, &b), b], &[1.0, 1.0, 1.0]);
        }
        n
    }
}

#[derive(Debug, Clone)]
enum Sec {
    Loop { parts: Vec<Part>, plane: Option<Plane>, face: Option<FaceInput> },
    Point(P3),
}

fn invalid(why: &str) -> KernelError {
    KernelError::InvalidParameter(why.into())
}

/// A section as 3D parts.
fn section(k: &NativeKernel, s: &LoftSection) -> Result<Sec> {
    match s {
        LoftSection::Point(p) => Ok(Sec::Point(*p)),
        LoftSection::Profile(p) => {
            let curves = section_loop(p)?;
            let m = super::build::plane_map(&p.plane, &V3::zeros(), 0.0);
            let mut parts = Vec::new();
            for (i, c) in curves.iter().enumerate() {
                let src = c.source().unwrap_or(unsourced_curve(0, i));
                let pc = super::profile::piece_of(c, super::profile::Src::Curve { region: 0, curve: src })?;
                parts.push(Part { c: pc.c.transformed(&m), t0: pc.t0, t1: pc.t1, rev: pc.rev, src });
            }
            Ok(Sec::Loop { parts, plane: Some(p.plane), face: None })
        }
        LoftSection::Face(fi) => {
            let b = k.get(fi.body)?;
            let f = b.faces.get(fi.face.0 as usize).ok_or_else(|| invalid("unknown face"))?;
            // The outer loop, run so the face's material is on the left seen from outside.
            let lp = &f.loops[0];
            let mut parts = Vec::new();
            for c in lp {
                let e = &b.edges[c.edge];
                if e.degenerate {
                    continue;
                }
                parts.push(Part { c: e.curve.clone(), t0: e.t[0], t1: e.t[1], rev: !c.fwd, src: c.edge as u64 });
            }
            let plane = match &f.surf {
                Surface::Plane { f: fr } => {
                    let n = fr.x.cross(&fr.y) * if f.sense { 1.0 } else { -1.0 };
                    Some(Plane { origin: fr.o, x_dir: nalgebra::Unit::new_normalize(fr.x), normal: nalgebra::Unit::new_normalize(n) })
                }
                _ => None,
            };
            Ok(Sec::Loop { parts, plane, face: Some(*fi) })
        }
    }
}

fn centroid(s: &Sec) -> P3 {
    match s {
        Sec::Point(p) => *p,
        Sec::Loop { parts, .. } => {
            let mut acc = V3::zeros();
            let mut n = 0.0;
            for p in parts {
                for k in 0..16 {
                    let t = p.t0 + (p.t1 - p.t0) * k as f64 / 16.0;
                    acc += p.c.point(t).coords;
                    n += 1.0;
                }
            }
            P3::from(acc / n)
        }
    }
}

/// Signed area of a loop about an axis (Newell), to orient sections the same way round.
fn turning(parts: &[Part], axis: &V3) -> f64 {
    let mut pts = Vec::new();
    for p in parts {
        for k in 0..32 {
            let s = k as f64 / 32.0;
            let t = if p.rev { p.t1 + (p.t0 - p.t1) * s } else { p.t0 + (p.t1 - p.t0) * s };
            pts.push(p.c.point(t));
        }
    }
    let n = pts.len();
    let mut a = V3::zeros();
    for i in 0..n {
        a += pts[i].coords.cross(&pts[(i + 1) % n].coords);
    }
    a.dot(axis)
}

fn reverse_parts(parts: &[Part]) -> Vec<Part> {
    parts.iter().rev().map(|p| Part { rev: !p.rev, ..p.clone() }).collect()
}

/// Makes curves compatible: the same degree (all quadratic or all cubic) and knots.
fn compatible(curves: &mut [NurbsCurve]) -> bool {
    let deg = curves.iter().map(|c| c.degree).max().unwrap_or(1);
    if curves.iter().any(|c| c.degree != deg) {
        return false;
    }
    // Union of the interior knots (with multiplicity).
    let mut all: Vec<f64> = Vec::new();
    for c in curves.iter() {
        let mut ks: Vec<f64> = c.knots[deg + 1..c.knots.len() - deg - 1].to_vec();
        // merge with multiplicities
        let mut merged: Vec<f64> = Vec::new();
        let (mut i, mut j) = (0, 0);
        ks.sort_by(f64::total_cmp);
        while i < all.len() || j < ks.len() {
            if j >= ks.len() || (i < all.len() && all[i] < ks[j] - 1e-12) {
                merged.push(all[i]);
                i += 1;
            } else if i >= all.len() || ks[j] < all[i] - 1e-12 {
                merged.push(ks[j]);
                j += 1;
            } else {
                merged.push(all[i]);
                i += 1;
                j += 1;
            }
        }
        all = merged;
    }
    for c in curves.iter_mut() {
        let mut have: Vec<f64> = c.knots[deg + 1..c.knots.len() - deg - 1].to_vec();
        for &k in &all {
            let count_all = all.iter().filter(|x| (*x - k).abs() < 1e-12).count();
            let count_have = have.iter().filter(|x| (*x - k).abs() < 1e-12).count();
            if count_have < count_all {
                *c = c.insert_knot(k);
                have.push(k);
            }
        }
    }
    true
}

/// The loft's lateral surfaces from per-section compatible curves (homogeneous poles
/// interpolated across the sections at `v`).
fn skin(curves: &[NurbsCurve], v: &[f64]) -> NurbsSurface {
    let ncp = curves[0].cps.len();
    if curves.len() == 2 {
        // Ruled.
        let cps: Vec<Vec<H>> = (0..ncp).map(|i| vec![curves[0].cps[i], curves[1].cps[i]]).collect();
        return NurbsSurface { du: curves[0].degree, dv: 1, ku: curves[0].knots.clone(), kv: vec![0.0, 0.0, 1.0, 1.0], cps };
    }
    // Interpolate each column of homogeneous poles (as points in 4D: x, y, z and w separately).
    let q = 3.min(curves.len() - 1);
    let mut cols: Vec<Vec<H>> = Vec::new();
    let mut kv = Vec::new();
    for i in 0..ncp {
        let xyz: Vec<P3> = curves.iter().map(|c| P3::new(c.cps[i].x, c.cps[i].y, c.cps[i].z)).collect();
        let ww: Vec<P3> = curves.iter().map(|c| P3::new(c.cps[i].w, 0.0, 0.0)).collect();
        let a = NurbsCurve::interpolate(&xyz, v, q, None, None);
        let b = NurbsCurve::interpolate(&ww, v, q, None, None);
        kv = a.knots.clone();
        cols.push(a.cps.iter().zip(&b.cps).map(|(p, w)| H::new(p.x, p.y, p.z, w.x)).collect());
    }
    NurbsSurface { du: curves[0].degree, dv: q, ku: curves[0].knots.clone(), kv, cps: cols }
}

/// A derivative field along a section end (first, and second derivatives with curvature).
type Field = (Box<dyn Fn(&P3) -> V3>, Option<Box<dyn Fn(&P3) -> V3>>);

/// Per-section geometry for the end conditions.
struct SecGeo {
    centers: Vec<P3>,
    normals: Vec<V3>,
    radii: Vec<f64>,
}

impl SecGeo {
    fn of(secs: &[Sec], axis: &V3) -> SecGeo {
        let centers: Vec<P3> = secs.iter().map(centroid).collect();
        let normals = secs
            .iter()
            .map(|s| match s {
                Sec::Loop { plane: Some(pl), .. } => {
                    let n = pl.normal.into_inner();
                    if n.dot(axis) >= 0.0 { n } else { -n }
                }
                _ => axis.normalize(),
            })
            .collect();
        let radii = secs
            .iter()
            .zip(&centers)
            .map(|(s, c)| match s {
                Sec::Point(_) => 0.0,
                Sec::Loop { parts, .. } => {
                    let mut acc = 0.0;
                    let mut n = 0.0;
                    for p in parts {
                        for k in 0..16 {
                            acc += (p.c.point(p.t0 + (p.t1 - p.t0) * k as f64 / 16.0) - c).norm();
                            n += 1.0;
                        }
                    }
                    acc / n
                }
            })
            .collect();
        SecGeo { centers, normals, radii }
    }
}

/// The derivative field of an end condition (the OCCT backend's conventions: magnitude × the
/// length through the section centres; a tangent condition flares from the centre, or towards
/// it when the next section is smaller; at the last section the loft arrives).
#[allow(clippy::too_many_arguments)]
fn field(k: &NativeKernel, end: &LoftEnd, sec: &LoftSection, g: &SecGeo, i: usize, toward: usize, total: f64, first: bool) -> Result<Option<Field>> {
    let n = g.normals[i];
    let len = end.magnitude * total;
    let c = g.centers[i];
    let sign = if g.radii[toward] < g.radii[i] { -1.0 } else { 1.0 } * if first { 1.0 } else { -1.0 };
    let radial = move |normal: V3, length: f64| -> Box<dyn Fn(&P3) -> V3> {
        let nn = normal.normalize();
        Box::new(move |p: &P3| {
            let d = p - c;
            let r = d - nn * d.dot(&nn);
            if r.norm() < 1e-12 { V3::zeros() } else { r.normalize() * length }
        })
    };
    Ok(Some(match end.condition {
        LoftCondition::Default => return Ok(None),
        LoftCondition::NormalToProfile => (Box::new(move |_: &P3| n * len), None),
        LoftCondition::TangentToProfile => (radial(n, sign * len), None),
        LoftCondition::NormalDirection(d) | LoftCondition::TangentDirection(d) => {
            let d = V3::new(d[0], d[1], d[2]);
            if d.norm() < 1e-12 || !d.iter().all(|x| x.is_finite()) {
                return Err(invalid("A loft direction needs a non-zero vector"));
            }
            let along = if n.dot(&d).abs() > 1e-9 {
                n
            } else {
                let cc = g.centers[toward] - g.centers[i];
                if first { cc } else { -cc }
            };
            let d = if along.dot(&d) < 0.0 { -d.normalize() } else { d.normalize() };
            if matches!(end.condition, LoftCondition::NormalDirection(_)) {
                (Box::new(move |_: &P3| d * len), None)
            } else {
                (radial(d, sign * len), None)
            }
        }
        LoftCondition::MatchTangent | LoftCondition::MatchCurvature => {
            let LoftSection::Face(fi) = sec else {
                return Err(invalid("Match tangent and Match curvature need a face as the profile (its neighbouring faces)"));
            };
            let b = k.get(fi.body)?.clone();
            let face = fi.face.0 as usize;
            let ef = b.edge_faces();
            let mut nb: Vec<(usize, usize)> = Vec::new();
            for co in b.faces[face].loops.iter().flatten() {
                if let Some(&gf) = ef[co.edge].iter().find(|&&x| x != face) {
                    nb.push((co.edge, gf));
                }
            }
            if nb.is_empty() {
                return Err(invalid("The face has no neighbouring faces to match"));
            }
            let b = std::sync::Arc::new(b);
            let at = {
                let b = b.clone();
                let nb = nb.clone();
                move |p: &P3| -> (V3, usize, (f64, f64)) {
                    let mut best: Option<(f64, usize, usize)> = None;
                    for &(e, gf) in &nb {
                        let ed = &b.edges[e];
                        let t = ed.curve.closest(p, ed.t[0], ed.t[1]);
                        let dd = (ed.point(t) - p).norm();
                        if best.map_or(true, |(bd, ..)| dd < bd) {
                            best = Some((dd, e, gf));
                        }
                    }
                    let (_, e, gf) = best.unwrap();
                    let ed = &b.edges[e];
                    let t = ed.curve.closest(p, ed.t[0], ed.t[1]);
                    let tan = ed.curve.tangent(t).normalize();
                    let f = &b.faces[gf];
                    let (u, v) = f.surf.project(p, None);
                    let nrm = f.surf.normal(u, v) * if f.sense { 1.0 } else { -1.0 };
                    let mut dir = tan.cross(&nrm).normalize();
                    if dir.dot(&n) < 0.0 {
                        dir = -dir;
                    }
                    (dir, gf, (u, v))
                }
            };
            let at = std::sync::Arc::new(at);
            let a1 = at.clone();
            let d1: Box<dyn Fn(&P3) -> V3> = Box::new(move |p: &P3| a1(p).0 * len);
            let d2: Option<Box<dyn Fn(&P3) -> V3>> = if end.condition == LoftCondition::MatchCurvature {
                let a2 = at.clone();
                let b2 = b.clone();
                Some(Box::new(move |p: &P3| {
                    let (dir, gf, (u, v)) = a2(p);
                    let f = &b2.faces[gf];
                    let s = f.surf.d2(u, v);
                    let (su, sv) = (s[1], s[2]);
                    let (e1, f1, g1) = (su.dot(&su), su.dot(&sv), sv.dot(&sv));
                    let nn = su.cross(&sv).normalize();
                    let (l2, m2, n2) = (s[3].dot(&nn), s[4].dot(&nn), s[5].dot(&nn));
                    let Some((du, dv)) = solve2(e1, f1, f1, g1, dir.dot(&su), dir.dot(&sv)) else { return V3::zeros() };
                    let num = l2 * du * du + 2.0 * m2 * du * dv + n2 * dv * dv;
                    let den = e1 * du * du + 2.0 * f1 * du * dv + g1 * dv * dv;
                    let kappa = if den.abs() > 1e-300 { num / den } else { 0.0 };
                    nn * (kappa * len * len)
                }))
            } else {
                None
            };
            (d1, d2)
        }
    }))
}

/// Pole vectors D_i with Σ R_i(u) D_i fitting `f` along the curve (least squares over dense
/// samples; exact when `f` is affine in position).
fn fit_poles(c: &NurbsCurve, f: &dyn Fn(&P3) -> V3) -> Vec<V3> {
    let n = c.cps.len();
    let (a, b) = c.range();
    let m = (n * 6).max(24);
    let mut rows = nalgebra::DMatrix::<f64>::zeros(m, n);
    let mut rhs = nalgebra::DMatrix::<f64>::zeros(m, 3);
    for j in 0..m {
        let u = a + (b - a) * (j as f64 + 0.5) / m as f64;
        // Rational basis values: R_i = N_i w_i / Σ N w.
        let span = super::nurbs::find_span(n - 1, c.degree, u, &c.knots);
        let nd = super::nurbs::basis_ders(span, u, c.degree, 0, &c.knots);
        let mut wsum = 0.0;
        for k in 0..=c.degree {
            wsum += nd[0][k] * c.cps[span - c.degree + k].w;
        }
        for k in 0..=c.degree {
            let i = span - c.degree + k;
            rows[(j, i)] = nd[0][k] * c.cps[i].w / wsum;
        }
        let v = f(&c.point(u));
        rhs[(j, 0)] = v.x;
        rhs[(j, 1)] = v.y;
        rhs[(j, 2)] = v.z;
    }
    let svd = rows.svd(true, true);
    let sol = svd.solve(&rhs, 1e-12).unwrap_or_else(|_| nalgebra::DMatrix::zeros(n, 3));
    (0..n).map(|i| V3::new(sol[(i, 0)], sol[(i, 1)], sol[(i, 2)])).collect()
}

/// Skinning with end derivatives: homogeneous poles interpolated across the sections, with the
/// derivative w·D on the weighted coordinates and none on the weights (so S_v = D exactly at
/// the end sections).
fn skin_conditioned(curves: &[NurbsCurve], v: &[f64], d1: [Option<Vec<V3>>; 2], d2: [Option<Vec<V3>>; 2]) -> NurbsSurface {
    let ncp = curves[0].cps.len();
    let last = curves.len() - 1;
    let mut cols: Vec<Vec<H>> = Vec::new();
    let mut kv = Vec::new();
    let mut dv = 3;
    for i in 0..ncp {
        let xyz: Vec<P3> = curves.iter().map(|c| P3::new(c.cps[i].x, c.cps[i].y, c.cps[i].z)).collect();
        let ww: Vec<P3> = curves.iter().map(|c| P3::new(c.cps[i].w, 0.0, 0.0)).collect();
        let (w0, w1) = (curves[0].cps[i].w, curves[last].cps[i].w);
        let dx = [d1[0].as_ref().map(|d| d[i] * w0), d1[1].as_ref().map(|d| d[i] * w1)];
        let ddx = [d2[0].as_ref().map(|d| d[i] * w0), d2[1].as_ref().map(|d| d[i] * w1)];
        let dw = [d1[0].as_ref().map(|_| V3::zeros()), d1[1].as_ref().map(|_| V3::zeros())];
        let ddw = [d2[0].as_ref().map(|_| V3::zeros()), d2[1].as_ref().map(|_| V3::zeros())];
        let deg = if ddx[0].is_some() || ddx[1].is_some() { 5 } else { 3 };
        let a = NurbsCurve::interpolate_ext(&xyz, v, deg, dx, ddx);
        let b = NurbsCurve::interpolate_ext(&ww, v, deg, dw, ddw);
        kv = a.knots.clone();
        dv = a.degree;
        cols.push(a.cps.iter().zip(&b.cps).map(|(p, w)| H::new(p.x, p.y, p.z, w.x)).collect());
    }
    NurbsSurface { du: curves[0].degree, dv, ku: curves[0].knots.clone(), kv, cps: cols }
}

/// The iso-curve u = const or v = const of a surface whose knots are clamped, as an exact
/// B-spline (a row or a column of poles).
fn iso(s: &NurbsSurface, along_u: bool, at_end: bool) -> NurbsCurve {
    if along_u {
        let j = if at_end { s.cps[0].len() - 1 } else { 0 };
        NurbsCurve { degree: s.du, knots: s.ku.clone(), cps: s.cps.iter().map(|row| row[j]).collect() }
    } else {
        let i = if at_end { s.cps.len() - 1 } else { 0 };
        NurbsCurve { degree: s.dv, knots: s.kv.clone(), cps: s.cps[i].clone() }
    }
}

fn line2(o: P2, d: V2) -> Curve {
    Curve::Line { o: P3::new(o.x, o.y, 0.0), d: V3::new(d.x, d.y, 0.0) }
}

/// Builds the solid (or sheet) from lateral surfaces, one per part (u ∈ [0, 1], v ∈ [0, 1],
/// v = 0 the first section). `caps`: the first and last sections' cap surfaces (None for a
/// point or a sheet). `point_ends`: sections that are points.
fn assemble(surfs: Vec<NurbsSurface>, closed_u: bool, srcs: &[u64], caps: [Option<(Surface, Option<FaceInput>)>; 2], points: [bool; 2], solid: bool, source: u64) -> Result<Built> {
    let n = surfs.len();
    let mut b = Body::default();
    // Corner vertices: bottom and top at each joint (u = 0 of part i).
    let nj = if closed_u { n } else { n + 1 };
    let corner = |j: usize, top: bool| -> P3 {
        let (s, at_end) = if j < n { (&surfs[j], false) } else { (&surfs[n - 1], true) };
        let c = iso(s, false, at_end);
        let (a, bb) = c.range();
        c.point(if top { bb } else { a })
    };
    let mut vb: Vec<usize> = Vec::new();
    let mut vt: Vec<usize> = Vec::new();
    for j in 0..nj {
        let p = corner(j, false);
        vb.push(if points[0] && j > 0 { vb[0] } else { b.add_vertex(p) });
        let q = corner(j, true);
        vt.push(if points[1] && j > 0 { vt[0] } else { b.add_vertex(q) });
    }
    // Rails (v-curves) at the joints.
    let rails: Vec<usize> = (0..nj)
        .map(|j| {
            let (s, at_end) = if j < n { (&surfs[j], false) } else { (&surfs[n - 1], true) };
            let c = iso(s, false, at_end);
            let (a, bb) = c.range();
            b.add_edge(Curve::Nurbs(Arc::new(c)), a, bb, vb[j], vt[j])
        })
        .collect();
    let mut bottom = Vec::new();
    let mut top = Vec::new();
    let mut faces_out: Vec<(usize, u64)> = Vec::new();
    for (i, s) in surfs.iter().enumerate() {
        let j = if closed_u { (i + 1) % n } else { i + 1 };
        let cb = iso(s, true, false);
        let ct = iso(s, true, true);
        let (ua, ub) = cb.range();
        let eb = if points[0] { b.add_degenerate(b.verts[vb[0]].p, ua, ub, vb[0]) } else { b.add_edge(Curve::Nurbs(Arc::new(cb)), ua, ub, vb[i], vb[j]) };
        let et = if points[1] { b.add_degenerate(b.verts[vt[0]].p, ua, ub, vt[0]) } else { b.add_edge(Curve::Nurbs(Arc::new(ct)), ua, ub, vt[i], vt[j]) };
        let ((u0, u1), (v0, v1)) = s.range();
        let lp = vec![
            Coedge { edge: eb, fwd: true, pc: line2(P2::new(0.0, v0), V2::new(1.0, 0.0)) },
            Coedge { edge: rails[j], fwd: true, pc: line2(P2::new(u1, 0.0), V2::new(0.0, 1.0)) },
            Coedge { edge: et, fwd: false, pc: line2(P2::new(0.0, v1), V2::new(1.0, 0.0)) },
            Coedge { edge: rails[i], fwd: false, pc: line2(P2::new(u0, 0.0), V2::new(0.0, 1.0)) },
        ];
        b.faces.push(Face { surf: Surface::Nurbs(Arc::new(s.clone())), sense: true, loops: vec![lp] });
        faces_out.push((b.faces.len() - 1, srcs.get(i).copied().unwrap_or(unsourced_curve(0, i))));
        bottom.push(eb);
        top.push(et);
    }
    fix_senses(&mut b, 0..n);
    let mut origins: Vec<(usize, Origin)> = faces_out.iter().map(|&(f, c)| (f, Origin::ProfileCurve { region: source, curve: c })).collect();
    if solid {
        for (end, cap) in caps.iter().enumerate() {
            let Some((surf, _)) = cap else { continue };
            let edges = if end == 0 { &bottom } else { &top };
            // The sides use the bottom edges forward and the top edges backward; the caps use
            // them the other way: the bottom in reverse order, backward; the top in order, forward.
            let order: Vec<usize> = if end == 0 { edges.iter().rev().copied().collect() } else { edges.clone() };
            let mut lp: Vec<Coedge> = order
                .iter()
                .map(|&e| {
                    let ed = &b.edges[e];
                    let pc = super::pcurve::pcurve(surf, &ed.curve, ed.t[0], ed.t[1], None);
                    Coedge { edge: e, fwd: end == 1, pc }
                })
                .collect();
            // Make the p-curves continuous on periodic caps.
            let mut at: Option<P2> = None;
            for c in lp.iter_mut() {
                let ed = &b.edges[c.edge];
                let (t0, t1) = super::topo::co_range(ed, c.fwd);
                if let Some(a) = at {
                    let q = c.pc.point(t0);
                    let du = if surf.u_periodic() { ((a.x - q.x) / TAU).round() * TAU } else { 0.0 };
                    let dv = if surf.v_periodic() { ((a.y - q.y) / TAU).round() * TAU } else { 0.0 };
                    if du != 0.0 || dv != 0.0 {
                        c.pc = c.pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)));
                    }
                }
                let q = c.pc.point(t1);
                at = Some(P2::new(q.x, q.y));
            }
            b.faces.push(Face { surf: surf.clone(), sense: true, loops: vec![lp] });
            let fi = b.faces.len() - 1;
            fix_senses(&mut b, fi..fi + 1);
            origins.push((fi, if end == 0 { Origin::StartCap { region: source } } else { Origin::EndCap { region: source } }));
        }
    }
    let nf = b.faces.len();
    b.shells.push(Shell { faces: (0..nf).collect(), closed: solid });
    if solid {
        b.solids.push(Solid { shells: vec![0] });
        // Outward: flip everything if the volume came out negative.
        if super::mass::body_mass(&b).volume < 0.0 {
            for f in &mut b.faces {
                Body::reverse_face(f);
            }
        }
        // Each face's sense must agree with the solid: re-fix the lateral faces' orientation
        // against the volume sign is enough for a consistent shell.
    }
    Ok(Built { body: b, origins })
}

/// The cap surface of a section: its plane, or (a face section) the face's own surface.
fn cap_surface(k: &NativeKernel, s: &Sec) -> Option<(Surface, Option<FaceInput>)> {
    match s {
        Sec::Point(_) => None,
        Sec::Loop { plane: Some(pl), face, .. } => Some((Surface::Plane { f: super::build::plane_frame(pl, V3::zeros()) }, *face)),
        Sec::Loop { plane: None, face: Some(fi), .. } => {
            let b = k.get(fi.body).ok()?;
            Some((b.faces[fi.face.0 as usize].surf.clone(), Some(*fi)))
        }
        _ => None,
    }
}

/// Exact skinning; None when the sections don't match.
fn loft_exact(k: &NativeKernel, spec: &LoftSpec, secs: &mut Vec<Sec>, solid: bool) -> Result<Option<Built>> {
    let loops: Vec<usize> = secs.iter().filter_map(|s| if let Sec::Loop { parts, .. } = s { Some(parts.len()) } else { None }).collect();
    if loops.is_empty() || loops.iter().any(|n| *n != loops[0]) {
        return Ok(None);
    }
    let count = loops[0];
    let first = centroid(&secs[0]);
    let last = centroid(secs.last().unwrap());
    let axis = last - first;
    if axis.norm() < 1e-9 {
        return Err(invalid("The loft's profiles lie on top of each other"));
    }
    // The same way round the axis, each started where it lines up with the previous one.
    for s in secs.iter_mut() {
        if let Sec::Loop { parts, .. } = s {
            if turning(parts, &axis) < 0.0 {
                *parts = reverse_parts(parts);
            }
        }
    }
    let mut prev_dir: Option<V3> = None;
    for i in 0..secs.len() {
        let c = centroid(&secs[i]);
        if let Sec::Loop { parts, .. } = &mut secs[i] {
            let perp = |d: V3| d - axis.normalize() * d.dot(&axis.normalize());
            if count == 1 && parts[0].c.is_periodic() && (parts[0].t1 - parts[0].t0) >= TAU - 1e-9 {
                // A whole circle or ellipse: restart it where it lines up.
                if let Some(pd) = prev_dir {
                    let p = &mut parts[0];
                    let target = c + pd;
                    let t = p.c.closest(&target, p.t0 - PI, p.t0 + TAU + PI);
                    let span = p.t1 - p.t0;
                    p.t0 = t;
                    p.t1 = t + span;
                }
            } else if let Some(pd) = prev_dir {
                let best = (0..count)
                    .min_by(|&a, &b| {
                        let cost = |r: usize| (perp(parts[r].start() - c).normalize() - pd.normalize()).norm_squared();
                        cost(a).total_cmp(&cost(b))
                    })
                    .unwrap_or(0);
                parts.rotate_left(best);
            }
            prev_dir = Some(perp(parts[0].start() - c));
        }
    }
    // Compatible curves per part across the sections.
    let n_secs = secs.len();
    let mut srcs = Vec::new();
    let mut surfs = Vec::new();
    // v parameters by the distance between centres.
    let cents: Vec<P3> = secs.iter().map(centroid).collect();
    let mut v = vec![0.0];
    for i in 1..n_secs {
        v.push(v[i - 1] + (cents[i] - cents[i - 1]).norm());
    }
    let total = *v.last().unwrap();
    let v: Vec<f64> = v.iter().map(|x| x / total).collect();
    for pi in 0..count {
        let mut curves: Vec<NurbsCurve> = Vec::new();
        let mut template: Option<NurbsCurve> = None;
        for s in secs.iter() {
            if let Sec::Loop { parts, .. } = s {
                let c = parts[pi].nurbs();
                template.get_or_insert(c.clone());
                curves.push(c);
            } else {
                curves.push(NurbsCurve { degree: 0, knots: vec![], cps: vec![] });
            }
        }
        // Points: the template's shape collapsed onto the point.
        let tpl = template.unwrap();
        for (si, s) in secs.iter().enumerate() {
            if let Sec::Point(p) = s {
                curves[si] = NurbsCurve { degree: tpl.degree, knots: tpl.knots.clone(), cps: tpl.cps.iter().map(|h| hom(p, h.w)).collect() };
            }
        }
        if !compatible(&mut curves) {
            return Ok(None);
        }
        // Points must follow the knot insertions too.
        for (si, s) in secs.iter().enumerate() {
            if let Sec::Point(p) = s {
                let reff = curves.iter().enumerate().find(|(j, _)| matches!(secs[*j], Sec::Loop { .. })).map(|(_, c)| c.clone()).unwrap();
                curves[si] = NurbsCurve { degree: reff.degree, knots: reff.knots.clone(), cps: reff.cps.iter().map(|h| hom(p, h.w)).collect() };
            }
        }
        srcs.push(secs.iter().find_map(|s| if let Sec::Loop { parts, .. } = s { Some(parts[pi].src) } else { None }).unwrap());
        let conditioned = spec.start.condition != LoftCondition::Default || spec.end.condition != LoftCondition::Default;
        if conditioned {
            let geo = SecGeo::of(secs, &axis);
            let d0 = field(k, &spec.start, &spec.sections[0], &geo, 0, 1, total, true)?;
            let d1 = field(k, &spec.end, &spec.sections[n_secs - 1], &geo, n_secs - 1, n_secs - 2, total, false)?;
            let fit = |f: &Option<Field>, c: &NurbsCurve, second: bool| -> Option<Vec<V3>> {
                let f = f.as_ref()?;
                let g: &dyn Fn(&P3) -> V3 = if second { f.1.as_deref()? } else { &*f.0 };
                Some(fit_poles(c, g))
            };
            let (a0, a1) = (fit(&d0, &curves[0], false), fit(&d1, &curves[n_secs - 1], false));
            let (b0, b1) = (fit(&d0, &curves[0], true), fit(&d1, &curves[n_secs - 1], true));
            surfs.push(skin_conditioned(&curves, &v, [a0, a1], [b0, b1]));
        } else {
            surfs.push(skin(&curves, &v));
        }
    }
    let closed_u = true;
    let caps = [cap_surface(k, &secs[0]), cap_surface(k, &secs[n_secs - 1])];
    let points = [matches!(secs[0], Sec::Point(_)), matches!(secs[n_secs - 1], Sec::Point(_))];
    Ok(Some(assemble(surfs, closed_u, &srcs, caps, points, solid, spec.source)?))
}

/// The loft's end derivative at a section (OCCT backend conventions: magnitude × the length
/// through the section centres).
#[allow(clippy::too_many_arguments)]
fn end_derivative(k: &NativeKernel, end: &LoftEnd, sec: &LoftSection, samples: &crate::loft::Samples, i: usize, toward: usize, total: f64, first: bool) -> Result<Option<(Vec<Vec<V3>>, Option<Vec<Vec<V3>>>)>> {
    let n = samples.normals[i];
    let len = end.magnitude * total;
    let c = samples.centers[i];
    let radius = |j: usize| -> f64 {
        let cj = samples.centers[j];
        let pts: Vec<&P3> = samples.patches.iter().flat_map(|p| p[j].iter()).collect();
        pts.iter().map(|p| (*p - cj).norm()).sum::<f64>() / pts.len().max(1) as f64
    };
    let per_sample = |f: &dyn Fn(&P3) -> V3| -> Vec<Vec<V3>> { samples.patches.iter().map(|p| p[i].iter().map(f).collect()).collect() };
    let radial = |normal: V3, length: f64| {
        let nn = normal.normalize();
        move |p: &P3| {
            let d = p - c;
            let r = d - nn * d.dot(&nn);
            if r.norm() < 1e-12 { V3::zeros() } else { r.normalize() * length }
        }
    };
    let sign_toward = if radius(toward) < radius(i) { -1.0 } else { 1.0 };
    // At the last section the loft arrives: a radial derivative flares the other way.
    let end_flip = if first { 1.0 } else { -1.0 };
    Ok(match end.condition {
        LoftCondition::Default => None,
        LoftCondition::NormalToProfile => Some((per_sample(&|_| n * len), None)),
        LoftCondition::TangentToProfile => Some((per_sample(&radial(n, sign_toward * len * end_flip)), None)),
        LoftCondition::NormalDirection(d) | LoftCondition::TangentDirection(d) => {
            let d = V3::new(d[0], d[1], d[2]);
            if d.norm() < 1e-12 || !d.iter().all(|x| x.is_finite()) {
                return Err(invalid("A loft direction needs a non-zero vector"));
            }
            let along = if n.dot(&d).abs() > 1e-9 {
                n
            } else {
                let cc = samples.centers[toward] - samples.centers[i];
                if first { cc } else { -cc }
            };
            let d = if along.dot(&d) < 0.0 { -d.normalize() } else { d.normalize() };
            if matches!(end.condition, LoftCondition::NormalDirection(_)) {
                Some((per_sample(&|_| d * len), None))
            } else {
                Some((per_sample(&radial(d, sign_toward * len * end_flip)), None))
            }
        }
        LoftCondition::MatchTangent | LoftCondition::MatchCurvature => {
            let LoftSection::Face(fi) = sec else {
                return Err(invalid("Match tangent and Match curvature need a face as the profile (its neighbouring faces)"));
            };
            let curvature = end.condition == LoftCondition::MatchCurvature;
            Some(match_derivatives(k, fi, samples, i, first, len, curvature)?)
        }
    })
}

/// Derivatives that continue the faces next to a face section: at each sample, the direction
/// in the neighbour's tangent plane square to the boundary, leaving the first section away
/// from its body (arriving into the last); with curvature, the neighbour's normal curvature in
/// that direction as the second derivative.
fn match_derivatives(k: &NativeKernel, fi: &FaceInput, samples: &crate::loft::Samples, i: usize, first: bool, len: f64, curvature: bool) -> Result<(Vec<Vec<V3>>, Option<Vec<Vec<V3>>>)> {
    let b = k.get(fi.body)?;
    let face = fi.face.0 as usize;
    let ef = b.edge_faces();
    // The neighbouring face across each boundary edge of the face.
    let mut neighbours: Vec<(usize, usize)> = Vec::new(); // (edge, other face)
    for c in b.faces[face].loops.iter().flatten() {
        if let Some(&g) = ef[c.edge].iter().find(|&&g| g != face) {
            neighbours.push((c.edge, g));
        }
    }
    let n_sec = samples.normals[i];
    let mut d1 = Vec::new();
    let mut d2 = Vec::new();
    for patch in &samples.patches {
        let pts = &patch[i];
        let mut r1 = Vec::new();
        let mut r2 = Vec::new();
        for (j, p) in pts.iter().enumerate() {
            // The nearest boundary edge and its neighbour.
            let mut best: Option<(f64, usize, usize)> = None;
            for &(e, g) in &neighbours {
                let ed = &b.edges[e];
                let t = ed.curve.closest(p, ed.t[0], ed.t[1]);
                let d = (ed.point(t) - p).norm();
                if best.map_or(true, |(bd, ..)| d < bd) {
                    best = Some((d, e, g));
                }
            }
            let Some((_, e, g)) = best else { return Err(invalid("The face has no neighbouring faces to match")) };
            let ed = &b.edges[e];
            let t = ed.curve.closest(p, ed.t[0], ed.t[1]);
            let tan = ed.curve.tangent(t).normalize();
            let gf = &b.faces[g];
            let (u, v) = gf.surf.project(p, None);
            let nrm = gf.surf.normal(u, v) * if gf.sense { 1.0 } else { -1.0 };
            let mut dir = tan.cross(&nrm).normalize();
            // Leave the first section along the loft (away from the face's body), arrive into
            // the last.
            if dir.dot(&n_sec) < 0.0 {
                dir = -dir;
            }
            let _ = (first, j);
            r1.push(dir * len);
            if curvature {
                // Normal curvature of the neighbour in direction dir.
                let s = gf.surf.d2(u, v);
                let (su, sv) = (s[1], s[2]);
                let (e1, f1, g1) = (su.dot(&su), su.dot(&sv), sv.dot(&sv));
                let nn = su.cross(&sv).normalize();
                let (l2, m2, n2) = (s[3].dot(&nn), s[4].dot(&nn), s[5].dot(&nn));
                // dir in (du, dv).
                let Some((du, dv)) = solve2(e1, f1, f1, g1, dir.dot(&su), dir.dot(&sv)) else { r2.push(V3::zeros()); continue };
                let num = l2 * du * du + 2.0 * m2 * du * dv + n2 * dv * dv;
                let den = e1 * du * du + 2.0 * f1 * du * dv + g1 * dv * dv;
                let kappa = if den.abs() > 1e-300 { num / den } else { 0.0 };
                r2.push(nn * (kappa * len * len));
            }
        }
        d1.push(r1);
        d2.push(r2);
    }
    Ok((d1, if curvature { Some(d2) } else { None }))
}

/// The interpolating loft through sampled sections (end conditions, unmatched sections).
fn loft_sampled(k: &NativeKernel, spec: &LoftSpec, solid: bool) -> Result<Built> {
    let mut sc = Vec::new();
    for s in &spec.sections {
        match s {
            LoftSection::Profile(p) => sc.push(SectionCurves::new(p.plane, section_loop(p)?)),
            LoftSection::Face(fi) => sc.push(face_section(k, fi)?),
            LoftSection::Point(_) => return Err(invalid("End conditions need profiles at both ends, not points")),
        }
    }
    let samples = sample_sections(&sc)?;
    let n = samples.centers.len();
    let mut v = vec![0.0];
    for i in 1..n {
        v.push(v[i - 1] + (samples.centers[i] - samples.centers[i - 1]).norm());
    }
    let total = v[n - 1];
    if total < 1e-9 {
        return Err(invalid("The loft's profiles lie on top of each other"));
    }
    let vp: Vec<f64> = v.iter().map(|x| x / total).collect();
    let d0 = end_derivative(k, &spec.start, &spec.sections[0], &samples, 0, 1, total, true)?;
    let d1 = end_derivative(k, &spec.end, &spec.sections[n - 1], &samples, n - 1, n - 2, total, false)?;
    let periodic = samples.periodic;
    let mut surfs = Vec::new();
    for (pi, patch) in samples.patches.iter().enumerate() {
        let m = patch[0].len();
        // Parameters along the section (shared by every section).
        let up: Vec<f64> = if periodic { (0..=m).map(|j| j as f64 / m as f64).collect() } else { (0..m).map(|j| j as f64 / (m - 1) as f64).collect() };
        let pts_of = |row: &Vec<P3>| -> Vec<P3> {
            let mut r = row.clone();
            if periodic {
                r.push(row[0]);
            }
            r
        };
        let curves: Vec<NurbsCurve> = patch.iter().map(|row| NurbsCurve::interpolate(&pts_of(row), &up, 3, None, None)).collect();
        // Derivatives of the poles: interpolate the sample derivatives with the same basis.
        let deriv_poles = |ds: &Option<(Vec<Vec<V3>>, Option<Vec<Vec<V3>>>)>, second: bool| -> Option<Vec<V3>> {
            let (a, b) = ds.as_ref()?;
            let rows = if second { b.as_ref()? } else { a };
            let mut row: Vec<P3> = rows[pi].iter().map(|d| P3::from(*d)).collect();
            if periodic {
                row.push(row[0]);
            }
            let c = NurbsCurve::interpolate(&row, &up, 3, None, None);
            Some(c.cps.iter().map(|h| V3::new(h.x, h.y, h.z)).collect())
        };
        let (sd0, sd1) = (deriv_poles(&d0, false), deriv_poles(&d1, false));
        let (ss0, ss1) = (deriv_poles(&d0, true), deriv_poles(&d1, true));
        let ncp = curves[0].cps.len();
        let mut cols: Vec<Vec<H>> = Vec::new();
        let mut kv = Vec::new();
        for i in 0..ncp {
            let col: Vec<P3> = curves.iter().map(|c| dehom(&c.cps[i])).collect();
            let c = NurbsCurve::interpolate_ext(&col, &vp, 3, [sd0.as_ref().map(|d| d[i]), sd1.as_ref().map(|d| d[i])], [ss0.as_ref().map(|d| d[i]), ss1.as_ref().map(|d| d[i])]);
            kv = c.knots.clone();
            cols.push(c.cps);
        }
        surfs.push(NurbsSurface { du: curves[0].degree, dv: 3.min(cols[0].len() - 1), ku: curves[0].knots.clone(), kv, cps: cols });
    }
    let srcs: Vec<u64> = samples.sources.iter().enumerate().map(|(i, s)| s.unwrap_or(unsourced_curve(0, i))).collect();
    // Caps: planar sections' planes; face sections: the face itself.
    let cap = |s: &LoftSection| -> Option<(Surface, Option<FaceInput>)> {
        match s {
            LoftSection::Profile(p) => Some((Surface::Plane { f: super::build::plane_frame(&p.plane, V3::zeros()) }, None)),
            LoftSection::Face(fi) => k.get(fi.body).ok().map(|b| (b.faces[fi.face.0 as usize].surf.clone(), Some(*fi))),
            LoftSection::Point(_) => None,
        }
    };
    assemble(surfs, periodic || true, &srcs, [cap(&spec.sections[0]), cap(&spec.sections[n - 1])], [false, false], solid, spec.source)
}

/// A face's outer loop as a sampled section.
fn face_section(k: &NativeKernel, fi: &FaceInput) -> Result<SectionCurves> {
    let b = k.get(fi.body)?;
    let f = b.faces.get(fi.face.0 as usize).ok_or_else(|| invalid("unknown face"))?;
    let mut polys = Vec::new();
    for c in &f.loops[0] {
        let e = &b.edges[c.edge];
        if e.degenerate {
            continue;
        }
        let n = if matches!(e.curve, Curve::Line { .. }) { 1 } else { 4096 };
        let mut pts: Vec<P3> = (0..=n).map(|i| e.point(e.t[0] + (e.t[1] - e.t[0]) * i as f64 / n as f64)).collect();
        if !c.fwd {
            pts.reverse();
        }
        polys.push((pts, Some(c.edge as u64)));
    }
    // The plane: the face's, or the loop's best fit.
    let pts: Vec<P3> = polys.iter().flat_map(|(p, _)| p.iter().copied()).collect();
    let c = pts.iter().fold(V3::zeros(), |a, p| a + p.coords) / pts.len() as f64;
    let mut nn = V3::zeros();
    for i in 0..pts.len() {
        nn += (pts[i].coords - c).cross(&(pts[(i + 1) % pts.len()].coords - c));
    }
    let nn = if nn.norm() > 1e-12 { nn.normalize() } else { V3::z() };
    let fr = Frame::from_z(P3::from(c), nn);
    let plane = Plane { origin: P3::from(c), x_dir: nalgebra::Unit::new_normalize(fr.x), normal: nalgebra::Unit::new_normalize(nn) };
    Ok(SectionCurves::sampled(plane, polys))
}

pub fn loft_with(k: &mut NativeKernel, spec: &LoftSpec) -> Result<OpResult> {
    if spec.sections.len() < 2 {
        return Err(invalid("Select at least two profiles to loft"));
    }
    for (i, s) in spec.sections.iter().enumerate() {
        if matches!(s, LoftSection::Point(_)) && i != 0 && i + 1 != spec.sections.len() {
            return Err(invalid("A point can only be the first or last loft profile"));
        }
    }
    for e in [&spec.start, &spec.end] {
        if e.condition != LoftCondition::Default && !(e.magnitude.is_finite() && e.magnitude > 0.0) {
            return Err(invalid("A loft end's magnitude must be greater than zero"));
        }
    }
    if let BodyKind::Thin { left, right } = spec.body {
        return loft_thin(k, spec, left, right);
    }
    let solid = spec.body == BodyKind::Solid;
    let conditioned = spec.start.condition != LoftCondition::Default || spec.end.condition != LoftCondition::Default;
    let mut secs: Vec<Sec> = spec.sections.iter().map(|s| section(k, s)).collect::<Result<_>>()?;
    if conditioned {
        for (end, sec) in [(&spec.start, spec.sections.first()), (&spec.end, spec.sections.last())] {
            let matching = matches!(end.condition, LoftCondition::MatchTangent | LoftCondition::MatchCurvature);
            if matching && !matches!(sec, Some(LoftSection::Face(_))) {
                return Err(invalid("Match tangent and Match curvature need a face as the profile (its neighbouring faces)"));
            }
            if end.condition != LoftCondition::Default && matches!(sec, Some(LoftSection::Point(_))) {
                return Err(invalid("End conditions need profiles at both ends, not points"));
            }
        }
    }
    let built = match loft_exact(k, spec, &mut secs, solid)? {
        Some(b) => b,
        None => loft_sampled(k, spec, solid)?,
    };
    let mut t = Tagged::from_built(built);
    // A face section that isn't flat is the loft's own cap; a flat face section's cap lies on the
    // face (both work the same here).
    let _ = &mut t;
    Ok(t.into_result(k))
}

/// A thin loft: the loft of the sections' outer walls less that of their inner walls.
fn loft_thin(k: &mut NativeKernel, spec: &LoftSpec, left: f64, right: f64) -> Result<OpResult> {
    let mut outer = Vec::new();
    let mut inner = Vec::new();
    for s in &spec.sections {
        let LoftSection::Profile(p) = s else { return Err(KernelError::Unsupported("thin lofts of faces or points")) };
        let bands = crate::thin::thin_profile(p, left, right)?;
        let regions = super::profile::regions(&bands)?;
        let mf = super::profile::merge_regions(&regions)?;
        if mf.len() != 1 || mf[0].holes.len() != 1 {
            return Err(KernelError::Unsupported("thin lofts of open or branching profiles"));
        }
        let to_profile = |l: &[super::profile::Piece]| -> Result<Profile> {
            // Back to sketch curves for the section (exact circles and lines).
            let curves: Vec<Curve2> = l.iter().map(piece_curve2).collect::<Option<Vec<_>>>().ok_or_else(|| KernelError::Unsupported("thin lofts of these curves"))?;
            Ok(Profile::new(p.plane, vec![Region { outer: Loop { curves }, holes: vec![], source: None }]))
        };
        outer.push(LoftSection::Profile(to_profile(&mf[0].outer)?));
        inner.push(LoftSection::Profile(to_profile(&mf[0].holes[0])?));
    }
    let a = loft_with(k, &LoftSpec { body: BodyKind::Solid, sections: outer, ..spec.clone() })?;
    let b = loft_with(k, &LoftSpec { body: BodyKind::Solid, sections: inner, ..spec.clone() })?;
    let r = k.boolean(BoolOp::Subtract, a.bodies[0], &[b.bodies[0]])?;
    k.release(a.bodies[0]);
    k.release(b.bodies[0]);
    Ok(r)
}

fn piece_curve2(p: &super::profile::Piece) -> Option<Curve2> {
    let pt = |q: P3| nalgebra::Point2::new(q.x, q.y);
    match &p.c {
        Curve::Line { .. } => Some(Curve2::Line { a: p.start(), b: p.end(), source: None }),
        Curve::Circle { c, x, y, r } => {
            // Angles in the sketch frame.
            let ang = |t: f64| {
                let q = p.c.point(t) - c;
                q.y.atan2(q.x)
            };
            let _ = (x, y);
            let full = (p.t1 - p.t0) >= TAU - 1e-9;
            if full {
                return Some(Curve2::Circle { center: pt(*c), radius: *r, source: None });
            }
            let start = ang(if p.rev { p.t1 } else { p.t0 });
            let sweep = (p.t1 - p.t0) * if p.rev { -1.0 } else { 1.0 } * x.cross(y).z.signum();
            Some(Curve2::Arc { center: pt(*c), radius: *r, start_angle: start, sweep, source: None })
        }
        _ => None,
    }
}
