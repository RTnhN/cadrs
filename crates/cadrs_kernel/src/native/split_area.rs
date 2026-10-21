//! A face divided into patches by a number of parts (OCCT's `ShapeUpgrade_ShapeDivideArea` with
//! splitting by number, behind DRAW's `DT_SplitByNumber`).
//!
//! The face's surface is cut along iso-lines into a grid: either `nu` × `nv` cells as given, or
//! cells about square with the face's area shared by the number of parts — the surface's sizes
//! across the middle (the lengths of its middle iso-curves over the face's uv bounds, the
//! bounds of a non-periodic direction widened by 1% within the surface as OCCT does) divided by
//! the side of a square of `area / parts`, rounded down, at least one each. Each cell is a face
//! of its own. Only faces that fill their uv rectangle are divided.

use super::face_build::face_from_surface;
use super::math::*;
use super::topo::Body;
use crate::{KernelError, Result};

/// The cell counts along u and v for `parts` parts of a face of `area` over the uv box
/// (`ShapeUpgrade_SplitSurfaceArea::Compute`).
/// The second count is raised to two when it is one (OCCT: the direction across the longer one
/// is always cut).
pub fn cell_counts(surf: &super::surface::Surface, (u0, u1): (f64, f64), (v0, v1): (f64, f64), area: f64, parts: usize) -> (usize, usize) {
    let (um, vm) = (0.5 * (u0 + u1), 0.5 * (v0 + v1));
    let iso_len = |f: &dyn Fn(f64) -> P3, a: f64, b: f64| {
        let n = 256;
        (0..n).map(|i| (f(a + (b - a) * (i + 1) as f64 / n as f64) - f(a + (b - a) * i as f64 / n as f64)).norm()).sum::<f64>()
    };
    let usize_ = iso_len(&|u| surf.eval(u, vm), u0, u1);
    let vsize = iso_len(&|v| surf.eval(um, v), v0, v1);
    let side = (area / parts as f64).sqrt();
    let (nu, nv) = (((usize_ / side) as usize).max(1), ((vsize / side) as usize).max(1));
    if usize_ > vsize && nv == 1 {
        (nu, 2)
    } else if usize_ <= vsize && nu == 1 {
        (2, nv)
    } else {
        (nu, nv)
    }
}

/// A uv rectangle: ((u0, u1), (v0, v1)).
pub type Cell = ((f64, f64), (f64, f64));

/// The cells face `fi` of `b` is divided into (see the module docs): `parts` parts, or `nu` ×
/// `nv` cells when given; each clipped to the face's uv box. Also whether the face fills that
/// box (else its cells must still be clipped to its boundary).
pub fn cells(b: &Body, fi: usize, parts: usize, fixed: Option<(usize, usize)>) -> Result<(Vec<Cell>, bool)> {
    let face = b.faces.get(fi).ok_or_else(|| KernelError::InvalidParameter("an unknown face".into()))?;
    let surf = &face.surf;
    // The face's uv box from its p-curves; it must be filled.
    let (mut ulo, mut uhi, mut vlo, mut vhi) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    for c in face.loops.iter().flatten() {
        let e = &b.edges[c.edge];
        for i in 0..=16 {
            let p = c.pc.point(e.t[0] + (e.t[1] - e.t[0]) * i as f64 / 16.0);
            ulo = ulo.min(p.x);
            uhi = uhi.max(p.x);
            vlo = vlo.min(p.y);
            vhi = vhi.max(p.y);
        }
    }
    if !(uhi > ulo) || !(vhi > vlo) {
        return Err(KernelError::Unsupported("dividing faces of no uv extent"));
    }
    let area = super::mass::face_area(b, fi).0;
    let box_area = {
        let n = 64;
        let mut a = 0.0;
        for i in 0..n {
            for j in 0..n {
                let (u, v) = (ulo + (uhi - ulo) * (i as f64 + 0.5) / n as f64, vlo + (vhi - vlo) * (j as f64 + 0.5) / n as f64);
                let d = surf.d2(u, v);
                a += d[1].cross(&d[2]).norm() * (uhi - ulo) * (vhi - vlo) / (n * n) as f64;
            }
        }
        a
    };
    let filled = face.loops.len() == 1 && (box_area - area).abs() <= 1e-3 * area.max(1e-12);
    // OCCT widens a bounded direction by 1% within the surface before measuring and cutting.
    let ((su0, su1), (sv0, sv1)) = surf.param_box();
    let widen = |lo: f64, hi: f64, s0: f64, s1: f64, periodic: bool| {
        if periodic {
            return (lo, hi);
        }
        let d = (hi - lo) * 0.01;
        (if lo > s0 { lo - d.min(lo - s0) } else { lo }, if hi < s1 { hi + d.min(s1 - hi) } else { hi })
    };
    let (cu0, cu1) = widen(ulo, uhi, su0, su1, surf.u_periodic());
    let (cv0, cv1) = widen(vlo, vhi, sv0, sv1, surf.v_periodic());
    let (nu, nv) = match fixed {
        Some(n) => n,
        None => cell_counts(surf, (cu0, cu1), (cv0, cv1), area, parts.max(1)),
    };
    let mut out = Vec::new();
    for i in 0..nu {
        for j in 0..nv {
            let (a0, a1) = (cu0 + (cu1 - cu0) * i as f64 / nu as f64, cu0 + (cu1 - cu0) * (i + 1) as f64 / nu as f64);
            let (b0, b1) = (cv0 + (cv1 - cv0) * j as f64 / nv as f64, cv0 + (cv1 - cv0) * (j + 1) as f64 / nv as f64);
            // The cell within the face.
            let (a0, a1, b0, b1) = (a0.max(ulo), a1.min(uhi), b0.max(vlo), b1.min(vhi));
            if a1 - a0 <= 1e-12 || b1 - b0 <= 1e-12 {
                continue;
            }
            out.push(((a0, a1), (b0, b1)));
        }
    }
    Ok((out, filled))
}

/// Divides a face that fills its uv box into its cells' faces, as one-face sheets side by side
/// in one body; None when the face doesn't fill its box.
pub fn split_face(b: &Body, fi: usize, parts: usize, fixed: Option<(usize, usize)>) -> Result<Option<Body>> {
    let (cells, filled) = cells(b, fi, parts, fixed)?;
    if !filled {
        return Ok(None);
    }
    let face = &b.faces[fi];
    let mut out = Body::default();
    for c in cells {
        let mut cell = face_from_surface(&face.surf, Some(c))?;
        if !face.sense {
            for f in &mut cell.faces {
                Body::reverse_face(f);
            }
        }
        let _ = out.append(&cell);
    }
    Ok(Some(out))
}
