//! The CAM Studio dialogs: Insert component, Job, Machine, Setup, Tool, Toolpath, Work plane and
//! Edit component, as Onshape's (a feature dialog card at the top left of the view, ✓ and ✕).
//!
//! Each dialog is a [`Draft`] being edited: [`Draft::fields`] lays it out with [`form`],
//! [`Draft::set`] takes a changed field, [`Draft::pick`] a pick in the view (bodies,
//! workholding, drive edges, faces and holes, the face a work plane's Z follows), and
//! [`Draft::commit`] gives the edits ✓ makes, one undo step each. A toolpath previews live: its
//! draft is calculated (a moment after the last change) and drawn in the view; ✓ keeps the
//! result (the job cache).

use bevy::prelude::*;
use cadrs_core::cam::cadrs_cam::machine::{Machine, MachineKind, catalog};
use cadrs_core::cam::cadrs_cam::post::LaserMode;
use cadrs_core::cam::cadrs_cam::strategy::{
    CuttingMethod, Direction, Entry, HeightRef, HoleSelect, LeadKind, LinkAction, MachineBy,
    Params, Pattern, Side, SubPattern, Type,
};
use cadrs_core::cam::cadrs_cam::tool::{
    Hand, MachiningType, Profile, Tool, ToolKind, ToolLibrary, Unit as LenUnit,
};
use cadrs_core::cam::cadrs_cam::toolpath::{Coolant, CycleKind};
use cadrs_core::cam::{
    CamId, CamStudio, Component, Edit, Job, Mating, NodeRef, OpKind, Operation, Origin, Role,
    Setup, StockDef, StockDirection, StockKind, ToolOp, ToolpathOp, WorkPlane, WorkPlaneType,
    geometry,
};
use cadrs_core::{ElementId, ElementKind, PartId};
use cadrs_ui::prelude::*;
use cadrs_ui::{FeatureDialog, FeatureDialogAccept, FeatureDialogCancel, FeatureDialogState};

use super::form::{self, Field, FieldChange, FieldUnits, Unit, Value};
use super::{CamRow, CamUi, active_cam_owned, edit, target_setup, toast};
use crate::viewport::{Pick, PickRequest, ViewportArea};
use crate::{ActiveDocument, AppState};

pub struct DialogsPlugin;

impl Plugin for DialogsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (take_changes, take_picks, sync_dialog, live_preview)
                .chain()
                .run_if(in_state(AppState::Document)),
        )
        .add_systems(OnExit(AppState::Document), |mut commands: Commands| {
            commands.remove_resource::<CamDialog>();
        })
        .add_observer(on_accept)
        .add_observer(on_cancel);
    }
}

/// The open dialog.
#[derive(Resource)]
pub struct CamDialog {
    pub draft: Draft,
    /// The pick list that takes the view's picks.
    pub picking: Option<String>,
    /// The toolpath's tab (Tool path, Link, Misc, Beam).
    pub tab: usize,
    /// The fields the dialog shows (rebuilt when they change).
    shown: Option<Vec<Field>>,
    /// When the draft last changed (the live preview waits a moment).
    changed: Option<std::time::Instant>,
}

impl CamDialog {
    /// A change the live preview hasn't taken yet (a toolpath's).
    pub fn waiting(&self) -> bool {
        self.changed.is_some() && matches!(self.draft, Draft::Toolpath { .. })
    }
}

/// What a dialog edits.
#[derive(Debug, Clone)]
pub enum Draft {
    Insert(InsertDraft),
    Component(Box<Component>),
    Job {
        job: Box<Job>,
        is_new: bool,
    },
    Machine {
        job: CamId,
        machine: Box<Machine>,
        /// The catalogue entry it started from (`None`: as the job had it).
        from: Option<usize>,
        by_kind: bool,
    },
    Setup {
        job: CamId,
        setup: Box<Setup>,
        is_new: bool,
    },
    Tool {
        setup: CamId,
        after: Option<CamId>,
        op: Box<Operation>,
        is_new: bool,
        library: bool,
        filter: Option<MachiningType>,
    },
    Toolpath {
        setup: CamId,
        after: Option<CamId>,
        op: Box<Operation>,
        is_new: bool,
        /// The type picked (the patterns offered).
        kind: Type,
    },
    WorkPlane {
        setup: CamId,
        after: Option<CamId>,
        op: Box<Operation>,
        is_new: bool,
    },
}

/// Insert part, workholding or stock.
#[derive(Debug, Clone)]
pub struct InsertDraft {
    pub role: Role,
    /// The Part Studio (an index into the document's Part Studios).
    pub studio: usize,
    /// 0: the workspace; 1…: the document's versions.
    pub version: usize,
    /// The studio's parts: (id, name, ticked).
    pub parts: Vec<(PartId, String, bool)>,
}

/// What a draft reads from the document.
pub struct Ctx<'a> {
    pub doc: &'a cadrs_core::Document,
    pub cam: &'a CamStudio,
    /// The studio's built parts.
    pub parts: &'a [cadrs_core::Part],
    /// This document's versions (name, id).
    pub versions: &'a [(String, cadrs_core::history_log::VersionId)],
}

impl Ctx<'_> {
    fn part_studios(&self) -> Vec<(ElementId, String)> {
        self.doc
            .elements
            .iter()
            .filter(|e| matches!(e.kind, ElementKind::PartStudio { .. }))
            .map(|e| (e.id, e.name.clone()))
            .collect()
    }

    fn part_name(&self, id: PartId) -> String {
        self.parts
            .iter()
            .find(|p| p.id == id)
            .map(|p| {
                let props = self
                    .cam
                    .component_of(id)
                    .map(|c| c.derived.props.clone())
                    .unwrap_or_default();
                cadrs_core::parts::display_name(p, &props).to_string()
            })
            .or_else(|| self.cam.component_of(id).map(|c| c.name.clone()))
            .unwrap_or_else(|| "(missing part)".into())
    }
}

// ---------------------------------------------------------------------------------------------
// Opening

fn ctx_parts(world: &World) -> Vec<cadrs_core::Part> {
    super::active_cam_id(world)
        .and_then(|e| super::compute::studio_parts(world, e))
        .unwrap_or_default()
}

fn versions(world: &World) -> Vec<(String, cadrs_core::history_log::VersionId)> {
    world
        .get_resource::<crate::history_panel::DocLog>()
        .and_then(|l| l.log.as_ref())
        .map(|l| {
            l.versions()
                .iter()
                .map(|v| (v.name().to_string(), v.id()))
                .collect()
        })
        .unwrap_or_default()
}

/// Opens a dialog on a draft (closing any other first).
pub fn open(world: &mut World, draft: Draft) {
    close(world);
    let picking = match &draft {
        Draft::Job { job, .. } if job.bodies.is_empty() => None,
        _ => None,
    };
    world.insert_resource(CamDialog {
        draft,
        picking,
        tab: 0,
        shown: None,
        changed: Some(std::time::Instant::now()),
    });
}

/// Closes the dialog (no edits).
pub fn close(world: &mut World) {
    let mut q = world.query_filtered::<Entity, With<CamDialogRoot>>();
    for e in q.iter(world).collect::<Vec<_>>() {
        world.entity_mut(e).despawn();
    }
    world.remove_resource::<CamDialog>();
    world
        .resource_mut::<crate::viewport::PickFilterOverride>()
        .0 = None;
    world
        .resource_mut::<super::compute::Computed>()
        .clear_preview();
}

pub fn insert_component(world: &mut World, role: Role) {
    let Some((_, _cam)) = active_cam_owned(world) else {
        return;
    };
    let doc = world.resource::<ActiveDocument>().doc.clone();
    let studios: Vec<ElementId> = doc
        .elements
        .iter()
        .filter(|e| matches!(e.kind, ElementKind::PartStudio { .. }))
        .map(|e| e.id)
        .collect();
    if studios.is_empty() {
        toast(world, "The document has no Part Studio to take parts from");
        return;
    }
    let mut d = InsertDraft {
        role,
        studio: 0,
        version: 0,
        parts: Vec::new(),
    };
    fill_parts(world, &mut d);
    open(world, Draft::Insert(d));
}

/// The parts of the draft's Part Studio at its reference.
fn fill_parts(world: &mut World, d: &mut InsertDraft) {
    let Some(d2) = source_of(world, d) else {
        d.parts.clear();
        return;
    };
    let parts = cadrs_core::derived::source_parts(&d2.0).unwrap_or_default();
    d.parts = parts
        .into_iter()
        .enumerate()
        .map(|(i, (id, name))| (id, name, i == 0 && d.role == Role::Part))
        .collect();
}

/// The Derived reference an insert draft makes (and the frozen copies a version needs).
fn source_of(
    world: &mut World,
    d: &InsertDraft,
) -> Option<(
    cadrs_core::derived::DerivedFeature,
    Vec<cadrs_core::external::LinkedElement>,
)> {
    let doc = world.resource::<ActiveDocument>().doc.clone();
    let studios: Vec<ElementId> = doc
        .elements
        .iter()
        .filter(|e| matches!(e.kind, ElementKind::PartStudio { .. }))
        .map(|e| e.id)
        .collect();
    let element = *studios.get(d.studio)?;
    if d.version == 0 {
        let el = doc.element(element)?;
        let mut f = cadrs_core::derived::DerivedFeature::new(None, element);
        f.fill_from(el, "", "");
        return Some((f, Vec::new()));
    }
    let vs = versions(world);
    let (_, vid) = vs.get(d.version - 1)?.clone();
    let log = world.resource::<crate::history_panel::DocLog>().log.clone();
    let got = {
        let mut res = crate::linked::resolver(world);
        cadrs_core::derived::resolve(
            &mut res.0,
            &doc,
            log.as_ref(),
            cadrs_core::derived::DerivedFeature::default(),
            cadrs_core::external::SourceRef::version(None, element, vid),
        )
    };
    match got {
        Ok(r) => Some((r.derived, r.links)),
        Err(e) => {
            toast(world, &e.to_string());
            None
        }
    }
}

pub fn create_job(world: &mut World) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let job = Job {
        id: CamId::new(),
        name: cam.next_name("Job"),
        bodies: Vec::new(),
        stock: StockDef::default(),
        workholdings: Vec::new(),
        machine: None,
        setups: Vec::new(),
        suppressed: false,
    };
    open(
        world,
        Draft::Job {
            job: Box::new(job),
            is_new: true,
        },
    );
    if let Some(mut d) = world.get_resource_mut::<CamDialog>() {
        d.picking = Some("bodies".into());
    }
}

pub fn create_machine(world: &mut World) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let Some(job) = cam.active() else {
        return;
    };
    edit_machine(world, job.id);
}

/// Create or edit a job's machine.
pub fn edit_machine(world: &mut World, job: CamId) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let Some(j) = cam.job(job) else { return };
    let (machine, from) = match &j.machine {
        Some(m) => (m.clone(), None),
        None => (catalog().remove(0), Some(0)),
    };
    open(
        world,
        Draft::Machine {
            job,
            machine: Box::new(machine),
            from,
            by_kind: true,
        },
    );
}

pub fn create_setup(world: &mut World) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    // The selected job, else the active one.
    let sel = world.resource::<CamUi>().selected.last().copied();
    let job = sel
        .and_then(|s| match cam.find(s) {
            Some(NodeRef::Job(j))
            | Some(NodeRef::Setup(j, _))
            | Some(NodeRef::Operation(j, ..)) => Some(cam.jobs[j].id),
            _ => None,
        })
        .or_else(|| cam.active().map(|j| j.id));
    let Some(job) = job else { return };
    if cam.job(job).is_some_and(|j| j.machine.is_none()) {
        toast(world, "Create the job's machine first");
        return;
    }
    let n = cam.job(job).map_or(0, |j| j.setups.len()) as u32;
    let setup = Setup {
        id: CamId::new(),
        name: cam.next_name("Setup"),
        program_number: 1001 + n,
        mating: Mating::Automatic,
        wcs: WorkPlane::default(),
        operations: Vec::new(),
        suppressed: false,
    };
    open(
        world,
        Draft::Setup {
            job,
            setup: Box::new(setup),
            is_new: true,
        },
    );
}

pub fn create_tool(world: &mut World) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let ui = world.resource::<CamUi>().clone();
    let Some((setup, after)) = target_setup(&cam, &ui) else {
        toast(world, "Create a setup first");
        return;
    };
    let Some((job, _)) = cam.setup(setup) else {
        return;
    };
    let kind = job.machine.as_ref().map_or(MachineKind::Mill, |m| m.kind);
    let tk = ToolKind::for_machine(kind)[0];
    let tool = Tool::new(
        tk,
        if tk.is_beam() {
            default_kerf(kind)
        } else {
            6.0
        },
    );
    let op = Operation {
        id: CamId::new(),
        name: tool.name.clone(),
        kind: OpKind::Tool(Box::new(ToolOp {
            tool,
            number: cam.next_tool_number(job.id),
        })),
        suppressed: false,
        hidden: false,
    };
    let library = cam.library.tools.iter().any(|t| t.kind.fits(kind));
    open(
        world,
        Draft::Tool {
            setup,
            after,
            op: Box::new(op),
            is_new: true,
            library,
            filter: None,
        },
    );
}

