//! The native (pure Rust) B-rep kernel: exact curves and surfaces, boundary representation,
//! booleans, features, tessellation and exchange, with no C or C++ dependency.

pub mod bodyio;
pub mod boolean;
pub mod build;
pub mod classify;
pub mod curve;
pub mod exchange;
pub mod features;
pub mod iges;
pub mod kernel;
pub mod mass;
pub mod math;
pub mod meshsolid;
pub mod nurbs;
pub mod ops;
pub mod pcurve;
pub mod profile;
pub mod split;
pub mod ssi;
pub mod step;
pub mod surface;
pub mod sweep;
pub mod tess;
pub mod topo;
pub mod unify;
pub mod xbuild;

pub use kernel::NativeKernel;
