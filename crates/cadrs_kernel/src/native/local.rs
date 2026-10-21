//! Local operations that keep a body's topology and change its faces' surfaces: sharp
//! offsets (faces moved along their normals), drafts (faces turned about a neutral plane) and
//! the simple inward shell (the body less its inward offset). Vertices are recomputed as the
//! points where their faces' new surfaces meet, edges as the new surfaces' intersection curves
//! between those points (a seam as the new surface's iso-line), p-curves from the new edges.

use super::curve::Curve;
use super::kernel::NativeKernel;
use super::math::*;
use super::nurbs::NurbsCurve;
use super::surface::Surface;
use super::topo::{Body, Coedge};
use crate::*;

/// The faces around each vertex (distinct).
fn vertex_faces(b: &Body) -> Vec<Vec<usize>> {
    let mut out = vec![Vec::new(); b.verts.len()];
    for (fi, f) in b.faces.iter().enumerate() {
        for c in f.loops.iter().flatten() {
            for v in b.edges[c.edge].v {
                if !out[v].contains(&fi) {
                    out[v].push(fi);
                }
            }
        }
    }
    out
}

/// A point on all the surfaces near `p` (Newton on their signed distances, least squares when
/// more than three meet).
fn meet(surfs: &[&Surface], p: P3) -> Option<P3> {
    let mut x = p;
    for _ in 0..60 {
        let mut jt = nalgebra::DMatrix::<f64>::zeros(surfs.len(), 3);
        let mut r = nalgebra::DVector::<f64>::zeros(surfs.len());
        for (i, s) in surfs.iter().enumerate() {
            let d = super::ssi::sdist(s, &x);
            let e = 1e-6;
            let g = V3::new(
                (super::ssi::sdist(s, &(x + V3::x() * e)) - super::ssi::sdist(s, &(x - V3::x() * e))) / (2.0 * e),
                (super::ssi::sdist(s, &(x + V3::y() * e)) - super::ssi::sdist(s, &(x - V3::y() * e))) / (2.0 * e),
                (super::ssi::sdist(s, &(x + V3::z() * e)) - super::ssi::sdist(s, &(x - V3::z() * e))) / (2.0 * e),
            );
            jt[(i, 0)] = g.x;
            jt[(i, 1)] = g.y;
            jt[(i, 2)] = g.z;
            r[i] = -d;
        }
        // Minimum-norm least-squares step.
        let svd = jt.clone().svd(true, true);
        let dx = svd.solve(&r, 1e-12).ok()?;
        let step = V3::new(dx[0], dx[1], dx[2]);
        x += step;
        if step.norm() < 1e-14 * (1.0 + x.coords.norm()) {
            break;
        }
    }
    let worst = surfs.iter().map(|s| s.distance(&x)).fold(0.0, f64::max);
    (worst < 1e-7).then_some(x)
}

/// The straight edges whose two faces meet tangentially and still do on their new surfaces,
/// with their new lines. Intersecting tangent surfaces is ill-conditioned, but the line is
/// known: each of its points moved along the faces' common normal onto the new surfaces (a
/// drafted or offset tangent pair moves together).
fn tangent_lines(b: &Body, surfs: &[Surface], ef: &[Vec<usize>]) -> Vec<(usize, Curve)> {
    let normal = |s: &Surface, sense: bool, p: &P3| {
        let (u, v) = s.project(p, None);
        s.normal(u, v) * if sense { 1.0 } else { -1.0 }
    };
    let mut out = Vec::new();
    for (e, ed) in b.edges.iter().enumerate() {
        let fs = &ef[e];
        let Curve::Line { d, .. } = ed.curve else { continue };
        if ed.degenerate || fs.len() != 2 || fs[0] == fs[1] || fs.iter().all(|&f| surfs[f] == b.faces[f].surf) {
            continue;
        }
        let (fa, fb) = (&b.faces[fs[0]], &b.faces[fs[1]]);
        let len = (ed.t[1] - ed.t[0]).abs();
        let ts = [ed.t[0], ed.t[0] + (ed.t[1] - ed.t[0]) * 0.5, ed.t[1]];
        let mut moved = Vec::new();
        for &t in &ts {
            let q = ed.point(t);
            let (na, nb) = (normal(&fa.surf, fa.sense, &q), normal(&fb.surf, fb.sense, &q));
            if na.dot(&nb) < 1.0 - 1e-9 {
                break;
            }
            // Along the normal onto the first new surface (Newton on the signed distance).
            let mut s = 0.0;
            for _ in 0..30 {
                let f = super::ssi::sdist(&surfs[fs[0]], &(q + na * s));
                let h = 1e-6 * (1.0 + len);
                let g = (super::ssi::sdist(&surfs[fs[0]], &(q + na * (s + h))) - super::ssi::sdist(&surfs[fs[0]], &(q + na * (s - h)))) / (2.0 * h);
                if g.abs() < 1e-12 {
                    break;
                }
                let ds = f / g;
                s -= ds;
                if ds.abs() < 1e-15 * (1.0 + len) {
                    break;
                }
            }
            moved.push(q + na * s);
        }
        if moved.len() != 3 {
            continue;
        }
        let Some(dir) = (moved[2] - moved[0]).try_normalize(1e-12) else { continue };
        let dir = if dir.dot(&d) < 0.0 { -dir } else { dir };
        let line = Curve::Line { o: moved[0], d: dir };
        // Still a line on both new surfaces, and still tangent there.
        let ok = moved.iter().all(|p| {
            let off = (p - moved[0]) - dir * (p - moved[0]).dot(&dir);
            off.norm() < 1e-9 * (1.0 + len)
                && fs.iter().all(|&f| surfs[f].distance(p) < 1e-9 * (1.0 + len))
                && normal(&surfs[fs[0]], true, p).cross(&normal(&surfs[fs[1]], true, p)).norm() < 1e-7
        });
        if ok {
            out.push((e, line));
        }
    }
    out
}

/// The point on the curve `c` near `p` where it crosses the surfaces (Gauss-Newton on the
/// curve's parameter).
fn line_meet(c: &Curve, surfs: &[&Surface], p: P3) -> Option<P3> {
    if surfs.is_empty() {
        return None;
    }
    let mut t = c.closest(&p, -1e12, 1e12);
    for _ in 0..50 {
        let (mut num, mut den) = (0.0, 0.0);
        for s in surfs {
            let h = 1e-6 * (1.0 + p.coords.norm());
            let d = super::ssi::sdist(s, &c.point(t));
            let g = (super::ssi::sdist(s, &c.point(t + h)) - super::ssi::sdist(s, &c.point(t - h))) / (2.0 * h);
            num += d * g;
            den += g * g;
        }
        if den < 1e-24 {
            return None;
        }
        let dt = num / den;
        t -= dt;
        if dt.abs() < 1e-15 * (1.0 + t.abs()) {
            break;
        }
    }
    let q = c.point(t);
    surfs.iter().all(|s| s.distance(&q) < 1e-7).then_some(q)
}

