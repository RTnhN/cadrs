//! P3.6 / PS17: the Jackhammer Gear Cover exercise on the stand-in
//! ([`cadrs_core::samples::gear_cover`]), every course step through the document's commands, with
//! the volume checked against closed forms and the fillets against the mesh.
//!
//! The course's steps (intro-to-part-studios.md PS17, gap doc "What each exercise needs"):
//!
//! 1. Shell the bottom face, 4 mm.
//! 2. Sketch the 6 points on the top (here: on the bosses' top face; two concentric with the
//!    bosses' circular edges, four on the flat top, symmetric about the YZ plane).
//! 3. Hole: M5 Close counterbore Ø9.75 × 5, Through all, Start from part.
//! 4. Drag Shell 1 to the bottom.
//! 5. Chamfer 2 mm × 45° (Distance and angle, Offset) on the two bosses' top edges.
//! 6. Fillet R1 with overflow on the bottom rim (three edges picked, their tangent chain).
//! 7. Fillet Width 3 mm on the slope's edge.
//! 8. Mass properties in kg.
#![cfg(feature = "brep")]
#![allow(clippy::field_reassign_with_default)]

use std::f64::consts::PI;

use cadrs_core::applied::{
    ChamferFeature, ChamferType, EdgeOrFace, FilletFeature, FilletMeasurement, HoleFeature,
    HolePoint, ShellFeature,
};
use cadrs_core::commands::{AddFeature, AddSketch, EditSketch, MoveFeatures};
use cadrs_core::document::{Document, EdgeRef, FaceRef};
use cadrs_core::hole::{Fit, HoleEnd, HoleSpec, HoleStart, HoleStyle, HoleType};
use cadrs_core::parts::mass_report;
use cadrs_core::rebuild;
use cadrs_core::samples::gear_cover as gc;
use cadrs_core::{ElementId, Feature, FeatureId, History, Part};
use cadrs_sketch::SketchOp;

#[track_caller]
fn close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "got {a}, expected {b} ± {tol}");
}

struct Studio {
    d: Document,
    h: History,
    el: ElementId,
}

impl Studio {
    fn new() -> Self {
        let d = gc::document().expect("the stand-in builds");
        let el = d.elements[0].id;
        Self {
            d,
            h: History::default(),
            el,
        }
    }

    fn features(&self) -> Vec<Feature> {
        self.d.element(self.el).unwrap().features().to_vec()
    }

    fn part(&self) -> Part {
        let b = rebuild::build(&self.features());
        assert!(b.errors.is_empty(), "rebuild errors: {:?}", b.errors);
        assert_eq!(b.parts.len(), 1);
        b.parts[0].clone()
    }

    fn volume(&self) -> f64 {
        self.part().mass.unwrap().volume
    }

    fn add(&mut self, kind: AddFeature) -> FeatureId {
        let id = kind.feature;
        self.h.execute(&mut self.d, &kind).unwrap();
        id
    }
}

/// A face of the part whose plane faces `n` and contains `p`.
fn face_ref(part: &Part, n: [f64; 3], p: [f64; 3]) -> FaceRef {
    let s = &part.solid;
    let i = s
        .faces
        .iter()
        .enumerate()
        .position(|(i, f)| {
            f.plane.is_some_and(|pl| {
                let m = pl.normal();
                let len = (m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).sqrt();
                (m[0] * n[0] + m[1] * n[1] + m[2] * n[2]) / len > 0.999
            }) && s.face_contains(i, p)
        })
        .expect("a face there");
    FaceRef {
        part: part.id,
        face: s.faces[i].name,
        seed: p,
    }
}

/// The edge of the part passing nearest `p`.
fn edge_ref(part: &Part, p: [f64; 3]) -> EdgeRef {
    let e = part
        .solid
        .edges
        .iter()
        .min_by(|a, b| a.distance(p).total_cmp(&b.distance(p)))
        .unwrap();
    assert!(e.distance(p) < 1e-3, "no edge at {p:?}");
    EdgeRef {
        part: part.id,
        edge: e.name,
        seed: p,
    }
}

