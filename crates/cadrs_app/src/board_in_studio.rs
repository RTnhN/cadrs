//! PCB Studio boards in a Part Studio (`docs/PCB_IN_PART_STUDIO.md`): what the app adds to a
//! Derived feature of a board ([`cadrs_core::derived::DerivedBoard`]).
//!
//! - **3D model or layout** (§6): a Derived board shows its 3D model (the board with its skins
//!   and its components' models) or its **layout**: the board without its skins, the components
//!   not drawn, and the layout as the PCB Studio draws it as a picture on the board's top face
//!   (its bottom layers on its bottom face, [`overlay_mesh`]), hidden behind other parts like
//!   any geometry. The choice
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
            .add_observer(on_edit_bar)
            .add_systems(Update, navigate3d.before(crate::eda::navigate).run_if(in_state(AppState::Document)))
            .add_systems(Update, (end_edit_when_gone, sync_board_views, sync_drag_ghosts, sync_overlays, sync_edit_bar, edit_keys).chain().after(crate::parts::PartsSet).run_if(in_state(AppState::Document)))
            .add_systems(OnExit(AppState::Document), |mut e: ResMut<BoardEditing>| e.target = None);
    }
}

/// The derived board copy whose layout is being edited in the Part Studio (§7), if any: its
/// layout is drawn whatever the feature's view, with the Layout tools' preview (a drag in
/// progress) and their selection, layers and highlighted net ([`crate::eda::Preview`],
/// [`crate::eda::SceneInputs`]).
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct BoardEditing {
    pub target: Option<BoardTarget>,
}

/// A copy of a derived board in a Part Studio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardTarget {
    pub studio: ElementId,
    pub feature: FeatureId,
    pub copy: usize,
}

/// What the layout overlay shows beyond the layout (the Layout tools' state while editing).
#[derive(Debug, Clone, Default, PartialEq, Hash)]
pub struct OverlayView {
    pub hidden: Vec<cadrs_eda::layer::Layer>,
    pub active: Option<cadrs_eda::layer::Layer>,
    pub selected: Vec<uuid::Uuid>,
    pub net: Option<String>,
    pub dim: bool,
}

/// A derived board of the active Part Studio, without its layout: its tab and feature, its view,
/// its layout's hash and its name.
#[derive(Clone, Debug)]
struct BoardRef {
    studio: ElementId,
    feature: FeatureId,
    view: BoardView,
    hash: u64,
    name: String,
    off_board: bool,
}

/// The derived boards of the active Part Studio (no layout copied: this runs every frame).
fn boards_in(doc: &ActiveDocument) -> Vec<BoardRef> {
    let Some(el) = doc.active_element() else { return vec![] };
    el.features()
        .iter()
        .filter_map(|f| match &f.kind {
            FeatureKind::Derived(d) => d.board.as_ref().map(|b| BoardRef { studio: el.id, feature: f.id, view: b.options.view, hash: b.hash, name: b.board_name.clone(), off_board: b.options.off_board }),
            _ => None,
        })
        .collect()
}

/// The layout of the derived board `feature` of the active Part Studio.
fn layout_of(doc: &ActiveDocument, feature: FeatureId) -> Option<&cadrs_eda::board::Board> {
    match &doc.active_element()?.feature(feature)?.kind {
        FeatureKind::Derived(d) => d.board.as_ref().map(|b| &b.layout),
        _ => None,
    }
}

