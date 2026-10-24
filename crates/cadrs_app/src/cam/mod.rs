//! The CAM Studio tab (`cadrs_core::cam`): Onshape's CAM Studio workflow for mills, routers,
//! lasers, plasma tables and waterjets.
//!
//! - **Toolbar** (left to right, each active once the step before it is done): Insert part,
//!   workholding or stock · Create job · Create machine · Create setup · Create tool · Create
//!   toolpath · Create work plane · Regenerate · Preview operation · Post · Show stock.
//! - **Panel**: the **Jobs** tree (job → machine → setups → their tools, toolpaths and work
//!   planes; a toolpath's row shows its status: its time when calculated, out of date,
//!   calculating, failed) and the **Components** list (parts, workholding and stock with
//!   their reference and an eye). A click selects (a job: makes it the active one),
//!   double-click edits, right-click has Edit, Regenerate, Preview, Post, Move up/down,
//!   Duplicate, Suppress, Hide and Delete; Delete deletes the selection.
//! - **Dialogs** ([`dialogs`]): Insert component, Job, Machine (with its post settings), Setup,
//!   Tool (from the library or made here), Toolpath (Tool path, Link, Misc and Beam tabs; the
//!   toolpath previews live while it's edited) and Work plane. Every change is one
//!   `EditCam` command, so it undoes.
//! - **Computing** ([`compute`]): toolpaths are calculated off the main thread and kept by
//!   their inputs (the job cache).
//! - **View** ([`view`]): the stock, the work coordinate systems, the toolpaths (rapids red,
//!   cuts blue, entries green, leads cyan, links yellow, beam cuts orange).
//! - **Preview operation** ([`preview`]): Back plot, Verify (the stock as the toolpaths leave
//!   it, gouges red) and Simulate (with the tool and holder).
//! - **Post** ([`export`]): G-code for the job's machine, saved to a file.

pub mod compute;
pub mod dialogs;
pub mod export;
pub mod preview;
pub mod view;

use std::collections::HashSet;
use std::time::Instant;

use bevy::prelude::*;
use bevy::ui_widgets::{Activate, observe};
use cadrs_core::cam::{CamId, CamStudio, Edit, EditCam, NodeRef, OpKind};
use cadrs_core::{ElementId, ElementKind};
use cadrs_ui::menu::ContextMenuAnchor;
use cadrs_ui::prelude::*;
use cadrs_ui::{
    ContextMenuRequested, MenuAction, Notification, TreeItem, TreeRowToggled, TreeToggle,
    open_context_menu, show_notification, toolbar_separator,
};

use crate::{ActiveDocument, AppState};

pub use compute::{Computed, Status};

/// The panel's width on a CAM Studio tab.
pub const PANEL_W: f32 = 290.0;

/// The CAM Studio dialogs' widget names: `cam-f-<key>`.
pub const FORM_PREFIX: &str = "cam-f-";

/// A number typed into a CAM Studio dialog, as the sketch's dimension fields read it:
/// expressions and units (`6*2`, `1/4 in`, `10 mm + 2`, `30 deg`).
fn read_number(text: &str, unit: cadrs_ui::form::Unit) -> Option<f64> {
    use cadrs_sketch::units::{Quantity, eval};
    use cadrs_ui::form::Unit;
    let v = match unit {
        Unit::Mm => eval(text, Quantity::Length).ok()?,
        Unit::Deg => eval(text, Quantity::Angle).ok()?,
        // (Its own suffix off: what's left is a plain expression.)
        _ => {
            let t = text.trim();
            let body = [" mm/min", "mm/min", " rpm", "rpm", " %", "%", " s", "s"]
                .iter()
                .find_map(|s| t.strip_suffix(s))
                .unwrap_or(t);
            eval(body, Quantity::Length).ok()?
        }
    };
    v.is_finite().then_some(v)
}

pub struct CamUiPlugin;

impl Plugin for CamUiPlugin {
    fn build(&self, app: &mut App) {
        cadrs_core::cam::geometry::register_tasks();
        app.init_resource::<CamUi>()
            .init_resource::<Computed>()
            .add_plugins((
                cadrs_ui::form::FormPlugin::new(FORM_PREFIX).eval(read_number),
                dialogs::DialogsPlugin,
                view::ViewPlugin,
                preview::PreviewPlugin,
                export::ExportPlugin,
            ))
            .add_systems(
                Update,
                (compute::drive, rebuild_tree, sync_toolbar, sync_parts)
                    .chain()
                    .after(crate::parts::PartsSet)
                    .run_if(in_state(AppState::Document)),
            )
            .add_systems(
                OnExit(AppState::Document),
                |mut ui: ResMut<CamUi>, mut c: ResMut<Computed>| {
                    *ui = CamUi::default();
                    c.clear();
                },
            )
            .add_observer(on_row_activate)
            .add_observer(on_row_toggle)
            .add_observer(on_row_eye)
            .add_observer(on_row_menu)
            .add_observer(on_menu_action)
            .add_observer(on_insert_menu_action)
            .add_systems(Update, delete_key.run_if(in_state(AppState::Document)))
            .add_systems(
                PostUpdate,
                flag_pending_work.run_if(in_state(AppState::Document)),
            );
    }
}

/// The CAM Studio panel's view state (not part of the document).
#[derive(Resource, Debug, Clone)]
pub struct CamUi {
    /// The selected rows.
    pub selected: Vec<CamId>,
    /// Collapsed jobs and setups.
    pub collapsed: HashSet<CamId>,
    /// Show the active job's stock in the view.
    pub show_stock: bool,
    /// The last row clicked and when (double-clicks edit).
    last_click: Option<(CamId, Instant)>,
    /// Frame the parts once they're built (the first ones were just inserted).
    pub fit_pending: bool,
}

