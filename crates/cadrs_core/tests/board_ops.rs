//! Board edits by name and number (`cadrs_core::pcb::ops`, the MCP server's board tools) and a
//! PCB Studio board inserted straight into an assembly (`docs/PCB_IN_PART_STUDIO.md` P6–P7), on
//! the mains-sensor sample: its AC sensor board derived onto "Board location" in the
//! Enclosure, whose plug's Live, Earth and Neutral pins stand at (0, 8), (0, 0) and (0, −8).
#![cfg(feature = "occt")]

use cadrs_core::assembly::bom::{self, BomOptions};
use cadrs_core::assembly::commands::InsertInstance;
use cadrs_core::assembly::structure::occurrences;
use cadrs_core::assembly::{Instance, InstanceId, Pose};
use cadrs_core::derived::{AddDerived, DerivedBoard, DerivedFeature};
use cadrs_core::document::Element;
use cadrs_core::external::{RefAt, Resolver, SourceRef};
use cadrs_core::mate::ConnectorRef;
use cadrs_core::pcb::{BoardId, SetDesign, ops};
use cadrs_core::samples::pcb_enclosure as pe;
use cadrs_core::{Document, ElementId, FeatureId, History, Store};

const DERIVED: FeatureId = FeatureId::from_u128(0x7d00_0000_0000_0000_0000_0000_0000_0001);

fn run(doc: &mut Document, h: &mut History, c: &dyn cadrs_core::Command) {
    h.execute(doc, c).unwrap_or_else(|e| panic!("{}: {e}", c.label()));
}

/// The sample with its board derived onto Board location.
fn derived() -> (Document, History, BoardId) {
    let mut doc = pe::document().unwrap();
    let mut h = History::default();
    let board = doc.element(pe::PCB).unwrap().pcb().unwrap().boards[0].id;
    let d = DerivedFeature { locations: vec![ConnectorRef::Feature(pe::BOARD_LOCATION)], board: Some(Box::new(DerivedBoard::new(board, "", Default::default()))), ..Default::default() };
    let mut res = Resolver::new(Store::new(std::env::temp_dir().join("cadrs-board-ops-unused")));
    let r = SourceRef { document: None, at: RefAt::Workspace, element: pe::PCB, pinned: false };
    let got = cadrs_core::derived::resolve(&mut res, &doc, None, d, r).unwrap();
    run(&mut doc, &mut h, &AddDerived { element: pe::STUDIO, feature: DERIVED, derived: got.derived, links: got.links });
    (doc, h, board)
}

fn store(doc: &mut Document, h: &mut History, board: BoardId, f: impl FnOnce(&mut cadrs_eda::Design) -> Result<(), String>) {
    let mut d = ops::design_of(doc, pe::PCB, board).unwrap();
    f(&mut d).unwrap();
    run(doc, h, &SetDesign { element: pe::PCB, board, design: Box::new(d), label: "Edit".into() });
}

fn pad(doc: &Document, reference: &str, number: &str) -> [f64; 2] {
    let r = ops::board_report(doc, pe::PCB, doc.element(pe::PCB).unwrap().pcb().unwrap().boards[0].id).unwrap();
    r.footprints.iter().find(|f| f.reference == reference).unwrap().pads.iter().find(|p| p.number == number).unwrap().at
}

#[test]
fn a_board_is_found_by_name_and_reported() {
    let (doc, _, board) = derived();
    assert_eq!(ops::find_board(&doc, None, "AC sensor").unwrap(), (pe::PCB, board));
    assert!(ops::find_board(&doc, None, "Nope").unwrap_err().contains("AC sensor"));
    let r = ops::board_report(&doc, pe::PCB, board).unwrap();
    assert_eq!(r.name, "AC sensor");
    assert!((r.thickness - 1.6).abs() < 1e-9);
    let j4 = r.footprints.iter().find(|f| f.reference == "J4").unwrap();
    assert_eq!(j4.pads[0].net, "LIVE");
    // Derived into the Enclosure, with PS1's collisions as its warning.
    assert_eq!(r.derived_in.len(), 1);
    assert!(r.derived_in[0].warning.as_deref().is_some_and(|w| w.contains("PS1")), "{:?}", r.derived_in);
}

