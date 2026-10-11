//! PCB Studio boards in a Part Studio (`docs/PCB_IN_PART_STUDIO.md`): what the app adds to a
//! Derived feature of a board ([`cadrs_core::derived::DerivedBoard`]).
//!
//! - **3D model or layout** (§6): a Derived board shows its 3D model (the board with its skins
//!   and its components' models) or its **layout**: the board without its skins, the components
//!   not drawn, and the layout as the PCB Studio draws it lying on the board's top face — one
//!   mesh of coloured triangles ([`cadrs_eda::render::triangles`]) in the board's frame, drawn
//!   in paint order over the face and hidden behind other parts like any geometry. The choice
//!   is the feature's (its dialog's Show, the Feature list's menu), saved and undone like any
//!   option. [`sync_board_views`] keeps the Parts cache's hidden components and skinless boards
//!   in step; [`sync_overlays`] the layout meshes.
//! - Scenario set-ups ([`script`]): `setup` opens the mains-sensor enclosure
//!   ([`cadrs_core::samples::pcb_enclosure`]); `derive` derives its board onto Board location.
//!
//! Names: `board-layout-<board>` (a layout mesh).

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use cadrs_core::derived::{AddDerived, DerivedBoard, DerivedFeature};
use cadrs_core::external::{RefAt, SourceRef};
use cadrs_core::mate::ConnectorRef;
use cadrs_core::pcb::board3d::BoardView;
use cadrs_core::{ElementId, FeatureId, FeatureKind, PartId};
use cadrs_sketch::PlaneFrame;

use crate::parts::PartCache;
use crate::{ActiveDocument, AppState};

pub struct BoardInStudioPlugin;

impl Plugin for BoardInStudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoardEditing>()
            .init_resource::<OverlayCache>()
            .add_systems(Update, (sync_board_views, sync_overlays).chain().after(crate::parts::PartsSet).run_if(in_state(AppState::Document)));
    }
}

/// The derived board copy whose layout is being edited in the Part Studio (§7), if any: its
/// layout is drawn whatever the feature's view, and the edit mode draws its preview.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct BoardEditing {
    pub target: Option<BoardTarget>,
    /// A layout to draw instead of the stored one (a drag in progress), with its generation.
    pub preview: Option<(Box<cadrs_eda::board::Board>, u64)>,
    /// Hidden layers, the active layer, the selection and the highlighted net, as the layout
    /// tools have them, for the overlay.
    pub view: OverlayView,
}

/// A copy of a derived board in a Part Studio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardTarget {
    pub studio: ElementId,
    pub feature: FeatureId,
    pub copy: usize,
}

/// What the layout overlay shows beyond the layout (the layout tools' state).
#[derive(Debug, Clone, Default, PartialEq, Hash)]
pub struct OverlayView {
    pub hidden: Vec<cadrs_eda::layer::Layer>,
    pub active: Option<cadrs_eda::layer::Layer>,
    pub selected: Vec<uuid::Uuid>,
    pub net: Option<String>,
    pub dim: bool,
}

/// The derived boards of the active Part Studio: (feature, its board source).
fn boards_in(doc: &ActiveDocument) -> Vec<(ElementId, FeatureId, DerivedBoard)> {
    let Some(el) = doc.active_element() else { return vec![] };
    el.active_features()
        .iter()
        .filter_map(|f| match &f.kind {
            FeatureKind::Derived(d) => d.board.as_ref().map(|b| (el.id, f.id, (**b).clone())),
            _ => None,
        })
        .collect()
}

