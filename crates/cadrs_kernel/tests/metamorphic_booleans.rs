//! Metamorphic booleans (research/PAPER_SCAN.md item 18): relations every correct boolean keeps,
//! checked on generated solids without a reference kernel.
//!
//! For each random pair A, B (boxes, cylinders along an axis, spheres, some rotated, half their
//! coordinates on a grid so faces and edges coincide):
//! - vol(A ∪ B) + vol(A ∩ B) = vol(A) + vol(B), and vol(A − B) + vol(A ∩ B) = vol(A);
//! - tool order: vol(B ∪ A) = vol(A ∪ B), vol(B ∩ A) = vol(A ∩ B);
//! - every result valid (`CheckLevel::Full`) and a solid;
//! - the union written to STEP and read back keeps its volume and validity.
//!
//! For each shape on its own: A − A = ∅, A ∪ A = A, A ∩ A = A.
//!
//! `CADRS_META_CASES` sets the number of pairs (default 16, a few seconds), `CADRS_META_SEED` the
//! seed (`random` for a fresh one, printed), `CADRS_META_CASE=i` runs case i alone. A failing case prints its seed, index and shapes;
//! a failure the kernel has today is pinned below as its own `#[ignore = "known failure…"]` test
//! and skipped by the random run (`KNOWN`).

mod meta_common;

use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::*;
use meta_common::*;

/// Relative volume tolerance: mass properties are exact for these surfaces to about 1e-12.
const REL: f64 = 1e-7;

/// (seed, case) pairs of the default run that fail today: each has a pinned test below.
const KNOWN: &[(u64, usize)] = &[];

/// A random shape about the cube [0, 10]³.
fn shape(r: &mut Rng) -> Placed {
    let s = match r.below(3) {
        0 => Shape::Box {
            min: [
                r.snapped(-2.0, 8.0, 1.0),
                r.snapped(-2.0, 8.0, 1.0),
                r.snapped(-2.0, 8.0, 1.0),
            ],
            size: [
                r.snapped(1.0, 8.0, 1.0),
                r.snapped(1.0, 8.0, 1.0),
                r.snapped(1.0, 8.0, 1.0),
            ],
        },
        1 => Shape::Cyl {
            axis: r.below(3),
            base: [
                r.snapped(0.0, 10.0, 1.0),
                r.snapped(0.0, 10.0, 1.0),
                r.snapped(-2.0, 4.0, 1.0),
            ],
            r: r.snapped(0.5, 4.0, 0.5),
            len: r.snapped(1.0, 10.0, 1.0),
        },
        _ => Shape::Sphere {
            c: [
                r.snapped(0.0, 10.0, 1.0),
                r.snapped(0.0, 10.0, 1.0),
                r.snapped(0.0, 10.0, 1.0),
            ],
            r: r.snapped(0.5, 5.0, 0.5),
        },
    };
    let mut p = Placed::fixed(s);
    // A third of them rotated about a random axis through the middle of the cube.
    if r.below(3) == 0 {
        p.axis = [r.range(-1.0, 1.0), r.range(-1.0, 1.0), r.range(-1.0, 1.0)];
        p.angle = r.range(-3.0, 3.0);
        p.about = [5.0; 3];
    }
    p
}

fn close(a: f64, b: f64, scale: f64) -> bool {
    (a - b).abs() <= REL * scale.max(1.0)
}

/// The failures of the relations on one pair (empty when they all hold).
fn check_pair(a: &Placed, b: &Placed) -> Vec<String> {
    let mut k = NativeKernel::new();
    let mut bad = Vec::new();
    let (ia, ib) = match (build(&mut k, a), build(&mut k, b)) {
        (Ok(x), Ok(y)) => (x, y),
        (x, y) => return vec![format!("build: {:?} {:?}", x.err(), y.err())],
    };
    let va = volume(&k, &[ia]).unwrap_or(f64::NAN);
    let vb = volume(&k, &[ib]).unwrap_or(f64::NAN);
    let scale = va + vb;
    for (v, s) in [(va, &a.shape), (vb, &b.shape)] {
        if !close(v, exact_volume(s), scale) {
            bad.push(format!("vol({s:?}) = {v}, closed form {}", exact_volume(s)));
        }
    }
    // The results' tolerance bands, summed: what the relations may be off by beyond rounding.
    let mut bands = 0.0;
    let mut run = |op: BoolOp, x: BodyId, y: BodyId, what: &str| -> Option<(Vec<BodyId>, f64)> {
        match boolean(&mut k, op, x, &[y]) {
            Ok(r) => {
                if let Some(why) = invalid(&k, &r) {
                    bad.push(format!("{what}: invalid: {why}"));
                }
                bands += band(&k, &r);
                match volume(&k, &r) {
                    Ok(v) => Some((r, v)),
                    Err(e) => {
                        bad.push(format!("{what}: volume: {e:?}"));
                        None
                    }
                }
            }
            Err(e) => {
                bad.push(format!("{what}: {e:?}"));
                None
            }
        }
    };
    let u = run(BoolOp::Union, ia, ib, "A ∪ B");
    let i = run(BoolOp::Intersect, ia, ib, "A ∩ B");
    let d = run(BoolOp::Subtract, ia, ib, "A − B");
    let u2 = run(BoolOp::Union, ib, ia, "B ∪ A");
    let i2 = run(BoolOp::Intersect, ib, ia, "B ∩ A");
    let close = |a: f64, b: f64, scale: f64| (a - b).abs() <= REL * scale.max(1.0) + bands;
    if let (Some(u), Some(i)) = (&u, &i)
        && !close(u.1 + i.1, va + vb, scale)
    {
        bad.push(format!(
            "vol(A∪B) + vol(A∩B) = {} + {} = {}, vol(A) + vol(B) = {}",
            u.1,
            i.1,
            u.1 + i.1,
            va + vb
        ));
    }
    if let (Some(d), Some(i)) = (&d, &i)
        && !close(d.1 + i.1, va, scale)
    {
        bad.push(format!(
            "vol(A−B) + vol(A∩B) = {}, vol(A) = {va}",
            d.1 + i.1
        ));
    }
    if let (Some(x), Some(y)) = (&u, &u2)
        && !close(x.1, y.1, scale)
    {
        bad.push(format!("vol(A∪B) = {} but vol(B∪A) = {}", x.1, y.1));
    }
    if let (Some(x), Some(y)) = (&i, &i2)
        && !close(x.1, y.1, scale)
    {
        bad.push(format!("vol(A∩B) = {} but vol(B∩A) = {}", x.1, y.1));
    }
    // The union through STEP and back.
    if let Some((r, v)) = &u {
        let parts: Vec<(BodyId, String)> = r.iter().map(|&b| (b, format!("u{}", b.0))).collect();
        let back = k
            .export_model(exchange::ExchangeFormat::Step, "metamorphic", &parts, &[])
            .and_then(|s| k.import_model(exchange::ExchangeFormat::Step, &s));
        match back {
            Ok(m) => {
                let ids: Vec<BodyId> = m.parts.iter().map(|p| p.body).collect();
                if ids.len() != r.len() {
                    bad.push(format!(
                        "STEP: {} bodies read back of {}",
                        ids.len(),
                        r.len()
                    ));
                }
                if let Some(why) = invalid(&k, &ids) {
                    bad.push(format!("STEP: read back invalid: {why}"));
                }
                match volume(&k, &ids) {
                    Ok(w) if close(w, *v, scale) => {}
                    w => bad.push(format!("STEP: volume {w:?} read back, {v} written")),
                }
            }
            Err(e) => bad.push(format!("STEP round trip: {e:?}")),
        }
    }
    bad
}

/// The failures of A − A = ∅, A ∪ A = A, A ∩ A = A.
fn check_self(a: &Placed) -> Vec<String> {
    let mut k = NativeKernel::new();
    let mut bad = Vec::new();
    let Ok(ia) = build(&mut k, a) else {
        return vec!["build failed".into()];
    };
    // A second, separate body of the same shape.
    let Ok(ib) = build(&mut k, a) else {
        return vec!["build failed".into()];
    };
    let va = volume(&k, &[ia]).unwrap_or(f64::NAN);
    match boolean(&mut k, BoolOp::Subtract, ia, &[ib]) {
        Ok(r) => {
            let v = volume(&k, &r).unwrap_or(f64::NAN);
            if !(v.abs() <= REL * va) {
                bad.push(format!("A − A has volume {v} ({} bodies)", r.len()));
            }
        }
        Err(e) => bad.push(format!("A − A: {e:?}")),
    }
    for (op, what) in [(BoolOp::Union, "A ∪ A"), (BoolOp::Intersect, "A ∩ A")] {
        match boolean(&mut k, op, ia, &[ib]) {
            Ok(r) => {
                if let Some(why) = invalid(&k, &r) {
                    bad.push(format!("{what}: invalid: {why}"));
                }
                let v = volume(&k, &r).unwrap_or(f64::NAN);
                if !close(v, va, va) || r.len() != 1 {
                    bad.push(format!(
                        "{what}: volume {v} in {} bodies, vol(A) = {va}",
                        r.len()
                    ));
                }
            }
            Err(e) => bad.push(format!("{what}: {e:?}")),
        }
    }
    bad
}

