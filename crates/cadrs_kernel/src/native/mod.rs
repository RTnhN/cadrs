//! The native (pure Rust) B-rep kernel: exact curves and surfaces, boundary representation,
//! booleans, features, tessellation and exchange, with no C or C++ dependency.

pub mod blend;
pub mod blend_chain;
pub mod blend_corner;
pub mod bodyio;
pub mod brep_occt;
pub mod boolean;
pub mod build;
pub mod classify;
pub mod curve;
pub mod draw;
pub mod edge_bop;
pub mod exchange;
pub mod features;
pub mod hlr;
pub mod iges;
pub mod kernel;
pub mod local;
pub mod loft_build;
pub mod mass;
pub mod math;
pub mod meshsolid;
pub mod nurbs;
pub mod offset2d;
pub mod offset_round;
pub mod ops;
pub mod pcurve;
pub mod profile;
pub mod split;
pub mod ssi;
pub mod step;
pub mod surface;
pub mod surfacing;
pub mod sweep;
pub mod sweep_general;
pub mod sweep_path;
pub mod tess;
pub mod topo;
pub mod unify;
pub mod xbuild;

pub use kernel::NativeKernel;