/// Keeps the Parts cache's layout-hidden components and skinless boards in step with the
/// derived boards' views (and the board being edited).
pub fn sync_board_views(doc: Option<Res<ActiveDocument>>, editing: Res<BoardEditing>, mut cache: ResMut<PartCache>) {
    let Some(doc) = doc else { return };
    let mut hidden = HashSet::new();
    let mut skinless = HashSet::new();
    for b in boards_in(&doc) {
        let Some(out) = cache.derived.get(&b.feature) else { continue };
        let edited = editing.target.filter(|t| t.studio == b.studio && t.feature == b.feature);
        for (k, copy) in out.boards.iter().enumerate() {
            let layout = b.view == BoardView::Layout;
            if layout || edited.is_some_and(|t| t.copy == k) {
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
    /// By (feature, copy, top face).
    shown: HashMap<(FeatureId, usize, bool), (u64, Entity)>,
}

/// Height of the layout above the board's top face (mm): clear of the face, close enough to
/// read as printed on it.
pub const LIFT: f64 = 0.012;

/// Pixels per mm of a layout overlay (fine enough for 0.8 mm silkscreen text).
pub const OVERLAY_PX_PER_MM: f32 = 32.0;

/// The layout drawn on one face of a board copy whose frame is `frame` (its bottom face;
/// `thickness` mm thick), in Part Studio coordinates: a picture of the layout as the PCB Studio
/// draws it ([`cadrs_eda::skin::raster`]) on one rectangle — on the top face the top layers and
/// those of no side (the outline, drawings, inner copper), on the bottom face the bottom ones,
/// in layout coordinates (so seen from below they read as the bottom does). Only the board's
/// area unless `off_board` (then everything, clear where nothing is drawn). The mesh and its
/// picture.
pub fn overlay_mesh(layout: &cadrs_eda::board::Board, view: &OverlayView, frame: &PlaneFrame, thickness: f64, side: cadrs_eda::layer::Side, off_board: bool) -> Option<(Mesh, Image)> {
    use cadrs_eda::layer::{Layer, Side};
    let mut v = cadrs_eda::render::BoardView::all(layout);
    let other = match side {
        Side::Top => Side::Bottom,
        Side::Bottom => Side::Top,
    };
    v.visible.retain(|l| !view.hidden.contains(l) && l.side() != Some(other) && (side == Side::Top || l.side() == Some(side) || *l == Layer::Outline));
    v.dim_inactive = view.dim;
    v.highlight_net = view.net.clone();
    if let Some(a) = view.active {
        v.active = a;
    }
    v.selected = view.selected.clone();
    let list = cadrs_eda::render::board(layout, &cadrs_eda::render::BoardTheme::default(), &v);
    let region = cadrs_eda::outline::board_region(layout);
    let of = |r: &cadrs_eda::poly::Region| r.iter().flatten().fold(None, |acc, p| Some(cadrs_eda::units::Bounds::union(acc, cadrs_eda::units::Bounds::of(*p))));
    let (bounds, clip) = if off_board { (list.bounds.or_else(|| of(&region))?, None) } else { (of(&region)?, Some(&region)) };
    let skin = cadrs_eda::skin::raster(&list, bounds, OVERLAY_PX_PER_MM, 4096, clip)?;
    let n = frame.normal();
    let z = match side {
        Side::Top => thickness + LIFT,
        Side::Bottom => -LIFT,
    };
    let at = |x: f64, y: f64| {
        [
            (frame.origin[0] + frame.u[0] * x + frame.v[0] * y + n[0] * z) as f32,
            (frame.origin[1] + frame.u[1] * x + frame.v[1] * y + n[1] * z) as f32,
            (frame.origin[2] + frame.u[2] * x + frame.v[2] * y + n[2] * z) as f32,
        ]
    };
    let mm = |v: i64| v as f64 / 1e6;
    let (x0, y0, x1, y1) = (mm(bounds.min.x), mm(bounds.min.y), mm(bounds.max.x), mm(bounds.max.y));
    let positions = vec![at(x0, y0), at(x1, y0), at(x1, y1), at(x0, y1)];
    let k = if side == Side::Top { 1.0 } else { -1.0 };
    let normal = [(n[0] * k) as f32, (n[1] * k) as f32, (n[2] * k) as f32];
    let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![normal; 4])
        // Row 0 of the picture is the board's highest y.
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]])
        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    let image = Image::new(
        bevy::render::render_resource::Extent3d { width: skin.width, height: skin.height, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2,
        skin.rgba,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    Some((mesh, image))
}

/// Keeps layout meshes (top and bottom face) on every derived board copy shown as its layout,
/// and on the one being edited; the layout is copied only when a mesh is made again.
pub fn sync_overlays(world: &mut World) {
    use cadrs_eda::layer::Side;
    let Some(doc) = world.get_resource::<ActiveDocument>() else { return };
    let boards = boards_in(doc);
    let editing = world.resource::<BoardEditing>().clone();
    let preview = world.resource::<crate::eda::Preview>();
    let (has_preview, preview_generation) = (preview.get().is_some(), preview.generation());
    let inputs = world.resource::<crate::eda::SceneInputs>().clone();
    let edit_view = OverlayView { hidden: inputs.hidden_layers.clone(), active: inputs.active_layer, selected: inputs.board_selected.clone(), net: inputs.board_net.clone(), dim: inputs.dim_inactive };
    // Not editing: the layers panel's hidden layers still hold.
    let plain_view = OverlayView { hidden: world.resource::<crate::eda::layout_tools::LayoutState>().hidden.clone(), ..Default::default() };
    let cache = world.resource::<PartCache>();
    // (key, signature, frame, thickness, name, what else it shows, edited, parts off the board)
    type Want = ((FeatureId, usize, bool), u64, PlaneFrame, f64, String, OverlayView, bool, bool);
    let mut want: Vec<Want> = vec![];
    for b in &boards {
        let Some(out) = cache.derived.get(&b.feature) else { continue };
        for (k, copy) in out.boards.iter().enumerate() {
            let edited = editing.target == Some(BoardTarget { studio: b.studio, feature: b.feature, copy: k });
            if !(b.view == BoardView::Layout || edited) || cache.is_hidden(copy.part) {
                continue;
            }
            let view = if edited { edit_view.clone() } else { plain_view.clone() };
            for top in [true, false] {
                let mut h = std::collections::hash_map::DefaultHasher::new();
                (b.hash, top).hash(&mut h);
                if edited {
                    (has_preview, preview_generation).hash(&mut h);
                }
                view.hash(&mut h);
                format!("{:?}{}", copy.frame, copy.thickness).hash(&mut h);
                b.off_board.hash(&mut h);
                want.push(((b.feature, k, top), h.finish(), copy.frame, copy.thickness, b.name.clone(), view.clone(), edited, b.off_board));
            }
        }
    }
    let keys: HashSet<(FeatureId, usize, bool)> = want.iter().map(|w| w.0).collect();
    let stale: Vec<((FeatureId, usize, bool), Entity)> = world.resource::<OverlayCache>().shown.iter().filter(|(k, _)| !keys.contains(k)).map(|(k, (_, e))| (*k, *e)).collect();
    for (k, e) in stale {
        if let Ok(ec) = world.get_entity_mut(e) {
            ec.despawn();
        }
        world.resource_mut::<OverlayCache>().shown.remove(&k);
    }
    for (key, sig, frame, thickness, name, view, edited, off_board) in want {
        if world.resource::<OverlayCache>().shown.get(&key).is_some_and(|(s, _)| *s == sig) {
            continue;
        }
        if let Some((_, e)) = world.resource_mut::<OverlayCache>().shown.remove(&key)
            && let Ok(ec) = world.get_entity_mut(e)
        {
            ec.despawn();
        }
        let side = if key.2 { Side::Top } else { Side::Bottom };
        let made = {
            let preview = world.resource::<crate::eda::Preview>().get().filter(|_| edited).map(|d| &d.board);
            let doc = world.resource::<ActiveDocument>();
            let Some(layout) = preview.or_else(|| layout_of(doc, key.0)) else { continue };
            overlay_mesh(layout, &view, &frame, thickness, side, off_board)
        };
        let Some((mesh, image)) = made else { continue };
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let image = world.resource_mut::<Assets<Image>>().add(image);
        let material = world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
            base_color_texture: Some(image),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            // Over the board's own face.
            depth_bias: 200.0,
            ..default()
        });
        let label = format!("board-layout-{}{}", crate::pcb::slug(&name), if key.2 { "" } else { "-bottom" });
        let e = world.spawn((Name::new(label), LayoutOverlay, Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, DespawnOnExit(AppState::Document))).id();
        world.resource_mut::<OverlayCache>().shown.insert(key, (sig, e));
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
// Editing a board's layout in the Part Studio (§7)

/// The board copy being edited, if it can be: the derived board's PCB Studio, board and plane
/// in this Part Studio. Only a workspace reference of this document can be edited (a version's
/// copy is read-only, as derived parts are).
pub fn edited_board(doc: &ActiveDocument, editing: &BoardEditing, cache: &PartCache) -> Option<(ElementId, cadrs_core::pcb::BoardId, crate::eda::PlaneHost)> {
    let t = editing.target?;
    let el = doc.active_element().filter(|e| e.id == t.studio)?;
    let FeatureKind::Derived(d) = &el.feature(t.feature)?.kind else { return None };
    let (r, b) = (d.source?, d.board.as_ref()?);
    if r.at != RefAt::Workspace || r.document_or(doc.doc.id) != doc.doc.id {
        return None;
    }
    doc.doc.element(r.element)?.pcb()?.board(b.board)?.design.as_ref()?;
    let copy = cache.derived.get(&t.feature)?.boards.get(t.copy)?;
    Some((r.element, b.board, crate::eda::PlaneHost { target: t, frame: copy.frame, thickness: copy.thickness }))
}

/// Whether the derived board `feature` of the active Part Studio can have its layout edited
/// here (a workspace reference of this document), and if not why.
pub fn editable(world: &World, feature: FeatureId) -> Result<(), String> {
    let doc = world.resource::<ActiveDocument>();
    let d = crate::derived_ui::derived_in_active(world, feature).ok_or("not a Derived feature")?;
    let (r, b) = (d.source.ok_or("no source")?, d.board.as_ref().ok_or("not a board")?);
    if r.at != RefAt::Workspace || r.document_or(doc.doc.id) != doc.doc.id {
        return Err(format!("{} is referenced at a version: open the linked document to edit its layout", b.board_name));
    }
    Ok(())
}

/// Starts editing the layout of copy `copy` of the derived board `feature` (the Feature list's
/// Edit layout): the Layout's tools work on it in the view, the layout drawn on its face.
pub fn begin_edit(world: &mut World, feature: FeatureId, copy: usize) {
    if let Err(e) = editable(world, feature) {
        crate::linked::error_toast(world, e);
        return;
    }
    let Some(studio) = world.resource::<ActiveDocument>().active_element().map(|e| e.id) else { return };
    world.resource_mut::<BoardEditing>().target = Some(BoardTarget { studio, feature, copy });
    crate::eda::layout_tools::set_tool(world, crate::eda::layout_tools::Tool::Select);
    world.resource_mut::<crate::eda::layout_tools::LayoutState>().selection.clear();
}

/// Stops editing (Done, Esc with nothing going on, another tab).
pub fn end_edit(world: &mut World) {
    if world.resource::<BoardEditing>().target.is_none() {
        return;
    }
    crate::eda::layout_tools::set_tool(world, crate::eda::layout_tools::Tool::Select);
    world.resource_mut::<crate::eda::layout_tools::LayoutState>().selection.clear();
    world.resource_mut::<crate::board_fit::FitState>().align = crate::board_fit::AlignStep::Off;
    world.resource_mut::<BoardEditing>().target = None;
}

/// Leaves the edit when its board is gone or another tab is shown.
fn end_edit_when_gone(world: &mut World) {
    let Some(t) = world.resource::<BoardEditing>().target else { return };
    let gone = {
        let doc = world.resource::<ActiveDocument>();
        doc.active_element().is_none_or(|e| e.id != t.studio) || crate::derived_ui::derived_in_active(world, t.feature).is_none_or(|d| d.board.is_none())
    };
    if gone {
        end_edit(world);
    }
}

/// The bar over the view while a board's layout is edited.
#[derive(Component)]
struct EditBar;

/// Keeps the edit bar on screen while editing: "Editing the layout of <board> (<studio>)",
/// Open in PCB Studio, Done.
fn sync_edit_bar(world: &mut World) {
    let want = world.resource::<crate::eda::Eda2d>().plane().map(|h| h.target);
    let title = want.and_then(|t| {
        let d = crate::derived_ui::derived_in_active(world, t.feature)?;
        let b = d.board?;
        Some(format!("Editing the layout of {} ({})", b.board_name, d.source_name))
    });
    let mut q = world.query_filtered::<Entity, With<EditBar>>();
    let existing = q.iter(world).next();
    match (title, existing) {
        (Some(title), None) => {
            let mut qa = world.query_filtered::<Entity, With<crate::viewport::ViewportArea>>();
            let Some(area) = qa.iter(world).next() else { return };
            let t = world.resource::<cadrs_ui::Theme>().clone();
            world.commands().entity(area).with_children(|vp| {
                vp.spawn((
                    Name::new("board-edit-bar"),
                    EditBar,
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(8.0),
                        left: Val::Percent(50.0),
                        margin: UiRect::left(Val::Px(-380.0)),
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        padding: UiRect::new(Val::Px(10.0), Val::Px(4.0), Val::Px(4.0), Val::Px(4.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(t.background),
                    BoxShadow::default(),
                ))
                .with_children(|b| {
                    b.spawn(cadrs_ui::icon("pcb-studio", 14.0, t.muted_foreground));
                    b.spawn((Name::new("board-edit-title"), t.text(title, t.font_base, bevy::text::FontWeight::BOLD, t.foreground)));
                    for (name, label, tip) in crate::board_fit::BUTTONS {
                        b.spawn(cadrs_ui::Button::new(name).label(label).small().outline().tooltip(tip).build(&t));
                    }
                    b.spawn(cadrs_ui::Button::new("board-edit-open").label("Open in PCB Studio").small().outline().tooltip("Show the board in its PCB Studio").build(&t));
                    b.spawn(cadrs_ui::Button::new("board-edit-done").label("Done").small().primary().tooltip("Stop editing the layout (Esc)").build(&t));
                });
            });
            world.flush();
        }
        (None, Some(e)) => {
            world.entity_mut(e).despawn();
        }
        _ => {}
    }
    sync_fit_buttons(world);
}

/// The fit buttons show their state (on: selected); Align's prompt shows under the bar.
fn sync_fit_buttons(world: &mut World) {
    let state = world.resource::<crate::board_fit::FitState>().clone();
    let mut q = world.query::<(Entity, &Name, Has<cadrs_ui::style::Selected>)>();
    let changes: Vec<(Entity, bool)> = q
        .iter(world)
        .filter(|(_, n, _)| crate::board_fit::BUTTONS.iter().any(|b| b.0 == n.as_str()))
        .filter_map(|(e, n, sel)| {
            let on = crate::board_fit::is_on(&state, n.as_str());
            (on != sel).then_some((e, on))
        })
        .collect();
    for (e, on) in changes {
        if on {
            world.entity_mut(e).insert(cadrs_ui::style::Selected);
        } else {
            world.entity_mut(e).remove::<cadrs_ui::style::Selected>();
        }
    }
    let prompt = crate::board_fit::align_prompt(&state).filter(|_| world.resource::<crate::eda::Eda2d>().plane().is_some());
    let mut q = world.query::<(Entity, &Name)>();
    let existing = q.iter(world).find(|(_, n)| n.as_str() == "board-fit-status").map(|(e, _)| e);
    match (prompt, existing) {
        (Some(text), None) => {
            let mut qa = world.query_filtered::<Entity, With<crate::viewport::ViewportArea>>();
            let Some(area) = qa.iter(world).next() else { return };
            let t = world.resource::<cadrs_ui::Theme>().clone();
            world.commands().entity(area).with_children(|vp| {
                vp.spawn((
                    Name::new("board-fit-status"),
                    Node { position_type: PositionType::Absolute, top: Val::Px(44.0), left: Val::Percent(50.0), margin: UiRect::left(Val::Px(-160.0)), padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)), border_radius: BorderRadius::all(Val::Px(4.0)), ..default() },
                    BackgroundColor(t.info_background),
                    children![t.text(text, t.font_sm, bevy::text::FontWeight::NORMAL, t.foreground)],
                ));
            });
            world.flush();
        }
        (Some(text), Some(e)) => {
            let child = world.get::<Children>(e).and_then(|c| c.first().copied());
            if let Some(c) = child
                && let Some(mut tx) = world.get_mut::<Text>(c)
                && tx.0 != text
            {
                tx.0 = text.to_string();
            }
        }
        (None, Some(e)) => {
            world.entity_mut(e).despawn();
        }
        _ => {}
    }
}

fn on_edit_bar(a: On<bevy::ui_widgets::Activate>, q: Query<&Name>, mut commands: Commands) {
    let Ok(n) = q.get(a.entity) else { return };
    match n.as_str() {
        "board-edit-done" => commands.queue(end_edit),
        "board-edit-open" => commands.queue(open_in_pcb_studio),
        _ => {}
    }
}

/// Shows the edited board in its PCB Studio (and stops editing here).
fn open_in_pcb_studio(world: &mut World) {
    let Some(host) = world.resource::<crate::eda::Eda2d>().plane().copied() else { return };
    let Some((pcb, board, _)) = world.resource::<crate::eda::Eda2d>().board() else { return };
    let _ = host;
    end_edit(world);
    world.resource_mut::<ActiveDocument>().set_active(pcb);
    crate::pcb::show_board(world, pcb, board);
    crate::eda::set_mode(world, crate::eda::Mode::Layout);
}

/// Esc with the Select tool, nothing moving and nothing selected (as of the frame before)
/// stops editing.
fn edit_keys(world: &mut World, mut idle: Local<bool>) {
    let keys: Vec<bevy::input::keyboard::KeyboardInput> = world.resource_mut::<Messages<bevy::input::keyboard::KeyboardInput>>().iter_current_update_messages().cloned().collect();
    if world.resource::<crate::eda::Eda2d>().plane().is_none() {
        *idle = false;
        return;
    }
    let esc = keys.iter().any(|k| k.state == bevy::input::ButtonState::Pressed && k.key_code == KeyCode::Escape);
    if esc && *idle && crate::eda::ui::keys_for(world, crate::eda::Mode::Layout) {
        end_edit(world);
    }
    let s = world.resource::<crate::eda::layout_tools::LayoutState>();
    *idle = matches!(s.tool, crate::eda::layout_tools::Tool::Select) && s.moving.is_none() && s.selection.is_empty();
}

/// Where a view ray meets the board face facing the camera: the layout point (nm). `None` when
/// the ray runs along the board.
pub fn ray_to_layout(host: &crate::eda::PlaneHost, origin: Vec3, dir: Vec3) -> Option<cadrs_eda::units::Pt> {
    let f = &host.frame;
    let n = f.normal();
    let n = Vec3::new(n[0] as f32, n[1] as f32, n[2] as f32);
    let (u, v) = (Vec3::new(f.u[0] as f32, f.u[1] as f32, f.u[2] as f32), Vec3::new(f.v[0] as f32, f.v[1] as f32, f.v[2] as f32));
    let o = Vec3::new(f.origin[0] as f32, f.origin[1] as f32, f.origin[2] as f32);
    // The top face seen from above, the bottom face from below.
    let z = if dir.dot(n) < 0.0 { host.thickness as f32 } else { 0.0 };
    let on = o + n * z;
    let d = dir.dot(n);
    if d.abs() < 1e-6 {
        return None;
    }
    let t = (on - origin).dot(n) / d;
    let p = origin + dir * t - o;
    Some(cadrs_eda::units::Pt::mm(p.dot(u) as f64, p.dot(v) as f64))
}

/// The component under a view ray, of the edited board copy: its footprint's place on the
/// layout, if the ray meets its model before the board face (a click on a tall part's top
/// picks it).
fn component_under(world: &World, host: &crate::eda::PlaneHost, origin: Vec3, dir: Vec3) -> Option<cadrs_eda::units::Pt> {
    let cache = world.resource::<PartCache>();
    let out = cache.derived.get(&host.target.feature)?;
    let parts = component_parts(out, host.target.copy);
    let layout = crate::derived_ui::derived_in_active(world, host.target.feature)?.board?.layout;
    let plane_t = {
        let n = host.frame.normal();
        let n = Vec3::new(n[0] as f32, n[1] as f32, n[2] as f32);
        let o = Vec3::new(host.frame.origin[0] as f32, host.frame.origin[1] as f32, host.frame.origin[2] as f32);
        let z = if dir.dot(n) < 0.0 { host.thickness as f32 } else { 0.0 };
        let d = dir.dot(n);
        if d.abs() > 1e-6 { ((o + n * z) - origin).dot(n) / d } else { f32::MAX }
    };
    let mut best: Option<(f32, uuid::Uuid)> = None;
    for pid in parts {
        let Some(part) = cache.part(pid) else { continue };
        let Some(fp) = out.footprints.iter().find(|(p, _)| *p == pid).map(|(_, f)| *f) else { continue };
        let s = &part.solid;
        let p = |i: u32| {
            let q = s.positions[i as usize];
            Vec3::new(q[0] as f32, q[1] as f32, q[2] as f32)
        };
        for t in s.indices.as_chunks::<3>().0 {
            if let Some(d) = ray_triangle(origin, dir, p(t[0]), p(t[1]), p(t[2]))
                && d < plane_t
                && best.is_none_or(|(b, _)| d < b)
            {
                best = Some((d, fp));
            }
        }
    }
    let (_, fp) = best?;
    layout.footprints.iter().find(|f| f.id == fp).map(|f| f.placement.at)
}

/// Möller–Trumbore: the distance along the ray to the triangle, if it meets it.
fn ray_triangle(o: Vec3, d: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let (e1, e2) = (b - a, c - a);
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv = 1.0 / det;
    let s = o - a;
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = d.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t > 0.0).then_some(t)
}