impl Default for CamUi {
    fn default() -> Self {
        CamUi {
            selected: Vec::new(),
            collapsed: HashSet::new(),
            show_stock: true,
            last_click: None,
            fit_pending: false,
        }
    }
}

/// Scripted steps and screenshots wait while toolpaths are calculated (and while a dialog's
/// live preview is about to be).
fn flag_pending_work(
    computed: Res<Computed>,
    dialog: Option<Res<dialogs::CamDialog>>,
    preview: Option<Res<preview::PreviewState>>,
    insert: Option<Res<dialogs::InsertParts>>,
    mut pending: ResMut<cadrs_ui::PendingWork>,
    mut why: ResMut<cadrs_ui::PendingWhy>,
) {
    let waiting = dialog.is_some_and(|d| d.waiting());
    if computed.busy() || waiting {
        pending.0 = true;
        why.add("cam toolpaths");
    }
    if insert.is_some() {
        pending.0 = true;
        why.add("cam insert parts");
    }
    if preview.is_some_and(|p| p.checking()) {
        pending.0 = true;
        why.add("cam verify");
    }
}

/// Scripted steps (`Custom("cam …")`): `select job|setup|op N` selects the Nth job, the active
/// job's Nth setup or its first setup's Nth operation (from 0); `play` and `pause` the preview;
/// `preview-end` jumps it to the end; `preview-at F` to a fraction of it; `fit` zooms the view
/// to the studio's parts and stock.
pub fn script(world: &mut World, arg: &str) {
    let words: Vec<&str> = arg.split_whitespace().collect();
    let Some((_, cam)) = active_cam_owned(world) else {
        warn!("cam {arg}: no CAM Studio");
        return;
    };
    match words.as_slice() {
        ["select", what, n] => {
            let Ok(n) = n.parse::<usize>() else { return };
            let id = match *what {
                "job" => cam.jobs.get(n).map(|j| j.id),
                "setup" => cam.active().and_then(|j| j.setups.get(n)).map(|s| s.id),
                "op" => cam
                    .active()
                    .and_then(|j| j.setups.first())
                    .and_then(|s| s.operations.get(n))
                    .map(|o| o.id),
                "component" => cam.components.get(n).map(|c| c.id),
                _ => None,
            };
            match id {
                Some(id) => world.resource_mut::<CamUi>().selected = vec![id],
                None => warn!("cam {arg}: nothing there"),
            }
        }
        ["play"] | ["pause"] | ["preview-end"] | ["preview-at", _] => {
            let Some(mut s) = world.get_resource_mut::<preview::PreviewState>() else {
                warn!("cam {arg}: no preview");
                return;
            };
            match words[0] {
                "play" => s.playing = true,
                "pause" => s.playing = false,
                "preview-end" => {
                    s.playing = false;
                    s.seek(1.0);
                }
                _ => {
                    s.playing = false;
                    if let Ok(f) = words[1].parse::<f64>() {
                        s.seek(f);
                    }
                }
            }
        }
        ["fit"] => crate::viewport::zoom_to_fit(world),
        _ => warn!("cam {arg}: not a CAM Studio step"),
    }
}

/// The active tab's CAM Studio.
pub fn active_cam(doc: &ActiveDocument) -> Option<(ElementId, &CamStudio)> {
    let el = doc.active_element()?;
    match &el.kind {
        ElementKind::CamStudio(c) => Some((el.id, c)),
        _ => None,
    }
}

/// The active tab's id when it is a CAM Studio.
pub fn active_cam_id(world: &World) -> Option<ElementId> {
    active_cam(world.get_resource::<ActiveDocument>()?).map(|(e, _)| e)
}

/// A copy of the active CAM Studio (for work that also needs the world).
pub fn active_cam_owned(world: &World) -> Option<(ElementId, CamStudio)> {
    active_cam(world.get_resource::<ActiveDocument>()?).map(|(e, c)| (e, c.clone()))
}

/// Runs an edit on the active CAM Studio; a refused one shows why.
pub fn edit(world: &mut World, e: Edit) -> bool {
    let Some(element) = active_cam_id(world) else {
        return false;
    };
    let r = world
        .resource_mut::<ActiveDocument>()
        .execute(&EditCam::new(element, e));
    match r {
        Ok(()) => true,
        Err(err) => {
            toast(world, &err.to_string());
            false
        }
    }
}

/// A warning toast.
pub fn toast(world: &mut World, text: &str) {
    let theme = world.resource::<Theme>().clone();
    let mut commands = world.commands();
    show_notification(
        &mut commands,
        &theme,
        Notification::warning(text.to_string()).name("cam-toast"),
    );
    world.flush();
}

/// The setup new tools and toolpaths go into, and the operation they go after: the selected
/// setup or operation's, else the active job's last setup (at its end).
pub fn target_setup(cam: &CamStudio, ui: &CamUi) -> Option<(CamId, Option<CamId>)> {
    if let Some(&sel) = ui.selected.last() {
        match cam.find(sel) {
            Some(NodeRef::Setup(j, s)) => return Some((cam.jobs[j].setups[s].id, None)),
            Some(NodeRef::Operation(j, s, _)) => {
                return Some((cam.jobs[j].setups[s].id, Some(sel)));
            }
            _ => {}
        }
    }
    let job = cam.active()?;
    job.setups.last().map(|s| (s.id, None))
}

// ---------------------------------------------------------------------------------------------
// Toolbar

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum Needs {
    Part,
    Job,
    Machine,
    Setup,
    Tool,
    Toolpath,
    Calculated,
}