fn default_kerf(kind: MachineKind) -> f64 {
    match kind {
        MachineKind::Laser => 0.15,
        MachineKind::Plasma => 1.5,
        MachineKind::Waterjet => 0.9,
        _ => 6.0,
    }
}

pub fn create_toolpath(world: &mut World) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let ui = world.resource::<CamUi>().clone();
    let Some((setup, after)) = target_setup(&cam, &ui) else {
        toast(world, "Create a setup first");
        return;
    };
    let Some((job, s)) = cam.setup(setup) else {
        return;
    };
    let kind = job.machine.as_ref().map_or(MachineKind::Mill, |m| m.kind);
    // The tool it will use: the last above where it goes.
    let at = after
        .and_then(|a| s.operations.iter().position(|o| o.id == a))
        .map_or(s.operations.len(), |i| i + 1);
    let Some(tool) = s.operations[..at]
        .iter()
        .rev()
        .filter(|o| !o.suppressed)
        .find_map(|o| o.tool())
    else {
        toast(world, "Create a tool above the toolpath first");
        return;
    };
    let types: Vec<Type> = Type::ALL
        .into_iter()
        .filter(|t| !Pattern::available(*t, tool.tool.kind, kind).is_empty())
        .collect();
    let Some(&t) = types.first() else {
        toast(world, "The tool has no toolpath patterns on this machine");
        return;
    };
    let pattern = Pattern::available(t, tool.tool.kind, kind)[0];
    let params = Params::new(pattern, &tool.tool, kind);
    let op = Operation {
        id: CamId::new(),
        name: cam.next_name(pattern.label()),
        kind: OpKind::Toolpath(Box::new(ToolpathOp {
            params,
            name_edited: false,
        })),
        suppressed: false,
        hidden: false,
    };
    open(
        world,
        Draft::Toolpath {
            setup,
            after,
            op: Box::new(op),
            is_new: true,
            kind: t,
        },
    );
}

pub fn create_work_plane(world: &mut World) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let ui = world.resource::<CamUi>().clone();
    let Some((setup, after)) = target_setup(&cam, &ui) else {
        toast(world, "Create a setup first");
        return;
    };
    let wcs = cam.setup(setup).map(|(_, s)| s.wcs).unwrap_or_default();
    let op = Operation {
        id: CamId::new(),
        name: cam.next_name("Work plane"),
        kind: OpKind::WorkPlane(Box::new(WorkPlane { datum: None, ..wcs })),
        suppressed: false,
        hidden: false,
    };
    open(
        world,
        Draft::WorkPlane {
            setup,
            after,
            op: Box::new(op),
            is_new: true,
        },
    );
}

/// Edit… on a row (and a double-click).
pub fn edit_row(world: &mut World, row: CamRow) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    match row {
        CamRow::Component(id) => {
            if let Some(c) = cam.component(id) {
                open(world, Draft::Component(Box::new(c.clone())));
            }
        }
        CamRow::Job(id) => {
            if let Some(j) = cam.job(id) {
                open(
                    world,
                    Draft::Job {
                        job: Box::new(j.clone()),
                        is_new: false,
                    },
                );
            }
        }
        CamRow::Machine(id) => edit_machine(world, id),
        CamRow::Setup(id) => {
            if let Some((j, s)) = cam.setup(id) {
                open(
                    world,
                    Draft::Setup {
                        job: j.id,
                        setup: Box::new(s.clone()),
                        is_new: false,
                    },
                );
            }
        }
        CamRow::Operation(id) => {
            let Some((job, s, o)) = cam.operation(id) else {
                return;
            };
            let kind = job.machine.as_ref().map_or(MachineKind::Mill, |m| m.kind);
            let draft = match &o.kind {
                OpKind::Tool(_) => Draft::Tool {
                    setup: s.id,
                    after: None,
                    op: Box::new(o.clone()),
                    is_new: false,
                    library: false,
                    filter: None,
                },
                OpKind::Toolpath(tp) => Draft::Toolpath {
                    setup: s.id,
                    after: None,
                    op: Box::new(o.clone()),
                    is_new: false,
                    kind: tp.params.pattern.type_of(),
                },
                OpKind::WorkPlane(_) => Draft::WorkPlane {
                    setup: s.id,
                    after: None,
                    op: Box::new(o.clone()),
                    is_new: false,
                },
            };
            let _ = kind;
            open(world, draft);
        }
    }
}

/// A component at a version: to the newest version of its document.
pub fn update_component(world: &mut World, id: CamId) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let Some(c) = cam.component(id).cloned() else {
        return;
    };
    let Some(src) = c.derived.source else { return };
    let vs = versions(world);
    let Some((name, vid)) = vs.last().cloned() else {
        toast(world, "The document has no version");
        return;
    };
    if c.derived.version_name == name {
        toast(world, "The component is at the newest version");
        return;
    }
    let doc = world.resource::<ActiveDocument>().doc.clone();
    let log = world.resource::<crate::history_panel::DocLog>().log.clone();
    let got = {
        let mut res = crate::linked::resolver(world);
        cadrs_core::derived::resolve(
            &mut res.0,
            &doc,
            log.as_ref(),
            c.derived.clone(),
            cadrs_core::external::SourceRef::version(src.document, src.element, vid),
        )
    };
    match got {
        Ok(r) => {
            let mut c2 = c.clone();
            c2.derived = r.derived;
            edit(
                world,
                Edit::SetComponent {
                    component: Box::new(c2),
                    links: r.links,
                },
            );
        }
        Err(e) => toast(world, &e.to_string()),
    }
}

// ---------------------------------------------------------------------------------------------
// Fields

const ORIGINS: [&str; 4] = ["World", "Component box point", "Stock box point", "Manual"];
const BOX_AXIS: [&str; 3] = ["Minimum", "Middle", "Maximum"];
/// The axis directions offered for a work plane's Z and X, and a stock's direction.
const AXES: [(&str, [f64; 3]); 6] = [
    ("+Z", [0.0, 0.0, 1.0]),
    ("−Z", [0.0, 0.0, -1.0]),
    ("+X", [1.0, 0.0, 0.0]),
    ("−X", [-1.0, 0.0, 0.0]),
    ("+Y", [0.0, 1.0, 0.0]),
    ("−Y", [0.0, -1.0, 0.0]),
];

fn axis_index(v: [f64; 3]) -> Option<usize> {
    AXES.iter()
        .position(|(_, a)| (a[0] - v[0]).abs() + (a[1] - v[1]).abs() + (a[2] - v[2]).abs() < 1e-6)
}

fn index_of<T: PartialEq + Copy>(all: &[T], v: T) -> usize {
    all.iter().position(|x| *x == v).unwrap_or(0)
}

fn labels<T: Copy>(all: &[T], f: impl Fn(T) -> &'static str) -> Vec<String> {
    all.iter().map(|x| f(*x).to_string()).collect()
}

/// A work plane's fields (`p` prefixes the keys).
fn plane_fields(out: &mut Vec<Field>, w: &WorkPlane, picking: bool, with_type: bool) {
    out.push(Field::section("Origin"));
    let o = match w.origin {
        Origin::World => 0,
        Origin::ComponentBox(_) => 1,
        Origin::StockBox(_) => 2,
        Origin::Manual(_) => 3,
    };
    out.push(Field::select("p-origin", "Work plane origin", ORIGINS, o));
    match w.origin {
        Origin::ComponentBox(bp) | Origin::StockBox(bp) => {
            for (i, a) in ["X", "Y", "Z"].iter().enumerate() {
                out.push(Field::select(
                    &format!("p-box-{i}"),
                    &format!("Position {a}"),
                    BOX_AXIS,
                    (bp[i] + 1).clamp(0, 2) as usize,
                ));
            }
        }
        Origin::Manual(p) => {
            for (i, a) in ["X", "Y", "Z"].iter().enumerate() {
                out.push(Field::number(&format!("p-man-{i}"), a, p[i], Unit::Mm));
            }
        }
        Origin::World => {}
    }
    out.push(Field::section("Orientation"));
    let zi = axis_index(w.z);
    let mut zs: Vec<String> = AXES.iter().map(|(n, _)| format!("Z along {n}")).collect();
    zs.push("Normal of a picked face".into());
    out.push(Field::select(
        "p-z",
        "Tool axis (Z)",
        zs,
        zi.unwrap_or(AXES.len()),
    ));
    if zi.is_none() || picking {
        out.push(Field::picks(
            "p-face",
            "Planar face for Z",
            if zi.is_none() {
                vec![format!("Z ({:.3}, {:.3}, {:.3})", w.z[0], w.z[1], w.z[2])]
            } else {
                Vec::new()
            },
            picking,
        ));
    }
    let xs: Vec<String> = AXES.iter().map(|(n, _)| format!("X along {n}")).collect();
    out.push(Field::select(
        "p-x",
        "X direction",
        xs,
        axis_index(w.x).unwrap_or(2),
    ));
    if with_type {
        out.push(Field::select(
            "p-type",
            "Type",
            labels(&WorkPlaneType::ALL, WorkPlaneType::label),
            index_of(&WorkPlaneType::ALL, w.kind),
        ));
    }
    let mut datums = vec!["Automatic".to_string()];
    datums.extend((1..=6).map(cadrs_core::cam::cadrs_cam::post::datum_code));
    out.push(Field::select(
        "p-datum",
        "Override datum",
        datums,
        w.datum.map_or(0, |d| d.min(6) as usize),
    ));
    out.push(Field::check(
        "p-triad",
        "Show work coordinate system",
        w.show_triad,
    ));
}

fn set_plane(w: &mut WorkPlane, key: &str, v: &Value) -> bool {
    match (key, v) {
        ("p-origin", Value::Index(i)) => {
            w.origin = match i {
                0 => Origin::World,
                1 => Origin::ComponentBox([0, 0, 1]),
                2 => Origin::StockBox([0, 0, 1]),
                _ => Origin::Manual([0.0; 3]),
            }
        }
        (k, Value::Index(i)) if k.starts_with("p-box-") => {
            let a: usize = k[6..].parse().unwrap_or(0);
            let val = *i as i8 - 1;
            match &mut w.origin {
                Origin::ComponentBox(bp) | Origin::StockBox(bp) => bp[a] = val,
                _ => {}
            }
        }
        (k, Value::Number(n)) if k.starts_with("p-man-") => {
            let a: usize = k[6..].parse().unwrap_or(0);
            if let Origin::Manual(p) = &mut w.origin {
                p[a] = *n;
            }
        }
        ("p-z", Value::Index(i)) => {
            if let Some((_, a)) = AXES.get(*i) {
                w.z = *a;
                // Keep X across Z.
                if (w.z[0] * w.x[0] + w.z[1] * w.x[1] + w.z[2] * w.x[2]).abs() > 0.5 {
                    w.x = if w.z[0].abs() > 0.5 {
                        [0.0, 1.0, 0.0]
                    } else {
                        [1.0, 0.0, 0.0]
                    };
                }
            } else {
                return false;
            }
        }
        ("p-x", Value::Index(i)) => {
            if let Some((_, a)) = AXES.get(*i) {
                w.x = *a;
            }
        }
        ("p-type", Value::Index(i)) => w.kind = WorkPlaneType::ALL[(*i).min(2)],
        ("p-datum", Value::Index(i)) => w.datum = (*i > 0).then_some(*i as u32),
        ("p-triad", Value::Bool(b)) => w.show_triad = *b,
        _ => return false,
    }
    true
}

fn stock_direction_options() -> Vec<String> {
    let mut v = vec!["World".to_string()];
    v.extend(AXES.iter().skip(1).map(|(n, _)| format!("Z along {n}")));
    v
}

fn stock_direction_index(d: StockDirection) -> usize {
    match d {
        StockDirection::World => 0,
        StockDirection::Axes { z, .. } => axis_index(z).filter(|i| *i > 0).unwrap_or(0),
    }
}

impl Draft {
    /// The dialog's title.
    pub fn title(&self) -> String {
        match self {
            Draft::Insert(d) => format!("Insert {}", d.role.label().to_lowercase()),
            Draft::Component(c) => c.name.clone(),
            Draft::Job { job, .. } => job.name.clone(),
            Draft::Machine { .. } => "Machine".into(),
            Draft::Setup { setup, .. } => setup.name.clone(),
            Draft::Tool { op, .. } | Draft::Toolpath { op, .. } | Draft::WorkPlane { op, .. } => {
                op.name.clone()
            }
        }
    }

