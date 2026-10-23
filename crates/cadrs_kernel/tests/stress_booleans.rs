//! Boolean stress (research/PAPER_SCAN.md item 19), no reference kernel needed:
//! - a cube united with (and intersected by) copies of itself rotated by tiny angles about each
//!   axis and a skew one, sweeping across the modelling tolerance;
//! - iterated unions (12 by default) checked after each step for validity, volume against the
//!   closed form, and tolerance drift;
//! - an iterated drill-bit cut chain (EMBER's benchmark): blind 118° drills, each overlapping the
//!   last, cut one at a time; the result must equal the plate less the drills' union cut at once.
//!   Each cut's time is printed (`--nocapture`).
//!
//! Soak knobs: `CADRS_STRESS_ANGLES=n` adds n random angles (log-uniform, 1e-13 to 1e-2) about
//! random axes; `CADRS_STRESS_UNIONS`, `CADRS_STRESS_DRILLS` lengthen the chains;
//! `CADRS_META_SEED` seeds them (`random` for a fresh seed).

mod meta_common;

use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::*;
use meta_common::*;

/// The largest vertex or edge tolerance a result of exact inputs may have.
const TOL_CAP: f64 = 1e-5;

/// The volume of a union of axis-aligned boxes (min, size), exactly: over the cells of the grid
/// their faces make.
fn boxes_union_volume(boxes: &[([f64; 3], [f64; 3])]) -> f64 {
    let axis = |i: usize| {
        let mut c: Vec<f64> = boxes
            .iter()
            .flat_map(|(m, s)| [m[i], m[i] + s[i]])
            .collect();
        c.sort_by(f64::total_cmp);
        c.dedup();
        c
    };
    let (xs, ys, zs) = (axis(0), axis(1), axis(2));
    let mut v = 0.0;
    for x in xs.windows(2) {
        for y in ys.windows(2) {
            for z in zs.windows(2) {
                let c = [
                    (x[0] + x[1]) / 2.0,
                    (y[0] + y[1]) / 2.0,
                    (z[0] + z[1]) / 2.0,
                ];
                let inside = boxes
                    .iter()
                    .any(|(m, s)| (0..3).all(|i| c[i] > m[i] && c[i] < m[i] + s[i]));
                if inside {
                    v += (x[1] - x[0]) * (y[1] - y[0]) * (z[1] - z[0]);
                }
            }
        }
    }
    v
}

/// The volumes of cube ∪ turned cube and cube ∩ turned cube to first order in the angle, for the
/// cube [0, 10]³ turned by `angle` about `axis` through `about`: each face of the turned cube
/// lies off the cube's face by δ(x) = angle (ω × (x − c))·n, linear over the face, so the union
/// gains ∫ max(δ, 0) over every face and the common part loses ∫ max(−δ, 0). The error is
/// second order (about angle² × 6·10⁴ here).
fn first_order_volumes(axis: [f64; 3], angle: f64, about: [f64; 3]) -> (f64, f64) {
    let w = nalgebra::Vector3::from(axis).normalize() * angle;
    let c = nalgebra::Vector3::from(about);
    let (mut gain, mut loss) = (0.0, 0.0);
    for i in 0..3 {
        for side in [0.0, 10.0] {
            let mut n = nalgebra::Vector3::zeros();
            n[i] = if side == 0.0 { -1.0 } else { 1.0 };
            let (j, k) = ((i + 1) % 3, (i + 2) % 3);
            // δ at a face point (s, t): linear, so exact over the square clipped where δ > 0.
            let delta = |s: f64, t: f64| {
                let mut x = nalgebra::Vector3::zeros();
                x[i] = side;
                x[j] = s;
                x[k] = t;
                w.cross(&(x - c)).dot(&n)
            };
            let square = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
            for sign in [1.0, -1.0] {
                // Clip the square to sign·δ > 0 (one half-plane: Sutherland–Hodgman).
                let f = |p: (f64, f64)| sign * delta(p.0, p.1);
                let mut poly = Vec::new();
                for e in 0..4 {
                    let (p, q) = (square[e], square[(e + 1) % 4]);
                    let (fp, fq) = (f(p), f(q));
                    if fp > 0.0 {
                        poly.push(p);
                    }
                    if (fp > 0.0) != (fq > 0.0) {
                        let u = fp / (fp - fq);
                        poly.push((p.0 + u * (q.0 - p.0), p.1 + u * (q.1 - p.1)));
                    }
                }
                // ∫ over the polygon of a linear function: by triangles, area × the mean of
                // its corners' values.
                let mut v = 0.0;
                for e in 1..poly.len().saturating_sub(1) {
                    let (p, q, r) = (poly[0], poly[e], poly[e + 1]);
                    let area = ((q.0 - p.0) * (r.1 - p.1) - (r.0 - p.0) * (q.1 - p.1)).abs() / 2.0;
                    v += area * (f(p) + f(q) + f(r)) / 3.0;
                }
                if sign > 0.0 {
                    gain += v;
                } else {
                    loss += v;
                }
            }
        }
    }
    (1000.0 + gain, 1000.0 - loss)
}