/// Runs `n` random cases from `seed`, skipping `KNOWN`; panics listing every failure.
fn random_cases(seed: u64, n: usize) {
    let mut fails = Vec::new();
    let only: Option<usize> = std::env::var("CADRS_META_CASE")
        .ok()
        .and_then(|v| v.parse().ok());
    for case in 0..n {
        if only.is_some_and(|o| o != case) {
            continue;
        }
        // Each case its own stream, so a case reruns alone from (seed, case).
        let mut r = Rng::new(seed ^ (case as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let a = shape(&mut r);
        let b = shape(&mut r);
        if KNOWN.contains(&(seed, case)) {
            continue;
        }
        let mut bad = check_self(&a);
        bad.extend(check_pair(&a, &b));
        if !bad.is_empty() {
            fails.push(format!(
                "seed {seed} case {case}:\n  A = {a:?}\n  B = {b:?}\n  {}",
                bad.join("\n  ")
            ));
        }
    }
    assert!(
        fails.is_empty(),
        "{} of {n} cases fail:\n{}",
        fails.len(),
        fails.join("\n")
    );
}

#[test]
fn random_pairs() {
    let only = env_num("CADRS_META_CASE", 0usize);
    random_cases(seed(1), env_num("CADRS_META_CASES", 16).max(only + 1));
}

/// Pairs that share faces exactly: a box and the same box moved by a whole step along each axis
/// and diagonal (faces, edges and corners in contact), and boxes overlapping by half.
#[test]
fn touching_boxes() {
    let a = Placed::fixed(Shape::Box {
        min: [0.0; 3],
        size: [4.0, 4.0, 4.0],
    });
    let mut fails = Vec::new();
    for d in [
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 0.0],
        [4.0, 4.0, 4.0],
        [2.0, 0.0, 0.0],
        [2.0, 2.0, 2.0],
        [0.0, 0.0, 2.0],
    ] {
        let b = Placed::fixed(Shape::Box {
            min: d,
            size: [4.0, 4.0, 4.0],
        });
        let bad = check_pair(&a, &b);
        if !bad.is_empty() {
            fails.push(format!("B at {d:?}: {}", bad.join("; ")));
        }
    }
    assert!(fails.is_empty(), "{}", fails.join("\n"));
}

/// One extrude-sequence step: the shape and how it combines with the model so far.
#[derive(Debug, Clone, Copy)]
struct Step {
    op: BoolOp,
    shape: Shape,
}

/// A random axis-aligned sketch extruded from one of the three planes, as a CAD model's
/// features are (Fusion 360 Gallery's reconstruction sequences: sketch, extrude, join / cut /
/// intersect), on a grid of 0.5 so faces of different steps coincide.
fn prism(r: &mut Rng) -> Shape {
    let g = |r: &mut Rng, lo: f64, hi: f64| (r.range(lo, hi) * 2.0).round() / 2.0;
    let (x0, y0) = (g(r, 0.0, 6.0), g(r, 0.0, 6.0));
    let (x1, y1) = (x0 + g(r, 1.5, 6.0), y0 + g(r, 1.5, 6.0));
    // A third with a hole: a rectangle well inside (a wall at least 0.5 thick).
    let hole = (r.below(3) == 0 && x1 - x0 >= 2.0 && y1 - y0 >= 2.0).then(|| {
        let (hx0, hy0) = (x0 + 0.5, y0 + 0.5);
        [
            hx0,
            hy0,
            (hx0 + g(r, 0.5, x1 - x0 - 1.0)).min(x1 - 0.5),
            (hy0 + g(r, 0.5, y1 - y0 - 1.0)).min(y1 - 0.5),
        ]
    });
    let axis = r.below(3);
    let mut o = [0.0; 3];
    o[axis] = g(r, 0.0, 6.0);
    Shape::Prism {
        axis,
        o,
        outer: [x0, y0, x1, y1],
        hole: hole.filter(|h| h[2] > h[0] && h[3] > h[1]),
        depth: g(r, 1.0, 6.0),
    }
}

/// The exact volume of the model a sequence of steps builds: over the cells of the grid its
/// faces make, each cell's centre classified through the steps.
fn sequence_volume(steps: &[Step]) -> f64 {
    let mut axes: [Vec<f64>; 3] = Default::default();
    for s in steps {
        for (i, c) in s.shape.planes().into_iter().enumerate() {
            axes[i].extend(c);
        }
    }
    for a in &mut axes {
        a.sort_by(f64::total_cmp);
        a.dedup();
    }
    let mut v = 0.0;
    for x in axes[0].windows(2) {
        for y in axes[1].windows(2) {
            for z in axes[2].windows(2) {
                let c = [
                    (x[0] + x[1]) / 2.0,
                    (y[0] + y[1]) / 2.0,
                    (z[0] + z[1]) / 2.0,
                ];
                let inside = steps.iter().fold(false, |acc, s| {
                    let p = s.shape.contains(c);
                    match s.op {
                        BoolOp::Union => acc || p,
                        BoolOp::Subtract => acc && !p,
                        BoolOp::Intersect => acc && p,
                    }
                });
                if inside {
                    v += (x[1] - x[0]) * (y[1] - y[0]) * (z[1] - z[0]);
                }
            }
        }
    }
    v
}

/// The failures of one extrude sequence: after each step the model is valid and has the exact
/// volume (an empty model ends the sequence).
fn check_sequence(steps: &[Step], done: &mut usize) -> Vec<String> {
    let mut k = NativeKernel::new();
    let mut bad = Vec::new();
    let mut model: Vec<BodyId> = Vec::new();
    for (i, s) in steps.iter().enumerate() {
        let tool = match build(&mut k, &Placed::fixed(s.shape)) {
            Ok(t) => t,
            Err(e) => return vec![format!("step {i}: build: {e:?}")],
        };
        model = if i == 0 {
            vec![tool]
        } else {
            // A model of several solids: each combined with the tool (a union joins them all).
            let r = match s.op {
                BoolOp::Union => {
                    let (first, rest) = model
                        .split_first()
                        .map(|(f, r)| (*f, r.to_vec()))
                        .unwrap_or((tool, vec![]));
                    let mut tools = rest;
                    tools.push(tool);
                    boolean(&mut k, BoolOp::Union, first, &tools)
                }
                op => model.iter().try_fold(Vec::new(), |mut acc, &m| {
                    acc.extend(boolean(&mut k, op, m, &[tool])?);
                    Ok(acc)
                }),
            };
            match r {
                Ok(r) => r,
                Err(e) => {
                    bad.push(format!("step {i} ({:?}): {e:?}", s.op));
                    break;
                }
            }
        };
        if let Some(why) = invalid(&k, &model) {
            bad.push(format!("step {i} ({:?}): invalid: {why}", s.op));
            break;
        }
        let want = sequence_volume(&steps[..=i]);
        let v = volume(&k, &model).unwrap_or(f64::NAN);
        // (Within the model's tolerance band: a result of tolerance t stands for any solid
        // within t of it.)
        if !((v - want).abs() <= 1e-9 * want.max(1.0) + band(&k, &model)) {
            bad.push(format!("step {i} ({:?}): volume {v}, exact {want}", s.op));
            break;
        }
        *done += 1;
        if model.is_empty() {
            break;
        }
    }
    bad
}

/// Random extrude sequences (3 to 8 steps of join, cut and intersect) checked against their
/// exact volumes after every step. `CADRS_META_CASES` scales the count as for `random_pairs`.
#[test]
fn extrude_sequences() {
    let s = seed(1);
    let n = env_num("CADRS_META_CASES", 16usize);
    let mut fails = Vec::new();
    let mut done = 0;
    for case in 0..n {
        let mut r = Rng::new(s ^ (case as u64).wrapping_mul(0xd1b5_4a32_d192_ed03));
        let len = 3 + r.below(6);
        let steps: Vec<Step> = (0..len)
            .map(|i| Step {
                op: match (i, r.below(20)) {
                    (0, _) => BoolOp::Union,
                    (_, 0..=9) => BoolOp::Union,
                    (_, 10..=17) => BoolOp::Subtract,
                    _ => BoolOp::Intersect,
                },
                shape: prism(&mut r),
            })
            .collect();
        let bad = check_sequence(&steps, &mut done);
        if !bad.is_empty() {
            fails.push(format!(
                "seed {s} sequence {case}: {steps:?}\n  {}",
                bad.join("\n  ")
            ));
        }
    }
    assert!(
        fails.is_empty(),
        "{} of {n} sequences fail:\n{}",
        fails.len(),
        fails.join("\n")
    ); // (Not vacuous: most sequences run several steps before an intersection can empty them.)
    println!("extrude_sequences: {done} steps checked");
    assert!(done >= 3 * n, "only {done} steps checked");
}

/// Runs the pair relations on one pinned pair, panicking with its failures.
fn pinned(a: Placed, b: Placed) {
    let bad = check_pair(&a, &b);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// Two cylinders crossing at skew angles (seed 12345 case 2): every boolean of the pair left
/// an open shell (the small cylinder's cap box, from samples of its rim, fell short of the rim
/// where the cylinders' curve leaves it, and the cap's own curve lost its end there).
#[test]
fn skew_cylinders() {
    pinned(
        Placed {
            shape: Shape::Cyl {
                axis: 2,
                base: [7.0, 7.0, -0.3707804499725267],
                r: 1.1004411956127174,
                len: 2.269831665071858,
            },
            axis: [-0.8516050882627577, -0.978177783068829, -0.3816516819404845],
            angle: 1.3877809304771738,
            about: [5.0; 3],
        },
        Placed {
            shape: Shape::Cyl {
                axis: 2,
                base: [7.0, 3.817329177849694, 3.5352404522767253],
                r: 3.753280953966347,
                len: 6.782899416459303,
            },
            axis: [
                0.19230537965956995,
                -0.9084555615490686,
                -0.3459386681894472,
            ],
            angle: -1.1134832725489603,
            about: [5.0; 3],
        },
    );
}

/// A tilted box and a tilted cylinder (seed 12345 case 75): every boolean left an open shell
/// (as [`skew_cylinders`]).
#[test]
fn tilted_box_and_cylinder() {
    pinned(
        Placed {
            shape: Shape::Box {
                min: [-1.7287351321660231, 2.0, 7.0],
                size: [6.524599352812511, 5.962324057922664, 2.308564273937577],
            },
            axis: [0.5530960115474062, 0.6896281828475401, 0.3117214767784895],
            angle: -2.9200964712114432,
            about: [5.0; 3],
        },
        Placed {
            shape: Shape::Cyl {
                axis: 2,
                base: [6.5860680206798445, 6.931195387961152, -1.418602229435946],
                r: 3.0,
                len: 4.0,
            },
            axis: [
                -0.6386842108454478,
                0.14934690994994448,
                0.21526908347248686,
            ],
            angle: -1.8272343669399191,
            about: [5.0; 3],
        },
    );
}

/// Two crossing cylinders on the axes (seed 12345 case 142): the union's STEP file read back
/// without one of the small cylinder's faces. An edge of the closed intersection curve ends where
/// the curve closes; the reader took that point as the curve's start, ran the edge the other,
/// long way round, and the face's loop then wrapped round the cylinder.
#[test]
fn step_round_trip_seamed_cylinder() {
    pinned(
        Placed::fixed(Shape::Cyl {
            axis: 1,
            base: [1.1684983475486566, 0.5293421570789636, 2.0824245633488534],
            r: 2.736774377748768,
            len: 7.323229369549773,
        }),
        Placed::fixed(Shape::Cyl {
            axis: 2,
            base: [0.0, 3.0790339291622146, -0.019167617471158582],
            r: 1.2504993337665509,
            len: 5.269151724617697,
        }),
    );
}

/// A box cut by a cylinder along x that touches the box's bottom face along a line (soak seed
/// 6060224037237635290 case 49): the difference's faces cross away from their edges (an error
/// from the boolean's validity stage; it was handed back as a body, with none).
#[test]
#[ignore = "known failure: a box less a cylinder tangent to its face fails (its faces cross away from their edges)"]
fn cylinder_tangent_to_box_face() {
    pinned(
        Placed::fixed(Shape::Box {
            min: [4.0, 4.923492988846983, 2.0],
            size: [4.610421114922351, 4.0648740291358285, 3.0],
        }),
        Placed::fixed(Shape::Cyl {
            axis: 0,
            base: [8.0, 6.9670406723616765, 4.0],
            r: 2.0,
            len: 5.0,
        }),
    );
}

/// Every boolean of `x` and `y` (both ways round) that succeeds gives a valid body: one the
/// boolean can't get right is an error (strictly, as the helper takes it: a result handed back
/// flagged as having crossing faces is one), never an unflagged invalid result.
fn never_invalid(x: Placed, y: Placed) {
    let mut k = NativeKernel::new();
    let a = build(&mut k, &x).unwrap();
    let b = build(&mut k, &y).unwrap();
    let mut bad = Vec::new();
    for (p, q, order) in [(a, b, ""), (b, a, " (B first)")] {
        for op in [BoolOp::Union, BoolOp::Intersect, BoolOp::Subtract] {
            if let Ok(r) = boolean(&mut k, op, p, &[q])
                && let Some(why) = invalid(&k, &r)
            {
                bad.push(format!("{op:?}{order}: {why}"));
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// [`cylinder_tangent_to_box_face`] and [`spheres_loops_cross_at_seam`]: until they are
/// computed right, flagged (the boolean finds the faces crossing, the loops crossing in
/// (u, v)), never invalid bodies handed back as sound.
#[test]
fn known_failures_are_never_invalid() {
    never_invalid(
        Placed::fixed(Shape::Box {
            min: [4.0, 4.923492988846983, 2.0],
            size: [4.610421114922351, 4.0648740291358285, 3.0],
        }),
        Placed::fixed(Shape::Cyl {
            axis: 0,
            base: [8.0, 6.9670406723616765, 4.0],
            r: 2.0,
            len: 5.0,
        }),
    );
    never_invalid(
        Placed {
            shape: Shape::Sphere {
                c: [7.0, 6.62966283372091, 0.9659503482469911],
                r: 4.0,
            },
            axis: [0.7765431131320071, 0.5361794883583875, 0.5889070314711073],
            angle: -0.3773634823933971,
            about: [5.0; 3],
        },
        Placed::fixed(Shape::Sphere {
            c: [7.0, 4.0, 9.0],
            r: 4.532638031669147,
        }),
    );
}

/// A cylinder along x and a tilted box (seed 12345 case 107): the union's STEP file read back
/// with two planar faces' conic edges 0.2 mm off their p-curves. The edges use part of a
/// rational arc whose p-curve covers the whole arc, and the reader's gap closing moved the
/// arc's end control point to the edge's end.
#[test]
fn step_round_trip_conic_edge() {
    pinned(
        Placed::fixed(Shape::Cyl {
            axis: 0,
            base: [6.760270809297007, 2.0, 2.0],
            r: 3.347707988538065,
            len: 3.87749651721185,
        }),
        Placed {
            shape: Shape::Box {
                min: [3.0, 1.0, 4.0],
                size: [5.0, 4.0, 4.0],
            },
            axis: [
                -0.8011568824086492,
                0.16992539633470694,
                -0.6291828619687938,
            ],
            angle: -0.09767065517102713,
            about: [5.0; 3],
        },
    );
}

/// Four boxes united in turn (an extrude sequence of the soak, minimised): the first two touch
/// along an edge only (x = 4.5, z = 5.5), the third bridges them, and the fourth's union left
/// an open shell: the model keeps the two boxes' corners there as two vertices (and the edge as
/// two), which the boolean pooled for the tool (by position) but not for the target.
#[test]
fn boxes_touching_along_an_edge() {
    let bx = |x: [f64; 2], y: [f64; 2], z: [f64; 2]| Step {
        op: BoolOp::Union,
        shape: Shape::Box {
            min: [x[0], y[0], z[0]],
            size: [x[1] - x[0], y[1] - y[0], z[1] - z[0]],
        },
    };
    let steps = [
        bx([2.5, 4.5], [3.0, 5.0], [5.5, 11.0]),
        bx([4.5, 8.0], [3.0, 5.5], [3.5, 5.5]),
        bx([4.5, 6.5], [4.5, 6.0], [3.5, 8.0]),
        bx([4.5, 9.5], [3.0, 7.0], [4.0, 6.0]),
    ];
    let mut bad = check_sequence(&steps, &mut 0);
    // And a cut (seed 955695009267410166 sequence 500), the same boxes as sketches extruded
    // along y and z (as boxes along z alone it passes): two touching along the edge x = 5.5,
    // y = 5, a third on both, then a block cut through them all.
    let prism = |op, axis, o, outer, depth| Step {
        op,
        shape: Shape::Prism {
            axis,
            o,
            outer,
            hole: None,
            depth,
        },
    };
    let steps = [
        prism(BoolOp::Union, 1, [0.0, 2.0, 0.0], [1.0, 3.5, 5.0, 5.5], 3.0),
        prism(BoolOp::Union, 1, [0.0, 5.0, 0.0], [2.5, 5.5, 5.0, 7.5], 2.5),
        prism(
            BoolOp::Union,
            2,
            [0.0, 0.0, 5.0],
            [2.0, 5.0, 6.5, 10.5],
            2.0,
        ),
        prism(
            BoolOp::Subtract,
            1,
            [0.0, 2.5, 0.0],
            [1.0, 1.0, 5.0, 6.0],
            5.0,
        ),
    ];
    bad.extend(check_sequence(&steps, &mut 0));
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// Two spheres, one turned (soak seed 16150658447098336548 case 2323): the union and the
/// difference came back invalid, a sphere face's intersection loop crossing its outer loop in
/// (u, v) at the seam near the pole: the seam's crossings of the other sphere, close together
/// there, fell between two samples of the edge–face search.
#[test]
fn spheres_loops_cross_at_seam() {
    pinned(
        Placed {
            shape: Shape::Sphere {
                c: [7.0, 6.62966283372091, 0.9659503482469911],
                r: 4.0,
            },
            axis: [0.7765431131320071, 0.5361794883583875, 0.5889070314711073],
            angle: -0.3773634823933971,
            about: [5.0; 3],
        },
        Placed::fixed(Shape::Sphere {
            c: [7.0, 4.0, 9.0],
            r: 4.532638031669147,
        }),
    );
}

// --- Soak findings (the build box's worker, research/SMLIB_COVERAGE.md item 1 of review 5), replayed
// at kernel-perf d0bcb72 and still failing, pinned by class: each test asserts every pair of its
// class keeps the metamorphic relations (`check_pair`), and is a known failure until they all do.

fn pinned_all(cases: &[(Placed, Placed)]) {
    let bad: Vec<String> = cases
        .iter()
        .enumerate()
        .filter_map(|(i, (a, b))| {
            let v = check_pair(a, b);
            (!v.is_empty()).then(|| format!("case {i}: {}", v.join("; ")))
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

fn sequences_all(cases: &[Vec<Step>]) {
    let bad: Vec<String> = cases
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let mut done = 0;
            let v = check_sequence(s, &mut done);
            (!v.is_empty()).then(|| format!("sequence {i}: {}", v.join("; ")))
        })
        .collect();
    assert!(
        bad.is_empty(),
        "{} of {} sequences fail:\n{}",
        bad.len(),
        cases.len(),
        bad.join("\n")
    );
}

/// Soak: 17 random pairs where the boolean fails at its crossing stage: the result's faces cross away from their edges.
#[test]
#[ignore = "known failure: the boolean fails at its crossing stage: the result's faces cross away from their edges"]
fn soak_crossing_faces() {
    pinned_all(&[
        (
            Placed {
                shape: Shape::Box {
                    min: [3.1488392781016845, 2.0, 5.415457820519964],
                    size: [2.5775026575876434, 2.993664838327189, 7.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [3.2777639729094377, 4.0, 0.3277249182163051],
                    r: 2.0,
                    len: 10.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [3.0, 1.0, -1.6552081844961057],
                    size: [7.425345122003242, 3.8824692111139014, 5.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [7.0, 2.0, 2.0],
                    r: 1.0,
                    len: 1.9907728111761818,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [3.0, 5.0, -1.1192961792550353],
                    size: [4.0, 8.0, 3.0894177820943702],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [6.0, 7.811081994700105, 1.7850137238828556],
                    r: 3.0,
                    len: 5.179609932458098,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [0.6703509525537394, 2.0, 3.2095117910279054],
                    size: [5.0, 5.0, 7.449262269911995],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [4.479540623128727, 3.0, 2.0],
                    r: 1.0,
                    len: 7.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [5.0, 1.7162657418552336, 5.0],
                    size: [2.0, 7.0, 2.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [6.234698950847345, 5.0, 3.0],
                    r: 4.0,
                    len: 10.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [1.0, 2.129970595700306, -0.0],
                    size: [5.0, 7.0, 7.0],
                },
                axis: [
                    0.47309574822356093,
                    -0.7193089584241474,
                    0.42462293731945344,
                ],
                angle: 2.612967241405763,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [7.0, 5.0, 4.409159399362541],
                    r: 3.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [2.904577844494187, 1.0, -1.1478926031450687],
                    r: 2.5,
                    len: 2.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [4.979590691209418, 2.0, 0.15396478701042327],
                    r: 1.0,
                    len: 9.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [4.0, 6.0, -1.0],
                    size: [3.3474666428826376, 4.796250692899969, 5.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [6.0, 5.0, 3.0487869939197516],
                    r: 2.0,
                    len: 5.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [4.0, -2.0, 2.971810697434627],
                    size: [1.287877446705254, 5.616682306956167, 3.3816316843508187],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [5.0, 2.277079167749927, -1.0],
                    r: 1.0,
                    len: 5.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [2.0, -2.0, -1.0],
                    size: [1.423523801502283, 4.074168865495656, 2.329037191507853],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [2.5395458353610123, 2.0, -0.082576575909165],
                    r: 4.0,
                    len: 4.857336548667688,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [7.0, 6.0, 3.0],
                    r: 2.5,
                    len: 3.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [8.0, 4.0, 3.0],
                    r: 3.0,
                    len: 9.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [7.326124763537955, 4.0, -0.0],
                    r: 1.5,
                    len: 7.7940733668451045,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [6.0, 6.0, -0.6428255720948879],
                    r: 2.0,
                    len: 4.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [4.0, 4.923492988846983, 2.0],
                    size: [4.610421114922351, 4.0648740291358285, 3.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [8.0, 6.9670406723616765, 4.0],
                    r: 2.0,
                    len: 5.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [1.6684430578308547, 4.901515519719919, 0.0],
                    size: [7.0, 6.482562028266191, 1.7397724370635053],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [0.33818236935171164, 9.882592705712538, 3.0],
                    r: 3.0,
                    len: 4.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [3.0, 2.0, 0.0],
                    size: [6.0, 7.0, 5.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [7.197723646252125, 6.0, 2.0],
                    r: 3.0,
                    len: 2.5609611312150458,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [4.0, 2.428017040640359, 2.979643113970912],
                    size: [4.886567852570065, 6.278842127038555, 5.868951996735679],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [6.0, 4.0, 3.2127336652078924],
                    r: 2.0,
                    len: 7.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [5.297795445380122, -0.0, 1.9240297504410804],
                    size: [2.0, 6.0, 7.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [6.0, 4.0, 3.257106145582065],
                    r: 2.0,
                    len: 8.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
    ]);
}

/// Soak: 2 random pairs where the boolean fails at its validity stage: an edge used by three or more faces.
#[test]
#[ignore = "known failure: the boolean fails at its validity stage: an edge used by three or more faces"]
fn soak_edge_three_faces() {
    pinned_all(&[
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [5.0, 9.062184225218743, 1.6485531977305565],
                    r: 3.0,
                    len: 1.6669387850233994,
                },
                axis: [
                    -0.32529514219475986,
                    -0.6699122407946754,
                    -0.7403285648339855,
                ],
                angle: -2.821600784091703,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [1.0923451681008334, 1.0, 3.099824784070817],
                    r: 0.5,
                    len: 9.061895362994564,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [8.0, 7.999079142777768, 2.5348935755832933],
                    r: 1.5,
                    len: 4.229109054168786,
                },
                axis: [0.7234507352253889, -0.03205853539092951, 0.6582523963293634],
                angle: -2.1969795899937927,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [8.0, 1.8740332769622892, 3.0],
                    r: 1.0,
                    len: 7.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
    ]);
}

/// Soak: 11 random pairs where the boolean hands back, unflagged, a result whose face loop crosses itself in (u, v).
#[test]
#[ignore = "known failure: the boolean hands back, unflagged, a result whose face loop crosses itself in (u, v)"]
fn soak_invalid_loop_crosses() {
    pinned_all(&[
        (
            Placed {
                shape: Shape::Sphere {
                    c: [5.0, 1.0, 7.036992286824121],
                    r: 4.0,
                },
                axis: [
                    -0.016903651407258025,
                    -0.6319371979215478,
                    0.01917277134638007,
                ],
                angle: 1.983124849791852,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [2.451763362263052, 2.679831757519989, 0.4050515218439692],
                    r: 3.5,
                    len: 9.203503472977564,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [0.30730274158414894, 3.3471300653584644, 3.2610510871895153],
                    r: 2.1853372643973366,
                    len: 6.976898545676256,
                },
                axis: [
                    -0.15343422303941479,
                    0.26443605601808895,
                    0.3296884654918881,
                ],
                angle: 0.910670263126697,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [3.8035741714678375, 1.0, 7.999795557235356],
                    r: 1.0896343246868545,
                },
                axis: [
                    0.3453146671677114,
                    0.006915646599200631,
                    -0.02613831535475386,
                ],
                angle: 0.3093429980737681,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [4.0, 8.0, 0.0],
                    r: 1.1137697093271288,
                },
                axis: [0.34335354310430666, -0.714586058636884, 0.0450338161863042],
                angle: 0.5788995564451795,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [5.86867490057557, 6.411131787815862, -2.0],
                    size: [1.236419152892064, 3.6821611692181895, 4.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [2.6100040372112963, 6.439500494033513, 1.0],
                    size: [6.938696866803948, 6.0, 7.662315330454483],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 8.394055532817044, 5.0],
                    r: 3.5,
                },
                axis: [
                    0.3566625828544734,
                    0.33274708224032157,
                    -0.39805568739903685,
                ],
                angle: 0.5545399438932885,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 5.0, 2.0],
                    r: 2.980339809912162,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [9.035373813430503, 4.1203896017818415, 0.4992912739423758],
                    r: 3.473912573152656,
                    len: 6.646732410632577,
                },
                axis: [-0.8864745378033834, 0.9786404098303616, 0.05441381399669187],
                angle: 0.7924141381877985,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.996894190799838, 4.0, 5.0],
                    r: 1.3342437205130213,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [7.0, 7.0, 0.0],
                    r: 3.0,
                    len: 7.065676368618286,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [2.0, 4.0, -1.0],
                    r: 3.5,
                    len: 5.779065853308616,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [8.538353379595648, 3.0, 2.0],
                    r: 4.031107281298314,
                },
                axis: [0.3764551098790825, 0.7870837287356063, 0.3323565681660594],
                angle: 2.2243662938145317,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 5.0, 0.7621194309164514],
                    r: 3.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [4.0, 3.0, -1.0],
                    size: [6.0, 4.328608181857929, 6.0],
                },
                axis: [-0.9224847761165642, 0.8666630864176297, -0.6712290354640544],
                angle: 0.5730438459315543,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [3.0, 6.0, 9.181849600269507],
                    r: 4.5,
                },
                axis: [-0.8391753743413968, 0.33093422722969024, 0.5672914947220002],
                angle: -1.0075670696714896,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [-1.0, 0.5816972135370442, 6.27434704568471],
                    size: [4.364557409255513, 3.0993980754023966, 6.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [3.6785276653163805, -1.0, 4.0],
                    size: [7.5978580482149525, 6.0, 7.0],
                },
                axis: [
                    -0.39171045904580604,
                    -0.7489796776218292,
                    0.23997004680603884,
                ],
                angle: 0.7667534951624022,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [8.0, 3.0, 7.0],
                    r: 3.0,
                },
                axis: [
                    0.26995614291374825,
                    -0.6185202513846557,
                    -0.7028418693102327,
                ],
                angle: 0.5728832660355039,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [5.869552530800455, 7.79672197742597, 2.8323410670523517],
                    r: 3.5,
                },
                axis: [0.187041187095623, 0.6382022551188762, 0.8334678186334672],
                angle: -1.7160225743532982,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [3.0, 1.0, 5.0],
                    size: [2.009858634524574, 4.070640015098378, 4.0],
                },
                axis: [0.4402556742296815, -0.639322695576795, 0.4730965594553238],
                angle: -2.6736831898494837,
                about: [5.0, 5.0, 5.0],
            },
        ),
    ]);
}

/// Soak: 21 random pairs where the boolean hands back, unflagged, a result with a p-curve off its edge.
#[test]
#[ignore = "known failure: the boolean hands back, unflagged, a result with a p-curve off its edge"]
fn soak_invalid_pcurve_off_edge() {
    pinned_all(&[
        (
            Placed {
                shape: Shape::Sphere {
                    c: [3.0, 3.5964921304064914, 7.0],
                    r: 3.0,
                },
                axis: [0.8740683557520093, 0.5946663164446664, 0.18422064023992424],
                angle: -0.3671756481698427,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [5.565806622667734, 7.245460504989447, 2.909244378417654],
                    r: 2.0,
                    len: 9.0,
                },
                axis: [0.10414403449215204, 0.7682369420611528, -0.575095179579546],
                angle: -0.6057343281948215,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [8.30171567106301, 2.0, 7.178058619016291],
                    r: 1.232710444491984,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [8.18467560227879, 3.0, 3.8840967783371205],
                    r: 1.0,
                    len: 9.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [7.752082383141018, 5.0, 8.272819088643107],
                    r: 3.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [7.518625283276053, 8.0, 4.0],
                    r: 3.0,
                    len: 7.736992919348919,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [5.426302600022799, 9.047016072310113, 6.0],
                    r: 4.417276703681405,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [3.100523778955778, 8.0, -1.6661897447935548],
                    r: 2.2006408464708587,
                    len: 9.57170041581304,
                },
                axis: [
                    -0.17154007424918238,
                    0.7019457045070243,
                    -0.8130036100696649,
                ],
                angle: -2.437939028385145,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [9.0, 8.478019057687288, 4.0],
                    r: 3.5,
                    len: 8.667225964013955,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [9.210657467722198, 5.0, 9.50410706603668],
                    r: 1.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [3.0, 5.510591592570519, 3.3518996829818146],
                    r: 3.0,
                    len: 6.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [4.430146008877465, 5.811351741950521, 3.4568289355755146],
                    r: 1.721913857926097,
                },
                axis: [-0.8779743603340888, -0.9400338084640107, 0.5203516841750919],
                angle: -1.7631690032057137,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [5.573596963934357, 4.0, -1.0],
                    r: 3.946529218546851,
                    len: 5.1154859417394265,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [7.0, 4.224579487054539, 3.0],
                    r: 3.644519078529156,
                },
                axis: [
                    -0.29644086949998316,
                    -0.9519055830719356,
                    -0.19781501336574903,
                ],
                angle: -1.7258301376132135,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [6.0, 1.992735975141846, 3.1985277585230083],
                    r: 3.0,
                    len: 8.455465683778204,
                },
                axis: [
                    -0.2710377973052249,
                    0.37956758080226627,
                    -0.9882285114993434,
                ],
                angle: -1.360114436597878,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [8.710098010841099, 1.634602183584899, 2.4761970783530796],
                    r: 3.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [2.0, 7.487983957974272, 0.7103279846779049],
                    r: 3.0,
                    len: 9.865283920863423,
                },
                axis: [
                    -0.17024758305058696,
                    -0.8038158987607189,
                    -0.37766569219097623,
                ],
                angle: -1.716125786725102,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [4.14035553625108, 1.8844048123620305, 6.0],
                    r: 2.5385432647173767,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [7.339892758257037, 3.0, 8.0],
                    r: 4.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [8.958224960406804, 0.9468768210694001, 2.1772388716459634],
                    r: 1.5265737493765186,
                    len: 9.999179309729598,
                },
                axis: [-0.4119852871920664, 0.8552854617328802, -0.9770251780741743],
                angle: -1.767696859341498,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [4.0, 7.0, 0.026763516681992394],
                    r: 2.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [1.0, 5.519314255380429, 2.2109188995756783],
                    r: 1.5,
                    len: 4.340733379868339,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 5.967198120947412, 1.3201086165025766],
                    r: 2.5,
                },
                axis: [
                    -0.4610176892595057,
                    -0.6349745228552639,
                    -0.8491646950247591,
                ],
                angle: -2.110152749984531,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [1.0116880965690056, 1.0, 1.0],
                    r: 3.5,
                    len: 9.757117782318216,
                },
                axis: [0.42003537039745775, 0.07408496689683441, -0.905860871338495],
                angle: 0.3984036471572061,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [3.533531831960559, 5.0, 1.4905572026901626],
                    r: 3.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [6.484965995666484, 7.0, 1.6152985773011324],
                    r: 2.0,
                    len: 5.713583453072534,
                },
                axis: [-0.4201447061889194, 0.7035013624031152, 0.6837817609523418],
                angle: 1.4509989309497175,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [9.0, 4.0, 5.286182631445983],
                    r: 1.7700498608176969,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [9.123765136375617, 7.0, 2.9644677971059012],
                    r: 3.0,
                    len: 3.5823918747588386,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [3.0, 3.4508572553687857, 1.1741695439515754],
                    r: 4.352059554542215,
                },
                axis: [-0.623194670530117, 0.6071792674147829, -0.45280673550448647],
                angle: 2.123706868300065,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [1.0, 7.0, 3.0611225879422586],
                    r: 3.515716768157306,
                    len: 5.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [3.7285145289480415, 0.0, 4.0],
                    r: 2.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [3.277981984075702, 4.0, 2.0],
                    r: 4.0,
                    len: 7.518559444411812,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [6.0, 2.0, 3.0],
                    r: 1.8028514056297553,
                    len: 5.834647283758605,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 5.0, 5.539910683821861],
                    r: 2.0,
                },
                axis: [-0.9558162594794977, 0.753781116093575, 0.03282484425846999],
                angle: 1.5602342268200804,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [4.438690629794159, 3.0, -1.0],
                    r: 1.092984221175746,
                    len: 7.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [5.18306139811974, 2.1905451969910006, 8.73488667473168],
                    r: 4.399884312242616,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [3.0, 1.087315898292529, 2.0],
                    r: 4.0,
                    len: 6.218600015251311,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [7.0, 6.0, 2.7445570217097073],
                    r: 2.5,
                },
                axis: [-0.7464704462641558, 0.1614415822299733, 0.13728801137268243],
                angle: 2.9555255074567084,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [1.5280100529587404, 0.0, 3.1486084027455323],
                    r: 2.0,
                    len: 6.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [2.0, 2.0, 1.7073630710915633],
                    r: 4.873410167929423,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [9.0, 2.0, 1.2270525330166056],
                    r: 3.8575686554476603,
                    len: 5.553252029941808,
                },
                axis: [0.6749886085318255, -0.8112167079063606, -0.7875663213220556],
                angle: -2.901828804107521,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [7.0, 5.3698178423595975, 4.31054556836877],
                    r: 3.097454272933569,
                },
                axis: [0.873604046713444, 0.8214887568679554, -0.8127046844339298],
                angle: 2.671552441256348,
                about: [5.0, 5.0, 5.0],
            },
        ),
    ]);
}

