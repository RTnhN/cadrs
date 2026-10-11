//! A PCB Studio board derived into a Part Studio (`docs/PCB_IN_PART_STUDIO.md` P1): the board's
//! solid and its components' mesh parts, placed by the Derived feature's locations, following
//! the board's layout live, with ids that survive layout edits.
//!
//! The board is the KiCad course's finished board ([`cadrs_eda::getting_started::gs18`]):
//! a 45 × 50 mm outline, 1.6 mm thick, with BT1 on the bottom and D1, R1 on top. Expected
//! values are worked out from the layout here, not read back from the model.
#![cfg(feature = "occt")]

use cadrs_core::command::Command;
use cadrs_core::commands::AddFeature;
use cadrs_core::derived::{self, AddDerived, DerivedBoard, DerivedFeature, SetDerived};
use cadrs_core::document::{Element, FeatureKind};
use cadrs_core::external::{RefAt, Resolver, SourceRef};
use cadrs_core::mate::{ConnectorOrigin, ConnectorRef, MateConnectorFeature};
use cadrs_core::pcb::board3d::{BoardAnchor, BoardSeat, ComponentFilter};
use cadrs_core::pcb::{AddBoard, BoardId, SetDesign};
use cadrs_core::{Document, ElementId, FeatureId, History, Part, Store};
use cadrs_eda::units::{mm, to_mm};

const STUDIO: ElementId = ElementId::from_u128(0x7c00_0000_0000_0000_0000_0000_0000_0001);
const PCB: ElementId = ElementId::from_u128(0x7c00_0000_0000_0000_0000_0000_0000_0002);
const DERIVED: FeatureId = FeatureId::from_u128(0x7c00_0000_0000_0000_0000_0000_0000_0003);
const LOC: FeatureId = FeatureId::from_u128(0x7c00_0000_0000_0000_0000_0000_0000_0004);

fn run(doc: &mut Document, h: &mut History, c: &dyn Command) {
    h.execute(doc, c).unwrap_or_else(|e| panic!("{}: {e}", c.label()));
}

fn course() -> cadrs_eda::Design {
    cadrs_eda::getting_started::gs18(&cadrs_eda::library::LibraryTable::builtin())
}

/// A document with a PCB Studio holding the course board and a Part Studio with a mate
/// connector at (0, 0, 100).
fn document() -> (Document, History, BoardId) {
    let mut doc = Document::empty("Board in a box");
    let mut ps = Element::part_studio("Part Studio 1");
    ps.id = STUDIO;
    let mut pcb = Element::pcb_studio("PCB Studio 1");
    pcb.id = PCB;
    doc.elements.push(ps);
    doc.elements.push(pcb);
    let mut h = History::default();
    run(&mut doc, &mut h, &AddBoard { element: PCB, name: Some("Course".into()), design: Box::new(course()), imported_from: None });
    let id = doc.element(PCB).unwrap().pcb().unwrap().boards[0].id;
    let mc = MateConnectorFeature { origin: Some(ConnectorOrigin::Origin), offset: [0.0, 0.0, 100.0], offset_expr: ["0 mm", "0 mm", "100 mm"].map(String::from), ..Default::default() };
    run(&mut doc, &mut h, &AddFeature { element: STUDIO, feature: LOC, base_name: "Mate connector".into(), kind: FeatureKind::MateConnector(mc) });
    (doc, h, id)
}

/// Derives the board into the Part Studio with `d`'s options.
fn derive(doc: &mut Document, h: &mut History, board: BoardId, mut d: DerivedFeature) -> Result<(), cadrs_core::CommandError> {
    d.board.get_or_insert_with(|| Box::new(DerivedBoard::new(board, "", Default::default()))).board = board;
    let mut res = Resolver::new(Store::new(std::env::temp_dir().join("cadrs-derived-board-unused")));
    let r = SourceRef { document: None, at: RefAt::Workspace, element: PCB, pinned: false };
    let got = derived::resolve(&mut res, doc, None, d, r).map_err(cadrs_core::CommandError::from)?;
    h.execute(doc, &AddDerived { element: STUDIO, feature: DERIVED, derived: got.derived, links: got.links })
}

