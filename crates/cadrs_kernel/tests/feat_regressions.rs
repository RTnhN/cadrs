//! Form features (DRAW's `featprism`/`featdprism`/`featperform`, `native::draw`) against
//! closed forms: pockets up to and between curved faces, through-all drafted prisms, and
//! features up to several faces.

use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::native::draw::Shape;
use cadrs_kernel::{Kernel, SurfaceKind};

/// The face of a body with a surface of `kind`.
fn face_of(k: &NativeKernel, s: &Shape, kind: SurfaceKind) -> Shape {
    let Shape::Body(b) = s else {
        panic!("not a body")
    };
    let f = k
        .faces(*b)
        .unwrap()
        .into_iter()
        .find(|f| f.kind == kind)
        .unwrap();
    Shape::BodyFace(*b, f.id)
}

/// A rectangle x ∈ [x0, x1], z ∈ [z0, z1] in the plane y = `y`, as a planar face.
fn rect(k: &NativeKernel, y: f64, (x0, x1): (f64, f64), (z0, z1): (f64, f64)) -> Shape {
    let w = k
        .polyline(&[
            [x0, y, z0],
            [x1, y, z0],
            [x1, y, z1],
            [x0, y, z1],
            [x0, y, z0],
        ])
        .unwrap();
    k.mkplane(&w).unwrap()
}

/// The area under the circle x² + y² = r² across x0..x1, both halves: ∫ 2√(r² − x²) dx.
fn chord_area(r: f64, x0: f64, x1: f64) -> f64 {
    let f = |x: f64| x * (r * r - x * x).sqrt() + r * r * (x / r).asin();
    f(x1) - f(x0)
}

#[test]
fn pocket_up_to_a_cylinder_goes_through_it() {
    // A pocket from outside a cylinder up to its own side face stops where the line through
    // the profile leaves the material: the far side, so it cuts right through (OCCT's
    // featprism A3).
    let (r, h) = (50.0, 200.0);
    let mut k = NativeKernel::new();
    let cyl = k.pcylinder(r, h).unwrap();
    let side = face_of(&k, &cyl, SurfaceKind::Cylinder);
    let (xs, zs) = ((-20.0, 25.0), (60.0, 150.0));
    let prof = rect(&k, -100.0, xs, zs);
    let res = k
        .featperform(
            &cyl,
            &prof,
            &prof,
            "prism",
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            0,
            None,
            Some(&side),
        )
        .unwrap();
    let v = k.vprops(&res).unwrap();
    let want = std::f64::consts::PI * r * r * h - (zs.1 - zs.0) * chord_area(r, xs.0, xs.1);
    assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
}

#[test]
fn pocket_between_the_same_curved_face_spans_it() {
    // From and Until the same cylindrical face: the part of the long prism inside it (OCCT's
    // featprism C8).
    let (r, h) = (40.0, 120.0);
    let mut k = NativeKernel::new();
    let cyl = k.pcylinder(r, h).unwrap();
    let side = face_of(&k, &cyl, SurfaceKind::Cylinder);
    let (xs, zs) = ((-10.0, 30.0), (20.0, 70.0));
    let prof = rect(&k, -90.0, xs, zs);
    let res = k
        .featperform(
            &cyl,
            &prof,
            &prof,
            "prism",
            [0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
            0,
            Some(&side),
            Some(&side),
        )
        .unwrap();
    let v = k.vprops(&res).unwrap();
    let want = std::f64::consts::PI * r * r * h - (zs.1 - zs.0) * chord_area(r, xs.0, xs.1);
    assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
}

#[test]
fn drafted_prism_through_all_tapers_through_the_profile() {
    // A square drafted pocket through a slab, its profile off the slab: the taper is linear
    // along the profile's normal on both sides of it, so the slab loses the frustum between
    // its faces (side a ∓ 2 t tan α at distance t), whichever way the taper runs.
    let (a, t0, t1) = (20.0, 30.0, 50.0);
    let mut k = NativeKernel::new();
    let slab = k.box_(100.0, t1 - t0, 100.0).unwrap();
    let slab = k.ttranslate(&slab, -50.0, t0, -50.0).unwrap();
    let prof = rect(&k, 0.0, (-a / 2.0, a / 2.0), (-a / 2.0, a / 2.0));
    let angle_deg: f64 = 10.0; // DRAW's: the draft is half of it
    let res = k
        .featperform(
            &slab,
            &prof,
            &prof,
            "dprism",
            [angle_deg, 0.0, 0.0, 0.0, 0.0, 0.0],
            0,
            None,
            None,
        )
        .unwrap();
    let v = k.vprops(&res).unwrap();
    let ta = (angle_deg.to_radians() / 2.0).tan();
    let frustum = |s: f64| {
        // ∫ (a + 2 s t tan α)² dt over t ∈ [t0, t1], s = ±1.
        let f = |t: f64| (a + 2.0 * s * t * ta).powi(3) / (6.0 * s * ta);
        f(t1) - f(t0)
    };
    let full = 100.0 * (t1 - t0) * 100.0;
    let (w1, w2) = (full - frustum(1.0), full - frustum(-1.0));
    assert!(
        (v - w1).abs() < 1e-6 * w1 || (v - w2).abs() < 1e-6 * w2,
        "{v} vs {w1} or {w2}"
    );
}
