//! The sketch Linear and Circular pattern tools (Onshape's sketch *Linear pattern* and *Circular
//! pattern*): copies of sketch curves, linked to their seeds ([`cadrs_sketch::pattern`]).
//!
//! Choosing a tool (Pattern ▾) opens its dialog with the selected curves as the entities to
//! pattern; picking curves while the Entities field is active changes them.
//! - **Linear pattern:** each direction runs along a sketch line or axis picked into its
//!   Direction field (the copies follow the line as it turns), or, with none, at its Angle. It
//!   has a distance and an instance count (the seed included); a second direction makes a grid.
//! - **Circular pattern:** the instances turn about the point (or circle's or arc's center)
//!   picked into its Center field, the sketch origin until one is; a count and the angle they
//!   span (360°: evenly round).
//!
//! Clicking a Direction or Center field makes it take the next pick, after which the Entities
//! field takes picks again. The copies are previewed while the dialog is open. ✓ or Enter (also
//! in a field) makes them ([`SketchOp::LinearPattern`], [`SketchOp::CircularPattern`]); ✕ or
//! Esc drops them. Right-clicking a seed or a copy offers **Edit linear pattern** (or circular),
//! which reopens the dialog with the pattern's values ([`SketchOp::EditPattern`]: copies are
//! added or removed to match).
//!
//! Names: `sketch-pattern-dialog`, `sketch-pattern-entities`; linear:
//! `sketch-pattern-direction-1`, `sketch-pattern-distance-1`, `sketch-pattern-count-1`,
//! `sketch-pattern-angle-1` (and `-2` for the second direction, in
//! `sketch-pattern-direction-2-rows`), `sketch-pattern-second`; circular:
//! `sketch-pattern-center`, `sketch-pattern-count`, `sketch-pattern-angle`; the menu
//! `sketch-pattern-menu`.

use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use cadrs_sketch::pattern::{
    Circular, Layout, PatternDir, check, line_dir, pattern_of_curve, xforms,
};
use cadrs_sketch::units::Quantity;
use cadrs_sketch::{
    CurveId, CurveKind, CurveRef, PatternId, PointRef, SketchEntity, SketchOp, Vec2 as SVec2,
};
use cadrs_ui::i18n::{tr, tr_with};
use cadrs_ui::menu::{Menu, MenuAction, MenuItem};
use cadrs_ui::{
    CheckboxChange, FeatureDialog, FeatureDialogAccept, FeatureDialogCancel, FeatureDialogState,
    NumberField, NumberFieldCommit, NumberFieldState, OptionRow, SelectionField,
    SelectionFieldActivate, SelectionFieldClear, SelectionFieldState, Theme,
};

use crate::sketch::{ActiveSketchTool, PartStudioMode, SketchSession, SketchTool};
use crate::sketch_draw::SketchRubberGizmos;
use crate::sketch_tools::{SketchSelection, world_sketch};
use crate::viewport::ViewportArea;
use crate::{ActiveDocument, AppState};

pub struct SketchPatternPlugin;

impl Plugin for SketchPatternPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PatternEditing>()
            .add_observer(on_accept)
            .add_observer(on_cancel)
            .add_observer(on_number)
            .add_observer(on_second)
            .add_observer(on_activate)
            .add_observer(on_clear)
            .add_observer(on_pattern_menu)
            .add_systems(
                Update,
                (
                    start_from_tool,
                    sync_dialog,
                    draw_preview.after(crate::sketch_draw::SketchDrawSet),
                )
                    .chain()
                    .run_if(in_state(AppState::Document)),
            )
            .add_systems(OnExit(PartStudioMode::Sketching), close_on_exit);
    }
}

/// Which pattern a dialog makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternKind {
    Linear,
    Circular,
}

impl PatternKind {
    fn of(layout: &Layout) -> Self {
        match layout {
            Layout::Linear(_) => PatternKind::Linear,
            Layout::Circular(_) => PatternKind::Circular,
        }
    }

    fn title(self) -> String {
        tr(match self {
            PatternKind::Linear => "sketch-toolbar-linear-pattern",
            PatternKind::Circular => "shell-tool-circular-pattern",
        })
    }
}

