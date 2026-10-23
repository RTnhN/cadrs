//! Blocks with fillets, chamfers, shells and holes whose volume, area, face count, box and
//! centroid follow from a formula, not from an earlier run: every expectation here is derived
//! by hand (the derivation is next to it), so a failure is the kernel's, never a stale fixture's.
//! Every result must also pass the full validity check.

use std::f64::consts::{PI, SQRT_2};

use cadrs_kernel::native::kernel::NativeKernel;
use cadrs_kernel::native::topo::CheckLevel;
use cadrs_kernel::*;
use nalgebra::{Point2, Point3};

/// The cube's side (mm).
const A: f64 = 10.0;

/// Relative tolerance on volume and area: the surfaces are exact (planes, cylinders, spheres,
/// tori), so only rounding is allowed.
const REL: f64 = 1e-9;

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
    let c = [
        Point2::new(x0, y0),
        Point2::new(x1, y0),
        Point2::new(x1, y1),
        Point2::new(x0, y1),
    ];
    Loop {
        curves: (0..4)
            .map(|i| Curve2::Line {
                a: c[i],
                b: c[(i + 1) % 4],
                source: None,
            })
            .collect(),
    }
}

fn circle(x: f64, y: f64, r: f64) -> Loop {
    Loop {
        curves: vec![Curve2::Circle {
            center: Point2::new(x, y),
            radius: r,
            source: None,
        }],
    }
}

fn extrude(k: &mut NativeKernel, outer: Loop, holes: Vec<Loop>, h: f64) -> BodyId {
    let region = Region {
        outer,
        holes,
        source: None,
    };
    k.extrude(&Profile::new(Plane::top(), vec![region]), Extent::Blind(h))
        .unwrap()
        .bodies[0]
}

/// The cube [0, A]³.
fn cube(k: &mut NativeKernel) -> BodyId {
    extrude(k, rect(0.0, 0.0, A, A), vec![], A)
}

/// The edges of `body` whose middle `pick` accepts.
fn edges(k: &NativeKernel, body: BodyId, pick: impl Fn(Point3<f64>) -> bool) -> Vec<EdgeId> {
    k.edges(body)
        .unwrap()
        .into_iter()
        .filter(|e| pick(e.mid))
        .map(|e| e.id)
        .collect()
}

/// The face of `body` whose centroid is `at`.
fn face(k: &NativeKernel, body: BodyId, at: [f64; 3]) -> FaceId {
    let at = Point3::from(at);
    k.faces(body)
        .unwrap()
        .into_iter()
        .find(|f| (f.center - at).norm() < 1e-9)
        .unwrap_or_else(|| panic!("no face centred at {at:?}"))
        .id
}

/// Is `v` within `1e-9` of `w`?
fn near(v: f64, w: f64) -> bool {
    (v - w).abs() < 1e-9
}

/// What a result must be.
struct Want {
    volume: f64,
    area: f64,
    faces: usize,
    min: [f64; 3],
    max: [f64; 3],
    /// The centroid, where symmetry or a short sum gives it.
    centroid: Option<[f64; 3]>,
}

#[track_caller]
fn check(k: &NativeKernel, body: BodyId, want: Want) {
    let issues = k.get(body).unwrap().check_with(CheckLevel::Full);
    assert!(issues.is_empty(), "invalid body: {issues:?}");
    let m = k.mass_properties(body).unwrap();
    let close = |what: &str, got: f64, want: f64| {
        assert!(
            (got - want).abs() <= REL * want.abs().max(1.0),
            "{what}: got {got}, want {want} (off by {:e})",
            got - want
        );
    };
    close("volume", m.volume, want.volume);
    close("surface area", m.surface_area, want.area);
    let faces = k.faces(body).unwrap();
    let face_area: f64 = faces.iter().map(|f| f.area).sum();
    close("sum of face areas", face_area, want.area);
    assert_eq!(faces.len(), want.faces, "face count");
    let b = k.bounding_box(body).unwrap();
    for i in 0..3 {
        // `bounding_box` is documented tight.
        close("box min", b.min[i], want.min[i]);
        close("box max", b.max[i], want.max[i]);
    }
    if let Some(c) = want.centroid {
        for i in 0..3 {
            close("centroid", m.center_of_mass[i], c[i]);
        }
    }
}