// ---------------------------------------------------------------------------------------------
// The expected volumes (mm³), worked out by hand from the stand-in's dimensions (the
// `gear_cover` constants) and the sizes the course's features use (below): no kernel is
// involved. The test rebuilds the part with the kernel after each step and checks the volume
// the kernel measures (`Studio::volume`, the part's mass properties) against these.
//
// The cover: x −HALF_WIDTH..HALF_WIDTH, y −LENGTH..0, z 0..HEIGHT, its plan's corners rounded
// CORNER_R; the slope z = SLOPE_Z0 + SLOPE_K·(y + LENGTH) for y < SLOPE_END; two bosses of
// radius BOSS_R, BOSS_H high on the top; six M5 Close holes (Ø 2·RH) through all with a Ø 2·RC
// × CBORE counterbore, two on the bosses and four on the flat top; a shell of T with the
// bottom open; a CHAMFER × 45° chamfer round each boss's top edge.

/// Shell 1's thickness.
const T: f64 = 4.0;
/// The M5 Close hole's radius and its counterbore's radius and depth (the hole table's).
const RH: f64 = 2.65;
const RC: f64 = 4.875;
const CBORE: f64 = 5.0;
/// The bosses' chamfer.
const CHAMFER: f64 = 2.0;
/// The holes on the bosses and on the flat top.
const BOSS_HOLES: f64 = 2.0;
const TOP_HOLES: f64 = 4.0;

/// The rounded corners' centres sit CORNER_R in from the sides: x ±(HALF_WIDTH − CORNER_R),
/// y −LENGTH + CORNER_R (front) and −CORNER_R (back).
fn corner_x() -> f64 {
    gc::HALF_WIDTH - gc::CORNER_R
}

/// The slope's run from the front corners' centre line to where it meets the top.
fn slope_run() -> f64 {
    gc::SLOPE_END + gc::LENGTH - gc::CORNER_R
}

/// The body before any applied feature: the rounded prism, less the slope's wedge, plus the
/// bosses. With R = CORNER_R, K = SLOPE_K, c = corner_x() and D = slope_run():
///   prism: HEIGHT·(2·HALF_WIDTH·LENGTH − (4 − π)R²);
///   wedge: ∫ w(y)·(HEIGHT − z_s(y)) dy over the slope, where HEIGHT − z_s = K·(SLOPE_END − y).
///     Between the front corners and SLOPE_END the width is 2·HALF_WIDTH: HALF_WIDTH·K·D². In
///     the front corners (u = y + LENGTH − R ∈ [−R, 0], w = 2(c + √(R² − u²)), HEIGHT − z_s =
///     K(D − u)): 2K [c·(D·R + R²/2) + D·πR²/4 + R³/3];
///   bosses: 2·π·BOSS_R²·BOSS_H.
fn expected_body_volume() -> f64 {
    let (r, k, c, d) = (gc::CORNER_R, gc::SLOPE_K, corner_x(), slope_run());
    let prism = gc::HEIGHT * (2.0 * gc::HALF_WIDTH * gc::LENGTH - (4.0 - PI) * r * r);
    let wedge = gc::HALF_WIDTH * k * d * d
        + 2.0 * k * (c * (d * r + r * r / 2.0) + d * PI * r * r / 4.0 + r.powi(3) / 3.0);
    let bosses = 2.0 * PI * gc::BOSS_R * gc::BOSS_R * gc::BOSS_H;
    prism - wedge + bosses
}

/// What the counterbored holes, through all, remove from the body: each the counterbore
/// πRC²·CBORE and the hole under it πRH²·(depth − CBORE), from the bosses' top (HEIGHT +
/// BOSS_H) or the flat top (HEIGHT).
fn expected_holes_volume() -> f64 {
    let hole = |depth: f64| PI * RC * RC * CBORE + PI * RH * RH * (depth - CBORE);
    BOSS_HOLES * hole(gc::HEIGHT + gc::BOSS_H) + TOP_HOLES * hole(gc::HEIGHT)
}

