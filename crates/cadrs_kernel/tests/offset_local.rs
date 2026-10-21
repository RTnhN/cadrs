//! Regressions for offsets, shells and local operations (`native::local`, `native::offset_round`,
//! `native::slab`) through the `Kernel` trait, checked against closed forms.

use std::f64::consts::PI;

use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::math::Frame;
use cadrs_kernel::*;
use nalgebra::{Point2, Point3, Vector3};

fn one(r: OpResult) -> BodyId {
    assert_eq!(r.bodies.len(), 1, "one body");
    r.bodies[0]
}

fn polygon(pts: &[(f64, f64)]) -> Loop {
    let p = |q: (f64, f64)| Point2::new(q.0, q.1);
    Loop { curves: (0..pts.len()).map(|i| Curve2::Line { a: p(pts[i]), b: p(pts[(i + 1) % pts.len()]), source: None }).collect() }
}

fn circle(c: (f64, f64), r: f64) -> Loop {
    Loop { curves: vec![Curve2::Circle { center: Point2::new(c.0, c.1), radius: r, source: None }] }
}

/// The region `outer` less `holes` on the top plane, extruded up by `h`.
fn prism(k: &mut NativeKernel, outer: Loop, holes: Vec<Loop>, h: f64) -> BodyId {
    one(k.extrude(&Profile::new(Plane::top(), vec![Region { outer, holes, source: None }]), Extent::Blind(h)).unwrap())
}

fn volume(k: &NativeKernel, b: BodyId) -> f64 {
    k.mass_properties(b).unwrap().volume
}

fn close(got: f64, want: f64, rel: f64) {
    assert!((got - want).abs() <= rel * want.abs(), "{got} vs {want} ({:.1e} off)", (got - want).abs() / want.abs());
}

/// The planar face with outward normal `n` at `at` along it.
fn face_at(k: &NativeKernel, b: BodyId, n: Vector3<f64>, at: f64) -> FaceId {
    k.faces(b)
        .unwrap()
        .into_iter()
        .find(|f| f.plane.is_some_and(|p| p.normal.dot(&n) > 1.0 - 1e-9) && (f.center.coords.dot(&n) - at).abs() < 1e-9)
        .expect("no such face")
        .id
}

/// A box a × b × c grown by a ball of radius d (Steiner's formula).
fn grown_box(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a * b * c + 2.0 * d * (a * b + b * c + c * a) + PI * d * d * (a + b + c) + 4.0 / 3.0 * PI * d * d * d
}

/// A 10 mm cube with a through hole of radius 1 grown by 2 (more than the hole's radius): the
/// hole's wall (a concave cylinder whose offset would have no radius) fills to its axis, its
/// rims' tubes (wider than the rims) are spindle tori. Inside the hole every point is within 1
/// of the wall; above its mouths a point at radius ρ < 1 and height z is nearest the rim, so
/// it is in only when (1 − ρ)² + z² ≤ 4: the grown cube less a dimple of volume
/// 2π∫₀¹ ρ (2 − √(4 − (1 − ρ)²)) dρ at each mouth.
#[test]
fn rounded_offset_past_a_hole_radius() {
    let mut k = NativeKernel::new();
    let (r, d) = (1.0, 2.0);
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]), vec![circle((5.0, 5.0), r)], 10.0);
    let out = one(k.offset(b, &OffsetSpec { distance: d, faces: vec![], sharp: false }).unwrap());
    // Simpson's rule on the dimple's (smooth) integrand.
    let n = 2000;
    let f = |rho: f64| rho * (d - (d * d - (r - rho) * (r - rho)).sqrt());
    let h = r / n as f64;
    let dimple = 2.0 * PI * h / 3.0 * (0..=n).map(|i| f(i as f64 * h) * if i == 0 || i == n { 1.0 } else if i % 2 == 1 { 4.0 } else { 2.0 }).sum::<f64>();
    close(volume(&k, out), grown_box(10.0, 10.0, 10.0, d) - 2.0 * dimple, 1e-6);
}