/// Soak: 2 random pairs where the difference hands back, unflagged, a solid whose void shell lies outside its outer shell.
#[test]
#[ignore = "known failure: the difference hands back, unflagged, a solid whose void shell lies outside its outer shell"]
fn soak_invalid_void_outside() {
    pinned_all(&[
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [3.232010047633229, 4.0, 3.7143366008049528],
                    r: 1.5,
                    len: 7.864654785428158,
                },
                axis: [0.4985201443733951, 0.5196639929092461, -0.730928258005686],
                angle: -2.984315989521584,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [-1.829884274572478, 2.0, 7.0],
                    size: [5.6065017970502025, 3.046364076929485, 1.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [5.539152361065838, 6.393352829870515, 0.0],
                    r: 1.5,
                    len: 5.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [3.1390138098444806, 3.5824414981712165, 5.5571971529622886],
                    size: [1.0, 5.0, 1.0],
                },
                axis: [
                    0.34394659695910423,
                    -0.21929430635878777,
                    -0.029143349900050186,
                ],
                angle: 2.6927287989564217,
                about: [5.0, 5.0, 5.0],
            },
        ),
    ]);
}

/// Soak: 26 random pairs where the boolean fails at its validity stage: a result face's loop crosses itself in (u, v).
#[test]
#[ignore = "known failure: the boolean fails at its validity stage: a result face's loop crosses itself in (u, v)"]
fn soak_loop_crosses_itself() {
    pinned_all(&[
        (
            Placed {
                shape: Shape::Sphere {
                    c: [2.67335668948638, 3.0, 5.248837537630534],
                    r: 3.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [2.133079211569356, 6.149964366224982, 1.0],
                    r: 3.9943650741174164,
                    len: 7.0,
                },
                axis: [0.5513118753726063, 0.9133130226140538, 0.6760139996280479],
                angle: 0.3488210201888782,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [6.629461592983147, 7.067146304347562, 2.0],
                    r: 3.5,
                    len: 8.0,
                },
                axis: [0.3813543764118217, -0.6233436708403308, 0.8037378153882111],
                angle: -1.262824427021483,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 4.19919358023059, 0.603859303968729],
                    r: 3.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [4.0, 2.835955148111881, 6.2746491542939395],
                    size: [8.0, 6.090882772042182, 4.978457908408268],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [5.0, 7.956912451803354, 10.0],
                    r: 3.0,
                },
                axis: [0.8882627832858403, -0.8118346212957848, -0.7591898381852629],
                angle: -0.3768305108066117,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [1.0, -1.0, -1.0],
                    size: [6.097964382142503, 4.0, 5.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [2.1280044691920064, 3.0, 5.334210760290295],
                    r: 1.5,
                },
                axis: [0.6762453934513717, 0.6434212879750436, 0.34567597289096796],
                angle: 2.1701343118778045,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [1.0, -2.0, 1.9344960423944562],
                    size: [1.6299001339256323, 4.075382072110277, 8.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [1.174213754862502, 0.8828447433228437, 3.7568850196612544],
                    r: 2.1441717240895186,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [6.40782663632935, -0.6726815085253839, 3.0],
                    size: [1.8745638134405764, 5.0, 7.9387477099714925],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [8.0, 1.8436850117620218, 4.0],
                    r: 2.7955955817423446,
                },
                axis: [
                    -0.38767625048317855,
                    -0.6844661324155927,
                    -0.5763213265273435,
                ],
                angle: 0.9640401993068397,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [2.936379123347772, 8.145272812672534, 5.0],
                    r: 2.5,
                },
                axis: [0.5219765595949553, -0.8199556094589042, -0.3008136120805782],
                angle: -2.91348071424685,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [2.0, 5.146500865878438, 2.233844574257981],
                    size: [7.496761958576149, 3.764126934177873, 7.6709661939894715],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [4.0, 1.9970910418243166, 7.007410019237934],
                    size: [3.7141647413816727, 7.594542907294061, 7.0],
                },
                axis: [0.1363054630913183, 0.9126597898095059, -0.3107631562934463],
                angle: -2.505682076709248,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [0.630823007039113, 3.288978308125805, 2.8950725379689812],
                    r: 3.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [6.0, 9.530869063186028, 3.0],
                    r: 3.0,
                    len: 5.701761329463238,
                },
                axis: [-0.8381449815868036, 0.7282348408730881, 0.13351236915507703],
                angle: -1.2684292611332473,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [10.0, 7.118939199039325, 9.141882321707998],
                    r: 2.2580987079132933,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [3.187328546327434, 1.0, 4.258889577916882],
                    r: 3.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [5.591865631720917, 1.0, 4.0],
                    r: 3.8764916485149827,
                    len: 4.0,
                },
                axis: [
                    -0.16124840186602518,
                    -0.5585271829879142,
                    0.2557808452336783,
                ],
                angle: -1.7902093303410997,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [10.0, 3.1053830364616064, 1.8969268629285485],
                    r: 3.598781023272151,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [4.028368691971602, 6.074873148899251, 3.0],
                    r: 4.0,
                    len: 9.472724923924865,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 3.0, 10.0],
                    r: 3.0,
                },
                axis: [
                    -0.5371180103919122,
                    0.2907910670092886,
                    -0.48102171498102075,
                ],
                angle: 1.9241696037990188,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [5.249866190993939, 6.0, -1.0020789992822803],
                    size: [2.0, 8.0, 6.0],
                },
                axis: [0.7969447880976499, 0.7911689829160498, 0.47314158665122497],
                angle: -2.9987751736698502,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [8.0, 9.105150711680306, 5.0],
                    r: 3.5,
                },
                axis: [-0.6225800420843861, -0.9787782522858888, 0.2356854578585672],
                angle: 1.5055481877328818,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [2.0, -1.146161992621611, -1.4402777213990685],
                    size: [7.0, 3.0, 7.389644140013463],
                },
                axis: [
                    0.6068522954915847,
                    -0.2458005103092462,
                    -0.03883097906087407,
                ],
                angle: 2.8726451192708335,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [1.0, 1.0, 9.0],
                    r: 3.098745099011297,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [1.9021195992478508, 0.0, 0.566214658671039],
                    size: [6.134940332179487, 1.0, 5.0],
                },
                axis: [
                    -0.8931598366932647,
                    -0.8729645981701577,
                    -0.13872898717444215,
                ],
                angle: 2.01561861029616,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 0.18312770368814846, 3.8191774794299285],
                    r: 3.0,
                },
                axis: [-0.630447040716027, -0.6456307118136864, 0.5256004110068428],
                angle: -1.5109376390599756,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [-0.8484568919616013, 2.896757236094645, 0.0],
                    size: [2.2014030822800152, 6.0, 7.0],
                },
                axis: [0.19803605866089402, -0.15966383228966818, 0.574310988215277],
                angle: 0.35052241319800714,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [9.91151565304014, 4.705671475153216, 3.640592901933921],
                    r: 0.9001034908669397,
                },
                axis: [
                    -0.13835066605046853,
                    0.21836630478590502,
                    0.15776035606953887,
                ],
                angle: -2.473463347081881,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [-0.0, -0.8261173067025667, 5.0],
                    size: [6.201929264132342, 3.3804486824367355, 4.172006229525173],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 9.0, 6.0],
                    r: 2.5,
                },
                axis: [0.6842818597501164, -0.9396490429952493, -0.6569458579972594],
                angle: -2.346520971871167,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [0.0, 4.0, 2.884087166344586],
                    size: [3.3648817762549665, 6.0, 7.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.969407721867762, 2.0, 0.0],
                    r: 4.611352939611856,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [2.6100091659586973, -0.7865876644102281, 3.8691798029726616],
                    size: [6.0, 3.0, 3.0],
                },
                axis: [
                    -0.5896525916106503,
                    -0.025065724304312198,
                    -0.985003320520718,
                ],
                angle: -1.1859352152002405,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.673193942547778, 7.0, 5.0],
                    r: 1.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Box {
                    min: [5.0, 3.0, 4.0],
                    size: [3.0, 4.081838812724041, 7.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [5.562508712287107, 5.071750972350101, 3.0],
                    size: [6.145664277351402, 5.790058489321127, 6.0],
                },
                axis: [0.5825313610226854, -0.1170680348787283, -0.967460963319658],
                angle: 0.903919767113921,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [1.3358581058904961, 6.0, 1.9911207288106525],
                    r: 2.8553614956037356,
                },
                axis: [-0.282282997422298, -0.2800322474302954, 0.6809108548389913],
                angle: 2.709705759006539,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [5.5281853883724095, 7.88418888601067, 0.15544035373011522],
                    r: 3.0,
                },
                axis: [0.700949030862114, 0.9826812354994414, 0.3036620260775713],
                angle: 0.47990984072159826,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [3.746671554303709, 7.0, 1.0],
                    r: 3.9647722942236223,
                    len: 2.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [-0.028902322834752248, 4.0, -2.0],
                    size: [7.801782804991926, 7.849846697839471, 7.027427721166661],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [4.791299112034936, 4.328332273023021, 2.6588318934850887],
                    r: 3.7914004310166955,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [2.766489875086309, 3.0, 3.7310739430834845],
                    size: [3.0, 6.953547697909326, 7.029622650200116],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [3.0, 8.509791286670609, 7.729296256755351],
                    r: 4.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [0.0, 5.276187186316463, 7.0],
                    size: [7.0, 7.0, 6.0],
                },
                axis: [-0.8038600459246708, -0.9534794356650178, 0.9413757605336157],
                angle: -2.852015804128163,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [5.155202344263161, 1.4468560401020658, 3.8857600187634733],
                    r: 2.803355850436832,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [2.9720066421486875, 6.707763013910743, 5.0542559183734355],
                    size: [6.532026171315754, 5.0, 7.0],
                },
                axis: [-0.3195015743202376, -0.5958822571775679, 0.9900501669875676],
                angle: -0.8995294456086329,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [1.0, 1.0, 5.362496105482668],
                    r: 1.0,
                },
                axis: [
                    -0.07754245487471811,
                    0.3723788642487742,
                    0.056644068058767205,
                ],
                angle: 2.7478071718147703,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Box {
                    min: [
                        7.331566080361714,
                        -0.0001960440443342648,
                        -0.26572581784619675,
                    ],
                    size: [2.86736068005138, 6.592716134359267, 5.0],
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [8.981496138838262, 0.4349941721594952, 3.2618775104040507],
                    r: 2.5,
                },
                axis: [
                    -0.3923994764544789,
                    -0.05559804920171074,
                    0.21580758384385845,
                ],
                angle: -0.17232605739834206,
                about: [5.0, 5.0, 5.0],
            },
        ),
    ]);
}