    /// What's wrong with it (✓ is off while there is).
    pub fn problem(&self, ctx: &Ctx) -> Option<String> {
        match self {
            Draft::Insert(d) => (!d.parts.iter().any(|p| p.2)).then(|| "Tick a part".into()),
            Draft::Job { job, .. } => {
                if job.name.trim().is_empty() {
                    Some("The job needs a name".into())
                } else if cadrs_core::cam::program::job_bodies(ctx.cam, job, ctx.parts).is_empty() {
                    Some("Pick the bodies to machine (or insert a part)".into())
                } else {
                    None
                }
            }
            Draft::Machine { machine, .. } => machine.check().err(),
            Draft::Tool { op, .. } => match &op.kind {
                OpKind::Tool(t) => t.tool.check().err(),
                _ => None,
            },
            Draft::Toolpath { op, setup, .. } => match &op.kind {
                OpKind::Toolpath(tp) => {
                    let tool = tool_above(ctx.cam, *setup, op.id);
                    match tool {
                        None => Some("No tool above the toolpath".into()),
                        Some(t) => tp.params.check(&t).err().or_else(|| {
                            let p = &tp.params;
                            let needs_edges =
                                matches!(p.pattern, Pattern::Engrave) && p.drive.edges.is_empty();
                            let needs_faces =
                                p.pattern == Pattern::Fill && p.drive.faces.is_empty();
                            if needs_edges {
                                Some("Pick the edges to engrave".into())
                            } else if needs_faces {
                                Some("Pick the faces to fill".into())
                            } else {
                                None
                            }
                        }),
                    }
                }
                _ => None,
            },
            _ => None,
        }
    }

    /// The dialog's fields.
    pub fn fields(&self, ctx: &Ctx, picking: Option<&str>, tab: usize) -> Vec<Field> {
        let mut f = Vec::new();
        match self {
            Draft::Insert(d) => {
                f.push(Field::select(
                    "role",
                    "Insert as",
                    labels(&Role::ALL, Role::label),
                    index_of(&Role::ALL, d.role),
                ));
                let studios = ctx.part_studios();
                f.push(Field::select(
                    "studio",
                    "Part Studio",
                    studios.iter().map(|s| s.1.clone()),
                    d.studio,
                ));
                let mut refs = vec!["Workspace (follows edits)".to_string()];
                refs.extend(ctx.versions.iter().map(|v| format!("Version {}", v.0)));
                f.push(Field::select("version", "Reference", refs, d.version));
                if d.version == 0 {
                    f.push(Field::note(
                        "A version keeps the part as it is; the workspace follows every edit.",
                    ));
                }
                f.push(Field::section("Parts"));
                if d.parts.is_empty() {
                    f.push(Field::error("The Part Studio has no parts"));
                }
                for (i, (_, name, on)) in d.parts.iter().enumerate() {
                    f.push(Field::check(&format!("part-{i}"), name, *on));
                }
            }
            Draft::Component(c) => {
                f.push(Field::text("name", "Name", &c.name));
                f.push(Field::select(
                    "role",
                    "Used as",
                    labels(&Role::ALL, Role::label),
                    index_of(&Role::ALL, c.role),
                ));
                f.push(Field::note(format!("Reference: {}", c.reference_label())));
            }
            Draft::Job { job, .. } => {
                f.push(Field::text("name", "Job name", &job.name));
                f.push(Field::picks(
                    "bodies",
                    "Bodies (all parts when empty)",
                    job.bodies.iter().map(|p| ctx.part_name(*p)).collect(),
                    picking == Some("bodies"),
                ));
                f.push(Field::section("Stock"));
                let stocks: Vec<&cadrs_core::cam::Component> = ctx
                    .cam
                    .components
                    .iter()
                    .filter(|c| c.role == Role::Stock)
                    .collect();
                let kinds = ["Box", "Cylinder", "User defined"];
                let k = match job.stock.kind {
                    StockKind::Box => 0,
                    StockKind::Cylinder => 1,
                    StockKind::Component(_) => 2,
                };
                f.push(Field::select("stock-kind", "Stock type", kinds, k));
                match job.stock.kind {
                    StockKind::Component(id) => {
                        if stocks.is_empty() {
                            f.push(Field::error("Insert a stock component first"));
                        } else {
                            f.push(Field::select(
                                "stock-comp",
                                "Stock body",
                                stocks.iter().map(|c| c.name.clone()),
                                stocks.iter().position(|c| c.id == id).unwrap_or(0),
                            ));
                        }
                    }
                    _ => {
                        f.push(Field::select(
                            "stock-dir",
                            "Stock direction",
                            stock_direction_options(),
                            stock_direction_index(job.stock.direction),
                        ));
                        if job.stock.kind == StockKind::Cylinder {
                            f.push(Field::number(
                                "radial",
                                "Radial offset",
                                job.stock.radial,
                                Unit::Mm,
                            ));
                            f.push(Field::number(
                                "off-4",
                                "Offset −Z",
                                job.stock.offsets[4],
                                Unit::Mm,
                            ));
                            f.push(Field::number(
                                "off-5",
                                "Offset +Z",
                                job.stock.offsets[5],
                                Unit::Mm,
                            ));
                        } else {
                            for (i, n) in [
                                "Offset −X",
                                "Offset +X",
                                "Offset −Y",
                                "Offset +Y",
                                "Offset −Z",
                                "Offset +Z",
                            ]
                            .iter()
                            .enumerate()
                            {
                                f.push(Field::number(
                                    &format!("off-{i}"),
                                    n,
                                    job.stock.offsets[i],
                                    Unit::Mm,
                                ));
                            }
                        }
                        let bodies = cadrs_core::cam::program::job_bodies(ctx.cam, job, ctx.parts);
                        if !bodies.is_empty()
                            && let Some(s) =
                                cadrs_core::cam::program::stock_shape(ctx.cam, job, ctx.parts)
                        {
                            let b = s.bounds_in(&job.stock.frame());
                            let z = b.size();
                            f.push(Field::note(format!(
                                "{} × {} × {} mm",
                                fmt(z[0]),
                                fmt(z[1]),
                                fmt(z[2])
                            )));
                        }
                    }
                }
                f.push(Field::section("Workholding"));
                f.push(Field::picks(
                    "workholdings",
                    "Workholding bodies",
                    job.workholdings.iter().map(|p| ctx.part_name(*p)).collect(),
                    picking == Some("workholdings"),
                ));
            }
            Draft::Machine {
                machine, by_kind, ..
            } => {
                let cat = catalog();
                f.push(Field::check("by-kind", "Group by machine type", *by_kind));
                let mut order: Vec<usize> = (0..cat.len()).collect();
                if *by_kind {
                    order.sort_by_key(|i| index_of(&MachineKind::ALL, cat[*i].kind));
                } else {
                    order.sort_by_key(|i| cat[*i].title());
                }
                let opts: Vec<String> = order
                    .iter()
                    .map(|i| {
                        if *by_kind {
                            format!("{}: {}", cat[*i].kind.label(), cat[*i].model)
                        } else {
                            cat[*i].title()
                        }
                    })
                    .collect();
                let current = order
                    .iter()
                    .position(|i| cat[*i].title() == machine.title())
                    .unwrap_or(0);
                f.push(Field::select("machine", "Machine", opts, current));
                f.push(Field::note(format!(
                    "{} · {} controller",
                    machine.kind.label(),
                    machine.dialect.label()
                )));
                f.push(Field::section("Machine"));
                for (i, a) in ["X", "Y", "Z"].iter().enumerate() {
                    f.push(Field::number(
                        &format!("travel-{i}"),
                        &format!("{a} travel"),
                        machine.travel[i],
                        Unit::Mm,
                    ));
                }
                f.push(Field::number(
                    "rapid",
                    "Rapid rate",
                    machine.rapid_rate,
                    Unit::Feed,
                ));
                f.push(Field::number(
                    "max-feed",
                    "Fastest feed",
                    machine.max_feed,
                    Unit::Feed,
                ));
                if machine.kind.is_milling() {
                    f.push(Field::number(
                        "max-spindle",
                        "Fastest spindle",
                        machine.max_spindle,
                        Unit::Rpm,
                    ));
                    f.push(Field::check(
                        "changer",
                        "Automatic tool changer",
                        machine.tool_changer,
                    ));
                } else if machine.kind == MachineKind::Laser {
                    f.push(Field::number(
                        "max-spindle",
                        "S at full power",
                        machine.max_spindle,
                        Unit::None,
                    ));
                }
                let s = &machine.post;
                f.push(Field::section("Post settings"));
                f.push(Field::select(
                    "unit",
                    "Output unit",
                    ["Millimetres", "Inches"],
                    usize::from(s.output_unit == LenUnit::Inch),
                ));
                f.push(Field::check(
                    "adv-wp",
                    "Advanced work planes (own datums)",
                    s.advanced_work_planes,
                ));
                f.push(Field::check("seq", "Sequence numbers", s.sequence_numbers));
                if s.sequence_numbers {
                    f.push(Field::count(
                        "block-start",
                        "Block number start",
                        s.block_start as i64,
                    ));
                    f.push(Field::count(
                        "block-inc",
                        "Block increment",
                        s.block_increment as i64,
                    ));
                    f.push(Field::check(
                        "tools-only",
                        "Numbers at tool changes only",
                        s.numbers_at_tools_only,
                    ));
                }
                f.push(Field::check(
                    "rapid-pos",
                    "Rapid positioning (G0)",
                    s.rapid_positioning,
                ));
                if !s.rapid_positioning {
                    f.push(Field::number(
                        "post-rapid",
                        "Rapid feed (G1)",
                        s.rapid_rate,
                        Unit::Feed,
                    ));
                }
                f.push(Field::check("arcs", "Output arcs (G2/G3)", s.output_arcs));
                if s.output_arcs {
                    f.push(Field::check("helices", "Output helices", s.output_helices));
                }
                f.push(Field::count(
                    "decimals",
                    "Decimal places",
                    s.decimals as i64,
                ));
                f.push(Field::check(
                    "dec-point",
                    "Always write a decimal point",
                    s.decimal_point,
                ));
                if machine.kind.is_milling() {
                    f.push(Field::check(
                        "ret-tool",
                        "Retract on tool changes",
                        s.force_retract_on_tool_change,
                    ));
                    f.push(Field::check(
                        "ret-plane",
                        "Retract on work plane changes",
                        s.force_retract_on_plane_change,
                    ));
                    f.push(Field::check(
                        "ret-end",
                        "Retract at the program end",
                        s.force_retract_on_end,
                    ));
                    if machine.dialect.has_cycles() {
                        f.push(Field::section("Fixed cycles"));
                        f.push(Field::check(
                            "cy-drill",
                            "Drilling canned cycles",
                            s.canned_drill,
                        ));
                        f.push(Field::check(
                            "cy-peck",
                            "Peck drilling canned cycles",
                            s.canned_peck,
                        ));
                        f.push(Field::check(
                            "cy-tap",
                            "Thread tapping canned cycles",
                            s.canned_tap,
                        ));
                        f.push(Field::check(
                            "cy-chip",
                            "Chip break canned cycles",
                            s.canned_chip_break,
                        ));
                    }
                }
                if machine.kind == MachineKind::Laser {
                    f.push(Field::select(
                        "laser-mode",
                        "Laser power",
                        ["Dynamic (M4)", "Constant (M3)"],
                        usize::from(s.laser_mode == LaserMode::Constant),
                    ));
                }
                f.push(Field::check(
                    "comments",
                    "Comments in the program",
                    s.comments,
                ));
            }
            Draft::Setup { setup, job, .. } => {
                f.push(Field::text("name", "Setup name", &setup.name));
                f.push(Field::count(
                    "program",
                    "Program number",
                    setup.program_number as i64,
                ));
                let man = matches!(setup.mating, Mating::Manual(_));
                f.push(Field::select(
                    "mating",
                    "Machine mating",
                    ["Automatic", "Manual"],
                    usize::from(man),
                ));
                if let Mating::Manual(o) = setup.mating {
                    for (i, a) in ["Offset X", "Offset Y", "Offset Z"].iter().enumerate() {
                        f.push(Field::number(&format!("mate-{i}"), a, o[i], Unit::Mm));
                    }
                }
                plane_fields(&mut f, &setup.wcs, picking == Some("p-face"), false);
                if ctx
                    .cam
                    .job(*job)
                    .and_then(|j| j.machine.as_ref())
                    .is_some_and(|m| m.kind.is_cutting())
                    && setup.wcs.z != [0.0, 0.0, 1.0]
                {
                    f.push(Field::note(
                        "A cutting machine cuts down its Z: lay the sheet flat.",
                    ));
                }
            }
            Draft::WorkPlane { op, .. } => {
                f.push(Field::text("name", "Name", &op.name));
                if let OpKind::WorkPlane(w) = &op.kind {
                    plane_fields(&mut f, w, picking == Some("p-face"), true);
                }
            }
            Draft::Tool {
                op,
                library,
                filter,
                setup,
                ..
            } => {
                let OpKind::Tool(t) = &op.kind else {
                    return f;
                };
                let machine = machine_of(ctx.cam, *setup);
                let lib: Vec<(usize, &Tool)> = ctx
                    .cam
                    .library
                    .filtered(*filter, None)
                    .into_iter()
                    .filter(|(_, x)| x.kind.fits(machine))
                    .collect();
                f.push(Field::check("library", "From the tool library", *library));
                if *library {
                    let mut types = vec!["All".to_string()];
                    types.extend(labels(&MachiningType::ALL, MachiningType::label));
                    f.push(Field::select(
                        "lib-type",
                        "Machining type",
                        types,
                        filter.map_or(0, |m| index_of(&MachiningType::ALL, m) + 1),
                    ));
                    if lib.is_empty() {
                        f.push(Field::note("No tool of this type for this machine yet"));
                    } else {
                        let cur = lib.iter().position(|(_, x)| **x == t.tool);
                        let mut opts: Vec<String> =
                            lib.iter().map(|(_, x)| x.name.clone()).collect();
                        if cur.is_none() {
                            opts.insert(0, "Choose a tool…".into());
                        }
                        f.push(Field::select("lib-tool", "Tool", opts, cur.unwrap_or(0)));
                    }
                    f.push(Field::button(
                        "lib-import",
                        "Import tool library…",
                        "file-import",
                    ));
                    f.push(Field::button(
                        "lib-export",
                        "Export tool library…",
                        "file-export",
                    ));
                }
                let tool = &t.tool;
                f.push(Field::section("Cutter"));
                f.push(Field::text("name", "Name", &op.name));
                let kinds = ToolKind::for_machine(machine);
                f.push(Field::select(
                    "kind",
                    "Cutter type",
                    labels(&kinds, ToolKind::label),
                    index_of(&kinds, tool.kind),
                ));
                if tool.is_beam() {
                    f.push(Field::number(
                        "diameter",
                        "Kerf width",
                        tool.diameter,
                        Unit::Mm,
                    ));
                } else {
                    f.push(Field::select(
                        "unit",
                        "Unit",
                        ["Millimetres", "Inches"],
                        usize::from(tool.unit == LenUnit::Inch),
                    ));
                    f.push(Field::select(
                        "hand",
                        "Tool hand",
                        ["Right", "Left"],
                        usize::from(tool.hand == Hand::Left),
                    ));
                    f.push(Field::number(
                        "diameter",
                        "Diameter",
                        tool.diameter,
                        Unit::Mm,
                    ));
                    match tool.kind {
                        ToolKind::BullMill => {
                            f.push(Field::number(
                                "corner",
                                "Corner radius",
                                tool.corner_radius,
                                Unit::Mm,
                            ));
                        }
                        ToolKind::ChamferMill | ToolKind::TaperMill => {
                            f.push(Field::number(
                                "angle",
                                "Included angle",
                                tool.angle,
                                Unit::Deg,
                            ));
                            f.push(Field::number(
                                "tip",
                                "Tip diameter",
                                tool.tip_diameter,
                                Unit::Mm,
                            ));
                        }
                        ToolKind::Drill => {
                            f.push(Field::number("angle", "Point angle", tool.angle, Unit::Deg));
                        }
                        ToolKind::Tap => {
                            f.push(Field::number("pitch", "Thread pitch", tool.pitch, Unit::Mm));
                        }
                        _ => {}
                    }
                    f.push(Field::number(
                        "cut-len",
                        "Cutting length",
                        tool.cutting_length,
                        Unit::Mm,
                    ));
                    f.push(Field::number(
                        "overall",
                        "Overall length",
                        tool.overall_length,
                        Unit::Mm,
                    ));
                    f.push(Field::count("flutes", "Flutes", tool.flutes as i64));
                    f.push(Field::check(
                        "center",
                        "Center cutting",
                        tool.center_cutting,
                    ));
                }
                if tool.is_beam() {
                    f.push(Field::section("Beam"));
                    if tool.kind != ToolKind::Plasma {
                        f.push(Field::number(
                            "power",
                            "Power",
                            tool.beam.power,
                            Unit::Percent,
                        ));
                    }
                    f.push(Field::number(
                        "pierce",
                        "Pierce delay",
                        tool.beam.pierce_delay,
                        Unit::Seconds,
                    ));
                    if tool.kind == ToolKind::Plasma {
                        f.push(Field::number(
                            "pierce-h",
                            "Pierce height",
                            tool.beam.pierce_height,
                            Unit::Mm,
                        ));
                        f.push(Field::check("thc", "Torch height control", tool.beam.thc));
                        f.push(Field::count(
                            "material",
                            "Material number (0: as set)",
                            tool.beam.material as i64,
                        ));
                    }
                    if tool.kind != ToolKind::Laser {
                        f.push(Field::number(
                            "cut-h",
                            "Cut height",
                            tool.beam.cut_height,
                            Unit::Mm,
                        ));
                    } else {
                        f.push(Field::check("air", "Air assist", tool.beam.air_assist));
                        f.push(Field::count(
                            "beam-passes",
                            "Passes",
                            tool.beam.passes as i64,
                        ));
                    }
                }
                f.push(Field::section("Cutting data"));
                if !tool.is_beam() {
                    f.push(Field::number(
                        "rpm",
                        "Spindle speed",
                        tool.cutting.spindle_rpm,
                        Unit::Rpm,
                    ));
                }
                f.push(Field::number(
                    "feed",
                    "Feed rate",
                    tool.cutting.feed,
                    Unit::Feed,
                ));
                if !tool.is_beam() {
                    f.push(Field::number(
                        "plunge",
                        "Plunge rate",
                        tool.cutting.plunge_feed,
                        Unit::Feed,
                    ));
                    f.push(Field::number(
                        "stepdown",
                        "Step down",
                        tool.cutting.step_down,
                        Unit::Mm,
                    ));
                    f.push(Field::number(
                        "stepover",
                        "Step over",
                        tool.cutting.step_over,
                        Unit::Mm,
                    ));
                    f.push(Field::section("Holder"));
                    f.push(Field::check("holder", "Holder", tool.holder.is_some()));
                    if let Some(h) = &tool.holder {
                        let holders: Vec<String> = ctx
                            .cam
                            .library
                            .holders
                            .iter()
                            .map(|x| x.name.clone())
                            .collect();
                        if !holders.is_empty() {
                            let mut opts = vec!["Cylinder".to_string()];
                            opts.extend(holders);
                            let cur = ctx
                                .cam
                                .library
                                .holders
                                .iter()
                                .position(|x| x == h)
                                .map_or(0, |i| i + 1);
                            f.push(Field::select("holder-lib", "Holder", opts, cur));
                        }
                        f.push(Field::number(
                            "holder-d",
                            "Holder diameter",
                            h.max_radius() * 2.0,
                            Unit::Mm,
                        ));
                        f.push(Field::number(
                            "holder-l",
                            "Holder length",
                            h.length(),
                            Unit::Mm,
                        ));
                    }
                }
                f.push(Field::section("Assembly"));
                f.push(Field::count(
                    "number",
                    "Assembly number (T)",
                    t.number as i64,
                ));
            }
            Draft::Toolpath {
                op, kind, setup, ..
            } => {
                let OpKind::Toolpath(tp) = &op.kind else {
                    return f;
                };
                let p = &tp.params;
                let machine = machine_of(ctx.cam, *setup);
                let tool = tool_above(ctx.cam, *setup, op.id);
                let tk = tool.as_ref().map_or(ToolKind::EndMill, |t| t.kind);
                let beam = tk.is_beam();
                let mut tabs = vec!["Tool path", "Link", "Misc"];
                if beam {
                    tabs.push("Beam");
                }
                f.push(Field::Tabs {
                    key: "tab".into(),
                    labels: tabs.iter().map(|s| s.to_string()).collect(),
                    selected: tab.min(tabs.len() - 1),
                });
                match tab {
                    1 => link_fields(&mut f, p, beam),
                    2 => {
                        f.push(Field::section("Quality"));
                        f.push(Field::number(
                            "tolerance",
                            "Cut tolerance",
                            p.tolerance,
                            Unit::Mm,
                        ));
                        if !beam {
                            f.push(Field::section("Smoothing"));
                            f.push(Field::number(
                                "smooth",
                                "Smooth corners",
                                p.smooth_corners,
                                Unit::Mm,
                            ));
                        }
                    }
                    3 if beam => {
                        f.push(Field::section("Beam"));
                        f.push(Field::number(
                            "beam-power",
                            "Power",
                            p.beam.power,
                            Unit::Percent,
                        ));
                        f.push(Field::count("beam-passes", "Passes", p.beam.passes as i64));
                        if p.beam.passes > 1 {
                            f.push(Field::number(
                                "pass-depth",
                                "Depth per pass",
                                p.beam.pass_depth,
                                Unit::Mm,
                            ));
                        }
                        if p.pattern == Pattern::Cut {
                            f.push(Field::count(
                                "micro",
                                "Microjoints per part",
                                p.beam.microjoints as i64,
                            ));
                            if p.beam.microjoints > 0 {
                                f.push(Field::number(
                                    "micro-w",
                                    "Microjoint width",
                                    p.beam.microjoint_width,
                                    Unit::Mm,
                                ));
                            }
                            f.push(Field::number(
                                "small-hole",
                                "Small holes under",
                                p.beam.small_hole,
                                Unit::Mm,
                            ));
                            if p.beam.small_hole > 0.0 {
                                f.push(Field::number(
                                    "small-feed",
                                    "Small hole feed",
                                    p.beam.small_hole_feed * 100.0,
                                    Unit::Percent,
                                ));
                            }
                        }
                    }
                    _ => toolpath_fields(&mut f, ctx, op, p, *kind, tk, machine, picking),
                }
                if tool.is_none() {
                    f.push(Field::error("No tool above the toolpath"));
                }
            }
        }
        if let Some(e) = self.problem(ctx) {
            f.push(Field::error(e));
        }
        f
    }
}

