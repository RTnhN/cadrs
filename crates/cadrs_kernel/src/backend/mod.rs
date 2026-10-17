//! Kernel backends. `native` is the pure-Rust kernel; `occt` (feature "occt") wraps OpenCascade.

#[cfg(feature = "occt")]
pub mod occt;

pub mod native {
    pub use crate::native::NativeKernel;
}
