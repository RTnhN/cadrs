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
pub mod pcurve;
pub mod profile;
pub mod split;
pub mod ssi;
pub mod surface;
pub mod sweep;
pub mod tess;
pub mod topo;
pub mod unify;

pub use kernel::NativeKernel;
