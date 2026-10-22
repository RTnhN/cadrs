//! STEP and IGES reader fixes, each on a minimal fixture.

use cadrs_kernel::native::{step, NativeKernel};
use cadrs_kernel::*;
use nalgebra::Point2;

fn block(k: &mut NativeKernel) -> BodyId {
    let c = [Point2::new(0.0, 0.0), Point2::new(100.0, 0.0), Point2::new(100.0, 60.0), Point2::new(0.0, 60.0)];
    let curves = (0..4).map(|i| Curve2::Line { a: c[i], b: c[(i + 1) % 4], source: None }).collect();
    let profile = Profile::new(Plane::top(), vec![Region { outer: Loop { curves }, holes: vec![], source: None }]);
    k.extrude(&profile, Extent::Blind(40.0)).unwrap().bodies[0]
}

/// A STEP file whose vertex lies off its edges' ends (by 5e-3 on a 100 mm block, far past the
/// file's stated 1e-6): the vertex's tolerance grows to the gap (ShapeFix's same-parameter), so
/// the block reads clean.
#[test]
fn step_vertex_off_its_edges_reads_clean() {
    let mut k = NativeKernel::new();
    let b = block(&mut k);
    let body = k.get(b).unwrap().as_ref().clone();
    let text = String::from_utf8(step::write("block", &[(&body, "block")], &[])).unwrap();
    // The first vertex's point, moved 5e-3 along z.
    let vp = text.lines().find(|l| l.contains("=VERTEX_POINT('',#")).unwrap();
    let pid: String = vp.split("#").nth(2).unwrap().chars().take_while(|c| c.is_ascii_digit()).collect();
    let head = format!("#{pid}=CARTESIAN_POINT('',(");
    let line = text.lines().find(|l| l.starts_with(&head)).unwrap().to_string();
    let nums: Vec<f64> = line[head.len()..line.find("))").unwrap()].split(',').map(|x| x.trim().parse().unwrap()).collect();
    let moved = format!("{head}{},{},{}));", nums[0], nums[1], nums[2] + 5e-3);
    let text = text.replace(&line, &moved);
    let m = step::read(text.as_bytes()).unwrap();
    let errs = m.parts[0].body.check();
    assert!(errs.is_empty(), "{errs:?}");
}
