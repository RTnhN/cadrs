//! Evolved solids (`native::evolved`) against closed forms: a rectangle spine with square and
//! quarter-disc profiles, outside and inside it; wire profiles (DRAW's `evolved`), open with
//! lids and closed across the spine.

use cadrs_kernel::native::curve::Curve;
use cadrs_kernel::native::evolved::evolved;
use cadrs_kernel::native::math::P3;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::*;
use nalgebra::Point2;
use std::f64::consts::PI;

fn rect_spine(a: f64, b: f64) -> Vec<(Curve, f64, f64)> {
    let p = [P3::new(0.0, 0.0, 0.0), P3::new(a, 0.0, 0.0), P3::new(a, b, 0.0), P3::new(0.0, b, 0.0)];
    (0..4)
        .map(|i| {
            let (c, l) = Curve::line(p[i], p[(i + 1) % 4]);
            (c, 0.0, l)
        })
        .collect()
}

fn line(a: (f64, f64), b: (f64, f64)) -> Curve2 {
    Curve2::Line { a: Point2::new(a.0, a.1), b: Point2::new(b.0, b.1), source: None }
}

#[test]
fn rectangle_spine_square_profile_outside() {
    // A w × h square outside a rectangle: h (2w(a + b) + πw²).
    let (a, b, w, h) = (40.0, 20.0, 5.0, 3.0);
    let prof = Region { outer: Loop { curves: vec![line((0.0, 0.0), (w, 0.0)), line((w, 0.0), (w, h)), line((w, h), (0.0, h)), line((0.0, h), (0.0, 0.0))] }, holes: vec![], source: None };
    let mut k = NativeKernel::new();
    let s = evolved(&mut k, &rect_spine(a, b), &[prof]).unwrap();
    let v = cadrs_kernel::Kernel::mass_properties(&k, s).unwrap().volume;
    let want = h * (2.0 * w * (a + b) + PI * w * w);
    assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
}

#[test]
fn rectangle_spine_quarter_disc_profile() {
    // A quarter disc of radius r against the spine: πr²/4 · perimeter + (2/3)πr³ (Pappus,
    // the corners a hemisphere together).
    let (a, b, r) = (30.0, 12.0, 4.0);
    let prof = Region {
        outer: Loop {
            curves: vec![
                line((0.0, 0.0), (r, 0.0)),
                Curve2::Arc { center: Point2::origin(), radius: r, start_angle: 0.0, sweep: PI / 2.0, source: None },
                line((0.0, r), (0.0, 0.0)),
            ],
        },
        holes: vec![],
        source: None,
    };
    let mut k = NativeKernel::new();
    let s = evolved(&mut k, &rect_spine(a, b), &[prof]).unwrap();
    let v = cadrs_kernel::Kernel::mass_properties(&k, s).unwrap().volume;
    let want = PI * r * r / 4.0 * 2.0 * (a + b) + 2.0 / 3.0 * PI * r * r * r;
    assert!((v - want).abs() < 1e-5 * want, "{v} vs {want}");
}

#[test]
fn rectangle_spine_profile_inside() {
    // A w × h square inside the rectangle: the inner offset's band, h (2w(a + b) − 4w²).
    let (a, b, w, h) = (40.0, 20.0, 5.0, 3.0);
    let prof = Region { outer: Loop { curves: vec![line((-w, 0.0), (0.0, 0.0)), line((0.0, 0.0), (0.0, h)), line((0.0, h), (-w, h)), line((-w, h), (-w, 0.0))] }, holes: vec![], source: None };
    let mut k = NativeKernel::new();
    let s = evolved(&mut k, &rect_spine(a, b), &[prof]).unwrap();
    let v = cadrs_kernel::Kernel::mass_properties(&k, s).unwrap().volume;
    let want = h * (2.0 * w * (a + b) - 4.0 * w * w);
    assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
}

#[test]
fn circle_spine_square_profile_outside() {
    // A w × h square outside a circle of radius R: an annulus h π((R + w)² − R²).
    let (r, w, h) = (10.0, 4.0, 2.5);
    let c = Curve::Circle { c: P3::new(0.0, 0.0, 0.0), x: nalgebra::Vector3::x(), y: nalgebra::Vector3::y(), r };
    let prof = Region { outer: Loop { curves: vec![line((0.0, 0.0), (w, 0.0)), line((w, 0.0), (w, h)), line((w, h), (0.0, h)), line((0.0, h), (0.0, 0.0))] }, holes: vec![], source: None };
    let mut k = NativeKernel::new();
    let s = evolved(&mut k, &[(c, 0.0, 2.0 * PI)], &[prof]).unwrap();
    let v = cadrs_kernel::Kernel::mass_properties(&k, s).unwrap().volume;
    let want = h * PI * ((r + w).powi(2) - r * r);
    assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
}