/// One tiny-rotation case: the failures of cube ∪ turned cube and cube ∩ turned cube, in both
/// tool orders. Up to an angle of 1e-4 each volume must match the first-order volumes within
/// the result's own tolerance band; beyond, the union and the common part must still add up.
fn tiny_rotation(axis: [f64; 3], angle: f64, about: [f64; 3]) -> Vec<String> {
    let mut k = NativeKernel::new();
    let cube = Shape::Box {
        min: [0.0; 3],
        size: [10.0; 3],
    };
    let a = build(&mut k, &Placed::fixed(cube)).unwrap();
    let rot = Placed {
        shape: cube,
        axis,
        angle,
        about,
    };
    let b = build(&mut k, &rot).unwrap();
    // How far any point of the cube moves: |angle| × the farthest corner from the centre.
    let reach =
        angle.abs() * 3f64.sqrt() * (10.0 + about.iter().map(|c| c.abs()).fold(0.0, f64::max));
    let slack = 600.0 * reach + 1e-7 * 1000.0;
    let truth = (angle.abs() <= 1e-4).then(|| first_order_volumes(axis, angle, about));
    let mut bad = Vec::new();
    let mut vols = [f64::NAN; 2];
    let mut bands = [0.0; 2];
    for (x, y, order) in [(a, b, ""), (b, a, " (turned cube first)")] {
        for (n, (op, what)) in [(BoolOp::Union, "∪"), (BoolOp::Intersect, "∩")]
            .into_iter()
            .enumerate()
        {
            let what = format!("{what}{order}");
            match boolean(&mut k, op, x, &[y]) {
                Ok(r) => {
                    if let Some(why) = invalid(&k, &r) {
                        bad.push(format!("{what}: invalid: {why}"));
                    }
                    if r.len() != 1 {
                        bad.push(format!("{what}: {} bodies", r.len()));
                    }
                    let t = max_tol(&k, &r);
                    if t > TOL_CAP.max(reach) {
                        bad.push(format!("{what}: tolerance {t:e}"));
                    }
                    let v = volume(&k, &r).unwrap_or(f64::NAN);
                    let bd = band(&k, &r);
                    if !((v - 1000.0).abs() <= slack + bd) {
                        bad.push(format!("{what}: volume {v} (1000 ± {slack:e})"));
                    }
                    if let Some((tu, ti)) = truth {
                        let want = if n == 0 { tu } else { ti };
                        let allow = bd + 1e-7 * 1000.0 + 6e4 * angle * angle;
                        if !((v - want).abs() <= allow) {
                            bad.push(format!(
                                "{what}: volume {v}, first-order {want} (off {:.2e}, tolerance band {bd:.1e})",
                                v - want
                            ));
                        }
                    }
                    if order.is_empty() {
                        vols[n] = v;
                        bands[n] = bd;
                    }
                }
                Err(e) => bad.push(format!("{what}: {e:?}")),
            }
        }
    }
    if !((vols[0] + vols[1] - 2000.0).abs() <= 1e-7 * 2000.0 + bands[0] + bands[1]) {
        bad.push(format!("vol(∪) + vol(∩) = {} (2000)", vols[0] + vols[1]));
    }
    bad
}

#[test]
fn cube_and_tiny_rotations() {
    tiny_rotations(
        &[1e-12, 1e-10, 1e-8, 1e-7, 1e-4, 1e-2],
        env_num("CADRS_STRESS_ANGLES", 0),
    );
}

/// Tilts of 1e-6 and 1e-5 (the cube's corners move 1e-5 to 1e-4): the boolean takes the
/// near-coincident faces as one and returns the cube itself, in both tool orders, the union
/// ~500·angle of volume short and the common part as much over; its faces are up to ~9e-5 from
/// where the inputs had them, and its tolerances must say so (they were left at 1e-7).
#[test]
fn cube_tilted_into_coincidence() {
    tiny_rotations(&[1e-6, 1e-5], 0);
}

