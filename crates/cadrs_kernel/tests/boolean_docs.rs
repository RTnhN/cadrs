//! Booleans from the user's documents that failed on the native kernel and built on OCCT: each
//! one's inputs, dumped where it failed (`CADRS_DUMP_DIR`), as a small fixture in
//! `tests/data/docs/`, or the geometry rebuilt in code. Every result must be a valid closed
//! solid (`CheckLevel::Full`) of the volume the case gives.

use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::native::topo::{Body, CheckLevel};
use cadrs_kernel::*;
use nalgebra::{Point2, Point3, Vector3};
use std::f64::consts::{PI, TAU};

fn body(name: &str) -> Body {
    let path = format!("{}/tests/data/docs/{name}.body", env!("CARGO_MANIFEST_DIR"));
    cadrs_kernel::native::bodyio::read(&std::fs::read(&path).unwrap()).unwrap()
}

/// `op` of the fixtures (the target and its tools in turn): the result, checked closed and
/// valid, and its volume checked against `want` (relative 1e-6).
fn run(target: &str, tools: &[&str], op: BoolOp, want: f64) -> BodyId {
    let mut k = NativeKernel::new();
    let a = k.insert(body(target));
    let ts: Vec<BodyId> = tools.iter().map(|t| k.insert(body(t))).collect();
    let r = k
        .boolean(op, a, &ts)
        .unwrap_or_else(|e| panic!("{target} {op:?} {tools:?}: {e}"))
        .bodies[0];
    let b = k.get(r).unwrap();
    let issues = b.check_with(CheckLevel::Full);
    assert!(issues.is_empty(), "{target}: {issues:?}");
    assert!(!b.solids.is_empty(), "{target}: no solid");
    let v = k.mass_properties(r).unwrap().volume;
    assert!(
        (v - want).abs() <= 1e-6 * want,
        "{target}: volume {v}, expected {want}"
    );
    r
}

/// Modeling Practice Drawings 93, Extrude 7: an upright whose round slots rest on the base's top
/// face, tangent to it along lines that touch the rims of the base's holes. The surfaces cross
/// at a point of touch, where their curve runs within the tolerance of the rim for a stretch:
/// a point, not a cut (and the touch's root found again a hair off is the same vertex).
#[test]
fn upright_resting_on_hole_rims() {
    run("mp93_base", &["mp93_upright"], BoolOp::Union, 79380.0);
}

/// Modeling Practice Drawings 93, Extrudes 5 and 6: a cylinder over the upright's round top,
/// longer than it, merged with it on their common surface (its seam put where it doesn't run
/// along the top's straight edge), then a slot cut through both.
#[test]
fn merged_cylinder_seam_then_slot() {
    let mut k = NativeKernel::new();
    let a = k.insert(body("mp93_part"));
    let b = k.insert(body("mp93_round"));
    let u = k.boolean(BoolOp::Union, a, &[b]).unwrap().bodies[0];
    // The merged cylinder face has its seam on its own, not over another edge of it.
    let ub = k.get(u).unwrap();
    for f in &ub.faces {
        for l in &f.loops {
            let mut uses = std::collections::HashMap::new();
            for c in l {
                *uses.entry(c.edge).or_insert(0) += 1;
            }
            assert!(
                uses.values().all(|&n| n <= 2),
                "an edge used three times in a loop"
            );
        }
    }
    assert!((k.mass_properties(u).unwrap().volume - 82914.2917352884).abs() < 1e-6 * 82914.3);
    let s = k.insert(body("mp93_slot"));
    let r = k.boolean(BoolOp::Subtract, u, &[s]).unwrap().bodies[0];
    let issues = k.get(r).unwrap().check_with(CheckLevel::Full);
    assert!(issues.is_empty(), "{issues:?}");
}

/// weller led block, Extrude 1: a slot against a countersink, a cone past 90° (nearly flat).
/// Its plane sections keep to the cone's own sheet (the double cone's other nappe lies a hair
/// away), and a plane through its axis meets it in rulings that open down the axis.
#[test]
fn slot_through_flat_cone() {
    run(
        "weller_cone",
        &["weller_slot"],
        BoolOp::Intersect,
        3.4996247875427877,
    );
}