/// The Layout's pointer on a board in the 3D view: viewport presses, releases and moves become
/// [`crate::eda::EdaClick`]s and [`crate::eda::EdaPointer`] at the layout point under the
/// pointer (on the board face facing the camera, or a component's place when its model is
/// under the pointer), and the pick tolerance follows the zoom at the board.
#[allow(clippy::too_many_arguments)]
pub(crate) fn navigate3d(
    eda: Res<crate::eda::Eda2d>,
    rect: Res<crate::viewport::ViewportRect>,
    view: Res<crate::viewport::ViewportView>,
    hover: Res<bevy::picking::hover::HoverMap>,
    q_area: Query<Entity, With<crate::viewport::ViewportArea>>,
    mut inputs: MessageReader<bevy::picking::pointer::PointerInput>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut pointer: ResMut<crate::eda::EdaPointer>,
    mut ui: ResMut<crate::eda::EdaUi>,
    mut last_press: Local<Option<(f64, Vec2)>>,
    fit: Res<crate::board_fit::FitState>,
    mut commands: Commands,
) {
    use bevy::picking::pointer::{PointerAction, PointerButton};
    let fit_align_active = fit.align != crate::board_fit::AlignStep::Off;
    let events: Vec<_> = inputs.read().cloned().collect();
    let (Some(host), Some(key)) = (eda.plane().copied(), eda.0) else { return };
    // Pixels per mm at the board (for the tools' pick tolerance).
    let o = Vec3::new(host.frame.origin[0] as f32, host.frame.origin[1] as f32, host.frame.origin[2] as f32);
    let scale = (view.view.magnification(o) / view.view.scale.max(1e-6)) as f64;
    let size = [rect.0.width() as f64, rect.0.height() as f64];
    ui.views.insert(key, (cadrs_eda::view::View { center: [0.0, 0.0], scale, size }, false));
    let over = crate::viewport::pointer_over_viewport(&hover, &q_area);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    for input in events {
        let pos = input.location.position;
        let (origin, dir) = view.view.ray(rect.offset(pos));
        let Some(at) = ray_to_layout(&host, origin, dir) else { continue };
        pointer.screen = pos;
        pointer.at = at;
        pointer.over = over;
        match input.action {
            PointerAction::Press(PointerButton::Primary) if over => {
                // Align waits for a pick (`crate::board_fit`): the press is its.
                let offset = rect.offset(pos);
                if fit_align_active {
                    commands.queue(move |world: &mut World| {
                        crate::board_fit::align_press(world, offset);
                    });
                    continue;
                }
                let now = time.elapsed_secs_f64();
                let double = last_press.is_some_and(|(t, p)| now - t < 0.4 && p.distance(pos) < 4.0);
                *last_press = Some((now, pos));
                commands.queue(move |world: &mut World| {
                    // A component's model under the pointer: press on its footprint.
                    let at = component_under(world, &host, origin, dir).unwrap_or(at);
                    world.write_message(crate::eda::EdaClick::Press { at, shift, ctrl });
                    if double {
                        world.write_message(crate::eda::EdaClick::Double { at });
                    }
                });
                if double {
                    *last_press = None;
                }
            }
            PointerAction::Release(PointerButton::Primary) if !fit_align_active => {
                commands.queue(move |world: &mut World| {
                    world.write_message(crate::eda::EdaClick::Release { at });
                });
            }
            _ => {}
        }
    }
}