/// The toolbar of a CAM Studio tab.
pub fn toolbar(tb: &mut ChildSpawnerCommands, t: &Theme) {
    crate::document::undo_redo(tb, t);
    tb.spawn(toolbar_separator(t));
    let tool = |name: &'static str, icon: &'static str, tip: &'static str| {
        ToolButton::new(name, icon)
            .icon_size(20.0)
            .tooltip(tip)
            .build(t)
    };
    tb.spawn((
        ToolButton::new("cam-tb-insert", "cam-insert")
            .icon_size(20.0)
            .dropdown(true)
            .tooltip("Insert part, workholding and stock")
            .build(t),
        observe(
            |ev: On<Activate>,
             q: Query<(&UiGlobalTransform, &ComputedNode)>,
             theme: Res<Theme>,
             mut commands: Commands| {
                let at = q
                    .get(ev.entity)
                    .map(|(tf, n)| {
                        let s = n.inverse_scale_factor();
                        Vec2::new(
                            (tf.translation.x - n.size().x / 2.0) * s,
                            (tf.translation.y + n.size().y / 2.0) * s + 2.0,
                        )
                    })
                    .unwrap_or(Vec2::new(60.0, 80.0));
                let menu = Menu::new("cam-insert-menu")
                    .min_width(190.0)
                    .item(MenuItem::new("cam-insert-part", "Insert part").icon("part"))
                    .item(
                        MenuItem::new("cam-insert-workholding", "Insert workholding")
                            .icon("cam-workholding"),
                    )
                    .item(MenuItem::new("cam-insert-stock", "Insert stock").icon("cam-stock"));
                let anchor = open_context_menu(&mut commands, at, menu.build(&theme));
                commands
                    .entity(anchor)
                    .insert((InsertMenu, DespawnOnExit(AppState::Document)));
            },
        ),
    ));
    let button = |tb: &mut ChildSpawnerCommands,
                  name: &'static str,
                  icon: &'static str,
                  tip: &'static str,
                  needs: Needs,
                  run: fn(&mut World)| {
        tb.spawn((
            tool(name, icon, tip),
            needs,
            observe(move |_: On<Activate>, mut commands: Commands| {
                commands.queue(run);
            }),
        ));
    };
    button(
        tb,
        "cam-tb-job",
        "cam-job",
        "Create job",
        Needs::Part,
        dialogs::create_job,
    );
    button(
        tb,
        "cam-tb-machine",
        "cam-machine",
        "Create machine",
        Needs::Job,
        dialogs::create_machine,
    );
    button(
        tb,
        "cam-tb-setup",
        "cam-setup",
        "Create setup",
        Needs::Machine,
        dialogs::create_setup,
    );
    button(
        tb,
        "cam-tb-tool",
        "cam-tool",
        "Create tool",
        Needs::Setup,
        dialogs::create_tool,
    );
    button(
        tb,
        "cam-tb-toolpath",
        "cam-toolpath",
        "Create toolpath",
        Needs::Tool,
        dialogs::create_toolpath,
    );
    button(
        tb,
        "cam-tb-workplane",
        "cam-work-plane",
        "Create work plane",
        Needs::Setup,
        dialogs::create_work_plane,
    );
    tb.spawn(toolbar_separator(t));
    button(
        tb,
        "cam-tb-regenerate",
        "cam-regenerate",
        "Regenerate the selected toolpaths (all out-of-date ones of the active job when none is selected)",
        Needs::Toolpath,
        regenerate_selection,
    );
    button(
        tb,
        "cam-tb-preview",
        "cam-preview",
        "Preview operation: back plot, verify and simulate",
        Needs::Calculated,
        preview::open_for_selection,
    );
    button(
        tb,
        "cam-tb-post",
        "cam-post",
        "Post the setup to G-code",
        Needs::Calculated,
        export::open_for_selection,
    );
    tb.spawn(toolbar_separator(t));
    tb.spawn((
        tool(
            "cam-tb-stock",
            "cam-stock",
            "Show or hide the active job's stock",
        ),
        observe(|_: On<Activate>, mut ui: ResMut<CamUi>| {
            ui.show_stock = !ui.show_stock;
        }),
    ));
}

#[derive(Component)]
struct InsertMenu;

fn on_insert_menu_action(
    ev: On<MenuAction>,
    q: Query<(), (With<InsertMenu>, With<ContextMenuAnchor>)>,
    mut commands: Commands,
) {
    if !q.contains(ev.entity) {
        return;
    }
    let role = match ev.item.as_str() {
        "cam-insert-part" => cadrs_core::cam::Role::Part,
        "cam-insert-workholding" => cadrs_core::cam::Role::Workholding,
        "cam-insert-stock" => cadrs_core::cam::Role::Stock,
        _ => return,
    };
    commands.queue(move |w: &mut World| dialogs::insert_component(w, role));
}

/// Greys the toolbar's buttons until their step is possible.
fn sync_toolbar(
    doc: Option<Res<ActiveDocument>>,
    computed: Res<Computed>,
    ui: Res<CamUi>,
    q: Query<(Entity, &Needs, Has<bevy::ui::InteractionDisabled>)>,
    mut commands: Commands,
) {
    let Some(doc) = doc else { return };
    let Some((_, cam)) = active_cam(&doc) else {
        return;
    };
    let job = cam.active();
    let target = target_setup(cam, &ui);
    let setup = target.and_then(|(s, _)| cam.setup(s));
    let has_tool = setup.is_some_and(|(_, s)| s.operations.iter().any(|o| o.tool().is_some()));
    let has_toolpath = job.is_some_and(|j| {
        j.setups
            .iter()
            .any(|s| s.operations.iter().any(|o| o.toolpath().is_some()))
    });
    let calculated = job.is_some_and(|j| {
        j.setups.iter().any(|s| {
            s.operations
                .iter()
                .any(|o| computed.current(o.id).is_some())
        })
    });
    for (e, needs, disabled) in &q {
        let on = match needs {
            Needs::Part => cam
                .components
                .iter()
                .any(|c| c.role == cadrs_core::cam::Role::Part),
            Needs::Job => job.is_some(),
            Needs::Machine => job.is_some_and(|j| j.machine.is_some()),
            Needs::Setup => setup.is_some(),
            Needs::Tool => has_tool,
            Needs::Toolpath => has_toolpath,
            Needs::Calculated => calculated,
        };
        if on && disabled {
            commands
                .entity(e)
                .try_remove::<bevy::ui::InteractionDisabled>();
        } else if !on && !disabled {
            commands.entity(e).try_insert(bevy::ui::InteractionDisabled);
        }
    }
}

