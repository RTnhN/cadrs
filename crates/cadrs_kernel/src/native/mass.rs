//! Exact mass properties by the divergence theorem, integrated over each face's trimmed (u, v)
//! domain with Green's theorem: ∫∫_D h du dv = ∮_∂D H dv, H(u, v) = ∫_{u₀}^{u} h(s, v) ds,
//! where the boundary runs along the exact p-curves. Both integrals are composite
//! Gauss–Legendre, so analytic faces come out to rounding error.

use rayon::prelude::*;

use super::math::*;
use super::curve::Curve;
use super::surface::Surface;
use super::topo::{co_range, Body, Face};

/// Number of integrated quantities (see `integrand`).
pub const NQ: usize = 14;

pub type Q = [f64; NQ];

/// The integrand at (u, v) relative to `shift`: area terms (scaled by |N|) and volume terms
/// (by the vector N = Su × Sv; the divergence theorem turns them into volume integrals).
#[inline]
fn integrand(s: &Surface, u: f64, v: f64, shift: &V3, out: &mut Q, w: f64) {
    let (p, su, sv) = s.d1(u, v);
    let n = su.cross(&sv);
    let a = n.norm() * w;
    let (x, y, z) = (p.x - shift.x, p.y - shift.y, p.z - shift.z);
    let (nx, ny, nz) = (n.x * w, n.y * w, n.z * w);
    out[0] += a;
    out[1] += x * a;
    out[2] += y * a;
    out[3] += z * a;
    out[4] += (x * nx + y * ny + z * nz) / 3.0;
    out[5] += x * x * 0.5 * nx;
    out[6] += y * y * 0.5 * ny;
    out[7] += z * z * 0.5 * nz;
    out[8] += x * x * x / 3.0 * nx;
    out[9] += y * y * y / 3.0 * ny;
    out[10] += z * z * z / 3.0 * nz;
    out[11] += x * x * y * 0.5 * nx;
    out[12] += y * y * z * 0.5 * ny;
    out[13] += z * z * x * 0.5 * nz;
}

/// Breakpoints in u between `a` and `b` for the inner integral.
fn u_breaks(s: &Surface, a: f64, b: f64, out: &mut Vec<f64>) {
    out.clear();
    out.push(a);
    let span = (b - a).abs();
    let n = match s {
        Surface::Plane { .. } => 1,
        Surface::Nurbs(_) | Surface::Extrusion { .. } => ((span / (s_u_scale(s))).ceil() as usize).clamp(1, 64),
        _ => ((span / (PI / 8.0)).ceil() as usize).max(1),
    };
    if let Surface::Nurbs(ns) = s {
        let (bu, _) = ns.breaks();
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let mut ks: Vec<f64> = bu.into_iter().filter(|k| *k > lo + 1e-14 && *k < hi - 1e-14).collect();
        if a > b {
            ks.reverse();
        }
        let mut pts = vec![a];
        pts.extend(ks);
        pts.push(b);
        // One or two spans: split each (|N| varies within a long span); more are short already
        // (the GPU kernel's `inner_job` does the same).
        let m = if pts.len() <= 3 { 4 } else { 1 };
        for w in pts.windows(2) {
            for i in 1..=m {
                out.push(w[0] + (w[1] - w[0]) * i as f64 / m as f64);
            }
        }
        return;
    }
    if let Surface::Extrusion { c, .. } = s {
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let mut pts = c.pieces(lo, hi);
        if a > b {
            pts.reverse();
        }
        out.clear();
        out.push(a);
        for w in pts.windows(2) {
            let m = 4;
            for i in 1..=m {
                out.push(w[0] + (w[1] - w[0]) * i as f64 / m as f64);
            }
        }
        return;
    }
    for i in 1..=n {
        out.push(a + (b - a) * i as f64 / n as f64);
    }
}

fn s_u_scale(_s: &Surface) -> f64 {
    0.25
}