/// What the open dialog makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternMode {
    /// A new pattern of the selected curves.
    New(PatternKind),
    /// Edit linear (or circular) pattern.
    Edit(PatternId),
}

/// Which field takes the next pick in the sketch.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    Entities,
    /// A linear pattern's direction (0 or 1): a sketch line or axis.
    Direction(usize),
    /// A circular pattern's center: a point, the origin, or a circle's or arc's center.
    Center,
}

/// The open pattern dialog's values.
#[derive(Debug, Clone, PartialEq)]
pub struct PatternDialog {
    pub entity: Entity,
    pub mode: PatternMode,
    pub kind: PatternKind,
    /// The curves patterned (the selection, for a new pattern).
    pub seeds: Vec<CurveId>,
    pub dirs: [PatternDir; 2],
    /// The second direction is on.
    pub second: bool,
    pub circular: Circular,
    pub picking: Pick,
}

impl PatternDialog {
    /// The layout the dialog makes.
    pub fn layout(&self) -> Layout {
        match self.kind {
            PatternKind::Linear if self.second => Layout::Linear(self.dirs.to_vec()),
            PatternKind::Linear => Layout::Linear(vec![self.dirs[0]]),
            PatternKind::Circular => Layout::Circular(self.circular),
        }
    }
}

/// The open pattern dialog.
#[derive(Resource, Debug, Default)]
pub struct PatternEditing {
    pub open: Option<PatternDialog>,
    /// The frame the dialog closed in: its Enter or Esc is not the sketch's.
    pub closed_frame: Option<u32>,
}

/// Marks the dialog's wrapper.
#[derive(Component)]
struct PatternDialogRoot;

/// A number field of the dialog: a direction's (0 or 1) distance, count or angle, or a
/// circular pattern's count or angle.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Distance(usize),
    Count(usize),
    Angle(usize),
    TurnCount,
    TurnAngle,
}

impl Field {
    fn quantity(self) -> Quantity {
        match self {
            Field::Distance(_) => Quantity::Length,
            Field::Count(_) | Field::TurnCount => Quantity::Count,
            Field::Angle(_) | Field::TurnAngle => Quantity::Angle,
        }
    }

    /// The field's value in the dialog, as shown (degrees for an angle).
    fn value(self, d: &PatternDialog) -> f64 {
        match self {
            Field::Distance(i) => d.dirs[i].spacing,
            Field::Count(i) => d.dirs[i].count as f64,
            Field::Angle(i) => d.dirs[i].angle.to_degrees(),
            Field::TurnCount => d.circular.count as f64,
            Field::TurnAngle => d.circular.angle.to_degrees(),
        }
    }

    fn name(self) -> String {
        match self {
            Field::Distance(i) => format!("sketch-pattern-distance-{}", i + 1),
            Field::Count(i) => format!("sketch-pattern-count-{}", i + 1),
            Field::Angle(i) => format!("sketch-pattern-angle-{}", i + 1),
            Field::TurnCount => "sketch-pattern-count".into(),
            Field::TurnAngle => "sketch-pattern-angle".into(),
        }
    }

    fn label(self) -> String {
        tr(match self {
            Field::Distance(_) => "sketch-pattern-distance",
            Field::Count(_) | Field::TurnCount => "sketch-pattern-instances",
            Field::Angle(_) | Field::TurnAngle => "sketch-pattern-angle",
        })
    }
}

/// The second direction's rows.
#[derive(Component)]
struct SecondRows;

/// A direction's Angle row: hidden while the direction runs along a line.
#[derive(Component)]
struct AngleRow(usize);

/// The curves of a selection.
fn curves_of(selection: &[SketchEntity]) -> Vec<CurveId> {
    selection
        .iter()
        .filter_map(|e| match *e {
            SketchEntity::Curve(c) => Some(c),
            _ => None,
        })
        .collect()
}

/// A round distance a little more than `size` (the seeds' width), so the first copies sit
/// clear of them.
fn default_spacing(size: f64) -> f64 {
    if size <= 1e-6 {
        return 10.0;
    }
    let v = size * 1.5;
    let step = 10f64.powf(v.log10().floor() - 1.0).max(0.01);
    (v / step).ceil() * step
}