/// Regenerate: the selected toolpaths (a setup or job: all of theirs), else every toolpath of
/// the active job that isn't up to date.
pub fn regenerate_selection(world: &mut World) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let sel = world.resource::<CamUi>().selected.clone();
    let mut ops = Vec::new();
    for id in &sel {
        match cam.find(*id) {
            Some(NodeRef::Operation(..)) => {
                if cam
                    .operation(*id)
                    .is_some_and(|(_, _, o)| o.toolpath().is_some())
                {
                    ops.push(*id);
                }
            }
            Some(NodeRef::Setup(j, s)) => ops.extend(toolpaths_of(&cam.jobs[j].setups[s])),
            Some(NodeRef::Job(j)) => {
                for s in &cam.jobs[j].setups {
                    ops.extend(toolpaths_of(s));
                }
            }
            _ => {}
        }
    }
    let c = world.resource::<Computed>();
    if ops.is_empty()
        && let Some(job) = cam.active()
    {
        for s in &job.setups {
            ops.extend(
                toolpaths_of(s)
                    .into_iter()
                    .filter(|o| !matches!(c.status(*o), Status::Done(_))),
            );
        }
    }
    world.resource_mut::<Computed>().request(ops);
}

fn toolpaths_of(s: &cadrs_core::cam::Setup) -> Vec<CamId> {
    s.operations
        .iter()
        .filter(|o| !o.suppressed && o.toolpath().is_some())
        .map(|o| o.id)
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Panel

#[derive(Component)]
pub struct CamTree;

/// The panel's host, spawned by the document shell for a CAM Studio tab.
pub fn panel(p: &mut ChildSpawnerCommands, _t: &Theme) {
    p.spawn((
        Name::new("cam-tree"),
        CamTree,
        Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            padding: UiRect::new(Val::Px(4.0), Val::Px(4.0), Val::Px(6.0), Val::Px(4.0)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
    ));
}

/// What a tree row stands for.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CamRow {
    Component(CamId),
    Job(CamId),
    Machine(CamId),
    Setup(CamId),
    Operation(CamId),
}

impl CamRow {
    pub fn id(self) -> CamId {
        match self {
            CamRow::Component(i)
            | CamRow::Job(i)
            | CamRow::Machine(i)
            | CamRow::Setup(i)
            | CamRow::Operation(i) => i,
        }
    }
}

/// A row as the tree shows it.
#[derive(Debug, Clone, PartialEq)]
struct RowSnap {
    row: CamRow,
    depth: u32,
    label: String,
    icon: &'static str,
    open: Option<bool>,
    bold: bool,
    muted: bool,
    struck: bool,
    selected: bool,
    trailing: Option<String>,
    error: bool,
    eye: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct TreeSnap {
    tree: Option<Entity>,
    rows: Vec<RowSnap>,
    components: usize,
    jobs: usize,
}

fn rows_of(cam: &CamStudio, ui: &CamUi, computed: &Computed) -> (Vec<RowSnap>, Vec<RowSnap>) {
    let mut jobs = Vec::new();
    let active = cam.active().map(|j| j.id);
    for job in &cam.jobs {
        let open = !ui.collapsed.contains(&job.id);
        jobs.push(RowSnap {
            row: CamRow::Job(job.id),
            depth: 0,
            label: job.name.clone(),
            icon: "cam-job",
            open: Some(open),
            bold: Some(job.id) == active,
            muted: job.suppressed,
            struck: job.suppressed,
            selected: ui.selected.contains(&job.id),
            trailing: None,
            error: false,
            eye: None,
        });
        if !open {
            continue;
        }
        jobs.push(RowSnap {
            row: CamRow::Machine(job.id),
            depth: 1,
            label: job
                .machine
                .as_ref()
                // (The model alone for the catalogue's generic machines: the row is narrow.)
                .map(|m| {
                    if m.builder == "Generic" {
                        m.model.clone()
                    } else {
                        m.title()
                    }
                })
                .unwrap_or_else(|| "No machine: Create machine".into()),
            icon: job
                .machine
                .as_ref()
                .map_or("cam-machine", |m| m.kind.icon()),
            open: None,
            bold: false,
            muted: job.machine.is_none(),
            struck: false,
            selected: false,
            trailing: None,
            error: false,
            eye: None,
        });
        for setup in &job.setups {
            let sopen = !ui.collapsed.contains(&setup.id);
            jobs.push(RowSnap {
                row: CamRow::Setup(setup.id),
                depth: 1,
                label: format!("{} (O{})", setup.name, setup.program_number),
                icon: "cam-setup",
                open: Some(sopen),
                bold: false,
                muted: setup.suppressed,
                struck: setup.suppressed,
                selected: ui.selected.contains(&setup.id),
                trailing: None,
                error: false,
                eye: None,
            });
            if !sopen {
                continue;
            }
            for o in &setup.operations {
                let (label, trailing, error, eye) = match &o.kind {
                    OpKind::Tool(t) => (format!("T{} {}", t.number, o.name), None, false, None),
                    OpKind::Toolpath(_) => {
                        let st = computed.status(o.id);
                        (
                            o.name.clone(),
                            Some(st.short()),
                            st.problem().is_some(),
                            Some(!o.hidden),
                        )
                    }
                    OpKind::WorkPlane(_) => (o.name.clone(), None, false, Some(!o.hidden)),
                };
                jobs.push(RowSnap {
                    row: CamRow::Operation(o.id),
                    depth: 2,
                    label,
                    icon: o.icon(),
                    open: None,
                    bold: false,
                    muted: o.suppressed,
                    struck: o.suppressed,
                    selected: ui.selected.contains(&o.id),
                    trailing,
                    error,
                    eye,
                });
            }
        }
    }
    let comps = cam
        .components
        .iter()
        .map(|c| RowSnap {
            row: CamRow::Component(c.id),
            depth: 0,
            label: c.name.clone(),
            icon: c.role.icon(),
            open: None,
            bold: false,
            muted: c.hidden,
            struck: false,
            selected: ui.selected.contains(&c.id),
            trailing: Some(if c.derived.version_name.is_empty() {
                "workspace".into()
            } else {
                c.derived.version_name.clone()
            }),
            error: false,
            eye: Some(!c.hidden),
        })
        .collect();
    (jobs, comps)
}

fn rebuild_tree(
    doc: Option<Res<ActiveDocument>>,
    ui: Res<CamUi>,
    computed: Res<Computed>,
    q_tree: Query<Entity, With<CamTree>>,
    theme: Res<Theme>,
    mut last: Local<TreeSnap>,
    mut commands: Commands,
) {
    let Ok(tree) = q_tree.single() else {
        *last = TreeSnap::default();
        return;
    };
    let Some(doc) = doc else { return };
    let Some((_, cam)) = active_cam(&doc) else {
        return;
    };
    let (jobs, comps) = rows_of(cam, &ui, &computed);
    let snap = TreeSnap {
        tree: Some(tree),
        components: comps.len(),
        jobs: cam.jobs.len(),
        rows: jobs.iter().chain(comps.iter()).cloned().collect(),
    };
    if *last == snap {
        return;
    }
    let t = theme.clone();
    commands.entity(tree).despawn_children();
    commands.entity(tree).with_children(|p| {
        header(
            p,
            &t,
            "cam-jobs-header",
            &format!("Jobs ({})", cam.jobs.len()),
        );
        if jobs.is_empty() {
            note(
                p,
                &t,
                if cam.components.is_empty() {
                    "Insert the part to machine, then create a job."
                } else {
                    "Create a job for the parts."
                },
            );
        }
        for r in &jobs {
            spawn_row(p, &t, r);
        }
        p.spawn(Node {
            height: Val::Px(10.0),
            flex_shrink: 0.0,
            ..default()
        });
        header(
            p,
            &t,
            "cam-components-header",
            &format!("Components ({})", comps.len()),
        );
        if comps.is_empty() {
            note(
                p,
                &t,
                "Insert part, workholding and stock from Part Studios.",
            );
        }
        for r in &comps {
            spawn_row(p, &t, r);
        }
    });
    *last = snap;
}

fn header(p: &mut ChildSpawnerCommands, t: &Theme, name: &str, text: &str) {
    p.spawn((
        Name::new(name.to_string()),
        t.text(text, 12.0, bevy::text::FontWeight::BOLD, t.foreground),
        Node {
            margin: UiRect::new(Val::Px(6.0), Val::Px(4.0), Val::Px(4.0), Val::Px(4.0)),
            ..default()
        },
    ));
}

fn note(p: &mut ChildSpawnerCommands, t: &Theme, text: &str) {
    p.spawn((
        t.text(
            text,
            11.0,
            bevy::text::FontWeight::NORMAL,
            t.muted_foreground,
        ),
        Node {
            margin: UiRect::new(Val::Px(8.0), Val::Px(6.0), Val::Px(2.0), Val::Px(4.0)),
            ..default()
        },
    ));
}

/// A row's `Name`: its kind and id (`cam-op-1f2e…`).
pub fn row_name(r: CamRow) -> String {
    let (k, id) = match r {
        CamRow::Component(i) => ("comp", i),
        CamRow::Job(i) => ("job", i),
        CamRow::Machine(i) => ("machine", i),
        CamRow::Setup(i) => ("setup", i),
        CamRow::Operation(i) => ("op", i),
    };
    format!("cam-{k}-{:x}", id.0)
}

fn spawn_row(p: &mut ChildSpawnerCommands, t: &Theme, r: &RowSnap) {
    let mut item = TreeItem::new(row_name(r.row), r.label.clone())
        .icon(r.icon, 16.0)
        .depth(r.depth)
        .disclosure(r.open)
        .selected(r.selected)
        .muted(r.muted)
        .strikethrough(r.struck)
        .middle_ellipsis();
    if r.bold {
        item = item.weight(bevy::text::FontWeight::BOLD);
    }
    if let Some(tr) = &r.trailing {
        let c = if r.error {
            t.danger
        } else {
            t.muted_foreground
        };
        item = item.trailing(Some(tr.clone()), c);
    }
    if r.error {
        item = item.foreground(t.danger);
    }
    if let Some(on) = r.eye {
        item = item
            .toggle(format!("{}-eye", row_name(r.row)), "visible", "hidden", on)
            .toggle_tooltip(if on { "Hide" } else { "Show" });
        if !on {
            item = item.toggle_keeps_off();
        }
    }
    p.spawn((item.build(t), r.row));
}

fn on_row_activate(ev: On<Activate>, q: Query<&CamRow>, mut commands: Commands) {
    let Ok(&row) = q.get(ev.entity) else {
        return;
    };
    commands.queue(move |w: &mut World| click_row(w, row));
}

/// A click on a row: selects it (Ctrl adds to the selection); a job becomes the active one; a
/// second click soon after edits it.
fn click_row(world: &mut World, row: CamRow) {
    let ctrl = world
        .get_resource::<ButtonInput<KeyCode>>()
        .is_some_and(|k| k.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]));
    let id = row.id();
    let now = Instant::now();
    let double = {
        let mut ui = world.resource_mut::<CamUi>();
        let double = ui
            .last_click
            .is_some_and(|(i, t)| i == id && now.duration_since(t).as_secs_f32() < 0.4);
        ui.last_click = Some((id, now));
        if ctrl {
            if let Some(i) = ui.selected.iter().position(|x| *x == id) {
                ui.selected.remove(i);
            } else {
                ui.selected.push(id);
            }
        } else {
            ui.selected = vec![id];
        }
        double
    };
    if let CamRow::Job(j) = row
        && active_cam_owned(world).is_some_and(|(_, c)| c.active().map(|x| x.id) != Some(j))
    {
        edit(world, Edit::SetActiveJob(j));
    }
    if double {
        dialogs::edit_row(world, row);
    }
}

