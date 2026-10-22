//! Offsets of bodies and of planar wires, by stages:
//!
//! 1. **Surfaces** ([`sharp::offset_surface`]): each face's surface moved along its normal by
//!    its distance (planes, quadrics, tori, extrusions, revolutions, B-splines).
//! 2. **Sharp offsets** ([`sharp`]): the moved faces meeting as they did (a local operation that
//!    keeps the topology), prisms by their profile's 2D offset, crossed faces collapsed.
//! 3. **Rounded offsets** ([`round`]): every face's slab ([`slab`], the face swept along its
//!    normal), tubes round edges and balls round vertices, united (outward) or subtracted
//!    (inward); shells and thick solids from those.
//! 4. **Planar wires** ([`wire`]): DRAW's `mkoffset`, as OCCT's `BRepOffsetAPI_MakeOffset`.
//!
//! [`offset`] is `Kernel::offset`: sharp or rounded by the spec, with the faces' history.

pub mod round;
pub mod sharp;
pub mod slab;
pub mod wire;

use crate::native::kernel::NativeKernel;
use crate::native::surface::Surface;
use crate::*;
use crate::native::tol;

pub fn offset(k: &mut NativeKernel, body: BodyId, spec: &OffsetSpec) -> Result<OpResult> {
    let b = (**k.get(body)?).clone();
    let mut dist = vec![spec.distance; b.faces.len()];
    for (f, d) in &spec.faces {
        if let Some(x) = dist.get_mut(f.0 as usize) {
            *x = *d;
        }
    }
    // Move face (some faces moved, the rest kept): a move that would change the body's
    // topology isn't one the sharp offset makes (a hole's wall moved onto or past its axis, a
    // face run into one it doesn't border): refused cleanly rather than a wrong body.
    let moving = spec.distance == 0.0 && spec.faces.iter().any(|(_, d)| *d != 0.0);
    let too_far = |why: &str| KernelError::InvalidParameter(format!("The faces can't move that far: {why}"));
    if moving
        && spec.faces.iter().any(|(f, d)| {
            b.faces.get(f.0 as usize).is_some_and(|face| *d != 0.0 && matches!(face.surf, Surface::Cylinder { .. } | Surface::Cone { .. } | Surface::Sphere { .. } | Surface::Torus { .. }) && sharp::offset_surface(&face.surf, face.sense, *d).is_none())
        })
    {
        return Err(too_far("a face would shrink to nothing"));
    }
    let out = if spec.sharp || dist.iter().all(|d| *d == 0.0) {
        let out = sharp::offset_sharp(&b, &dist)?;
        if moving && !out.check_with(crate::native::topo::CheckLevel::Full).is_empty() {
            return Err(too_far("a face would run into another face"));
        }
        out
    } else if dist.iter().all(|d| *d <= 0.0) {
        round::inset_rounded(&b, &dist)?
    } else {
        round::offset_rounded(&b, &dist)?
    };
    let v0 = crate::native::mass::body_mass(&b).volume;
    let v1 = crate::native::mass::body_mass(&out).volume;
    let grow = dist.iter().any(|d| *d > 0.0);
    let shrink = dist.iter().any(|d| *d < 0.0);
    if (grow && !shrink && v1 < v0 - tol::ROUNDOFF) || (shrink && !grow && v1 > v0 + tol::ROUNDOFF) || v1 <= 0.0 {
        return Err(KernelError::TooLarge { op: "offset", why: "the offset turned the body inside out".into() });
    }
    // History: each face the offset of input faces (modified) or a round of an edge or vertex
    // (generated); an input face with no offset left (vanished, or inside the rounds) deleted.
    let srcs = round::offset_sources(&b, body, &out, &dist, !(spec.sharp || dist.iter().all(|d| *d == 0.0)));
    let id = k.insert(out);
    let mut history = History::default();
    let mut kept = vec![false; b.faces.len()];
    for (f, src) in srcs.iter().enumerate() {
        for s in src {
            match s {
                crate::native::blend::FaceSrc::Input(i) => {
                    kept[*i] = true;
                    history.modified.push((FaceId(f as u64), InputFace { body, face: FaceId(*i as u64) }));
                }
                crate::native::blend::FaceSrc::New(o) => history.generated.push((FaceId(f as u64), *o)),
            }
        }
    }
    history.deleted = (0..b.faces.len()).filter(|&i| !kept[i]).map(|i| InputFace { body, face: FaceId(i as u64) }).collect();
    Ok(OpResult { bodies: vec![id], history })
}