/// On a B-spline surface closed in u or v (not periodic: its p-curves stay in its range), the
/// p-curve `pc` over [t0, t1] moved across the range when its start is a whole range from `at`
/// (the previous coedge's end) and stays in the range moved: a seam's other use.
fn closed_spline_shift(s: &Surface, pc: &Curve, t0: f64, t1: f64, at: [f64; 2]) -> Option<Curve> {
    let Surface::Nurbs(n) = s else { return None };
    let ((u0, u1), (v0, v1)) = n.range();
    let (q, r) = (pc.point(t0), pc.point(t1));
    let (q, r) = ([q.x, q.y], [r.x, r.y]);
    let mut shift = [0.0; 2];
    for (k, (a, b), closed) in [(0, (u0, u1), (0..=4).all(|j| (s.eval(u0, v0 + (v1 - v0) * j as f64 / 4.0) - s.eval(u1, v0 + (v1 - v0) * j as f64 / 4.0)).norm() < 1e-9)), (1, (v0, v1), (0..=4).all(|i| (s.eval(u0 + (u1 - u0) * i as f64 / 4.0, v0) - s.eval(u0 + (u1 - u0) * i as f64 / 4.0, v1)).norm() < 1e-9))] {
        let w = b - a;
        if !closed || w <= 0.0 {
            continue;
        }
        let m = ((at[k] - q[k]) / w).round();
        let tol = 1e-9 * w;
        let inside = |x: f64| x >= a - tol && x <= b + tol;
        if m != 0.0 && inside(q[k] + m * w) && inside(r[k] + m * w) {
            shift[k] = m * w;
        }
    }
    (shift != [0.0; 2]).then(|| pc.transformed(&Affine::translation(V3::new(shift[0], shift[1], 0.0))))
}

/// The exact iso curve of a B-spline surface: u = `t` (running along v) when `along_v`, else
/// v = `t` (along u). Its poles are the surface's rows blended by the other direction's basis
/// at `t`, in homogeneous coordinates.
fn nurbs_iso(n: &super::nurbs::NurbsSurface, t: f64, along_v: bool) -> NurbsCurve {
    let (deg, knots, count) = if along_v { (n.du, &n.ku, n.cps.len()) } else { (n.dv, &n.kv, n.cps[0].len()) };
    let span = super::nurbs::find_span(count - 1, deg, t, knots);
    let basis = &super::nurbs::basis_ders(span, t, deg, 0, knots)[0];
    if along_v {
        let cps = (0..n.cps[0].len()).map(|j| (0..=deg).map(|k| n.cps[span - deg + k][j] * basis[k]).sum()).collect();
        NurbsCurve { degree: n.dv, knots: n.kv.clone(), cps }
    } else {
        let cps = (0..n.cps.len()).map(|i| (0..=deg).map(|k| n.cps[i][span - deg + k] * basis[k]).sum()).collect();
        NurbsCurve { degree: n.du, knots: n.ku.clone(), cps }
    }
}

/// The points where a surface collapses: a cone's apex, a sphere's poles.
fn singular_points(s: &Surface) -> Vec<P3> {
    match s {
        Surface::Cone { f, r, a } if a.sin().abs() > 1e-12 => vec![f.o + f.z * (-r / a.tan())],
        Surface::Sphere { f, r } => vec![f.o + f.z * *r, f.o - f.z * *r],
        _ => vec![],
    }
}

/// A part of a surface's parameters: `((u0, u1), (v0, v1))`.
pub type UvBox = ((f64, f64), (f64, f64));

/// Rebuilds the body's geometry with new surfaces for its faces (same topology).
pub fn replace_surfaces(b: &Body, surfs: &[Surface]) -> Result<Body> {
    replace_surfaces_in(b, surfs, &[])
}

/// A straight edge between two surfaces swept along its direction (extrusions, cylinders and
/// planes along it) stays a line along it: the one through the new end (exact; marching along
/// two parallel sweeps finds nothing to follow when their profiles cross at a point).
fn swept_line(surfs: &[Surface], fs: &[usize], old: &Curve, p0: &P3, p1: &P3) -> Option<Curve> {
    let Curve::Line { d, .. } = old else { return None };
    let d = d.try_normalize(1e-12)?;
    let along = |s: &Surface| match s {
        Surface::Extrusion { d: e, .. } => e.normalize().cross(&d).norm() < 1e-12,
        Surface::Cylinder { f, .. } => f.z.cross(&d).norm() < 1e-12,
        Surface::Plane { f } => f.x.cross(&f.y).dot(&d).abs() < 1e-12,
        _ => false,
    };
    if fs.len() != 2 || fs[0] == fs[1] || !fs.iter().all(|&f| along(&surfs[f])) || fs.iter().all(|&f| matches!(surfs[f], Surface::Plane { .. })) {
        return None;
    }
    let line = Curve::Line { o: *p0, d };
    let w = p1 - p0;
    let size = 1.0 + w.norm() + p0.coords.norm();
    let on = (w - d * w.dot(&d)).norm() < 1e-9 * size && fs.iter().all(|&f| [p0, p1].iter().all(|p| surfs[f].distance(p) < 1e-7 * size));
    on.then_some(line)
}

/// Whether `p` projects inside the part `dom` of the surface (an angle modulo a turn).
fn in_domain(s: &Surface, dom: &UvBox, p: &P3) -> bool {
    let (u, v) = s.project(p, None);
    let inside = |x: f64, (a, b): (f64, f64), periodic: bool| {
        let tol = 1e-9 * (1.0 + (b - a).abs() + a.abs());
        let x = if periodic { a + (x - a).rem_euclid(TAU) } else { x };
        x >= a - tol && x <= b + tol || (periodic && x - TAU >= a - tol && x - TAU <= b + tol)
    };
    inside(u, dom.0, s.u_periodic()) && inside(v, dom.1, s.v_periodic())
}