/// The shell's cavity when it comes after the holes (the course's order after step 4): every
/// face moved T into the body and extended until they meet (the bottom is open). The cavity's
/// corners are sharp, under a concave corner of the body (a counterbore's floor meeting its
/// wall, a boss meeting the top) as under a convex one, as Onshape shells (issue #105).
///
/// Away from the bosses and holes it is the cover moved in by T: half-width HALF_WIDTH − T,
/// y −LENGTH + T..−T, corners of radius r = CORNER_R − T on the same centres, up to z
/// HEIGHT − T and under the slope moved T along its normal (0, −K, 1)/√(1 + K²). In u as above
/// that is z = a + K·u, a = SLOPE_Z0 + K·CORNER_R − T√(1 + K²), meeting z HEIGHT − T at
/// u* = (HEIGHT − T − a)/K:
///   front corners (u ∈ [−r, 0]): 2[c·(a·r − K·r²/2) + a·πr²/4 − K·r³/3];
///   [0, u*]: w·(a·u* + K·u*²/2), w = 2(HALF_WIDTH − T);
///   [u*, LENGTH − 2·CORNER_R] (the back corners' centre line): w·(HEIGHT − T)·(… − u*);
///   back corners: (HEIGHT − T)·2(c·r + πr²/4).
/// Round each hole and boss it is a solid of revolution about the axis, whose top z(r), from
/// the hole's wall moved out (r = RH + T), replaces the HEIGHT − T above:
///   boss: r ∈ [RH + T, RC + T] the counterbore's floor less T; [RC + T, BOSS_R − T] the boss
///     top less T; [BOSS_R − T, BOSS_R] the top less T, which is HEIGHT − T anyway;
///   flat hole: r ∈ [RH + T, RC + T] its counterbore's floor less T.
fn expected_cavity_volume() -> f64 {
    let (k, c) = (gc::SLOPE_K, corner_x());
    let r = gc::CORNER_R - T;
    let top = gc::HEIGHT - T;
    let w = 2.0 * (gc::HALF_WIDTH - T);
    let a = gc::SLOPE_Z0 + k * gc::CORNER_R - T * (1.0 + k * k).sqrt();
    let us = (top - a) / k;
    let back_line = gc::LENGTH - 2.0 * gc::CORNER_R;
    let front = 2.0 * (c * (a * r - k * r * r / 2.0) + a * PI * r * r / 4.0 - k * r.powi(3) / 3.0);
    let slope = w * (a * us + k * us * us / 2.0);
    let flat = w * top * (back_line - us);
    let back = top * 2.0 * (c * r + PI * r * r / 4.0);
    let main = front + slope + flat + back;
    // A ring of the cavity, r ∈ [r0, r1], up to z.
    let annulus = |z: f64, r0: f64, r1: f64| z * PI * (r1 * r1 - r0 * r0);
    let (wall, cbore) = (RH + T, RC + T);
    let boss_top = gc::HEIGHT + gc::BOSS_H;
    let boss = annulus(boss_top - CBORE - T, wall, cbore)
        + annulus(boss_top - T, cbore, gc::BOSS_R - T)
        - annulus(top, 0.0, gc::BOSS_R - T);
    let hole = annulus(gc::HEIGHT - CBORE - T, wall, cbore) - annulus(top, 0.0, cbore);
    main + BOSS_HOLES * boss + TOP_HOLES * hole
}

/// A CHAMFER × 45° chamfer round a boss's top edge removes a right triangle of area
/// CHAMFER²/2 whose centroid is CHAMFER/3 in from the edge (Pappus): 2π(BOSS_R − CHAMFER/3)
/// times that, on each boss.
fn expected_chamfer_volume() -> f64 {
    2.0 * 2.0 * PI * (gc::BOSS_R - CHAMFER / 3.0) * CHAMFER * CHAMFER / 2.0
}

#[test]
fn closed_forms_hold_on_their_own() {
    // The cavity's plain parts: with no holes or bosses the cavity would be the offset cover.
    assert!(expected_cavity_volume() > 0.0 && expected_body_volume() > expected_cavity_volume());
    // A sanity figure for the report: the finished cover is about 2.1e5 mm³.
    let v = expected_body_volume()
        - expected_holes_volume()
        - expected_cavity_volume()
        - expected_chamfer_volume();
    assert!(v > 1.5e5 && v < 3.0e5, "{v}");
}