/// babygatefix, Boolean 1: clip arms touching along an edge (several solids in one body)
/// united with a ring: one solid of the target at a time. (The union's volume lies between the
/// largest input's and their sum.)
#[test]
fn union_of_solids_touching_along_an_edge() {
    let names = [
        "babygate_000",
        "babygate_001",
        "babygate_002",
        "babygate_003",
        "babygate_004",
    ];
    let mut k = NativeKernel::new();
    let vols: Vec<f64> = names
        .iter()
        .map(|n| {
            let b = k.insert(body(n));
            k.mass_properties(b).unwrap().volume
        })
        .collect();
    let (max, sum) = (
        vols.iter().copied().fold(0.0, f64::max),
        vols.iter().sum::<f64>(),
    );
    let v = 13851.357294478015;
    assert!(v > max && v < sum, "{vols:?}");
    run(names[0], &names[1..], BoolOp::Union, v);
}

/// d4 desk device, Extrude 17: bosses whose holes merge with the part's holes on one cylinder;
/// re-seaming one merged face splits an edge another merged face already uses.
#[test]
fn merged_faces_share_a_split_edge() {
    run("d4_target", &["d4_tool"], BoolOp::Union, 1669.5132122234957);
}

/// trimmer, Extrude 11: a tool of two solids touching face to face (an extrude of two regions
/// side by side), intersected with and added to a part: one tool solid at a time.
#[test]
fn tool_of_solids_touching_face_to_face() {
    let mut k = NativeKernel::new();
    let rect = |x0: f64, y0: f64, x1: f64, y1: f64| {
        let c = [
            Point2::new(x0, y0),
            Point2::new(x1, y0),
            Point2::new(x1, y1),
            Point2::new(x0, y1),
        ];
        let curves = (0..4)
            .map(|i| Curve2::Line {
                a: c[i],
                b: c[(i + 1) % 4],
                source: None,
            })
            .collect();
        Profile::new(
            Plane::top(),
            vec![Region {
                outer: Loop { curves },
                holes: vec![],
                source: None,
            }],
        )
    };
    let part = k
        .extrude(&rect(0.0, 0.0, 40.0, 10.0), Extent::Blind(2.0))
        .unwrap()
        .bodies[0];
    let left = k
        .extrude(&rect(5.0, -5.0, 20.0, 15.0), Extent::Blind(4.0))
        .unwrap()
        .bodies[0];
    let right = k
        .extrude(&rect(20.0, -5.0, 35.0, 15.0), Extent::Blind(4.0))
        .unwrap()
        .bodies[0];
    let mut tool = (**k.get(left).unwrap()).clone();
    tool.append(k.get(right).unwrap());
    let tool = k.insert(tool);
    for (op, want) in [
        (BoolOp::Intersect, 30.0 * 10.0 * 2.0),
        (
            BoolOp::Union,
            40.0 * 10.0 * 2.0 + 2.0 * 15.0 * 20.0 * 4.0 - 30.0 * 10.0 * 2.0,
        ),
    ] {
        let r = k.boolean(op, part, &[tool]).unwrap().bodies[0];
        let issues = k.get(r).unwrap().check_with(CheckLevel::Full);
        assert!(issues.is_empty(), "{op:?}: {issues:?}");
        let v = k.mass_properties(r).unwrap().volume;
        assert!((v - want).abs() < 1e-6 * want, "{op:?}: {v} vs {want}");
    }
}

