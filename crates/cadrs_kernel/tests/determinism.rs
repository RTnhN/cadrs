//! Determinism: the same operations give bit-identical results when run again, and on 1 or 4
//! rayon threads. Compared: each body as the body format writes it (`write_body`, exact
//! doubles), its tessellation and mass properties (by their `Debug` text, which spells every
//! double so it reads back to the same bits), the operation's history, and the STEP file of all
//! of them (less the time stamp in its header).

use cadrs_kernel::native::kernel::NativeKernel;
use cadrs_kernel::native::topo::CheckLevel;
use cadrs_kernel::*;
use nalgebra::{Point2, Point3, Vector3};

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

fn solid(k: &mut NativeKernel, plane: Plane, outer: Loop, extent: Extent) -> BodyId {
    let r = Region { outer, holes: vec![], source: None };
    k.extrude(&Profile::new(plane, vec![r]), extent).unwrap().bodies[0]
}

/// The XZ plane (normal −Y).
fn front() -> Plane {
    Plane { origin: Point3::origin(), x_dir: Vector3::x_axis(), normal: -Vector3::y_axis() }
}

/// What one run produced, case by case.
#[derive(Debug, PartialEq)]
struct Run {
    /// (case, body bytes, mesh, mass properties, history)
    cases: Vec<(&'static str, Vec<u8>, String, String, String)>,
    step: Vec<u8>,
}

/// The sample of operations: booleans with analytic and numeric intersections, fillets, a
/// rounded offset and a shell, each result checked, tessellated and measured, and all of them
/// written to one STEP file.
fn run() -> Run {
    let mut k = NativeKernel::new();
    let bx = solid(&mut k, Plane::top(), rect(-5.0, -4.0, 5.0, 4.0), Extent::Blind(6.0));
    let hole = solid(&mut k, Plane::top(), circle(1.0, 0.5, 2.0), Extent::Symmetric(20.0));
    let cross = solid(&mut k, front(), circle(0.0, 3.0, 1.5), Extent::Symmetric(30.0));
    let upright = solid(&mut k, Plane::top(), circle(0.0, 0.0, 2.5), Extent::Blind(12.0));
    let mut cases: Vec<(&'static str, OpResult)> = Vec::new();
    let cut = k.boolean(BoolOp::Subtract, bx, &[hole]).unwrap();
    let cut_id = cut.bodies[0];
    cases.push(("box minus cylinder", cut));
    // Two cylinders crossing square: a numeric (non-planar) intersection curve.
    cases.push(("crossed cylinders", k.boolean(BoolOp::Union, upright, &[cross]).unwrap()));
    cases.push(("box and cylinder in common", k.boolean(BoolOp::Intersect, bx, &[upright]).unwrap()));
    let vertical: Vec<EdgeId> = k.edges(bx).unwrap().iter().filter(|e| (e.start.z - e.end.z).abs() > 1.0).map(|e| e.id).collect();
    cases.push(("box's vertical edges filleted", k.fillet(bx, &vertical, 1.0).unwrap()));
    let rim: Vec<EdgeId> = k.edges(cut_id).unwrap().iter().filter(|e| e.curve == CurveKind::Circle).map(|e| e.id).collect();
    cases.push(("hole's rims filleted", k.fillet(cut_id, &rim, 0.5).unwrap()));
    cases.push(("rounded offset", k.offset(cut_id, &OffsetSpec { distance: 0.75, faces: vec![], sharp: false }).unwrap()));
    let top = k.faces(bx).unwrap().into_iter().max_by(|a, b| a.center.z.total_cmp(&b.center.z)).unwrap().id;
    cases.push(("shelled box", k.shell(bx, &[top], 0.8).unwrap()));
    let mut out = Vec::new();
    let mut parts = Vec::new();
    for (name, r) in cases {
        let id = r.bodies[0];
        let issues = k.get(id).unwrap().check_with(CheckLevel::Full);
        assert!(issues.is_empty(), "{name}: {issues:?}");
        let mesh = k.tessellate(id, Tessellation::default()).unwrap();
        let mass = k.mass_properties(id).unwrap();
        out.push((name, k.write_body(id).unwrap(), format!("{mesh:?}"), format!("{mass:?}"), format!("{:?}", r.history)));
        parts.push((id, name.to_string()));
    }
    let step = k.export_model(exchange::ExchangeFormat::Step, "determinism", &parts, &[]).unwrap();
    Run { cases: out, step: without_timestamp(step) }
}

/// The STEP text with its header's time stamp (the one line that should differ) blanked.
fn without_timestamp(step: Vec<u8>) -> Vec<u8> {
    let text = String::from_utf8(step).unwrap();
    text.lines().map(|l| if l.starts_with("FILE_NAME(") { "FILE_NAME(time stamp left out)" } else { l }).collect::<Vec<_>>().join("\n").into_bytes()
}

/// Says which output of which case differs (the whole texts are too long to print).
#[track_caller]
fn same(a: &Run, b: &Run, what: &str) {
    for (x, y) in a.cases.iter().zip(&b.cases) {
        assert!(x.1 == y.1, "{what}: {}: the body differs", x.0);
        assert!(x.2 == y.2, "{what}: {}: the mesh differs", x.0);
        assert!(x.3 == y.3, "{what}: {}: the mass properties differ: {} vs {}", x.0, x.3, y.3);
        assert!(x.4 == y.4, "{what}: {}: the history differs", x.0);
    }
    assert_eq!(a.cases.len(), b.cases.len());
    if a.step != b.step {
        let line = a.step.split(|&c| c == b'\n').zip(b.step.split(|&c| c == b'\n')).position(|(x, y)| x != y);
        panic!("{what}: the STEP files differ (first at line {line:?})");
    }
}

fn on_threads(n: usize) -> Run {
    rayon::ThreadPoolBuilder::new().num_threads(n).build().unwrap().install(run)
}

#[test]
fn repeat_runs_are_identical() {
    let first = on_threads(4);
    let second = on_threads(4);
    same(&first, &second, "a second run");
}

#[test]
fn thread_count_does_not_change_results() {
    let one = on_threads(1);
    let four = on_threads(4);
    same(&one, &four, "1 vs 4 threads");
}