/// Keeps the Parts cache's layout-hidden components and skinless boards in step with the
/// derived boards' views (and the board being edited).
pub fn sync_board_views(doc: Option<Res<ActiveDocument>>, editing: Res<BoardEditing>, mut cache: ResMut<PartCache>) {
    let Some(doc) = doc else { return };
    let mut hidden = HashSet::new();
    let mut skinless = HashSet::new();
    for (el, f, b) in boards_in(&doc) {
        let Some(out) = cache.derived.get(&f) else { continue };
        let edited = editing.target.filter(|t| t.studio == el && t.feature == f);
        for (k, copy) in out.boards.iter().enumerate() {
            let layout = b.options.view == BoardView::Layout;
            let this_edited = edited.is_some_and(|t| t.copy == k);
            if layout || this_edited {
                skinless.insert(copy.part);
            }
            if layout {
                hidden.extend(component_parts(out, k));
            }
        }
    }
    if cache.layout_hidden != hidden || cache.skinless != skinless {
        cache.layout_hidden = hidden;
        cache.skinless = skinless;
        cache.generation += 1;
    }
}

/// The component parts of copy `k` of a derived board (they follow its board part in the
/// output, up to the next copy's board part).
pub fn component_parts(out: &cadrs_core::derived::DerivedOutput, k: usize) -> Vec<PartId> {
    let Some(start) = out.parts.iter().position(|p| out.boards.get(k).is_some_and(|b| b.part == *p)) else { return vec![] };
    out.parts[start + 1..].iter().take_while(|p| !out.boards.iter().any(|b| b.part == **p)).copied().collect()
}

/// A layout mesh in the view.
#[derive(Component)]
pub struct LayoutOverlay;

/// The layout meshes on screen, by what they show.
#[derive(Resource, Default)]
pub struct OverlayCache {
    shown: HashMap<(FeatureId, usize), (u64, Entity)>,
}

/// Height of the layout above the board's top face (mm): clear of the face, close enough to
/// read as printed on it.
pub const LIFT: f64 = 0.012;

/// The layout as triangles lying on the top face of a board copy whose frame is `frame` (its
/// bottom face; `thickness` mm thick), in Part Studio coordinates.
pub fn overlay_mesh(layout: &cadrs_eda::board::Board, view: &OverlayView, frame: &PlaneFrame, thickness: f64) -> Option<Mesh> {
    let mut v = cadrs_eda::render::BoardView::all(layout);
    v.visible.retain(|l| !view.hidden.contains(l));
    v.dim_inactive = view.dim;
    v.highlight_net = view.net.clone();
    if let Some(a) = view.active {
        v.active = a;
    }
    v.selected = view.selected.clone();
    let list = cadrs_eda::render::board(layout, &cadrs_eda::render::BoardTheme::default(), &v);
    let tris = cadrs_eda::render::triangles(&list, cadrs_eda::units::mm(0.03));
    if tris.is_empty() {
        return None;
    }
    let n = frame.normal();
    let z = thickness + LIFT;
    let at = |x: f32, y: f32| {
        let (x, y) = (x as f64, y as f64);
        [
            (frame.origin[0] + frame.u[0] * x + frame.v[0] * y + n[0] * z) as f32,
            (frame.origin[1] + frame.u[1] * x + frame.v[1] * y + n[1] * z) as f32,
            (frame.origin[2] + frame.u[2] * x + frame.v[2] * y + n[2] * z) as f32,
        ]
    };
    let positions: Vec<[f32; 3]> = tris.iter().map(|(p, _)| at(p[0], p[1])).collect();
    let colors: Vec<[f32; 4]> = tris.iter().map(|(_, c)| Color::srgba_u8(c[0], c[1], c[2], c[3]).to_linear().to_f32_array()).collect();
    let count = positions.len();
    let normal = [n[0] as f32, n[1] as f32, n[2] as f32];
    Some(
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![normal; count])
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
            .with_inserted_indices(Indices::U32((0..count as u32).collect())),
    )
}