const LO: [f64; 3] = [0.0; 3];
const HI: [f64; 3] = [A; 3];
const MID: [f64; 3] = [A / 2.0; 3];

// ---------------------------------------------------------------------------------------------
// Extrudes

#[test]
fn a_box() {
    // 10 × 20 × 30: V = abc, S = 2(ab + bc + ca).
    let mut k = NativeKernel::new();
    let b = extrude(&mut k, rect(0.0, 0.0, 10.0, 20.0), vec![], 30.0);
    check(
        &k,
        b,
        Want {
            volume: 6000.0,
            area: 2.0 * (200.0 + 600.0 + 300.0),
            faces: 6,
            min: LO,
            max: [10.0, 20.0, 30.0],
            centroid: Some([5.0, 10.0, 15.0]),
        },
    );
    assert_eq!(k.edges(b).unwrap().len(), 12);
}

#[test]
fn a_cylinder() {
    // Radius 4, height 10: V = πr²h, S = 2πr² + 2πrh.
    let (r, h) = (4.0, 10.0);
    let mut k = NativeKernel::new();
    let b = extrude(&mut k, circle(0.0, 0.0, r), vec![], h);
    check(
        &k,
        b,
        Want {
            volume: PI * r * r * h,
            area: 2.0 * PI * r * r + 2.0 * PI * r * h,
            faces: 3,
            min: [-r, -r, 0.0],
            max: [r, r, h],
            centroid: Some([0.0, 0.0, h / 2.0]),
        },
    );
}

#[test]
fn a_cube_with_a_round_hole_through() {
    // A hole of radius 3 down the middle: the caps lose πr² each, the bore adds 2πr·A.
    let r = 3.0;
    let mut k = NativeKernel::new();
    let b = extrude(
        &mut k,
        rect(0.0, 0.0, A, A),
        vec![circle(A / 2.0, A / 2.0, r)],
        A,
    );
    check(
        &k,
        b,
        Want {
            volume: (A * A - PI * r * r) * A,
            area: 2.0 * (A * A - PI * r * r) + 4.0 * A * A + 2.0 * PI * r * A,
            faces: 7,
            min: LO,
            max: HI,
            centroid: Some(MID),
        },
    );
}

// ---------------------------------------------------------------------------------------------
// Fillets

#[test]
fn a_cube_with_one_filleted_edge() {
    // The vertical edge at (A, A): the section loses a corner of (1 − π/4) r² along A. The two
    // side faces lose an r × A strip each, the quarter cylinder adds πr/2 × A, the top and
    // bottom lose (1 − π/4) r² each.
    let r = 2.0;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let e = edges(&k, c, |m| near(m.x, A) && near(m.y, A));
    assert_eq!(e.len(), 1);
    let b = k.fillet(c, &e, r).unwrap().bodies[0];
    let corner = (1.0 - PI / 4.0) * r * r;
    check(
        &k,
        b,
        Want {
            volume: A * A * A - corner * A,
            area: 6.0 * A * A - 2.0 * r * A + PI / 2.0 * r * A - 2.0 * corner,
            faces: 7,
            min: LO,
            max: HI,
            centroid: None,
        },
    );
}

#[test]
fn a_cube_with_its_four_vertical_edges_filleted() {
    // A rounded square A × A, corners r, extruded A: section A² − (4 − π) r², perimeter
    // 4(A − 2r) + 2πr.
    let r = 2.5;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let e = edges(&k, c, |m| near(m.z, A / 2.0));
    assert_eq!(e.len(), 4);
    let b = k.fillet(c, &e, r).unwrap().bodies[0];
    let section = A * A - (4.0 - PI) * r * r;
    let perimeter = 4.0 * (A - 2.0 * r) + 2.0 * PI * r;
    check(
        &k,
        b,
        Want {
            volume: section * A,
            area: 2.0 * section + perimeter * A,
            faces: 10,
            min: LO,
            max: HI,
            centroid: Some(MID),
        },
    );
}