/// An L-shaped block (a 10 × 10 square less its 5 × 5 corner, 8 tall) inset by 1, rounded
/// (SMLib's inset, `SetExtendConvexEdges(FALSE)`):
/// the faces move in by 1 and the concave edge, which every point within 1 of keeps away,
/// becomes a quarter round of radius 1 about it: the sharp inset plus the square corner less
/// the quarter disc, (1 − π/4), over the inset height 6.
#[test]
fn rounded_inset_rounds_concave_edges() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (10.0, 0.0), (10.0, 5.0), (5.0, 5.0), (5.0, 10.0), (0.0, 10.0)]), vec![], 8.0);
    let out = one(k.offset(b, &OffsetSpec { distance: -1.0, faces: vec![], sharp: false }).unwrap());
    // The sharp inset: the 8 × 8 square less its 5 × 5 corner beyond (4, 4) …
    let sharp = (64.0 - 25.0) * 6.0;
    close(volume(&k, out), sharp + (1.0 - PI / 4.0) * 6.0, 1e-9);
    // … which the sharp offset gives.
    let s = one(k.offset(b, &OffsetSpec { distance: -1.0, faces: vec![], sharp: true }).unwrap());
    close(volume(&k, s), sharp, 1e-9);
}

/// The same L inset by 1 with its bottom kept: the faces round it move in, the bottom stays.
#[test]
fn rounded_inset_keeps_a_face() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (10.0, 0.0), (10.0, 5.0), (5.0, 5.0), (5.0, 10.0), (0.0, 10.0)]), vec![], 8.0);
    let bottom = face_at(&k, b, -Vector3::z(), 0.0);
    let out = one(k.offset(b, &OffsetSpec { distance: -1.0, faces: vec![(bottom, 0.0)], sharp: false }).unwrap());
    close(volume(&k, out), (64.0 - 25.0) * 7.0 + (1.0 - PI / 4.0) * 7.0, 1e-9);
}

/// An outward shell with sharp corners (SMLib's `bExtendCorners`): the open bottom stays where
/// it was, the other faces move out by 4 and meet where they cross: (108 · 68 · 44) − the body.
#[test]
fn outward_shell_with_sharp_corners() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (100.0, 0.0), (100.0, 60.0), (0.0, 60.0)]), vec![], 40.0);
    let bottom = face_at(&k, b, -Vector3::z(), 0.0);
    let spec = ShellSpec { remove: vec![bottom], thickness: 4.0, outward: true, hollow: false, sharp: true };
    let out = one(k.shell_with(b, &spec).unwrap());
    close(volume(&k, out), 108.0 * 68.0 * 44.0 - 100.0 * 60.0 * 40.0, 1e-9);
}

/// A cube's top replaced by a cylinder across it whose axis lies above the top: the side
/// planes cut the cylinder twice, and the domain (the upper half) picks the arcs. The result
/// is the 10 × 10 × 12 block under a half round of radius 5 (SMLib's local operation 3).
#[test]
fn replace_face_takes_the_arc_in_the_domain() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]), vec![], 10.0);
    let top = face_at(&k, b, Vector3::z(), 10.0);
    // u from −x round to +z: the upper half for u in [0, π].
    let f = Frame::from_xz(Point3::new(5.0, 0.0, 12.0), -Vector3::x(), Vector3::y());
    let upper = FaceReplacement { face: top, surface: Surface::Cylinder { f, r: 5.0 }, domain: Some(((0.0, PI), (0.0, 10.0))) };
    let out = one(k.replace_faces(b, std::slice::from_ref(&upper)).unwrap());
    close(volume(&k, out), 1200.0 + 125.0 * PI, 1e-9);
    // Without a domain, the crossings nearest the old edges: the lower half.
    let lower = one(k.replace_faces(b, &[FaceReplacement { domain: None, ..upper }]).unwrap());
    close(volume(&k, lower), 1200.0 - 125.0 * PI, 1e-9);
}