/// The seeds' bounding box size (width, height).
fn seeds_size(sketch: &cadrs_sketch::Sketch, seeds: &[CurveId]) -> Option<SVec2> {
    let mut lo = SVec2::new(f64::INFINITY, f64::INFINITY);
    let mut hi = SVec2::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
    for &c in seeds {
        for p in cadrs_sketch::hit::curve_polyline(sketch, c) {
            lo = SVec2::new(lo.x.min(p.x), lo.y.min(p.y));
            hi = SVec2::new(hi.x.max(p.x), hi.y.max(p.y));
        }
    }
    lo.x.is_finite().then(|| hi - lo)
}

/// The entities field's text: how many curves are patterned.
fn entities_text(n: usize) -> Option<String> {
    let key = if n == 1 {
        "sketch-pattern-entity-count"
    } else {
        "sketch-pattern-entities-count"
    };
    (n > 0).then(|| tr_with(key, &[("count", &n.to_string())]))
}

/// A Direction field's text: what the direction runs along (none: its angle).
fn along_text(along: Option<CurveRef>) -> Option<String> {
    along.map(|l| {
        tr(match l {
            CurveRef::XAxis => "sketch-pattern-x-axis",
            CurveRef::YAxis => "sketch-pattern-y-axis",
            CurveRef::Curve(_) => "sketch-pattern-line",
        })
    })
}

/// The Center field's text.
fn center_text(c: PointRef) -> Option<String> {
    Some(tr(match c {
        PointRef::Origin => "sketch-pattern-origin",
        PointRef::Point(_) => "sketch-pattern-point",
    }))
}

/// What a pick in the sketch gives a Direction field: a line or an axis.
fn as_direction(sketch: &cadrs_sketch::Sketch, e: SketchEntity) -> Option<CurveRef> {
    let r = match e {
        SketchEntity::Curve(c) => match sketch.curves.get(c)?.kind {
            CurveKind::Line { .. } => CurveRef::Curve(c),
            _ => return None,
        },
        SketchEntity::XAxis => CurveRef::XAxis,
        SketchEntity::YAxis => CurveRef::YAxis,
        _ => return None,
    };
    line_dir(sketch, r).map(|_| r)
}

/// What a pick in the sketch gives the Center field: a point, the origin, or a circle's or
/// arc's center.
fn as_center(sketch: &cadrs_sketch::Sketch, e: SketchEntity) -> Option<PointRef> {
    match e {
        SketchEntity::Point(p) => Some(PointRef::Point(p)),
        SketchEntity::Origin => Some(PointRef::Origin),
        SketchEntity::Curve(c) => match sketch.curves.get(c)?.kind {
            CurveKind::Circle { center, .. }
            | CurveKind::Arc { center, .. }
            | CurveKind::Ellipse { center, .. }
            | CurveKind::EllipseArc { center, .. } => Some(PointRef::Point(center)),
            _ => None,
        },
        _ => None,
    }
}

