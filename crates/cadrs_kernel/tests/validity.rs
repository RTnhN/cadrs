//! `Body::check_with(CheckLevel::Full)`: valid models pass it, and each kind of defect it adds
//! to the fast check (loops crossing in (u, v), loop orientation and nesting, shell
//! connectivity, p-curves off their edges between the fast check's samples, voids) is found.

use std::f64::consts::{PI, TAU};

use cadrs_kernel::native::curve::Curve;
use cadrs_kernel::native::kernel::NativeKernel;
use cadrs_kernel::native::math::Affine;
use cadrs_kernel::native::nurbs::NurbsCurve;
use cadrs_kernel::native::topo::{Body, CheckLevel, Solid};
use cadrs_kernel::*;
use nalgebra::{Point2, Point3, Vector3, Vector4};

fn p(x: f64, y: f64) -> Point2<f64> {
    Point2::new(x, y)
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
    let c = [p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)];
    Loop { curves: (0..4).map(|i| Curve2::Line { a: c[i], b: c[(i + 1) % 4], source: None }).collect() }
}

fn circle(x: f64, y: f64, r: f64) -> Loop {
    Loop { curves: vec![Curve2::Circle { center: p(x, y), radius: r, source: None }] }
}

fn region(outer: Loop, holes: Vec<Loop>) -> Region {
    Region { outer, holes, source: None }
}

fn extrude(k: &mut NativeKernel, r: Region, depth: f64) -> BodyId {
    k.extrude(&Profile::new(Plane::top(), vec![r]), Extent::Blind(depth)).unwrap().bodies[0]
}

/// The XZ plane (normal −Y), for revolve profiles.
fn front() -> Plane {
    Plane { origin: Point3::origin(), x_dir: Vector3::x_axis(), normal: -Vector3::y_axis() }
}

fn body(k: &NativeKernel, id: BodyId) -> Body {
    Body::clone(k.get(id).unwrap())
}

#[track_caller]
fn valid(b: &Body) {
    let issues = b.check_with(CheckLevel::Full);
    assert!(issues.is_empty(), "{issues:?}");
}

#[track_caller]
fn finds(b: &Body, what: &str) {
    let issues = b.check_with(CheckLevel::Full);
    assert!(issues.iter().any(|i| i.contains(what)), "expected {what:?} in {issues:?}");
}

/// A plate with a round hole: 10 × 10 × 2, hole of radius 2 at (5, 5).
fn plate(k: &mut NativeKernel) -> BodyId {
    extrude(k, region(rect(0.0, 0.0, 10.0, 10.0), vec![circle(5.0, 5.0, 2.0)]), 2.0)
}

/// A move by `du` along u (of p-curves).
fn moved(du: f64) -> Affine {
    Affine { m: nalgebra::Matrix3::identity(), t: Vector3::new(du, 0.0, 0.0) }
}

/// The planar face of `b` with the most loops (the plate's top or bottom).
fn holed_face(b: &Body) -> usize {
    (0..b.faces.len()).max_by_key(|&f| b.faces[f].loops.len()).unwrap()
}

#[test]
fn valid_models_pass_the_full_check() {
    let mut k = NativeKernel::new();
    let bx = extrude(&mut k, region(rect(0.0, 0.0, 10.0, 6.0), vec![]), 4.0);
    let cyl = extrude(&mut k, region(circle(0.0, 0.0, 3.0), vec![]), 5.0);
    let pl = plate(&mut k);
    // A sphere and a torus, revolved about z.
    let half = Region {
        outer: Loop {
            curves: vec![
                Curve2::Arc { center: p(0.0, 0.0), radius: 2.0, start_angle: -PI / 2.0, sweep: PI, source: None },
                Curve2::Line { a: p(0.0, 2.0), b: p(0.0, -2.0), source: None },
            ],
        },
        holes: vec![],
        source: None,
    };
    let z = Axis { origin: Point3::origin(), dir: Vector3::z_axis() };
    let sphere = k.revolve(&Profile::new(front(), vec![half]), z, TAU).unwrap().bodies[0];
    let torus = k.revolve(&Profile::new(front(), vec![region(circle(5.0, 0.0, 1.0), vec![])]), z, TAU).unwrap().bodies[0];
    // A boolean with a curved cut, a fillet and a hollow shell (a solid with a void).
    let cut = k.boolean(BoolOp::Subtract, bx, &[cyl]).unwrap().bodies[0];
    let edge = k.edges(bx).unwrap()[0].id;
    let filleted = k.fillet(bx, &[edge], 1.0).unwrap().bodies[0];
    let hollow = k.shell_with(bx, &ShellSpec { remove: vec![], thickness: 1.0, outward: false, hollow: true, sharp: true }).unwrap().bodies[0];
    for id in [bx, cyl, pl, sphere, torus, cut, filleted, hollow] {
        valid(&body(&k, id));
    }
    assert_eq!(k.get(hollow).unwrap().solids[0].shells.len(), 2, "the hollow box has a void");
}

#[test]
fn finds_loops_that_cross() {
    let mut k = NativeKernel::new();
    let id = plate(&mut k);
    let mut b = body(&k, id);
    let fi = holed_face(&b);
    // Move the hole's p-curve so it straddles the outer loop's side x = 10.
    let shift = moved(4.0);
    for c in b.faces[fi].loops[1].iter_mut() {
        c.pc = c.pc.transformed(&shift);
    }
    finds(&b, "cross in (u, v)");
}