#[test]
fn gear_cover_course_steps() {
    let mut s = Studio::new();
    // The stand-in as it comes: one part, "Gear Cover", Aluminum - 380, its six features in the
    // closed "Base Features" folder.
    let el = s.d.element(s.el).unwrap();
    assert_eq!(el.folders().len(), 1);
    assert_eq!(el.folders()[0].name, "Base Features");
    assert_eq!(el.folders()[0].features.len(), 6);
    assert!(!el.folders()[0].open);
    close(s.volume(), expected_body_volume(), 1e-4);
    let part = s.part();

    // Step 1 (PS17.3): Shell 1, the bottom face (Face of Extrude 1), 4 mm.
    let bottom = face_ref(&part, [0.0, 0.0, -1.0], [0.0, -100.0, 0.0]);
    assert_eq!(bottom.face.op, gc::EXTRUDE_1.0);
    let shell = s.add(AddFeature::shell(
        s.el,
        FeatureId::new(),
        ShellFeature {
            faces: vec![bottom],
            thickness: T,
            thickness_expr: format!("{T} mm"),
            ..ShellFeature::default()
        },
    ));
    let shelled = s.volume();
    assert!(shelled < expected_body_volume() / 2.0, "{shelled}");

    // Step 2 (PS17.4): Sketch 4 on the bosses' top face: the two points concentric with the
    // bosses' edges, and four on the flat top, symmetric about the YZ plane.
    let part = s.part();
    let boss_top = face_ref(&part, [0.0, 0.0, 1.0], [gc::BOSS_X, gc::BOSS_Y, 46.0]);
    let plane = cadrs_core::parts::face_plane(&s.features(), gc::EXTRUDE_3, boss_top.face)
        .expect("a planar face");
    let frame = plane.frame();
    let sketch = FeatureId::new();
    s.h.execute(
        &mut s.d,
        &AddSketch {
            element: s.el,
            feature: sketch,
            plane: Some(plane),
        },
    )
    .unwrap();
    let pts = [
        [-gc::BOSS_X, gc::BOSS_Y],
        [gc::BOSS_X, gc::BOSS_Y],
        [-gc::HOLE_X, gc::HOLE_Y],
        [gc::HOLE_X, gc::HOLE_Y],
        [-gc::TOP_X, gc::TOP_Y],
        [gc::TOP_X, gc::TOP_Y],
    ];
    // The six points: the bosses' centres; (±34, −100) and (±16, −16) on the flat top (each
    // hole's neighbourhood, r ≤ r_c + t, stays clear of the walls' and the slope's offsets, so
    // the closed form's pieces don't meet).
    let ops: Vec<SketchOp> = pts
        .iter()
        .map(|p| SketchOp::AddPoint {
            pos: frame.to_sketch([p[0], p[1], 46.0]),
        })
        .collect();
    s.h.execute(
        &mut s.d,
        &EditSketch {
            element: s.el,
            feature: sketch,
            op: SketchOp::Batch(ops),
        },
    )
    .unwrap();
    let g =
        s.d.element(s.el)
            .unwrap()
            .feature(sketch)
            .unwrap()
            .sketch()
            .unwrap()
            .geometry
            .clone();
    let points: Vec<HolePoint> = g
        .points
        .keys()
        .map(|point| HolePoint { sketch, point })
        .collect();
    assert_eq!(points.len(), 6);

    // Step 3 (PS17.5): the hole, Metric, Counterbore, Clearance M5 Close → Ø5.3, Start from
    // part, Through all, counterbore Ø9.75 × 5. Its name is the callout (PS15.10).
    let mut spec = HoleSpec::default();
    spec.style = HoleStyle::Counterbore;
    spec.hole_type = HoleType::Clearance;
    spec.size = "M5".into();
    spec.fit = Fit::Close;
    spec.start = HoleStart::Part;
    spec.end = HoleEnd::ThroughAll;
    spec.apply_table();
    assert_eq!(
        (
            spec.diameter.value,
            spec.cbore_diameter.value,
            spec.cbore_depth.value
        ),
        (2.0 * RH, 2.0 * RC, CBORE)
    );
    let hole = s.add(AddFeature::hole(
        s.el,
        FeatureId::new(),
        HoleFeature {
            points,
            merge_scope: vec![gc::PART],
            spec,
            ..HoleFeature::default()
        },
    ));
    let hole_name =
        s.d.element(s.el)
            .unwrap()
            .feature(hole)
            .unwrap()
            .name
            .clone();
    assert_eq!(hole_name, "Ø 5.3 mm THRU | ⌴Ø 9.75 mm ↧ 5 mm");
    // With the shell first, the counterbores are cut through 4 mm walls ("most holes look like
    // plain holes"): less is removed than from the solid body.
    let shell_first = s.volume();
    assert!(shell_first < shelled);

    // Step 4 (PS17.6): drag Shell 1 to the bottom of the list: now the shell wraps the holes.
    let n = s.features().len();
    s.h.execute(
        &mut s.d,
        &MoveFeatures {
            element: s.el,
            features: vec![shell],
            to: n - 1,
            folder: None,
            label: "Reorder Shell 1".into(),
        },
    )
    .unwrap();
    assert_eq!(s.features().last().unwrap().id, shell);
    // The closed form: the holed body less the cavity.
    let v_shell = expected_body_volume() - expected_holes_volume() - expected_cavity_volume();
    close(s.volume(), v_shell, 1e-3);

    // Step 5 (PS17.7): Chamfer 2 mm × 45°, Distance and angle, Offset, on the two bosses' top
    // edges (tangent propagation on: a circle is its own chain).
    let part = s.part();
    let e1 = edge_ref(&part, [-gc::BOSS_X + 15.0, gc::BOSS_Y, 46.0]);
    let e2 = edge_ref(&part, [gc::BOSS_X + 15.0, gc::BOSS_Y, 46.0]);
    s.add(AddFeature::chamfer(
        s.el,
        FeatureId::new(),
        ChamferFeature {
            entities: vec![EdgeOrFace::Edge(e1), EdgeOrFace::Edge(e2)],
            kind: ChamferType::DistanceAngle,
            distance: CHAMFER,
            distance_expr: format!("{CHAMFER} mm"),
            angle: 45.0,
            angle_expr: "45 deg".into(),
            ..ChamferFeature::default()
        },
    ));
    // Before the fillets: the closed form.
    let v_before_fillets = v_shell - expected_chamfer_volume();
    let kernel_before_fillets = s.volume();
    close(kernel_before_fillets, v_before_fillets, 1e-3);
    let before = s.part();
    // CoM X = 0 by symmetry.
    close(before.mass.unwrap().center_of_mass.x, 0.0, 1e-6);

    // Step 6 (PS17.8): Fillet R1, overflow on, three bottom rim edges (their tangent chain is
    // the whole outer rim).
    let rim = [
        [0.0, -200.0, 0.0],
        [55.0, -100.0, 0.0],
        [-55.0, -100.0, 0.0],
    ];
    let entities = rim
        .iter()
        .map(|p| EdgeOrFace::Edge(edge_ref(&before, *p)))
        .collect();
    s.add(AddFeature::fillet(
        s.el,
        FeatureId::new(),
        FilletFeature {
            entities,
            size: 1.0,
            size_expr: "1 mm".into(),
            ..FilletFeature::default()
        },
    ));
    let v6 = s.volume();
    // Removed along the rim's length: (1 − π/4)·1² per mm (all the rim's corners are 90°), the
    // rim being the rounded rectangle's perimeter 2(110 + 200) − (8 − 2π)·20.
    let perimeter = 2.0 * (110.0 + 200.0) - (8.0 - 2.0 * PI) * 20.0;
    close(v_before_fillets - v6, (1.0 - PI / 4.0) * perimeter, 0.5);

    // Step 7 (PS17.9): Fillet 3 on the slope's edge (Edge of Extrude 2): Width 3 mm.
    let part = s.part();
    let slope_edge = edge_ref(&part, [55.0, -140.0, 24.0 + 0.2 * 60.0]);
    assert!(
        slope_edge
            .edge
            .faces
            .iter()
            .any(|f| f.op == gc::EXTRUDE_2.0),
        "an edge of Extrude 2"
    );
    s.add(AddFeature::fillet(
        s.el,
        FeatureId::new(),
        FilletFeature {
            entities: vec![EdgeOrFace::Edge(slope_edge)],
            measurement: FilletMeasurement::Width,
            size: 3.0,
            size_expr: "3 mm".into(),
            ..FilletFeature::default()
        },
    ));
    let done = s.part();
    let m = done.mass.unwrap();
    // The Width fillet's removal, bounded analytically. Its tangent chain is the slope's whole
    // rim: the two side edges (faces 90° apart), the front edge (78.69°: cos φ = 0.2/√1.04) and
    // the two R20 corners between (φ between those). Between flat faces whose normals are φ
    // apart, width w gives r = w/(2 sin(φ/2)) and removes A(φ) = r²(tan(φ/2) − φ/2) per mm.
    let removed = v6 - m.volume;
    let area = |phi: f64| {
        let r = 3.0 / (2.0 * (phi / 2.0).sin());
        (r * r * ((phi / 2.0).tan() - phi / 2.0), r)
    };
    let (a90, _) = area(PI / 2.0);
    let (a_front, r_front) = area((0.2 / 1.04f64.sqrt()).acos());
    let side = 60.0 * 1.04f64.sqrt();
    let front = 2.0 * (gc::HALF_WIDTH - gc::CORNER_R);
    // A corner: x = 35 + 20 cos a, y = −180 + 20 sin a, z = 24 + 0.2(y + 200), a in [−π/2, 0]:
    // |d/da| = √(400 + 16 cos²a) (Simpson).
    let n = 1000;
    let corner = (0..=n)
        .map(|i| {
            let a = -PI / 2.0 + PI / 2.0 * i as f64 / n as f64;
            let wgt = if i == 0 || i == n {
                1.0
            } else if i % 2 == 1 {
                4.0
            } else {
                2.0
            };
            wgt * (400.0 + 16.0 * a.cos().powi(2)).sqrt()
        })
        .sum::<f64>()
        * (PI / 2.0 / n as f64)
        / 3.0;
    // Round a convex corner of radius R the section's centroid (within r of the edge) runs
    // on a shorter path: at least (1 − r/R) of the edge (Pappus). The two ends, where the
    // fillet stops at the slope's crease, change it by at most A·r each.
    let straight = 2.0 * side * a90 + front * a_front;
    let low =
        straight + 2.0 * corner * a_front * (1.0 - r_front / gc::CORNER_R) - 2.0 * a90 * r_front;
    let high = straight + 2.0 * corner * a90 + 2.0 * a90 * r_front;
    assert!(
        low < removed && removed < high,
        "the Width fillet removed {removed}, expected {low}..{high}"
    );
    // And the kernel's exact volume against the display mesh's (0.05 mm deflection): the mesh's
    // chords cut into the curved faces, so it is a little smaller, by under 1.5e-4.
    let mesh = done.solid.volume();
    assert!(
        (m.volume - mesh).abs() < 1.5e-4 * m.volume,
        "kernel {} vs mesh {mesh}",
        m.volume
    );
    close(m.center_of_mass.x, 0.0, 1e-6);

    // Step 8 (PS17.10): Mass properties in kg (Aluminum - 380, 2760 kg/m³).
    let props = s.d.element(s.el).unwrap().part_props().to_vec();
    let r = mass_report(&[&done], &props).unwrap();
    let mass = r.mass.expect("a material").mass;
    close(mass, m.volume * 2.76e-6, 1e-12);
    println!(
        "gear cover stand-in: V before fillets {:.3} mm³ (closed form {:.3}), after {:.3} mm³ (mesh {:.3}), \
         width fillet removed {removed:.3} ({low:.3}..{high:.3}), mass {:.4} kg",
        kernel_before_fillets, v_before_fillets, m.volume, mesh, mass
    );
    // The golden value: the finished stand-in.
    close(mass, GOLDEN_MASS_KG, 5e-4);
}