/// Opens a pattern dialog (closing one that is open).
pub fn open_pattern_dialog(world: &mut World, mode: PatternMode) {
    close_dialog(world);
    let Some(sketch) = world_sketch(world).cloned() else {
        return;
    };
    let selection = curves_of(&world.resource::<SketchSelection>().0);
    let size = seeds_size(&sketch, &selection).unwrap_or_default();
    let mut d = PatternDialog {
        entity: Entity::PLACEHOLDER,
        mode,
        kind: PatternKind::Linear,
        seeds: selection,
        dirs: [
            PatternDir::at_angle(0.0, default_spacing(size.x), 2),
            PatternDir::at_angle(90.0, default_spacing(size.y), 2),
        ],
        second: false,
        circular: Circular {
            center: PointRef::Origin,
            count: 4,
            angle: std::f64::consts::TAU,
        },
        picking: Pick::Entities,
    };
    match mode {
        PatternMode::New(kind) => d.kind = kind,
        PatternMode::Edit(id) => {
            let Some(p) = sketch.patterns.get(id) else {
                return;
            };
            d.kind = PatternKind::of(&p.layout);
            d.seeds = p.seeds.clone();
            match &p.layout {
                Layout::Linear(dirs) => {
                    d.dirs[0] = dirs[0];
                    match dirs.get(1) {
                        Some(b) => {
                            d.dirs[1] = *b;
                            d.second = true;
                        }
                        None => {
                            d.dirs[1] = PatternDir::at_angle(
                                dirs[0].angle.to_degrees() + 90.0,
                                dirs[0].spacing,
                                2,
                            )
                        }
                    }
                }
                Layout::Circular(c) => d.circular = *c,
            }
        }
    }
    let theme = world.resource::<Theme>().clone();
    let units = world.resource::<crate::WorkspaceUnits>().0;
    let Some(area) = world
        .query_filtered::<Entity, With<ViewportArea>>()
        .iter(world)
        .next()
    else {
        return;
    };
    let t = theme.clone();
    let st = d.clone();
    let number = move |p: &mut ChildSpawner, f: Field, st: &PatternDialog| {
        let mut e = p.spawn((
            NumberField::new(f.name(), f.label())
                .text(units.with_unit(f.value(st), f.quantity()))
                .label_width(72.0)
                .build(&t),
            f,
        ));
        if let Field::Angle(i) = f {
            e.insert(AngleRow(i));
        }
    };
    let t2 = theme.clone();
    let pick_field = move |p: &mut ChildSpawner,
                           name: String,
                           placeholder: String,
                           value: Option<String>,
                           pick| {
        p.spawn((
            SelectionField::new(name)
                .placeholder(placeholder)
                .value(value)
                .active(false)
                .build(&t2),
            pick,
        ));
    };
    let t3 = theme.clone();
    let dialog = FeatureDialog::new("sketch-pattern-dialog")
        .title(d.kind.title())
        .valid(!d.seeds.is_empty())
        .body(move |p: &mut ChildSpawner| {
            pick_field(
                p,
                "sketch-pattern-entities".into(),
                tr("sketch-pattern-entities"),
                entities_text(st.seeds.len()),
                Pick::Entities,
            );
            match st.kind {
                PatternKind::Linear => {
                    let direction = |p: &mut ChildSpawner, i: usize| {
                        pick_field(
                            p,
                            format!("sketch-pattern-direction-{}", i + 1),
                            tr("sketch-pattern-direction"),
                            along_text(st.dirs[i].along),
                            Pick::Direction(i),
                        );
                        for f in [Field::Distance(i), Field::Count(i), Field::Angle(i)] {
                            number(p, f, &st);
                        }
                    };
                    direction(p, 0);
                    p.spawn(
                        OptionRow::new(
                            "sketch-pattern-second",
                            tr("sketch-pattern-second-direction"),
                        )
                        .checked(st.second)
                        .build(&t3),
                    );
                    p.spawn((
                        Name::new("sketch-pattern-direction-2-rows"),
                        SecondRows,
                        Node {
                            flex_direction: FlexDirection::Column,
                            display: if st.second {
                                Display::Flex
                            } else {
                                Display::None
                            },
                            ..default()
                        },
                    ))
                    .with_children(|r| direction(r, 1));
                }
                PatternKind::Circular => {
                    pick_field(
                        p,
                        "sketch-pattern-center".into(),
                        tr("sketch-pattern-center"),
                        center_text(st.circular.center),
                        Pick::Center,
                    );
                    for f in [Field::TurnCount, Field::TurnAngle] {
                        number(p, f, &st);
                    }
                }
            }
        })
        .build(&theme);
    // Beside the sketch dialog, as the Text dialog is.
    let wrapper = world
        .spawn((
            Name::new("sketch-pattern-dialog-host"),
            PatternDialogRoot,
            DespawnOnExit(AppState::Document),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(250.0),
                top: Val::Px(0.0),
                ..default()
            },
        ))
        .id();
    d.entity = world.spawn(dialog).id();
    world.entity_mut(wrapper).add_child(d.entity);
    world.entity_mut(area).add_child(wrapper);
    world.resource_mut::<PatternEditing>().open = Some(d);
}

