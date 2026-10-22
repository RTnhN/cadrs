//! The native (pure Rust) B-rep kernel: exact curves and surfaces, boundary representation,
//! booleans, features, tessellation and exchange, with no C or C++ dependency.
//!
//! The stable surface (the README's "Curated API") is what this module re-exports: the session
//! [`NativeKernel`]; the B-rep data model ([`Body`] with its [`Vertex`], [`Edge`], [`Coedge`],
//! [`Face`], [`Shell`], [`Solid`] and [`Wire`], checked at a [`CheckLevel`]); its geometry
//! ([`Curve`], [`Surface`] and its derivatives [`D2`], [`NurbsCurve`], [`NurbsSurface`]) and
//! vector types ([`P3`], [`V3`], [`Frame`], [`Affine`]); the tolerance model [`tol`]; and the
//! exact body format [`bodyio`]. That is what a direct-edit or analysis tool built on the
//! kernel needs.
//!
//! Every other module is the kernel's internals: private to the crate, and public only with the
//! `testkit` feature, which the crate's own tests and tools turn on (they reach the internals
//! through `cadrs_kernel::testkit`, or through `native::<module>` while they move there).

/// Declares the kernel's internal modules: `pub(crate)`, or `pub` with the `testkit` feature.
/// Without it, what only the test kit reaches (the DRAW harness's commands: the OCCT BRep and
/// SMLib readers, 2D offsets, tails, faces from edges, ...) is unused; with it (the crate's
/// tests and clippy gate) dead code is found as before.
macro_rules! internal {
    ($($(#[$attr:meta])* $name:ident;)*) => {$(
        $(#[$attr])*
        #[cfg(feature = "testkit")]
        pub mod $name;
        $(#[$attr])*
        #[cfg(not(feature = "testkit"))]
        #[allow(dead_code, reason = "items only the test kit uses")]
        pub(crate) mod $name;
    )*};
}

pub mod bodyio;
pub mod tol;

internal! {
    axes;
    blend;
    blend_chain;
    blend_corner;
    boolean;
    boolean_kinds;
    brep_occt;
    build;
    bvh;
    cells;
    classify;
    curve;
    distance;
    edge_bop;
    edges_faces;
    evolved;
    exchange;
    face_build;
    features;
    fillet;
    fillet_geom;
    hlr;
    iges;
    kernel;
    local;
    loft_build;
    mass;
    math;
    meshsolid;
    nurbs;
    offset2d;
    offset_round;
    ops;
    pcurve;
    profile;
    recognize;
    silhouette;
    slab;
    smb;
    split;
    split_area;
    ssi;
    ssi_cci;
    ssi_coincident;
    ssi_numeric;
    step;
    surface;
    surfacing;
    sweep;
    tails;
    tess;
    topo;
    unify;
    uvdomain;
    xbuild;
}

/// The DRAW test harness the generated OCCT suites run on (a test kit, not kernel API): built
/// for tests and with the `testkit` feature.
#[cfg(any(test, feature = "testkit"))]
pub mod draw;
/// Bodies for the SMLib test oracle (test kit, as `draw`).
#[cfg(any(test, feature = "testkit"))]
pub mod oracle;

pub use curve::Curve;
pub use kernel::NativeKernel;
pub use math::{Affine, Frame, P3, V3};
pub use nurbs::{NurbsCurve, NurbsSurface};
pub use surface::{Surface, D2};
pub use topo::{Body, CheckLevel, Coedge, Edge, Face, Shell, Solid, Vertex, Wire};