fn on_row_toggle(ev: On<TreeToggle>, q: Query<&CamRow>, mut ui: ResMut<CamUi>) {
    let Ok(row) = q.get(ev.entity) else { return };
    let id = row.id();
    if !ui.collapsed.remove(&id) {
        ui.collapsed.insert(id);
    }
}

fn on_row_eye(ev: On<TreeRowToggled>, q: Query<&CamRow>, mut commands: Commands) {
    let Ok(&row) = q.get(ev.entity) else { return };
    commands.queue(move |w: &mut World| {
        let Some((_, cam)) = active_cam_owned(w) else {
            return;
        };
        let hidden = match row {
            CamRow::Component(id) => cam.component(id).map(|c| c.hidden),
            CamRow::Operation(id) => cam.operation(id).map(|(_, _, o)| o.hidden),
            _ => None,
        };
        if let Some(h) = hidden {
            edit(w, Edit::Hide(row.id(), !h));
        }
    });
}

#[derive(Component, Clone, Copy)]
struct RowMenuFor(CamRow);

fn on_row_menu(
    ev: On<ContextMenuRequested>,
    q: Query<&CamRow>,
    doc: Option<Res<ActiveDocument>>,
    computed: Res<Computed>,
    theme: Res<Theme>,
    mut ui: ResMut<CamUi>,
    mut commands: Commands,
) {
    let Ok(&row) = q.get(ev.entity) else { return };
    let Some(doc) = doc else { return };
    let Some((_, cam)) = active_cam(&doc) else {
        return;
    };
    if !ui.selected.contains(&row.id()) {
        ui.selected = vec![row.id()];
    }
    let item = |id: &str, label: &str, icon: &str| {
        let it = MenuItem::new(id.to_string(), label.to_string());
        if icon.is_empty() {
            it
        } else {
            it.icon(icon.to_string())
        }
    };
    let mut menu = Menu::new("cam-row-menu").min_width(200.0);
    match row {
        CamRow::Component(id) => {
            let Some(c) = cam.component(id) else { return };
            menu = menu.item(item("cam-m-edit", "Edit…", "edit"));
            menu = menu.item(item(
                "cam-m-hide",
                if c.hidden { "Show" } else { "Hide" },
                if c.hidden { "visible" } else { "hidden" },
            ));
            if !c.derived.version_name.is_empty() {
                menu = menu.item(item("cam-m-update", "Update to the latest version", "link"));
            }
            menu = menu
                .separator()
                .item(item("cam-m-delete", "Delete", "delete"));
        }
        CamRow::Job(id) => {
            let Some(j) = cam.job(id) else { return };
            menu = menu
                .item(item("cam-m-edit", "Edit job…", "edit"))
                .item(item(
                    "cam-m-machine",
                    if j.machine.is_some() {
                        "Edit machine…"
                    } else {
                        "Create machine…"
                    },
                    "cam-machine",
                ));
            if j.machine.is_some() {
                menu = menu.item(item("cam-m-setup", "Create setup…", "cam-setup"));
            }
            menu = menu
                .item(item(
                    "cam-m-regenerate",
                    "Regenerate all toolpaths",
                    "cam-regenerate",
                ))
                .separator()
                .item(item("cam-m-duplicate", "Duplicate", "copy"))
                .item(item(
                    "cam-m-suppress",
                    if j.suppressed {
                        "Unsuppress"
                    } else {
                        "Suppress"
                    },
                    "",
                ))
                .item(item("cam-m-delete", "Delete", "delete"));
        }
        CamRow::Machine(_) => {
            menu = menu.item(item("cam-m-machine", "Edit machine…", "cam-machine"));
        }
        CamRow::Setup(id) => {
            let Some((_, s)) = cam.setup(id) else { return };
            menu = menu
                .item(item("cam-m-edit", "Edit setup…", "edit"))
                .item(item("cam-m-tool", "Create tool…", "cam-tool"))
                .item(item("cam-m-toolpath", "Create toolpath…", "cam-toolpath"))
                .item(item(
                    "cam-m-workplane",
                    "Create work plane…",
                    "cam-work-plane",
                ))
                .separator()
                .item(item(
                    "cam-m-regenerate",
                    "Regenerate all toolpaths",
                    "cam-regenerate",
                ))
                .item(item("cam-m-preview", "Preview operation", "cam-preview"))
                .item(item("cam-m-post", "Post child operations…", "cam-post"))
                .separator()
                .item(item("cam-m-duplicate", "Duplicate", "copy"))
                .item(item(
                    "cam-m-suppress",
                    if s.suppressed {
                        "Unsuppress"
                    } else {
                        "Suppress"
                    },
                    "",
                ))
                .item(item("cam-m-delete", "Delete", "delete"));
        }
        CamRow::Operation(id) => {
            let Some((_, setup, o)) = cam.operation(id) else {
                return;
            };
            menu = menu.item(item("cam-m-edit", "Edit…", "edit"));
            if o.toolpath().is_some() {
                let calc = computed.current(id).is_some();
                menu = menu
                    .item(item(
                        "cam-m-regenerate",
                        "Regenerate operation",
                        "cam-regenerate",
                    ))
                    .item(item("cam-m-preview", "Preview operation", "cam-preview").disabled(!calc))
                    .item(
                        item(
                            "cam-m-post-selected",
                            "Post selected operations…",
                            "cam-post",
                        )
                        .disabled(!calc),
                    );
            }
            let i = setup
                .operations
                .iter()
                .position(|x| x.id == id)
                .unwrap_or(0);
            menu = menu
                .separator()
                .item(item("cam-m-up", "Move up", "arrow-up").disabled(i == 0))
                .item(
                    item("cam-m-down", "Move down", "arrow-down")
                        .disabled(i + 1 >= setup.operations.len()),
                )
                .item(item("cam-m-duplicate", "Duplicate", "copy"));
            // To the other setups: moved within its job, copied to any.
            for (jj, other_job) in cam.jobs.iter().enumerate() {
                for other in &other_job.setups {
                    if other.id == setup.id {
                        continue;
                    }
                    let place = if cam.jobs.len() > 1 {
                        format!("{} · {}", other_job.name, other.name)
                    } else {
                        other.name.clone()
                    };
                    if jj
                        == cam.find(id).map_or(usize::MAX, |n| match n {
                            NodeRef::Operation(j, _, _) => j,
                            _ => usize::MAX,
                        })
                    {
                        menu = menu.item(item(
                            &format!("cam-m-moveto-{:x}", other.id.0),
                            &format!("Move to {place}"),
                            "",
                        ));
                    }
                    menu = menu.item(item(
                        &format!("cam-m-copyto-{:x}", other.id.0),
                        &format!("Copy to {place}"),
                        "copy",
                    ));
                }
            }
            menu = menu.item(item(
                "cam-m-suppress",
                if o.suppressed {
                    "Unsuppress"
                } else {
                    "Suppress"
                },
                "",
            ));
            if !matches!(o.kind, OpKind::Tool(_)) {
                menu = menu.item(item(
                    "cam-m-hide",
                    if o.hidden { "Show" } else { "Hide" },
                    if o.hidden { "visible" } else { "hidden" },
                ));
            }
            menu = menu.item(item("cam-m-delete", "Delete", "delete"));
        }
    }
    let at = Vec2::new(ev.position.x + 2.0, ev.position.y + 4.0);
    let anchor = open_context_menu(&mut commands, at, menu.build(&theme));
    commands
        .entity(anchor)
        .insert((RowMenuFor(row), DespawnOnExit(AppState::Document)));
}