fn build(doc: &Document) -> std::sync::Arc<cadrs_core::rebuild::Build> {
    cadrs_core::rebuild::build(doc.element(STUDIO).unwrap().features())
}

fn bounds(p: &Part) -> ([f64; 3], [f64; 3]) {
    let mut lo = [f64::MAX; 3];
    let mut hi = [f64::MIN; 3];
    for q in &p.solid.positions {
        for k in 0..3 {
            lo[k] = lo[k].min(q[k]);
            hi[k] = hi[k].max(q[k]);
        }
    }
    (lo, hi)
}

fn derived_parts(b: &cadrs_core::rebuild::Build) -> Vec<&Part> {
    b.parts.iter().filter(|p| p.feature == DERIVED).collect()
}

#[test]
fn the_board_and_its_components_come_in_at_the_location() {
    let (mut doc, mut h, board) = document();
    let d = DerivedFeature { locations: vec![ConnectorRef::Feature(LOC)], ..Default::default() };
    derive(&mut doc, &mut h, board, d).unwrap();
    let b = build(&doc);
    assert!(b.errors.is_empty(), "{:?}", b.errors);
    let parts = derived_parts(&b);
    let names: Vec<&str> = parts.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names.len(), 4, "{names:?}");
    assert_eq!(names[0], "Course");
    for r in ["BT1", "D1", "R1"] {
        assert!(names.iter().any(|n| n.starts_with(&format!("{r} "))), "{r} in {names:?}");
    }
    // The board: 45 × 50 × 1.6, its outline's centre on the connector, bottom face on z = 100.
    let board_part = parts[0];
    let (lo, hi) = bounds(board_part);
    assert!((hi[0] - lo[0] - 45.0).abs() < 1e-6 && (hi[1] - lo[1] - 50.0).abs() < 1e-6, "{lo:?} {hi:?}");
    assert!((lo[0] + 22.5).abs() < 1e-6 && (lo[1] + 25.0).abs() < 1e-6);
    assert!((lo[2] - 100.0).abs() < 1e-9 && (hi[2] - 101.6).abs() < 1e-9);
    // Its volume: the outline's area times the thickness, less the holes through it.
    let layout = &course().board;
    let holes: f64 = cadrs_eda::fab::holes(layout).iter().filter(|h| to_mm(h.size.w) >= cadrs_core::pcb::board3d::MIN_HOLE).map(|h| std::f64::consts::PI * (to_mm(h.size.w) / 2.0).powi(2)).sum();
    let v = board_part.mass.unwrap().volume;
    assert!((v - (45.0 * 50.0 - holes) * 1.6).abs() < 1e-3, "volume {v}");
    // The components are mesh parts (no mass), on their side of the board.
    let d1 = parts.iter().find(|p| p.name.starts_with("D1 ")).unwrap();
    assert!(d1.mass.is_none());
    assert!(bounds(d1).1[2] > 101.6 + 3.0);
    let bt1 = parts.iter().find(|p| p.name.starts_with("BT1 ")).unwrap();
    assert!(bounds(bt1).0[2] < 100.0 - 1.0, "BT1 hangs below");
    // The board's skins ride on it.
    assert_eq!(board_part.solid.images.len(), 2);
    // What the Feature list and the edit mode read.
    let out = &b.derived[&DERIVED];
    assert_eq!(out.boards.len(), 1);
    assert_eq!(out.footprints.len(), 3);
    let f = out.boards[0].frame;
    assert!((f.origin[0] - (0.0 - 22.5)).abs() < 1e-9 && (f.origin[2] - 100.0).abs() < 1e-9, "{f:?}");
}