/// The tiny-rotation cases at `angles` about each axis and a skew one through the cube's centre
/// (and the skew one through its corner), plus `random` random ones.
fn tiny_rotations(angles: &[f64], random: usize) {
    let axes = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
    ];
    let mut cases: Vec<([f64; 3], f64, [f64; 3])> = Vec::new();
    for &angle in angles {
        for axis in axes {
            // About the cube's centre, and (for a tilt on top of the shift) about its corner.
            cases.push((axis, angle, [5.0; 3]));
        }
        cases.push(([1.0, 1.0, 1.0], angle, [0.0; 3]));
    }
    let mut r = Rng::new(seed(7));
    for _ in 0..random {
        let axis = [r.range(-1.0, 1.0), r.range(-1.0, 1.0), r.range(-1.0, 1.0)];
        let angle = 10f64.powf(r.range(-13.0, -2.0)) * if r.below(2) == 0 { 1.0 } else { -1.0 };
        let about = [r.range(0.0, 10.0), r.range(0.0, 10.0), r.range(0.0, 10.0)];
        cases.push((axis, angle, about));
    }
    let mut fails = Vec::new();
    for (axis, angle, about) in &cases {
        let bad = tiny_rotation(*axis, *angle, *about);
        if !bad.is_empty() {
            fails.push(format!(
                "axis {axis:?} angle {angle:e} about {about:?}: {}",
                bad.join("; ")
            ));
        }
    }
    assert!(
        fails.is_empty(),
        "{} of {} cases fail:\n{}",
        fails.len(),
        cases.len(),
        fails.join("\n")
    );
}

/// Boxes in a staircase, each overlapping the last and some earlier ones, united one at a time:
/// after each union the result is valid, its volume the exact union's, its tolerance exact.
#[test]
fn iterated_box_unions() {
    let n = env_num("CADRS_STRESS_UNIONS", 12usize);
    let mut r = Rng::new(seed(11));
    let boxes: Vec<([f64; 3], [f64; 3])> = (0..n)
        .map(|i| {
            let jitter = if i == 0 {
                0.0
            } else {
                r.snapped(-0.5, 0.5, 0.25)
            };
            let min = [
                i as f64 * 2.5 + jitter,
                (i % 3) as f64 * 1.5,
                (i % 2) as f64 * 1.25,
            ];
            (min, [5.0, 4.0 + (i % 4) as f64 * 0.5, 3.0])
        })
        .collect();
    let mut k = NativeKernel::new();
    let mut acc = build(
        &mut k,
        &Placed::fixed(Shape::Box {
            min: boxes[0].0,
            size: boxes[0].1,
        }),
    )
    .unwrap();
    let mut fails = Vec::new();
    for i in 1..n {
        let b = build(
            &mut k,
            &Placed::fixed(Shape::Box {
                min: boxes[i].0,
                size: boxes[i].1,
            }),
        )
        .unwrap();
        let r = match boolean(&mut k, BoolOp::Union, acc, &[b]) {
            Ok(r) if r.len() == 1 => r,
            other => {
                fails.push(format!("union {i}: {other:?}"));
                break;
            }
        };
        acc = r[0];
        let want = boxes_union_volume(&boxes[..=i]);
        let v = volume(&k, &r).unwrap_or(f64::NAN);
        if !((v - want).abs() <= 1e-9 * want) {
            fails.push(format!("union {i}: volume {v}, exact {want}"));
        }
        if let Some(why) = invalid(&k, &r) {
            fails.push(format!("union {i}: invalid: {why}"));
        }
        let t = max_tol(&k, &r);
        if t > TOL_CAP {
            fails.push(format!("union {i}: tolerance {t:e}"));
        }
    }
    assert!(fails.is_empty(), "boxes {boxes:?}:\n{}", fails.join("\n"));
}