/// A component moving with a drag (§7.2): its part drawn where the Layout's preview has its
/// footprint, the part itself hidden until the rebuild after the drop has it there.
#[derive(Component)]
struct DragGhost {
    part: PartId,
}

/// What the ghosts were made for: per part, its solid (to tell the rebuilt one) and footprint.
#[derive(Default)]
struct Ghosts {
    parts: HashMap<PartId, (usize, uuid::Uuid)>,
}

/// The motion of a footprint frame onto the Part Studio for a board copy (`frame`, `thickness`).
fn footprint_world(frame: &PlaneFrame, thickness: f64, p: &cadrs_eda::footprint::FootprintPlacement) -> cadrs_kernel::Motion {
    let n = frame.normal();
    let col = |a: [f64; 3]| nalgebra::Vector3::new(a[0], a[1], a[2]);
    let board = cadrs_kernel::Motion { linear: nalgebra::Matrix3::from_columns(&[col(frame.u), col(frame.v), col(n)]), translation: col(frame.origin) };
    cadrs_core::pcb::model::footprint_motion(p, thickness).then(&board)
}

fn motion_transform(m: &cadrs_kernel::Motion) -> Transform {
    let l = m.linear;
    let mat = Mat4::from_cols(
        Vec4::new(l[(0, 0)] as f32, l[(1, 0)] as f32, l[(2, 0)] as f32, 0.0),
        Vec4::new(l[(0, 1)] as f32, l[(1, 1)] as f32, l[(2, 1)] as f32, 0.0),
        Vec4::new(l[(0, 2)] as f32, l[(1, 2)] as f32, l[(2, 2)] as f32, 0.0),
        Vec4::new(m.translation.x as f32, m.translation.y as f32, m.translation.z as f32, 1.0),
    );
    Transform::from_matrix(mat)
}