#[test]
fn layout_edits_follow_and_keep_part_and_face_ids() {
    let (mut doc, mut h, board) = document();
    derive(&mut doc, &mut h, board, DerivedFeature::default()).unwrap();
    let before = build(&doc);
    let d1 = derived_parts(&before).into_iter().find(|p| p.name.starts_with("D1 ")).unwrap().clone();
    // A mate connector on D1's top face keeps its place on it.
    let top = d1.solid.faces.iter().filter(|f| f.plane.is_some_and(|p| p.normal()[2] > 0.99)).max_by(|a, b| a.center.unwrap()[2].total_cmp(&b.center.unwrap()[2])).unwrap();
    let face = cadrs_core::document::FaceRef { part: d1.id, face: top.name, seed: top.center.unwrap() };
    let mc = MateConnectorFeature { origin: Some(ConnectorOrigin::Face(face)), ..Default::default() };
    let on_d1 = FeatureId::new();
    run(&mut doc, &mut h, &AddFeature { element: STUDIO, feature: on_d1, base_name: "Mate connector".into(), kind: FeatureKind::MateConnector(mc) });
    let at = build(&doc).connectors[&on_d1];
    // Move D1 5 mm along x in the PCB Studio.
    let mut design = doc.element(PCB).unwrap().pcb().unwrap().board(board).unwrap().design.as_deref().unwrap().clone();
    let i = cadrs_eda::board_edit::footprint_index(&design.board, "D1").unwrap();
    let to = design.board.footprints[i].placement.at + cadrs_eda::units::Pt::new(mm(5.0), 0);
    cadrs_eda::board_edit::move_footprint(&mut design.board, i, to);
    run(&mut doc, &mut h, &SetDesign { element: PCB, board, design: Box::new(design), label: "Move D1".into() });
    // The Derived feature follows (one command, refreshed by the history).
    let after = build(&doc);
    assert!(after.errors.is_empty(), "{:?}", after.errors);
    let moved = after.part(d1.id).expect("D1 keeps its part id");
    let names = |p: &Part| p.solid.faces.iter().map(|f| f.name).collect::<Vec<_>>();
    assert_eq!(names(moved), names(&d1), "and its face names");
    let (a, b) = (bounds(&d1), bounds(moved));
    assert!((b.0[0] - a.0[0] - 5.0).abs() < 1e-6 && (b.0[1] - a.0[1]).abs() < 1e-6);
    let now = after.connectors[&on_d1];
    assert!((now.origin[0] - at.origin[0] - 5.0).abs() < 1e-6 && (now.origin[1] - at.origin[1]).abs() < 1e-6, "{at:?} → {now:?}");
    // Undo puts it back.
    h.undo(&mut doc).unwrap();
    let back = build(&doc);
    assert!((bounds(back.part(d1.id).unwrap()).0[0] - a.0[0]).abs() < 1e-6);
}

#[test]
fn options_filter_and_seat_the_board() {
    let (mut doc, mut h, board) = document();
    derive(&mut doc, &mut h, board, DerivedFeature::default()).unwrap();
    let FeatureKind::Derived(mut d) = doc.element(STUDIO).unwrap().feature(DERIVED).unwrap().kind.clone() else { unreachable!() };
    let o = &mut d.board.as_mut().unwrap().options;
    o.components = ComponentFilter::Top;
    o.seat = BoardSeat::Top;
    o.anchor = BoardAnchor::Origin;
    run(&mut doc, &mut h, &SetDerived { element: STUDIO, feature: DERIVED, derived: *d, links: vec![], label: "Edit derived".into() });
    let b = build(&doc);
    let parts = derived_parts(&b);
    assert_eq!(parts.len(), 3, "the board, D1 and R1");
    // The top face on the origin's plane: the board below it; the layout's origin on the origin.
    let (lo, hi) = bounds(parts[0]);
    assert!((hi[2]).abs() < 1e-9 && (lo[2] + 1.6).abs() < 1e-9, "{lo:?} {hi:?}");
    let region = cadrs_eda::outline::board_region(&course().board);
    let xmin = region.iter().flatten().map(|p| to_mm(p.x)).fold(f64::MAX, f64::min);
    assert!((lo[0] - xmin).abs() < 1e-6);
}