/// Soak: 23 random pairs where the boolean leaves an open shell.
#[test]
#[ignore = "known failure: the boolean leaves an open shell"]
fn soak_open_shell() {
    pinned_all(&[
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 2.9639498969296305, 3.0],
                    r: 1.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [5.0, 0.6878040445761269, 2.0],
                    r: 2.5,
                    len: 7.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [5.0, 5.0, 5.0],
                    r: 2.0,
                },
                axis: [0.13785887879017467, 0.45148025680242276, 0.1500450881728681],
                angle: 2.340247453082779,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [1.0, 4.733430335785204, 3.383458395215561],
                    r: 2.197195673451134,
                    len: 6.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [1.2504422063002452, 9.81151266499874, 4.0],
                    r: 2.8532117599358626,
                    len: 3.0,
                },
                axis: [
                    -0.6493970480256999,
                    0.02500202210250202,
                    -0.40115899126965626,
                ],
                angle: 2.978721088664029,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [7.096890978078264, 3.0, 2.0],
                    r: 3.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [4.100278374190173, 2.458874831102129, -1.2713304095777376],
                    r: 3.5,
                    len: 2.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [9.0, 0.40288445922484706, 2.887864297434227],
                    r: 3.5,
                    len: 9.0,
                },
                axis: [
                    -0.2996112249278837,
                    0.029715028158757484,
                    0.7359012991969245,
                ],
                angle: 0.4427673756784021,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [9.0, 5.260201880745284, 1.0],
                    r: 3.5,
                    len: 1.9157298429116065,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [3.0, 7.015878339829031, 2.0],
                    r: 3.124712579044463,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [0.9637925768105704, 5.85969112553305, 2.8501176564262396],
                    r: 0.5,
                    len: 2.565934958291875,
                },
                axis: [
                    0.14130430631267132,
                    -0.4849520272045509,
                    -0.8989017373136305,
                ],
                angle: 1.5511517072974845,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [7.0, 6.0, 2.0],
                    r: 1.21562618759219,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [9.851421496556446, 5.0, 3.6936721303007536],
                    r: 4.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [3.6274521797327743, 5.0, 0.29793369553558424],
                    r: 3.734146905986992,
                    len: 5.910646835494776,
                },
                axis: [-0.8478309421841099, 0.8579943606166973, 0.7339856442143251],
                angle: -2.4848954176255225,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [9.41140476631392, 2.1557948487620204, 0.0],
                    r: 4.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [9.777376999805407, 7.0, 1.0018371346866868],
                    r: 2.0,
                    len: 6.0,
                },
                axis: [
                    -0.45077093452219996,
                    0.1433365251786578,
                    0.25915205251266094,
                ],
                angle: 1.4804356950744362,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [9.193769180693696, 7.425700341153431, 9.0],
                    r: 1.999632255258067,
                },
                axis: [0.6083782517941576, -0.526867676506197, 0.3769486761537768],
                angle: -0.37964151996818263,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [8.029563731762694, 6.0, 2.0],
                    r: 3.5,
                    len: 2.5817345966342495,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [8.0, 9.0, 4.0],
                    r: 2.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [8.0, 8.0, -1.0],
                    r: 1.0,
                    len: 9.277442735623508,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [4.0, 5.0, 2.0],
                    r: 2.0,
                    len: 10.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [3.077295213238748, 4.9113757795839135, 2.567539697125709],
                    r: 3.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [6.0, 3.852744311642656, 2.0],
                    r: 1.1087598906216116,
                    len: 8.0,
                },
                axis: [
                    -0.10340755613788288,
                    0.08945687934599644,
                    0.5006606882614861,
                ],
                angle: 0.6544842532884152,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [2.0, 6.0, 1.0],
                    r: 4.95782941920383,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [8.758153736109753, 6.0, -1.0],
                    r: 1.0,
                    len: 10.0,
                },
                axis: [-0.721541252652645, 0.6868740498180987, 0.2091873060620102],
                angle: -2.9368384511234007,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [4.404959593003731, 7.647986462959696, 6.0],
                    r: 5.0,
                },
                axis: [0.7557038103867944, 0.6373826756874432, -0.14790855224808563],
                angle: 1.5410756783374806,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.0, 5.0, 5.0],
                    r: 2.642497824532193,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [1.0, 4.0, 2.0],
                    r: 2.548976691756756,
                    len: 7.0,
                },
                axis: [
                    0.6649846799861485,
                    0.39792582527940024,
                    -0.11159814855884154,
                ],
                angle: 1.0515029257522883,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [2.8007803766196626, 2.672501018721498, 0.8533154883416723],
                    r: 2.5,
                    len: 6.739368023291523,
                },
                axis: [0.6734202771036994, 0.37791923051144805, 0.3058828474434576],
                angle: 2.768461704031372,
                about: [5.0, 5.0, 5.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [4.0, 1.0, 3.0],
                    r: 4.889364900799912,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [8.0, 10.0, 4.736127821043552],
                    r: 2.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [10.0, 9.0, -2.0],
                    r: 1.0,
                    len: 8.912798217475569,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [9.0, 8.0, 2.193850769992017],
                    r: 3.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [4.514519722719516, 7.917244534924396, 3.0],
                    r: 3.3410185847136944,
                    len: 7.751279408656108,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [6.418800690069108, 10.0, 4.571359792289296],
                    r: 3.7847683486462143,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [3.0, 10.0, -2.0],
                    r: 3.686765047261734,
                    len: 6.59873669203034,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [7.233596188125141, 6.791335243190405, 2.4185980035365775],
                    r: 0.5751990826141342,
                    len: 3.3307199571376924,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [3.0, 3.131540133655195, 3.0262316360413735],
                    r: 1.0,
                    len: 6.0,
                },
                axis: [0.3663795669441494, -0.4150133820383879, 0.7753514205057861],
                angle: 2.155956588555818,
                about: [5.0, 5.0, 5.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [5.0, 0.0, 6.0],
                    r: 4.313377388152732,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [2.8147203502393667, 0.11718833927825356, 2.162126743670587],
                    r: 3.0,
                    len: 6.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [8.137091890256396, 7.029043523649929, 3.850887153288125],
                    r: 4.074887659796242,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [2.0, 9.0, -1.0],
                    r: 3.9999863469415895,
                    len: 2.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [8.0, 3.0, 1.0],
                    r: 4.0,
                    len: 9.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [2.0, 2.176795489460358, 6.0],
                    r: 4.5,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [3.9370032316150914, 2.0, 2.0],
                    r: 1.9697216376440632,
                    len: 7.60391204727828,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [2.543331517068139, 0.030289233692251116, 9.0],
                    r: 4.50237848654653,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
    ]);
}