/// Keeps a layout mesh on every derived board copy shown as its layout, and on the one being
/// edited.
pub fn sync_overlays(world: &mut World) {
    let Some(doc) = world.get_resource::<ActiveDocument>() else { return };
    let boards = boards_in(doc);
    let editing = world.resource::<BoardEditing>().clone();
    let cache = world.resource::<PartCache>();
    // (key, signature, layout, frame, thickness, name)
    type Want = ((FeatureId, usize), u64, cadrs_eda::board::Board, PlaneFrame, f64, String);
    let mut want: Vec<Want> = vec![];
    for (el, f, b) in &boards {
        let Some(out) = cache.derived.get(f) else { continue };
        for (k, copy) in out.boards.iter().enumerate() {
            let edited = editing.target == Some(BoardTarget { studio: *el, feature: *f, copy: k });
            if !(b.options.view == BoardView::Layout || edited) || cache.is_hidden(copy.part) {
                continue;
            }
            let layout = match (&editing.preview, edited) {
                (Some((p, _)), true) => (**p).clone(),
                _ => b.layout.clone(),
            };
            let mut h = std::collections::hash_map::DefaultHasher::new();
            b.hash.hash(&mut h);
            if edited {
                editing.preview.as_ref().map(|p| p.1).hash(&mut h);
                editing.view.hash(&mut h);
            }
            format!("{:?}{}", copy.frame, copy.thickness).hash(&mut h);
            want.push(((*f, k), h.finish(), layout, copy.frame, copy.thickness, b.board_name.clone()));
        }
    }
    let keys: HashSet<(FeatureId, usize)> = want.iter().map(|w| w.0).collect();
    let stale: Vec<((FeatureId, usize), Entity)> = world.resource::<OverlayCache>().shown.iter().filter(|(k, _)| !keys.contains(k)).map(|(k, (_, e))| (*k, *e)).collect();
    for (k, e) in stale {
        if let Ok(ec) = world.get_entity_mut(e) {
            ec.despawn();
        }
        world.resource_mut::<OverlayCache>().shown.remove(&k);
    }
    for (key, sig, layout, frame, thickness, name) in want {
        if world.resource::<OverlayCache>().shown.get(&key).is_some_and(|(s, _)| *s == sig) {
            continue;
        }
        if let Some((_, e)) = world.resource_mut::<OverlayCache>().shown.remove(&key)
            && let Ok(ec) = world.get_entity_mut(e)
        {
            ec.despawn();
        }
        let Some(mesh) = overlay_mesh(&layout, &editing_view(&editing, key), &frame, thickness) else { continue };
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            // Over the board's own face.
            depth_bias: 200.0,
            ..default()
        });
        let e = world.spawn((Name::new(format!("board-layout-{}", crate::pcb::slug(&name))), LayoutOverlay, Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, DespawnOnExit(AppState::Document))).id();
        world.resource_mut::<OverlayCache>().shown.insert(key, (sig, e));
    }
}

fn editing_view(editing: &BoardEditing, key: (FeatureId, usize)) -> OverlayView {
    match editing.target {
        Some(t) if t.feature == key.0 && t.copy == key.1 => editing.view.clone(),
        _ => OverlayView::default(),
    }
}

/// Sets a derived board's view (the Feature list's menu): one undo step.
pub fn set_view(world: &mut World, feature: FeatureId, view: BoardView) {
    let Some(el) = world.resource::<ActiveDocument>().active_element().map(|e| e.id) else { return };
    let Some(mut d) = crate::derived_ui::derived_in_active(world, feature) else { return };
    let Some(b) = d.board.as_mut() else { return };
    if b.options.view == view {
        return;
    }
    b.options.view = view;
    let label = match view {
        BoardView::Layout => "Show board layout",
        BoardView::ThreeD => "Show board 3D model",
    };
    if let Err(e) = world.resource_mut::<ActiveDocument>().execute(&cadrs_core::derived::SetDerived { element: el, feature, derived: d, links: vec![], label: label.into() }) {
        crate::linked::error_toast(world, e.to_string());
    }
}

// ---------------------------------------------------------------------------------------------
// Scenario set-ups

/// The Derived feature `derive` makes.
pub const DERIVED: FeatureId = FeatureId::from_u128(0x7e00_0000_0000_0000_0000_0000_0000_0e01);