#[test]
fn a_board_is_derived_once_per_part_studio() {
    let (mut doc, mut h, board) = document();
    derive(&mut doc, &mut h, board, DerivedFeature::default()).unwrap();
    let mut res = Resolver::new(Store::new(std::env::temp_dir().join("cadrs-derived-board-unused")));
    let r = SourceRef { document: None, at: RefAt::Workspace, element: PCB, pinned: false };
    let d = DerivedFeature { board: Some(Box::new(DerivedBoard::new(board, "", Default::default()))), ..Default::default() };
    let got = derived::resolve(&mut res, &doc, None, d, r).unwrap();
    let e = h.execute(&mut doc, &AddDerived { element: STUDIO, feature: FeatureId::new(), derived: got.derived, links: vec![] }).unwrap_err();
    assert!(e.to_string().contains("derived only once"), "{e}");
}

#[test]
fn renaming_the_board_renames_the_part_and_the_source() {
    let (mut doc, mut h, board) = document();
    derive(&mut doc, &mut h, board, DerivedFeature::default()).unwrap();
    run(&mut doc, &mut h, &cadrs_core::pcb::RenameBoard { element: PCB, board, name: "Main board".into() });
    let FeatureKind::Derived(d) = &doc.element(STUDIO).unwrap().feature(DERIVED).unwrap().kind else { unreachable!() };
    assert_eq!(d.describe(), "PCB Studio 1 › Main board");
    assert_eq!(derived_parts(&build(&doc))[0].name, "Main board");
}

#[test]
fn the_enclosure_sample_builds_with_its_named_parts() {
    use cadrs_core::samples::pcb_enclosure as pe;
    let doc = pe::document().unwrap();
    let b = cadrs_core::rebuild::build(doc.element(pe::STUDIO).unwrap().features());
    assert!(b.errors.is_empty(), "{:?}", b.errors);
    let props = doc.element(pe::STUDIO).unwrap().part_props();
    let names: Vec<&str> = b.parts.iter().map(|p| cadrs_core::parts::display_name(p, props)).collect();
    for n in ["Tube", "End plate", "Plug", "Live pin", "Earth pin", "Neutral pin"] {
        assert!(names.contains(&n), "{n} in {names:?}");
    }
    let loc = b.connectors[&pe::BOARD_LOCATION];
    assert!((loc.origin[2] - pe::BOARD_Z).abs() < 1e-9);
    assert_eq!(doc.element(pe::PCB).unwrap().pcb().unwrap().boards[0].name(), "AC sensor");
}

#[test]
fn the_enclosure_board_derives_cleanly() {
    use cadrs_core::samples::pcb_enclosure as pe;
    let mut doc = pe::document().unwrap();
    let mut h = History::default();
    let board = doc.element(pe::PCB).unwrap().pcb().unwrap().boards[0].id;
    let d = DerivedFeature { locations: vec![ConnectorRef::Feature(pe::BOARD_LOCATION)], board: Some(Box::new(DerivedBoard::new(board, "", Default::default()))), ..Default::default() };
    let mut res = Resolver::new(Store::new(std::env::temp_dir().join("cadrs-derived-board-unused")));
    let r = SourceRef { document: None, at: RefAt::Workspace, element: pe::PCB, pinned: false };
    let got = derived::resolve(&mut res, &doc, None, d, r).unwrap();
    h.execute(&mut doc, &AddDerived { element: pe::STUDIO, feature: DERIVED, derived: got.derived, links: vec![] }).unwrap();
    let b = cadrs_core::rebuild::build(doc.element(pe::STUDIO).unwrap().features());
    assert!(b.errors.is_empty() && b.warnings.is_empty(), "{:?} {:?}", b.errors, b.warnings);
    // Its bottom face on Board location, 2 mm below the pins' tips.
    let out = &b.derived[&DERIVED];
    assert!((out.boards[0].frame.origin[2] - pe::BOARD_Z).abs() < 1e-9);
}
