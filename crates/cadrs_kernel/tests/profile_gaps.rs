//! Sketch loops whose consecutive curves leave small gaps: a sketch merges curve ends within its
//! own tolerance (1e-5 of its size) when it finds regions, so its loops can arrive with gaps
//! wider than the kernel's joint tolerance. They extrude (OCCT joined each curve to the next);
//! genuinely open loops still don't. The loops are cut down from the users' documents where the
//! extrudes failed with "the loop is not closed".

use std::f64::consts::PI;

use cadrs_kernel::native::kernel::NativeKernel;
use cadrs_kernel::native::tol;
use cadrs_kernel::native::topo::CheckLevel;
use cadrs_kernel::*;
use nalgebra::Point2;

fn p(x: f64, y: f64) -> Point2<f64> {
    Point2::new(x, y)
}

fn line(a: Point2<f64>, b: Point2<f64>) -> Curve2 {
    Curve2::Line { a, b, source: None }
}

fn arc(c: Point2<f64>, radius: f64, start_angle: f64, sweep: f64) -> Curve2 {
    Curve2::Arc {
        center: c,
        radius,
        start_angle,
        sweep,
        source: None,
    }
}

/// Extrudes the loop 2 up and checks the solid; returns its volume.
#[track_caller]
fn extrude(curves: Vec<Curve2>) -> Result<f64> {
    let mut k = NativeKernel::new();
    let region = Region {
        outer: Loop { curves },
        holes: vec![],
        source: None,
    };
    let r = k.extrude(
        &Profile::new(Plane::top(), vec![region]),
        Extent::Blind(2.0),
    )?;
    let id = r.bodies[0];
    let issues = k.get(id).unwrap().check_with(CheckLevel::Full);
    assert!(issues.is_empty(), "{issues:?}");
    Ok(k.mass_properties(id)?.volume)
}

/// The volumes' relative agreement with the sampled areas (the polygon's chord error).
const VOLUME_REL: f64 = 1e-3;

/// Points per curve for [`area`].
const SAMPLES: usize = 4096;

/// The area of a polygon through the ordered loop's sampled curves (for expected volumes).
fn area(curves: &[Curve2]) -> f64 {
    let pts: Vec<Point2<f64>> = curves
        .iter()
        .flat_map(|c| (0..SAMPLES).map(move |i| c.point_at(i as f64 / SAMPLES as f64)))
        .collect();
    let n = pts.len();
    (0..n)
        .map(|i| pts[i].x * pts[(i + 1) % n].y - pts[(i + 1) % n].x * pts[i].y)
        .sum::<f64>()
        .abs()
        / 2.0
}

#[track_caller]
fn extrudes(curves: Vec<Curve2>) {
    let want = 2.0 * area(&curves);
    let v = extrude(curves).unwrap();
    assert!(
        (v - want).abs() < VOLUME_REL * want,
        "volume {v}, want {want}"
    );
}

/// d4_desk_device_test, Extrude 18: a rectangle whose bottom edge misses the right edge's
/// start by 3.7e-4 mm (two lines that cross: both stay on their lines).
#[test]
fn a_rectangle_with_a_gap_of_four_tenths_of_a_micron() {
    extrudes(vec![
        line(
            p(20.100000000000016, 19.029999999999994),
            p(20.100000000000016, 20.050000000000004),
        ),
        line(
            p(20.100000000000012, 20.050000000000004),
            p(17.10000000000002, 20.050000000000004),
        ),
        line(
            p(17.10000000000002, 20.050000000000004),
            p(17.10000000000002, 19.0319969009892),
        ),
        line(
            p(17.10000000000002, 19.0319969009892),
            p(20.10000000000002, 19.030370114934005),
        ),
    ]);
}

/// emax gt2815/05, Extrude 3: an arc ends 5.4e-5 short of the line it is tangent to (they
/// don't cross: the line's end moves onto the arc's).
#[test]
fn an_arc_tangent_to_a_line_short_of_it() {
    let c = p(14.21775474532216, 5.889186842006865);
    let r = 4.889186842006873;
    let o = p(5.329070518200751e-15, -9.794584650479608e-16);
    extrudes(vec![
        arc(o, 17.5, 0.05717400108066306, 0.6710501612361226),
        line(
            p(13.061255881792384, 11.647042319419292),
            p(10.760577574851059, 9.346364012477965),
        ),
        arc(c, r, 2.3561944901923453, 0.20603610529762273),
        arc(
            p(5.488273299612761e-15, -9.794584650479608e-16),
            13.263494491577148,
            0.7021105148784044,
            -0.6188228663593605,
        ),
        arc(c, r, 4.506352875087066, 0.20603610529762362),
        line(p(14.217809070776289, 1.0), p(17.471405209656155, 1.0)),
    ]);
}