fn on_menu_action(
    ev: On<MenuAction>,
    q: Query<&RowMenuFor, With<ContextMenuAnchor>>,
    mut commands: Commands,
) {
    let Ok(&RowMenuFor(row)) = q.get(ev.entity) else {
        return;
    };
    let action = ev.item.clone();
    commands.queue(move |w: &mut World| row_action(w, row, &action));
}

/// A row menu's action.
pub fn row_action(world: &mut World, row: CamRow, action: &str) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let id = row.id();
    match action {
        "cam-m-edit" => dialogs::edit_row(world, row),
        "cam-m-machine" => dialogs::edit_machine(world, id),
        "cam-m-setup" => {
            world.resource_mut::<CamUi>().selected = vec![id];
            dialogs::create_setup(world);
        }
        "cam-m-tool" => {
            world.resource_mut::<CamUi>().selected = vec![id];
            dialogs::create_tool(world);
        }
        "cam-m-toolpath" => {
            world.resource_mut::<CamUi>().selected = vec![id];
            dialogs::create_toolpath(world);
        }
        "cam-m-workplane" => {
            world.resource_mut::<CamUi>().selected = vec![id];
            dialogs::create_work_plane(world);
        }
        "cam-m-regenerate" => {
            world.resource_mut::<CamUi>().selected = vec![id];
            regenerate_selection(world);
        }
        "cam-m-preview" => {
            world.resource_mut::<CamUi>().selected = vec![id];
            preview::open_for_selection(world);
        }
        "cam-m-post" => {
            world.resource_mut::<CamUi>().selected = vec![id];
            export::open_for_selection(world);
        }
        "cam-m-post-selected" => export::open_for_selection(world),
        "cam-m-up" | "cam-m-down" => {
            if let Some((_, setup, _)) = cam.operation(id) {
                let i = setup
                    .operations
                    .iter()
                    .position(|o| o.id == id)
                    .unwrap_or(0);
                let to = if action == "cam-m-up" {
                    i.saturating_sub(1)
                } else {
                    i + 1
                };
                edit(
                    world,
                    Edit::Move {
                        op: id,
                        setup: setup.id,
                        index: to,
                    },
                );
            }
        }
        "cam-m-duplicate" => {
            let seed = uuid::Uuid::new_v4().as_u64_pair().0;
            edit(world, Edit::Duplicate { id, seed });
        }
        "cam-m-suppress" => {
            let on = match cam.find(id) {
                Some(NodeRef::Job(j)) => !cam.jobs[j].suppressed,
                Some(NodeRef::Setup(j, s)) => !cam.jobs[j].setups[s].suppressed,
                Some(NodeRef::Operation(j, s, o)) => {
                    !cam.jobs[j].setups[s].operations[o].suppressed
                }
                _ => return,
            };
            edit(world, Edit::Suppress(id, on));
        }
        "cam-m-hide" => {
            let on = match cam.find(id) {
                Some(NodeRef::Component(i)) => !cam.components[i].hidden,
                Some(NodeRef::Operation(j, s, o)) => !cam.jobs[j].setups[s].operations[o].hidden,
                _ => return,
            };
            edit(world, Edit::Hide(id, on));
        }
        "cam-m-update" => dialogs::update_component(world, id),
        a if a.starts_with("cam-m-moveto-") || a.starts_with("cam-m-copyto-") => {
            let want = u64::from_str_radix(&a["cam-m-moveto-".len()..], 16).ok();
            let Some(to) = cam
                .jobs
                .iter()
                .flat_map(|j| &j.setups)
                .find(|s| Some(s.id.0) == want)
            else {
                return;
            };
            if a.starts_with("cam-m-moveto-") {
                edit(
                    world,
                    Edit::Move {
                        op: id,
                        setup: to.id,
                        index: to.operations.len(),
                    },
                );
            } else {
                let seed = uuid::Uuid::new_v4().as_u64_pair().0;
                edit(
                    world,
                    Edit::CopyTo {
                        op: id,
                        setup: to.id,
                        seed,
                    },
                );
            }
        }
        "cam-m-delete" => delete(world, &[id]),
        _ => {}
    }
}