/// Dr Browns cap, Revolve: a profile bounded by an arc whose centre lies behind the axis (a
/// spindle torus, minor radius past the major) turned full circle. The face lies on the torus's
/// part past its axis, whose p-curves are on the far meridian; the body is valid and its volume
/// is Pappus's.
#[test]
fn revolve_arc_centred_behind_the_axis() {
    let mut k = NativeKernel::new();
    // In the XZ plane (profile y is z): x from 2 out to the arc of radius 26 about (−20, 0),
    // z from 0 to 3.
    let (r, cx, h) = (26.0_f64, -20.0_f64, 3.0_f64);
    let top = (h / r).asin();
    let xa = |z: f64| cx + (r * r - z * z).sqrt();
    let curves = vec![
        Curve2::Line {
            a: Point2::new(2.0, 0.0),
            b: Point2::new(xa(0.0), 0.0),
            source: None,
        },
        Curve2::Arc {
            center: Point2::new(cx, 0.0),
            radius: r,
            start_angle: 0.0,
            sweep: top,
            source: None,
        },
        Curve2::Line {
            a: Point2::new(xa(h), h),
            b: Point2::new(2.0, h),
            source: None,
        },
        Curve2::Line {
            a: Point2::new(2.0, h),
            b: Point2::new(2.0, 0.0),
            source: None,
        },
    ];
    let xz = Plane {
        origin: Point3::origin(),
        x_dir: Vector3::x_axis(),
        normal: -Vector3::y_axis(),
    };
    let profile = Profile::new(
        xz,
        vec![Region {
            outer: Loop { curves },
            holes: vec![],
            source: None,
        }],
    );
    let z = Axis {
        origin: Point3::origin(),
        dir: Vector3::z_axis(),
    };
    let b = k.revolve(&profile, z, TAU).unwrap().bodies[0];
    let issues = k.get(b).unwrap().check_with(CheckLevel::Full);
    assert!(issues.is_empty(), "{issues:?}");
    // V = π ∫ (x_a(z)² − 2²) dz, by Simpson's rule.
    let n = 2000;
    let f = |z: f64| PI * (xa(z) * xa(z) - 4.0);
    let dz = h / n as f64;
    let want = (0..=n)
        .map(|i| {
            f(i as f64 * dz)
                * if i == 0 || i == n {
                    1.0
                } else if i % 2 == 1 {
                    4.0
                } else {
                    2.0
                }
        })
        .sum::<f64>()
        * dz
        / 3.0;
    let v = k.mass_properties(b).unwrap().volume;
    assert!((v - want).abs() < 1e-6 * want, "{v} vs {want}");
    // And it unites with a cylinder through it (the document's next step).
    let disk = {
        let c = Curve2::Circle {
            center: Point2::origin(),
            radius: 4.0,
            source: None,
        };
        Profile::new(
            Plane::top(),
            vec![Region {
                outer: Loop { curves: vec![c] },
                holes: vec![],
                source: None,
            }],
        )
    };
    let post = k.extrude(&disk, Extent::Blind(5.0)).unwrap().bodies[0];
    let u = k.boolean(BoolOp::Union, b, &[post]).unwrap().bodies[0];
    assert!(k.get(u).unwrap().check_with(CheckLevel::Full).is_empty());
}

/// gridfinity die tap 2x2, Boolean 3: walls of a pattern turned by a hair (7e-5 rad) against a
/// knob along its axis: their planes cut its cylinders in ellipses 2·10⁵ long, whose part in
/// the faces' box is a sliver of the parameter between even samples, found from the box's
/// heights along the axis.
#[test]
fn walls_nearly_along_a_cylinder_axis() {
    run(
        "gridfinity_knob",
        &["gridfinity_wall"],
        BoolOp::Intersect,
        552.188828679068,
    );
}

/// Dr Browns cap, Boolean 3: a post through the cap's wall, a spindle torus. The torus's
/// distance (by which the surface intersection culls cells) is to the nearer of its two
/// meridians: from the near one alone, the wall seemed far from the post and the union kept
/// both whole (its volume their sum).
#[test]
fn post_through_a_spindle_torus_wall() {
    let mut k = NativeKernel::new();
    let vols: Vec<f64> = ["drbrowns_cap", "drbrowns_post"]
        .iter()
        .map(|n| {
            let b = k.insert(body(n));
            k.mass_properties(b).unwrap().volume
        })
        .collect();
    let v = 25903.21270796864;
    assert!(
        v < vols[0] + vols[1] - 100.0 && v > vols[0].max(vols[1]),
        "{vols:?}"
    );
    run("drbrowns_cap", &["drbrowns_post"], BoolOp::Union, v);
}

/// A slab from offsetting SMLib's `offset_bracket2.smb` (its face 27, a cylinder, by 3.5): the
/// bracket's edge there is a cubic 7e-4 off its cylinder (tolerance 1.4e-3), so the slab's rim
/// is too. A box whose face y = 0 cuts the slab's outer cylinder at a glancing angle: the
/// rim's crossing of that face lies 1.6e-3 along it from the plane–cylinder line, within the
/// vertex's tolerance but past the run's; the line has to end there all the same (it had no
/// end at that crossing and left the shell open).
#[test]
fn glancing_cut_at_a_rim_off_its_cylinder() {
    let slab =
        cadrs_kernel::native::bodyio::read(include_bytes!("data/offset_slab_spline_rim.body"))
            .unwrap();
    for op in [BoolOp::Union, BoolOp::Subtract, BoolOp::Intersect] {
        let mut k = NativeKernel::new();
        let a = k.insert(slab.clone());
        let cadrs_kernel::native::draw::Shape::Body(b) =
            k.box_at(-16.0, -3.0, 5.0, 12.0, 3.0, 3.0).unwrap()
        else {
            panic!("a box");
        };
        let r = k
            .boolean(op, a, &[b])
            .unwrap_or_else(|e| panic!("{op:?}: {e}"));
        for id in r.bodies {
            let body = k.get(id).unwrap();
            assert!(body.check().is_empty(), "{op:?}: {:?}", body.check());
        }
    }
}