/// [`replace_surfaces`], with the part of each face's new surface the face lies on (by face;
/// missing or None: anywhere). Where a new surface meets a neighbour more than once, each edge
/// is the crossing inside it.
pub fn replace_surfaces_in(b: &Body, surfs: &[Surface], domains: &[Option<UvBox>]) -> Result<Body> {
    let dom = |f: usize| domains.get(f).copied().flatten();
    let mut out = b.clone();
    let vf = vertex_faces(b);
    let ef = b.edge_faces();
    let tangent = tangent_lines(b, surfs, &ef);
    // Vertices.
    for v in 0..b.verts.len() {
        let fs: Vec<&Surface> = vf[v].iter().map(|&f| &surfs[f]).collect();
        if fs.is_empty() {
            continue;
        }
        let moved = vf[v].iter().any(|&f| surfs[f] != b.faces[f].surf);
        if !moved {
            continue;
        }
        // On a line where two faces stay tangent: where it crosses the other faces (the
        // tangent pair alone pins nothing along the line).
        let on_line = tangent.iter().find(|(e, _)| b.edges[*e].v.contains(&v)).and_then(|(e, c)| {
            let others: Vec<&Surface> = vf[v].iter().filter(|f| !ef[*e].contains(f)).map(|&f| &surfs[f]).collect();
            let p = line_meet(c, &others, b.verts[v].p)?;
            fs.iter().all(|s| s.distance(&p) < 1e-7).then_some(p)
        });
        // On a seam: on the new surface's seam line (the faces around pin it only to their
        // intersection curve).
        let on_seam = || {
            let e = (0..b.edges.len()).find(|&e| b.edges[e].v.contains(&v) && !b.edges[e].degenerate && ef[e].len() == 2 && ef[e][0] == ef[e][1])?;
            let f = ef[e][0];
            let (u, _) = b.faces[f].surf.project(&b.verts[v].p, None);
            let iso = surfs[f].iso_u(u)?;
            let others: Vec<&Surface> = vf[v].iter().filter(|&&g| g != f).map(|&g| &surfs[g]).collect();
            let p = line_meet(&iso, &others, b.verts[v].p)?;
            fs.iter().all(|s| s.distance(&p) < 1e-7).then_some(p)
        };
        // On an edge whose faces all stay: on that edge's curve, where it crosses the moved
        // faces.
        let on_kept = || {
            let e = (0..b.edges.len()).find(|&e| b.edges[e].v.contains(&v) && !b.edges[e].degenerate && ef[e].iter().all(|&f| surfs[f] == b.faces[f].surf))?;
            let others: Vec<&Surface> = vf[v].iter().filter(|&&g| surfs[g] != b.faces[g].surf).map(|&g| &surfs[g]).collect();
            let p = line_meet(&b.edges[e].curve, &others, b.verts[v].p)?;
            fs.iter().all(|s| s.distance(&p) < 1e-7).then_some(p)
        };
        // At a pole or apex (the end of a point edge): the new surface's own, nearest.
        let on_pole = || {
            let f = (0..b.edges.len()).filter(|&e| b.edges[e].degenerate && b.edges[e].v.contains(&v)).flat_map(|e| ef[e].iter().copied()).next()?;
            let old = b.verts[v].p;
            let p = singular_points(&surfs[f]).into_iter().min_by(|x, y| (x - old).norm().total_cmp(&(y - old).norm()))?;
            fs.iter().all(|s| s.distance(&p) < 1e-7).then_some(p)
        };
        let p = on_pole().or(on_line).or_else(on_seam).or_else(on_kept).or_else(|| meet(&fs, b.verts[v].p)).ok_or_else(|| KernelError::OperationFailed("the moved faces no longer meet at a corner".into()))?;
        out.verts[v].p = p;
    }
    // Edges.
    for e in 0..b.edges.len() {
        let ed = &b.edges[e];
        let fs = &ef[e];
        let moved = fs.iter().any(|&f| surfs[f] != b.faces[f].surf);
        let (p0, p1) = (out.verts[ed.v[0]].p, out.verts[ed.v[1]].p);
        if !moved {
            // Same curve; the ends may have moved along it.
            if !ed.degenerate && ((p0 - b.verts[ed.v[0]].p).norm() > 1e-12 || (p1 - b.verts[ed.v[1]].p).norm() > 1e-12) {
                let q = ed.point(ed.t[0] + (ed.t[1] - ed.t[0]) * 0.25);
                let (t0, t1) = param_range(&ed.curve, &p0, &p1, ed.v[0] == ed.v[1], &q);
                if let Curve::Line { .. } = ed.curve {
                    let (a, c) = (ed.curve.closest(&p0, -1e12, 1e12), ed.curve.closest(&p1, -1e12, 1e12));
                    out.edges[e].t = if a <= c { [a, c] } else { [c, a] };
                } else {
                    out.edges[e].t = if t0 <= t1 { [t0, t1] } else { [t1, t0] };
                }
            }
            continue;
        }
        if ed.degenerate {
            out.edges[e].curve = Curve::Line { o: p0, d: V3::zeros() };
            continue;
        }
        let old_mid = ed.point(ed.mid_t());
        let curve = if let Some((_, c)) = tangent.iter().find(|(x, _)| *x == e) {
            c.clone()
        } else if let Some(c) = swept_line(surfs, fs, &ed.curve, &p0, &p1) {
            c
        } else if fs.len() == 2 && fs[0] == fs[1] {
            // A seam: the new surface's iso-u line through the vertex (on a closed B-spline,
            // the iso line along the old seam's constant parameter, at the side it lies on).
            let s = &surfs[fs[0]];
            match s {
                Surface::Nurbs(n) => {
                    let old = &b.faces[fs[0]].surf;
                    let (ua, va) = old.project(&ed.point(ed.t[0]), None);
                    let (ub, vb) = old.project(&ed.point(ed.mid_t()), None);
                    let along_v = (ua - ub).abs() <= (va - vb).abs();
                    let (u, v) = s.project(&p0, None);
                    let ((u0, u1), (v0, v1)) = n.range();
                    let snap = |x: f64, a: f64, b: f64| if (x - a).abs() < 1e-6 * (b - a) { a } else if (x - b).abs() < 1e-6 * (b - a) { b } else { x };
                    let iso = if along_v { nurbs_iso(n, snap(u, u0, u1), true) } else { nurbs_iso(n, snap(v, v0, v1), false) };
                    Curve::Nurbs(std::sync::Arc::new(iso))
                }
                _ => {
                    let (u, _) = s.project(&p0, None);
                    s.iso_u(u).ok_or(KernelError::Unsupported("moving a seam of this surface"))?
                }
            }
        } else if fs.len() == 2 {
            let (sa, sb) = (&surfs[fs[0]], &surfs[fs[1]]);
            let mut bb = Bbox::empty();
            bb.add(&p0);
            bb.add(&p1);
            bb.add(&old_mid);
            for i in 0..=8 {
                bb.add(&ed.point(ed.t[0] + (ed.t[1] - ed.t[0]) * i as f64 / 8.0));
            }
            let bb = bb.grow(bb.diagonal() * 0.5 + 1.0);
            // Each surface's parameters over the search box (its corners projected), so an
            // unbounded direction is sampled only where the edge can be.
            let box_of = |s: &Surface| {
                let ((u0, u1), (v0, v1)) = s.param_box();
                let (mut lu, mut lv) = ((f64::MAX, f64::MIN), (f64::MAX, f64::MIN));
                for i in 0..8 {
                    let c = P3::new(if i & 1 == 0 { bb.min.x } else { bb.max.x }, if i & 2 == 0 { bb.min.y } else { bb.max.y }, if i & 4 == 0 { bb.min.z } else { bb.max.z });
                    let (u, v) = s.project(&c, None);
                    lu = (lu.0.min(u), lu.1.max(u));
                    lv = (lv.0.min(v), lv.1.max(v));
                }
                let pad = |(a, b): (f64, f64)| ((a - (b - a) * 0.1), (b + (b - a) * 0.1));
                let (lu, lv) = (pad(lu), pad(lv));
                let u = if s.u_periodic() || u1 - u0 < 1e3 { (u0, u1) } else { (u0.max(lu.0), u1.min(lu.1)) };
                let v = if s.v_periodic() || v1 - v0 < 1e3 { (v0, v1) } else { (v0.max(lv.0), v1.min(lv.1)) };
                (u, v)
            };
            let cs = super::ssi::surface_surface(sa, box_of(sa), sb, box_of(sb), &bb);
            // The component through both new end points (nearest the old middle).
            let mut best: Option<(f64, Curve)> = None;
            for ic in cs {
                let t0 = ic.c.closest(&p0, ic.t0, ic.t1);
                let t1 = ic.c.closest(&p1, ic.t0, ic.t1);
                let d = (ic.c.point(t0) - p0).norm() + (ic.c.point(t1) - p1).norm();
                if best.as_ref().is_none_or(|(bd, _)| d < *bd) {
                    best = Some((d, ic.c));
                }
            }
            if std::env::var_os("CADRS_DRAFT_DEBUG").is_some() {
                eprintln!("edge {e} faces {fs:?} ({:?} × {:?}): {:?}", sa.kind(), sb.kind(), best.as_ref().map(|b| (b.0, format!("{:?}", b.1).chars().take(12).collect::<String>())));
            }
            let (d, c) = best.ok_or_else(|| KernelError::OperationFailed("moved faces no longer meet along an edge".into()))?;
            if d > 1e-6 {
                return Err(KernelError::OperationFailed("moved faces no longer meet along an edge".into()));
            }
            c
        } else {
            return Err(KernelError::Unsupported("moving faces of a non-manifold edge"));
        };
        // Parameters at the new ends, in the old edge's direction.
        let quarter = ed.point(ed.t[0] + (ed.t[1] - ed.t[0]) * 0.25);
        // The way the old edge left its start (its direction survives the move better than
        // any of its points).
        let dir0 = ed.curve.tangent(ed.t[0]).try_normalize(1e-12).unwrap_or_else(V3::zeros);
        let (curve, (t0, t1)) = match closed_arc(&curve, &p0, &p1, ed.v[0] == ed.v[1], &dir0) {
            Some((c, t0, t1)) => (c, (t0, t1)),
            None => {
                let r = match &curve {
                    Curve::Nurbs(_) => nurbs_arc(&curve, &p0, &p1, &dir0),
                    _ => None,
                };
                let r = r.unwrap_or_else(|| param_range(&curve, &p0, &p1, ed.v[0] == ed.v[1], &quarter));
                (curve, r)
            }
        };
        // The arc must lie in its faces' domains: else the other way round (a circle crossing
        // a plane twice runs both ways between the crossings).
        let inside = |c: &Curve, t0: f64, t1: f64| {
            fs.iter().all(|&f| dom(f).is_none_or(|d| [0.25, 0.5, 0.75].iter().all(|s| in_domain(&surfs[f], &d, &c.point(t0 + (t1 - t0) * s)))))
        };
        let (t0, t1) = match &curve {
            Curve::Circle { .. } | Curve::Ellipse { .. } if ed.v[0] != ed.v[1] && !inside(&curve, t0, t1) => {
                let other = if t1 > t0 { (t0, t1 - TAU) } else { (t0, t1 + TAU) };
                if inside(&curve, other.0, other.1) { other } else { (t0, t1) }
            }
            Curve::Nurbs(_) if !inside(&curve, t0, t1) => nurbs_arc(&curve, &p0, &p1, &-dir0).filter(|(a, z)| inside(&curve, *a, *z)).unwrap_or((t0, t1)),
            _ => (t0, t1),
        };
        let (curve, t0, t1) = if t1 < t0 { let (c, a, b2) = curve.reversed(t0, t1); (c, b2.min(a), a.max(b2)) } else { (curve, t0, t1) };
        let (t0, t1) = if t1 < t0 { (t1, t0) } else { (t0, t1) };
        out.edges[e].curve = curve;
        out.edges[e].t = [t0, t1];
    }
    for (fi, s) in surfs.iter().enumerate() {
        out.faces[fi].surf = s.clone();
    }
    // p-curves, continuous along each loop.
    for fi in 0..out.faces.len() {
        let surf = out.faces[fi].surf.clone();
        for li in 0..out.faces[fi].loops.len() {
            let lp = out.faces[fi].loops[li].clone();
            let mut at: Option<P2> = None;
            let mut nl: Vec<Coedge> = Vec::new();
            // Seams on periodic faces: the second use of an edge goes a period on.
            for c in &lp {
                let e = &out.edges[c.edge];
                let old = &b.edges[c.edge];
                // Unchanged on an unchanged surface keeps its p-curve (a pole's spans the
                // surface's u, which no projection recovers).
                let same = surf == b.faces[fi].surf && (old.degenerate || (e.t == old.t && e.curve == old.curve));
                let mut pc = if same { c.pc.clone() } else { super::pcurve::pcurve(&surf, &e.curve, e.t[0], e.t[1], None) };
                let (t0, t1) = super::topo::co_range(e, c.fwd);
                if let Some(a) = at {
                    let q = pc.point(t0);
                    let du = if surf.u_periodic() { ((a.x - q.x) / TAU).round() * TAU } else { 0.0 };
                    let dv = if surf.v_periodic() { ((a.y - q.y) / TAU).round() * TAU } else { 0.0 };
                    if du != 0.0 || dv != 0.0 {
                        pc = pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)));
                    }
                    if let Some(shifted) = closed_spline_shift(&surf, &pc, t0, t1, [a.x, a.y]) {
                        pc = shifted;
                    }
                }
                let (_, t1) = super::topo::co_range(e, c.fwd);
                let q = pc.point(t1);
                at = Some(P2::new(q.x, q.y));
                nl.push(Coedge { edge: c.edge, fwd: c.fwd, pc });
            }
            out.faces[fi].loops[li] = nl;
        }
    }
    // Seams: both coedges of a seam got the same p-curve; shift the one that should be a
    // period away so the loop closes (on a closed spline, to the other side of its range).
    for fi in 0..out.faces.len() {
        let surf = out.faces[fi].surf.clone();
        if !surf.u_periodic() {
            for li in 0..out.faces[fi].loops.len() {
                let n = out.faces[fi].loops[li].len();
                for k in 0..n {
                    let prev = out.faces[fi].loops[li][(k + n - 1) % n].clone();
                    let cur = out.faces[fi].loops[li][k].clone();
                    let (_, pt1) = super::topo::co_range(&out.edges[prev.edge], prev.fwd);
                    let (ct0, ct1) = super::topo::co_range(&out.edges[cur.edge], cur.fwd);
                    if let Some(shifted) = closed_spline_shift(&surf, &cur.pc, ct0, ct1, { let a = prev.pc.point(pt1); [a.x, a.y] }) {
                        out.faces[fi].loops[li][k].pc = shifted;
                    }
                }
            }
            continue;
        }
        for li in 0..out.faces[fi].loops.len() {
            let n = out.faces[fi].loops[li].len();
            for k in 0..n {
                let prev = out.faces[fi].loops[li][(k + n - 1) % n].clone();
                let cur = out.faces[fi].loops[li][k].clone();
                let pe = &out.edges[prev.edge];
                let ce = &out.edges[cur.edge];
                let (_, pt1) = super::topo::co_range(pe, prev.fwd);
                let (ct0, _) = super::topo::co_range(ce, cur.fwd);
                let a = prev.pc.point(pt1);
                let q = cur.pc.point(ct0);
                let du = ((a.x - q.x) / TAU).round() * TAU;
                if du != 0.0 {
                    out.faces[fi].loops[li][k].pc = cur.pc.transformed(&Affine::translation(V3::new(du, 0.0, 0.0)));
                }
            }
        }
    }
    super::build::fix_senses(&mut out, 0..b.faces.len());
    // Tolerances: an edge between moved faces is a new intersection curve, its p-curves fits
    // on the new surfaces: its tolerance is how far they stray from one another (not the old
    // edge's, which a read model sets for its own geometry); likewise a moved vertex's.
    let changed: Vec<bool> = (0..b.faces.len()).map(|f| surfs[f] != b.faces[f].surf).collect();
    let mut stray = vec![0.0f64; out.edges.len()];
    for f in out.faces.iter() {
        for c in f.loops.iter().flatten() {
            let e = &out.edges[c.edge];
            if e.degenerate {
                continue;
            }
            for i in 0..=8 {
                let t = e.t[0] + (e.t[1] - e.t[0]) * i as f64 / 8.0;
                let uv = c.pc.point(t);
                let p = e.point(t);
                stray[c.edge] = stray[c.edge].max((f.surf.eval(uv.x, uv.y) - p).norm()).max(f.surf.distance(&p));
            }
        }
    }
    for (e, x) in stray.iter().enumerate() {
        if ef[e].iter().any(|&f| changed[f]) && !out.edges[e].degenerate {
            out.edges[e].tol = (1.5 * x).max(super::topo::TOL_MIN);
        }
    }
    for v in 0..out.verts.len() {
        if !vf[v].iter().any(|&f| changed[f]) {
            continue;
        }
        let p = out.verts[v].p;
        let mut d = vf[v].iter().map(|&f| out.faces[f].surf.distance(&p)).fold(0.0, f64::max);
        for e in out.edges.iter().filter(|e| !e.degenerate) {
            for k in 0..2 {
                if e.v[k] == v {
                    d = d.max((e.point(e.t[k]) - p).norm());
                }
            }
        }
        out.verts[v].tol = (1.5 * d).max(super::topo::TOL_MIN);
    }
    let errs = out.check();
    if !errs.is_empty() {
        if std::env::var_os("CADRS_DRAFT_DEBUG").is_some() {
            for (fi, f) in out.faces.iter().enumerate() {
                for l in &f.loops {
                    for c in l {
                        let e = &out.edges[c.edge];
                        let (a, z) = super::topo::co_range(e, c.fwd);
                        let (old, oe) = (&b.edges[c.edge], &b.faces[fi]);
                        let _ = oe;
                        eprintln!("face {fi} edge {} fwd {} t {:?} (old {:?}) pc {:?} -> {:?} {}", c.edge, c.fwd, e.t, old.t, c.pc.point(a), c.pc.point(z), format!("{:?}", e.curve).chars().take(60).collect::<String>());
                    }
                }
            }
        }
        return Err(KernelError::OperationFailed(format!("the moved faces don't close: {}", errs[0])));
    }
    Ok(out)
}

