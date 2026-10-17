//! Exact mass properties by the divergence theorem, integrated over each face's trimmed (u, v)
//! domain with Green's theorem: ∫∫_D h du dv = ∮_∂D H dv, H(u, v) = ∫_{u₀}^{u} h(s, v) ds,
//! where the boundary runs along the exact p-curves. Both integrals are composite
//! Gauss–Legendre, so analytic faces come out to rounding error.

use rayon::prelude::*;

use super::math::*;
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
        for w in pts.windows(2) {
            let m = 4;
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

/// The 14 face integrals of a face, relative to `shift`, oriented by its sense.
pub fn face_integrals(body: &Body, f: &Face, shift: &V3) -> Q {
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
            for w in ts.windows(2) {
                let (a, b) = (w[0], w[1]);
                let (m, r) = ((a + b) * 0.5, (b - a) * 0.5);
                for (x, gw) in GAUSS16.x.iter().zip(GAUSS16.w) {
                    let t = m + r * x;
                    let (uv, duv, _) = c.pc.d2(t);
                    let dv = duv.y;
                    if dv == 0.0 {
                        continue;
                    }
                    inner(&f.surf, u_ref, uv.x, uv.y, shift, gw * r * dv, &mut acc, &mut scratch);
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