fn fmt(v: f64) -> String {
    form::fmt_num(v, Unit::None)
}

fn machine_of(cam: &CamStudio, setup: CamId) -> MachineKind {
    cam.setup(setup)
        .and_then(|(j, _)| j.machine.as_ref())
        .map_or(MachineKind::Mill, |m| m.kind)
}

/// The tool a toolpath draft would use (the one above it, or above where it goes).
fn tool_above(cam: &CamStudio, setup: CamId, op: CamId) -> Option<Tool> {
    if let Some(t) = cam.tool_for(op) {
        return Some(t.tool.clone());
    }
    cam.setup(setup).and_then(|(_, s)| {
        s.operations
            .iter()
            .rev()
            .filter(|o| !o.suppressed)
            .find_map(|o| o.tool().map(|t| t.tool.clone()))
    })
}

#[allow(clippy::too_many_arguments)]
fn toolpath_fields(
    f: &mut Vec<Field>,
    ctx: &Ctx,
    op: &Operation,
    p: &Params,
    kind: Type,
    tk: ToolKind,
    machine: MachineKind,
    picking: Option<&str>,
) {
    f.push(Field::text("name", "Name", &op.name));
    let types: Vec<Type> = Type::ALL
        .into_iter()
        .filter(|t| !Pattern::available(*t, tk, machine).is_empty())
        .collect();
    f.push(Field::select(
        "type",
        "Type",
        labels(&types, Type::label),
        index_of(&types, kind),
    ));
    let pats = Pattern::available(kind, tk, machine);
    f.push(Field::select(
        "pattern",
        "Pattern",
        labels(&pats, Pattern::label),
        index_of(&pats, p.pattern),
    ));
    use Pattern::*;
    let pat = p.pattern;
    if matches!(pat, Rough2D | Rough3D) {
        f.push(Field::select(
            "sub",
            "Sub-pattern",
            labels(&SubPattern::ALL, SubPattern::label),
            index_of(&SubPattern::ALL, p.sub_pattern),
        ));
    }
    // Drive geometry.
    let (placeholder, items): (&str, Vec<String>) = match kind {
        Type::Edges => (
            "Drive edges (the part's outline when empty)",
            p.drive
                .edges
                .iter()
                .map(|k| entity_label(ctx, k, "Edge"))
                .collect(),
        ),
        Type::Faces => (
            match pat {
                Fill => "Faces to fill",
                Face => "Faces (the stock's top when empty)",
                _ => "Floor faces (all when empty)",
            },
            p.drive
                .faces
                .iter()
                .map(|k| entity_label(ctx, k, "Face"))
                .collect(),
        ),
        Type::Bodies => (
            "Faces to finish (all when empty)",
            p.drive
                .faces
                .iter()
                .map(|k| entity_label(ctx, k, "Face"))
                .collect(),
        ),
        Type::Holes => (
            "Holes (by the cycle's selection when empty)",
            p.drive
                .holes
                .iter()
                .map(|h| format!("Hole at ({}, {})", fmt(h[0]), fmt(h[1])))
                .collect(),
        ),
    };
    if pat != Engrave || kind == Type::Edges {
        f.push(Field::picks(
            "drive",
            placeholder,
            items,
            picking == Some("drive"),
        ));
    }
    if kind == Type::Edges && !p.drive.edges.is_empty() {
        f.push(Field::check(
            "reverse",
            "Reverse direction",
            p.drive.reverse,
        ));
    }
    if matches!(pat, Profile2D | Cut) {
        f.push(Field::select(
            "side",
            "Side",
            labels(&Side::ALL, Side::label),
            index_of(&Side::ALL, p.side),
        ));
    }
    let beam = tk.is_beam();
    if !beam && !matches!(pat, Drill) {
        f.push(Field::section("Offset"));
        if !matches!(pat, Face | Engrave | Chamfer2D) {
            f.push(Field::number(
                "wall",
                "Wall stock to leave",
                p.wall_stock,
                Unit::Mm,
            ));
        }
        if !matches!(pat, Engrave | Chamfer2D) {
            f.push(Field::number(
                "floor",
                "Floor stock to leave",
                p.floor_stock,
                Unit::Mm,
            ));
        }
    } else if beam && pat == Cut {
        f.push(Field::number(
            "wall",
            "Extra offset",
            p.wall_stock,
            Unit::Mm,
        ));
    }
    if matches!(
        pat,
        Rough2D | Rough3D | Profile2D | ConstantZ | Face | Engrave | Bore
    ) {
        f.push(Field::section("Step down"));
        f.push(Field::number(
            "stepdown",
            if pat == Bore {
                "Helix pitch"
            } else {
                "Step down"
            },
            p.step_down,
            Unit::Mm,
        ));
        if matches!(pat, Rough2D | Rough3D | Profile2D) {
            f.push(Field::number(
                "first",
                "First step down (0: same)",
                p.first_step_down,
                Unit::Mm,
            ));
        }
    }
    if matches!(
        pat,
        Rough2D | Rough3D | Face | Flatlands | Parallel | Fill | Profile2D
    ) {
        f.push(Field::section("Step over"));
        f.push(Field::number(
            "stepover",
            if pat == Fill {
                "Line spacing"
            } else {
                "Step over"
            },
            p.step_over,
            Unit::Mm,
        ));
        if pat == Profile2D {
            f.push(Field::count("passes", "Passes", p.passes as i64));
        }
    }
    if !matches!(pat, Drill) {
        f.push(Field::section("Sorting"));
        f.push(Field::select(
            "direction",
            if beam {
                "Direction"
            } else {
                "Cutting direction"
            },
            if beam {
                vec![
                    "Part on the right".to_string(),
                    "Part on the left".to_string(),
                ]
            } else {
                labels(&Direction::ALL, Direction::label)
            },
            index_of(&Direction::ALL, p.direction),
        ));
        let rastered = matches!(pat, Face | Parallel | Fill)
            || (matches!(pat, Rough2D | Rough3D) && p.sub_pattern == SubPattern::Parallel);
        if rastered {
            f.push(Field::select(
                "method",
                "Cutting method",
                labels(&CuttingMethod::ALL, CuttingMethod::label),
                index_of(&CuttingMethod::ALL, p.cutting_method),
            ));
            f.push(Field::number("angle", "Angle", p.angle, Unit::Deg));
        }
        if matches!(pat, Rough2D | Rough3D | Profile2D) {
            f.push(Field::select(
                "machine-by",
                "Machine by",
                labels(&MachineBy::ALL, MachineBy::label),
                index_of(&MachineBy::ALL, p.machine_by),
            ));
        }
    }
    if !beam && !matches!(pat, Drill | Bore | Chamfer2D | Engrave) {
        f.push(Field::section("Heights"));
        for (k, name, h) in [
            ("top", "Top", p.heights.top),
            ("bottom", "Bottom", p.heights.bottom),
        ] {
            f.push(Field::select(
                &format!("{k}-from"),
                &format!("{name} from"),
                labels(&HeightRef::ALL, HeightRef::label),
                index_of(&HeightRef::ALL, h.from),
            ));
            f.push(Field::number(
                &format!("{k}-off"),
                &format!("{name} offset"),
                h.offset,
                Unit::Mm,
            ));
        }
    }
    match pat {
        Chamfer2D => {
            f.push(Field::section("Chamfer"));
            f.push(Field::number("depth", "Chamfer width", p.depth, Unit::Mm));
            f.push(Field::number(
                "tip-off",
                "Tip offset",
                p.tip_offset,
                Unit::Mm,
            ));
        }
        Engrave => {
            f.push(Field::section("Engrave"));
            f.push(Field::number("depth", "Depth", p.depth, Unit::Mm));
        }
        Profile2D => {
            f.push(Field::section("Tabs"));
            f.push(Field::check("tabs", "Tabs", p.tabs.enabled));
            if p.tabs.enabled {
                f.push(Field::count(
                    "tab-count",
                    "Tabs per contour",
                    p.tabs.count as i64,
                ));
                f.push(Field::number("tab-w", "Tab width", p.tabs.width, Unit::Mm));
                f.push(Field::number(
                    "tab-h",
                    "Tab height",
                    p.tabs.height,
                    Unit::Mm,
                ));
            }
        }
        Drill | Bore => {
            f.push(Field::section(if pat == Drill { "Cycle" } else { "Holes" }));
            f.push(Field::select(
                "hole-select",
                "Holes",
                labels(&HoleSelect::ALL, HoleSelect::label),
                index_of(&HoleSelect::ALL, p.cycle.select),
            ));
            if pat == Drill {
                let cycles = [
                    CycleKind::Drill,
                    CycleKind::Peck,
                    CycleKind::ChipBreak,
                    CycleKind::Tap,
                ];
                f.push(Field::select(
                    "cycle",
                    "Cycle",
                    labels(&cycles, CycleKind::label),
                    index_of(&cycles, p.cycle.kind),
                ));
                if matches!(p.cycle.kind, CycleKind::Peck | CycleKind::ChipBreak) {
                    f.push(Field::number("peck", "Peck depth", p.cycle.peck, Unit::Mm));
                }
                f.push(Field::number(
                    "dwell",
                    "Dwell",
                    p.cycle.dwell,
                    Unit::Seconds,
                ));
                f.push(Field::check(
                    "retract-full",
                    "Retract to clearance between holes",
                    p.cycle.retract_full,
                ));
            }
            f.push(Field::number(
                "break",
                "Break through",
                p.cycle.break_through,
                Unit::Mm,
            ));
        }
        _ => {}
    }
    f.push(Field::section("Speeds"));
    if !beam {
        f.push(Field::number(
            "rpm",
            "Spindle speed",
            p.speeds.spindle_rpm,
            Unit::Rpm,
        ));
    }
    f.push(Field::number(
        "feed",
        "Feed rate",
        p.speeds.feed,
        Unit::Feed,
    ));
    if !beam {
        f.push(Field::number(
            "plunge",
            "Plunge rate",
            p.speeds.plunge,
            Unit::Feed,
        ));
        if !matches!(pat, Drill) {
            f.push(Field::number(
                "ramp-feed",
                "Ramp rate",
                p.speeds.ramp,
                Unit::Feed,
            ));
        }
    }
    f.push(Field::number(
        "lead-feed",
        "Lead rate",
        p.speeds.lead,
        Unit::Feed,
    ));
    f.push(Field::section("Posting"));
    if !beam {
        f.push(Field::select(
            "coolant",
            "Coolant",
            labels(&Coolant::ALL, Coolant::label),
            index_of(&Coolant::ALL, p.posting.coolant),
        ));
        f.push(Field::check(
            "force-tc",
            "Force tool change",
            p.posting.force_tool_change,
        ));
    }
    f.push(Field::check(
        "stop",
        "Stop before (M0)",
        p.posting.stop_before,
    ));
    f.push(Field::text("comment", "Comment", &p.posting.comment));
    if matches!(pat, Rough2D | Rough3D) {
        f.push(Field::check(
            "rest",
            "Rest machining (what the operations above left)",
            p.rest,
        ));
    }
}