#[test]
fn a_pad_goes_onto_a_pin_of_the_part_studio() {
    let (mut doc, mut h, board) = derived();
    // The Live pin's axis on the layout: the pin's (0, 8) with the board's outline centre
    // (15.5, 19.5) on the origin.
    let axes = ops::axes_of_part(&doc, pe::STUDIO, DERIVED, "Live pin").unwrap();
    let at = axes[0].0;
    assert!((at[0] - 15.5).abs() < 1e-6 && (at[1] - 27.5).abs() < 1e-6, "{at:?}");
    store(&mut doc, &mut h, board, |d| ops::pad_onto(d, "J4", "1", at));
    let p = pad(&doc, "J4", "1");
    assert!((p[0] - at[0]).abs() < 1e-3 && (p[1] - at[1]).abs() < 1e-3, "{p:?}");
    // In the Part Studio, J4's wire end is on the pin's axis.
    let b = cadrs_core::rebuild::build(doc.element(pe::STUDIO).unwrap().features());
    let j4 = b.parts.iter().find(|x| x.name.starts_with("J4 ")).unwrap();
    let (lo, hi) = j4.solid.bounds().unwrap();
    assert!(((lo[0] + hi[0]) / 2.0).abs() < 1e-3 && ((lo[1] + hi[1]) / 2.0 - 8.0).abs() < 1e-3, "{lo:?} {hi:?}");
}

#[test]
fn footprints_are_placed_copied_and_removed() {
    let (mut doc, mut h, board) = derived();
    // Placed, turned and flipped.
    store(&mut doc, &mut h, board, |d| ops::place_footprint(d, "R1", Some([10.0, 12.0]), Some(90.0), Some(cadrs_eda::layer::Side::Bottom)));
    let r = ops::board_report(&doc, pe::PCB, board).unwrap();
    let r1 = r.footprints.iter().find(|f| f.reference == "R1").unwrap();
    assert_eq!((r1.at, r1.side), ([10.0, 12.0], "bottom"));
    // A copy of J4 as an Earth input on the Earth pin, on its own net.
    let earth = ops::axes_of_part(&doc, pe::STUDIO, DERIVED, "Earth pin").unwrap()[0].0;
    let parts_before = cadrs_core::rebuild::build(doc.element(pe::STUDIO).unwrap().features()).parts.len();
    store(&mut doc, &mut h, board, |d| ops::copy_footprint(d, "J4", "J6", "AC Earth IN", earth, &[("1".into(), "EARTH".into())]).map(|_| ()));
    let r = ops::board_report(&doc, pe::PCB, board).unwrap();
    let j6 = r.footprints.iter().find(|f| f.reference == "J6").unwrap();
    assert_eq!((j6.value.as_str(), j6.pads[0].net.as_str()), ("AC Earth IN", "EARTH"));
    assert!(r.nets.contains(&"EARTH".to_string()));
    assert!(ops::copy_footprint(&mut ops::design_of(&doc, pe::PCB, board).unwrap(), "J4", "J6", "", [0.0, 0.0], &[]).is_err(), "J6 is taken");
    // Its part comes into the Part Studio.
    assert_eq!(cadrs_core::rebuild::build(doc.element(pe::STUDIO).unwrap().features()).parts.len(), parts_before + 1);
    // PS1 removed: gone from the board and the schematic, and its collisions with it.
    store(&mut doc, &mut h, board, |d| ops::remove_footprint(d, "PS1"));
    let design = ops::design_of(&doc, pe::PCB, board).unwrap();
    assert!(design.board.footprints.iter().all(|f| f.reference() != "PS1"));
    assert!(design.schematic.sheets.iter().flat_map(|s| &s.symbols).all(|s| s.reference() != "PS1"));
    let r = ops::board_report(&doc, pe::PCB, board).unwrap();
    assert!(r.derived_in[0].warning.as_deref().is_none_or(|w| !w.contains("PS1")), "{:?}", r.derived_in);
    // Shifting the layout moves the parts and the tracks, not the outline.
    let mut d = ops::design_of(&doc, pe::PCB, board).unwrap();
    let before = cadrs_core::pcb::board3d::outline_center(&d.board);
    let r1_at = d.board.footprints.iter().find(|f| f.reference() == "R1").unwrap().placement.at;
    ops::shift_layout(&mut d.board, [5.0, -2.0]);
    assert_eq!(cadrs_core::pcb::board3d::outline_center(&d.board), before);
    assert_eq!(d.board.footprints.iter().find(|f| f.reference() == "R1").unwrap().placement.at, r1_at + cadrs_eda::units::Pt::mm(5.0, -2.0));
}

