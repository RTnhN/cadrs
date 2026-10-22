//! GPU feasibility spike for the native B-rep kernel: two representative workloads in plain
//! `no_std` Rust that the CPU calls directly and rust-gpu compiles to SPIR-V.
//!
//! - [`nurbs_point_normal`]: a rational B-spline surface of degree 3×3 at one `(u, v)`, point
//!   and unit normal (tessellation, sampling). Generic over [`Real`] so the same source runs in
//!   `f32` and `f64`.
//! - [`classify_point`]: inside/outside of a closed triangle mesh by ray parity along +X, with
//!   the triangles bucketed in a uniform grid over `(y, z)` (CSR: `cell_start`, `cell_tris`).
//!
//! - [`inner_job`]: the kernel's mass-property quadrature on a B-spline face, the inner
//!   integral of Green's theorem at one boundary point (degrees 1 to 3, rational).
//!
//! Everything is flat slices of scalars and `while` loops so the SPIR-V stays simple.

#![no_std]

// The kernels only use `core`; on the host std is linked for the `dylib` crate type rust-gpu
// needs (its panic handler).
#[cfg(not(target_arch = "spirv"))]
extern crate std;

use core::ops::{Add, Div, Mul, Neg, Sub};

/// The scalar the kernels are written in (`f32` or `f64`).
pub trait Real:
    Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
    fn from_f32(x: f32) -> Self;
    fn sqrt(self) -> Self;
}

impl Real for f32 {
    fn zero() -> f32 {
        0.0
    }
    fn one() -> f32 {
        1.0
    }
    fn from_f32(x: f32) -> f32 {
        x
    }
    #[cfg(target_arch = "spirv")]
    fn sqrt(self) -> f32 {
        spirv_std::num_traits::Float::sqrt(self)
    }
    #[cfg(not(target_arch = "spirv"))]
    fn sqrt(self) -> f32 {
        libm::sqrtf(self)
    }
}

impl Real for f64 {
    fn zero() -> f64 {
        0.0
    }
    fn one() -> f64 {
        1.0
    }
    fn from_f32(x: f32) -> f64 {
        x as f64
    }
    #[cfg(target_arch = "spirv")]
    fn sqrt(self) -> f64 {
        spirv_std::num_traits::Float::sqrt(self)
    }
    #[cfg(not(target_arch = "spirv"))]
    fn sqrt(self) -> f64 {
        libm::sqrt(self)
    }
}

/// Surface degree in both directions.
pub const DEG: usize = 3;

/// The surface and its sampling grid, as the shader reads them (a uniform: `u32`s only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct SurfParams {
    /// Control points in `u` and `v` (the net is `nu × nv`, `u` fastest).
    pub nu: u32,
    pub nv: u32,
    /// Samples in `u` and `v` (each spans `[0, 1]`, end points included).
    pub su: u32,
    pub sv: u32,
}

/// The knot span holding `t` (`k[span] <= t < k[span + 1]`, the last span at `t = 1`).
pub fn find_span<R: Real>(k: &[R], n: usize, t: R) -> usize {
    let mut s = DEG;
    while s + 1 < n && t >= k[s + 1] {
        s += 1;
    }
    s
}

/// The four non-zero cubic basis functions at `t` in `span` and their first derivatives
/// (Piegl & Tiller A2.2, keeping the quadratic row for the derivative).
pub fn basis<R: Real>(k: &[R], span: usize, t: R) -> ([R; 4], [R; 4]) {
    let z = R::zero();
    let mut n = [R::one(), z, z, z];
    let mut left = [z; 4];
    let mut right = [z; 4];
    let mut quad = [z; 3];
    let mut j = 1;
    while j <= DEG {
        left[j] = t - k[span + 1 - j];
        right[j] = k[span + j] - t;
        let mut saved = z;
        let mut r = 0;
        while r < j {
            let den = right[r + 1] + left[j - r];
            let tmp = if den > z { n[r] / den } else { z };
            n[r] = saved + right[r + 1] * tmp;
            saved = left[j - r] * tmp;
            r += 1;
        }
        n[j] = saved;
        if j == DEG - 1 {
            quad = [n[0], n[1], n[2]];
        }
        j += 1;
    }
    // d/dt N_{i,3} = 3 (N_{i,2} / (k_{i+3} - k_i) - N_{i+1,2} / (k_{i+4} - k_{i+1})), i = span-3+m.
    let p = R::from_f32(DEG as f32);
    let mut d = [z; 4];
    let mut m = 0;
    while m < 4 {
        let i = span + m - DEG;
        let mut v = z;
        if m >= 1 {
            let den = k[i + DEG] - k[i];
            if den > z {
                v = v + quad[m - 1] / den;
            }
        }
        if m < 3 {
            let den = k[i + DEG + 1] - k[i + 1];
            if den > z {
                v = v - quad[m] / den;
            }
        }
        d[m] = p * v;
        m += 1;
    }
    (n, d)
}

