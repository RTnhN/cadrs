//! The full extrude and revolve (ends, directions, thin and surface bodies).

use super::sweep;
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
    let kind = sweep::Prism { plane: profile.plane, dir };
    let b = sweep::sweep(&profile.plane, &faces, &kind, s0, s1, false)?;
    Ok(k.insert_result(b))
}

/// The revolve frame: z along the axis, x from the axis towards the profile (which must lie on
/// one side of it, in a plane through it).
pub fn revolve_frame(profile: &Profile, regions: &[profile::Region2], axis: &Axis) -> Result<Frame> {
    let n = profile.plane.normal.into_inner();
    let z = axis.dir.into_inner();
    if z.dot(&n).abs() > 1e-9 {
        return Err(KernelError::InvalidParameter("The revolve axis must lie in the sketch plane".into()));
    }
    if (axis.origin - profile.plane.origin).dot(&n).abs() > 1e-6 {
        return Err(KernelError::InvalidParameter("The revolve axis must lie in the sketch plane".into()));
    }
    let x = z.cross(&n).normalize();
    let m = build_map(profile);
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for r in regions {
        for l in std::iter::once(&r.outer).chain(r.holes.iter()) {
            for p in l {
                for q in p.sample(1e-3) {
                    let w = m.point(&P3::new(q.x, q.y, 0.0));
                    let d = (w - axis.origin).dot(&x);
                    lo = lo.min(d);
                    hi = hi.max(d);
                }
            }
        }
    }
    let x = if hi > 1e-6 && lo < -1e-6 {
        return Err(KernelError::InvalidParameter("The profile crosses the revolve axis".into()));
    } else if hi > 1e-6 {
        x
    } else {
        -x
    };
    Ok(Frame { o: axis.origin, x, y: z.cross(&x), z })
}

fn build_map(profile: &Profile) -> Affine {
    super::build::plane_map(&profile.plane, &V3::zeros(), 0.0)
}

pub fn revolve_with(k: &mut NativeKernel, profile: &Profile, spec: &RevolveSpec) -> Result<OpResult> {
    if spec.body != BodyKind::Solid || !spec.faces.is_empty() {
        return Err(KernelError::Unsupported("surface, thin and face revolves"));
    }
    let regions = profile::regions(profile)?;
    if regions.is_empty() {
        return Err(KernelError::InvalidProfile("no regions".into()));
    }
    let f = revolve_frame(profile, &regions, &spec.axis)?;
    let angle = |e: &RevolveEnd| -> Result<f64> {
        match *e {
            RevolveEnd::Angle(a) => Ok(a),
            RevolveEnd::UpToVertex { point, offset } => {
                let l = f.local(&point);
                Ok(wrap(l.y.atan2(l.x), 0.0, TAU) - offset)
            }
            _ => Err(KernelError::Unsupported("revolve up to next / face / part")),
        }
    };
    let (s0, s1, whole) = if spec.full {
        (0.0, TAU, true)
    } else if spec.symmetric {
        let a = angle(&spec.end)?.abs();
        (-a / 2.0, a / 2.0, false)
    } else {
        let a1 = angle(&spec.end)?;
        let a0 = match &spec.second {
            Some(e) => -angle(e)?,
            None => 0.0,
        };
        let (a0, a1) = if a1 >= a0 { (a0, a1) } else { (a1, a0) };
        (a0, a1, false)
    };
    if s1 - s0 > TAU + 1e-9 {
        return Err(KernelError::InvalidParameter("The revolve turns more than a whole turn".into()));
    }
    let whole = whole || (s1 - s0) >= TAU - 1e-12;
    let faces = profile::merge_regions(&regions)?;
    let kind = sweep::Revolve { plane: profile.plane, f };
    let b = sweep::sweep(&profile.plane, &faces, &kind, s0, if whole { s0 + TAU } else { s1 }, whole)?;
    Ok(k.insert_result(b))
}