/// H(u, v) for all quantities, weighted by `w` (the outer quadrature weight × v'(t)).
fn inner(s: &Surface, u0: f64, u: f64, v: f64, shift: &V3, w: f64, acc: &mut Q, scratch: &mut Vec<f64>) {
    if (u - u0).abs() < 1e-300 {
        return;
    }
    u_breaks(s, u0, u, scratch);
    for k in 0..scratch.len() - 1 {
        let (lo, hi) = (scratch[k], scratch[k + 1]);
        let (m, r) = ((lo + hi) * 0.5, (hi - lo) * 0.5);
        for (x, gw) in GAUSS16.x.iter().zip(GAUSS16.w) {
            integrand(s, m + r * x, v, shift, acc, w * gw * r);
        }
    }
}

/// Surfaces whose integrand is a low-degree polynomial along v (ruled along v): their inner
/// integral runs along v, exactly with one 8-point rule.
fn ruled_in_v(s: &Surface) -> bool {
    matches!(s, Surface::Cylinder { .. } | Surface::Cone { .. } | Surface::Extrusion { .. } | Surface::Plane { .. })
}

/// G(u, v) = ∫_{v0}^{v} h(u, s) ds for all quantities, weighted by `w`.
fn inner_v(s: &Surface, u: f64, v0: f64, v: f64, shift: &V3, w: f64, acc: &mut Q) {
    if (v - v0).abs() < 1e-300 {
        return;
    }
    let (m, r) = ((v0 + v) * 0.5, (v - v0) * 0.5);
    for (x, gw) in GAUSS8.x.iter().zip(GAUSS8.w) {
        integrand(s, u, m + r * x, shift, acc, w * gw * r);
    }
}

/// Face integrals with the inner integral along v: ∫∫ h = −∮ G du.
fn face_integrals_v(body: &Body, f: &Face, shift: &V3) -> Q {
    let mut acc = [0.0; NQ];
    let mut v_ref = f64::INFINITY;
    for l in &f.loops {
        for c in l {
            let e = &body.edges[c.edge];
            v_ref = v_ref.min(c.pc.point(e.t[0]).y).min(c.pc.point(e.t[1]).y);
        }
    }
    if !v_ref.is_finite() {
        return acc;
    }
    for l in &f.loops {
        for c in l {
            let e = &body.edges[c.edge];
            let (t0, t1) = co_range(e, c.fwd);
            let (lo, hi) = (t0.min(t1), t0.max(t1));
            let mut ts = c.pc.pieces(lo, hi);
            let p0 = c.pc.point(lo);
            let p1 = c.pc.point(hi);
            let du = (p1.x - p0.x).abs();
            // Periodic: pieces of at most π/8; a general extrusion's curve pieces in four (|N|
            // varies along them).
            let extra = if f.surf.u_periodic() {
                (du / (PI / 8.0)).ceil() as usize
            } else if matches!(f.surf, Surface::Extrusion { .. }) && !matches!(c.pc, Curve::Nurbs(_)) {
                4
            } else {
                1
            }
            .max(1);
            if extra > 1 {
                let mut r = Vec::new();
                for w in ts.windows(2) {
                    for i in 0..extra {
                        r.push(w[0] + (w[1] - w[0]) * i as f64 / extra as f64);
                    }
                }
                r.push(hi);
                ts = r;
            }
            if t0 > t1 {
                ts.reverse();
            }
            let fine = matches!(c.pc, Curve::Nurbs(_)) && ts.len() > 64;
            let g = if fine { &GAUSS8 } else { &GAUSS16 };
            for w in ts.windows(2) {
                let (a, b) = (w[0], w[1]);
                let (m, r) = ((a + b) * 0.5, (b - a) * 0.5);
                for (x, gw) in g.x.iter().zip(g.w) {
                    let t = m + r * x;
                    let (uv, duv, _) = c.pc.d2(t);
                    if duv.x == 0.0 {
                        continue;
                    }
                    inner_v(&f.surf, uv.x, v_ref, uv.y, shift, -gw * r * duv.x, &mut acc);
                }
            }
        }
    }
    if !f.sense {
        for q in acc.iter_mut().take(4) {
            *q = -*q;
        }
    }
    acc
}

