//! References that must survive another kernel's topology (kernel-10's native kernel against
//! documents built with OCCT): a mate connector's edge found where it was when its name is
//! gone, and the merged-away face names of a part carried over to its copies.

use cadrs_core::Document;
use cadrs_core::command::History;
use cadrs_core::commands::{AddExtrude, AddFeature, AddSketch, EditSketch};
use cadrs_core::document::{EdgeRef, ExtrudeFeature, FeatureKind};
use cadrs_core::ids::{ElementId, FeatureId, PartId};
use cadrs_core::mate::{self, ConnectorOrigin};
use cadrs_core::transform::{TransformFeature, TransformType};
use cadrs_kernel::naming::{EdgeName, FaceName, FaceOrigin};
use cadrs_sketch::{PlaneRef, SketchOp, Vec2};

/// How close a frame found where its edge was is to that place (mm).
const AT: f64 = 1e-9;

fn fid(n: u128) -> FeatureId {
    FeatureId::from_u128(0x5e21_0000 + n)
}

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> SketchOp {
    let pts = vec![
        Vec2::new(x0, y0),
        Vec2::new(x1, y0),
        Vec2::new(x1, y1),
        Vec2::new(x0, y1),
    ];
    SketchOp::AddPolyline {
        points: pts,
        closed: true,
        construction: false,
        label: "Add rectangle",
    }
}

/// A Part Studio with one sketch of `rects` extruded 10 (one part); returns it and the part.
fn studio(rects: &[[f64; 4]]) -> (Document, History, ElementId, PartId) {
    let mut doc = Document::empty("robust");
    doc.elements
        .push(cadrs_core::document::Element::part_studio("Part Studio 1"));
    let el = doc.elements[0].id;
    let mut h = History::default();
    let (sk, ex) = (fid(1), fid(2));
    h.execute(
        &mut doc,
        &AddSketch {
            element: el,
            feature: sk,
            plane: Some(PlaneRef::Top),
        },
    )
    .unwrap();
    for &[x0, y0, x1, y1] in rects {
        h.execute(
            &mut doc,
            &EditSketch {
                element: el,
                feature: sk,
                op: rect(x0, y0, x1, y1),
            },
        )
        .unwrap();
    }
    let e = ExtrudeFeature {
        sketches: vec![sk],
        depth: 10.0,
        depth_expr: "10 mm".into(),
        ..ExtrudeFeature::default()
    };
    h.execute(
        &mut doc,
        &AddExtrude {
            element: el,
            feature: ex,
            extrude: e,
        },
    )
    .unwrap();
    (doc, h, el, PartId::new(ex, 0))
}

fn build(doc: &Document, el: ElementId) -> std::sync::Arc<cadrs_core::rebuild::Build> {
    cadrs_core::rebuild::build(&doc.element(el).unwrap().active_features())
}

/// alu_extrusion_parts, cable organizer, desk_current_sensor_gadget: a mate connector on an
/// edge whose stored name no longer exists (the kernel named the faces along it differently)
/// is found where it was, not lost.
#[test]
fn a_mate_connector_edge_is_found_where_it_was() {
    let (doc, _, el, part) = studio(&[[0.0, 0.0, 20.0, 10.0]]);
    let b = build(&doc, el);
    assert!(b.errors.is_empty(), "{:?}", b.errors);
    let solid = &b.part(part).unwrap().solid;
    let e = solid.edges.iter().find(|e| e.circle.is_none()).unwrap();
    let seed = e.midpoint();
    // Its faces under names no face has.
    let gone = |c: u64| {
        FaceName::new(
            fid(2).0,
            FaceOrigin::Side {
                region: 77,
                curve: c,
            },
        )
    };
    let r = EdgeRef {
        part,
        edge: EdgeName::new(gone(1), gone(2), 0),
        seed,
    };
    let features = doc.element(el).unwrap().active_features();
    let f = mate::origin_frame(&ConnectorOrigin::Edge(r), &features, &b.parts)
        .expect("found where it was");
    for k in 0..3 {
        assert!(
            (f.origin[k] - seed[k]).abs() < AT,
            "{:?} vs {seed:?}",
            f.origin
        );
    }
}

/// desk_current_sensor_gadget, t20 esp32 mount: a reference to a copy of a face the kernel
/// merged into another (its name an alias of the merged face's) finds the copy of the merged
/// face: the copies carry their original's aliases.
#[test]
fn a_copy_keeps_its_originals_merged_away_names() {
    // Two touching rectangles: their regions' caps and the sides in line merge.
    let (mut doc, mut h, el, part) = studio(&[[0.0, 0.0, 10.0, 10.0], [10.0, 0.0, 20.0, 10.0]]);
    let t = TransformFeature {
        parts: vec![part],
        ..TransformFeature::new(TransformType::CopyInPlace)
    };
    h.execute(
        &mut doc,
        &AddFeature {
            element: el,
            feature: fid(3),
            base_name: "Transform".into(),
            kind: FeatureKind::Transform(t),
        },
    )
    .unwrap();
    let b = build(&doc, el);
    assert!(b.errors.is_empty(), "{:?}", b.errors);
    let original = b.part(part).unwrap();
    assert!(
        !original.solid.face_aliases.is_empty(),
        "the touching regions merge"
    );
    let copy = b.parts.iter().find(|p| p.id != part).unwrap();
    assert_eq!(
        copy.solid.face_aliases.len(),
        original.solid.face_aliases.len()
    );
    for a in &copy.solid.face_aliases {
        let (i, _) = copy
            .solid
            .resolve_face(&a.name, None, None)
            .expect("the alias resolves");
        assert_eq!(copy.solid.faces[i].name, a.face);
    }
}