/// Deletes rows (one undo step).
pub fn delete(world: &mut World, ids: &[CamId]) {
    match ids {
        [] => {}
        [id] => {
            edit(world, Edit::Delete(*id));
        }
        _ => {
            edit(
                world,
                Edit::Batch(ids.iter().map(|id| Edit::Delete(*id)).collect()),
            );
        }
    }
    world.resource_mut::<CamUi>().selected.clear();
}

/// Delete deletes the selected rows (while no text field has the keyboard).
fn delete_key(
    keys: Res<ButtonInput<KeyCode>>,
    ui: Res<CamUi>,
    doc: Option<Res<ActiveDocument>>,
    focus: Res<bevy::input_focus::InputFocus>,
    dialog: Option<Res<dialogs::CamDialog>>,
    mut commands: Commands,
) {
    if !keys.just_pressed(KeyCode::Delete)
        || ui.selected.is_empty()
        || focus.get().is_some()
        || dialog.is_some()
    {
        return;
    }
    if doc.as_deref().and_then(active_cam).is_none() {
        return;
    }
    let ids = ui.selected.clone();
    commands.queue(move |w: &mut World| delete(w, &ids));
}

/// What [`sync_parts`] last set up: the hidden and stock components, the parts' generation.
type PartsKey = (Vec<CamId>, Vec<CamId>, u64);

