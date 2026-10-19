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
    // The chamfer ends flush with the round cut's floor (z = 15.5) instead of digging into it:
    // the floor only gains the chamfer's two end sections (under 2 mm² each).
    let floor = |id: BodyId| -> f64 {
        k.faces(id).unwrap().iter().filter(|f| f.kind == SurfaceKind::Plane && (f.center.z - 15.5).abs() < 1e-6).map(|f| f.area).sum()
    };
    let gained = floor(out) - floor(b);
    assert!(gained > 0.0 && gained < 4.0, "floor area {} -> {}", floor(b), floor(out));
}

/// The same model's top arc alone (Tangent propagation off): the chamfer stops square at the
/// arc's ends, where the rim runs on smoothly over the fillets, instead of running on past them.
#[test]
fn chamfer_stops_where_the_chain_runs_on() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/chamfer_over_fillets.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let r = k.chamfer_with(b, &[EdgeId(22)], &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    // The arc (103.5 mm) times the section (2 mm², about 3% more for the section's offset from
    // the round cut's axis).
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((207.0..216.0).contains(&removed), "{removed}");
}

/// A 2 mm chamfer round a block's top: an outline with a notch (two inside corners) and a
/// rectangular hole (four more). Each pair of chamfers meets on the plane bisecting its corner:
/// a mitre at the convex corners, a valley at the inside ones. Exact volume: the section
/// (2 mm²) along the outline offset d/3 into the material (2d/3 shorter at a convex corner,
/// longer at an inside one); and no faces but the 14 the block had and the 12 chamfers.
#[test]
fn chamfer_inside_corners() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/chamfer_inside_corners.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let edges = [2, 6, 9, 12, 15, 18, 21, 23, 26, 30, 33, 35].map(EdgeId);
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((removed - 855.168388).abs() < 1e-4, "{removed}");
    assert_eq!(k.faces(out).unwrap().len(), 14 + 12);
}