/// Soak: 4 random pairs where the union written to STEP doesn't read back as the same valid solid.
#[test]
#[ignore = "known failure: the union written to STEP doesn't read back as the same valid solid"]
fn soak_step_round_trip() {
    pinned_all(&[
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [7.0, 2.0, -0.38475413885053467],
                    r: 2.5,
                    len: 7.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 0,
                    base: [4.0, 1.0, 1.0],
                    r: 1.0,
                    len: 4.366635668214542,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [5.0, 8.0, 3.0],
                    r: 4.442422371119262,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [9.332710091316532, 6.0, 0.0],
                    r: 2.0,
                    len: 3.902626479054698,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Cyl {
                    axis: 2,
                    base: [4.093826004876336, 6.201257193206578, 0.758541856752819],
                    r: 2.5,
                    len: 5.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Cyl {
                    axis: 1,
                    base: [5.0, 4.0, 3.8987018561876408],
                    r: 3.481478884084643,
                    len: 4.491268709909727,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
        (
            Placed {
                shape: Shape::Sphere {
                    c: [5.0, 2.0, 10.0],
                    r: 1.0,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
            Placed {
                shape: Shape::Sphere {
                    c: [4.0, 2.0, 7.065422676430007],
                    r: 2.936799833630305,
                },
                axis: [0.0, 0.0, 1.0],
                angle: 0.0,
                about: [0.0, 0.0, 0.0],
            },
        ),
    ]);
}

/// Soak: 1 random pairs where the volumes disagree with the tool order and with vol(A ∪ B) + vol(A ∩ B) = vol(A) + vol(B) (by 3e-3).
#[test]
#[ignore = "known failure: the volumes disagree with the tool order and with vol(A ∪ B) + vol(A ∩ B) = vol(A) + vol(B) (by 3e-3)"]
fn soak_tool_order_volume() {
    pinned_all(&[(
        Placed {
            shape: Shape::Cyl {
                axis: 2,
                base: [8.0, 6.0, 2.0],
                r: 2.972306671732756,
                len: 10.0,
            },
            axis: [
                0.000999810887182262,
                0.09609918717348598,
                0.24598691636771663,
            ],
            angle: -2.621825482659542,
            about: [5.0, 5.0, 5.0],
        },
        Placed {
            shape: Shape::Sphere {
                c: [2.0, 0.33387813388889187, 8.75439111677984],
                r: 3.0,
            },
            axis: [-0.9571491625653146, 0.12051437556382916, 0.5906449755715617],
            angle: 0.45475891268164625,
            about: [5.0, 5.0, 5.0],
        },
    )]);
}

/// Soak: 3 extrude sequences where a step fails at the boolean's validity stage: an edge used by three or more faces.
#[test]
#[ignore = "known failure: a step fails at the boolean's validity stage: an edge used by three or more faces"]
fn soak_sequence_edge_three_faces() {
    sequences_all(&[
        vec![
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 0,
                    o: [0.5, 0.0, 0.0],
                    outer: [2.0, 2.0, 4.5, 7.0],
                    hole: Some([2.5, 2.5, 4.0, 6.0]),
                    depth: 6.0,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 1.0],
                    outer: [3.0, 0.5, 5.0, 3.5],
                    hole: None,
                    depth: 3.5,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 4.5],
                    outer: [5.0, 1.5, 9.5, 6.5],
                    hole: None,
                    depth: 1.5,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 0,
                    o: [0.0, 0.0, 0.0],
                    outer: [3.0, 1.5, 8.0, 4.0],
                    hole: Some([3.5, 2.0, 7.5, 3.0]),
                    depth: 4.0,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 2.5],
                    outer: [1.5, 2.5, 6.5, 7.0],
                    hole: None,
                    depth: 2.0,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 4.5, 0.0],
                    outer: [4.5, 3.0, 8.0, 7.0],
                    hole: None,
                    depth: 1.0,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 0.5],
                    outer: [5.5, 2.5, 7.5, 4.0],
                    hole: None,
                    depth: 4.5,
                },
            },
        ],
        vec![
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 2.5],
                    outer: [2.0, 2.5, 5.0, 5.5],
                    hole: Some([2.5, 3.0, 3.5, 4.5]),
                    depth: 4.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 2.0],
                    outer: [1.5, 5.5, 5.0, 8.0],
                    hole: None,
                    depth: 5.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 0,
                    o: [0.5, 0.0, 0.0],
                    outer: [4.5, 1.0, 10.5, 6.0],
                    hole: None,
                    depth: 3.5,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 0,
                    o: [2.0, 0.0, 0.0],
                    outer: [6.0, 2.5, 9.0, 4.5],
                    hole: None,
                    depth: 1.5,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 0,
                    o: [3.5, 0.0, 0.0],
                    outer: [1.0, 6.0, 2.5, 9.5],
                    hole: None,
                    depth: 3.0,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 5.0, 0.0],
                    outer: [3.5, 3.0, 6.5, 7.5],
                    hole: None,
                    depth: 4.5,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 5.5, 0.0],
                    outer: [2.0, 5.0, 7.0, 8.0],
                    hole: None,
                    depth: 3.5,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 5.5, 0.0],
                    outer: [0.5, 1.5, 5.5, 5.0],
                    hole: None,
                    depth: 1.5,
                },
            },
        ],
        vec![
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 3.0, 0.0],
                    outer: [4.0, 5.0, 6.0, 8.5],
                    hole: Some([4.5, 5.5, 5.5, 7.0]),
                    depth: 1.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 0,
                    o: [1.0, 0.0, 0.0],
                    outer: [5.5, 3.0, 11.0, 5.5],
                    hole: None,
                    depth: 3.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 0,
                    o: [0.5, 0.0, 0.0],
                    outer: [2.5, 5.0, 7.5, 8.5],
                    hole: None,
                    depth: 4.0,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 0.5, 0.0],
                    outer: [5.0, 4.0, 8.5, 6.5],
                    hole: None,
                    depth: 4.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 4.0, 0.0],
                    outer: [3.5, 5.5, 8.5, 9.5],
                    hole: None,
                    depth: 4.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 1.5, 0.0],
                    outer: [2.0, 4.0, 5.0, 6.5],
                    hole: Some([2.5, 4.5, 4.0, 5.5]),
                    depth: 2.5,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 2.5],
                    outer: [3.5, 4.0, 8.5, 6.0],
                    hole: None,
                    depth: 4.0,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 3.0],
                    outer: [4.0, 5.0, 7.0, 8.0],
                    hole: None,
                    depth: 6.0,
                },
            },
        ],
    ]);
}

