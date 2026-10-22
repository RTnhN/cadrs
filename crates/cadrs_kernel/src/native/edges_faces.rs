//! Planar faces from loose edges (OCCT's `BOPAlgo_Tools::EdgesToWires` and `WiresToFaces`,
//! behind DRAW's `edgestofaces`): every plane the edges span (a circle's, or two crossing
//! lines') gets the faces its edges enclose. In each plane the edges are split where they meet,
//! hanging edges are dropped, the pieces' arrangement is walked into its bounded regions (each
//! keeping the region on its left), and each connected group's outer boundary becomes a hole of
//! the smallest region of another group around it.
//!
//! Lines and circles only.

use super::curve::Curve;
use super::math::*;
use crate::{KernelError, Plane, Result};
use std::f64::consts::TAU;
use super::tol;

/// A piece in a plane's (x, y): a segment or an arc (centre, radius, start angle, signed sweep).
#[derive(Debug, Clone, Copy)]
enum Piece2 {
    Seg(P2, P2),
    Arc(P2, f64, f64, f64),
}

impl Piece2 {
    fn at(&self, s: f64) -> P2 {
        match *self {
            Piece2::Seg(a, b) => a + (b - a) * s,
            Piece2::Arc(c, r, a0, sw) => {
                let t = a0 + sw * s;
                P2::new(c.x + r * t.cos(), c.y + r * t.sin())
            }
        }
    }
    fn reversed(&self) -> Piece2 {
        match *self {
            Piece2::Seg(a, b) => Piece2::Seg(b, a),
            Piece2::Arc(c, r, a0, sw) => Piece2::Arc(c, r, a0 + sw, -sw),
        }
    }
    /// The direction it leaves its start in.
    fn out_dir(&self) -> V2 {
        match *self {
            Piece2::Seg(a, b) => (b - a).normalize(),
            Piece2::Arc(_, _, a0, sw) => V2::new(-a0.sin(), a0.cos()) * sw.signum(),
        }
    }
    /// The signed curvature leaving the start (left turns positive).
    fn turn(&self) -> f64 {
        match *self {
            Piece2::Seg(..) => 0.0,
            Piece2::Arc(_, r, _, sw) => sw.signum() / r,
        }
    }
}

/// The circle through a plane, or None when the curve is a line.
fn as_circle(c: &Curve) -> Option<(P3, V3, V3, f64)> {
    match c {
        Curve::Circle { c, x, y, r } => Some((*c, *x, *y, *r)),
        _ => None,
    }
}

/// An edge: a curve between two parameters.
pub type Edge3 = (Curve, f64, f64);

/// A planar face: its plane and its loops (outer first).
pub type PlaneFace = (Plane, Vec<Vec<Edge3>>);