/// A cube united with copies of itself turned about its vertical axis by growing small angles:
/// a fan of nearly coincident faces. Each step's volume grows, by no more than the turn sweeps.
#[test]
fn iterated_rotated_unions() {
    let n = env_num("CADRS_STRESS_UNIONS", 12usize);
    let cube = Shape::Box {
        min: [-5.0, -5.0, 0.0],
        size: [10.0; 3],
    };
    let mut k = NativeKernel::new();
    let mut acc = build(&mut k, &Placed::fixed(cube)).unwrap();
    let mut last = 1000.0;
    let mut fails = Vec::new();
    for i in 1..n {
        let angle = 1e-3 * i as f64;
        let b = build(
            &mut k,
            &Placed {
                shape: cube,
                axis: [0.0, 0.0, 1.0],
                angle,
                about: [0.0; 3],
            },
        )
        .unwrap();
        let r = match boolean(&mut k, BoolOp::Union, acc, &[b]) {
            Ok(r) if r.len() == 1 => r,
            other => {
                fails.push(format!("union {i} (angle {angle}): {other:?}"));
                break;
            }
        };
        acc = r[0];
        let v = volume(&k, &r).unwrap_or(f64::NAN);
        // A square of side 10 turned by δ about its centre sticks out of the last one by 4
        // slivers of about ½·5·5δ each: 500δ of volume over the height 10 (twice that allowed).
        if !(v >= last - 1e-9 * v && v <= last + 2.0 * 500.0 * 1e-3 + 1e-9 * v) {
            fails.push(format!("union {i}: volume {v} after {last}"));
        }
        last = v;
        if let Some(why) = invalid(&k, &r) {
            fails.push(format!("union {i}: invalid: {why}"));
        }
        let t = max_tol(&k, &r);
        if t > TOL_CAP {
            fails.push(format!("union {i}: tolerance {t:e}"));
        }
    }
    assert!(fails.is_empty(), "{}", fails.join("\n"));
}

/// The drill positions of the chain: along x, each 1.7 past the last (drills of radius 2.5
/// overlap), wandering a little in y.
fn drills(n: usize) -> Vec<Placed> {
    (0..n)
        .map(|i| {
            Placed::fixed(Shape::Drill {
                tip: [
                    5.0 + 1.7 * i as f64,
                    10.0 + 1.5 * (i as f64 * 0.7).sin(),
                    3.0,
                ],
                r: 2.5,
                len: 20.0,
            })
        })
        .collect()
}

#[test]
fn drill_chain() {
    let n = env_num("CADRS_STRESS_DRILLS", 12usize);
    let plate = Placed::fixed(Shape::Box {
        min: [0.0; 3],
        size: [10.0 + 1.7 * n as f64, 20.0, 10.0],
    });
    let ds = drills(n);
    let mut k = NativeKernel::new();
    let mut acc = build(&mut k, &plate).unwrap();
    let mut last = volume(&k, &[acc]).unwrap();
    let mut fails = Vec::new();
    let t0 = std::time::Instant::now();
    for (i, d) in ds.iter().enumerate() {
        let t = std::time::Instant::now();
        let tool = build(&mut k, d).unwrap();
        let r = match boolean(&mut k, BoolOp::Subtract, acc, &[tool]) {
            Ok(r) if r.len() == 1 => r,
            other => {
                fails.push(format!("cut {i}: {other:?}"));
                break;
            }
        };
        acc = r[0];
        let v = volume(&k, &r).unwrap_or(f64::NAN);
        println!(
            "drill_chain: cut {i} in {:.1} ms, {} faces",
            t.elapsed().as_secs_f64() * 1e3,
            k.get(acc).map(|b| b.faces().len()).unwrap_or(0)
        );
        if !(v < last && v > last - exact_volume(&d.shape)) {
            fails.push(format!("cut {i}: volume {v} after {last}"));
        }
        last = v;
        if let Some(why) = invalid(&k, &r) {
            fails.push(format!("cut {i}: invalid: {why}"));
        }
        let t = max_tol(&k, &r);
        if t > TOL_CAP {
            fails.push(format!("cut {i}: tolerance {t:e}"));
        }
    }
    println!(
        "drill_chain: {n} cuts in {:.1} ms",
        t0.elapsed().as_secs_f64() * 1e3
    );
    // The same holes cut at once: the plate less the drills' union.
    if fails.is_empty() {
        let mut u = build(&mut k, &ds[0]).unwrap();
        for d in &ds[1..] {
            let tool = build(&mut k, d).unwrap();
            match boolean(&mut k, BoolOp::Union, u, &[tool]) {
                Ok(r) if r.len() == 1 => u = r[0],
                other => {
                    fails.push(format!("drills' union: {other:?}"));
                    break;
                }
            }
        }
        if fails.is_empty() {
            let p = build(&mut k, &plate).unwrap();
            match boolean(&mut k, BoolOp::Subtract, p, &[u]) {
                Ok(r) => {
                    let v = volume(&k, &r).unwrap_or(f64::NAN);
                    if !((v - last).abs() <= 1e-7 * last) {
                        fails.push(format!("cut one at a time: {last}; at once: {v}"));
                    }
                }
                Err(e) => fails.push(format!("plate − union: {e:?}")),
            }
        }
    }
    assert!(fails.is_empty(), "{}", fails.join("\n"));
}

