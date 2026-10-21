//! The kernel's tolerance model: the distances under which two things are taken as one, named
//! once here and carried explicitly (in [`Tolerances`], held by the kernel session) instead of
//! as literals and globals.
//!
//! Lengths are in mm. Three levels, each ten times the one below:
//! - [`CONFUSION`] (1e-7): exact geometry. Two points closer are the same point; an entity's own
//!   tolerance ([`super::topo::Vertex::tol`], [`super::topo::Edge::tol`]) never goes below it
//!   (OCCT's `Precision::Confusion`).
//! - [`POINT`] (1e-6): what an operation merges. A boolean pools vertices closer than this and
//!   puts edge ends on edges within it; intersections computed numerically land within it. A
//!   body's [`super::topo::Body::check`] accepts an edge end or a p-curve this far off when the
//!   entity's own tolerance is smaller.
//! - [`LOOP_GAP`] (1e-5): a p-curve fitted to an edge may miss the next coedge's start by this
//!   much (in model space, through the surface's metric), which `check` allows at a vertex of
//!   smaller tolerance.
//!
//! Above these, entities carry their own tolerance: a read file's or a healed body's vertices
//! and edges hold the gaps actually there ([`super::topo::Body::fit_tolerances`], as OCCT), and
//! operations take the larger of their own and the entities'.
//!
//! Models lie within [`SIZE`] of the origin: beyond it a double no longer resolves
//! [`CONFUSION`] (2⁻⁵² · 1e8 ≈ 2e-8 mm). Readers refuse files with geometry outside.
//!
//! A boolean whose first run leaves an open shell runs again at the [`Tolerances::retries`]
//! (1e-5, then 1e-4), as OCCT grows its tolerances; the result's entities record the tolerance
//! the run used.
//!
//! Dimensionless epsilons (relative to a size, on parameters, for normalising vectors) are not
//! lengths and stay with the code that uses them.

/// Exact geometry: points closer than this are one point (mm). See the module docs.
pub const CONFUSION: f64 = 1e-7;
/// The default merge distance of an operation (mm). See the module docs.
pub const POINT: f64 = 1e-6;
/// The largest gap between consecutive p-curves of a loop that is still closed (mm).
pub const LOOP_GAP: f64 = 1e-5;
/// The size box: coordinates and radii of a model stay within ±`SIZE` (mm, 100 km).
pub const SIZE: f64 = 1e8;

/// The tolerances an operation runs with, carried by the kernel session
/// ([`super::NativeKernel::tolerances`]) and passed explicitly to the operations that take
/// them. `Default` is the model above.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tolerances {
    /// The merge distance of a boolean's first run ([`POINT`]).
    pub point: f64,
    /// The merge distances of the runs after a first that leaves an open shell, in order.
    pub retries: [f64; 2],
    /// The smallest tolerance an entity holds ([`CONFUSION`]).
    pub confusion: f64,
}

impl Default for Tolerances {
    fn default() -> Self {
        // (Written out: 10.0 * POINT isn't exactly 1e-5 in floating point.)
        Self { point: POINT, retries: [LOOP_GAP, 1e-4], confusion: CONFUSION }
    }
}

impl Tolerances {
    /// Checks that the tolerances are usable: finite, positive and growing
    /// (`confusion <= point <= retries`).
    pub fn validate(&self) -> crate::Result<()> {
        let all = [self.confusion, self.point, self.retries[0], self.retries[1]];
        if all.iter().any(|t| !t.is_finite() || *t <= 0.0) || all.windows(2).any(|w| w[1] < w[0]) {
            return Err(crate::KernelError::InvalidParameter(format!("tolerances must be positive and growing: {self:?}")));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The defaults are the documented levels, and the constants the kernel used before
    /// (`topo::TOL_MIN`, `boolean::VTOL`, the boolean's retries) are these.
    #[test]
    fn defaults_are_the_model() {
        let t = Tolerances::default();
        assert_eq!((t.confusion, t.point, t.retries), (1e-7, 1e-6, [1e-5, 1e-4]));
        assert_eq!(super::super::topo::TOL_MIN, CONFUSION);
        assert_eq!(super::super::boolean::VTOL, POINT);
        assert!(t.validate().is_ok());
        assert!(Tolerances { point: f64::NAN, ..t }.validate().is_err());
        assert!(Tolerances { point: 1e-3, ..t }.validate().is_err());
        assert!(Tolerances { confusion: 0.0, ..t }.validate().is_err());
    }

    /// The session holds its tolerances, refusing unusable ones.
    #[test]
    fn session_tolerances() {
        let mut k = super::super::NativeKernel::new();
        assert_eq!(k.tolerances(), Tolerances::default());
        assert!(k.set_tolerances(Tolerances { point: -1.0, ..Tolerances::default() }).is_err());
        assert_eq!(k.tolerances(), Tolerances::default());
        k.set_tolerances(Tolerances { point: 2e-6, ..Tolerances::default() }).unwrap();
        assert_eq!(k.tolerances().point, 2e-6);
    }
}