/// The faces the edges (lines and circles) enclose.
pub fn edges_to_faces(edges: &[Edge3]) -> Result<Vec<PlaneFace>> {
    for (c, _, _) in edges {
        if !matches!(c, Curve::Line { .. } | Curve::Circle { .. }) {
            return Err(KernelError::Unsupported("faces from edges other than lines and circles"));
        }
    }
    let mut bb = Bbox::empty();
    for (c, t0, t1) in edges {
        for i in 0..=8 {
            bb.add(&c.point(t0 + (t1 - t0) * i as f64 / 8.0));
        }
    }
    let size = bb.diagonal().max(tol::ROUNDOFF);
    let tol = 1e-7 * size;
    // The planes: circles', and those of two lines that meet at an angle.
    let mut planes: Vec<(P3, V3)> = Vec::new();
    let mut add_plane = |o: P3, n: V3| {
        let n = n.normalize();
        if !planes.iter().any(|(po, pn)| pn.cross(&n).norm() < tol::ANGULAR && (o - po).dot(pn).abs() < tol) {
            planes.push((o, n));
        }
    };
    for (c, _, _) in edges {
        if let Some((o, x, y, _)) = as_circle(c) {
            add_plane(o, x.cross(&y));
        }
    }
    for i in 0..edges.len() {
        for j in i + 1..edges.len() {
            let (Curve::Line { o: oa, d: da }, Curve::Line { o: ob, d: db }) = (&edges[i].0, &edges[j].0) else { continue };
            let n = da.cross(db);
            if n.norm() < tol::REL * da.norm() * db.norm() {
                continue;
            }
            if (ob - oa).dot(&n.normalize()).abs() < tol {
                add_plane(*oa, n);
            }
        }
    }
    let mut out = Vec::new();
    for (o, n) in planes {
        let xd = super::axes::ax2_x(&n);
        let yd = n.cross(&xd);
        let to2 = |p: &P3| P2::new((p - o).dot(&xd), (p - o).dot(&yd));
        let to3 = |q: &P2| o + xd * q.x + yd * q.y;
        let on = |p: &P3| (p - o).dot(&n).abs() < tol;
        // The edges in the plane, as 2D curves (circles as whole circles, arcs as ranges).
        let mut curves: Vec<Piece2> = Vec::new();
        for (c, t0, t1) in edges {
            match as_circle(c) {
                Some((cc, x, y, r)) => {
                    if x.cross(&y).normalize().cross(&n).norm() > tol::ANGULAR || !on(&cc) {
                        continue;
                    }
                    let c2 = to2(&cc);
                    let sgn = if x.cross(&y).dot(&n) > 0.0 { 1.0 } else { -1.0 };
                    let p0 = to2(&c.point(*t0)) - c2;
                    curves.push(Piece2::Arc(c2, r, p0.y.atan2(p0.x), (t1 - t0) * sgn));
                }
                None => {
                    let (a, b) = (c.point(*t0), c.point(*t1));
                    if on(&a) && on(&b) && (b - a).norm() > tol {
                        curves.push(Piece2::Seg(to2(&a), to2(&b)));
                    }
                }
            }
        }
        if curves.is_empty() {
            continue;
        }
        for f in faces_in_plane(&curves, tol) {
            let loops = f
                .iter()
                .map(|l| {
                    l.iter()
                        .map(|p| match *p {
                            Piece2::Seg(a, b) => {
                                let (c, len) = Curve::line(to3(&a), to3(&b));
                                (c, 0.0, len)
                            }
                            Piece2::Arc(c, r, a0, sw) => {
                                let (x, y) = (xd * a0.cos() + yd * a0.sin(), (n.cross(&(xd * a0.cos() + yd * a0.sin()))) * sw.signum());
                                (Curve::Circle { c: to3(&c), x, y, r }, 0.0, sw.abs())
                            }
                        })
                        .collect()
                })
                .collect();
            out.push((Plane { origin: o, x_dir: nalgebra::Unit::new_normalize(xd), normal: nalgebra::Unit::new_normalize(n) }, loops));
        }
    }
    Ok(out)
}

/// The parameters (fractions) where `a` meets `b`.
fn meets(a: &Piece2, b: &Piece2, tol: f64) -> Vec<f64> {
    let mut out = Vec::new();
    // Where b's ends and crossings lie on a.
    let mut pts: Vec<P2> = vec![b.at(0.0), b.at(1.0)];
    match (*a, *b) {
        (Piece2::Seg(p, q), Piece2::Seg(r, s)) => {
            let (d1, d2) = (q - p, s - r);
            let den = d1.x * d2.y - d1.y * d2.x;
            if den.abs() > 1e-14 * d1.norm() * d2.norm() {
                let w = r - p;
                let t = (w.x * d2.y - w.y * d2.x) / den;
                pts.push(p + d1 * t);
            }
        }
        (Piece2::Seg(p, q), Piece2::Arc(c, rr, ..)) | (Piece2::Arc(c, rr, ..), Piece2::Seg(p, q)) => {
            let d = (q - p).normalize();
            let w = p - c;
            let bq = w.dot(&d);
            let disc = bq * bq - (w.dot(&w) - rr * rr);
            if disc >= -tol * rr {
                let sq = disc.max(0.0).sqrt();
                pts.push(p + d * (-bq - sq));
                pts.push(p + d * (-bq + sq));
            }
        }
        (Piece2::Arc(c1, r1, ..), Piece2::Arc(c2, r2, ..)) => {
            let dv = c2 - c1;
            let dd = dv.norm();
            if dd > 1e-14 && dd <= r1 + r2 + tol && dd >= (r1 - r2).abs() - tol {
                let aa = (r1 * r1 - r2 * r2 + dd * dd) / (2.0 * dd);
                let h = (r1 * r1 - aa * aa).max(0.0).sqrt();
                let m = c1 + dv * (aa / dd);
                let perp = V2::new(-dv.y, dv.x) / dd;
                pts.push(m + perp * h);
                pts.push(m - perp * h);
            }
        }
    }
    for p in pts {
        if b_dist(b, &p) > tol {
            continue;
        }
        if let Some(s) = frac_on(a, &p, tol) {
            out.push(s);
        }
    }
    out
}