/// squish_bottle_adapter, Extrude 6: two arcs of nearly internally tangent circles, 7.1e-5 mm
/// apart at one joint (the shorter arc is refitted through its start, middle and the joint).
#[test]
fn two_arcs_nearly_tangent_with_a_gap() {
    extrudes(vec![
        arc(
            p(3.6864927272716216e-6, -7.851615757559773e-8),
            18.375,
            -3.307784318727326e-15,
            6.226154072614993,
        ),
        arc(
            p(-0.04307193012796443, -1.994225441078776e-5),
            18.418004616631404,
            -0.05689684674264036,
            0.05689792523415087,
        ),
    ]);
}

/// d4 desk device v2/v3, Extrude 1: a slot's end cap misses the outer arc by 1.1e-5 mm, just
/// over the joint tolerance (the loop closed by a line here).
#[test]
fn a_slot_cap_just_over_the_joint_tolerance() {
    let cap = arc(
        p(18.060592291222733, -23.36662121871194),
        0.5500000000492694,
        -0.7589245408948748,
        PI,
    );
    let outer = arc(
        p(19.675000000000004, -24.897736090538178),
        2.7750000004168633,
        2.3826720609363723,
        1.5913289357254117,
    );
    let back = line(outer.end(), cap.start());
    extrudes(vec![cap, outer, back]);
}

/// A Bézier curve whose end misses the line it is tangent to: its end pole moves.
#[test]
fn a_bezier_short_of_its_tangent_line() {
    let gap = 2e-4;
    extrudes(vec![
        Curve2::Bezier {
            poles: [p(0.0, 0.0), p(4.0, 0.0), p(10.0, 2.0), p(10.0, 6.0 - gap)],
            source: None,
        },
        line(p(10.0, 6.0), p(10.0, 10.0)),
        line(p(10.0, 10.0), p(0.0, 10.0)),
        line(p(0.0, 10.0), p(0.0, 0.0)),
    ]);
}

/// The curves listed out of order and direction still chain across a gap.
#[test]
fn shuffled_curves_with_a_gap() {
    let gap = 3e-4;
    let v = extrude(vec![
        line(p(10.0, 10.0), p(10.0, 0.0)),
        line(p(0.0, 0.0), p(0.0, 10.0)),
        line(p(10.0 - gap, 0.0), p(0.0, 0.0)),
        line(p(0.0, 10.0), p(10.0, 10.0)),
    ])
    .unwrap();
    assert!((v - 200.0).abs() < VOLUME_REL * 200.0, "{v}");
}

/// A loop open by a tenth of a millimetre is not closed.
#[test]
fn a_loop_open_by_a_tenth_stays_open() {
    let e = extrude(vec![
        line(p(0.0, 0.0), p(10.0, 0.0)),
        line(p(10.0, 0.0), p(10.0, 10.0)),
        line(p(10.0, 10.0), p(0.0, 10.0)),
        line(p(0.0, 10.0), p(0.0, 0.1)),
    ])
    .unwrap_err();
    assert!(matches!(e, KernelError::InvalidProfile(_)), "{e}");
}

/// plant_pot_dish_square: a straight edge used as an extrude direction points the way the
/// profile's loop runs (outer loops counter-clockwise about the plane's normal), as OCCT's
/// profile edges did, whichever way its sketch line was drawn.
#[test]
fn a_line_drawn_against_the_loop_runs_with_it() {
    // Drawn clockwise: the bottom line from (10, 0) to (0, 0).
    let curves = vec![
        line(p(0.0, 0.0), p(0.0, 10.0)),
        line(p(0.0, 10.0), p(10.0, 10.0)),
        line(p(10.0, 10.0), p(10.0, 0.0)),
        line(p(10.0, 0.0), p(0.0, 0.0)),
    ];
    let mut k = NativeKernel::new();
    let region = Region {
        outer: Loop { curves },
        holes: vec![],
        source: None,
    };
    let id = k
        .extrude(
            &Profile::new(Plane::top(), vec![region]),
            Extent::Blind(2.0),
        )
        .unwrap()
        .bodies[0];
    let bottom: Vec<EdgeInfo> = k
        .edges(id)
        .unwrap()
        .into_iter()
        .filter(|e| {
            [e.start.z, e.end.z, e.start.y, e.end.y]
                .iter()
                .all(|c| c.abs() < tol::ROUNDOFF)
        })
        .collect();
    assert_eq!(bottom.len(), 1);
    assert!(
        bottom[0].end.x > bottom[0].start.x,
        "{:?} → {:?}",
        bottom[0].start,
        bottom[0].end
    );
}