/// Keeps a ghost on each component whose footprint the Layout's preview moves, turns or flips
/// (live 3D while dragging); the ghost stays after the drop until the rebuilt part is there.
fn sync_drag_ghosts(world: &mut World, mut ghosts: Local<Ghosts>) {
    let host = world.resource::<crate::eda::Eda2d>().plane().copied();
    let preview = world.resource::<crate::eda::Preview>().get().map(|d| d.board.footprints.clone());
    // Each moved footprint's part and motion (preview placement ∘ stored placement⁻¹).
    let mut moved: Vec<(PartId, uuid::Uuid, Transform)> = vec![];
    if let (Some(h), Some(preview)) = (host, preview.as_ref())
        && let Some(layout) = crate::derived_ui::derived_in_active(world, h.target.feature).and_then(|d| d.board).map(|b| b.layout)
    {
        let cache = world.resource::<PartCache>();
        if let Some(out) = cache.derived.get(&h.target.feature) {
            let parts = component_parts(out, h.target.copy);
            for (pid, fp) in out.footprints.iter().filter(|(p, _)| parts.contains(p)) {
                let (Some(old), Some(new)) = (layout.footprints.iter().find(|f| f.id == *fp), preview.iter().find(|f| f.id == *fp)) else { continue };
                if old.placement == new.placement {
                    continue;
                }
                let m = footprint_world(&h.frame, h.thickness, &old.placement).inverse().then(&footprint_world(&h.frame, h.thickness, &new.placement));
                moved.push((*pid, *fp, motion_transform(&m)));
            }
        }
    }
    // Keep a ghost whose part isn't rebuilt yet (the drop is being rebuilt).
    let cache = world.resource::<PartCache>();
    let pending: Vec<PartId> = ghosts
        .parts
        .iter()
        .filter(|(pid, (ptr, _))| !moved.iter().any(|m| m.0 == **pid) && cache.part(**pid).is_some_and(|p| std::sync::Arc::as_ptr(&p.solid) as usize == *ptr) && host.is_some())
        .map(|(p, _)| *p)
        .collect();
    // The parts hidden for their ghosts.
    let hidden: HashSet<PartId> = moved.iter().map(|m| m.0).chain(pending.iter().copied()).collect();
    let mut q = world.query::<(Entity, &DragGhost, &mut Transform)>();
    let mut have: HashSet<PartId> = HashSet::new();
    let mut gone = vec![];
    for (e, g, mut tr) in q.iter_mut(world) {
        if let Some((_, _, t)) = moved.iter().find(|m| m.0 == g.part) {
            *tr = *t;
            have.insert(g.part);
        } else if pending.contains(&g.part) {
            have.insert(g.part);
        } else {
            gone.push((e, g.part));
        }
    }
    for (e, p) in gone {
        world.entity_mut(e).despawn();
        ghosts.parts.remove(&p);
    }
    for (pid, fp, t) in &moved {
        if have.contains(pid) {
            continue;
        }
        let Some(part) = world.resource::<PartCache>().part(*pid).cloned() else { continue };
        ghosts.parts.insert(*pid, (std::sync::Arc::as_ptr(&part.solid) as usize, *fp));
        // Drawn as the part is (its faces' colours, the parts' shading).
        let bases = world.resource::<PartCache>().bases(&part);
        let mesh = world.resource_mut::<Assets<Mesh>>().add(crate::parts::part_base_mesh(&part, false, &bases));
        let material = world.resource_mut::<Assets<crate::part_shading::PartShading>>().add(crate::part_shading::PartShading::new(false, false));
        world.spawn((Name::new(format!("board-drag-{}", crate::pcb::slug(&part.name))), DragGhost { part: *pid }, Mesh3d(mesh), MeshMaterial3d(material), *t, DespawnOnExit(AppState::Document)));
    }
    let mut cache = world.resource_mut::<PartCache>();
    if cache.drag_hidden != hidden {
        cache.drag_hidden = hidden;
        cache.generation += 1;
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
        "edit" => begin_edit(world, DERIVED, 0),
        "done" => end_edit(world),
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
        let (m, image) = overlay_mesh(&layout, &OverlayView::default(), &frame, 1.6, cadrs_eda::layer::Side::Top, false).unwrap();
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(p)) = m.attribute(Mesh::ATTRIBUTE_POSITION) else { panic!() };
        assert!(p.iter().all(|q| (q[0] - (10.0 + 1.6 + LIFT as f32)).abs() < 1e-4), "on the top face");
        // The board's box: 31 × 39 mm, from its corner at the layout's origin.
        let (ymin, ymax) = p.iter().fold((f32::MAX, f32::MIN), |(a, b), q| (a.min(q[1]), b.max(q[1])));
        assert!(ymin.abs() < 1e-4 && (ymax - cadrs_eda::ac_board::WIDTH as f32).abs() < 1e-4, "{ymin} {ymax}");
        assert_eq!(image.width(), (cadrs_eda::ac_board::WIDTH as f32 * OVERLAY_PX_PER_MM).ceil() as u32);
        // The bottom face's picture lies under the board.
        let (m, _) = overlay_mesh(&layout, &OverlayView::default(), &frame, 1.6, cadrs_eda::layer::Side::Bottom, false).unwrap();
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(p)) = m.attribute(Mesh::ATTRIBUTE_POSITION) else { panic!() };
        assert!(p.iter().all(|q| (q[0] - (10.0 - LIFT as f32)).abs() < 1e-4));
    }

    #[test]
    fn a_view_ray_meets_the_face_it_sees() {
        let frame = PlaneFrame { origin: [0.0, 0.0, 12.0], u: [1.0, 0.0, 0.0], v: [0.0, 1.0, 0.0] };
        let host = crate::eda::PlaneHost { target: BoardTarget { studio: ElementId::new(), feature: FeatureId::new(), copy: 0 }, frame, thickness: 1.6 };
        // From above, slanting (as a perspective ray does): it meets the top face (z 13.6).
        let o = Vec3::new(10.0, -20.0, 113.6);
        let d = (Vec3::new(4.0, 6.0, 13.6) - o).normalize();
        let p = ray_to_layout(&host, o, d).unwrap();
        assert!((p.x as f64 / 1e6 - 4.0).abs() < 1e-3 && (p.y as f64 / 1e6 - 6.0).abs() < 1e-3, "{p:?}");
        // From below: the bottom face (z 12).
        let o = Vec3::new(0.0, 0.0, -50.0);
        let d = (Vec3::new(-3.0, 2.0, 12.0) - o).normalize();
        let p = ray_to_layout(&host, o, d).unwrap();
        assert!((p.x as f64 / 1e6 + 3.0).abs() < 1e-3 && (p.y as f64 / 1e6 - 2.0).abs() < 1e-3, "{p:?}");
        // Along the board: nowhere.
        assert!(ray_to_layout(&host, Vec3::new(0.0, 0.0, 13.0), Vec3::X).is_none());
    }
}
