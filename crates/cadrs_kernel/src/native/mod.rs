//! The native (pure Rust) B-rep kernel: exact curves and surfaces, boundary representation,
//! booleans, features, tessellation and exchange, with no C or C++ dependency.

pub mod axes;
pub mod blend;
pub mod blend_chain;
pub mod blend_corner;
pub mod bodyio;
pub mod brep_occt;
pub mod boolean;
pub mod boolean_kinds;
pub mod build;
pub mod cells;
pub mod classify;
pub mod curve;
pub mod distance;
/// The DRAW test harness the generated OCCT suites run on (a test kit, not kernel API): built
/// for tests and with the `testkit` feature.
#[cfg(any(test, feature = "testkit"))]
pub mod draw;
pub mod edge_bop;
pub mod edges_faces;
pub mod evolved;
pub mod exchange;
pub mod face_build;
pub mod features;
pub mod fillet;
pub mod fillet_geom;
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
/// Bodies for the SMLib test oracle (test kit, as `draw`).
#[cfg(any(test, feature = "testkit"))]
pub mod oracle;
pub mod silhouette;
pub mod slab;
pub mod smb;
pub mod recognize;
pub mod bvh;
pub mod offset_round;
pub mod ops;
pub mod pcurve;
pub mod profile;
pub mod split;
pub mod split_area;
pub mod ssi;
pub mod ssi_cci;
pub mod ssi_coincident;
pub mod ssi_numeric;
pub mod step;
pub mod surface;
pub mod surfacing;
pub mod sweep;
pub mod sweep_general;
pub mod sweep_path;
pub mod tails;
pub mod tess;
pub mod tol;
pub mod topo;
pub mod unify;
pub mod uvdomain;
pub mod xbuild;

pub use kernel::NativeKernel;