/// plant_pot_dish_square: two sketch lines in line make one side, as OCCT's merging of faces on
/// one surface made them, and that side's edge on the far cap runs back along the loop (the
/// way the side's boundary runs, as OCCT rebuilt it), so an extrude directed by that edge goes
/// the way the document expects; the near cap's edge runs with the loop.
#[test]
fn lines_in_line_make_one_side_whose_far_edge_runs_back() {
    // Counter-clockwise, its bottom drawn as two lines.
    let curves = vec![
        line(p(0.0, 0.0), p(4.0, 0.0)),
        line(p(4.0, 0.0), p(10.0, 0.0)),
        line(p(10.0, 0.0), p(10.0, 10.0)),
        line(p(10.0, 10.0), p(0.0, 10.0)),
        line(p(0.0, 10.0), p(0.0, 0.0)),
    ];
    let mut k = NativeKernel::new();
    let region = Region {
        outer: Loop { curves },
        holes: vec![],
        source: None,
    };
    // A feature's extrude (a DRAW-level prism keeps the two sides, as OCCT's does).
    let spec = ExtrudeSpec::blind(nalgebra::Vector3::z_axis(), 2.0);
    let id = k
        .extrude_with(&Profile::new(Plane::top(), vec![region]), &spec)
        .unwrap()
        .bodies[0];
    let issues = k.get(id).unwrap().check_with(CheckLevel::Full);
    assert!(issues.is_empty(), "{issues:?}");
    assert_eq!(k.faces(id).unwrap().len(), 6, "a box");
    let along_y0 = |z: f64| -> Vec<EdgeInfo> {
        k.edges(id)
            .unwrap()
            .into_iter()
            .filter(|e| {
                [e.start.y, e.end.y, e.start.z - z, e.end.z - z]
                    .iter()
                    .all(|c| c.abs() < tol::ROUNDOFF)
            })
            .collect()
    };
    let (near, far) = (along_y0(0.0), along_y0(2.0));
    assert_eq!((near.len(), far.len()), (1, 1));
    assert!(
        near[0].end.x > near[0].start.x,
        "near: {:?} → {:?}",
        near[0].start,
        near[0].end
    );
    assert!(
        far[0].end.x < far[0].start.x,
        "far: {:?} → {:?}",
        far[0].start,
        far[0].end
    );
}

/// squish_bottle_adapter, Extrude 6: a feature's extrude of a loop with a gap closed carries the
/// gap as the tolerance of its edges and vertices (OCCT's did), so a boolean can take its faces
/// for the faces of the sketch's other features they are that close to.
#[test]
fn a_closed_gap_is_carried_as_tolerance() {
    let gap = 3e-4;
    let curves = vec![
        line(p(0.0, 0.0), p(10.0, 0.0)),
        line(p(10.0, 0.0), p(10.0, 10.0)),
        line(p(10.0, 10.0), p(0.0, 10.0)),
        line(p(0.0, 10.0), p(0.0, gap)),
    ];
    let mut k = NativeKernel::new();
    let region = Region {
        outer: Loop { curves },
        holes: vec![],
        source: None,
    };
    let spec = ExtrudeSpec::blind(nalgebra::Vector3::z_axis(), 2.0);
    let id = k
        .extrude_with(&Profile::new(Plane::top(), vec![region]), &spec)
        .unwrap()
        .bodies[0];
    let b = k.get(id).unwrap();
    let carried = gap - tol::ROUNDOFF;
    assert!(b.edges.iter().all(|e| e.tol >= carried) && b.verts.iter().all(|v| v.tol >= carried));
    assert!(b.check_with(CheckLevel::Full).is_empty());
}