/// The uv rectangle of a face bounded by one loop of iso-lines (an untrimmed patch: lofts,
/// sweeps), with the loop's orientation (+1 counter-clockwise in uv).
fn iso_rectangle(body: &Body, f: &Face) -> Option<([f64; 4], f64)> {
    let [l] = f.loops.as_slice() else { return None };
    let (mut u0, mut u1, mut v0, mut v1) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    let mut twice = 0.0;
    for c in l {
        let e = &body.edges[c.edge];
        let (t0, t1) = co_range(e, c.fwd);
        let (a, b) = (c.pc.point(t0), c.pc.point(t1));
        match &c.pc {
            Curve::Line { d, .. } if d.x == 0.0 || d.y == 0.0 => {}
            Curve::Nurbs(_) => {
                // A fitted iso-line.
                let tol = 1e-12 * (1.0 + a.x.abs().max(a.y.abs()));
                let (same_u, same_v) = ((a.x - b.x).abs() < tol, (a.y - b.y).abs() < tol);
                if same_u == same_v {
                    return None;
                }
                for k in 1..8 {
                    let q = c.pc.point(t0 + (t1 - t0) * k as f64 / 8.0);
                    if (same_u && (q.x - a.x).abs() > tol) || (same_v && (q.y - a.y).abs() > tol) {
                        return None;
                    }
                }
            }
            _ => return None,
        }
        for p in [a, b] {
            u0 = u0.min(p.x);
            u1 = u1.max(p.x);
            v0 = v0.min(p.y);
            v1 = v1.max(p.y);
        }
        twice += a.x * b.y - b.x * a.y;
    }
    let area = (u1 - u0) * (v1 - v0);
    if !(area > 0.0) || ((twice.abs() * 0.5) - area).abs() > 1e-9 * area {
        return None;
    }
    Some(([u0, u1, v0, v1], twice.signum()))
}

/// The face integrals over an iso-rectangle of a B-spline surface: a tensor Gauss rule on each
/// knot span (the integrand is polynomial along polynomial directions, smooth along rational
/// ones).
fn rect_integrals(s: &Surface, ns: &super::nurbs::NurbsSurface, r: [f64; 4], shift: &V3) -> Q {
    let mut acc = [0.0; NQ];
    let (bu, bv) = ns.breaks();
    let cut = |ks: Vec<f64>, a: f64, b: f64| -> Vec<f64> {
        let mut pts = vec![a];
        pts.extend(ks.into_iter().filter(|k| *k > a + 1e-14 && *k < b - 1e-14));
        pts.push(b);
        // One or two spans: split each (|N| varies within a span); more are short already.
        let m = if pts.len() <= 3 { 4 } else { 1 };
        let mut out = vec![a];
        for w in pts.windows(2) {
            for i in 1..=m {
                out.push(w[0] + (w[1] - w[0]) * i as f64 / m as f64);
            }
        }
        out
    };
    let vs = cut(bv, r[2], r[3]);
    // The area integrand |N| is only smooth: 16 points a span; across many short spans 8.
    let gv = if vs.len() > 17 { &GAUSS8 } else { &GAUSS16 };
    // Along v the points and weights; along u the inner integral (as Green's theorem's jobs, so
    // a GPU can take them).
    let mut jobs: Vec<[f64; 4]> = Vec::with_capacity((vs.len() - 1) * gv.x.len());
    for wv in vs.windows(2) {
        let (mv, rv) = ((wv[0] + wv[1]) * 0.5, (wv[1] - wv[0]) * 0.5);
        for (y, hw) in gv.x.iter().zip(gv.w) {
            jobs.push([r[0], r[1], mv + rv * y, hw * rv]);
        }
    }
    let _ = bu;
    let mut scratch = Vec::with_capacity(64);
    run_jobs(s, &jobs, shift, &mut acc, &mut scratch);
    acc
}

