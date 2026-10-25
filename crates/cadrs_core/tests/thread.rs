//! The Thread feature (`cadrs_core::thread`) through the document's commands: a thread cut
//! into a shaft and into a hole, with volumes from the kernel.
#![cfg(feature = "brep")]

use cadrs_core::commands::{AddExtrude, AddFeature, AddSketch, EditSketch, SetExtrude};
use cadrs_core::document::{Document, EdgeRef, ExtrudeFeature, FeatureKind, RegionRef};
use cadrs_core::thread::{ThreadEnd, ThreadFeature};
use cadrs_core::{ElementId, Feature, FeatureId, History, Part, rebuild};
use cadrs_sketch::{PlaneRef, Sketch, SketchOp, Vec2};

struct Studio {
    d: Document,
    h: History,
    el: ElementId,
}

impl Studio {
    fn new() -> Self {
        let d = Document::new("Surfacing");
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

    fn g(&self, f: FeatureId) -> Sketch {
        self.d
            .element(self.el)
            .unwrap()
            .feature(f)
            .unwrap()
            .sketch()
            .unwrap()
            .geometry
            .clone()
    }

    fn sketch(&mut self, plane: PlaneRef, ops: Vec<SketchOp>) -> FeatureId {
        let f = FeatureId::new();
        self.h
            .execute(
                &mut self.d,
                &AddSketch {
                    element: self.el,
                    feature: f,
                    plane: Some(plane),
                },
            )
            .unwrap();
        for op in ops {
            self.h
                .execute(
                    &mut self.d,
                    &EditSketch {
                        element: self.el,
                        feature: f,
                        op,
                    },
                )
                .unwrap();
        }
        f
    }

    fn extrude(&mut self, e: ExtrudeFeature) -> FeatureId {
        let f = FeatureId::new();
        self.h
            .execute(
                &mut self.d,
                &AddExtrude {
                    element: self.el,
                    feature: f,
                    extrude: ExtrudeFeature::default(),
                },
            )
            .unwrap();
        self.h
            .execute(
                &mut self.d,
                &SetExtrude {
                    element: self.el,
                    feature: f,
                    extrude: e,
                    label: "Extrude".into(),
                },
            )
            .unwrap();
        f
    }

    fn add(&mut self, base: &str, kind: FeatureKind) -> FeatureId {
        let feature = FeatureId::new();
        self.h
            .execute(
                &mut self.d,
                &AddFeature {
                    element: self.el,
                    feature,
                    base_name: base.into(),
                    kind,
                },
            )
            .unwrap();
        feature
    }

    fn build(&self) -> std::sync::Arc<rebuild::Build> {
        let b = rebuild::build(&self.features());
        assert!(b.errors.is_empty(), "rebuild errors: {:?}", b.errors);
        b
    }

    fn parts(&self) -> Vec<Part> {
        self.build().parts.clone()
    }
}

fn volume(p: &Part) -> f64 {
    p.mass.as_ref().expect("kernel mass").volume
}

fn close(a: f64, b: f64, rel: f64) -> bool {
    (a - b).abs() <= rel * b.abs().max(1e-9)
}

/// The edge of `part` passing nearest `p`.
fn edge_near(part: &Part, p: [f64; 3]) -> EdgeRef {
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

fn circle(r: f64) -> SketchOp {
    SketchOp::AddCircle {
        center: Vec2::new(0.0, 0.0),
        radius: r,
        construction: false,
    }
}

/// The thread's groove per mm of thread (its section within the cylinder, revolved, per pitch):
/// an M10x1.5 cut into Ø10 takes ~13.3 mm³/mm.
const M10_PER_MM: f64 = 13.3;

#[test]
fn an_external_thread_cuts_its_groove_into_a_shaft() {
    let mut st = Studio::new();
    let s = st.sketch(PlaneRef::Top, vec![circle(5.0)]);
    st.extrude(ExtrudeFeature {
        sketches: vec![s],
        depth: 20.0,
        depth_expr: "20 mm".into(),
        ..Default::default()
    });
    let shaft = st.parts()[0].clone();
    let before = volume(&shaft);
    let t = std::time::Instant::now();
    st.add(
        "External thread",
        FeatureKind::Thread(ThreadFeature {
            edge: Some(edge_near(&shaft, [5.0, 0.0, 20.0])),
            length: 12.0,
            length_expr: "12 mm".into(),
            ..Default::default()
        }),
    );
    let b = st.build();
    eprintln!("thread built in {:?}", t.elapsed());
    assert!(b.warnings.is_empty(), "{:?}", b.warnings);
    assert_eq!(b.parts.len(), 1);
    let cut = before - volume(&b.parts[0]);
    // About 11.6 mm of the groove is in the shaft (from its end to 12 − P/2, and its half width).
    let want = M10_PER_MM * 11.6;
    assert!(close(cut, want, 0.1), "{cut} vs {want}");
}

#[test]
fn an_internal_thread_cuts_into_a_tapped_hole_and_runs_through() {
    let mut st = Studio::new();
    let s = st.sketch(
        PlaneRef::Top,
        vec![
            SketchOp::AddPolyline {
                points: vec![
                    Vec2::new(-10.0, -10.0),
                    Vec2::new(10.0, -10.0),
                    Vec2::new(10.0, 10.0),
                    Vec2::new(-10.0, 10.0),
                ],
                closed: true,
                construction: false,
                label: "Add rectangle",
            },
            circle(4.25),
        ],
    );
    let regions = cadrs_sketch::region::regions(&st.g(s));
    let plate = regions.iter().find(|r| !r.holes.is_empty()).unwrap();
    st.extrude(ExtrudeFeature {
        regions: vec![RegionRef::new(s, plate)],
        depth: 8.0,
        depth_expr: "8 mm".into(),
        ..Default::default()
    });
    let block = st.parts()[0].clone();
    let before = volume(&block);
    st.add(
        "Internal thread",
        FeatureKind::Thread(ThreadFeature {
            edge: Some(edge_near(&block, [4.25, 0.0, 8.0])),
            end: ThreadEnd::Full,
            clockwise: false,
            ..Default::default()
        }),
    );
    let b = st.build();
    assert!(b.warnings.is_empty(), "{:?}", b.warnings);
    assert_eq!(b.parts.len(), 1);
    let cut = before - volume(&b.parts[0]);
    // The groove from Ø8.5 out to Ø10 (narrower than the shaft's), all 8 mm of the hole.
    assert!(cut > 4.0 * 8.0 && cut < M10_PER_MM * 8.0, "{cut}");
}

#[test]
fn a_shaft_of_no_standard_size_warns() {
    let mut st = Studio::new();
    let s = st.sketch(PlaneRef::Top, vec![circle(4.8)]);
    st.extrude(ExtrudeFeature {
        sketches: vec![s],
        depth: 10.0,
        depth_expr: "10 mm".into(),
        ..Default::default()
    });
    let shaft = st.parts()[0].clone();
    st.add(
        "External thread",
        FeatureKind::Thread(ThreadFeature {
            edge: Some(edge_near(&shaft, [4.8, 0.0, 10.0])),
            length: 5.0,
            length_expr: "5 mm".into(),
            ..Default::default()
        }),
    );
    let b = st.build();
    assert_eq!(b.warnings.len(), 1, "{:?}", b.warnings);
}