/// `Kernel::replace_faces`: faces given new surfaces, each keeping the side its material is
/// on (SMLib's `SmBrep::LocalOperation` keeps the face's use).
pub fn replace_faces(k: &mut NativeKernel, body: BodyId, faces: &[FaceReplacement]) -> Result<OpResult> {
    let b = (**k.get(body)?).clone();
    let mut surfs: Vec<Surface> = b.faces.iter().map(|f| f.surf.clone()).collect();
    let mut domains: Vec<Option<UvBox>> = vec![None; b.faces.len()];
    for r in faces {
        let fi = r.face.0 as usize;
        if fi >= b.faces.len() {
            return Err(KernelError::InvalidParameter(format!("unknown {:?}", r.face)));
        }
        surfs[fi] = r.surface.clone();
        domains[fi] = r.domain;
    }
    let mut out = replace_surfaces_in(&b, &surfs, &domains)?;
    // The outward normal at the middle of each new face agrees with the old face's at its own.
    let outward = |body: &Body, fi: usize, p: &P3| {
        let f = &body.faces[fi];
        let (u, v) = f.surf.project(p, None);
        f.surf.normal(u, v) * if f.sense { 1.0 } else { -1.0 }
    };
    for r in faces {
        let fi = r.face.0 as usize;
        let middle = |body: &Body| super::smb::face_centroids(&super::kernel::subset(body, &[fi], false)).first().copied();
        let (Some(was), Some(now)) = (middle(&b), middle(&out)) else { continue };
        let before = outward(&b, fi, &was);
        if outward(&out, fi, &now).dot(&before) < 0.0 {
            out.faces[fi].sense = !out.faces[fi].sense;
        }
    }
    let errs = out.check();
    if let Some(e) = errs.first() {
        return Err(KernelError::OperationFailed(format!("the replaced faces don't close: {e}")));
    }
    if super::mass::body_mass(&out).volume <= 0.0 {
        return Err(KernelError::OperationFailed("the replaced faces turn the body inside out".into()));
    }
    let nf = out.faces.len();
    let id = k.insert(out);
    let history = History { modified: (0..nf).map(|i| (FaceId(i as u64), InputFace { body, face: FaceId(i as u64) })).collect(), ..Default::default() };
    Ok(OpResult { bodies: vec![id], history })
}