fn link_fields(f: &mut Vec<Field>, p: &Params, beam: bool) {
    let l = &p.links;
    f.push(Field::section("Clearance area"));
    f.push(Field::number(
        "clearance",
        "Clearance above stock",
        l.clearance,
        Unit::Mm,
    ));
    if !beam {
        f.push(Field::number(
            "rapid-d",
            "Rapid distance",
            l.rapid_distance,
            Unit::Mm,
        ));
        f.push(Field::number(
            "feed-d",
            "Feed distance",
            l.feed_distance,
            Unit::Mm,
        ));
        f.push(Field::section("Gaps"));
        f.push(Field::number(
            "gap",
            "Small gaps up to",
            l.small_gap,
            Unit::Mm,
        ));
        let acts = labels(&LinkAction::ALL, LinkAction::label);
        f.push(Field::select(
            "small",
            "Small gaps",
            acts.clone(),
            index_of(&LinkAction::ALL, l.small_gaps),
        ));
        f.push(Field::select(
            "large",
            "Large gaps",
            acts,
            index_of(&LinkAction::ALL, l.large_gaps),
        ));
        f.push(Field::section("First entry"));
        f.push(Field::select(
            "entry",
            "Entry",
            labels(&Entry::ALL, Entry::label),
            index_of(&Entry::ALL, l.entry),
        ));
        if l.entry == Entry::Ramp {
            f.push(Field::number(
                "ramp-angle",
                "Ramp angle",
                l.ramp_angle,
                Unit::Deg,
            ));
        }
    }
    f.push(Field::section("Leads"));
    let leads = labels(&LeadKind::ALL, LeadKind::label);
    f.push(Field::select(
        "lead-in",
        "Lead in",
        leads.clone(),
        index_of(&LeadKind::ALL, l.lead_in),
    ));
    f.push(Field::select(
        "lead-out",
        "Lead out",
        leads,
        index_of(&LeadKind::ALL, l.lead_out),
    ));
    if l.lead_in != LeadKind::None || l.lead_out != LeadKind::None {
        f.push(Field::number(
            "lead-r",
            "Lead radius",
            l.lead_radius,
            Unit::Mm,
        ));
    }
    f.push(Field::number("overlap", "Overlap", l.overlap, Unit::Mm));
}

/// A drive entity's label: the face or edge and its part.
fn entity_label(ctx: &Ctx, key: &str, what: &str) -> String {
    match pick_of_key(key).and_then(|p| p.part()) {
        Some(part) => format!("{what} of {}", ctx.part_name(part)),
        None => what.to_string(),
    }
}

/// The view pick a drive key names (for highlighting it).
pub fn pick_of_key(key: &str) -> Option<Pick> {
    let mut it = key.splitn(3, '/');
    let feature = it.next()?.parse::<uuid::Uuid>().ok()?;
    let index = it.next()?.parse::<u32>().ok()?;
    let rest = it.next()?;
    let part = PartId::new(cadrs_core::FeatureId(feature), index);
    if let Ok(f) = ron::from_str::<cadrs_core::solid::FaceName>(rest) {
        return Some(Pick::Face(part, f));
    }
    ron::from_str::<cadrs_core::solid::EdgeName>(rest)
        .ok()
        .map(|e| Pick::Edge(part, e))
}

// ---------------------------------------------------------------------------------------------
// Changes