/// The rectangle's region as the spine's face (for `evolved_wire`).
fn rect_face(a: f64, b: f64) -> Profile {
    let outer = Loop { curves: vec![line((0.0, 0.0), (a, 0.0)), line((a, 0.0), (a, b)), line((a, b), (0.0, b)), line((0.0, b), (0.0, 0.0))] };
    Profile::new(Plane { origin: P3::origin(), x_dir: nalgebra::Unit::new_normalize(nalgebra::Vector3::x()), normal: nalgebra::Unit::new_normalize(nalgebra::Vector3::z()) }, vec![Region { outer, holes: vec![], source: None }])
}

/// A wire through the points (lines).
fn wire(pts: &[P3]) -> Vec<(Curve, f64, f64)> {
    pts.windows(2)
        .map(|w| {
            let (c, l) = Curve::line(w[0], w[1]);
            (c, 0.0, l)
        })
        .collect()
}

#[test]
fn wire_profile_open_outside_with_lids() {
    // DRAW's `evolved -v -solid` of an open vertical segment w outside a rectangle: lids close it
    // inwards, so the solid is the rectangle's offset by w (round corners) h high.
    let (a, b, w, h) = (30.0, 12.0, 2.5, 4.0);
    let mut k = NativeKernel::new();
    let prof = wire(&[P3::new(a / 2.0, -w, 0.0), P3::new(a / 2.0, -w, h)]);
    let r = cadrs_kernel::native::evolved::evolved_wire(&mut k, &rect_spine(a, b), &rect_face(a, b), &prof).unwrap();
    let v = cadrs_kernel::Kernel::mass_properties(&k, r.body).unwrap().volume;
    let want = h * (a * b + 2.0 * w * (a + b) + PI * w * w);
    assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
    assert_eq!(r.lids.len(), 2);
}

#[test]
fn wire_profile_open_slanted_inside() {
    // A segment from the spine at z = 0 to d inside it at z = h: the region shrinks by d z / h,
    // ∫ (a − 2s)(b − 2s) = h (ab − (a + b) d + 4d²/3) (OCCT's AGT001 in closed form).
    let (a, b, d, h) = (20.0, 10.0, 1.5, 5.0);
    let mut k = NativeKernel::new();
    let prof = wire(&[P3::new(a / 2.0, 0.0, 0.0), P3::new(a / 2.0, d, h)]);
    let r = cadrs_kernel::native::evolved::evolved_wire(&mut k, &rect_spine(a, b), &rect_face(a, b), &prof).unwrap();
    let v = cadrs_kernel::Kernel::mass_properties(&k, r.body).unwrap().volume;
    let want = h * (a * b - (a + b) * d + 4.0 * d * d / 3.0);
    assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
}

#[test]
fn wire_profile_closed_across_the_spine() {
    // A closed w + u wide, h high rectangle straddling the spine (w outside, u inside): the
    // outer band h (2w(a + b) + πw²) and the inner one h (2u(a + b) − 4u²).
    let (a, b, w, u, h) = (24.0, 14.0, 2.0, 1.0, 3.0);
    let mut k = NativeKernel::new();
    let x = a / 2.0;
    let prof = wire(&[P3::new(x, -w, 0.0), P3::new(x, u, 0.0), P3::new(x, u, h), P3::new(x, -w, h), P3::new(x, -w, 0.0)]);
    let r = cadrs_kernel::native::evolved::evolved_wire(&mut k, &rect_spine(a, b), &rect_face(a, b), &prof).unwrap();
    let v = cadrs_kernel::Kernel::mass_properties(&k, r.body).unwrap().volume;
    let want = h * (2.0 * w * (a + b) + PI * w * w + 2.0 * u * (a + b) - 4.0 * u * u);
    assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
    assert!(r.lids.is_empty());
}

#[test]
fn wire_profile_deeper_than_the_region() {
    // A segment from the spine at z = 0 to 2b inside a b-wide strip at z = h, lids inwards: the
    // solid holds the points at least s(z) = 2b z/h inside, the inner offset (a − 2s)(b − 2s)
    // until s reaches b/2 and nothing above.
    let (a, b, h) = (20.0, 4.0, 8.0);
    let d = 2.0 * b;
    let mut k = NativeKernel::new();
    let prof = wire(&[P3::new(a / 2.0, 0.0, 0.0), P3::new(a / 2.0, d, h)]);
    let r = cadrs_kernel::native::evolved::evolved_wire(&mut k, &rect_spine(a, b), &rect_face(a, b), &prof).unwrap();
    let v = cadrs_kernel::Kernel::mass_properties(&k, r.body).unwrap().volume;
    // With t = s, dz = h/d dt: (h/d) ∫0^{b/2} (a − 2t)(b − 2t) dt.
    let t1 = b / 2.0;
    let want = h / d * (a * b * t1 - (a + b) * t1 * t1 + 4.0 * t1.powi(3) / 3.0);
    assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
}
