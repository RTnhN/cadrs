//! The MCP server's PCB board tools (`docs/PCB_IN_PART_STUDIO.md` P7): read a board, derive it
//! into a Part Studio, make one from a sketch, and move, add or remove its footprints, the
//! fitting done by name and number ([`cadrs_core::pcb::ops`]). Each change is one undo step of
//! the open document.

use bevy::prelude::*;
use cadrs_core::derived::{AddDerived, DerivedBoard, DerivedFeature};
use cadrs_core::external::{RefAt, SourceRef};
use cadrs_core::mate::ConnectorRef;
use cadrs_core::pcb::board3d::{BoardAnchor, BoardSeat, BoardView};
use cadrs_core::pcb::ops;
use cadrs_core::pcb::{BoardId, SetDesign};
use cadrs_core::{ElementId, FeatureId};
use cadrs_eda::layer::Side;
use cadrs_mcp::tools;
use serde_json::{Value, json};

use crate::ActiveDocument;

fn doc(world: &World) -> Result<&ActiveDocument, String> {
    world.get_resource::<ActiveDocument>().ok_or_else(|| "no document is open: call open_document first".into())
}

fn board(world: &World, b: &tools::BoardName) -> Result<(ElementId, BoardId), String> {
    ops::find_board(&doc(world)?.doc, b.pcb_studio.as_deref(), &b.board)
}

/// The Part Studio a call names, or the active tab.
fn studio(world: &World, name: Option<&str>) -> Result<ElementId, String> {
    let d = doc(world)?;
    let e = match name {
        Some(n) => d.doc.elements.iter().find(|e| e.name == n).ok_or_else(|| format!("no tab named {n:?}"))?,
        None => d.active_element().ok_or("no tab is active")?,
    };
    if !matches!(e.kind, cadrs_core::document::ElementKind::PartStudio { .. }) {
        return Err(format!("{:?} is not a Part Studio", e.name));
    }
    Ok(e.id)
}

fn feature_id(world: &World, studio: ElementId, name: &str) -> Result<FeatureId, String> {
    doc(world)?.doc.element(studio).and_then(|e| e.features().iter().find(|f| f.name == name)).map(|f| f.id).ok_or_else(|| format!("no feature named {name:?}"))
}

fn execute(world: &mut World, cmd: &dyn cadrs_core::Command) -> Result<(), String> {
    world.get_resource_mut::<ActiveDocument>().ok_or("no document is open")?.execute(cmd).map_err(|e| e.to_string())
}

fn report(world: &World, (el, b): (ElementId, BoardId)) -> Result<Value, String> {
    serde_json::to_value(ops::board_report(&doc(world)?.doc, el, b)?).map_err(|e| e.to_string())
}

/// Changes a board's design by `f` and stores it (one undo step named `label`); returns the
/// board's report.
fn edit(world: &mut World, b: &tools::BoardName, label: &str, f: impl FnOnce(&mut cadrs_eda::Design) -> Result<(), String>) -> Result<Value, String> {
    let (el, id) = board(world, b)?;
    let mut d = ops::design_of(&doc(world)?.doc, el, id)?;
    f(&mut d)?;
    execute(world, &SetDesign { element: el, board: id, design: Box::new(d), label: label.into() })?;
    report(world, (el, id))
}

pub fn get_board(world: &mut World, p: &tools::BoardName) -> Result<Value, String> {
    let b = board(world, p)?;
    report(world, b)
}

pub fn get_board_fit(world: &mut World, p: &tools::BoardFit) -> Result<Value, String> {
    let (pcb, id) = board(world, &p.board)?;
    let ps = studio(world, p.part_studio.as_deref())?;
    let d = &doc(world)?.doc;
    let f = ops::deriving_feature(d, ps, pcb, id).ok_or("the board isn't derived into that Part Studio: derive_board first")?;
    let el = d.element(ps).ok_or("no Part Studio")?;
    let build = cadrs_core::rebuild::build(el.features());
    let mut parts = vec![];
    for part in build.parts.iter().filter(|x| x.feature != f) {
        let name = cadrs_core::parts::display_name(part, el.part_props()).to_string();
        if let Ok(axes) = ops::axes_of_part(d, ps, f, &name) {
            parts.push(json!({ "part": name, "axes": axes.iter().map(|(a, r)| json!({ "at": [round(a[0]), round(a[1])], "radius": round(*r) })).collect::<Vec<_>>() }));
        }
    }
    let name = |x: cadrs_core::PartId| build.part(x).map(|p| cadrs_core::parts::display_name(p, el.part_props()).to_string()).unwrap_or_default();
    let collisions: Vec<String> = build.derived.get(&f).map(|o| o.collisions.iter().map(|c| cadrs_core::pcb::fit::collision_text(&name(c.component), &name(c.other), c)).collect()).unwrap_or_default();
    Ok(json!({ "feature": el.feature(f).map(|x| x.name.clone()), "pins_and_holes": parts, "collisions": collisions }))
}