#[test]
fn finds_holes_outside_and_wrong_way() {
    let mut k = NativeKernel::new();
    let id = plate(&mut k);
    let good = body(&k, id);
    let fi = holed_face(&good);
    // The hole moved wholly outside the outer loop.
    let mut b = good.clone();
    let far = moved(30.0);
    for c in b.faces[fi].loops[1].iter_mut() {
        c.pc = c.pc.transformed(&far);
    }
    finds(&b, "a hole outside the outer loop");
    // The hole and the outer loop swapped: the first loop is no longer the outer one.
    let mut b = good.clone();
    b.faces[fi].loops.swap(0, 1);
    finds(&b, "outer loop");
    // The whole face turned over without flipping its sense: both loops run the wrong way.
    let mut b = good.clone();
    for l in &mut b.faces[fi].loops {
        l.reverse();
        for c in l.iter_mut() {
            c.fwd = !c.fwd;
        }
    }
    finds(&b, "runs clockwise");
    finds(&b, "a hole running the same way");
}

#[test]
fn finds_disconnected_and_shared_shells() {
    let mut k = NativeKernel::new();
    let a = extrude(&mut k, region(rect(0.0, 0.0, 1.0, 1.0), vec![]), 1.0);
    let c = extrude(&mut k, region(rect(5.0, 0.0, 6.0, 1.0), vec![]), 1.0);
    let mut b = body(&k, a);
    b.append(&body(&k, c));
    // Both boxes' faces in one shell of one solid.
    let mut one = b.clone();
    let faces: Vec<usize> = (0..one.faces.len()).collect();
    one.shells = vec![cadrs_kernel::native::topo::Shell { faces, closed: true }];
    one.solids = vec![Solid { shells: vec![0] }];
    assert!(one.check().is_empty(), "the fast check doesn't look at connectivity: {:?}", one.check());
    finds(&one, "isn't connected");
    // A face in both shells.
    let mut two = b.clone();
    two.shells[1].faces.push(0);
    finds(&two, "face 0 is in shells 0 and 1");
}

#[test]
fn finds_voids_outside_or_inside_out() {
    let mut k = NativeKernel::new();
    let big = extrude(&mut k, region(rect(0.0, 0.0, 10.0, 10.0), vec![]), 10.0);
    let inner = extrude(&mut k, region(rect(4.0, 4.0, 6.0, 6.0), vec![]), 6.0);
    let away = extrude(&mut k, region(rect(40.0, 4.0, 42.0, 6.0), vec![]), 6.0);
    let with_void = |small: BodyId, turn: bool| {
        let mut b = body(&k, big);
        let mut s = body(&k, small);
        if turn {
            for f in &mut s.faces {
                Body::reverse_face(f);
            }
        }
        b.append(&s);
        b.solids = vec![Solid { shells: vec![0, 1] }];
        b
    };
    valid(&with_void(inner, true));
    finds(&with_void(away, true), "lies outside the outer shell");
    finds(&with_void(inner, false), "must point into the void");
}

#[test]
fn finds_a_pcurve_off_its_edge_between_samples() {
    let mut k = NativeKernel::new();
    let id = extrude(&mut k, region(rect(0.0, 0.0, 10.0, 10.0), vec![]), 2.0);
    let mut b = body(&k, id);
    // A straight edge's p-curve on a planar face replaced by a polyline with a 0.01 bump
    // between the fast check's first two samples (10 % and 25 % along it).
    let (fi, li, ci) = b
        .faces
        .iter()
        .enumerate()
        .flat_map(|(fi, f)| f.loops.iter().enumerate().flat_map(move |(li, l)| (0..l.len()).map(move |ci| (fi, li, ci))))
        .find(|&(fi, li, ci)| matches!(b.faces[fi].loops[li][ci].pc, Curve::Line { .. }) && !b.edges[b.faces[fi].loops[li][ci].edge].degenerate)
        .unwrap();
    let c = &b.faces[fi].loops[li][ci];
    let e = &b.edges[c.edge];
    let (t0, t1) = (e.t[0], e.t[1]);
    let at = |s: f64| c.pc.point(t0 + (t1 - t0) * s);
    let (a, z) = (at(0.0), at(1.0));
    let d = (z - a).normalize();
    let bump = at(0.17) + Vector3::new(-d.y, d.x, 0.0) * 0.01;
    let h = |q: Point3<f64>| Vector4::new(q.x, q.y, q.z, 1.0);
    let t = |s: f64| t0 + (t1 - t0) * s;
    let knots = vec![t0, t0, t(0.12), t(0.17), t(0.22), t1, t1];
    let cps = vec![h(a), h(at(0.12)), h(bump), h(at(0.22)), h(z)];
    b.faces[fi].loops[li][ci].pc = Curve::Nurbs(std::sync::Arc::new(NurbsCurve { degree: 1, knots, cps }));
    assert!(b.check().is_empty(), "the fast check samples past the bump: {:?}", b.check());
    finds(&b, "p-curve off the edge");
}