/// The surface's point and unit normal at `(u, v)`: `ctrl` holds homogeneous control points
/// `(w x, w y, w z, w)`, `u` fastest; `ku`, `kv` the clamped knot vectors (`n + 4` each).
pub fn nurbs_point_normal<R: Real>(
    p: &SurfParams,
    ctrl: &[R],
    ku: &[R],
    kv: &[R],
    u: R,
    v: R,
) -> [R; 6] {
    let nu = p.nu as usize;
    let nv = p.nv as usize;
    let spu = find_span(ku, nu, u);
    let spv = find_span(kv, nv, v);
    let (bu, du) = basis(ku, spu, u);
    let (bv, dv) = basis(kv, spv, v);
    let z = R::zero();
    // Homogeneous sums: A (point), Au, Av (partials), each (x, y, z, w).
    let mut a = [z; 4];
    let mut au = [z; 4];
    let mut av = [z; 4];
    let mut j = 0;
    while j < 4 {
        let row = spv - DEG + j;
        let mut i = 0;
        while i < 4 {
            let col = spu - DEG + i;
            let base = (row * nu + col) * 4;
            let w0 = bu[i] * bv[j];
            let wu = du[i] * bv[j];
            let wv = bu[i] * dv[j];
            let mut c = 0;
            while c < 4 {
                let q = ctrl[base + c];
                a[c] = a[c] + w0 * q;
                au[c] = au[c] + wu * q;
                av[c] = av[c] + wv * q;
                c += 1;
            }
            i += 1;
        }
        j += 1;
    }
    let iw = R::one() / a[3];
    let s = [a[0] * iw, a[1] * iw, a[2] * iw];
    // S_u = (A_u - w_u S) / w; the 1/w factor doesn't change the normal's direction.
    let su = [
        au[0] - au[3] * s[0],
        au[1] - au[3] * s[1],
        au[2] - au[3] * s[2],
    ];
    let sv = [
        av[0] - av[3] * s[0],
        av[1] - av[3] * s[1],
        av[2] - av[3] * s[2],
    ];
    let n = [
        su[1] * sv[2] - su[2] * sv[1],
        su[2] * sv[0] - su[0] * sv[2],
        su[0] * sv[1] - su[1] * sv[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    let il = if len > z { R::one() / len } else { z };
    [s[0], s[1], s[2], n[0] * il, n[1] * il, n[2] * il]
}

/// Sample `idx` of the grid (`u` fastest): its point and normal.
pub fn nurbs_sample<R: Real>(p: &SurfParams, ctrl: &[R], ku: &[R], kv: &[R], idx: u32) -> [R; 6] {
    let iu = idx % p.su;
    let iv = idx / p.su;
    let u = R::from_f32(iu as f32) / R::from_f32((p.su - 1) as f32);
    let v = R::from_f32(iv as f32) / R::from_f32((p.sv - 1) as f32);
    nurbs_point_normal(p, ctrl, ku, kv, u, v)
}

/// The mesh's `(y, z)` grid, as the shader reads it (a uniform: 4-byte words only).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct GridParams {
    /// Cells across `y` and `z`.
    pub gy: u32,
    pub gz: u32,
    /// Query points.
    pub points: u32,
    pub _pad: u32,
    /// The grid's lower corner and inverse cell size in `(y, z)`.
    pub y0: f32,
    pub z0: f32,
    pub inv_dy: f32,
    pub inv_dz: f32,
}

/// The grid cell of `(y, z)`, clamped into the grid.
pub fn cell_of(g: &GridParams, y: f32, z: f32) -> (u32, u32) {
    let fy = (y - g.y0) * g.inv_dy;
    let fz = (z - g.z0) * g.inv_dz;
    let cy = if fy < 0.0 {
        0
    } else if fy >= g.gy as f32 {
        g.gy - 1
    } else {
        fy as u32
    };
    let cz = if fz < 0.0 {
        0
    } else if fz >= g.gz as f32 {
        g.gz - 1
    } else {
        fz as u32
    };
    (cy, cz)
}

/// Whether `(x, y, z)` is inside the closed mesh: the parity of triangles the ray along +X
/// crosses. `tris` holds 9 floats per triangle; the point's `(y, z)` cell lists the triangles
/// whose `(y, z)` bounds overlap it. Outside the grid's bounds the point is outside.
pub fn classify_point(
    g: &GridParams,
    tris: &[f32],
    cell_start: &[u32],
    cell_tris: &[u32],
    x: f32,
    y: f32,
    z: f32,
) -> u32 {
    let (cy, cz) = cell_of(g, y, z);
    let cell = (cz * g.gy + cy) as usize;
    let mut k = cell_start[cell];
    let end = cell_start[cell + 1];
    let mut parity = 0u32;
    while k < end {
        let t = (cell_tris[k as usize] * 9) as usize;
        k += 1;
        let (ay, az) = (tris[t + 1], tris[t + 2]);
        let (by, bz) = (tris[t + 4], tris[t + 5]);
        let (cy, cz) = (tris[t + 7], tris[t + 8]);
        // Edge functions of the point against the triangle's (y, z) projection.
        let e0 = (by - y) * (cz - z) - (bz - z) * (cy - y);
        let e1 = (cy - y) * (az - z) - (cz - z) * (ay - y);
        let e2 = (ay - y) * (bz - z) - (az - z) * (by - y);
        let inside = (e0 > 0.0 && e1 > 0.0 && e2 > 0.0) || (e0 < 0.0 && e1 < 0.0 && e2 < 0.0);
        if inside {
            let hx = (e0 * tris[t] + e1 * tris[t + 3] + e2 * tris[t + 6]) / (e0 + e1 + e2);
            if hx > x {
                parity ^= 1;
            }
        }
    }
    parity
}

// ---------------------------------------------------------------------------------------------
// Mass-property quadrature on B-spline faces (the kernel's `mass` module, Green's theorem):
// for each boundary quadrature point (u, v) of a face, the inner integral along u from the
// face's reference u, ∫ h(s, v) ds, of the 14 integrated quantities.

/// The surface and the job count, as the shader reads them (a uniform: `u32`s only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(C)]
pub struct QuadParams {
    /// Control points in u and v (`ctrl` is u-major: `(i * nv + j) * 4`, homogeneous).
    pub nu: u32,
    pub nv: u32,
    /// Degrees (1 to 3) and knot counts.
    pub du: u32,
    pub dv: u32,
    pub nku: u32,
    pub nkv: u32,
    pub jobs: u32,
    pub _pad: u32,
}

/// Number of quantities per job (the kernel's `mass::NQ`).
pub const NQ: usize = 14;

/// The knot span of `t` for degree `p` with `n` control points.
pub fn find_span_p<R: Real>(k: &[R], n: usize, p: usize, t: R) -> usize {
    let mut s = p;
    while s + 1 < n && t >= k[s + 1] {
        s += 1;
    }
    s
}

/// The `p + 1` (p ≤ 3) non-zero basis functions of degree `p` at `t` in `span` and their first
/// derivatives (Piegl & Tiller A2.2, the degree p − 1 row kept for the derivatives).
pub fn basis_p<R: Real>(k: &[R], span: usize, t: R, p: usize) -> ([R; 4], [R; 4]) {
    let z = R::zero();
    let mut n = [R::one(), z, z, z];
    let mut lower = [R::one(), z, z, z];
    let mut left = [z; 4];
    let mut right = [z; 4];
    let mut j = 1;
    while j <= p {
        if j == p {
            lower = n;
        }
        left[j] = t - k[span + 1 - j];
        right[j] = k[span + j] - t;
        let mut saved = z;
        let mut r = 0;
        while r < j {
            let den = right[r + 1] + left[j - r];
            let tmp = if den > z { n[r] / den } else { z };
            n[r] = saved + right[r + 1] * tmp;
            saved = left[j - r] * tmp;
            r += 1;
        }
        n[j] = saved;
        j += 1;
    }
    let pr = R::from_f32(p as f32);
    let mut d = [z; 4];
    let mut m = 0;
    while m <= p {
        let i = span + m - p;
        let mut v = z;
        if m >= 1 {
            let den = k[i + p] - k[i];
            if den > z {
                v = v + lower[m - 1] / den;
            }
        }
        if m < p {
            let den = k[i + p + 1] - k[i + 1];
            if den > z {
                v = v - lower[m] / den;
            }
        }
        d[m] = pr * v;
        m += 1;
    }
    (n, d)
}

/// The surface's point and first partial derivatives at (u, v): `[S, S_u, S_v]`, 9 values.
pub fn surface_d1<R: Real>(q: &QuadParams, ctrl: &[R], ku: &[R], kv: &[R], u: R, v: R) -> [R; 9] {
    let (nu, nv, du, dv) = (q.nu as usize, q.nv as usize, q.du as usize, q.dv as usize);
    let su = find_span_p(ku, nu, du, u);
    let sv = find_span_p(kv, nv, dv, v);
    let (bu, dbu) = basis_p(ku, su, u, du);
    let (bv, dbv) = basis_p(kv, sv, v, dv);
    let z = R::zero();
    let mut a = [z; 4];
    let mut au = [z; 4];
    let mut av = [z; 4];
    let mut i = 0;
    while i <= du {
        let col = su - du + i;
        let mut j = 0;
        while j <= dv {
            let row = sv - dv + j;
            let base = (col * nv + row) * 4;
            let w0 = bu[i] * bv[j];
            let wu = dbu[i] * bv[j];
            let wv = bu[i] * dbv[j];
            let mut c = 0;
            while c < 4 {
                let h = ctrl[base + c];
                a[c] = a[c] + w0 * h;
                au[c] = au[c] + wu * h;
                av[c] = av[c] + wv * h;
                c += 1;
            }
            j += 1;
        }
        i += 1;
    }
    let iw = R::one() / a[3];
    let s = [a[0] * iw, a[1] * iw, a[2] * iw];
    [
        s[0],
        s[1],
        s[2],
        (au[0] - au[3] * s[0]) * iw,
        (au[1] - au[3] * s[1]) * iw,
        (au[2] - au[3] * s[2]) * iw,
        (av[0] - av[3] * s[0]) * iw,
        (av[1] - av[3] * s[1]) * iw,
        (av[2] - av[3] * s[2]) * iw,
    ]
}

/// Adds the integrand at (u, v), weighted by `w`, to `out` (the kernel's `mass::integrand`).
pub fn integrand<R: Real>(
    q: &QuadParams,
    ctrl: &[R],
    ku: &[R],
    kv: &[R],
    shift: [R; 3],
    u: R,
    v: R,
    w: R,
    out: &mut [R; NQ],
) {
    let d = surface_d1(q, ctrl, ku, kv, u, v);
    let (su, sv) = ([d[3], d[4], d[5]], [d[6], d[7], d[8]]);
    let n = [
        su[1] * sv[2] - su[2] * sv[1],
        su[2] * sv[0] - su[0] * sv[2],
        su[0] * sv[1] - su[1] * sv[0],
    ];
    let a = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() * w;
    let (x, y, z) = (d[0] - shift[0], d[1] - shift[1], d[2] - shift[2]);
    let (nx, ny, nz) = (n[0] * w, n[1] * w, n[2] * w);
    let half = R::one() / R::from_f32(2.0);
    let third = R::one() / R::from_f32(3.0);
    out[0] = out[0] + a;
    out[1] = out[1] + x * a;
    out[2] = out[2] + y * a;
    out[3] = out[3] + z * a;
    out[4] = out[4] + (x * nx + y * ny + z * nz) * third;
    out[5] = out[5] + x * x * half * nx;
    out[6] = out[6] + y * y * half * ny;
    out[7] = out[7] + z * z * half * nz;
    out[8] = out[8] + x * x * x * third * nx;
    out[9] = out[9] + y * y * y * third * ny;
    out[10] = out[10] + z * z * z * third * nz;
    out[11] = out[11] + x * x * y * half * nx;
    out[12] = out[12] + y * y * z * half * ny;
    out[13] = out[13] + z * z * x * half * nz;
}

/// One job: the inner integral along u from `u0` to `u1` at `v`, weighted by `w`, over the
/// distinct knots between them (each span in four when there are one or two, 16-point
/// Gauss–Legendre on each piece).
/// `consts`: the shift (3), then the Gauss nodes (16) and weights (16).
pub fn inner_job<R: Real>(
    q: &QuadParams,
    ctrl: &[R],
    ku: &[R],
    kv: &[R],
    consts: &[R],
    u0: R,
    u1: R,
    v: R,
    w: R,
) -> [R; NQ] {
    let z = R::zero();
    let mut acc = [z; NQ];
    let d = u1 - u0;
    let ad = if d < z { -d } else { d };
    if !(ad > R::from_f32(1e-30)) {
        return acc;
    }
    let shift = [consts[0], consts[1], consts[2]];
    let asc = u1 > u0;
    let (lo, hi) = if asc { (u0, u1) } else { (u1, u0) };
    let eps = R::from_f32(1e-14);
    let nk = q.nku as usize;
    // Distinct knots inside: one or two spans are split in four (as the kernel's `u_breaks`).
    let mut inside = 0;
    let mut seen = lo;
    let mut c = 0;
    while c < nk {
        let kk = ku[c];
        c += 1;
        if kk > lo + eps && kk < hi - eps {
            let diff = kk - seen;
            let ad = if diff < z { -diff } else { diff };
            if !(ad < eps && seen != lo) {
                inside += 1;
                seen = kk;
            }
        }
    }
    let parts = if inside <= 1 { 4 } else { 1 };
    let mut prev = u0;
    let mut last_knot = lo;
    let mut idx = 0;
    // Visit the knots strictly inside (lo, hi) in the direction of travel, each value once.
    while idx <= nk {
        let (kk, at_end) = if idx == nk {
            (u1, true)
        } else {
            let kk = if asc { ku[idx] } else { ku[nk - 1 - idx] };
            (kk, false)
        };
        idx += 1;
        if !at_end {
            if !(kk > lo + eps && kk < hi - eps) {
                continue;
            }
            let diff = kk - last_knot;
            let ad = if diff < z { -diff } else { diff };
            if ad < eps && last_knot != lo {
                continue;
            }
            last_knot = kk;
        }
        // The piece from prev to kk (in four for one or two spans).
        let mut m = 0;
        while m < parts {
            let a = prev + (kk - prev) * R::from_f32(m as f32) / R::from_f32(parts as f32);
            let b = prev + (kk - prev) * R::from_f32((m + 1) as f32) / R::from_f32(parts as f32);
            let mid = (a + b) / R::from_f32(2.0);
            let r = (b - a) / R::from_f32(2.0);
            let mut g = 0;
            while g < 16 {
                integrand(
                    q,
                    ctrl,
                    ku,
                    kv,
                    shift,
                    mid + r * consts[3 + g],
                    v,
                    w * consts[19 + g] * r,
                    &mut acc,
                );
                g += 1;
            }
            m += 1;
        }
        prev = kk;
    }
    acc
}

#[cfg(target_arch = "spirv")]
use spirv_std::{glam::UVec3, spirv};

/// SPIR-V entry `nurbs_f32`: one invocation per sample (`id.x + id.y * 65535 * 64`), 6 floats
/// out each (point, normal).
#[cfg(target_arch = "spirv")]
#[spirv(compute(threads(64)))]
pub fn nurbs_f32(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(num_workgroups)] groups: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] p: &SurfParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] ctrl: &[f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] ku: &[f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] kv: &[f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] out: &mut [f32],
) {
    let idx = id.x + id.y * groups.x * 64;
    if idx >= p.su * p.sv {
        return;
    }
    let r = nurbs_sample(p, ctrl, ku, kv, idx);
    let o = (idx * 6) as usize;
    let mut c = 0;
    while c < 6 {
        out[o + c] = r[c];
        c += 1;
    }
}

