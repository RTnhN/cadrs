//! Blends from models that failed in the app (their input bodies dumped with `CADRS_DUMP_DIR`).

use cadrs_kernel::native::*;
use cadrs_kernel::*;

/// A 2 mm chamfer along a round cut's rim, which runs over the top face, down two 5 mm fillets
/// and the side face: the contacts cross from the fillets onto the flat faces, and the sections
/// on either side of each crossing must agree (the boolean left an open shell).
#[test]
fn chamfer_over_fillets() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/chamfer_over_fillets.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let edges = [14, 16, 22, 25, 26].map(EdgeId);
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    // About the chain's length (176 mm) times the section (2 mm²), less where it bends.
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((320.0..352.0).contains(&removed), "{removed}");
}