fn b_dist(b: &Piece2, p: &P2) -> f64 {
    match frac_on(b, p, f64::INFINITY) {
        Some(s) => (b.at(s) - p).norm(),
        None => f64::INFINITY,
    }
}

/// The fraction of `a` at `p` when `p` lies on it (within `tol`).
fn frac_on(a: &Piece2, p: &P2, tol: f64) -> Option<f64> {
    let s = match *a {
        Piece2::Seg(x, y) => {
            let d = y - x;
            ((p - x).dot(&d) / d.dot(&d)).clamp(0.0, 1.0)
        }
        Piece2::Arc(c, _, a0, sw) => {
            let w = p - c;
            let ang = w.y.atan2(w.x);
            let mut rel = (ang - a0) * sw.signum();
            rel = rel.rem_euclid(TAU);
            if sw.abs() >= TAU - 1e-12 {
                rel / TAU
            } else {
                let s = rel / sw.abs();
                if s > 1.0 {
                    // Past the end: the nearer end.
                    if (rel - sw.abs()) < (TAU - rel) { 1.0 } else { 0.0 }
                } else {
                    s
                }
            }
        }
    };
    ((a.at(s) - p).norm() <= tol).then_some(s)
}

fn sub(a: &Piece2, s0: f64, s1: f64) -> Piece2 {
    match *a {
        Piece2::Seg(..) => Piece2::Seg(a.at(s0), a.at(s1)),
        Piece2::Arc(c, r, a0, sw) => Piece2::Arc(c, r, a0 + sw * s0, sw * (s1 - s0)),
    }
}

