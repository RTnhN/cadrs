//! The solid between a convex round face and its centre, for shells as thick as the round
//! (OCCT can't thicken a face by its radius: the offset collapses onto the axis).
#[cxx::bridge(namespace = "cadrs_sectors")]
pub(super) mod ffi {
    struct Piece { brep: Vec<u8> }
    unsafe extern "C++" {
        include!("shell_sectors.hxx");
        /// Nothing unless `face` (binary BREP, oriented as in its solid) is a cylinder, sphere
        /// or torus with the body on its centre side, untrimmed in its parameters, whose radius
        /// (a torus's minor one) is at most `t`. Then the sector from the face to its axis (a
        /// sphere's centre, a torus's centre circle), and when `t` is larger, each planar side
        /// of the sector pushed out by the difference, so the walls beside it meet in a corner.
        fn convex_sector(face: &[u8], t: f64) -> Result<Vec<Piece>>;
    }
}
