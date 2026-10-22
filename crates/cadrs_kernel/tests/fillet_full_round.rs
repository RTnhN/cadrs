//! Fillets that consume a face whole (a full round: the fillets of its two opposite edges meet
//! across it). cable organizer Copy 2, Fillet 2: R1.5 on a 3 mm rib's top and end edges makes
//! two full rounds that meet at the rib's corner, which OCCT's fillet builds and this kernel
//! doesn't yet: it says so instead of "the result is empty".

use std::f64::consts::PI;

use cadrs_kernel::native::kernel::NativeKernel;
use cadrs_kernel::native::tol;
use cadrs_kernel::*;
use nalgebra::Point2;

const LENGTH: f64 = 10.0;
const WIDTH: f64 = 3.0;
const HEIGHT: f64 = 5.0;
const R: f64 = WIDTH / 2.0;

/// A rib LENGTH × WIDTH × HEIGHT and its edges.
fn rib() -> (NativeKernel, BodyId, Vec<EdgeInfo>) {
    let c = [(0.0, 0.0), (LENGTH, 0.0), (LENGTH, WIDTH), (0.0, WIDTH)].map(|(x, y)| Point2::new(x, y));
    let outer = Loop { curves: (0..4).map(|i| Curve2::Line { a: c[i], b: c[(i + 1) % 4], source: None }).collect() };
    let mut k = NativeKernel::new();
    let id = k.extrude(&Profile::new(Plane::top(), vec![Region { outer, holes: vec![], source: None }]), Extent::Blind(HEIGHT)).unwrap().bodies[0];
    let edges = k.edges(id).unwrap();
    (k, id, edges)
}

fn spec() -> FilletSpec {
    FilletSpec { size: FilletSize::Radius(R), allow_overflow: true, profile: FilletProfile::Circular, propagate: true }
}

fn at(v: f64, w: f64) -> bool {
    (v - w).abs() < tol::ROUNDOFF
}

/// The rib's top long edges: one full round across the top.
#[test]
fn one_full_round_builds() {
    let (mut k, id, edges) = rib();
    let top: Vec<EdgeId> = edges.iter().filter(|e| at(e.start.z, HEIGHT) && at(e.end.z, HEIGHT) && !at(e.start.x, e.end.x)).map(|e| e.id).collect();
    let r = k.fillet_with(id, &top, &spec()).unwrap();
    let v = k.mass_properties(r.bodies[0]).unwrap().volume;
    let want = LENGTH * WIDTH * HEIGHT - 2.0 * (1.0 - PI / 4.0) * R * R * LENGTH;
    assert!((v - want).abs() < tol::POINT * want, "volume {v}, want {want}");
}

/// The top's and the end's edges: two full rounds meeting at the corner. Refused with the
/// reason (a radius a little smaller builds).
#[test]
fn full_rounds_meeting_at_a_corner_say_so() {
    let (mut k, id, edges) = rib();
    let picked: Vec<EdgeId> = edges
        .iter()
        .filter(|e| {
            let top = at(e.start.z, HEIGHT) && at(e.end.z, HEIGHT);
            let end = at(e.start.x, LENGTH) && at(e.end.x, LENGTH);
            top && (end || !at(e.start.x, e.end.x)) || end && !at(e.start.z, e.end.z)
        })
        .map(|e| e.id)
        .collect();
    assert_eq!(picked.len(), 5);
    let e = k.fillet_with(id, &picked, &spec()).unwrap_err();
    assert!(matches!(&e, KernelError::TooLarge { op: "fillet", why } if why.contains("full rounds")), "{e}");
    let smaller = FilletSpec { size: FilletSize::Radius(R * 0.99), ..spec() };
    assert!(k.fillet_with(id, &picked, &smaller).is_ok());
}