/// On a closed B-spline (an intersection loop), the arc from `p0` to `p1` leaving `p0` along
/// `dir0` (the whole loop for a closed edge) as one parameter range: the curve restarted at
/// `p0`, so the arc never runs through its start.
fn closed_arc(c: &Curve, p0: &P3, p1: &P3, closed_edge: bool, dir0: &V3) -> Option<(Curve, f64, f64)> {
    let Curve::Nurbs(n) = c else { return None };
    let (a, b) = n.range();
    let (s, e) = (n.point(a), n.point(b));
    if (s - e).norm() > 1e-9 * (1.0 + s.coords.norm()) {
        return None;
    }
    let t0 = nurbs_passes(c, p0).into_iter().next().unwrap_or_else(|| c.closest(p0, a, b));
    let r = n.restarted(t0);
    let (a, b) = r.range();
    let rc = Curve::Nurbs(std::sync::Arc::new(r));
    let forward = rc.tangent(a).dot(dir0) >= 0.0;
    if closed_edge {
        return Some(if forward { (rc, a, b) } else { (rc, b, a) });
    }
    let t1 = rc.closest(p1, a, b);
    Some(if forward { (rc, a, t1) } else { (rc, b, t1) })
}

/// Parameters of two points on a curve; a closed edge spans a whole turn from p0.
fn param_range(c: &Curve, p0: &P3, p1: &P3, closed: bool, quarter: &P3) -> (f64, f64) {
    match c {
        Curve::Circle { .. } | Curve::Ellipse { .. } => {
            let t0 = c.closest(p0, -PI, PI);
            if closed {
                // The direction from the old edge's quarter point.
                let tq = c.closest(quarter, t0 - PI, t0 + PI);
                if tq >= t0 { (t0, t0 + TAU) } else { (t0, t0 - TAU) }
            } else {
                let t1 = c.closest(p1, t0 - PI, t0 + PI);
                // Choose the way round through the old quarter point.
                let tq = c.closest(quarter, t0 - TAU, t0 + TAU);
                let between = |a: f64, b: f64, x: f64| (x - a) * (x - b) <= 0.0;
                if between(t0, t1, unwrap_near(tq, (t0 + t1) / 2.0, TAU)) {
                    (t0, t1)
                } else if t1 > t0 {
                    (t0, t1 - TAU)
                } else {
                    (t0, t1 + TAU)
                }
            }
        }
        _ => {
            let (a, b) = c.nurbs_range().unwrap_or((-1e7, 1e7));
            (c.closest(p0, a, b), c.closest(p1, a, b))
        }
    }
}

