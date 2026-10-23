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
    // offset is exact), cut where the removed top face was: the plane z = A, r above the small
    // cube's top.
    //
    // Above height s over the small cube's top, the grown cube's section is the square L with
    // corners ρ = √(R² − s²): L² + 4Lρ + πρ². The cap above z = A (s from r to R) is therefore
    //   L² t + 4L ∫ √(R² − s²) ds + π ∫ (R² − s²) ds,
    // with ∫ᵣᴿ √(R² − s²) ds = πR²/4 − (r √(R² − r²) + R² asin(r/R)) / 2 and
    // ∫ᵣᴿ (R² − s²) ds = R² t − (R³ − r³) / 3.
    //
    // Area: outside, the grown cube's 6L² + 6πRL + 4πR² less its cap (the top square L², the
    // four top edges' cylinders above φ = asin(r/R): 4RL(π/2 − asin(r/R)), the four top corners'
    // spheres above height r: a zone 2πR t); the rim at z = A, the section there less the
    // opening L²: 4Lρ₀ + πρ₀² with ρ₀ = √(R² − r²); inside, the filleted cube less its top face.
    // Faces: 25 outside, the rim, 25 inside.
    let (r, t) = (2.0, 1.0);
    let (l, big) = (A - 2.0 * r, r + t);
    let rounded_volume =
        |q: f64| l * l * l + 6.0 * l * l * q + 3.0 * PI * l * q * q + 4.0 / 3.0 * PI * q * q * q;
    let rounded_area = |q: f64| 6.0 * l * l + 6.0 * PI * q * l + 4.0 * PI * q * q;
    let phi = (r / big).asin();
    let rho0 = (big * big - r * r).sqrt();
    let root = PI * big * big / 4.0 - (r * rho0 + big * big * phi) / 2.0;
    let square = big * big * t - (big.powi(3) - r.powi(3)) / 3.0;
    let cap = l * l * t + 4.0 * l * root + PI * square;
    let cap_area = l * l + 4.0 * big * l * (PI / 2.0 - phi) + 2.0 * PI * big * t;
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
            volume: rounded_volume(big) - cap - rounded_volume(r),
            area: rounded_area(big) - cap_area
                + (4.0 * l * rho0 + PI * rho0 * rho0)
                + (rounded_area(r) - l * l),
            faces: 51,
            min: [-t, -t, -t],
            max: [A + t, A + t, A],
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