#[test]
fn a_cube_with_every_edge_filleted() {
    // All twelve edges at one radius make a ball corner at each vertex: the cube is the
    // Minkowski sum of a cube of side L = A − 2r and a ball of radius r, so
    // V = L³ + 6L²r + 3πLr² + 4/3 πr³ and S = 6L² + 6πrL + 4πr².
    // Faces: 6 flats, 12 quarter cylinders, 8 eighth spheres.
    let r = 2.0;
    let l = A - 2.0 * r;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let e = edges(&k, c, |_| true);
    assert_eq!(e.len(), 12);
    let b = k.fillet(c, &e, r).unwrap().bodies[0];
    check(
        &k,
        b,
        Want {
            volume: l * l * l + 6.0 * l * l * r + 3.0 * PI * l * r * r + 4.0 / 3.0 * PI * r * r * r,
            area: 6.0 * l * l + 6.0 * PI * r * l + 4.0 * PI * r * r,
            faces: 26,
            min: LO,
            max: HI,
            centroid: Some(MID),
        },
    );
}

// ---------------------------------------------------------------------------------------------
// Chamfers

#[test]
fn a_cube_with_one_chamfered_edge() {
    // The vertical edge at (A, A), equal distance d: the section loses a right triangle d²/2
    // along A. The side faces lose a d × A strip each, the bevel adds √2 d × A, the top and
    // bottom lose d²/2 each.
    let d = 2.0;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let e = edges(&k, c, |m| near(m.x, A) && near(m.y, A));
    let b = k
        .chamfer(c, &e, ChamferSpec::EqualDistance(d))
        .unwrap()
        .bodies[0];
    check(
        &k,
        b,
        Want {
            volume: A * A * A - d * d / 2.0 * A,
            area: 6.0 * A * A - 2.0 * d * A + SQRT_2 * d * A - d * d,
            faces: 7,
            min: LO,
            max: HI,
            centroid: None,
        },
    );
}

#[test]
fn a_cube_with_its_top_edges_chamfered() {
    // The four top edges, equal distance d: below z = A − d the cube is whole; above it is a
    // frustum from A × A up to (A − 2d) × (A − 2d) over height d,
    // V = A²(A − d) + d/3 (A² + (A − 2d)² + A(A − 2d)).
    // Faces: the bottom A², four sides A × (A − d), the top (A − 2d)², and four bevels,
    // trapezoids with parallel sides A and A − 2d and slant height √2 d.
    let d = 2.0;
    let t = A - 2.0 * d;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let e = edges(&k, c, |m| near(m.z, A));
    assert_eq!(e.len(), 4);
    let b = k
        .chamfer(c, &e, ChamferSpec::EqualDistance(d))
        .unwrap()
        .bodies[0];
    let volume = A * A * (A - d) + d / 3.0 * (A * A + t * t + A * t);
    // The centroid's height: the base slab's A²(A − d)·(A − d)/2 plus the frustum's, whose
    // centroid sits d (A1 + 2√(A1A2) + 3A2) / (4 (A1 + √(A1A2) + A2)) above its base.
    let (a1, a2) = (A * A, t * t);
    let frustum = d / 3.0 * (a1 + t * A + a2);
    let zf = (A - d) + d * (a1 + 2.0 * A * t + 3.0 * a2) / (4.0 * (a1 + A * t + a2));
    let z = (A * A * (A - d) * (A - d) / 2.0 + frustum * zf) / volume;
    check(
        &k,
        b,
        Want {
            volume,
            area: A * A + 4.0 * A * (A - d) + t * t + 4.0 * (A + t) / 2.0 * SQRT_2 * d,
            faces: 10,
            min: LO,
            max: HI,
            centroid: Some([A / 2.0, A / 2.0, z]),
        },
    );
}

// ---------------------------------------------------------------------------------------------
// Shells