fn round(v: f64) -> f64 {
    (v * 1e3).round() / 1e3
}

fn seat(s: Option<&str>) -> Result<BoardSeat, String> {
    match s.map(str::to_ascii_lowercase).as_deref() {
        None | Some("bottom") => Ok(BoardSeat::Bottom),
        Some("top") => Ok(BoardSeat::Top),
        Some(o) => Err(format!("seat {o:?}: bottom or top")),
    }
}

pub fn derive_board(world: &mut World, p: &tools::DeriveBoard) -> Result<Value, String> {
    let (pcb, id) = board(world, &p.board)?;
    let ps = studio(world, p.part_studio.as_deref())?;
    let location = p.location.as_deref().map(|n| feature_id(world, ps, n)).transpose()?;
    let on_sketch = p.on_sketch.as_deref().map(|n| feature_id(world, ps, n)).transpose()?;
    let mut db = DerivedBoard::new(id, "", Default::default());
    db.options.seat = seat(p.seat.as_deref())?;
    db.options.view = match p.view.as_deref().map(str::to_ascii_lowercase).as_deref() {
        None | Some("3d") => BoardView::ThreeD,
        Some("layout") => BoardView::Layout,
        Some(o) => return Err(format!("view {o:?}: 3d or layout")),
    };
    db.on_sketch = on_sketch;
    let d = DerivedFeature { locations: location.map(ConnectorRef::Feature).into_iter().collect(), board: Some(Box::new(db)), ..Default::default() };
    let r = SourceRef { document: None, at: RefAt::Workspace, element: pcb, pinned: false };
    let doc_now = doc(world)?.doc.clone();
    let got = {
        let mut res = crate::linked::resolver(world);
        cadrs_core::derived::resolve(&mut res.0, &doc_now, None, d, r).map_err(|e| e.to_string())?
    };
    let mut derived = got.derived;
    // The anchor: fixed by default (resolve set it); origin on a sketch; or as asked.
    if let Some(b) = derived.board.as_mut() {
        b.options.anchor = match p.anchor.as_deref().map(str::to_ascii_lowercase).as_deref() {
            None if on_sketch.is_some() => BoardAnchor::Origin,
            None | Some("fixed") => b.options.anchor,
            Some("origin") => BoardAnchor::Origin,
            Some("centre") | Some("center") => BoardAnchor::Center,
            Some(r) => BoardAnchor::Footprint(b.layout.footprints.iter().find(|f| f.reference().eq_ignore_ascii_case(r)).map(|f| f.id).ok_or_else(|| format!("anchor {r:?}: fixed, origin, centre or a footprint's reference"))?),
        };
    }
    let feature = FeatureId::new();
    execute(world, &AddDerived { element: ps, feature, derived, links: got.links })?;
    let el = doc(world)?.doc.element(ps).ok_or("no Part Studio")?;
    let build = cadrs_core::rebuild::build(el.features());
    Ok(json!({
        "feature": el.feature(feature).map(|f| f.name.clone()),
        "error": build.error(feature),
        "warning": build.warning(feature),
        "parts": build.parts.iter().filter(|x| x.feature == feature).map(|x| cadrs_core::parts::display_name(x, el.part_props()).to_string()).collect::<Vec<_>>(),
    }))
}

pub fn create_board_from_sketch(world: &mut World, p: &tools::CreateBoardFromSketch) -> Result<Value, String> {
    let ps = studio(world, p.part_studio.as_deref())?;
    let sketch = feature_id(world, ps, &p.sketch)?;
    let regions = {
        let d = doc(world)?;
        let g = d.doc.element(ps).and_then(|e| e.feature(sketch)).and_then(|f| f.sketch()).map(|s| s.geometry.clone()).ok_or("not a sketch")?;
        crate::board_in_studio::largest_region(&g, sketch)
    };
    if regions.is_empty() {
        return Err("the sketch has no closed region".into());
    }
    let pcb = match &p.pcb_studio {
        Some(n) => Some(doc(world)?.doc.elements.iter().find(|e| e.name == *n && e.pcb().is_some()).map(|e| e.id).ok_or_else(|| format!("no PCB Studio named {n:?}"))?),
        None => None,
    };
    let cmd = cadrs_core::pcb::outline_link::CreateBoardFromSketch {
        element: ps,
        sketch,
        regions,
        pcb,
        new_pcb: ElementId::new(),
        name: p.name.clone(),
        thickness: p.thickness.unwrap_or(1.6),
        seat: seat(p.seat.as_deref())?,
        feature: FeatureId::new(),
    };
    execute(world, &cmd)?;
    let b = ops::find_board(&doc(world)?.doc, None, &p.name).or_else(|_| {
        // (Made unique in its studio: the newest board.)
        let d = &doc(world)?.doc;
        d.elements.iter().rev().find_map(|e| e.pcb().and_then(|s| s.active).map(|b| (e.id, b))).ok_or_else(|| "the board is gone".to_string())
    })?;
    report(world, b)
}