/// Closes the dialog without changing the sketch.
pub fn close_dialog(world: &mut World) {
    let frame = world.resource::<bevy::diagnostic::FrameCount>().0;
    let mut editing = world.resource_mut::<PatternEditing>();
    if editing.open.take().is_some() {
        editing.closed_frame = Some(frame);
    }
    let roots: Vec<Entity> = world
        .query_filtered::<Entity, With<PatternDialogRoot>>()
        .iter(world)
        .collect();
    for e in roots {
        world.entity_mut(e).despawn();
    }
}

fn close_on_exit(mut commands: Commands) {
    commands.queue(close_dialog);
}

/// The edit the open dialog makes, if it can make one.
fn op_of(sketch: &cadrs_sketch::Sketch, d: &PatternDialog) -> Option<SketchOp> {
    let layout = d.layout();
    if d.seeds.is_empty() || check(sketch, &layout).is_err() {
        return None;
    }
    Some(match (d.mode, layout) {
        (PatternMode::Edit(id), layout) => SketchOp::EditPattern { id, layout },
        (PatternMode::New(_), Layout::Linear(dirs)) => SketchOp::LinearPattern {
            curves: d.seeds.clone(),
            dirs,
        },
        (PatternMode::New(_), Layout::Circular(circular)) => SketchOp::CircularPattern {
            curves: d.seeds.clone(),
            circular,
        },
    })
}

/// ✓ / Enter: makes or changes the pattern.
pub fn accept_open(world: &mut World) {
    let Some(d) = world.resource::<PatternEditing>().open.clone() else {
        return;
    };
    let Some(op) = world_sketch(world).and_then(|s| op_of(s, &d)) else {
        return;
    };
    close_dialog(world);
    world.resource_mut::<SketchSelection>().0.clear();
    let Some(s) = world.get_resource::<SketchSession>().cloned() else {
        return;
    };
    if let Some(mut doc) = world.get_resource_mut::<ActiveDocument>()
        && let Err(e) = doc.execute(&cadrs_core::commands::EditSketch {
            element: s.element,
            feature: s.feature,
            op,
        })
    {
        warn!("cannot make the pattern: {e}");
    }
}

fn is_ours(world: &World, e: Entity) -> bool {
    world
        .resource::<PatternEditing>()
        .open
        .as_ref()
        .is_some_and(|d| d.entity == e)
}

fn on_accept(ev: On<FeatureDialogAccept>, mut commands: Commands) {
    let e = ev.entity;
    commands.queue(move |world: &mut World| {
        if is_ours(world, e) {
            accept_open(world);
        }
    });
}

fn on_cancel(ev: On<FeatureDialogCancel>, mut commands: Commands) {
    let e = ev.entity;
    commands.queue(move |world: &mut World| {
        if is_ours(world, e) {
            close_dialog(world);
        }
    });
}

/// A typed distance, count or angle.
fn on_number(
    ev: On<NumberFieldCommit>,
    q: Query<&Field>,
    mut q_state: Query<&mut NumberFieldState>,
    units: Res<crate::WorkspaceUnits>,
    mut editing: ResMut<PatternEditing>,
    mut commands: Commands,
) {
    let Ok(field) = q.get(ev.entity).copied() else {
        return;
    };
    let Some(d) = editing.open.as_mut() else {
        return;
    };
    let max = cadrs_sketch::pattern::MAX_INSTANCES as f64;
    let v = units
        .0
        .eval(ev.text.trim(), field.quantity())
        .ok()
        .filter(|v| v.is_finite())
        .filter(|v| match field {
            Field::Count(_) => *v >= 1.0 && *v <= max,
            Field::TurnCount => *v >= 2.0 && *v <= max,
            Field::Distance(_) => v.abs() > 1e-9,
            Field::Angle(_) => true,
            Field::TurnAngle => v.abs() > 1e-9 && v.abs() <= 360.0 + 1e-9,
        });
    let Ok(mut st) = q_state.get_mut(ev.entity) else {
        return;
    };
    let Some(v) = v else {
        st.text = ev.text.trim().to_string();
        st.error = true;
        return;
    };
    match field {
        Field::Distance(i) => d.dirs[i].spacing = v,
        Field::Count(i) => d.dirs[i].count = v.round() as u32,
        Field::Angle(i) => d.dirs[i].angle = v.to_radians(),
        Field::TurnCount => d.circular.count = v.round() as u32,
        Field::TurnAngle => d.circular.angle = v.to_radians(),
    }
    st.text = units.0.with_unit(field.value(d), field.quantity());
    st.error = false;
    // Enter in a dialog field accepts, as in Onshape (and the Extrude dialog).
    if ev.enter {
        commands.queue(|world: &mut World| {
            world.resource_mut::<InputFocus>().clear();
            accept_open(world);
        });
    }
}