#[test]
fn a_cube_shelled_open_at_the_top() {
    // Inward, wall t, the top removed: the cavity is (A − 2t)² × (A − t), its floor at z = t.
    // Area: the five outer faces, the rim A² − (A − 2t)², the cavity's floor (A − 2t)² and
    // four walls (A − 2t) × (A − t): 6A² + 4(A − 2t)(A − t).
    let t = 1.0;
    let w = A - 2.0 * t;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let top = face(&k, c, [A / 2.0, A / 2.0, A]);
    let b = k.shell(c, &[top], t).unwrap().bodies[0];
    let cavity = w * w * (A - t);
    let volume = A * A * A - cavity;
    let z = (A * A * A * A / 2.0 - cavity * (t + (A - t) / 2.0)) / volume;
    check(
        &k,
        b,
        Want {
            volume,
            area: 6.0 * A * A + 4.0 * w * (A - t),
            faces: 11,
            min: LO,
            max: HI,
            centroid: Some([A / 2.0, A / 2.0, z]),
        },
    );
}

#[test]
fn a_cube_hollowed_inward() {
    // A closed void (A − 2t)³ in the middle: V = A³ − (A − 2t)³, S = 6A² + 6(A − 2t)².
    let t = 1.5;
    let w = A - 2.0 * t;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let b = k
        .shell_with(
            c,
            &ShellSpec {
                remove: vec![],
                thickness: t,
                outward: false,
                hollow: true,
                sharp: false,
            },
        )
        .unwrap()
        .bodies[0];
    check(
        &k,
        b,
        Want {
            volume: A * A * A - w * w * w,
            area: 6.0 * A * A + 6.0 * w * w,
            faces: 12,
            min: LO,
            max: HI,
            centroid: Some(MID),
        },
    );
}

#[test]
fn a_cube_hollowed_outward_with_sharp_corners() {
    // The walls grow out to a cube of side A + 2t: V = (A + 2t)³ − A³,
    // S = 6(A + 2t)² + 6A².
    let t = 1.0;
    let o = A + 2.0 * t;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let b = k
        .shell_with(
            c,
            &ShellSpec {
                remove: vec![],
                thickness: t,
                outward: true,
                hollow: true,
                sharp: true,
            },
        )
        .unwrap()
        .bodies[0];
    check(
        &k,
        b,
        Want {
            volume: o * o * o - A * A * A,
            area: 6.0 * o * o + 6.0 * A * A,
            faces: 12,
            min: [-t; 3],
            max: [A + t; 3],
            centroid: Some(MID),
        },
    );
}

#[test]
fn a_cube_hollowed_outward_with_round_corners() {
    // Rounded joins: the outside is the Minkowski sum of the cube and a ball of radius t,
    // A³ + 6A²t + 3πAt² + 4/3 πt³ with area 6A² + 6πtA + 4πt², less the void A³.
    // Faces: 6 + 12 + 8 outside, 6 inside.
    let t = 1.0;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let b = k
        .shell_with(
            c,
            &ShellSpec {
                remove: vec![],
                thickness: t,
                outward: true,
                hollow: true,
                sharp: false,
            },
        )
        .unwrap()
        .bodies[0];
    check(
        &k,
        b,
        Want {
            volume: 6.0 * A * A * t + 3.0 * PI * A * t * t + 4.0 / 3.0 * PI * t * t * t,
            area: 6.0 * A * A + 6.0 * PI * t * A + 4.0 * PI * t * t + 6.0 * A * A,
            faces: 32,
            min: [-t; 3],
            max: [A + t; 3],
            centroid: Some(MID),
        },
    );
}

// ---------------------------------------------------------------------------------------------
// Combinations