/// The parameters where a B-spline passes through `p` (more than one where it runs back near
/// itself).
fn nurbs_passes(c: &Curve, p: &P3) -> Vec<f64> {
    const N: usize = 400;
    let Some((a, b)) = c.nurbs_range() else { return vec![] };
    let ts: Vec<f64> = (0..=N).map(|i| a + (b - a) * i as f64 / N as f64).collect();
    let pts: Vec<P3> = ts.iter().map(|&t| c.point(t)).collect();
    let scale = 1.0 + pts.iter().map(|q| q.coords.norm()).fold(0.0, f64::max);
    let d: Vec<f64> = pts.iter().map(|q| (q - p).norm()).collect();
    let mut out: Vec<f64> = Vec::new();
    for i in 0..=N {
        let left = if i == 0 { f64::MAX } else { d[i - 1] };
        let right = if i == N { f64::MAX } else { d[i + 1] };
        if d[i] <= left && d[i] <= right {
            let t = c.closest(p, ts[i.saturating_sub(1)], ts[(i + 1).min(N)]);
            if (c.point(t) - p).norm() < 1e-6 * scale && !out.iter().any(|x| (x - t).abs() < (b - a) * 1e-6) {
                out.push(t);
            }
        }
    }
    out
}

/// On a B-spline, the arc from `p0` to `p1` leaving `p0` along `dir0`: of the parameters where
/// the curve passes each end, the shortest pair running that way.
fn nurbs_arc(c: &Curve, p0: &P3, p1: &P3, dir0: &V3) -> Option<(f64, f64)> {
    let (s0, s1) = (nurbs_passes(c, p0), nurbs_passes(c, p1));
    let mut best: Option<(f64, (f64, f64))> = None;
    for &t0 in &s0 {
        for &t1 in &s1 {
            if c.tangent(t0).dot(dir0) * (t1 - t0) <= 0.0 {
                continue;
            }
            if best.is_none_or(|(w, _)| (t1 - t0).abs() < w) {
                best = Some(((t1 - t0).abs(), (t0, t1)));
            }
        }
    }
    best.map(|(_, r)| r)
}

/// A surface moved `d` along its outward normal (sense applied).
pub fn offset_surface(s: &Surface, sense: bool, d: f64) -> Option<Surface> {
    // A B-spline that is a plane, cylinder, cone or sphere: its exact offset (a fit of a
    // cone's offset would have to pass its apex).
    if let Surface::Nurbs(n) = s
        && let Some(a) = super::recognize::surface(n)
    {
        let ((u0, u1), (v0, v1)) = n.range();
        let (u, v) = ((u0 + u1) / 2.0, (v0 + v1) / 2.0);
        let (ua, va) = a.project(&s.eval(u, v), None);
        let same = s.normal(u, v).dot(&a.normal(ua, va)) > 0.0;
        return offset_surface(&a, sense == same, d);
    }
    let d = if sense { d } else { -d };
    Some(match s {
        Surface::Plane { f } => {
            let n = f.x.cross(&f.y);
            Surface::Plane { f: Frame { o: f.o + n * d, ..*f } }
        }
        Surface::Cylinder { f, r } => {
            let r2 = r + d * if f.right_handed() { 1.0 } else { -1.0 };
            if r2 <= 1e-9 {
                return None;
            }
            Surface::Cylinder { f: *f, r: r2 }
        }
        Surface::Sphere { f, r } => {
            let r2 = r + d * if f.right_handed() { 1.0 } else { -1.0 };
            if r2 <= 1e-9 {
                return None;
            }
            Surface::Sphere { f: *f, r: r2 }
        }
        Surface::Torus { f, rmaj, rmin } => {
            let r2 = rmin + d * if f.right_handed() { 1.0 } else { -1.0 };
            if r2 <= 1e-9 {
                return None;
            }
            Surface::Torus { f: *f, rmaj: *rmaj, rmin: r2 }
        }
        Surface::Cone { f, r, a } => {
            // Moving along the normal (cos a, −sin a) in the (ρ, h) meridian.
            let k = d * if f.right_handed() { 1.0 } else { -1.0 };
            Surface::Cone { f: Frame { o: f.o - f.z * (k * a.sin()), ..*f }, r: r + k * a.cos(), a: *a }
        }
        // An extrusion: its curve moved square to the direction (the normal is), extruded the
        // same way.
        Surface::Extrusion { c, d: dir } => {
            let off = offset_curve_along(c, &|t| s.normal(t, 0.0), d)?;
            Surface::Extrusion { c: off, d: *dir }
        }
        // A revolution: its meridian (at u = 0) moved along the normal there, revolved.
        Surface::Revolution { f, m } => {
            let off = offset_curve_along(m, &|t| s.normal(0.0, t), d)?;
            Surface::Revolution { f: *f, m: off }
        }
        // A B-spline: a surface through a grid of offset points (exact offsets aren't
        // B-splines).
        Surface::Nurbs(n) => {
            // The grid doubled until the surface holds the offset at the cells' middles too.
            // It spans a fifth more of the surface past each side where the surface is open
            // there (its polynomial continuation): a face moved in must reach its neighbours'
            // offsets past its old edges (SMLib extends the surfaces too,
            // `GetCreateExtendedSurface`). Not past a side the surface closes up across, nor
            // past a pole (a side collapsed to a point).
            let ((a0, a1), (b0, b1)) = n.range();
            let size = 1.0 + n.cps.iter().flatten().map(|h| (h.xyz() / h.w).norm()).fold(0.0, f64::max);
            // Points along each side: u = a0, u = a1, v = b0, v = b1.
            let side = |k: usize| -> Vec<P3> {
                (0..=4)
                    .map(|j| {
                        let (fu, fv) = (a0 + (a1 - a0) * j as f64 / 4.0, b0 + (b1 - b0) * j as f64 / 4.0);
                        match k {
                            0 => s.eval(a0, fv),
                            1 => s.eval(a1, fv),
                            2 => s.eval(fu, b0),
                            _ => s.eval(fu, b1),
                        }
                    })
                    .collect()
            };
            let sides: Vec<Vec<P3>> = (0..4).map(side).collect();
            let same = |p: &P3, q: &P3| (p - q).norm() < 1e-9 * size;
            let closed = |k: usize| sides[k].iter().zip(&sides[k + 1]).all(|(p, q)| same(p, q));
            let pole = |k: usize| sides[k].iter().all(|p| same(p, &sides[k][0]));
            // (Nor a rational one: past its domain its weights can reach zero.)
            let w0 = n.cps[0][0].w;
            let rational = n.cps.iter().flatten().any(|h| (h.w - w0).abs() > 1e-12 * w0.abs());
            let grow = |(a, b): (f64, f64), k: usize| {
                if rational {
                    return (a, b);
                }
                let w = 0.2 * (b - a);
                (if closed(k) || pole(k) { a } else { a - w }, if closed(k) || pole(k + 1) { b } else { b + w })
            };
            let ((u0, u1), (v0, v1)) = (grow((a0, a1), 0), grow((b0, b1), 2));
            // At a pole the normal is the limit from inside the domain.
            let (uc, vc) = ((a0 + a1) / 2.0, (b0 + b1) / 2.0);
            let normal = |u: f64, v: f64| {
                let dd = s.d2(u, v);
                if dd[1].cross(&dd[2]).norm() > 1e-12 * (dd[1].norm_squared() + dd[2].norm_squared()) {
                    return s.normal(u, v);
                }
                s.normal(u + (uc - u) * 1e-6, v + (vc - v) * 1e-6)
            };
            let at = |u: f64, v: f64| s.eval(u, v) + normal(u, v) * d;
            let mut k = 32;
            loop {
                let us: Vec<f64> = (0..=k).map(|i| u0 + (u1 - u0) * i as f64 / k as f64).collect();
                let vs: Vec<f64> = (0..=k).map(|j| v0 + (v1 - v0) * j as f64 / k as f64).collect();
                let rows: Vec<NurbsCurve> = us
                    .iter()
                    .map(|&u| {
                        let pts: Vec<P3> = vs.iter().map(|&v| at(u, v)).collect();
                        NurbsCurve::interpolate(&pts, &vs, 3, None, None)
                    })
                    .collect();
                let surf = super::nurbs::NurbsSurface::through_curves(&rows, &us, 3, None, None);
                // through_curves runs its rows along v and interpolates them along u: transpose
                // to keep the parameters' roles.
                let cps: Vec<Vec<super::nurbs::H>> = (0..surf.cps[0].len()).map(|j| surf.cps.iter().map(|r| r[j]).collect()).collect();
                let off = Surface::Nurbs(std::sync::Arc::new(super::nurbs::NurbsSurface { du: surf.dv, dv: surf.du, ku: surf.kv.clone(), kv: surf.ku.clone(), cps }));
                let err = us
                    .windows(2)
                    .flat_map(|a| vs.windows(2).map(move |b| (0.5 * (a[0] + a[1]), 0.5 * (b[0] + b[1]))))
                    .map(|(u, v)| (off.eval(u, v) - at(u, v)).norm())
                    .fold(0.0, f64::max);
                if err < OFFSET_FIT * (1.0 + d.abs()) * 1e3 || k >= 32 {
                    break off;
                }
                k *= 2;
            }
        }
    })
}