impl Draft {
    /// Takes a changed field. Returns whether the dialog's layout may have changed.
    fn set(&mut self, world: &mut World, key: &str, v: &Value, ctx_cam: &CamStudio) {
        match self {
            Draft::Insert(d) => match (key, v) {
                ("role", Value::Index(i)) => d.role = Role::ALL[(*i).min(2)],
                ("studio", Value::Index(i)) => {
                    d.studio = *i;
                    fill_parts(world, d);
                }
                ("version", Value::Index(i)) => {
                    d.version = *i;
                    fill_parts(world, d);
                }
                (k, Value::Bool(b)) if k.starts_with("part-") => {
                    if let Some(p) = k[5..]
                        .parse::<usize>()
                        .ok()
                        .and_then(|i| d.parts.get_mut(i))
                    {
                        p.2 = *b;
                    }
                }
                _ => {}
            },
            Draft::Component(c) => match (key, v) {
                ("name", Value::Text(t)) => c.name = t.clone(),
                ("role", Value::Index(i)) => c.role = Role::ALL[(*i).min(2)],
                _ => {}
            },
            Draft::Job { job, .. } => match (key, v) {
                ("name", Value::Text(t)) => job.name = t.clone(),
                ("bodies", Value::Remove(i)) => {
                    if *i < job.bodies.len() {
                        job.bodies.remove(*i);
                    }
                }
                ("workholdings", Value::Remove(i)) => {
                    if *i < job.workholdings.len() {
                        job.workholdings.remove(*i);
                    }
                }
                ("stock-kind", Value::Index(i)) => {
                    job.stock.kind = match i {
                        0 => StockKind::Box,
                        1 => StockKind::Cylinder,
                        _ => ctx_cam
                            .components
                            .iter()
                            .find(|c| c.role == Role::Stock)
                            .map_or(StockKind::Component(CamId(0)), |c| {
                                StockKind::Component(c.id)
                            }),
                    }
                }
                ("stock-comp", Value::Index(i)) => {
                    if let Some(c) = ctx_cam
                        .components
                        .iter()
                        .filter(|c| c.role == Role::Stock)
                        .nth(*i)
                    {
                        job.stock.kind = StockKind::Component(c.id);
                    }
                }
                ("stock-dir", Value::Index(i)) => {
                    job.stock.direction = if *i == 0 {
                        StockDirection::World
                    } else {
                        let z = AXES[*i].1;
                        let x = if z[0].abs() > 0.5 {
                            [0.0, 1.0, 0.0]
                        } else {
                            [1.0, 0.0, 0.0]
                        };
                        StockDirection::Axes { z, x }
                    }
                }
                ("radial", Value::Number(n)) => job.stock.radial = n.max(0.0),
                (k, Value::Number(n)) if k.starts_with("off-") => {
                    if let Ok(i) = k[4..].parse::<usize>() {
                        job.stock.offsets[i.min(5)] = *n;
                    }
                }
                _ => {}
            },
            Draft::Machine {
                machine,
                by_kind,
                from,
                ..
            } => {
                let s = &mut machine.post;
                match (key, v) {
                    ("by-kind", Value::Bool(b)) => *by_kind = *b,
                    ("machine", Value::Index(i)) => {
                        let cat = catalog();
                        let mut order: Vec<usize> = (0..cat.len()).collect();
                        if *by_kind {
                            order.sort_by_key(|i| index_of(&MachineKind::ALL, cat[*i].kind));
                        } else {
                            order.sort_by_key(|i| cat[*i].title());
                        }
                        if let Some(&k) = order.get(*i) {
                            **machine = cat[k].clone();
                            *from = Some(k);
                        }
                    }
                    (k, Value::Number(n)) if k.starts_with("travel-") => {
                        if let Ok(i) = k[7..].parse::<usize>() {
                            machine.travel[i.min(2)] = *n;
                        }
                    }
                    ("rapid", Value::Number(n)) => machine.rapid_rate = *n,
                    ("max-feed", Value::Number(n)) => machine.max_feed = *n,
                    ("max-spindle", Value::Number(n)) => machine.max_spindle = *n,
                    ("changer", Value::Bool(b)) => machine.tool_changer = *b,
                    ("unit", Value::Index(i)) => {
                        s.output_unit = if *i == 1 { LenUnit::Inch } else { LenUnit::Mm }
                    }
                    ("adv-wp", Value::Bool(b)) => s.advanced_work_planes = *b,
                    ("seq", Value::Bool(b)) => s.sequence_numbers = *b,
                    ("block-start", Value::Number(n)) => s.block_start = n.max(0.0) as u32,
                    ("block-inc", Value::Number(n)) => s.block_increment = n.max(1.0) as u32,
                    ("tools-only", Value::Bool(b)) => s.numbers_at_tools_only = *b,
                    ("rapid-pos", Value::Bool(b)) => s.rapid_positioning = *b,
                    ("post-rapid", Value::Number(n)) => s.rapid_rate = n.max(1.0),
                    ("arcs", Value::Bool(b)) => s.output_arcs = *b,
                    ("helices", Value::Bool(b)) => s.output_helices = *b,
                    ("decimals", Value::Number(n)) => s.decimals = n.clamp(0.0, 6.0) as u32,
                    ("dec-point", Value::Bool(b)) => s.decimal_point = *b,
                    ("ret-tool", Value::Bool(b)) => s.force_retract_on_tool_change = *b,
                    ("ret-plane", Value::Bool(b)) => s.force_retract_on_plane_change = *b,
                    ("ret-end", Value::Bool(b)) => s.force_retract_on_end = *b,
                    ("cy-drill", Value::Bool(b)) => s.canned_drill = *b,
                    ("cy-peck", Value::Bool(b)) => s.canned_peck = *b,
                    ("cy-tap", Value::Bool(b)) => s.canned_tap = *b,
                    ("cy-chip", Value::Bool(b)) => s.canned_chip_break = *b,
                    ("laser-mode", Value::Index(i)) => {
                        s.laser_mode = if *i == 1 {
                            LaserMode::Constant
                        } else {
                            LaserMode::Dynamic
                        }
                    }
                    ("comments", Value::Bool(b)) => s.comments = *b,
                    _ => {}
                }
            }
            Draft::Setup { setup, .. } => match (key, v) {
                ("name", Value::Text(t)) => setup.name = t.clone(),
                ("program", Value::Number(n)) => setup.program_number = n.clamp(1.0, 9999.0) as u32,
                ("mating", Value::Index(i)) => {
                    setup.mating = if *i == 1 {
                        Mating::Manual([0.0; 3])
                    } else {
                        Mating::Automatic
                    }
                }
                (k, Value::Number(n)) if k.starts_with("mate-") => {
                    if let (Mating::Manual(o), Ok(i)) = (&mut setup.mating, k[5..].parse::<usize>())
                    {
                        o[i.min(2)] = *n;
                    }
                }
                _ => {
                    set_plane(&mut setup.wcs, key, v);
                }
            },
            Draft::WorkPlane { op, .. } => {
                if let ("name", Value::Text(t)) = (key, v) {
                    op.name = t.clone();
                } else if let OpKind::WorkPlane(w) = &mut op.kind {
                    set_plane(w, key, v);
                }
            }
            Draft::Tool {
                op,
                library,
                filter,
                setup,
                ..
            } => {
                let machine = machine_of(ctx_cam, *setup);
                let name_was_default = match &op.kind {
                    OpKind::Tool(t) => op.name == t.tool.name || op.name == t.tool.default_name(),
                    _ => false,
                };
                let OpKind::Tool(t) = &mut op.kind else {
                    return;
                };
                let tool = &mut t.tool;
                match (key, v) {
                    ("library", Value::Bool(b)) => *library = *b,
                    ("lib-type", Value::Index(i)) => {
                        *filter = if *i == 0 {
                            None
                        } else {
                            MachiningType::ALL.get(i - 1).copied()
                        }
                    }
                    ("lib-tool", Value::Index(i)) => {
                        let lib: Vec<Tool> = ctx_cam
                            .library
                            .filtered(*filter, None)
                            .into_iter()
                            .filter(|(_, x)| x.kind.fits(machine))
                            .map(|(_, x)| x.clone())
                            .collect();
                        let cur = lib.iter().position(|x| x == tool);
                        let i = if cur.is_none() {
                            i.checked_sub(1)
                        } else {
                            Some(*i)
                        };
                        if let Some(x) = i.and_then(|i| lib.get(i)) {
                            *tool = x.clone();
                            op.name = tool.name.clone();
                        }
                    }
                    ("lib-import", Value::Activate) => {
                        let dir = crate::export_dir(world).unwrap_or_default();
                        let theme = world.resource::<Theme>().clone();
                        let mut commands = world.commands();
                        cadrs_ui::open_file_picker(
                            &mut commands,
                            &theme,
                            "cam-library-import",
                            "Import tool library",
                            "cam-library-import",
                            dir,
                            &["json"],
                        );
                        world.flush();
                    }
                    ("lib-export", Value::Activate) => export_library(world, ctx_cam),
                    ("name", Value::Text(s)) => op.name = s.clone(),
                    ("kind", Value::Index(i)) => {
                        let kinds = ToolKind::for_machine(machine);
                        if let Some(k) = kinds.get(*i) {
                            let d = if k.is_beam() == tool.is_beam() {
                                tool.diameter
                            } else if k.is_beam() {
                                default_kerf(machine)
                            } else {
                                6.0
                            };
                            *tool = Tool::new(*k, d);
                        }
                    }
                    ("unit", Value::Index(i)) => {
                        tool.unit = if *i == 1 { LenUnit::Inch } else { LenUnit::Mm }
                    }
                    ("hand", Value::Index(i)) => {
                        tool.hand = if *i == 1 { Hand::Left } else { Hand::Right }
                    }
                    ("diameter", Value::Number(n)) => tool.diameter = *n,
                    ("corner", Value::Number(n)) => tool.corner_radius = *n,
                    ("angle", Value::Number(n)) => tool.angle = *n,
                    ("tip", Value::Number(n)) => tool.tip_diameter = *n,
                    ("pitch", Value::Number(n)) => tool.pitch = *n,
                    ("cut-len", Value::Number(n)) => tool.cutting_length = *n,
                    ("overall", Value::Number(n)) => tool.overall_length = *n,
                    ("flutes", Value::Number(n)) => tool.flutes = n.clamp(1.0, 32.0) as u32,
                    ("center", Value::Bool(b)) => tool.center_cutting = *b,
                    ("power", Value::Number(n)) => tool.beam.power = n.clamp(0.0, 100.0),
                    ("pierce", Value::Number(n)) => tool.beam.pierce_delay = n.max(0.0),
                    ("pierce-h", Value::Number(n)) => tool.beam.pierce_height = *n,
                    ("cut-h", Value::Number(n)) => tool.beam.cut_height = *n,
                    ("thc", Value::Bool(b)) => tool.beam.thc = *b,
                    ("material", Value::Number(n)) => tool.beam.material = n.max(0.0) as u32,
                    ("air", Value::Bool(b)) => tool.beam.air_assist = *b,
                    ("beam-passes", Value::Number(n)) => {
                        tool.beam.passes = n.clamp(1.0, 50.0) as u32
                    }
                    ("rpm", Value::Number(n)) => tool.cutting.spindle_rpm = n.max(0.0),
                    ("feed", Value::Number(n)) => tool.cutting.feed = *n,
                    ("plunge", Value::Number(n)) => tool.cutting.plunge_feed = *n,
                    ("stepdown", Value::Number(n)) => tool.cutting.step_down = *n,
                    ("stepover", Value::Number(n)) => tool.cutting.step_over = *n,
                    ("holder", Value::Bool(b)) => {
                        tool.holder = b.then(|| {
                            ctx_cam
                                .library
                                .holders
                                .first()
                                .cloned()
                                .unwrap_or_else(|| Profile::cylinder("Holder", 32.0, 40.0))
                        })
                    }
                    ("holder-lib", Value::Index(i)) => {
                        tool.holder = Some(if *i == 0 {
                            Profile::cylinder("Holder", 32.0, 40.0)
                        } else {
                            ctx_cam
                                .library
                                .holders
                                .get(i - 1)
                                .cloned()
                                .unwrap_or_else(|| Profile::cylinder("Holder", 32.0, 40.0))
                        })
                    }
                    ("holder-d", Value::Number(n)) => {
                        if let Some(h) = &mut tool.holder {
                            *h = Profile::cylinder(h.name.clone(), n.max(0.1), h.length().max(1.0));
                        }
                    }
                    ("holder-l", Value::Number(n)) => {
                        if let Some(h) = &mut tool.holder {
                            *h =
                                Profile::cylinder(h.name.clone(), h.max_radius() * 2.0, n.max(0.1));
                        }
                    }
                    ("number", Value::Number(n)) => t.number = n.clamp(1.0, 999.0) as u32,
                    _ => {}
                }
                // A name nobody typed follows the tool.
                if let OpKind::Tool(t) = &mut op.kind
                    && key != "name"
                    && name_was_default
                {
                    t.tool.name = t.tool.default_name();
                    op.name = t.tool.name.clone();
                }
            }
            Draft::Toolpath {
                op, kind, setup, ..
            } => {
                let machine = machine_of(ctx_cam, *setup);
                let tool = tool_above(ctx_cam, *setup, op.id);
                let name_edited = matches!(&op.kind, OpKind::Toolpath(t) if t.name_edited);
                let OpKind::Toolpath(tp) = &mut op.kind else {
                    return;
                };
                let p = &mut tp.params;
                let tk = tool.as_ref().map_or(ToolKind::EndMill, |t| t.kind);
                let mut renamed = None;
                match (key, v) {
                    ("name", Value::Text(s)) => {
                        renamed = Some(s.clone());
                    }
                    ("type", Value::Index(i)) => {
                        let types: Vec<Type> = Type::ALL
                            .into_iter()
                            .filter(|t| !Pattern::available(*t, tk, machine).is_empty())
                            .collect();
                        if let Some(t) = types.get(*i) {
                            *kind = *t;
                            let pat = Pattern::available(*t, tk, machine)[0];
                            if let Some(tool) = &tool {
                                let drive = std::mem::take(&mut p.drive);
                                *p = Params::new(pat, tool, machine);
                                // Picks of the same kind stay.
                                p.drive.faces = if *t == Type::Faces || *t == Type::Bodies {
                                    drive.faces
                                } else {
                                    Vec::new()
                                };
                                p.drive.edges = if *t == Type::Edges {
                                    drive.edges
                                } else {
                                    Vec::new()
                                };
                                p.drive.holes = if *t == Type::Holes {
                                    drive.holes
                                } else {
                                    Vec::new()
                                };
                            }
                        }
                    }
                    ("pattern", Value::Index(i)) => {
                        let pats = Pattern::available(*kind, tk, machine);
                        if let (Some(pat), Some(tool)) = (pats.get(*i), &tool) {
                            let drive = std::mem::take(&mut p.drive);
                            *p = Params::new(*pat, tool, machine);
                            p.drive = drive;
                        }
                    }
                    ("sub", Value::Index(i)) => p.sub_pattern = SubPattern::ALL[(*i).min(1)],
                    ("drive", Value::Remove(i)) => match kind {
                        Type::Edges => {
                            if *i < p.drive.edges.len() {
                                p.drive.edges.remove(*i);
                            }
                        }
                        Type::Holes => {
                            if *i < p.drive.holes.len() {
                                p.drive.holes.remove(*i);
                            }
                        }
                        _ => {
                            if *i < p.drive.faces.len() {
                                p.drive.faces.remove(*i);
                            }
                        }
                    },
                    ("reverse", Value::Bool(b)) => p.drive.reverse = *b,
                    ("side", Value::Index(i)) => p.side = Side::ALL[(*i).min(3)],
                    ("wall", Value::Number(n)) => p.wall_stock = *n,
                    ("floor", Value::Number(n)) => p.floor_stock = *n,
                    ("stepdown", Value::Number(n)) => p.step_down = *n,
                    ("first", Value::Number(n)) => p.first_step_down = n.max(0.0),
                    ("stepover", Value::Number(n)) => p.step_over = *n,
                    ("passes", Value::Number(n)) => p.passes = n.clamp(1.0, 50.0) as u32,
                    ("direction", Value::Index(i)) => p.direction = Direction::ALL[(*i).min(1)],
                    ("method", Value::Index(i)) => {
                        p.cutting_method = CuttingMethod::ALL[(*i).min(1)]
                    }
                    ("angle", Value::Number(n)) => p.angle = *n,
                    ("machine-by", Value::Index(i)) => p.machine_by = MachineBy::ALL[(*i).min(1)],
                    ("top-from", Value::Index(i)) => {
                        p.heights.top.from = HeightRef::ALL[(*i).min(5)]
                    }
                    ("top-off", Value::Number(n)) => p.heights.top.offset = *n,
                    ("bottom-from", Value::Index(i)) => {
                        p.heights.bottom.from = HeightRef::ALL[(*i).min(5)]
                    }
                    ("bottom-off", Value::Number(n)) => p.heights.bottom.offset = *n,
                    ("depth", Value::Number(n)) => p.depth = n.max(0.0),
                    ("tip-off", Value::Number(n)) => p.tip_offset = n.max(0.0),
                    ("tabs", Value::Bool(b)) => p.tabs.enabled = *b,
                    ("tab-count", Value::Number(n)) => p.tabs.count = n.clamp(1.0, 50.0) as usize,
                    ("tab-w", Value::Number(n)) => p.tabs.width = *n,
                    ("tab-h", Value::Number(n)) => p.tabs.height = *n,
                    ("hole-select", Value::Index(i)) => {
                        p.cycle.select = HoleSelect::ALL[(*i).min(3)]
                    }
                    ("cycle", Value::Index(i)) => {
                        p.cycle.kind = [
                            CycleKind::Drill,
                            CycleKind::Peck,
                            CycleKind::ChipBreak,
                            CycleKind::Tap,
                        ][(*i).min(3)]
                    }
                    ("peck", Value::Number(n)) => p.cycle.peck = n.max(0.0),
                    ("dwell", Value::Number(n)) => p.cycle.dwell = n.max(0.0),
                    ("retract-full", Value::Bool(b)) => p.cycle.retract_full = *b,
                    ("break", Value::Number(n)) => p.cycle.break_through = n.max(0.0),
                    ("rpm", Value::Number(n)) => p.speeds.spindle_rpm = n.max(0.0),
                    ("feed", Value::Number(n)) => p.speeds.feed = *n,
                    ("plunge", Value::Number(n)) => p.speeds.plunge = *n,
                    ("ramp-feed", Value::Number(n)) => p.speeds.ramp = *n,
                    ("lead-feed", Value::Number(n)) => p.speeds.lead = *n,
                    ("coolant", Value::Index(i)) => p.posting.coolant = Coolant::ALL[(*i).min(3)],
                    ("force-tc", Value::Bool(b)) => p.posting.force_tool_change = *b,
                    ("stop", Value::Bool(b)) => p.posting.stop_before = *b,
                    ("comment", Value::Text(s)) => p.posting.comment = s.clone(),
                    ("rest", Value::Bool(b)) => p.rest = *b,
                    ("clearance", Value::Number(n)) => p.links.clearance = n.max(0.0),
                    ("rapid-d", Value::Number(n)) => p.links.rapid_distance = n.max(0.0),
                    ("feed-d", Value::Number(n)) => p.links.feed_distance = n.max(0.0),
                    ("gap", Value::Number(n)) => p.links.small_gap = n.max(0.0),
                    ("small", Value::Index(i)) => p.links.small_gaps = LinkAction::ALL[(*i).min(3)],
                    ("large", Value::Index(i)) => p.links.large_gaps = LinkAction::ALL[(*i).min(3)],
                    ("entry", Value::Index(i)) => p.links.entry = Entry::ALL[(*i).min(1)],
                    ("ramp-angle", Value::Number(n)) => p.links.ramp_angle = n.clamp(0.5, 89.0),
                    ("lead-in", Value::Index(i)) => p.links.lead_in = LeadKind::ALL[(*i).min(2)],
                    ("lead-out", Value::Index(i)) => p.links.lead_out = LeadKind::ALL[(*i).min(2)],
                    ("lead-r", Value::Number(n)) => p.links.lead_radius = n.max(0.0),
                    ("overlap", Value::Number(n)) => p.links.overlap = n.max(0.0),
                    ("tolerance", Value::Number(n)) => p.tolerance = n.clamp(0.0005, 1.0),
                    ("smooth", Value::Number(n)) => p.smooth_corners = n.max(0.0),
                    ("beam-power", Value::Number(n)) => p.beam.power = n.clamp(0.0, 100.0),
                    ("beam-passes", Value::Number(n)) => p.beam.passes = n.clamp(1.0, 50.0) as u32,
                    ("pass-depth", Value::Number(n)) => p.beam.pass_depth = n.max(0.0),
                    ("micro", Value::Number(n)) => p.beam.microjoints = n.clamp(0.0, 64.0) as usize,
                    ("micro-w", Value::Number(n)) => p.beam.microjoint_width = n.max(0.0),
                    ("small-hole", Value::Number(n)) => p.beam.small_hole = n.max(0.0),
                    ("small-feed", Value::Number(n)) => {
                        p.beam.small_hole_feed = (n / 100.0).clamp(0.05, 1.0)
                    }
                    _ => {}
                }
                let pattern = p.pattern;
                if let Some(n) = renamed {
                    op.name = n;
                    if let OpKind::Toolpath(tp) = &mut op.kind {
                        tp.name_edited = true;
                    }
                } else if !name_edited && matches!(key, "type" | "pattern") {
                    // A name nobody typed follows the pattern.
                    let mut c2 = ctx_cam.clone();
                    if let Some(NodeRef::Operation(j, s, o)) = c2.find(op.id) {
                        c2.jobs[j].setups[s].operations.remove(o);
                    }
                    op.name = c2.next_name(pattern.label());
                }
            }
        }
    }