/// The finished stand-in's mass (kg), recorded from the kernel and cross-checked against the
/// mesh (again with the shell's sharp cavity corners, issue #105: 1 490 mm³ more, as the closed
/// form has it).
const GOLDEN_MASS_KG: f64 = 0.525_171;

#[test]
fn the_fixture_is_current() {
    // `fixtures/gear_cover_standin.cadrs` is the stand-in document as `gc::document()` builds
    // it. Regenerate it with `CADRS_WRITE_FIXTURES=1 cargo test -p cadrs_core --test gear_cover`.
    let doc = gc::document().unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gear_cover_standin.cadrs");
    let file = gc::file(doc.clone());
    let text = ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default()).unwrap();
    if std::env::var("CADRS_WRITE_FIXTURES").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &text).unwrap();
    }
    let stored = cadrs_core::Store::load_path(&path).expect("the fixture loads");
    assert_eq!(
        stored.document, doc,
        "fixtures/gear_cover_standin.cadrs is out of date"
    );
}

#[test]
fn a_feature_dragged_above_its_parent_fails() {
    // PS11.3: Extrude 1 dragged above Sketch 1 fails (its sketch is below it), and the parts
    // after it keep going without it; dragged back, it builds again.
    let mut s = Studio::new();
    let features = s.features();
    let i = features.iter().position(|f| f.id == gc::EXTRUDE_1).unwrap();
    assert_eq!(i, 1);
    s.h.execute(
        &mut s.d,
        &MoveFeatures {
            element: s.el,
            features: vec![gc::EXTRUDE_1],
            to: 0,
            folder: Some(gc::FOLDER),
            label: "Reorder".into(),
        },
    )
    .unwrap();
    let b = rebuild::build(&s.features());
    let why = b.error(gc::EXTRUDE_1).expect("Extrude 1 fails");
    assert!(why.contains("Sketch 1 is below this feature"), "{why}");
    // The folder still holds its six features, in their new order.
    assert_eq!(
        s.d.element(s.el).unwrap().folders()[0].features[0],
        gc::EXTRUDE_1
    );
    s.h.undo(&mut s.d).unwrap();
    assert!(rebuild::build(&s.features()).errors.is_empty());
}