/// The Second direction checkbox.
fn on_second(
    ev: On<CheckboxChange>,
    q: Query<&Name>,
    mut editing: ResMut<PatternEditing>,
    mut q_rows: Query<&mut Node, With<SecondRows>>,
) {
    if !q
        .get(ev.entity)
        .is_ok_and(|n| n.as_str() == "sketch-pattern-second-checkbox")
    {
        return;
    }
    let Some(d) = editing.open.as_mut() else {
        return;
    };
    d.second = ev.checked;
    if !ev.checked && d.picking == Pick::Direction(1) {
        d.picking = Pick::Entities;
    }
    for mut node in &mut q_rows {
        node.display = if ev.checked {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// The selection shown while `d`'s Entities field takes picks: a new pattern's seeds.
fn entities_selection(d: &PatternDialog) -> Vec<SketchEntity> {
    match d.mode {
        PatternMode::New(_) => d.seeds.iter().map(|c| SketchEntity::Curve(*c)).collect(),
        PatternMode::Edit(_) => Vec::new(),
    }
}

/// Clicking a field makes it take the next pick: the Entities field shows the seeds selected,
/// a Direction or Center field starts with nothing selected.
fn on_activate(
    ev: On<SelectionFieldActivate>,
    q: Query<&Pick>,
    mut editing: ResMut<PatternEditing>,
    mut selection: ResMut<SketchSelection>,
) {
    let Ok(pick) = q.get(ev.entity).copied() else {
        return;
    };
    let Some(d) = editing.open.as_mut() else {
        return;
    };
    d.picking = pick;
    selection.0 = match pick {
        Pick::Entities => entities_selection(d),
        _ => Vec::new(),
    };
}

/// A field's ✕: no entities, no direction line (its angle again), or the origin as center.
fn on_clear(
    ev: On<SelectionFieldClear>,
    q: Query<&Pick>,
    mut editing: ResMut<PatternEditing>,
    mut selection: ResMut<SketchSelection>,
) {
    let Ok(pick) = q.get(ev.entity).copied() else {
        return;
    };
    let Some(d) = editing.open.as_mut() else {
        return;
    };
    match pick {
        Pick::Entities => {
            if matches!(d.mode, PatternMode::New(_)) {
                d.seeds.clear();
                selection.0.clear();
            }
        }
        Pick::Direction(i) => d.dirs[i].along = None,
        Pick::Center => d.circular.center = PointRef::Origin,
    }
}

/// Choosing a pattern tool opens its dialog; picking stays with the Select tool meanwhile.
fn start_from_tool(
    mut tool: ResMut<ActiveSketchTool>,
    editing: Res<PatternEditing>,
    session: Option<Res<SketchSession>>,
    mut commands: Commands,
) {
    let kind = match tool.tool {
        SketchTool::Pattern => PatternKind::Linear,
        SketchTool::CircularPattern => PatternKind::Circular,
        _ => return,
    };
    tool.tool = SketchTool::Select;
    let same = editing
        .open
        .as_ref()
        .is_some_and(|d| d.mode == PatternMode::New(kind));
    if !same && session.is_some() {
        commands.queue(move |world: &mut World| open_pattern_dialog(world, PatternMode::New(kind)));
    }
}

/// Picks go into the active field; the fields show what the pattern has, the Angle rows hide
/// while their direction runs along a line, and the ✓ is on when there is something to make.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn sync_dialog(
    mut editing: ResMut<PatternEditing>,
    mut selection: ResMut<SketchSelection>,
    session: Option<Res<SketchSession>>,
    doc: Option<Res<ActiveDocument>>,
    units: Res<crate::WorkspaceUnits>,
    mut q_dialog: Query<&mut FeatureDialogState>,
    mut q_fields: Query<(&Pick, &mut SelectionFieldState)>,
    mut q_angles: Query<(&AngleRow, &mut Node, &mut NumberFieldState)>,
) {
    let Some(d) = editing.open.as_mut() else {
        return;
    };
    let Some(sketch) = crate::sketch_tools::session_sketch(session.as_deref(), doc.as_deref())
    else {
        return;
    };
    if let PatternMode::Edit(id) = d.mode
        && !sketch.patterns.contains_key(id)
    {
        // Undone, or deleted, while the dialog was open.
        return;
    }
    match d.picking {
        Pick::Entities => {
            if matches!(d.mode, PatternMode::New(_)) {
                let seeds = curves_of(&selection.0);
                if seeds != d.seeds {
                    d.seeds = seeds;
                }
            }
        }
        Pick::Direction(i) => {
            if let Some(l) = selection.0.iter().find_map(|e| as_direction(sketch, *e)) {
                d.dirs[i].along = Some(l);
                d.picking = Pick::Entities;
                selection.0 = entities_selection(d);
            }
        }
        Pick::Center => {
            if let Some(c) = selection.0.iter().find_map(|e| as_center(sketch, *e)) {
                d.circular.center = c;
                d.picking = Pick::Entities;
                selection.0 = entities_selection(d);
            }
        }
    }
    d.seeds.retain(|c| sketch.curves.contains_key(*c));
    for dir in &mut d.dirs {
        if let Some(l) = dir.along {
            match line_dir(sketch, l) {
                // The angle shown is the line's (kept if the line is unpicked).
                Some(u) => dir.angle = u.y.atan2(u.x),
                None => dir.along = None,
            }
        }
    }
    let valid = op_of(sketch, d).is_some();
    if let Ok(mut st) = q_dialog.get_mut(d.entity)
        && st.valid != valid
    {
        st.valid = valid;
    }
    for (pick, mut st) in &mut q_fields {
        let (value, active) = match *pick {
            Pick::Entities => (entities_text(d.seeds.len()), d.picking == Pick::Entities),
            Pick::Direction(i) => (along_text(d.dirs[i].along), d.picking == Pick::Direction(i)),
            Pick::Center => (center_text(d.circular.center), d.picking == Pick::Center),
        };
        if st.value != value {
            st.value = value;
        }
        if st.active != active {
            st.active = active;
        }
    }
    for (row, mut node, mut st) in &mut q_angles {
        let dir = &d.dirs[row.0];
        let display = if dir.along.is_some() {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != display {
            node.display = display;
        }
        if dir.along.is_some() {
            let text = units.0.with_unit(dir.angle.to_degrees(), Quantity::Angle);
            if st.text != text {
                st.text = text;
            }
        }
    }
}

/// The copies as they will be, in the sketch, while the dialog is open.
fn draw_preview(
    editing: Res<PatternEditing>,
    session: Option<Res<SketchSession>>,
    doc: Option<Res<ActiveDocument>>,
    mut g: Gizmos<SketchRubberGizmos>,
) {
    let Some(d) = editing.open.as_ref() else {
        return;
    };
    let Some(sketch) = crate::sketch_tools::session_sketch(session.as_deref(), doc.as_deref())
    else {
        return;
    };
    let layout = d.layout();
    if d.seeds.is_empty() || check(sketch, &layout).is_err() {
        return;
    }
    let Some(frame) =
        crate::sketch_tools::session_plane(session.as_deref(), doc.as_deref()).map(|p| p.frame())
    else {
        return;
    };
    let w = |p: SVec2| {
        let q = frame.to_world(p);
        Vec3::new(q[0] as f32, q[1] as f32, q[2] as f32)
    };
    let color = Color::srgb_u8(0x46, 0x9c, 0xcd);
    let shapes: Vec<Vec<SVec2>> = d
        .seeds
        .iter()
        .map(|c| cadrs_sketch::hit::curve_polyline(sketch, *c))
        .collect();
    for x in xforms(sketch, &layout) {
        for pts in &shapes {
            g.linestrip(pts.iter().map(|p| w(x.apply(*p))), color);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Context menu

#[derive(Component)]
struct PatternMenuFor(PatternId);

/// The pattern a right-click hit: one of its seeds or copies.
pub fn pattern_under(
    sketch: &cadrs_sketch::Sketch,
    hit: Option<SketchEntity>,
) -> Option<PatternId> {
    match hit? {
        SketchEntity::Curve(c) => pattern_of_curve(sketch, c),
        _ => None,
    }
}

/// Right-click on a patterned curve: Edit linear (or circular) pattern.
pub fn open_pattern_menu(world: &mut World, id: PatternId, at: Vec2) {
    let theme = world.resource::<Theme>().clone();
    let circular = world_sketch(world)
        .and_then(|s| s.patterns.get(id))
        .is_some_and(|p| matches!(p.layout, Layout::Circular(_)));
    let (label, icon) = if circular {
        ("sketch-pattern-edit-circular", "circular-pattern")
    } else {
        ("sketch-pattern-edit", "linear-pattern")
    };
    let menu = Menu::new("sketch-pattern-menu")
        .min_width(180.0)
        .item_height(23.0)
        .item(MenuItem::new("sketch-pattern-edit", tr(label)).icon(icon));
    let mut commands = world.commands();
    let anchor = cadrs_ui::open_context_menu(&mut commands, at, menu.build(&theme));
    commands
        .entity(anchor)
        .insert((PatternMenuFor(id), DespawnOnExit(AppState::Document)));
    world.flush();
}

fn on_pattern_menu(ev: On<MenuAction>, q: Query<&PatternMenuFor>, mut commands: Commands) {
    let Ok(anchor) = q.get(ev.entity) else { return };
    let id = anchor.0;
    if ev.item == "sketch-pattern-edit" {
        commands.queue(move |world: &mut World| {
            world.resource_mut::<SketchSelection>().0.clear();
            open_pattern_dialog(world, PatternMode::Edit(id));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_spacing_clears_the_seeds() {
        assert_eq!(default_spacing(0.0), 10.0);
        assert!((default_spacing(4.0) - 6.0).abs() < 1e-9);
        assert!((default_spacing(20.0) - 30.0).abs() < 1e-9);
        assert!(default_spacing(3.3) >= 3.3 * 1.5);
    }

    #[test]
    fn picks_go_to_the_right_field() {
        let mut s = cadrs_sketch::Sketch::default();
        SketchOp::AddCircle {
            center: SVec2::new(5.0, 0.0),
            radius: 1.0,
            construction: false,
        }
        .apply(&mut s)
        .unwrap();
        SketchOp::AddPolyline {
            points: vec![SVec2::new(0.0, 0.0), SVec2::new(3.0, 4.0)],
            closed: false,
            construction: false,
            label: "Add line",
        }
        .apply(&mut s)
        .unwrap();
        let (mut circle, mut line) = (None, None);
        for (k, c) in &s.curves {
            match c.kind {
                CurveKind::Circle { center, .. } => circle = Some((k, center)),
                CurveKind::Line { .. } => line = Some(k),
                _ => {}
            }
        }
        let ((circle, center), line) = (circle.unwrap(), line.unwrap());
        // A line or an axis is a direction; a circle is not.
        assert_eq!(
            as_direction(&s, SketchEntity::Curve(line)),
            Some(CurveRef::Curve(line))
        );
        assert_eq!(as_direction(&s, SketchEntity::YAxis), Some(CurveRef::YAxis));
        assert_eq!(as_direction(&s, SketchEntity::Curve(circle)), None);
        // A circle's center, a point or the origin is a center; a line is not.
        assert_eq!(
            as_center(&s, SketchEntity::Curve(circle)),
            Some(PointRef::Point(center))
        );
        assert_eq!(as_center(&s, SketchEntity::Origin), Some(PointRef::Origin));
        assert_eq!(as_center(&s, SketchEntity::Curve(line)), None);
    }
}
