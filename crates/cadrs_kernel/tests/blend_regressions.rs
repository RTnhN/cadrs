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

/// The same block chamfered on its notch wall all round (its top and bottom edges cut, the
/// vertical inside corner filled), a top edge and two hole edges. Where the fill meets a cut,
/// the cut's plane runs on over the fill's end; a chamfer ending against an unchamfered wall
/// ends flush, its end part of that wall. No faces but the block's 14 and the 7 chamfers.
#[test]
fn chamfer_mixed_corners() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/chamfer_mixed_corners.body")).unwrap());
    let edges = [26, 35, 8, 10, 11, 12, 9].map(EdgeId);
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    assert_eq!(k.faces(out).unwrap().len(), 14 + 7);
    for (one, faces) in [(&[11][..], 15), (&[12, 11][..], 16), (&[10, 11, 12][..], 17), (&[26][..], 15)] {
        let ids: Vec<EdgeId> = one.iter().map(|&i| EdgeId(i)).collect();
        let r = k.chamfer_with(b, &ids, &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
        assert_eq!(k.faces(r.bodies[0]).unwrap().len(), faces, "{one:?}");
    }
}

/// A wheel's five spoke windows chamfered 2 mm on both sides (a user's document): each side
/// face is split in two on its plane by a circle the windows' chamfers cross, and the edge list
/// holds an edge between two tangent cylinders. The tangent edge is passed over (as OCCT does),
/// the rails run on across the split onto the plane's other face, and where a window's outer
/// arc lies wholly past the split its strip starts on that face (the crossings of the split
/// circle are found however long it is).
#[test]
fn wheel_window_chamfer() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/docs/wheel_window_chamfer.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let edges = [1, 2, 28, 29, 30, 31, 32, 33, 67, 68, 69, 70, 71, 72, 101, 102, 34, 35, 36, 37, 38, 39, 73, 74, 75, 76, 77, 78, 107, 108, 10, 11, 12, 13, 14, 15, 49, 50, 51, 52, 53, 54, 84, 85, 16, 17, 18, 19, 20, 21, 55, 56, 57, 58, 59, 60, 92, 93, 22, 23, 24, 25, 26, 27, 61, 62, 63, 64, 65, 66, 98, 99].map(EdgeId);
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    // Ten windows of about 71 mm round, a 2 mm² section.
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((1350.0..1500.0).contains(&removed), "{removed}");
}

/// A pipe snap's edges chamfered 0.5 mm (a user's document), one of them where a cross hole
/// meets the bore: the faces' angle varies along it, so its chamfer is a law of radii (the
/// boolean tool's result wasn't valid).
#[test]
fn pipe_snap_chamfer() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/docs/pipe_snap_chamfer.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let edges = [4, 45, 46, 26, 2, 7, 10, 13, 16, 19, 32, 35, 38, 41, 43, 22, 25, 29, 0, 5, 8, 11, 14, 17, 30, 33, 36, 39, 42, 20, 23, 27, 24, 28, 31, 15, 18, 21].map(EdgeId);
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(0.5))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    // OCCT takes 62.45 mm³ off.
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((60.0..65.0).contains(&removed), "{removed}");
}

/// The rims of a cross hole through a D-shaped block filleted 5 mm (a user's document): the
/// hole's axis lies where the block's half cylinder meets its flat sides, so the ball rolls
/// from the cylinder onto the planes. A crossing on the cylinder's far side (its surface goes
/// on round past the face) isn't a roll, nor one whose spine parameter isn't a number (the
/// ball rolled back and forth for ever).
#[test]
fn cross_hole_fillet() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/docs/cross_hole_fillet.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let spec = FilletSpec { allow_overflow: true, propagate: true, ..FilletSpec::radius(5.0) };
    let r = k.fillet_with(b, &[11, 12, 2, 6].map(EdgeId), &spec).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    // OCCT takes 1986 mm³ off.
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((1880.0..2040.0).contains(&removed), "{removed}");
}

/// A degenerate edge among the picked ones (OCCT's `explode` lists a cone's apex; blend
/// encoderegularity_a5 picks two pole edges) is passed over, as OCCT's blend does.
#[test]
fn degenerate_picked_edge_is_passed_over() {
    let mut k = NativeKernel::new();
    let s = k.pcone_a(None, 50.0, 0.0, 120.0, 90.0).unwrap();
    let es = k.explode_edges(&s).unwrap();
    let (draw::Shape::Body(b), draw::Shape::BodyEdge(_, apex), draw::Shape::BodyEdge(_, arc)) = (&s, &es[0], &es[2]) else { panic!("not a body's edges") };
    assert!(k.get(*b).unwrap().edges()[apex.0 as usize].degenerate);
    let area = |k: &mut NativeKernel, edges: &[EdgeId]| {
        let r = k.fillet(*b, edges, 10.0).unwrap();
        k.mass_properties(r.bodies[0]).unwrap().surface_area
    };
    let both = area(&mut k, &[*apex, *arc]);
    let one = area(&mut k, &[*arc]);
    assert!((both - one).abs() < 1e-9 * one, "{both} {one}");
    assert!(k.fillet(*b, &[*apex], 10.0).is_err());
}