#[test]
fn a_board_in_an_assembly_follows_its_layout_and_nests_in_the_bom() {
    let (mut doc, mut h, board) = derived();
    let mut asm = Element::assembly("Assembly 1");
    let asm_id = ElementId::new();
    asm.id = asm_id;
    doc.elements.push(asm);
    let parts = doc.element(pe::PCB).unwrap().pcb().unwrap().board_parts(board);
    let inst = InstanceId::new();
    run(&mut doc, &mut h, &InsertInstance { element: asm_id, instance: Instance::board(inst, pe::PCB, board, parts.clone(), Pose::IDENTITY) });
    let count = |doc: &Document| occurrences(doc, doc.element(asm_id).unwrap().assembly_model().unwrap()).iter().filter(|o| o.top == inst).count();
    assert_eq!(count(&doc), parts.len());
    // The parts are the PCB Studio's rebuild's: the board's and its components'.
    let build = cadrs_core::rebuild::build(doc.element(pe::PCB).unwrap().part_features());
    for p in &parts {
        assert!(build.part(*p).is_some(), "{p:?} is built");
    }
    // A footprint added to the layout comes into the assembly.
    store(&mut doc, &mut h, board, |d| ops::copy_footprint(d, "J4", "J6", "AC Earth IN", [15.5, 19.5], &[]).map(|_| ()));
    assert_eq!(count(&doc), parts.len() + 1);
    // The structured BOM: the board, its components under it.
    let builds = |e: ElementId| doc.element(e).map(|x| cadrs_core::rebuild::build(x.part_features()));
    let options = BomOptions { expand_all: true, ..Default::default() };
    let b = bom::compute(&doc, asm_id, &options, &doc.units.clone(), builds).unwrap();
    let board_row = b.rows.iter().find(|r| r.has_children).expect("the board's row has children");
    assert_eq!(board_row.depth, 0);
    assert_eq!(b.rows.iter().filter(|r| r.depth == 1).count(), parts.len(), "its components (J6 too) under it");
}

#[test]
fn the_export_dialog_path_writes_a_board_and_its_components() {
    use cadrs_core::rebuild::exchange::{ExportItem, ExportRequest, ModelFormat};
    let (doc, _, _) = derived();
    let features = std::sync::Arc::new(doc.element(pe::STUDIO).unwrap().active_features());
    let b = cadrs_core::rebuild::build(&features);
    let items: Vec<ExportItem> = b.parts.iter().filter(|p| p.feature == DERIVED).map(|p| ExportItem { features: features.clone(), part: p.id, name: p.name.clone(), pose: None, source: (pe::STUDIO, p.id), source_name: p.name.clone() }).collect();
    let n = items.len();
    for format in [ModelFormat::Step, ModelFormat::Stl { binary: true }] {
        let files = cadrs_core::rebuild::exchange::export_files(ExportRequest::new(format, "Board", items.clone())).wait().expect("the job ends").unwrap_or_else(|e| panic!("{format:?}: {e}"));
        assert!(!files[0].bytes.is_empty());
        if format == ModelFormat::Step {
            assert!(String::from_utf8_lossy(&files[0].bytes).matches("MANIFOLD_SOLID_BREP").count() >= n, "a solid per part");
        }
    }
}
