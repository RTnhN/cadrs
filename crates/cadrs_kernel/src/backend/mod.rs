//! Kernel backends: `native`, the pure-Rust B-rep kernel.

/// The native (pure Rust) kernel: [`NativeKernel`](crate::native::NativeKernel).
pub mod native {
    pub use crate::native::NativeKernel;
}
