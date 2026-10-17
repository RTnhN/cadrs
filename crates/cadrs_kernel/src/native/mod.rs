//! The native (pure Rust) B-rep kernel: exact curves and surfaces, boundary representation,
//! booleans, features, tessellation and exchange, with no C or C++ dependency.

pub mod blend;
pub mod boolean;
pub mod build;
pub mod classify;
pub mod curve;
pub mod features;
pub mod hlr;
pub mod kernel;
pub mod local;
pub mod mass;
pub mod math;
pub mod nurbs;
pub mod offset_round;
pub mod ops;
pub mod pcurve;
pub mod profile;
pub mod split;
pub mod ssi;
pub mod surface;
pub mod sweep;
pub mod sweep_general;
pub mod sweep_path;
pub mod tess;
pub mod topo;
pub mod unify;

pub use kernel::NativeKernel;