#[test]
fn a_filleted_cube_shelled_open_at_the_top() {
    // The four vertical edges filleted r, then shelled inward t with the top removed: the
    // cavity is the rounded square offset by t (side A − 2t, corners r − t) over A − t.
    // Area: the outer bottom and the top (rim plus cavity floor) are both the outer section;
    // the outer walls are its perimeter × A; the cavity's walls its perimeter × (A − t).
    let (r, t) = (3.0, 1.0);
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let e = edges(&k, c, |m| near(m.z, A / 2.0));
    let f = k.fillet(c, &e, r).unwrap().bodies[0];
    let top = face(&k, f, [A / 2.0, A / 2.0, A]);
    let b = k.shell(f, &[top], t).unwrap().bodies[0];
    let (w, ri) = (A - 2.0 * t, r - t);
    let outer = A * A - (4.0 - PI) * r * r;
    let inner = w * w - (4.0 - PI) * ri * ri;
    let outer_perimeter = 4.0 * (A - 2.0 * r) + 2.0 * PI * r;
    let inner_perimeter = 4.0 * (w - 2.0 * ri) + 2.0 * PI * ri;
    check(
        &k,
        b,
        Want {
            volume: outer * A - inner * (A - t),
            area: 2.0 * outer + outer_perimeter * A + inner_perimeter * (A - t),
            // Outside: bottom, 4 sides, 4 fillets; the rim; inside: floor, 4 walls, 4 fillets.
            faces: 19,
            min: LO,
            max: HI,
            centroid: None,
        },
    );
}

#[test]
fn a_cube_with_every_edge_filleted_shelled_outward_open_at_the_top() {
    // Every edge filleted r makes the cube L = A − 2r grown by a ball r (as above); shelling it
    // outward t grows that same cube by a ball R = r + t (the rounded cube is convex, so its
    // offset is exact). The faces round the removed top are tangent to it, so the walls round
    // over the opening as Onshape's do; only the opening, the top square L × L, goes straight
    // through the new wall, t deep.
    //
    // V = V(R) − V(r) − L² t, with V(q) = L³ + 6L²q + 3πLq² + 4/3 πq³.
    // Area: outside, the grown cube's S(R) = 6L² + 6πRL + 4πR² less the opening L²; inside, the
    // filleted cube's S(r) less its top face L²; the opening's four walls, L × t each.
    // Faces: 25 outside, 25 inside, 4 round the opening.
    let (r, t) = (2.0, 1.0);
    let (l, big) = (A - 2.0 * r, r + t);
    let rounded_volume =
        |q: f64| l * l * l + 6.0 * l * l * q + 3.0 * PI * l * q * q + 4.0 / 3.0 * PI * q * q * q;
    let rounded_area = |q: f64| 6.0 * l * l + 6.0 * PI * q * l + 4.0 * PI * q * q;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let e = edges(&k, c, |_| true);
    let f = k.fillet(c, &e, r).unwrap().bodies[0];
    let top = face(&k, f, [A / 2.0, A / 2.0, A]);
    let b = k
        .shell_with(
            f,
            &ShellSpec {
                remove: vec![top],
                thickness: t,
                outward: true,
                hollow: false,
                sharp: false,
            },
        )
        .unwrap()
        .bodies[0];
    check(
        &k,
        b,
        Want {
            volume: rounded_volume(big) - rounded_volume(r) - l * l * t,
            area: rounded_area(big) - l * l + rounded_area(r) - l * l + 4.0 * l * t,
            faces: 54,
            min: [-t, -t, -t],
            max: [A + t, A + t, A + t],
            centroid: None,
        },
    );
}

#[test]
fn a_cube_with_a_corner_cut_away() {
    // A cube of side s overlapping the corner (A, A, A) removed: V = A³ − s³. The three faces
    // at the corner lose s² each and the cut adds three s² faces, so S = 6A².
    let s = 4.0;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let tool = extrude(
        &mut k,
        rect(A - s, A - s, A + 5.0, A + 5.0),
        vec![],
        A + 5.0,
    );
    let tool = k
        .transform(tool, &Transform::translation(0.0, 0.0, A - s))
        .unwrap()
        .bodies[0];
    let b = k.boolean(BoolOp::Subtract, c, &[tool]).unwrap().bodies[0];
    check(
        &k,
        b,
        Want {
            volume: A * A * A - s * s * s,
            area: 6.0 * A * A,
            faces: 9,
            min: LO,
            max: HI,
            centroid: None,
        },
    );
}

