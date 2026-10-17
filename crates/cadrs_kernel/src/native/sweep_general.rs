//! Sweeps along general path curves (B-splines, ellipses, curves out of a plane) with sampled
//! frames: the profile is placed at frames along the path and the sections are lofted.

use super::features::Tagged;
use super::math::*;
use super::profile::{MergedFace, Piece};
use super::sweep_path::Seg;
use crate::*;

pub fn sweep_general(_faces: &[MergedFace], _chains: &[Vec<Piece>], _plane: Plane, _spine: &[Seg], _control: SweepControl, _solid: bool) -> Result<Tagged> {
    let _ = V3::zeros();
    Err(KernelError::Unsupported("sweeping along this path"))
}