fn side(s: Option<&str>) -> Result<Option<Side>, String> {
    match s.map(str::to_ascii_lowercase).as_deref() {
        None => Ok(None),
        Some("top") => Ok(Some(Side::Top)),
        Some("bottom") => Ok(Some(Side::Bottom)),
        Some(o) => Err(format!("side {o:?}: top or bottom")),
    }
}

pub fn move_footprint(world: &mut World, p: &tools::MoveFootprint) -> Result<Value, String> {
    let at = match (p.x, p.y) {
        (Some(x), Some(y)) => Some([x, y]),
        (None, None) => None,
        _ => return Err("give both x and y".into()),
    };
    let side = side(p.side.as_deref())?;
    // A pad onto a part's pin: the axis nearest the pad (after any turn or flip asked for).
    let onto = match &p.onto_part {
        Some(part) => {
            let pad = p.pad.clone().ok_or("onto_part needs the pad to centre")?;
            let (pcb, id) = board(world, &p.board)?;
            let ps = studio(world, p.part_studio.as_deref())?;
            let d = &doc(world)?.doc;
            let f = ops::deriving_feature(d, ps, pcb, id).ok_or("the board isn't derived into that Part Studio")?;
            let axes = ops::axes_of_part(d, ps, f, part)?;
            Some((pad, axes))
        }
        None => None,
    };
    let label = format!("Move {}", p.reference);
    let reference = p.reference.clone();
    let (angle, which) = (p.angle, p.axis);
    edit(world, &p.board, &label, move |d| {
        ops::place_footprint(d, &reference, at, angle, side)?;
        if let Some((pad, axes)) = onto {
            let i = cadrs_eda::board_edit::footprint_index(&d.board, &reference).ok_or("gone")?;
            let f = &d.board.footprints[i];
            let pp = f.footprint.pads.iter().find(|x| x.number == pad).ok_or_else(|| format!("{reference} has no pad {pad:?}"))?;
            let c = f.placement.apply(pp.at);
            let c = [cadrs_eda::units::to_mm(c.x), cadrs_eda::units::to_mm(c.y)];
            let target = match which {
                Some(k) => axes.get(k).ok_or_else(|| format!("the part has {} axes", axes.len()))?.0,
                None => axes.iter().min_by(|a, b| (a.0[0] - c[0]).hypot(a.0[1] - c[1]).total_cmp(&(b.0[0] - c[0]).hypot(b.0[1] - c[1]))).map(|a| a.0).ok_or("no axis")?,
            };
            ops::pad_onto(d, &reference, &pad, target)?;
        }
        Ok(())
    })
}

pub fn add_footprint(world: &mut World, p: &tools::AddFootprint) -> Result<Value, String> {
    let nets: Vec<(String, String)> = p.nets.iter().map(|(a, b)| (a.clone(), b.clone())).collect();
    let (from, r, v, at) = (p.copy_of.clone(), p.reference.clone(), p.value.clone(), [p.x, p.y]);
    edit(world, &p.board, &format!("Add {}", p.reference), move |d| ops::copy_footprint(d, &from, &r, &v, at, &nets).map(|_| ()))
}

pub fn remove_footprint(world: &mut World, p: &tools::RemoveFootprint) -> Result<Value, String> {
    let r = p.reference.clone();
    edit(world, &p.board, &format!("Remove {}", p.reference), move |d| ops::remove_footprint(d, &r))
}

pub fn link_outline(world: &mut World, p: &tools::LinkOutline) -> Result<Value, String> {
    let (pcb, id) = board(world, &p.board)?;
    let Some(sketch_name) = &p.sketch else {
        execute(world, &cadrs_core::pcb::outline_link::SetOutlineLink { pcb, board: id, link: None, keep_parts: false })?;
        return report(world, (pcb, id));
    };
    let ps = studio(world, p.part_studio.as_deref())?;
    let sketch = feature_id(world, ps, sketch_name)?;
    let d = &doc(world)?.doc;
    let g = d.element(ps).and_then(|e| e.feature(sketch)).and_then(|f| f.sketch()).map(|s| s.geometry.clone()).ok_or("not a sketch")?;
    let regions = crate::board_in_studio::largest_region(&g, sketch);
    if regions.is_empty() {
        return Err("the sketch has no closed region".into());
    }
    let link = cadrs_core::pcb::outline_link::SetOutlineLink { pcb, board: id, link: Some((ps, sketch, regions)), keep_parts: p.keep_parts.unwrap_or(true) };
    execute(world, &link)?;
    report(world, (pcb, id))
}