/// Soak: 2 extrude sequences where a step leaves an open shell.
#[test]
#[ignore = "known failure: a step leaves an open shell"]
fn soak_sequence_open_shell() {
    sequences_all(&[
        vec![
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 1.0, 0.0],
                    outer: [0.0, 2.5, 4.5, 6.5],
                    hole: Some([0.5, 3.0, 3.0, 4.5]),
                    depth: 4.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 3.0],
                    outer: [4.5, 0.5, 8.0, 6.0],
                    hole: None,
                    depth: 3.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 0,
                    o: [4.5, 0.0, 0.0],
                    outer: [5.5, 0.5, 8.0, 3.0],
                    hole: None,
                    depth: 3.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 0.5],
                    outer: [3.0, 3.5, 8.5, 9.0],
                    hole: Some([3.5, 4.0, 4.5, 5.5]),
                    depth: 4.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 3.0],
                    outer: [3.5, 2.5, 7.0, 7.5],
                    hole: None,
                    depth: 4.5,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 2.5],
                    outer: [0.5, 2.5, 3.0, 4.5],
                    hole: Some([1.0, 3.0, 1.5, 3.5]),
                    depth: 3.5,
                },
            },
        ],
        vec![
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 4.0],
                    outer: [3.0, 1.0, 7.5, 4.5],
                    hole: None,
                    depth: 3.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 2.0, 0.0],
                    outer: [5.5, 4.5, 8.5, 10.5],
                    hole: Some([6.0, 5.0, 6.5, 5.5]),
                    depth: 5.0,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 3.5, 0.0],
                    outer: [5.0, 2.0, 7.0, 5.0],
                    hole: Some([5.5, 2.5, 6.5, 4.0]),
                    depth: 3.5,
                },
            },
            Step {
                op: BoolOp::Union,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 1.5],
                    outer: [2.0, 6.0, 4.0, 11.0],
                    hole: None,
                    depth: 2.5,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 0,
                    o: [4.0, 0.0, 0.0],
                    outer: [5.0, 2.0, 9.5, 5.5],
                    hole: None,
                    depth: 2.5,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 2,
                    o: [0.0, 0.0, 5.5],
                    outer: [5.0, 4.0, 8.0, 9.0],
                    hole: None,
                    depth: 6.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 1,
                    o: [0.0, 2.5, 0.0],
                    outer: [2.5, 2.0, 6.5, 6.5],
                    hole: Some([3.0, 2.5, 5.5, 6.0]),
                    depth: 5.0,
                },
            },
            Step {
                op: BoolOp::Subtract,
                shape: Shape::Prism {
                    axis: 0,
                    o: [5.5, 0.0, 0.0],
                    outer: [2.5, 4.0, 8.5, 6.5],
                    hole: Some([3.0, 4.5, 4.0, 5.0]),
                    depth: 4.0,
                },
            },
        ],
    ]);
}