/// The bounded regions of an arrangement of curves in a plane: each region's loops, outer first.
fn faces_in_plane(curves: &[Piece2], tol: f64) -> Vec<Vec<Vec<Piece2>>> {
    // Split every curve where the others meet it.
    let mut pieces: Vec<Piece2> = Vec::new();
    for (i, a) in curves.iter().enumerate() {
        let mut ss: Vec<f64> = vec![0.0, 1.0];
        if let Piece2::Arc(_, _, _, sw) = a
            && sw.abs() >= TAU - 1e-12
        {
            ss.push(0.5);
        }
        for (j, b) in curves.iter().enumerate() {
            if i != j {
                ss.extend(meets(a, b, tol));
            }
        }
        ss.sort_by(|x, y| x.total_cmp(y));
        let len = |s0: f64, s1: f64| (a.at(s1) - a.at(s0)).norm() + (a.at(0.5 * (s0 + s1)) - a.at(s0)).norm();
        let mut keep = vec![ss[0]];
        for &s in &ss[1..] {
            if len(*keep.last().unwrap_or(&0.0), s) > 4.0 * tol {
                keep.push(s);
            }
        }
        if let Some(l) = keep.last_mut() {
            *l = 1.0;
        }
        for w in keep.windows(2) {
            pieces.push(sub(a, w[0], w[1]));
        }
    }
    // Vertices.
    let mut verts: Vec<P2> = Vec::new();
    let vid = |p: P2, verts: &mut Vec<P2>| -> usize {
        match verts.iter().position(|q| (q - p).norm() <= 10.0 * tol) {
            Some(i) => i,
            None => {
                verts.push(p);
                verts.len() - 1
            }
        }
    };
    let mut edges: Vec<(usize, usize, Piece2)> = Vec::new();
    for p in pieces {
        let (a, b) = (vid(p.at(0.0), &mut verts), vid(p.at(1.0), &mut verts));
        let mid = p.at(0.5);
        // The same piece twice (shared or overlapping edges): once.
        if edges.iter().any(|(x, y, q)| ((*x == a && *y == b) || (*x == b && *y == a)) && (q.at(0.5) - mid).norm() <= 10.0 * tol) {
            continue;
        }
        if a == b && matches!(p, Piece2::Seg(..)) {
            continue;
        }
        edges.push((a, b, p));
    }
    // Hanging edges go.
    loop {
        let mut deg = vec![0usize; verts.len()];
        for (a, b, _) in &edges {
            deg[*a] += 1;
            deg[*b] += 1;
        }
        let before = edges.len();
        edges.retain(|(a, b, _)| deg[*a] > 1 && deg[*b] > 1);
        if edges.len() == before {
            break;
        }
    }
    // Half-edges h = 2e (forwards) and 2e + 1, sorted about their start vertices.
    let half = |h: usize| -> (usize, usize, Piece2) {
        let (a, b, p) = edges[h / 2];
        if h.is_multiple_of(2) { (a, b, p) } else { (b, a, p.reversed()) }
    };
    let nh = 2 * edges.len();
    let mut around: Vec<Vec<usize>> = vec![Vec::new(); verts.len()];
    for h in 0..nh {
        around[half(h).0].push(h);
    }
    let key = |h: usize| {
        let p = half(h).2;
        let d = p.out_dir();
        (d.y.atan2(d.x), p.turn())
    };
    for v in around.iter_mut() {
        v.sort_by(|&x, &y| {
            let (ax, tx) = key(x);
            let (ay, ty) = key(y);
            if (ax - ay).abs() > tol::ROUNDOFF { ax.total_cmp(&ay) } else { tx.total_cmp(&ty) }
        });
    }
    // The next half-edge round a region: at the end, the one just clockwise of the way back.
    let next = |h: usize| -> usize {
        let twin = h ^ 1;
        let v = half(h).1;
        let list = &around[v];
        let k = list.iter().position(|&x| x == twin).unwrap_or(0);
        list[(k + list.len() - 1) % list.len()]
    };
    let mut seen = vec![false; nh];
    let mut cycles: Vec<(Vec<usize>, f64)> = Vec::new();
    for h0 in 0..nh {
        if seen[h0] {
            continue;
        }
        let mut cyc = Vec::new();
        let mut h = h0;
        while !seen[h] {
            seen[h] = true;
            cyc.push(h);
            h = next(h);
        }
        let area: f64 = cyc
            .iter()
            .map(|&h| {
                let p = half(h).2;
                (0..16).map(|i| {
                    let (a, b) = (p.at(i as f64 / 16.0), p.at((i + 1) as f64 / 16.0));
                    a.x * b.y - b.x * a.y
                }).sum::<f64>()
                    + match p {
                        // The arc's segments beyond their chords.
                        Piece2::Arc(_, r, _, sw) => {
                            let d = sw / 16.0;
                            16.0 * r * r * (d - d.sin())
                        }
                        Piece2::Seg(..) => 0.0,
                    }
            })
            .sum::<f64>()
            * 0.5;
        cycles.push((cyc, area));
    }
    let poly = |cyc: &[usize]| -> Vec<P2> { cyc.iter().flat_map(|&h| (0..16).map(move |i| half(h).2.at(i as f64 / 16.0))).collect() };
    let inside = |pt: &P2, poly: &[P2]| {
        let mut c = false;
        for i in 0..poly.len() {
            let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
            if (a.y > pt.y) != (b.y > pt.y) && pt.x < a.x + (pt.y - a.y) * (b.x - a.x) / (b.y - a.y) {
                c = !c;
            }
        }
        c
    };
    let regions: Vec<usize> = (0..cycles.len()).filter(|&i| cycles[i].1 > tol * tol).collect();
    let mut faces: Vec<Vec<Vec<Piece2>>> = regions.iter().map(|&i| vec![cycles[i].0.iter().map(|&h| half(h).2).collect()]).collect();
    // Outer boundaries (clockwise) are holes of the smallest region around them.
    for (ci, (cyc, area)) in cycles.iter().enumerate() {
        if *area > -tol * tol || regions.contains(&ci) {
            continue;
        }
        let pt = half(cyc[0]).2.at(0.5);
        let own: Vec<usize> = cyc.iter().map(|h| h / 2).collect();
        let host = regions
            .iter()
            .enumerate()
            .filter(|&(_, &r)| !cycles[r].0.iter().any(|h| own.contains(&(h / 2))) && inside(&pt, &poly(&cycles[r].0)))
            .min_by(|a, b| cycles[*a.1].1.total_cmp(&cycles[*b.1].1));
        if let Some((k, _)) = host {
            faces[k].push(cyc.iter().map(|&h| half(h).2).collect());
        }
    }
    faces
}