/// The 14 face integrals of a face, relative to `shift`, oriented by its sense.
pub fn face_integrals(body: &Body, f: &Face, shift: &V3) -> Q {
    if ruled_in_v(&f.surf) && !matches!(f.surf, Surface::Plane { .. }) {
        return face_integrals_v(body, f, shift);
    }
    if let Surface::Nurbs(ns) = &f.surf {
        if ns.du <= 3 && ns.dv <= 3 {
            if let Some((r, orient)) = iso_rectangle(body, f) {
                let mut acc = rect_integrals(&f.surf, ns, r, shift);
                for q in acc.iter_mut() {
                    *q *= orient;
                }
                if !f.sense {
                    for q in acc.iter_mut().take(4) {
                        *q = -*q;
                    }
                }
                return acc;
            }
        }
    }
    let mut acc = [0.0; NQ];
    let mut scratch = Vec::with_capacity(64);
    // The reference u: the smallest u on the boundary keeps H small.
    let mut u_ref = f64::INFINITY;
    for l in &f.loops {
        for c in l {
            let e = &body.edges[c.edge];
            u_ref = u_ref.min(c.pc.point(e.t[0]).x).min(c.pc.point(e.t[1]).x);
        }
    }
    if !u_ref.is_finite() {
        return acc;
    }
    let v_periodic = f.surf.v_periodic() || matches!(f.surf, Surface::Sphere { .. });
    // B-spline faces gather their inner integrals as jobs (run on the GPU when there are many).
    let mut jobs: Option<Vec<[f64; 4]>> = matches!(f.surf, Surface::Nurbs(_)).then(Vec::new);
    for l in &f.loops {
        for c in l {
            let e = &body.edges[c.edge];
            let (t0, t1) = co_range(e, c.fwd);
            // Pieces along the p-curve.
            let (lo, hi) = (t0.min(t1), t0.max(t1));
            let mut ts = c.pc.pieces(lo, hi);
            // Refine where u or v sweep far on periodic surfaces.
            let p0 = c.pc.point(lo);
            let p1 = c.pc.point(hi);
            let du = (p1.x - p0.x).abs();
            let dv = (p1.y - p0.y).abs();
            let extra = if f.surf.u_periodic() || v_periodic {
                ((du.max(if v_periodic { dv } else { 0.0 })) / (PI / 8.0)).ceil() as usize
            } else {
                1
            }
            .max(1);
            if extra > 1 {
                let mut r = Vec::new();
                for w in ts.windows(2) {
                    for i in 0..extra {
                        r.push(w[0] + (w[1] - w[0]) * i as f64 / extra as f64);
                    }
                }
                r.push(hi);
                ts = r;
            }
            if t0 > t1 {
                ts.reverse();
            }
            let fine = matches!(c.pc, Curve::Nurbs(_)) && ts.len() > 64;
            let g = if fine { &GAUSS8 } else { &GAUSS16 };
            for w in ts.windows(2) {
                let (a, b) = (w[0], w[1]);
                let (m, r) = ((a + b) * 0.5, (b - a) * 0.5);
                for (x, gw) in g.x.iter().zip(g.w) {
                    let t = m + r * x;
                    let (uv, duv, _) = c.pc.d2(t);
                    let dv = duv.y;
                    if dv == 0.0 {
                        continue;
                    }
                    if let Some(jobs) = jobs.as_mut() {
                        jobs.push([u_ref, uv.x, uv.y, gw * r * dv]);
                    } else {
                        inner(&f.surf, u_ref, uv.x, uv.y, shift, gw * r * dv, &mut acc, &mut scratch);
                    }
                }
            }
        }
    }
    if let Some(jobs) = jobs {
        run_jobs(&f.surf, &jobs, shift, &mut acc, &mut scratch);
    }
    if !f.sense {
        for q in acc.iter_mut().take(4) {
            *q = -*q;
        }
    }
    acc
}

/// Jobs below this many stay on the CPU (a dispatch costs about a millisecond).
#[cfg(feature = "gpu")]
const GPU_MIN_JOBS: usize = 512;

