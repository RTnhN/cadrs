//! The full extrude and revolve (ends, directions, thin and surface bodies).

use super::build;
use super::kernel::NativeKernel;
use super::math::*;
use super::profile;
use crate::*;

/// How far along `dir` (from the sketch plane) an end reaches.
fn reach(k: &NativeKernel, profile: &Profile, dir: &V3, end: &ExtrudeEnd, scene: &[BodyId]) -> Result<f64> {
    let o = profile.plane.origin;
    let n = profile.plane.normal.into_inner();
    let kdn = dir.dot(&n);
    match *end {
        ExtrudeEnd::Blind(d) => Ok(d),
        ExtrudeEnd::UpToVertex { point, offset } => Ok((point - o).dot(&n) / kdn - offset),
        ExtrudeEnd::ThroughAll => {
            if scene.is_empty() {
                return Err(KernelError::InvalidParameter("Through all needs other parts to go through".into()));
            }
            let mut far = f64::NEG_INFINITY;
            for &b in scene {
                let (lo, hi) = k.extent_along(b, &o, &n)?;
                let (a, c) = if kdn > 0.0 { (lo / kdn, hi / kdn) } else { (hi / kdn, lo / kdn) };
                let _ = a;
                far = far.max(c);
            }
            if far <= 0.0 {
                return Err(KernelError::InvalidParameter("Through all: no part lies ahead of the sketch".into()));
            }
            Ok(far)
        }
        ExtrudeEnd::UpToFace { body, face, offset } => {
            let info = k.faces(body)?.into_iter().find(|f| f.id == face).ok_or_else(|| KernelError::InvalidParameter("unknown face".into()))?;
            match info.plane {
                Some(pl) if pl.normal.cross(&n).norm() < 1e-9 => {
                    let l = (pl.origin - o).dot(&n) / kdn;
                    if l <= 0.0 {
                        return Err(KernelError::InvalidParameter("The face lies behind the sketch plane".into()));
                    }
                    Ok(l - offset)
                }
                _ => Err(KernelError::Unsupported("extrude up to a non-parallel face")),
            }
        }
        _ => Err(KernelError::Unsupported("extrude up to next / part")),
    }
}

pub fn extrude_with(k: &mut NativeKernel, profile: &Profile, spec: &ExtrudeSpec) -> Result<OpResult> {
    let dir = spec.direction.into_inner();
    let n = profile.plane.normal.into_inner();
    if dir.dot(&n).abs() < 1e-9 {
        return Err(KernelError::InvalidParameter("The extrude direction lies in the sketch plane".into()));
    }
    if spec.body != BodyKind::Solid || !spec.faces.is_empty() {
        return Err(KernelError::Unsupported("surface, thin and face extrudes"));
    }
    let (s0, s1) = if spec.symmetric {
        match spec.end {
            ExtrudeEnd::Blind(d) => (-d.abs() / 2.0, d.abs() / 2.0),
            ExtrudeEnd::ThroughAll => {
                let a = reach(k, profile, &dir, &ExtrudeEnd::ThroughAll, &spec.scene)?;
                let b = reach(k, profile, &-dir, &ExtrudeEnd::ThroughAll, &spec.scene)?;
                (-b, a)
            }
            _ => return Err(KernelError::InvalidParameter("Symmetric needs a blind depth or Through all".into())),
        }
    } else {
        let s1 = reach(k, profile, &dir, &spec.end, &spec.scene)?;
        let s0 = match &spec.second {
            Some(e) => -reach(k, profile, &-dir, e, &spec.scene)?,
            None => spec.start_offset,
        };
        (s0, s1)
    };
    let (dir, s0, s1) = if s1 >= s0 { (dir, s0, s1) } else { (-dir, -s0, -s1) };
    let regions = profile::regions(profile)?;
    if regions.is_empty() {
        return Err(KernelError::InvalidProfile("no regions".into()));
    }
    let faces = profile::merge_regions(&regions)?;
    let b = build::prism(&profile.plane, &faces, dir, s0, s1)?;
    Ok(k.insert_result(b))
}

pub fn revolve_with(_k: &mut NativeKernel, _profile: &Profile, _spec: &RevolveSpec) -> Result<OpResult> {
    Err(KernelError::Unsupported("revolve"))
}