#[test]
fn two_overlapping_cubes_united() {
    // Two cubes A, the second moved (A/2, A/2, A/2): they share a cube (A/2)³, so
    // V = 2A³ − (A/2)³. Each loses three (A/2)² squares inside the other: S = 12A² − 6(A/2)².
    let h = A / 2.0;
    let mut k = NativeKernel::new();
    let c = cube(&mut k);
    let d = cube(&mut k);
    let d = k
        .transform(d, &Transform::translation(h, h, h))
        .unwrap()
        .bodies[0];
    let b = k.boolean(BoolOp::Union, c, &[d]).unwrap().bodies[0];
    check(
        &k,
        b,
        Want {
            volume: 2.0 * A * A * A - h * h * h,
            area: 12.0 * A * A - 6.0 * h * h,
            // Each cube keeps its three far faces whole and three L-shaped near faces.
            faces: 12,
            min: LO,
            max: [A + h; 3],
            centroid: Some([(A + h) / 2.0; 3]),
        },
    );
}

// ---------------------------------------------------------------------------------------------
// Threads

#[test]
fn a_thread_wound_twice_round_a_cylinder_joins_it() {
    // A trapezoid (2.4 high at x = R, 1.4 high at x = R + 1.425) in the XZ plane swept along two
    // turns of a helix of radius R and pitch P (cubic Bézier pieces, sixteen a turn), its inner
    // side on the cylinder of radius R it winds round. The cylinder's face round the thread
    // wraps it twice, so no seam runs from its bottom to its top clear of the thread: the
    // union must keep that face in pieces instead of merging them into an invalid face.
    //
    // The thread only touches the cylinder, so the union's volume is the sum of the two. By
    // Pappus (a screw motion), the thread's is the profile's area 2.7075 × 2π × its centroid's
    // x, 19.025, × 2 turns (the Bézier helix is within 1e-8 of it). The area loses the contact
    // twice: a band 2.4 wide wound twice, 2.4 × 2πR × 2, less its corner below z = 0, where the
    // band's lower edge z = −1.2 + P u / 2π is under the cylinder's bottom (u < u₀ = 2.4π / P):
    // R (1.2 u₀ − P u₀² / 4π).
    use std::f64::consts::TAU;
    let (r, pitch, per) = (18.375, 4.125, 16usize);
    let th = TAU / per as f64;
    let kk = 4.0 / 3.0 * (th / 4.0).tan();
    let pt = |a: f64| Point3::new(r * a.cos(), r * a.sin(), pitch * a / TAU);
    let tan = |a: f64| nalgebra::Vector3::new(-r * a.sin(), r * a.cos(), pitch / TAU);
    let path: Vec<PathCurve> = (0..2 * per)
        .map(|i| {
            let (a0, a1) = (th * i as f64, th * (i + 1) as f64);
            PathCurve::Bezier3 {
                poles: vec![pt(a0), pt(a0) + tan(a0) * kk, pt(a1) - tan(a1) * kk, pt(a1)],
                weights: vec![],
            }
        })
        .collect();
    let front = Plane {
        origin: Point3::origin(),
        x_dir: nalgebra::Vector3::x_axis(),
        normal: -nalgebra::Vector3::y_axis(),
    };
    let pts = [(r, -1.2), (19.8, -0.7), (19.8, 0.7), (r, 1.2)];
    let outer = Loop {
        curves: (0..4)
            .map(|i| Curve2::Line {
                a: Point2::new(pts[i].0, pts[i].1),
                b: Point2::new(pts[(i + 1) % 4].0, pts[(i + 1) % 4].1),
                source: None,
            })
            .collect(),
    };
    let profile = Profile::new(
        front,
        vec![Region {
            outer,
            holes: vec![],
            source: None,
        }],
    );
    let mut k = NativeKernel::new();
    let thread = k
        .sweep_with(
            &profile,
            &SweepSpec {
                body: BodyKind::Solid,
                path,
                control: SweepControl::None,
                faces: vec![],
            },
        )
        .unwrap()
        .bodies[0];
    let cyl = extrude(&mut k, circle(0.0, 0.0, r), vec![], 10.0);
    let (mt, mc) = (
        k.mass_properties(thread).unwrap(),
        k.mass_properties(cyl).unwrap(),
    );
    let pappus = 2.7075 * TAU * 19.025 * 2.0;
    assert!(
        (mt.volume - pappus).abs() < 1e-8 * pappus,
        "thread volume {} against Pappus {pappus}",
        mt.volume
    );
    let u0 = 2.4 * PI / pitch;
    let contact = 2.4 * TAU * r * 2.0 - r * (1.2 * u0 - pitch * u0 * u0 / (2.0 * TAU));
    let b = k.boolean(BoolOp::Union, cyl, &[thread]).unwrap().bodies[0];
    let issues = k.get(b).unwrap().check_with(CheckLevel::Full);
    assert!(issues.is_empty(), "invalid body: {issues:?}");
    let m = k.mass_properties(b).unwrap();
    let (vol, area) = (
        mc.volume + mt.volume,
        mc.surface_area + mt.surface_area - 2.0 * contact,
    );
    assert!(
        (m.volume - vol).abs() < 1e-8 * vol,
        "volume {} against {vol}",
        m.volume
    );
    assert!(
        (m.surface_area - area).abs() < 1e-8 * area,
        "area {} against {area}",
        m.surface_area
    );
}

