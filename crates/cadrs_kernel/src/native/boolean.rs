//! Booleans (union, subtract, intersect) of B-rep solids.

use super::kernel::NativeKernel;
use crate::*;

pub fn boolean(_k: &mut NativeKernel, _op: BoolOp, _target: BodyId, _tools: &[BodyId]) -> Result<OpResult> {
    Err(KernelError::Unsupported("booleans"))
}