/// A curve moved by `d` along the unit field `n(t)`: exact for a line moved by a constant
/// vector and a circle moved square to itself in its plane (concentric), else a cubic through
/// 65 offset points on its parameters. None when a circle would shrink to nothing.
fn offset_curve_along(c: &Curve, n: &dyn Fn(f64) -> V3, d: f64) -> Option<Curve> {
    let (t0, t1) = c.nurbs_range().unwrap_or(match c {
        Curve::Circle { .. } | Curve::Ellipse { .. } => (0.0, TAU),
        _ => (-1.0, 1.0),
    });
    match c {
        Curve::Line { o, d: dir } => {
            let (a, b) = (n(t0), n(t1));
            if (a - b).norm() < 1e-9 {
                return Some(Curve::Line { o: o + a * d, d: *dir });
            }
        }
        Curve::Circle { c: cc, x, y, r } => {
            // Normal along the radius (in the plane): a concentric circle.
            let p = c.point(0.0);
            let radial = (p - cc) / *r;
            let nn = n(0.0);
            let along = nn.dot(&radial);
            if (along.abs() - 1.0).abs() < 1e-9 {
                let r2 = r + d * along;
                return (r2 > 1e-9).then_some(Curve::Circle { c: *cc, x: *x, y: *y, r: r2 });
            }
            // Normal across the plane: the circle moved along it.
            let z = x.cross(y);
            if (nn.dot(&z).abs() - 1.0).abs() < 1e-9 {
                return Some(Curve::Circle { c: cc + nn * d, x: *x, y: *y, r: *r });
            }
        }
        _ => {}
    }
    // A cubic through offset points, their number doubled until it holds the offset between
    // them too (the curve's knots kink the offset).
    let at = |t: f64| c.point(t) + n(t) * d;
    let mut k = 64;
    loop {
        let ts: Vec<f64> = (0..=k).map(|i| t0 + (t1 - t0) * i as f64 / k as f64).collect();
        let pts: Vec<P3> = ts.iter().map(|&t| at(t)).collect();
        let fit = Curve::Nurbs(std::sync::Arc::new(NurbsCurve::interpolate(&pts, &ts, 3, None, None)));
        let err = ts.windows(2).map(|w| (fit.point(0.5 * (w[0] + w[1])) - at(0.5 * (w[0] + w[1]))).norm()).fold(0.0, f64::max);
        if err < OFFSET_FIT * (1.0 + d.abs()) || k >= 1024 {
            return Some(fit);
        }
        k *= 2;
    }
}

/// How closely an approximated offset (curve or surface) follows the true one (mm, relative to
/// the distance).
const OFFSET_FIT: f64 = 1e-9;

/// Sharp offset: every face moved by its distance, the edges where the moved faces meet.
pub fn offset_sharp(b: &Body, dist: &[f64]) -> Result<Body> {
    let surfs: Vec<Surface> = b
        .faces
        .iter()
        .zip(dist)
        .map(|(f, d)| if *d == 0.0 { Some(f.surf.clone()) } else { offset_surface(&f.surf, f.sense, *d) })
        .collect::<Option<Vec<_>>>()
        .ok_or(KernelError::Unsupported("offsetting this kind of face"))?;
    replace_surfaces(b, &surfs)
}

pub fn offset(k: &mut NativeKernel, body: BodyId, spec: &OffsetSpec) -> Result<OpResult> {
    let b = (**k.get(body)?).clone();
    let mut dist = vec![spec.distance; b.faces.len()];
    for (f, d) in &spec.faces {
        if let Some(x) = dist.get_mut(f.0 as usize) {
            *x = *d;
        }
    }
    let out = if spec.sharp || dist.iter().all(|d| *d == 0.0) {
        offset_sharp(&b, &dist)?
    } else if dist.iter().all(|d| *d <= 0.0) {
        super::offset_round::inset_rounded(&b, &dist)?
    } else {
        super::offset_round::offset_rounded(&b, &dist)?
    };
    let v0 = super::mass::body_mass(&b).volume;
    let v1 = super::mass::body_mass(&out).volume;
    let grow = dist.iter().any(|d| *d > 0.0);
    let shrink = dist.iter().any(|d| *d < 0.0);
    if (grow && !shrink && v1 < v0 - 1e-9) || (shrink && !grow && v1 > v0 + 1e-9) || v1 <= 0.0 {
        return Err(KernelError::OperationFailed("the offset turned the body inside out".into()));
    }
    let nf = out.faces.len();
    let id = k.insert(out);
    let history = History {
        modified: (0..nf.min(b.faces.len())).map(|i| (FaceId(i as u64), InputFace { body, face: FaceId(i as u64) })).collect(),
        ..Default::default()
    };
    Ok(OpResult { bodies: vec![id], history })
}

