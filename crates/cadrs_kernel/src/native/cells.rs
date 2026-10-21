//! Cells of an arrangement of solids (OCCT's `BOPAlgo_CellsBuilder` and `BOPAlgo_MakerVolume`,
//! behind DRAW's `bcbuild`/`bcadd` and `mkvolume`), on the kernel's booleans.
//!
//! A cell is a connected solid every argument either contains or misses: the arguments split
//! one another into cells. `bcadd`'s selections (in these arguments, out of those) are unions
//! of cells; `mkvolume`'s result is every bounded cell of the space its (unbounded) surfaces
//! divide, each surface given as the region on one side of it. `bsplit` splits objects by
//! tools.

use super::kernel::NativeKernel;
use crate::*;

/// The boolean of two bodies; None when it is empty.
fn bop(k: &mut NativeKernel, op: BoolOp, a: BodyId, b: BodyId) -> Result<Option<BodyId>> {
    match crate::Kernel::boolean(k, op, a, &[b]) {
        Ok(r) if r.bodies.is_empty() => Ok(None),
        Ok(r) => {
            let mut acc = r.bodies[0];
            for &x in &r.bodies[1..] {
                acc = crate::Kernel::boolean(k, BoolOp::Union, acc, &[x])?.bodies[0];
            }
            Ok(Some(acc))
        }
        Err(KernelError::EmptyResult(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

/// A body's connected solids, each a body of its own.
fn pieces(k: &mut NativeKernel, b: BodyId) -> Result<Vec<BodyId>> {
    let body = (**k.get(b)?).clone();
    let parts = super::exchange::split_into_solids(&body);
    if parts.len() <= 1 {
        return Ok(vec![b]);
    }
    Ok(parts.into_iter().map(|p| k.insert(p)).collect())
}

/// The cells `start` is split into by `regions`: each region's inside and outside of every
/// cell so far, connected pieces apart.
pub fn split_cells(k: &mut NativeKernel, start: BodyId, regions: &[BodyId]) -> Result<Vec<BodyId>> {
    let mut cells = vec![start];
    for &r in regions {
        let mut next = Vec::new();
        for &c in &cells {
            for op in [BoolOp::Intersect, BoolOp::Subtract] {
                if let Some(p) = bop(k, op, c, r)? {
                    next.extend(pieces(k, p)?);
                }
            }
        }
        cells = next;
    }
    Ok(cells)
}

/// The cells of `args` inside the arguments `spec` marks true and outside those it marks false
/// (the others either way). Cells outside every argument don't count.
pub fn select_cells(k: &mut NativeKernel, args: &[BodyId], spec: &[(usize, bool)]) -> Result<Vec<BodyId>> {
    let free: Vec<usize> = (0..args.len()).filter(|i| !spec.iter().any(|(j, _)| j == i)).collect();
    if free.len() > 12 {
        return Err(KernelError::Unsupported("cells of more than twelve free arguments"));
    }
    let mut out = Vec::new();
    for mask in 0u32..(1 << free.len()) {
        let mut inside: Vec<usize> = spec.iter().filter(|(_, b)| *b).map(|(i, _)| *i).collect();
        let mut outside: Vec<usize> = spec.iter().filter(|(_, b)| !*b).map(|(i, _)| *i).collect();
        for (bit, &i) in free.iter().enumerate() {
            if mask & (1 << bit) != 0 {
                inside.push(i);
            } else {
                outside.push(i);
            }
        }
        let Some((&first, rest)) = inside.split_first() else { continue };
        let mut cell = Some(args[first]);
        for &i in rest {
            cell = match cell {
                Some(c) => bop(k, BoolOp::Intersect, c, args[i])?,
                None => None,
            };
        }
        for &i in &outside {
            cell = match cell {
                Some(c) => bop(k, BoolOp::Subtract, c, args[i])?,
                None => None,
            };
        }
        if let Some(c) = cell {
            out.extend(pieces(k, c)?);
        }
    }
    Ok(out)
}

/// The cells of `cells` that stay clear of the box `outer`'s faces (by `margin`): the bounded
/// ones when `outer` stands for all of space.
pub fn bounded(k: &NativeKernel, cells: &[BodyId], outer: &Aabb, margin: f64) -> Result<Vec<BodyId>> {
    let mut out = Vec::new();
    for &c in cells {
        let bb = crate::Kernel::bounding_box(k, c)?;
        if (0..3).all(|i| bb.min[i] > outer.min[i] + margin && bb.max[i] < outer.max[i] - margin) {
            out.push(c);
        }
    }
    Ok(out)
}

/// `bsplit`'s split (BOPAlgo_Splitter): each object cut by every tool, solids by their
/// boundaries into inside and outside pieces, sheets by the kernel's split; the pieces.
pub fn split_by(k: &mut NativeKernel, objects: &[BodyId], tools: &[BodyId]) -> Result<Vec<BodyId>> {
    let mut out = Vec::new();
    for &o in objects {
        let mut parts = vec![o];
        for &t in tools {
            let solid_tool = !k.get(t)?.solids.is_empty();
            let mut next = Vec::new();
            for &p in &parts {
                if solid_tool {
                    next.extend(split_cells(k, p, &[t])?);
                } else {
                    // A one-face sheet splits as that face.
                    let tool = if k.get(t)?.faces.len() == 1 { SplitTool::Face { body: t, face: FaceId(0) } } else { SplitTool::Body(t) };
                    match crate::Kernel::split(k, p, &tool, 0) {
                        Ok(r) if !r.bodies.is_empty() => {
                            for b in r.bodies {
                                next.extend(pieces(k, b)?);
                            }
                        }
                        // The tool misses the piece.
                        Ok(_) | Err(KernelError::EmptyResult(_)) => next.push(p),
                        Err(e) => return Err(e),
                    }
                }
            }
            parts = next;
        }
        out.extend(parts);
    }
    Ok(out)
}