/// SPIR-V entry `nurbs_f64`: [`nurbs_f32`] in double precision (needs `Float64`; built only
/// with the `f64` feature, into a module of its own, so the f32 module needs no f64 support).
#[cfg(all(target_arch = "spirv", feature = "f64"))]
#[spirv(compute(threads(64)))]
pub fn nurbs_f64(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(num_workgroups)] groups: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] p: &SurfParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] ctrl: &[f64],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] ku: &[f64],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] kv: &[f64],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] out: &mut [f64],
) {
    let idx = id.x + id.y * groups.x * 64;
    if idx >= p.su * p.sv {
        return;
    }
    let r = nurbs_sample(p, ctrl, ku, kv, idx);
    let o = (idx * 6) as usize;
    let mut c = 0;
    while c < 6 {
        out[o + c] = r[c];
        c += 1;
    }
}

/// SPIR-V entry `classify`: one invocation per query point (`pts`: 4 floats each, `x y z _`),
/// writes 1 (inside) or 0.
#[cfg(target_arch = "spirv")]
#[spirv(compute(threads(64)))]
pub fn classify(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(num_workgroups)] groups: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] g: &GridParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] tris: &[f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] cell_start: &[u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] cell_tris: &[u32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] pts: &[f32],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] out: &mut [u32],
) {
    let i = id.x + id.y * groups.x * 64;
    if i >= g.points {
        return;
    }
    let b = (i * 4) as usize;
    out[i as usize] = classify_point(
        g,
        tris,
        cell_start,
        cell_tris,
        pts[b],
        pts[b + 1],
        pts[b + 2],
    );
}