/// A tilt of about 1e-5 about a skew axis (the cube's corners move ~1e-4, about the edges'
/// tolerance band): the union and the common part left open shells at every tolerance of the
/// ladder (found by the soak; the tilts 1.1e-5 to 1.5e-5, some with an edge used by three
/// faces), the faces' gap above the tolerance in places and below it in others.
#[test]
fn cube_tilted_near_tolerance() {
    let bad = tiny_rotation(
        [0.5976071051500398, 0.511301556512727, -0.5102939961996196],
        -1.161043810954499e-5,
        [2.809667712018844, 9.873518856878142, 3.249429577303461],
    );
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// A tilt of 1.2e-6 about a skew axis (soak): with the turned cube first, the union and the
/// common part were the turned cube itself, tolerance 1e-7, 9e-4 off the first-order volumes
/// (with the cube first, one merged cube of tolerance 1e-5, within its band).
#[test]
fn cube_tilted_volume_sum() {
    let bad = tiny_rotation(
        [0.4700611935019128, -0.9620889946033548, -0.8509348303880513],
        1.2181008428462193e-6,
        [4.626890525898524, 6.837536705820911, 2.1387233918948976],
    );
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

// --- Soak findings (the build box's worker), replayed at kernel-perf d0bcb72 and still failing,
// pinned by class: each test asserts every tilt of its class passes `tiny_rotation`, and is a
// known failure until they all do.

fn rotations_all(cases: &[([f64; 3], f64, [f64; 3])]) {
    let bad: Vec<String> = cases
        .iter()
        .filter_map(|&(axis, angle, about)| {
            let v = tiny_rotation(axis, angle, about);
            (!v.is_empty()).then(|| {
                format!(
                    "axis {axis:?} angle {angle:e} about {about:?}: {}",
                    v.join("; ")
                )
            })
        })
        .collect();
    assert!(
        bad.is_empty(),
        "{} of {} cases fail:\n{}",
        bad.len(),
        cases.len(),
        bad.join("\n")
    );
}

/// Soak: 5 tilts of the cube where the boolean fails at its validity stage: an edge used by three or more faces.
#[test]
#[ignore = "known failure: the boolean fails at its validity stage: an edge used by three or more faces"]
fn soak_tilt_edge_three_faces() {
    rotations_all(&[
        (
            [0.8762761093315652, 0.01967810629427791, -0.153553390426864],
            7.362529730502069e-5,
            [1.411776503513087, 9.755128539055931, 1.3891384016625796],
        ),
        (
            [-0.776459358485549, -0.5242898990460925, 0.375718774073873],
            -2.456399921982705e-5,
            [5.074699582579964, 3.3966019231597144, 7.052922954052175],
        ),
        (
            [
                -0.0679252642200836,
                -0.03952024902392037,
                0.8144591099934844,
            ],
            1.3560583257832132e-4,
            [0.9610013718125909, 9.68294457369963, 5.8498677538445065],
        ),
        (
            [
                0.028517646576798228,
                -0.9400944885778768,
                -0.7448020164713773,
            ],
            2.527632950967343e-5,
            [3.2778738239848226, 4.11096001169877, 3.3071380719005603],
        ),
        (
            [0.34094960166350363, 0.3333236213223534, 0.19277775570575062],
            2.7397990983956118e-5,
            [2.278341762704054, 2.2463924088381493, 8.241080354745378],
        ),
    ]);
}

/// Soak: 3 tilts of the cube where the cube ∩ its copy turned by about 1e-5 comes back empty (0 bodies) instead of about 1000.
#[test]
#[ignore = "known failure: the cube ∩ its copy turned by about 1e-5 comes back empty (0 bodies) instead of about 1000"]
fn soak_tilt_empty_intersection() {
    rotations_all(&[
        (
            [
                0.1981229154911317,
                0.37835552617498847,
                -0.04288502950177131,
            ],
            -9.896415773506805e-7,
            [2.906213636236481, 2.477716711638972, 5.523572530431569],
        ),
        (
            [
                0.3789906616257597,
                -0.8173147181710525,
                -0.03491878721269459,
            ],
            -1.0120305952052657e-6,
            [5.303535937650979, 5.873052561610192, 4.9173584920591615],
        ),
        (
            [
                0.4311375680284688,
                -0.7365856246739586,
                0.026588015156655098,
            ],
            8.717219752322903e-7,
            [2.8264146959333214, 7.829477491867993, 4.863630877587573],
        ),
    ]);
}

/// Soak: 26 tilts of the cube where the volume misses the first-order closed form by more than the tolerance band.
#[test]
#[ignore = "known failure: the volume misses the first-order closed form by more than the tolerance band"]
fn soak_tilt_first_order_volume() {
    rotations_all(&[
        (
            [-0.3635984955647158, 0.11815220674368687, 0.3492105600219111],
            -2.2505568637625233e-5,
            [2.4268692498978184, 1.6597891947000953, 3.5544214853620524],
        ),
        (
            [-0.7510860231242675, -0.18589804912330177, -0.32663362560361],
            -3.0053587374594055e-7,
            [6.570287190374362, 4.373080327124694, 4.895681649969331],
        ),
        (
            [
                -0.9368779803341745,
                -0.3650392795983588,
                0.03422732174801579,
            ],
            3.6651937917188893e-6,
            [8.6032630051715, 1.7582573610415053, 2.671356896821125],
        ),
        (
            [0.9059210366819443, -0.6524004869236997, -0.7442671742368565],
            1.625236522194934e-5,
            [1.2109976308936643, 5.016721827528024, 7.34071338091304],
        ),
        (
            [
                -0.2861755874248315,
                -0.20335384414196778,
                0.3003433794547017,
            ],
            -2.1931784082735574e-6,
            [1.0812532466994196, 0.2553943868311481, 3.032747515238362],
        ),
        (
            [
                0.6077896681648065,
                0.30835307807768864,
                -0.34274855138001437,
            ],
            -2.8701991954021724e-7,
            [7.639123043432706, 0.6780367937735099, 4.48587767668193],
        ),
        (
            [-0.6539587584986033, -0.6991767016975334, 0.5376143763545969],
            -2.901308795312403e-6,
            [7.3432641882244285, 2.627452596350058, 1.482330718806113],
        ),
        (
            [
                -0.7451892095133525,
                -0.5868284773966617,
                -0.9215041045916161,
            ],
            -2.1463427526054377e-7,
            [3.3322049702760017, 9.935076395638411, 8.883160689910408],
        ),
        (
            [0.09649282328338638, 0.567026706951089, -0.6286622190608069],
            -2.0768834913815806e-6,
            [2.042882594679681, 5.464946777568511, 9.473468632746142],
        ),
        (
            [
                -0.22820329639947778,
                -0.2182866450536598,
                0.04100890020118242,
            ],
            -2.3769391522039845e-5,
            [3.1661678384911074, 8.695854884043056, 3.8171368149956164],
        ),
        (
            [
                -0.5428319929438392,
                -0.8238006521591872,
                -0.9223394888353327,
            ],
            2.9080551070713943e-6,
            [0.2701241245920061, 2.1943456974629703, 5.196710257835191],
        ),
        (
            [
                -0.20020874638904607,
                0.04971852772840646,
                -0.15675953300433454,
            ],
            2.1944186125146686e-5,
            [2.237745247047074, 3.5331941125681423, 6.284008287308016],
        ),
        (
            [0.4487534988206541, 0.5129149758448, -0.7795326773471665],
            2.2053199562748197e-6,
            [9.538342879966404, 9.075572697679753, 6.341478978389351],
        ),
        (
            [
                0.043350726708261345,
                0.8650371207746848,
                -0.20209380507448826,
            ],
            -2.9457219493293266e-7,
            [2.7754014284061412, 9.067543446604985, 3.3000872094180433],
        ),
        (
            [0.5916742052020632, 0.7946618407489696, 0.9890901946715305],
            2.9785775252753974e-5,
            [2.1063611806905422, 3.5249727912840556, 9.501199273534427],
        ),
        (
            [
                0.8538588860574547,
                -0.6594252222165367,
                -0.29982506055542446,
            ],
            -4.839564629904798e-7,
            [1.209707523263407, 0.014302681433030484, 1.6801804537160792],
        ),
        (
            [-0.3731231439099849, 0.16050516728773512, 0.8729409943893489],
            -5.2182395824247964e-6,
            [5.11382587612581, 1.5019119339901787, 8.793142314318683],
        ),
        (
            [
                -0.7944186158678177,
                -0.11951455088368168,
                0.16621366470165833,
            ],
            4.950153362570798e-7,
            [4.011196994154966, 0.7795530666802486, 1.9891616622561015],
        ),
        (
            [
                -0.3883573969693239,
                -0.514972573012666,
                -0.0032644194752846634,
            ],
            2.6885191171113945e-6,
            [1.697659324350097, 2.383451484681729, 0.3082930631863978],
        ),
        (
            [0.7348133321673285, 0.21200246870847694, -0.5336000208915597],
            -2.3913050017946395e-7,
            [6.100142318089105, 3.8473865735282153, 7.579476451589718],
        ),
        (
            [
                -0.2931600556031828,
                -0.9786116569200434,
                -0.39998142393528124,
            ],
            -1.6422682789218898e-7,
            [0.09453333924504759, 2.943114269266184, 7.804621365554569],
        ),
        (
            [0.40266755309186175, 0.3340365821967597, 0.4744250684387239],
            2.636723237993767e-6,
            [9.397115993033124, 9.57721096961733, 4.528621044425848],
        ),
        (
            [
                -0.10097064891123209,
                0.18937100864247114,
                0.11284536242547949,
            ],
            2.0731699009495216e-5,
            [0.7190775101032387, 9.378237965462533, 1.8225101507094377],
        ),
        (
            [
                -0.47242487876006334,
                -0.912966997587561,
                0.45296476979142564,
            ],
            2.0500288434439497e-5,
            [2.2417619269228704, 8.028112451694529, 5.656601747145777],
        ),
        (
            [
                -0.6164768683287833,
                -0.23884065386775744,
                0.3208246369457559,
            ],
            -3.829567370094054e-6,
            [4.312948136321923, 3.6370026478585595, 2.0592935614521535],
        ),
        (
            [
                -0.027967758939338783,
                -0.779058905654074,
                0.2912207282913608,
            ],
            2.762970032885116e-5,
            [0.2487937064692003, 3.3325213832997935, 0.3259169914766158],
        ),
    ]);
}

/// Soak: 3 tilts of the cube where the boolean leaves an open shell.
#[test]
#[ignore = "known failure: the boolean leaves an open shell"]
fn soak_tilt_open_shell() {
    rotations_all(&[
        (
            [
                -0.08502435272398423,
                0.9887794560276653,
                -0.11776074342281806,
            ],
            1.4552961855454857e-4,
            [0.853495679095555, 6.73727889356134, 9.701019573107047],
        ),
        (
            [
                -0.09215391421266705,
                -0.9830412426717356,
                0.42838007271190337,
            ],
            8.662669490385993e-5,
            [9.71535244239534, 0.47108050732773465, 1.7759716459453645],
        ),
        (
            [
                -0.25900957635098676,
                0.054010836600590695,
                -0.5451139829324807,
            ],
            -2.7929182568903676e-5,
            [6.221494868282937, 9.270984362180402, 7.346423039644808],
        ),
    ]);
}

/// Soak: 24 tilts of the cube where the result's vertex or edge tolerance grows past both 1e-5 and how far the tilt moves the cube.
#[test]
#[ignore = "known failure: the result's vertex or edge tolerance grows past both 1e-5 and how far the tilt moves the cube"]
fn soak_tilt_tolerance_growth() {
    rotations_all(&[
        (
            [0.4485789176575756, -0.0851864345990474, 0.8908391143012699],
            2.5840500367181814e-5,
            [
                1.7269382004784917,
                0.26432037691835886,
                0.010613080113605067,
            ],
        ),
        (
            [
                -0.2729927425215679,
                0.34508830857600725,
                -0.3283414519919847,
            ],
            2.6247831711586254e-5,
            [1.9344607455032214, 0.5523598785438133, 0.13986369586270864],
        ),
        (
            [-0.06311991244107351, 0.066434973902318, 0.37481907599636677],
            -3.8096907054949565e-5,
            [2.8612904696397043, 0.4215845153014841, 1.0295254198090142],
        ),
        (
            [
                -0.1730640632691025,
                -0.11215383924830102,
                0.33201802587218876,
            ],
            3.787051859806621e-5,
            [2.1813242447968317, 1.0058129880703603, 1.1256124089707564],
        ),
        (
            [0.6491197462771818, 0.28656347739781407, 0.18879416908176072],
            1.5599108030907825e-5,
            [1.6169437847159218, 0.12134139097895291, 1.1586021601703356],
        ),
        (
            [0.5064459415460545, -0.3427094771018653, -0.8794040853349756],
            1.834405611677491e-5,
            [0.6999780258474775, 2.6552166378336075, 1.781996906050598],
        ),
        (
            [-0.3907228177848525, -0.299537740732277, -0.6604910578715344],
            1.5937107340253338e-5,
            [0.07137033302846207, 1.5438056725172, 2.0598824709622865],
        ),
        (
            [0.5838206512418735, 0.19941156566805818, 0.7457437489023195],
            1.5008419857917143e-5,
            [0.060631618782753804, 0.7721130131175957, 0.6217001559510871],
        ),
        (
            [
                -0.27451410387493813,
                0.9067346627791071,
                -0.5636955477895105,
            ],
            -1.3472900683051888e-5,
            [0.9108295854823134, 2.439978214939887, 0.0728138526670441],
        ),
        (
            [-0.6751257954449601, 0.6428396854563208, 0.7561772171114429],
            -1.4302634221326962e-5,
            [0.16768959939934436, 3.0863018592424165, 1.781355062893898],
        ),
        (
            [0.10336035755212203, 0.5633447577087283, -0.5404813258709624],
            -2.483673780335358e-5,
            [1.8771709881086118, 1.417522961120652, 2.460716728625668],
        ),
        (
            [
                -0.7790132330054365,
                -0.29748725371212736,
                0.43154841548629874,
            ],
            1.3710046524109981e-5,
            [2.361090592020113, 0.5910813905618995, 2.0503642503814556],
        ),
        (
            [0.2590534014203132, -0.8310608476194916, -0.5066660594130072],
            1.4189271681622316e-5,
            [1.786983343794355, 0.93164837708683, 1.180595839352957],
        ),
        (
            [0.8800401803572997, -0.9908965077026437, 0.7038388016765356],
            2.2933187323774197e-5,
            [0.6456283338683322, 0.5558337209645126, 0.05996609017820398],
        ),
        (
            [0.1783454624028098, -0.13699297694892243, -0.771543832631127],
            1.141950109460558e-5,
            [1.5469303277373903, 0.38944536576155486, 2.792881682788991],
        ),
        (
            [
                -0.7884124400552346,
                0.36768653118730477,
                0.27829881400163425,
            ],
            -3.555231146021725e-5,
            [1.3258169429471667, 0.8792643612211803, 2.9928784596047917],
        ),
        (
            [
                -0.04172864999538306,
                -0.18028969224774682,
                -0.4066878586859273,
            ],
            -1.8787436683305345e-5,
            [1.6869060755125664, 0.23723332816599885, 0.06071315472949257],
        ),
        (
            [
                0.3619491223979745,
                -0.6324036415742402,
                -0.37073569329182576,
            ],
            -1.5245913119307003e-5,
            [1.2401585285785865, 1.3339204543817584, 2.182113267987048],
        ),
        (
            [
                0.7321181390623195,
                -0.5115528390105413,
                -0.07477290532273484,
            ],
            2.0909150036614452e-5,
            [2.830655178482119, 2.9810675784632923, 0.699186163644866],
        ),
        (
            [
                -0.35119437900267014,
                -0.9794383263768978,
                0.9072529583771545,
            ],
            -1.804215409145055e-5,
            [0.3049170579527116, 2.441829445158372, 2.7467821631004217],
        ),
        (
            [-0.0965904447390038, 0.3318119188954143, -0.5740357041811874],
            -8.072006482130552e-6,
            [0.30688757049035065, 0.5830341891164625, 2.3643675280850562],
        ),
        (
            [
                -0.5859352436175478,
                -0.2977036034988476,
                0.12129561619283602,
            ],
            -1.396317535406694e-5,
            [3.3649336015040845, 0.6286614856620942, 1.3370680045186922],
        ),
        (
            [
                -0.38924939072571174,
                0.9664065628240788,
                -0.15891404616271143,
            ],
            8.928619355200581e-6,
            [0.7448133289045356, 3.540367258047361, 3.162824896336839],
        ),
        (
            [
                -0.39668778057996534,
                -0.7403415074527615,
                0.8850445484648282,
            ],
            1.8192480262987858e-5,
            [1.4812223929383295, 2.3029423827734155, 2.0367652822314373],
        ),
    ]);
}