    /// Takes a pick in the view.
    fn pick(
        &mut self,
        ctx: &Ctx,
        key: &str,
        pick: Pick,
        frame: Option<cadrs_core::cam::cadrs_cam::Frame>,
    ) {
        match self {
            Draft::Job { job, .. } => {
                let Some(part) = pick.part() else { return };
                let list = if key == "workholdings" {
                    &mut job.workholdings
                } else {
                    &mut job.bodies
                };
                match list.iter().position(|p| *p == part) {
                    Some(i) => {
                        list.remove(i);
                    }
                    None => list.push(part),
                }
            }
            Draft::Setup { setup, .. } => face_z(&mut setup.wcs, ctx, pick),
            Draft::WorkPlane { op, .. } => {
                if let OpKind::WorkPlane(w) = &mut op.kind {
                    face_z(w, ctx, pick);
                }
            }
            Draft::Toolpath { op, kind, .. } => {
                let OpKind::Toolpath(tp) = &mut op.kind else {
                    return;
                };
                let d = &mut tp.params.drive;
                let toggle = |v: &mut Vec<String>, k: String| match v.iter().position(|x| *x == k) {
                    Some(i) => {
                        v.remove(i);
                    }
                    None => v.push(k),
                };
                match (kind, pick) {
                    (Type::Edges, Pick::Edge(part, name)) => {
                        toggle(&mut d.edges, geometry::edge_key(part, &name))
                    }
                    (Type::Edges, Pick::Face(part, name)) => {
                        // A face's boundary edges.
                        if let Some(p) = ctx.parts.iter().find(|p| p.id == part) {
                            for e in p
                                .solid
                                .edges
                                .iter()
                                .filter(|e| e.name.faces.contains(&name))
                            {
                                toggle(&mut d.edges, geometry::edge_key(part, &e.name));
                            }
                        }
                    }
                    (Type::Faces | Type::Bodies, Pick::Face(part, name)) => {
                        toggle(&mut d.faces, geometry::face_key(part, &name))
                    }
                    (Type::Holes, Pick::Face(part, name)) => {
                        // A cylindrical face's axis, in the toolpath's coordinates.
                        let axis = ctx
                            .parts
                            .iter()
                            .find(|p| p.id == part)
                            .and_then(|p| p.solid.faces.iter().find(|f| f.name == name))
                            .and_then(|f| f.axis);
                        if let (Some((pt, _)), Some(fr)) = (axis, frame) {
                            let q = fr.to_local(pt);
                            let c = [q[0], q[1]];
                            match d
                                .holes
                                .iter()
                                .position(|h| (h[0] - c[0]).hypot(h[1] - c[1]) < 1e-3)
                            {
                                Some(i) => {
                                    d.holes.remove(i);
                                }
                                None => d.holes.push(c),
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// The edits ✓ makes.
    fn commit(&self, ctx: &Ctx, world: &mut World) -> Vec<Edit> {
        match self {
            Draft::Insert(d) => {
                let Some((src, links)) = source_of(world, d) else {
                    return Vec::new();
                };
                let components = d
                    .parts
                    .iter()
                    .filter(|p| p.2)
                    .map(|(id, name, _)| {
                        let mut f = src.clone();
                        f.selection = cadrs_core::derived::DerivedSelection {
                            all: false,
                            parts: vec![*id],
                            ..Default::default()
                        };
                        Component {
                            id: CamId::new(),
                            role: d.role,
                            name: name.clone(),
                            derived: f,
                            hidden: false,
                        }
                    })
                    .collect();
                vec![Edit::AddComponents { components, links }]
            }
            Draft::Component(c) => {
                let mut v = Vec::new();
                if let Some(old) = ctx.cam.component(c.id)
                    && old != &**c
                {
                    v.push(Edit::SetComponent {
                        component: c.clone(),
                        links: Vec::new(),
                    });
                }
                v
            }
            Draft::Job { job, is_new } => {
                if *is_new {
                    vec![Edit::AddJob(job.clone())]
                } else {
                    vec![Edit::SetJob(job.clone())]
                }
            }
            Draft::Machine { job, machine, .. } => vec![Edit::SetMachine {
                job: *job,
                machine: Some(machine.clone()),
            }],
            Draft::Setup { job, setup, is_new } => {
                if *is_new {
                    vec![Edit::AddSetup {
                        job: *job,
                        setup: setup.clone(),
                    }]
                } else {
                    vec![Edit::SetSetup {
                        job: *job,
                        setup: setup.clone(),
                    }]
                }
            }
            Draft::Tool {
                setup,
                after,
                op,
                is_new,
                ..
            }
            | Draft::Toolpath {
                setup,
                after,
                op,
                is_new,
                ..
            }
            | Draft::WorkPlane {
                setup,
                after,
                op,
                is_new,
            } => {
                if *is_new {
                    vec![Edit::AddOperation {
                        setup: *setup,
                        after: *after,
                        op: op.clone(),
                    }]
                } else {
                    vec![Edit::SetOperation(op.clone())]
                }
            }
        }
    }

    /// What the view's picks may be while `key` takes them.
    fn filter(&self, key: &str) -> crate::parts::PickFilter {
        let none = crate::parts::PickFilter::none();
        match (self, key) {
            (Draft::Job { .. }, _) => crate::parts::PickFilter {
                faces: true,
                ..none
            },
            (_, "p-face") => crate::parts::PickFilter {
                faces: true,
                planar_only: true,
                ..none
            },
            (
                Draft::Toolpath {
                    kind: Type::Edges, ..
                },
                _,
            ) => crate::parts::PickFilter {
                faces: true,
                edges: true,
                ..none
            },
            _ => crate::parts::PickFilter {
                faces: true,
                ..none
            },
        }
    }

    /// The draft as the studio would have it (for the live preview): the studio with the
    /// draft's operation in place.
    pub fn applied(&self, cam: &CamStudio) -> Option<(CamStudio, CamId)> {
        let Draft::Toolpath {
            setup,
            after,
            op,
            is_new,
            ..
        } = self
        else {
            return None;
        };
        let mut c = cam.clone();
        if *is_new {
            let Some(NodeRef::Setup(j, s)) = c.find(*setup) else {
                return None;
            };
            let ops = &mut c.jobs[j].setups[s].operations;
            let at = after
                .and_then(|a| ops.iter().position(|o| o.id == a))
                .map_or(ops.len(), |i| i + 1);
            ops.insert(at, (**op).clone());
        } else {
            let Some(NodeRef::Operation(j, s, o)) = c.find(op.id) else {
                return None;
            };
            c.jobs[j].setups[s].operations[o] = (**op).clone();
        }
        Some((c, op.id))
    }
}

/// Points a work plane's Z along a picked planar face's normal.
fn face_z(w: &mut WorkPlane, ctx: &Ctx, pick: Pick) {
    let Pick::Face(part, name) = pick else { return };
    let Some(n) = ctx
        .parts
        .iter()
        .find(|p| p.id == part)
        .and_then(|p| p.solid.faces.iter().find(|f| f.name == name))
        .and_then(|f| f.plane)
        .map(|pl| {
            let (u, v) = (pl.u, pl.v);
            [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ]
        })
    else {
        return;
    };
    let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if l < 1e-9 {
        return;
    }
    w.z = [n[0] / l, n[1] / l, n[2] / l];
    if (w.z[0] * w.x[0] + w.z[1] * w.x[1] + w.z[2] * w.x[2]).abs() > 0.9 {
        w.x = if w.z[0].abs() > 0.5 {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
    }
}

fn export_library(world: &mut World, cam: &CamStudio) {
    let Some(dir) = crate::export_dir(world) else {
        toast(world, "Nowhere to export to");
        return;
    };
    let path = cadrs_core::export::unique_path_ext(&dir, "cadrs-tool-library", "json");
    match std::fs::write(&path, cam.library.to_json()) {
        Ok(()) => {
            let theme = world.resource::<Theme>().clone();
            let mut commands = world.commands();
            cadrs_ui::show_notification(
                &mut commands,
                &theme,
                cadrs_ui::Notification::info(format!(
                    "Exported {} tools to {}",
                    cam.library.tools.len(),
                    path.display()
                ))
                .name("cam-toast"),
            );
            world.flush();
        }
        Err(e) => toast(world, &format!("Couldn't write the tool library: {e}")),
    }
}

fn import_library(world: &mut World, path: std::path::PathBuf) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            toast(world, &format!("Couldn't read {}: {e}", path.display()));
            return;
        }
    };
    match ToolLibrary::from_json(&text) {
        Ok(lib) => {
            let mut merged = cam.library.clone();
            let n = merged.merge(&lib);
            edit(world, Edit::SetLibrary(Box::new(merged)));
            let theme = world.resource::<Theme>().clone();
            let mut commands = world.commands();
            cadrs_ui::show_notification(
                &mut commands,
                &theme,
                cadrs_ui::Notification::info(format!("Imported {n} new tools")).name("cam-toast"),
            );
            world.flush();
            if let Some(mut d) = world.get_resource_mut::<CamDialog>() {
                d.shown = None;
            }
        }
        Err(e) => toast(world, &e),
    }
}

// ---------------------------------------------------------------------------------------------
// Systems

/// Builds the context drafts read (outside the borrow of the dialog).
fn with_ctx<R>(world: &mut World, f: impl FnOnce(&mut World, &Ctx) -> R) -> Option<R> {
    let (_, cam) = active_cam_owned(world)?;
    let doc = world.resource::<ActiveDocument>().doc.clone();
    let parts = ctx_parts(world);
    let vs = versions(world);
    let ctx = Ctx {
        doc: &doc,
        cam: &cam,
        parts: &parts,
        versions: &vs,
    };
    Some(f(world, &ctx))
}

fn take_changes(
    mut changes: MessageReader<FieldChange>,
    mut files: MessageReader<cadrs_ui::FilePicked>,
    mut commands: Commands,
) {
    let c: Vec<FieldChange> = changes.read().cloned().collect();
    let f: Vec<std::path::PathBuf> = files
        .read()
        .filter(|m| m.tag == "cam-library-import")
        .map(|m| m.path.clone())
        .collect();
    if c.is_empty() && f.is_empty() {
        return;
    }
    commands.queue(move |w: &mut World| apply_changes(w, c, f));
}

fn apply_changes(world: &mut World, changes: Vec<FieldChange>, files: Vec<std::path::PathBuf>) {
    for p in files {
        import_library(world, p);
    }
    if changes.is_empty() || !world.contains_resource::<CamDialog>() {
        return;
    }
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    for c in changes {
        let Some(mut d) = world.remove_resource::<CamDialog>() else {
            return;
        };
        // A click on a select or field (not a button or pick list) changes nothing.
        if c.value == Value::Activate && !is_pick_key(&c.key) && !is_button_key(&c.key) {
            world.insert_resource(d);
            continue;
        }
        match (c.key.as_str(), &c.value) {
            ("tab", Value::Index(i)) => d.tab = *i,
            (k, Value::Activate) if is_pick_key(k) => {
                d.picking = if d.picking.as_deref() == Some(k) {
                    None
                } else {
                    Some(k.to_string())
                };
            }
            ("p-z", Value::Index(i)) if *i >= AXES.len() => {
                d.picking = Some("p-face".into());
            }
            _ => {
                d.draft.set(world, &c.key, &c.value, &cam);
            }
        }
        d.changed = Some(std::time::Instant::now());
        // (Built again: a typed value that couldn't be read shows the old one.)
        d.shown = None;
        world.insert_resource(d);
    }
}

fn is_button_key(k: &str) -> bool {
    matches!(k, "lib-import" | "lib-export")
}

fn is_pick_key(k: &str) -> bool {
    matches!(k, "bodies" | "workholdings" | "drive" | "p-face")
}

fn take_picks(
    mut picks: MessageReader<PickRequest>,
    dialog: Option<Res<CamDialog>>,
    mut commands: Commands,
) {
    let v: Vec<Pick> = picks.read().filter_map(|p| p.0).collect();
    if v.is_empty() || dialog.is_none_or(|d| d.picking.is_none()) {
        return;
    }
    commands.queue(move |w: &mut World| apply_picks(w, v));
}

fn apply_picks(world: &mut World, picks: Vec<Pick>) {
    let Some(key) = world
        .get_resource::<CamDialog>()
        .and_then(|d| d.picking.clone())
    else {
        return;
    };
    for pick in picks {
        let frame = with_ctx(world, op_frame).flatten();
        with_ctx(world, |w, ctx| {
            if let Some(mut d) = w.remove_resource::<CamDialog>() {
                d.draft.pick(ctx, &key, pick, frame);
                if key == "p-face" {
                    d.picking = None;
                }
                d.changed = Some(std::time::Instant::now());
                d.shown = None;
                w.insert_resource(d);
            }
        });
    }
}

/// The frame a toolpath draft works in (hole picks are read in it).
fn op_frame(world: &mut World, ctx: &Ctx) -> Option<cadrs_core::cam::cadrs_cam::Frame> {
    let d = world.get_resource::<CamDialog>()?;
    let (cam, op) = d.draft.applied(ctx.cam)?;
    let (job, _, _) = cam.operation(op)?;
    let (_, wp) = cam.work_plane_for(op)?;
    Some(cadrs_core::cam::program::work_frame(
        &cam, job, wp, ctx.parts,
    ))
}

#[derive(Component)]
pub struct CamDialogRoot;

/// Builds the dialog, and builds it again when its fields change.
fn sync_dialog(world: &mut World) {
    if !world.contains_resource::<CamDialog>() {
        return;
    }
    let Some((fields, title, valid, filter)) = with_ctx(world, |w, ctx| {
        let d = w.resource::<CamDialog>();
        let fields = d.draft.fields(ctx, d.picking.as_deref(), d.tab);
        let valid = d.draft.problem(ctx).is_none();
        let filter = d.picking.as_ref().map(|k| d.draft.filter(k));
        (fields, d.draft.title(), valid, filter)
    }) else {
        return;
    };
    // Picks go to the dialog while a pick list takes them.
    let want = filter;
    if world.resource::<crate::viewport::PickFilterOverride>().0 != want {
        world
            .resource_mut::<crate::viewport::PickFilterOverride>()
            .0 = want;
    }
    let mut q = world.query_filtered::<Entity, With<CamDialogRoot>>();
    let existing = q.iter(world).next();
    let same = world.resource::<CamDialog>().shown.as_ref() == Some(&fields);
    if same && existing.is_some() {
        if let Some(e) = existing {
            let state = FeatureDialogState {
                title: title.clone(),
                valid,
                error: !valid,
            };
            if let Some(mut s) = world.get_mut::<FeatureDialogState>(e)
                && *s != state
            {
                *s = state;
            }
        }
        return;
    }
    if let Some(e) = existing {
        world.entity_mut(e).despawn();
    }
    let mut q_area = world.query_filtered::<Entity, With<ViewportArea>>();
    let Some(area) = q_area.iter(world).next() else {
        return;
    };
    let theme = world.resource::<Theme>().clone();
    let t2 = theme.clone();
    let fs = fields.clone();
    world.insert_resource(FieldUnits::of(&fields));
    let mut commands = world.commands();
    let d = commands
        .spawn((
            CamDialogRoot,
            DespawnOnExit(AppState::Document),
            FeatureDialog::new("cam-dialog")
                .title(title)
                .valid(valid)
                .width(330.0)
                .body(move |b| {
                    b.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        width: Val::Percent(100.0),
                        max_height: Val::Px(560.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    })
                    .with_children(|c| form::spawn_fields(c, &t2, &fs));
                })
                .build(&theme),
        ))
        .id();
    commands.entity(area).add_child(d);
    world.flush();
    world.resource_mut::<CamDialog>().shown = Some(fields);
}

fn on_accept(
    ev: On<FeatureDialogAccept>,
    q: Query<(), With<CamDialogRoot>>,
    mut commands: Commands,
) {
    if q.contains(ev.entity) {
        commands.queue(accept);
    }
}

fn on_cancel(
    ev: On<FeatureDialogCancel>,
    q: Query<(), With<CamDialogRoot>>,
    mut commands: Commands,
) {
    if q.contains(ev.entity) {
        commands.queue(close);
    }
}

/// ✓: the draft's edits (and a toolpath's calculation), then closes.
pub fn accept(world: &mut World) {
    // Text typed without Enter counts too.
    sync_typed_text(world);
    let Some(d) = world.remove_resource::<CamDialog>() else {
        return;
    };
    let problem = with_ctx(world, |_, ctx| d.draft.problem(ctx)).flatten();
    if let Some(p) = problem {
        toast(world, &p);
        world.insert_resource(d);
        return;
    }
    let edits = with_ctx(world, |w, ctx| d.draft.commit(ctx, w)).unwrap_or_default();
    let first_parts = matches!(d.draft, Draft::Insert(_))
        && active_cam_owned(world).is_some_and(|(_, c)| c.components.is_empty());
    let mut ok = true;
    for e in edits {
        ok &= edit(world, e);
    }
    if ok && first_parts {
        // The first parts in: frame them once they're built.
        world.resource_mut::<super::CamUi>().fit_pending = true;
    }
    if ok {
        match &d.draft {
            Draft::Toolpath { op, .. } => {
                world.resource_mut::<super::Computed>().request([op.id]);
                world.resource_mut::<CamUi>().selected = vec![op.id];
            }
            Draft::Tool { op, .. } | Draft::WorkPlane { op, .. } => {
                world.resource_mut::<CamUi>().selected = vec![op.id];
            }
            Draft::Setup { setup, .. } => world.resource_mut::<CamUi>().selected = vec![setup.id],
            Draft::Job { job, .. } => world.resource_mut::<CamUi>().selected = vec![job.id],
            _ => {}
        }
    }
    world.insert_resource(d);
    close(world);
}

/// Reads text fields typed into without Enter.
fn sync_typed_text(world: &mut World) {
    let mut q = world
        .query_filtered::<(&Name, &bevy::text::EditableText), With<cadrs_ui::TextInputField>>();
    let typed: Vec<(String, String)> = q
        .iter(world)
        .filter_map(|(n, t)| {
            let key = form::key_of(n.as_str())?.strip_suffix("-field")?;
            Some((key.to_string(), t.value().to_string()))
        })
        .collect();
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    for (k, v) in typed {
        if let Some(mut d) = world.remove_resource::<CamDialog>() {
            d.draft.set(world, &k, &Value::Text(v), &cam);
            world.insert_resource(d);
        }
    }
}

/// The toolpath being edited is calculated a moment after its last change.
fn live_preview(world: &mut World) {
    let Some(d) = world.get_resource::<CamDialog>() else {
        return;
    };
    let Some(t) = d.changed else { return };
    if t.elapsed().as_secs_f32() < 0.35 {
        return;
    }
    let draft = d.draft.clone();
    world.resource_mut::<CamDialog>().changed = None;
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let Some((applied, op)) = draft.applied(&cam) else {
        return;
    };
    super::compute::request_preview(world, applied, op);
}