/// Whether a hole of a face on a surface periodic in u straddles the face's seam (its
/// coedges' middles in different period windows counted from the seam's lower side): the seam
/// edge then runs through the hole.
fn hole_across_seam(b: &Body) -> bool {
    b.faces.iter().any(|f| {
        let l0 = &f.loops[0];
        let mid = |c: &cadrs_kernel::native::topo::Coedge| {
            let e = &b.edges[c.edge];
            c.pc.point(0.5 * (e.t[0] + e.t[1])).x
        };
        let seam = l0
            .iter()
            .filter(|c| l0.iter().any(|d| d.edge == c.edge && d.fwd != c.fwd))
            .map(mid)
            .reduce(f64::min);
        let Some(lo) = seam.filter(|_| f.surf.u_periodic()) else {
            return false;
        };
        f.loops.iter().skip(1).any(|l| {
            let w: Vec<f64> = l.iter().map(|c| ((mid(c) - lo) / TAU).floor()).collect();
            w.iter().any(|x| *x != w[0])
        })
    })
}

/// Dr Browns cap, the union before Fillet 1 (the document's Boolean 3): the dome's face has a
/// window across its seam (the seam split at the window, the window part of the outer loop);
/// the union imprints the collar's face on it and merges the pieces again, and the merged
/// face's new seam ran straight through the window, now a hole straddling it ("a hole outside
/// the outer loop"; the fillet then couldn't place its cuts). The merge puts the seam clear of
/// the hole. The inputs are a private document's, kept out of the repository:
/// `$CADRS_DOC_FIXTURES/drbrowns_window_{target,tool}.body` (skipped without it).
#[test]
fn drbrowns_window_across_a_torus_seam() {
    let Ok(dir) = std::env::var("CADRS_DOC_FIXTURES") else {
        return;
    };
    let read = |n: &str| {
        cadrs_kernel::native::bodyio::read(&std::fs::read(format!("{dir}/{n}.body")).unwrap())
            .unwrap()
    };
    let mut k = NativeKernel::new();
    let a = k.insert(read("drbrowns_window_target"));
    let t = k.insert(read("drbrowns_window_tool"));
    let r = k.boolean(BoolOp::Union, a, &[t]).unwrap().bodies[0];
    let b = k.get(r).unwrap();
    assert!(
        b.check_with(CheckLevel::Full).is_empty(),
        "{:?}",
        b.check_with(CheckLevel::Full)
    );
    assert!(!hole_across_seam(b), "a hole straddles its face's seam");
}

/// Booleans with an empty operand (SMLib's 2D exclusive or of disjoint squares: the union less
/// their empty common part): A − ∅ = A ∪ ∅ = ∅ ∪ A = A, A ∩ ∅ = ∅ − A = ∅.
#[test]
fn empty_operands() {
    use cadrs_kernel::native::draw::Shape;
    let mut k = NativeKernel::new();
    let a = k.box_at(0.0, 0.0, 0.0, 1.0, 1.0, 1.0).unwrap();
    let Shape::Body(id) = a else { panic!("a box") };
    for r in [
        k.bcut(&a, &Shape::Empty),
        k.bfuse(&a, &Shape::Empty),
        k.bfuse(&Shape::Empty, &a),
        k.bcut(&a, &Shape::Compound(vec![])),
    ] {
        assert!(matches!(r, Ok(Shape::Body(b)) if b == id), "{r:?}");
    }
    for r in [k.bcommon(&a, &Shape::Empty), k.bcut(&Shape::Empty, &a)] {
        assert!(matches!(r, Ok(Shape::Empty)), "{r:?}");
    }
}
