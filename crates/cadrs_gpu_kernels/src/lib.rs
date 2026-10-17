//! GPU feasibility spike for the native B-rep kernel: two representative workloads in plain
//! `no_std` Rust that the CPU calls directly and rust-gpu compiles to SPIR-V.
//!
//! - [`nurbs_point_normal`]: a rational B-spline surface of degree 3×3 at one `(u, v)`, point
//!   and unit normal (tessellation, sampling). Generic over [`Real`] so the same source runs in
//!   `f32` and `f64`.
//! - [`classify_point`]: inside/outside of a closed triangle mesh by ray parity along +X, with
//!   the triangles bucketed in a uniform grid over `(y, z)` (CSR: `cell_start`, `cell_tris`).
//!
//! Everything is flat slices of scalars and `while` loops so the SPIR-V stays simple.

#![no_std]

// The kernels only use `core`; on the host std is linked for the `dylib` crate type rust-gpu
// needs (its panic handler).
#[cfg(not(target_arch = "spirv"))]
extern crate std;

use core::ops::{Add, Div, Mul, Neg, Sub};

/// The scalar the kernels are written in (`f32` or `f64`).
pub trait Real: Copy + PartialOrd + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Div<Output = Self> + Neg<Output = Self> {
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
pub fn nurbs_point_normal<R: Real>(p: &SurfParams, ctrl: &[R], ku: &[R], kv: &[R], u: R, v: R) -> [R; 6] {
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
    let su = [au[0] - au[3] * s[0], au[1] - au[3] * s[1], au[2] - au[3] * s[2]];
    let sv = [av[0] - av[3] * s[0], av[1] - av[3] * s[1], av[2] - av[3] * s[2]];
    let n = [su[1] * sv[2] - su[2] * sv[1], su[2] * sv[0] - su[0] * sv[2], su[0] * sv[1] - su[1] * sv[0]];
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
    let cy = if fy < 0.0 { 0 } else if fy >= g.gy as f32 { g.gy - 1 } else { fy as u32 };
    let cz = if fz < 0.0 { 0 } else if fz >= g.gz as f32 { g.gz - 1 } else { fz as u32 };
    (cy, cz)
}

/// Whether `(x, y, z)` is inside the closed mesh: the parity of triangles the ray along +X
/// crosses. `tris` holds 9 floats per triangle; the point's `(y, z)` cell lists the triangles
/// whose `(y, z)` bounds overlap it. Outside the grid's bounds the point is outside.
pub fn classify_point(g: &GridParams, tris: &[f32], cell_start: &[u32], cell_tris: &[u32], x: f32, y: f32, z: f32) -> u32 {
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
    out[i as usize] = classify_point(g, tris, cell_start, cell_tris, pts[b], pts[b + 1], pts[b + 2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::prelude::rust_2024::*;

    /// The basis functions sum to one and their derivatives to zero.
    #[test]
    fn partition_of_unity() {
        let k = [0.0f64, 0.0, 0.0, 0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 1.0, 1.0, 1.0];
        for i in 0..=100 {
            let t = i as f64 / 100.0;
            let s = find_span(&k, 8, t);
            let (n, d) = basis(&k, s, t);
            assert!((n.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            assert!(d.iter().sum::<f64>().abs() < 1e-9);
        }
    }
}