// ---------------------------------------------------------------------------------------------
// Tangent chains

#[test]
fn a_tangent_chain_stops_where_its_corner_turns_convex() {
    // A plate 20 × 20 × 5 with a boss [0, 10]² × [5, 10] on it and a pocket [0, 10] × [10, 20]
    // × [0, 5] out of it: along x = 10, z = 5 the boss's wall meets the plate's top in a concave
    // edge (y from 0 to 10), and straight on the pocket's wall meets it in a convex one (y from
    // 10 to 20). They are tangent at (10, 10, 5), but a blend filling the one can't run on round
    // the other: the chain is the concave edge alone (Onshape's tangent propagation stops there
    // too). Chamfered 1 along it, the boss gains a prism ½ × 1² × 10 = 5.
    let mut k = NativeKernel::new();
    let plate = extrude(&mut k, rect(0.0, 0.0, 20.0, 20.0), vec![], 5.0);
    let boss = extrude(&mut k, rect(0.0, 0.0, 10.0, 10.0), vec![], 5.0);
    let boss = k
        .transform(boss, &Transform::translation(0.0, 0.0, 5.0))
        .unwrap()
        .bodies[0];
    let pocket = extrude(&mut k, rect(0.0, 10.0, 10.0, 20.0), vec![], 5.0);
    let b = k.boolean(BoolOp::Union, plate, &[boss]).unwrap().bodies[0];
    let b = k.boolean(BoolOp::Subtract, b, &[pocket]).unwrap().bodies[0];
    let v0 = k.mass_properties(b).unwrap().volume;
    assert!(
        (v0 - (2000.0 + 500.0 - 500.0)).abs() < 1e-9 * v0,
        "volume {v0}"
    );
    let on_line = |m: Point3<f64>| near(m.x, 10.0) && near(m.z, 5.0);
    let concave = edges(&k, b, |m| on_line(m) && m.y < 10.0);
    let convex = edges(&k, b, |m| on_line(m) && m.y > 10.0);
    assert_eq!((concave.len(), convex.len()), (1, 1));
    assert_eq!(k.tangent_chain(b, concave[0]).unwrap(), concave);
    let chain = k.tangent_chain(b, concave[0]).unwrap();
    let c = k
        .chamfer(b, &chain, ChamferSpec::EqualDistance(1.0))
        .unwrap()
        .bodies[0];
    let issues = k.get(c).unwrap().check_with(CheckLevel::Full);
    assert!(issues.is_empty(), "invalid body: {issues:?}");
    let v = k.mass_properties(c).unwrap().volume;
    assert!(
        (v - (v0 + 5.0)).abs() < 1e-9 * v,
        "volume {v}, want {}",
        v0 + 5.0
    );
}