/// SPIR-V entry `quad_inner_f64`: one invocation per job (`jobs`: 4 doubles each, `u0 u1 v w`),
/// 14 doubles out each (see [`inner_job`]).
#[cfg(all(target_arch = "spirv", feature = "f64"))]
#[spirv(compute(threads(64)))]
pub fn quad_inner_f64(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(num_workgroups)] groups: UVec3,
    #[spirv(uniform, descriptor_set = 0, binding = 0)] q: &QuadParams,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] ctrl: &[f64],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] ku: &[f64],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] kv: &[f64],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 4)] consts: &[f64],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 5)] jobs: &[f64],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 6)] out: &mut [f64],
) {
    let i = id.x + id.y * groups.x * 64;
    if i >= q.jobs {
        return;
    }
    let b = (i * 4) as usize;
    let r = inner_job(
        q,
        ctrl,
        ku,
        kv,
        consts,
        jobs[b],
        jobs[b + 1],
        jobs[b + 2],
        jobs[b + 3],
    );
    let o = (i as usize) * NQ;
    let mut c = 0;
    while c < NQ {
        out[o + c] = r[c];
        c += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::prelude::rust_2024::*;

    /// Every degree's basis functions sum to one and their derivatives to zero.
    #[test]
    fn partition_of_unity_any_degree() {
        for p in 1..=3usize {
            let mut k = std::vec![0.0f64; p + 1];
            k.extend([0.2, 0.45, 0.7]);
            k.extend(std::vec![1.0; p + 1]);
            let n = k.len() - p - 1;
            for i in 0..=100 {
                let t = i as f64 / 100.0;
                let s = find_span_p(&k, n, p, t);
                let (b, d) = basis_p(&k, s, t, p);
                assert!(
                    (b[..=p].iter().sum::<f64>() - 1.0).abs() < 1e-12,
                    "p {p} t {t}"
                );
                assert!(d[..=p].iter().sum::<f64>().abs() < 1e-9, "p {p} t {t}");
            }
        }
    }

    /// The basis functions sum to one and their derivatives to zero.
    #[test]
    fn partition_of_unity() {
        let k = [
            0.0f64, 0.0, 0.0, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.0, 1.0, 1.0,
        ];
        for i in 0..=100 {
            let t = i as f64 / 100.0;
            let s = find_span(&k, 8, t);
            let (n, d) = basis(&k, s, t);
            assert!((n.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            assert!(d.iter().sum::<f64>().abs() < 1e-9);
        }
    }
}