/// `setup`: the mains-sensor enclosure, stored and open on its Part Studio. `derive`: its
/// board derived onto Board location. `layout` / `3d`: that board's view. `transparent a, b`:
/// the parts named so see-through.
pub fn script(world: &mut World, arg: &str) {
    use cadrs_core::samples::pcb_enclosure as pe;
    match arg.trim() {
        "setup" => {
            let Ok(doc) = pe::document() else { return };
            let store = world.resource::<crate::DocumentStore>().0.clone();
            let now = world.resource::<crate::AppClock>().now();
            let user = world.resource::<crate::UserProfile>().id.clone();
            let meta = cadrs_core::DocumentMeta::new(&user, now - 600);
            if let Err(e) = store.create(&doc, &meta) {
                warn!("board setup: {e}");
            }
            let mut d = ActiveDocument::stored(doc, meta);
            d.set_active(pe::STUDIO);
            world.insert_resource(d);
        }
        "derive" => {
            let doc = world.resource::<ActiveDocument>().doc.clone();
            let Some(board) = doc.element(pe::PCB).and_then(|e| e.pcb()).and_then(|s| s.boards.first()).map(|b| b.id) else { return };
            let d = DerivedFeature { locations: vec![ConnectorRef::Feature(pe::BOARD_LOCATION)], board: Some(Box::new(DerivedBoard::new(board, "", Default::default()))), ..Default::default() };
            let r = SourceRef { document: None, at: RefAt::Workspace, element: pe::PCB, pinned: false };
            let got = {
                let mut res = crate::linked::resolver(world);
                cadrs_core::derived::resolve(&mut res.0, &doc, None, d, r)
            };
            match got {
                Ok(g) => {
                    if let Err(e) = world.resource_mut::<ActiveDocument>().execute(&AddDerived { element: pe::STUDIO, feature: DERIVED, derived: g.derived, links: g.links }) {
                        warn!("board derive: {e}");
                    }
                }
                Err(e) => warn!("board derive: {e}"),
            }
        }
        // `transparent Tube, End plate`: those parts see-through (Make transparent).
        a if a.starts_with("transparent ") => {
            let names: Vec<String> = a["transparent ".len()..].split(',').map(|n| n.trim().to_string()).collect();
            let mut cache = world.resource_mut::<PartCache>();
            let ids: Vec<PartId> = cache.parts.iter().filter(|p| names.iter().any(|n| *n == cadrs_core::parts::display_name(p, &cache.props))).map(|p| p.id).collect();
            cache.transparent.extend(ids);
            cache.generation += 1;
        }
        "layout" => set_view(world, DERIVED, BoardView::Layout),
        "3d" => set_view(world, DERIVED, BoardView::ThreeD),
        other => warn!("board: unknown set-up {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_overlay_lies_on_the_top_face_in_the_board_frame() {
        let lib = cadrs_eda::library::LibraryTable::builtin();
        let layout = cadrs_eda::ac_board::board(&lib);
        // A board frame standing up: x along world y, y along world z, the normal along +x.
        let frame = PlaneFrame { origin: [10.0, 0.0, 0.0], u: [0.0, 1.0, 0.0], v: [0.0, 0.0, 1.0] };
        let m = overlay_mesh(&layout, &OverlayView::default(), &frame, 1.6).unwrap();
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(p)) = m.attribute(Mesh::ATTRIBUTE_POSITION) else { panic!() };
        assert!(p.iter().all(|q| (q[0] - (10.0 + 1.6 + LIFT as f32)).abs() < 1e-4), "on the top face");
        let (ymin, ymax) = p.iter().fold((f32::MAX, f32::MIN), |(a, b), q| (a.min(q[1]), b.max(q[1])));
        assert!(ymin > -0.6 && ymax < cadrs_eda::ac_board::WIDTH as f32 + 0.6, "{ymin} {ymax}");
    }
}
