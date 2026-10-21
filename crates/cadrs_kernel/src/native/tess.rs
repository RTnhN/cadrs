//! Tessellation: every edge is sampled once (so neighbouring faces share their boundary points
//! and the mesh is watertight), then each face is triangulated in its (u, v) domain by a
//! constrained Delaunay triangulation with interior points spaced by the surface's curvature.
//! Faces are meshed in parallel and kept as separate chunks (`FaceMesh`) so callers can cache and
//! re-upload only the faces an operation changed.

use rayon::prelude::*;
use spade::{ConstrainedDelaunayTriangulation, Point2 as SPoint, Triangulation};

use super::math::*;
use super::surface::Surface;
use super::topo::{co_range, Body, Face};

#[derive(Debug, Clone, Default)]
pub struct FaceMesh {
    pub positions: Vec<P3>,
    pub normals: Vec<V3>,
    pub indices: Vec<[u32; 3]>,
}

/// Each edge's sample parameters and points.
pub type EdgeSamples = Vec<(Vec<f64>, Vec<P3>)>;

/// Each edge's sample parameters and points.
pub fn edge_samples(body: &Body, defl: f64, ang: f64) -> EdgeSamples {
    body.edges
        .par_iter()
        .map(|e| {
            if e.degenerate {
                return (vec![e.t[0], e.t[1]], vec![body.verts[e.v[0]].p; 2]);
            }
            let ts = e.curve.sample_params(e.t[0], e.t[1], defl, ang);
            let mut pts: Vec<P3> = ts.iter().map(|&t| e.point(t)).collect();
            // Ends exactly at the vertices.
            let n = pts.len();
            pts[0] = body.verts[e.v[0]].p;
            pts[n - 1] = body.verts[e.v[1]].p;
            (ts, pts)
        })
        .collect()
}

/// Parameter steps in u and v so the mesh stays within `defl` and `ang` of the surface,
/// estimated over a sample grid of the face's uv box.
fn steps(s: &Surface, ub: (f64, f64), vb: (f64, f64), defl: f64, ang: f64) -> (f64, f64) {
    if matches!(s, Surface::Plane { .. }) {
        return (f64::INFINITY, f64::INFINITY);
    }
    let n = 6;
    let (mut ku, mut kv): (f64, f64) = (0.0, 0.0); // max |S_uu·n| / |S_u| style rates
    let (mut cu, mut cv): (f64, f64) = (0.0, 0.0); // max |S_uu·n|
    for i in 0..=n {
        for j in 0..=n {
            let u = ub.0 + (ub.1 - ub.0) * i as f64 / n as f64;
            let v = vb.0 + (vb.1 - vb.0) * j as f64 / n as f64;
            let d = s.d2(u, v);
            let nn = d[1].cross(&d[2]);
            if nn.norm() < 1e-12 {
                continue;
            }
            let nn = nn.normalize();
            let suu = d[3].dot(&nn).abs();
            let svv = d[5].dot(&nn).abs();
            cu = cu.max(suu);
            cv = cv.max(svv);
            let lu = d[1].norm().max(1e-12);
            let lv = d[2].norm().max(1e-12);
            ku = ku.max(suu / lu);
            kv = kv.max(svv / lv);
        }
    }
    let step = |c: f64, k: f64| -> f64 {
        if c < 1e-12 {
            return f64::INFINITY;
        }
        let by_defl = (8.0 * defl / c).sqrt();
        // The same angular step as the edges (`curve::arc_step`): half of `ang`.
        let by_ang = if k > 1e-12 { 0.5 * ang / k } else { f64::INFINITY };
        by_defl.min(by_ang)
    };
    (step(cu, ku), step(cv, kv))
}

/// The mean |S_u| and |S_v| over a sample grid of the box.
fn mean_speeds(s: &Surface, ub: (f64, f64), vb: (f64, f64)) -> Option<(f64, f64)> {
    let n = 6;
    let (mut su, mut sv, mut k) = (0.0, 0.0, 0usize);
    for i in 0..=n {
        for j in 0..=n {
            let (_, du, dv) = s.d1(ub.0 + (ub.1 - ub.0) * i as f64 / n as f64, vb.0 + (vb.1 - vb.0) * j as f64 / n as f64);
            su += du.norm();
            sv += dv.norm();
            k += 1;
        }
    }
    let (su, sv) = (su / k as f64, sv / k as f64);
    (su > 1e-12 && sv > 1e-12 && su.is_finite() && sv.is_finite()).then_some((su, sv))
}