/// Hidden components' parts are left out of the view; stock components are see-through.
fn sync_parts(
    doc: Option<Res<ActiveDocument>>,
    mut cache: ResMut<crate::parts::PartCache>,
    ui: Res<CamUi>,
    mut last: Local<Option<PartsKey>>,
    mut commands: Commands,
) {
    let ui_fit = |c: &crate::parts::PartCache| ui.fit_pending && !c.parts.is_empty();
    let Some(doc) = doc else { return };
    let Some((el, cam)) = active_cam(&doc) else {
        *last = None;
        return;
    };
    if cache.settled().is_none_or(|(id, _, _)| id != el) {
        return;
    }
    let hidden: Vec<CamId> = cam
        .components
        .iter()
        .filter(|c| c.hidden)
        .map(|c| c.id)
        .collect();
    let stock: Vec<CamId> = cam
        .components
        .iter()
        .filter(|c| c.role == cadrs_core::cam::Role::Stock)
        .map(|c| c.id)
        .collect();
    let key = (hidden.clone(), stock.clone(), cache.generation);
    if last.as_ref() == Some(&key) {
        return;
    }
    let shown: Vec<cadrs_core::PartId> = cache
        .parts
        .iter()
        .filter(|p| cam.component_of(p.id).is_none_or(|c| !c.hidden))
        .map(|p| p.id)
        .collect();
    let see: Vec<cadrs_core::PartId> = cache
        .parts
        .iter()
        .filter(|p| {
            cam.component_of(p.id)
                .is_some_and(|c| stock.contains(&c.id))
        })
        .map(|p| p.id)
        .collect();
    cache.isolate(if hidden.is_empty() { None } else { Some(shown) });
    if ui_fit(&cache) {
        commands.queue(|w: &mut World| {
            if std::mem::take(&mut w.resource_mut::<CamUi>().fit_pending) {
                crate::viewport::zoom_to_fit(w);
            }
        });
    }
    if !see.is_empty() {
        cache.set_transparent(&see, true);
    }
    *last = Some((hidden, stock, cache.generation));
}

#[cfg(test)]
mod number_tests {
    use super::read_number;
    use cadrs_ui::form::Unit;

    #[test]
    fn typed_numbers_read_as_expressions() {
        // As the dialogs read them: the expression first, the plain reading after.
        let f = cadrs_ui::form::FormSettings {
            prefix: super::FORM_PREFIX.into(),
            eval: Some(read_number),
        };
        let near = |a: Option<f64>, b: f64| a.is_some_and(|a| (a - b).abs() < 1e-9);
        assert!(near(f.read("6*2", Unit::Mm), 12.0));
        assert!(near(f.read("1/4 in", Unit::Mm), 6.35));
        assert!(near(f.read("0.25 in", Unit::Mm), 6.35));
        assert!(near(f.read("10 mm + 2", Unit::Mm), 12.0));
        assert!(near(f.read("30 deg", Unit::Deg), 30.0));
        assert!(near(f.read("1200 mm/min", Unit::Feed), 1200.0));
        assert!(near(f.read("2000*1.5 rpm", Unit::Rpm), 3000.0));
        assert_eq!(f.read("six", Unit::Mm), None);
    }
}

#[cfg(test)]
mod icon_tests {
    use cadrs_core::cam::cadrs_cam::{MachineKind, Pattern, ToolKind};

    /// Every pattern, tool and machine names an icon icon-rs has.
    #[test]
    fn cam_icons_exist() {
        let names = Pattern::ALL
            .iter()
            .map(|p| p.icon())
            .chain(ToolKind::ALL.iter().map(|t| t.icon()))
            .chain(MachineKind::ALL.iter().map(|m| m.icon()));
        for n in names {
            assert!(cadrs_ui::icon::has_icon(n), "{n}");
        }
    }
}
