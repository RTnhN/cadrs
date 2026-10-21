//! Evolved solids (`native::evolved`) against closed forms: a rectangle spine with square and
//! quarter-disc profiles, outside and inside it.

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