/// Triangulates one face. `es` holds each edge's samples.
pub fn mesh_face(body: &Body, f: &Face, es: &[(Vec<f64>, Vec<P3>)], defl: f64, ang: f64) -> FaceMesh {
    let t_start = std::time::Instant::now();
    let prof = std::env::var("CADRS_TESS_PROFILE").is_ok();
    let (mu, mv) = f.surf.metric();
    // Boundary points: (uv, 3D point).
    let mut pts_uv: Vec<P2> = Vec::new();
    let mut pts_3d: Vec<P3> = Vec::new();
    let mut loops: Vec<(usize, usize)> = Vec::new(); // ranges into pts
    let mut ubox = (f64::INFINITY, f64::NEG_INFINITY);
    let mut vbox = (f64::INFINITY, f64::NEG_INFINITY);
    let mut deg_rate = 0.0f64;
    let _ = &mut deg_rate;
    // A rough u step for degenerate edges.
    for l in &f.loops {
        let start = pts_uv.len();
        for c in l {
            let e = &body.edges[c.edge];
            let (ts, ps) = &es[c.edge];
            let add = |t: f64, p: P3, pts_uv: &mut Vec<P2>, pts_3d: &mut Vec<P3>| {
                let q = c.pc.point(t);
                pts_uv.push(P2::new(q.x, q.y));
                pts_3d.push(p);
            };
            if e.degenerate {
                // Sample the p-curve across the pole.
                let (a, b) = co_range(e, c.fwd);
                let n = (((b - a).abs() / (ang.max(0.05))).ceil() as usize).clamp(1, 512);
                for i in 0..n {
                    let t = a + (b - a) * i as f64 / n as f64;
                    add(t, ps[0], &mut pts_uv, &mut pts_3d);
                }
                continue;
            }
            let n = ts.len();
            if c.fwd {
                for i in 0..n - 1 {
                    add(ts[i], ps[i], &mut pts_uv, &mut pts_3d);
                }
            } else {
                for i in (1..n).rev() {
                    add(ts[i], ps[i], &mut pts_uv, &mut pts_3d);
                }
            }
        }
        for p in &pts_uv[start..] {
            ubox = (ubox.0.min(p.x), ubox.1.max(p.x));
            vbox = (vbox.0.min(p.y), vbox.1.max(p.y));
        }
        loops.push((start, pts_uv.len()));
    }
    let nb = pts_uv.len();
    if nb < 3 {
        return FaceMesh::default();
    }
    // B-spline parameters carry no length: scale them by the surface's mean speeds over the
    // face's box, so the grid's steps and the triangulation work in about true lengths (a long
    // straight-along-v fillet strip with v in [0, 1] would otherwise get no interior rows).
    let (mu, mv) = if matches!(f.surf, Surface::Nurbs(_)) { mean_speeds(&f.surf, ubox, vbox).unwrap_or((mu, mv)) } else { (mu, mv) };
    // Interior grid.
    let (hu, hv) = steps(&f.surf, ubox, vbox, defl, ang);
    // A step longer than the face is no step (straight to rounding: a loft along a line).
    let hu = if hu >= ubox.1 - ubox.0 { f64::INFINITY } else { hu };
    let hv = if hv >= vbox.1 - vbox.0 { f64::INFINITY } else { hv };
    let mut grid: Vec<P2> = Vec::new();
    if hu.is_finite() || hv.is_finite() {
        let nu = if hu.is_finite() { (((ubox.1 - ubox.0) / hu).ceil() as usize).clamp(1, 400) } else { 1 };
        let mut nv = if hv.is_finite() { (((vbox.1 - vbox.0) / hv).ceil() as usize).clamp(1, 400) } else { 1 };
        // Straight along v (cylinders, cones, extrusions): long triangles from one end to the other
        // are exact there and keep the rulings whole; only very long ones are cut.
        if !hv.is_finite() && hu.is_finite() {
            let du = (ubox.1 - ubox.0) / nu as f64 * mu;
            let ext = (vbox.1 - vbox.0) * mv;
            // Rows no further apart than half the face's width, or Delaunay bridges a narrow
            // strip from rail to rail across its curve.
            let width = (ubox.1 - ubox.0) * mu;
            nv = ((ext / (du * 64.0).min(0.5 * width)).ceil() as usize).clamp(1, 400);
        }
        let nu = if !hu.is_finite() && hv.is_finite() {
            let dv = (vbox.1 - vbox.0) / nv as f64 * mv;
            let ext = (ubox.1 - ubox.0) * mu;
            let width = (vbox.1 - vbox.0) * mv;
            ((ext / (dv * 8.0).min(0.5 * width)).ceil() as usize).clamp(1, 400)
        } else {
            nu
        };
        // Rows (columns) no further apart than half the face's width across them, or Delaunay
        // bridges a narrow curved strip from side to side.
        let (wu, wv) = ((ubox.1 - ubox.0) * mu, (vbox.1 - vbox.0) * mv);
        if nu > 1 {
            nv = nv.max(((wv / (0.5 * wu)).ceil() as usize).clamp(1, 400));
        }
        let nu = if nv > 1 { nu.max(((wu / (0.5 * wv)).ceil() as usize).clamp(1, 400)) } else { nu };
        if nu > 1 || nv > 1 {
            let du = (ubox.1 - ubox.0) / nu as f64;
            let dv = (vbox.1 - vbox.0) / nv as f64;
            let min_d = 0.4 * (du * mu).min(dv * mv);
            // Boundary segments in scaled space, bucketed for the distance test.
            let segs: Vec<(P2, P2)> = loops
                .iter()
                .flat_map(|&(a, b)| {
                    let pts = &pts_uv;
                    (a..b).map(move |i| {
                        let j = if i + 1 == b { a } else { i + 1 };
                        (P2::new(pts[i].x * mu, pts[i].y * mv), P2::new(pts[j].x * mu, pts[j].y * mv))
                    })
                })
                .collect();
            // The boundary's segments bucketed by grid cell (scaled space), for the distance test.
            let (cw, chh) = (du * mu, dv * mv);
            let (ox, oy) = (ubox.0 * mu, vbox.0 * mv);
            let cell = |x: f64, y: f64| -> (i64, i64) { (((x - ox) / cw).floor() as i64, ((y - oy) / chh).floor() as i64) };
            let mut buckets: std::collections::HashMap<(i64, i64), Vec<usize>> = std::collections::HashMap::new();
            for (k, (a, b)) in segs.iter().enumerate() {
                let (c0, c1) = (cell(a.x.min(b.x) - min_d, a.y.min(b.y) - min_d), cell(a.x.max(b.x) + min_d, a.y.max(b.y) + min_d));
                if (c1.0 - c0.0 + 1) * (c1.1 - c0.1 + 1) > 4096 {
                    // A very long segment: in every cell of its rows would be too many; test it
                    // against every point instead.
                    buckets.entry((i64::MIN, i64::MIN)).or_default().push(k);
                    continue;
                }
                for cx in c0.0..=c1.0 {
                    for cy in c0.1..=c1.1 {
                        buckets.entry((cx, cy)).or_default().push(k);
                    }
                }
            }
            let always: Vec<usize> = buckets.get(&(i64::MIN, i64::MIN)).cloned().unwrap_or_default();
            // Per row of the grid, the inside intervals along u (one scanline pass over the
            // boundary instead of a point-in-polygon test per point).
            for j in 1..nv {
                let v = vbox.0 + dv * j as f64;
                let mut xs: Vec<f64> = Vec::new();
                for &(a, b) in &loops {
                    for i in a..b {
                        let k = if i + 1 == b { a } else { i + 1 };
                        let (p, q) = (pts_uv[i], pts_uv[k]);
                        if (p.y > v) != (q.y > v) {
                            xs.push(p.x + (v - p.y) * (q.x - p.x) / (q.y - p.y));
                        }
                    }
                }
                xs.sort_by(f64::total_cmp);
                for i in 1..nu {
                    let u = ubox.0 + du * i as f64;
                    let crossings = xs.partition_point(|x| *x < u);
                    if crossings % 2 == 0 {
                        continue;
                    }
                    let q = P2::new(u, v);
                    let qs = P2::new(q.x * mu, q.y * mv);
                    let c = cell(qs.x, qs.y);
                    let near = buckets.get(&c).is_some_and(|ks| ks.iter().any(|&k| seg_dist2(&qs, &segs[k].0, &segs[k].1) < min_d))
                        || always.iter().any(|&k| seg_dist2(&qs, &segs[k].0, &segs[k].1) < min_d);
                    if near {
                        continue;
                    }
                    grid.push(q);
                }
            }
        }
    }
    if prof {
        eprintln!("tess face: boundary {nb}, grid {} in {:?}, steps {:?}, scale {:?}, box {ubox:?} {vbox:?}", grid.len(), t_start.elapsed(), steps(&f.surf, ubox, vbox, defl, ang), (mu, mv));
    }
    // Constrained Delaunay in scaled uv, bulk loaded (exact duplicates merged first, so the
    // triangulation's vertices are our points in order).
    let tiny = |x: f64| if x.abs() < 1e-30 { 0.0 } else { x };
    let mut uniq: Vec<SPoint<f64>> = Vec::with_capacity(nb + grid.len());
    let mut seen: std::collections::HashMap<(u64, u64), usize> = std::collections::HashMap::with_capacity(nb);
    let mut handles: Vec<spade::handles::FixedVertexHandle> = Vec::with_capacity(nb + grid.len());
    for (k, p) in pts_uv.iter().chain(grid.iter()).enumerate() {
        let sp = SPoint::new(tiny(p.x * mu), tiny(p.y * mv));
        if !(sp.x.is_finite() && sp.y.is_finite()) {
            return FaceMesh::default();
        }
        // Only boundary points can repeat (the grid keeps clear of the boundary).
        let idx = if k < nb {
            *seen.entry((sp.x.to_bits(), sp.y.to_bits())).or_insert_with(|| {
                uniq.push(sp);
                uniq.len() - 1
            })
        } else {
            uniq.push(sp);
            uniq.len() - 1
        };
        handles.push(spade::handles::FixedVertexHandle::from_index(idx));
    }
    let mut cons: Vec<[usize; 2]> = Vec::new();
    for &(a, b) in &loops {
        for i in a..b {
            let j = if i + 1 == b { a } else { i + 1 };
            let (x, y) = (handles[i].index(), handles[j].index());
            if x != y {
                cons.push([x, y]);
            }
        }
    }
    let cdt: ConstrainedDelaunayTriangulation<SPoint<f64>> = match ConstrainedDelaunayTriangulation::try_bulk_load_cdt(uniq, cons, |_| {}) {
        Ok(c) => c,
        Err(_) => return FaceMesh::default(),
    };
    if prof {
        eprintln!("tess face: cdt at {:?}", t_start.elapsed());
    }
    // Inside faces by parity of constraint crossings from the hull.
    let nf = cdt.num_inner_faces();
    let mut state: Vec<i8> = vec![-1; nf + 1];
    let mut queue = std::collections::VecDeque::new();
    for face in cdt.inner_faces() {
        for e in face.adjacent_edges() {
            if e.rev().face().is_outer() {
                let idx = face.fix().index();
                let s = if e.is_constraint_edge() { 1 } else { 0 };
                if state[idx] == -1 {
                    state[idx] = s;
                    queue.push_back(face.fix());
                }
            }
        }
    }
    while let Some(fh) = queue.pop_front() {
        let face = cdt.face(fh);
        let s = state[fh.index()];
        for e in face.adjacent_edges() {
            let other = e.rev().face();
            if let Some(o) = other.as_inner() {
                let idx = o.fix().index();
                if state[idx] == -1 {
                    state[idx] = if e.is_constraint_edge() { 1 - s } else { s };
                    queue.push_back(o.fix());
                }
            }
        }
    }
    // Vertices: handle index → mesh index.
    let mut map: Vec<u32> = vec![u32::MAX; cdt.num_vertices()];
    let mut out = FaceMesh::default();
    let sign = if f.sense { 1.0 } else { -1.0 };
    let all_uv: Vec<P2> = pts_uv.iter().chain(grid.iter()).copied().collect();
    let all_3d = |k: usize| -> P3 {
        if k < nb {
            pts_3d[k]
        } else {
            let q = all_uv[k];
            f.surf.eval(q.x, q.y)
        }
    };
    // spade handle index → our point index (insertion order, duplicates collapse).
    let mut ours: Vec<usize> = vec![usize::MAX; cdt.num_vertices()];
    for (k, h) in handles.iter().enumerate() {
        if ours[h.index()] == usize::MAX {
            ours[h.index()] = k;
        }
    }
    for face in cdt.inner_faces() {
        if state[face.fix().index()] != 1 {
            continue;
        }
        let vs = face.vertices();
        let mut tri = [0u32; 3];
        for (t, v) in vs.iter().enumerate() {
            let hi = v.fix().index();
            if map[hi] == u32::MAX {
                let k = ours[hi];
                let q = all_uv[k];
                map[hi] = out.positions.len() as u32;
                out.positions.push(all_3d(k));
                out.normals.push(f.surf.normal(q.x, q.y) * sign);
            }
            tri[t] = map[hi];
        }
        if !f.sense {
            tri.swap(1, 2);
        }
        out.indices.push(tri);
    }
    out
}

/// The whole body's mesh, face by face.
pub fn mesh_body(body: &Body, defl: f64, ang: f64) -> (Vec<FaceMesh>, EdgeSamples) {
    let es = edge_samples(body, defl, ang);
    let faces: Vec<FaceMesh> = body.faces.par_iter().map(|f| mesh_face(body, f, &es, defl, ang)).collect();
    (faces, es)
}
