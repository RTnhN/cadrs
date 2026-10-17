//! The native (pure Rust) B-rep kernel: exact curves and surfaces, boundary representation,
//! booleans, features, tessellation and exchange, with no C or C++ dependency.

pub mod boolean;
pub mod build;
pub mod classify;
pub mod curve;
pub mod features;
pub mod kernel;
pub mod mass;
pub mod math;
pub mod nurbs;
pub mod profile;
pub mod surface;
pub mod tess;
pub mod topo;

pub use kernel::NativeKernel;