/// The faces reached from `seed` across smooth edges (the faces' normals agree along them).
pub fn tangent_closure(b: &Body, mut seed: Vec<usize>) -> Vec<usize> {
    let ef = b.edge_faces();
    let normal = |fi: usize, p: &P3| {
        let f = &b.faces[fi];
        let (u, v) = f.surf.project(p, None);
        f.surf.normal(u, v) * if f.sense { 1.0 } else { -1.0 }
    };
    let smooth = |e: usize| {
        let ed = &b.edges[e];
        let fs = &ef[e];
        if ed.degenerate || fs.len() != 2 || fs[0] == fs[1] {
            return None;
        }
        let ok = [0.25, 0.5, 0.75].iter().all(|&s| {
            let p = ed.point(ed.t[0] + (ed.t[1] - ed.t[0]) * s);
            normal(fs[0], &p).dot(&normal(fs[1], &p)) > 1.0 - 1e-6
        });
        ok.then_some((fs[0], fs[1]))
    };
    let pairs: Vec<(usize, usize)> = (0..b.edges.len()).filter_map(smooth).collect();
    let mut i = 0;
    while i < seed.len() {
        let f = seed[i];
        for &(x, y) in &pairs {
            let o = if x == f { y } else if y == f { x } else { continue };
            if !seed.contains(&o) {
                seed.push(o);
            }
        }
        i += 1;
    }
    seed
}

/// Draft: the faces turned about the neutral plane by the angle, leaning in as they rise along
/// the pull direction.
pub fn draft(k: &mut NativeKernel, body: BodyId, spec: &DraftSpec) -> Result<OpResult> {
    let b = (**k.get(body)?).clone();
    let pull = spec.pull.into_inner();
    let mut surfs: Vec<Surface> = b.faces.iter().map(|f| f.surf.clone()).collect();
    let mut faces: Vec<usize> = Vec::new();
    for fid in &spec.faces {
        let fi = fid.0 as usize;
        b.faces.get(fi).ok_or_else(|| KernelError::InvalidParameter(format!("unknown {fid:?}")))?;
        if !faces.contains(&fi) {
            faces.push(fi);
        }
    }
    if spec.tangent_propagation {
        faces = tangent_closure(&b, faces);
    }
    for fi in faces {
        let f = &b.faces[fi];
        surfs[fi] = match &f.surf {
            Surface::Plane { f: fr } => {
                let n = fr.x.cross(&fr.y) * if f.sense { 1.0 } else { -1.0 };
                if n.cross(&pull).norm() < 1e-9 {
                    return Err(KernelError::InvalidParameter("A face square to the pull direction can't be drafted".into()));
                }
                // The line where the face meets the neutral plane (the hinge), and the face
                // turned about it to make the angle with the pull (OCCT's: absolute, so a face
                // already inclined takes the angle given), its outward normal tipping towards
                // the pull (the face leans in as it rises).
                let nn = spec.neutral_normal.map_or(pull, |x| x.into_inner());
                let t = nn.cross(&n).try_normalize(1e-9).ok_or_else(|| KernelError::InvalidParameter("A face parallel to the neutral plane can't be drafted".into()))?;
                let m = t.cross(&n);
                let alpha = -(fr.o - spec.neutral).dot(&nn) / m.dot(&nn);
                let on = fr.o + m * alpha;
                // The new normal turns about the hinge (n cos θ + m sin θ) until its component
                // along the pull is sin a; of the two turns, the smaller.
                let (ca, cb) = (n.dot(&pull), m.dot(&pull));
                let r = (ca * ca + cb * cb).sqrt();
                if r < spec.angle.sin().abs() - 1e-12 {
                    return Err(KernelError::InvalidParameter("the face can't reach that angle turning about its hinge".into()));
                }
                let phi = cb.atan2(ca);
                let delta = (spec.angle.sin() / r).clamp(-1.0, 1.0).acos();
                let wrap = |x: f64| (x + PI).rem_euclid(TAU) - PI;
                let (th1, th2) = (wrap(phi + delta), wrap(phi - delta));
                let th = if th1.abs() <= th2.abs() { th1 } else { th2 };
                let n2 = n * th.cos() + m * th.sin();
                let natural = if f.sense { n2 } else { -n2 };
                Surface::Plane { f: Frame { o: on, x: t, y: natural.cross(&t), z: natural } }
            }
            Surface::Cylinder { f: fr, .. } | Surface::Cone { f: fr, .. } => {
                if spec.neutral_normal.is_some() {
                    return Err(KernelError::Unsupported("drafting a cylinder or cone about a neutral plane at an angle to the pull"));
                }
                if fr.z.cross(&pull).norm() > 1e-9 {
                    return Err(KernelError::Unsupported("drafting a cylinder or cone not along the pull"));
                }
                let (r, a0) = match &f.surf {
                    Surface::Cone { r, a, .. } => (*r, *a),
                    Surface::Cylinder { r, .. } => (*r, 0.0),
                    _ => unreachable!(),
                };
                // A cone through the face's circle at the neutral plane, narrowing along the
                // pull for a positive angle on an outward-facing face (the angle absolute, as
                // OCCT's: a cone takes the angle given, whatever its own).
                let h0 = (spec.neutral - fr.o).dot(&fr.z);
                let r0 = r + h0 / a0.cos() * a0.sin();
                if r0 <= 0.0 {
                    return Err(KernelError::OperationFailed("the cone's apex is on the far side of the neutral plane".into()));
                }
                let outward = f.sense == fr.right_handed();
                let s_pull = fr.z.dot(&pull).signum();
                // v along the generator; radius r + v sin a with a negative to narrow as z rises.
                let a = -spec.angle * s_pull * if outward { 1.0 } else { -1.0 };
                Surface::Cone { f: Frame { o: fr.o + fr.z * h0, ..*fr }, r: r0, a }
            }
            other => {
                if std::env::var_os("CADRS_DRAFT_DEBUG").is_some() {
                    eprintln!("draft: unsupported face {fi}: {:?}", other.kind());
                }
                return Err(KernelError::Unsupported("drafting this kind of face"));
            }
        };
    }
    let out = replace_surfaces(&b, &surfs)?;
    let nf = out.faces.len();
    let id = k.insert(out);
    let history = History { modified: (0..nf).map(|i| (FaceId(i as u64), InputFace { body, face: FaceId(i as u64) })).collect(), ..Default::default() };
    Ok(OpResult { bodies: vec![id], history })
}