static GPU_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Turns the GPU path off or back on (for comparisons; it is on when the kernel is built with
/// the `gpu` feature and a GPU is there).
pub fn set_gpu(on: bool) {
    GPU_ON.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// Whether mass properties may use the GPU now.
pub fn gpu_active() -> bool {
    #[cfg(feature = "gpu")]
    {
        GPU_ON.load(std::sync::atomic::Ordering::Relaxed) && cadrs_gpu::get().is_some()
    }
    #[cfg(not(feature = "gpu"))]
    {
        false
    }
}

/// The inner integrals `[u0, u, v, w]` of a face, summed into `acc`: on the GPU when the kernel
/// is built with it, one is there and there are enough jobs; else here.
fn run_jobs(s: &Surface, jobs: &[[f64; 4]], shift: &V3, acc: &mut Q, scratch: &mut Vec<f64>) {
    #[cfg(feature = "gpu")]
    if jobs.len() >= GPU_MIN_JOBS && GPU_ON.load(std::sync::atomic::Ordering::Relaxed) {
        if let (Surface::Nurbs(ns), Some(gpu)) = (s, cadrs_gpu::get()) {
            if ns.du <= 3 && ns.dv <= 3 {
                let (nu, nv) = (ns.cps.len(), ns.cps[0].len());
                let ctrl: Vec<f64> = ns.cps.iter().flatten().flat_map(|h| [h.x, h.y, h.z, h.w]).collect();
                let mut consts = vec![shift.x, shift.y, shift.z];
                consts.extend_from_slice(&GAUSS16.x);
                consts.extend_from_slice(&GAUSS16.w);
                let q = cadrs_gpu::QuadParams { nu: nu as u32, nv: nv as u32, du: ns.du as u32, dv: ns.dv as u32, nku: ns.ku.len() as u32, nkv: ns.kv.len() as u32, jobs: 0, _pad: 0 };
                for r in gpu.quad_inner(q, &ctrl, &ns.ku, &ns.kv, &consts, jobs) {
                    for (a, x) in acc.iter_mut().zip(r) {
                        *a += x;
                    }
                }
                return;
            }
        }
    }
    for j in jobs {
        inner(s, j[0], j[1], j[2], shift, j[3], acc, scratch);
    }
}

/// Mass properties of a body (solids; a body without solids is a sheet: area only).
pub fn body_mass(body: &Body) -> crate::MassProperties {
    let bb = body.bbox();
    let shift = if bb.is_empty() { V3::zeros() } else { bb.center().coords };
    let faces: Vec<usize> = (0..body.faces.len()).collect();
    let qs: Vec<Q> = faces.par_iter().map(|&i| face_integrals(body, &body.faces[i], &shift)).collect();
    let mut q = [0.0; NQ];
    for f in &qs {
        for k in 0..NQ {
            q[k] += f[k];
        }
    }
    let area = q[0];
    let solid = body.is_solid();
    let vol = if solid { q[4] } else { 0.0 };
    let (center, inertia) = if solid && vol.abs() > 1e-300 {
        let c = V3::new(q[5], q[6], q[7]) / vol;
        let (xx, yy, zz) = (q[8], q[9], q[10]);
        let (xy, yz, zx) = (q[11], q[12], q[13]);
        // About the shift origin.
        let io = nalgebra::Matrix3::new(yy + zz, -xy, -zx, -xy, xx + zz, -yz, -zx, -yz, xx + yy);
        let ic = io - (nalgebra::Matrix3::identity() * c.norm_squared() - c * c.transpose()) * vol;
        (P3::from(c + shift), ic)
    } else if area.abs() > 1e-300 {
        (P3::from(V3::new(q[1], q[2], q[3]) / area + shift), nalgebra::Matrix3::zeros())
    } else {
        (P3::from(shift), nalgebra::Matrix3::zeros())
    };
    crate::MassProperties { volume: vol, surface_area: area, center_of_mass: center, inertia }
}

/// A face's area and area centroid.
pub fn face_area(body: &Body, fi: usize) -> (f64, P3) {
    let f = &body.faces[fi];
    let mut bb = Bbox::empty();
    for l in &f.loops {
        for c in l {
            let e = &body.edges[c.edge];
            bb.add(&body.verts[e.v[0]].p);
            if !e.degenerate {
                bb.add(&e.point(e.mid_t()));
            }
        }
    }
    let shift = if bb.is_empty() { V3::zeros() } else { bb.center().coords };
    let q = face_integrals(body, f, &shift);
    let a = q[0];
    let c = if a.abs() > 1e-300 { V3::new(q[1], q[2], q[3]) / a + shift } else { shift };
    (a, P3::from(c))
}
