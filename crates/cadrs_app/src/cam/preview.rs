//! Preview operation: a toolpath (or a whole setup's) played back over its stock, from a bar
//! at the bottom of the view, as Onshape's Back plot, Verify and Simulate:
//!
//! - **Back plot**: the tool moves along the toolpath; what's done is drawn in its colours,
//!   what's left grey.
//! - **Verify**: the stock as the tool leaves it (a height map lowered under the cutter, or the
//!   kerf in a sheet), cut areas shaded by depth, gouges red.
//! - **Simulate**: Verify with the tool's holder drawn, and the collisions found (rapids into
//!   material, cuts deeper than the flutes) listed; it can pause on a collision.
//!
//! The bar: the three modes, the speed, Step back · Run/Pause · Step next · Go to end · Reset ·
//! Close, the progress (drag to scrub), and toggles for the tool, the toolpath, the part and
//! pausing on collisions. Its readout shows the time, the operation, the feed and the tool's
//! position; at the end, what Verify found.

use std::time::Instant;

use bevy::prelude::*;
use bevy::text::FontWeight;
use cadrs_core::cam::cadrs_cam::heightmap::{EMPTY, HeightMap};
use cadrs_core::cam::cadrs_cam::sim::{self, SimOp};
use cadrs_core::cam::cadrs_cam::strategy::Stock;
use cadrs_core::cam::cadrs_cam::tool::Tool;
use cadrs_core::cam::cadrs_cam::toolpath::{Segment, Timeline};
use cadrs_core::cam::cadrs_cam::{Frame, P3};
use cadrs_core::cam::{CamId, NodeRef, OpKind, program};
use cadrs_ui::prelude::*;
use cadrs_ui::{Button, Checkbox, CheckboxChange, Slider, SliderChange};

use super::view::{fill_motion, v3};
use super::{CamUi, Computed, active_cam_owned, toast};
use crate::AppState;
use crate::viewport::ViewportArea;

pub struct PreviewPlugin;

impl Plugin for PreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                poll_analysis,
                advance,
                sync_bar,
                sync_readout,
                draw,
                sync_stock_mesh,
            )
                .chain()
                .run_if(in_state(AppState::Document)),
        )
        .add_systems(OnExit(AppState::Document), |mut commands: Commands| {
            commands.remove_resource::<PreviewState>();
        })
        .add_observer(on_button)
        .add_observer(on_slider)
        .add_observer(on_check);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    BackPlot,
    Verify,
    Simulate,
}

/// An operation in the preview: where its moves start, and its name and tool.
#[derive(Clone)]
struct OpSpan {
    first: usize,
    name: String,
    tool: Tool,
}

/// The open preview.
#[derive(Resource)]
pub struct PreviewState {
    pub mode: Mode,
    /// "Rough 1", "Setup 1".
    pub label: String,
    /// The setup's frame: the moves are in its coordinates.
    frame: Frame,
    timeline: Timeline,
    ops: Vec<OpSpan>,
    beam: bool,
    pub t: f64,
    pub playing: bool,
    /// 0 (slow) to 1 (fast).
    pub speed: f32,
    pub show_tool: bool,
    pub show_path: bool,
    pub show_part: bool,
    pub pause_on_collision: bool,
    stock: Stock,
    /// The stock as cut so far (mills), and how many moves are in it.
    map: Option<HeightMap>,
    applied: usize,
    /// The part on the stock's grid, pulled in by a cell (gouges are below it).
    part: Option<HeightMap>,
    top: f32,
    mesh_dirty: bool,
    mesh_at: Option<Instant>,
    /// What the whole run finds: a summary line, and the collisions' times.
    pub summary: String,
    collisions: Vec<(f64, String)>,
    paused_at: Option<f64>,
    /// The check of the whole run, on a worker thread until it's done.
    analysis: Option<std::sync::Mutex<std::sync::mpsc::Receiver<Analysis>>>,
}

/// What simulating the whole run found (worked out off the main thread).
struct Analysis {
    summary: String,
    collisions: Vec<(f64, String)>,
    /// The part on the stock's grid, pulled in by a cell (mills).
    part: Option<HeightMap>,
}

impl PreviewState {
    /// Whether the run's check is still being worked out.
    pub fn checking(&self) -> bool {
        self.analysis.is_some()
    }

    /// Verify and Simulate show their own stock.
    pub fn hides_stock(&self) -> bool {
        self.mode != Mode::BackPlot
    }

    fn total(&self) -> f64 {
        self.timeline.total()
    }

    fn segment_index(&self) -> usize {
        self.timeline.at(self.t).map_or(0, |(_, i)| i)
    }

    fn op_at(&self, seg: usize) -> Option<&OpSpan> {
        self.ops.iter().rev().find(|o| o.first <= seg)
    }

    /// Minutes of program per second of playback.
    fn rate(&self) -> f64 {
        // From two minutes for the whole program (slowest) to about two seconds (fastest).
        let secs = 120.0 * 0.02f64.powf(self.speed as f64);
        (self.total() / secs).max(1e-6)
    }

    /// Jumps to a fraction of the program (0 to 1).
    pub fn seek(&mut self, f: f64) {
        let t = self.total() * f.clamp(0.0, 1.0);
        self.jump(t);
    }

    fn jump(&mut self, t: f64) {
        let t = t.clamp(0.0, self.total());
        if t < self.t - 1e-12 && self.map.is_some() {
            // Back: the stock again from the start.
            self.map = Some(self.stock.height_map(self.map.as_ref().unwrap().cell));
            self.applied = 0;
        }
        self.t = t;
        self.cut_to_now();
    }

    /// Lowers the stock under every move up to now.
    fn cut_to_now(&mut self) {
        let Some(map) = &mut self.map else { return };
        let upto = self.timeline.at(self.t).map_or(0, |(_, i)| i);
        let done = if self.t >= self.timeline.total() - 1e-12 {
            self.timeline.segments.len()
        } else {
            upto
        };
        while self.applied < done {
            let s = self.timeline.segments[self.applied];
            if !s.is_rapid() {
                let tool = &self
                    .ops
                    .iter()
                    .rev()
                    .find(|o| o.first <= self.applied)
                    .map(|o| o.tool.clone())
                    .unwrap_or_else(|| self.ops[0].tool.clone());
                map.cut(tool, s.from, s.to);
            }
            self.applied += 1;
            self.mesh_dirty = true;
        }
    }
}

/// Preview operation for the selection: a toolpath, or every toolpath of a setup (a job: its
/// first setup).
pub fn open_for_selection(world: &mut World) {
    let Some((_, cam)) = active_cam_owned(world) else {
        return;
    };
    let sel = world.resource::<CamUi>().selected.clone();
    let (job, setup, ops, label) = {
        let mut found = None;
        for id in sel.iter().rev() {
            match cam.find(*id) {
                Some(NodeRef::Operation(j, s, o)) => {
                    let op = &cam.jobs[j].setups[s].operations[o];
                    if matches!(op.kind, OpKind::Toolpath(_)) {
                        found = Some((j, s, Some(vec![op.id]), op.name.clone()));
                        break;
                    }
                }
                Some(NodeRef::Setup(j, s)) => {
                    found = Some((j, s, None, cam.jobs[j].setups[s].name.clone()));
                    break;
                }
                Some(NodeRef::Job(j)) if !cam.jobs[j].setups.is_empty() => {
                    found = Some((j, 0, None, cam.jobs[j].setups[0].name.clone()));
                    break;
                }
                _ => {}
            }
        }
        match found.or_else(|| {
            let j = cam
                .jobs
                .iter()
                .position(|j| Some(j.id) == cam.active().map(|a| a.id))?;
            (!cam.jobs[j].setups.is_empty())
                .then(|| (j, 0, None, cam.jobs[j].setups[0].name.clone()))
        }) {
            Some(f) => f,
            None => {
                toast(world, "Select a toolpath or setup to preview");
                return;
            }
        }
    };
    let job = &cam.jobs[job];
    let setup = &job.setups[setup];
    let Some(element) = super::active_cam_id(world) else {
        return;
    };
    let Some(parts) = super::compute::studio_parts(world, element) else {
        toast(world, "The parts are still being built");
        return;
    };
    // Every toolpath must be calculated.
    let wanted: Vec<CamId> = setup
        .operations
        .iter()
        .filter(|o| !o.suppressed && matches!(o.kind, OpKind::Toolpath(_)))
        .filter(|o| ops.as_ref().is_none_or(|v| v.contains(&o.id)))
        .map(|o| o.id)
        .collect();
    let missing: Vec<CamId> = wanted
        .iter()
        .copied()
        .filter(|o| world.resource::<Computed>().current(*o).is_none())
        .collect();
    if !missing.is_empty() {
        world.resource_mut::<Computed>().request(missing);
        toast(
            world,
            "Calculating the toolpaths first: preview again when they're done",
        );
        return;
    }
    if wanted.is_empty() {
        toast(world, "The setup has no toolpaths");
        return;
    }
    let paths = world.resource::<Computed>().paths();
    let run = match program::setup_run(&cam, job, setup, &parts, &paths, Some(&wanted)) {
        Ok(r) => r,
        Err(e) => {
            toast(world, &e);
            return;
        }
    };
    let Some(machine) = job.machine.clone() else {
        return;
    };
    let Some(shape) = program::stock_shape(&cam, job, &parts) else {
        return;
    };
    let stock = program::stock_in(&shape, &run.frame);
    let mut segments: Vec<Segment> = Vec::new();
    let mut spans = Vec::new();
    let mut at: P3 = [0.0, 0.0, stock.bounds.max[2] + 30.0];
    for o in &run.ops {
        spans.push(OpSpan {
            first: segments.len(),
            name: o.name.to_string(),
            tool: o.tool.clone(),
        });
        let m = o.toolpath.motion(at, machine.rapid_rate);
        if let Some(l) = m.last() {
            at = l.to;
        }
        segments.extend(m);
    }
    // The model the toolpaths were calculated against (the kernel's: the finest of them),
    // else the display meshes.
    let model = {
        let c = world.resource::<Computed>();
        wanted
            .iter()
            .filter_map(|o| c.current(*o))
            .filter(|d| d.frame == run.frame)
            .map(|d| d.model.clone())
            .max_by_key(|m| m.triangles.len())
    }
    .unwrap_or_else(|| {
        std::sync::Arc::new(cadrs_core::cam::geometry::model_from_parts(
            &program::job_bodies(&cam, job, &parts),
            &run.frame,
        ))
    });
    let beam = machine.kind.is_cutting();
    let cell = sim::sim_cell(&stock, 0.1);
    let (map, top) = if beam {
        (None, 0.0)
    } else {
        (Some(stock.height_map(cell)), stock.bounds.max[2] as f32)
    };
    // What the whole run finds, once, on a worker thread.
    let analysis = {
        let (tx, rx) = std::sync::mpsc::channel();
        let (stock, segments, spans) = (stock.clone(), segments.clone(), spans.clone());
        let spawned = std::thread::Builder::new()
            .name("cam-verify".into())
            .spawn(move || {
                let a = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    analyse(&stock, &model, &segments, &spans, beam, cell)
                }))
                .unwrap_or_else(|_| Analysis {
                    summary: "The check failed unexpectedly".into(),
                    collisions: Vec::new(),
                    part: None,
                });
                let _ = tx.send(a);
            });
        spawned.ok().map(|_| std::sync::Mutex::new(rx))
    };
    let timeline = Timeline::new(segments);
    super::dialogs::close(world);
    super::export::close(world);
    world.insert_resource(PreviewState {
        mode: Mode::BackPlot,
        label,
        frame: run.frame,
        timeline,
        ops: spans,
        beam,
        t: 0.0,
        playing: true,
        speed: 0.5,
        show_tool: true,
        show_path: true,
        show_part: true,
        pause_on_collision: true,
        stock,
        map,
        applied: 0,
        part: None,
        top,
        mesh_dirty: true,
        mesh_at: None,
        summary: "Checking the run…".into(),
        collisions: Vec::new(),
        paused_at: None,
        analysis,
    });
}

/// Simulates the whole run: the summary line, the collisions' times and (mills) the part on
/// the stock's grid.
fn analyse(
    stock: &Stock,
    model: &cadrs_core::cam::cadrs_cam::model::Model,
    segments: &[Segment],
    spans: &[OpSpan],
    beam: bool,
    cell: f64,
) -> Analysis {
    let ops: Vec<SimOp> = spans
        .iter()
        .enumerate()
        .map(|(k, sp)| {
            let end = spans.get(k + 1).map_or(segments.len(), |n| n.first);
            SimOp {
                tool: &sp.tool,
                segments: &segments[sp.first..end],
            }
        })
        .collect();
    if beam {
        let r = sim::simulate_sheet(stock, model, &ops);
        let summary = format!(
            "Cut {} mm · {} part{} free{}",
            fmt0(r.cut_length),
            r.parts_free,
            if r.parts_free == 1 { "" } else { "s" },
            if r.damage > 0.01 {
                format!(" · {} mm² cut from the part", fmt1(r.damage))
            } else {
                String::new()
            }
        );
        return Analysis {
            summary,
            collisions: Vec::new(),
            part: None,
        };
    }
    let r = sim::simulate_mill(stock, model, &ops, cell);
    let tl = Timeline::new(segments.to_vec());
    let mut collisions = Vec::new();
    for (k, p) in r.rapid_hits.iter() {
        collisions.push((
            time_near(&tl, spans, *k, *p),
            format!("{}: a rapid runs into the stock", spans[*k].name),
        ));
    }
    for (k, p) in r.holder_hits.iter() {
        collisions.push((
            time_near(&tl, spans, *k, *p),
            format!("{}: the cut is deeper than the flutes", spans[*k].name),
        ));
    }
    collisions.sort_by(|a, b| a.0.total_cmp(&b.0));
    let summary = format!(
        "Removed {} mm³ · {} · {}{}",
        fmt0(r.removed),
        if r.gouge_cells == 0 {
            "no gouges".to_string()
        } else {
            format!("gouges up to {} mm deep", fmt2(r.gouge_depth))
        },
        if collisions.is_empty() {
            "no collisions".to_string()
        } else {
            format!(
                "{} collision{}",
                collisions.len(),
                if collisions.len() == 1 { "" } else { "s" }
            )
        },
        if r.excess > 0.0 {
            format!(" · up to {} mm left", fmt2(r.excess))
        } else {
            String::new()
        }
    );
    let mut part = r.stock.blank();
    part.rasterize(&model.triangles, false);
    Analysis {
        summary,
        collisions,
        part: Some(part.eroded()),
    }
}

/// Takes the run's check when the worker has finished it.
fn poll_analysis(state: Option<ResMut<PreviewState>>) {
    let Some(mut s) = state else { return };
    let Some(rx) = s.analysis.as_mut() else {
        return;
    };
    let got = match rx.get_mut().map(|r| r.try_recv()) {
        Ok(Ok(a)) => a,
        Ok(Err(std::sync::mpsc::TryRecvError::Empty)) => return,
        _ => Analysis {
            summary: "The check stopped unexpectedly".into(),
            collisions: Vec::new(),
            part: None,
        },
    };
    s.analysis = None;
    s.summary = got.summary;
    s.collisions = got.collisions;
    s.part = got.part;
    s.mesh_dirty = true;
}

/// The time of the move of operation `k` nearest `p`.
fn time_near(tl: &Timeline, spans: &[OpSpan], k: usize, p: P3) -> f64 {
    let first = spans[k].first;
    let end = spans.get(k + 1).map_or(tl.segments.len(), |n| n.first);
    let mut best = (f64::INFINITY, first);
    for (i, s) in tl.segments.iter().enumerate().take(end).skip(first) {
        let d = cadrs_core::cam::cadrs_cam::geom::dist(s.to, p)
            .min(cadrs_core::cam::cadrs_cam::geom::dist(s.from, p));
        if d < best.0 {
            best = (d, i);
        }
    }
    tl.end_of(best.1.saturating_sub(1))
}

fn fmt0(v: f64) -> String {
    let s = format!("{:.0}", v);
    // Thin spaces between thousands.
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 && c != '-' {
            out.push('\u{2009}');
        }
        out.push(c);
    }
    out.chars().rev().collect()
}
fn fmt1(v: f64) -> String {
    format!("{v:.1}")
}
fn fmt2(v: f64) -> String {
    format!("{v:.2}")
}

/// Plays the preview on.
fn advance(time: Res<Time>, state: Option<ResMut<PreviewState>>) {
    let Some(mut s) = state else { return };
    if !s.playing {
        return;
    }
    let to = s.t + time.delta_secs_f64() * s.rate();
    // Pause on the next collision.
    if s.pause_on_collision
        && s.mode == Mode::Simulate
        && let Some((t, _)) = s
            .collisions
            .iter()
            .find(|(t, _)| *t > s.t + 1e-9 && *t <= to)
            .cloned()
        && s.paused_at != Some(t)
    {
        s.jump(t);
        s.playing = false;
        s.paused_at = Some(t);
        return;
    }
    s.jump(to);
    if s.t >= s.total() - 1e-12 {
        s.playing = false;
    }
}

// ---------------------------------------------------------------------------------------------
// The bar

#[derive(Component)]
struct PreviewBar;

#[derive(Component)]
struct Readout;

#[derive(Component, PartialEq, Clone)]
struct BarKey(Mode, bool, bool, bool, bool, bool, bool);

fn sync_bar(
    state: Option<Res<PreviewState>>,
    q: Query<(Entity, &BarKey), With<PreviewBar>>,
    q_area: Query<Entity, With<ViewportArea>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let Some(s) = state else {
        for (e, _) in &q {
            commands.entity(e).despawn();
        }
        return;
    };
    let key = BarKey(
        s.mode,
        s.playing,
        s.show_tool,
        s.show_path,
        s.show_part,
        s.pause_on_collision,
        s.beam,
    );
    if let Ok((e, k)) = q.single() {
        if *k == key {
            return;
        }
        commands.entity(e).despawn();
    }
    let Some(area) = q_area.iter().next() else {
        return;
    };
    let t = theme.clone();
    let speed = s.speed;
    let progress = if s.total() > 0.0 {
        (s.t / s.total()) as f32
    } else {
        0.0
    };
    let bar = commands
        .spawn((
            Name::new("cam-preview-bar"),
            PreviewBar,
            key.clone(),
            DespawnOnExit(AppState::Document),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                bottom: Val::Px(14.0),
                margin: UiRect::left(Val::Px(-330.0)),
                width: Val::Px(660.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(t.popover),
            BorderColor::all(t.border),
        ))
        .with_children(|b| {
            b.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|r| {
                for (name, label, m) in [
                    ("cam-pv-backplot", "Back plot", Mode::BackPlot),
                    ("cam-pv-verify", "Verify", Mode::Verify),
                    ("cam-pv-simulate", "Simulate", Mode::Simulate),
                ] {
                    let mut bt = Button::new(name).label(label).small();
                    bt = if s.mode == m {
                        bt.primary()
                    } else {
                        bt.outline()
                    };
                    r.spawn(bt.build(&t));
                }
                r.spawn(Node {
                    width: Val::Px(12.0),
                    ..default()
                });
                r.spawn(t.text("Speed", 11.0, FontWeight::NORMAL, t.muted_foreground));
                r.spawn(
                    Slider::new("cam-pv-speed")
                        .value(speed)
                        .width(110.0)
                        .tooltip("Playback speed")
                        .build(&t),
                );
                r.spawn(Node {
                    flex_grow: 1.0,
                    ..default()
                });
                for (name, icon, tip) in [
                    ("cam-pv-back", "chevron-left", "Step back"),
                    (
                        "cam-pv-run",
                        if s.playing { "pause" } else { "play" },
                        if s.playing { "Pause" } else { "Run" },
                    ),
                    ("cam-pv-next", "chevron-right", "Step next"),
                    ("cam-pv-end", "skip-forward", "Go to end"),
                    ("cam-pv-reset", "skip-back", "Reset"),
                    ("cam-pv-close", "close", "Close the preview"),
                ] {
                    r.spawn(IconButton::new(name, icon).tooltip(tip).build(&t));
                }
            });
            b.spawn(
                Slider::new("cam-pv-progress")
                    .value(progress)
                    .width(638.0)
                    .tooltip("Drag to move through the program")
                    .build(&t),
            );
            b.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|r| {
                r.spawn(
                    Checkbox::new("cam-pv-tool")
                        .label("Tool")
                        .checked(s.show_tool)
                        .build(&t),
                );
                r.spawn(
                    Checkbox::new("cam-pv-path")
                        .label("Toolpath")
                        .checked(s.show_path)
                        .build(&t),
                );
                r.spawn(
                    Checkbox::new("cam-pv-part")
                        .label("Part")
                        .checked(s.show_part)
                        .build(&t),
                );
                if s.mode == Mode::Simulate && !s.beam {
                    r.spawn(
                        Checkbox::new("cam-pv-pause")
                            .label("Pause on collision")
                            .checked(s.pause_on_collision)
                            .build(&t),
                    );
                }
            });
            b.spawn((
                Readout,
                Name::new("cam-pv-readout"),
                t.text("", 11.5, FontWeight::NORMAL, t.foreground),
            ));
        })
        .id();
    commands.entity(area).add_child(bar);
}

/// The readout and the progress slider follow the time.
fn sync_readout(
    state: Option<Res<PreviewState>>,
    mut q: Query<&mut Text, With<Readout>>,
    mut q_slider: Query<(&Name, &mut cadrs_ui::SliderState)>,
) {
    let Some(s) = state else { return };
    let i = s.segment_index();
    let seg = s.timeline.segments.get(i);
    let pos = s.timeline.at(s.t).map(|(p, _)| p).unwrap_or([0.0; 3]);
    let op = s.op_at(i).map(|o| o.name.as_str()).unwrap_or("");
    let feed = match seg.and_then(|g| g.feed) {
        Some(f) => format!("feed {} mm/min", fmt0(f)),
        None => "rapid".into(),
    };
    let mut text = format!(
        "{} · {} / {} · {} · {} · X{:.3} Y{:.3} Z{:.3}",
        s.label,
        super::compute::fmt_minutes(s.t),
        super::compute::fmt_minutes(s.total()),
        op,
        feed,
        pos[0],
        pos[1],
        pos[2]
    );
    if s.mode != Mode::BackPlot && s.t >= s.total() - 1e-12 {
        text = format!("{} · {}", s.label, s.summary);
    }
    if s.mode == Mode::Simulate
        && let Some(p) = s.paused_at
        && (p - s.t).abs() < 1e-9
        && let Some((_, why)) = s.collisions.iter().find(|(t, _)| (*t - p).abs() < 1e-9)
    {
        text = format!("Paused: {why}");
    }
    for mut t in &mut q {
        if t.0 != text {
            t.0 = text.clone();
        }
    }
    let progress = if s.total() > 0.0 {
        (s.t / s.total()) as f32
    } else {
        0.0
    };
    for (n, mut st) in &mut q_slider {
        if n.as_str() == "cam-pv-progress" && (st.value - progress).abs() > 1e-4 {
            st.value = progress;
        }
    }
}

fn on_button(ev: On<bevy::ui_widgets::Activate>, q: Query<&Name>, mut commands: Commands) {
    let Ok(n) = q.get(ev.entity) else { return };
    let name = n.as_str().to_string();
    if !name.starts_with("cam-pv-") {
        return;
    }
    commands.queue(move |w: &mut World| {
        if name == "cam-pv-close" {
            w.remove_resource::<PreviewState>();
            return;
        }
        let Some(mut s) = w.get_resource_mut::<PreviewState>() else {
            return;
        };
        match name.as_str() {
            "cam-pv-backplot" => s.mode = Mode::BackPlot,
            "cam-pv-verify" => {
                s.mode = Mode::Verify;
                s.mesh_dirty = true;
            }
            "cam-pv-simulate" => {
                s.mode = Mode::Simulate;
                s.mesh_dirty = true;
            }
            "cam-pv-run" => {
                if s.t >= s.total() - 1e-12 {
                    s.jump(0.0);
                }
                s.playing = !s.playing;
            }
            "cam-pv-back" => {
                s.playing = false;
                let i = s.segment_index();
                let t = if i == 0 {
                    0.0
                } else {
                    s.timeline.end_of(i - 1)
                };
                let t = if (t - s.t).abs() < 1e-12 && i >= 2 {
                    s.timeline.end_of(i - 2)
                } else {
                    t
                };
                s.jump(t);
            }
            "cam-pv-next" => {
                s.playing = false;
                let i = s.segment_index();
                let t = s.timeline.end_of(i);
                let t = if (t - s.t).abs() < 1e-12 {
                    s.timeline.end_of(i + 1)
                } else {
                    t
                };
                s.jump(t);
            }
            "cam-pv-end" => {
                s.playing = false;
                let t = s.total();
                s.jump(t);
            }
            "cam-pv-reset" => {
                s.playing = false;
                s.paused_at = None;
                s.jump(0.0);
            }
            _ => {}
        }
    });
}

fn on_slider(ev: On<SliderChange>, q: Query<&Name>, state: Option<ResMut<PreviewState>>) {
    let Some(mut s) = state else { return };
    let Ok(n) = q.get(ev.entity) else { return };
    match n.as_str() {
        "cam-pv-speed" => s.speed = ev.value.clamp(0.0, 1.0),
        "cam-pv-progress" => {
            let t = s.total() * ev.value.clamp(0.0, 1.0) as f64;
            s.playing = false;
            s.jump(t);
        }
        _ => {}
    }
}

fn on_check(ev: On<CheckboxChange>, q: Query<&Name>, state: Option<ResMut<PreviewState>>) {
    let Some(mut s) = state else { return };
    let Ok(n) = q.get(ev.entity) else { return };
    match n.as_str() {
        "cam-pv-tool" => s.show_tool = ev.checked,
        "cam-pv-path" => s.show_path = ev.checked,
        "cam-pv-part" => s.show_part = ev.checked,
        "cam-pv-pause" => s.pause_on_collision = ev.checked,
        _ => {}
    }
}

// ---------------------------------------------------------------------------------------------
// Drawing

/// The toolpath (done in colour, left grey) and the tool.
fn draw(
    state: Option<Res<PreviewState>>,
    mut g: Gizmos,
    mut assets: ResMut<Assets<bevy::gizmos::GizmoAsset>>,
    mut slot: Local<Option<(Entity, Handle<bevy::gizmos::GizmoAsset>)>>,
    mut last: Local<Option<(usize, bool, bool)>>,
    mut commands: Commands,
) {
    let Some(s) = state else {
        if let Some((e, _)) = slot.take() {
            commands.entity(e).try_despawn();
        }
        *last = None;
        return;
    };
    let i = s.segment_index();
    // The path: rebuilt when the segment changes.
    let key = (i, s.show_path, s.mode == Mode::BackPlot);
    if *last != Some(key) {
        *last = Some(key);
        let mut asset = bevy::gizmos::GizmoAsset::default();
        if s.show_path {
            let segs = &s.timeline.segments;
            let done = &segs[..(i + 1).min(segs.len())];
            fill_motion(&mut asset, done, &s.frame, s.beam, None);
            if s.mode == Mode::BackPlot {
                let rest = &segs[(i + 1).min(segs.len())..];
                fill_motion(
                    &mut asset,
                    rest,
                    &s.frame,
                    s.beam,
                    Some(Color::srgba(0.6, 0.62, 0.66, 0.45)),
                );
            }
        }
        match &*slot {
            Some((_, h)) => {
                let _ = assets.insert(h.id(), asset);
            }
            None => {
                let h = assets.add(asset);
                let e = commands
                    .spawn((
                        Name::new("cam-preview-path"),
                        bevy::gizmos::retained::Gizmo {
                            handle: h.clone(),
                            line_config: bevy::gizmos::config::GizmoLineConfig {
                                width: 1.8,
                                joints: bevy::gizmos::config::GizmoLineJoint::Round(4),
                                ..default()
                            },
                            depth_bias: -0.0005,
                        },
                        Transform::default(),
                        DespawnOnExit(AppState::Document),
                    ))
                    .id();
                *slot = Some((e, h));
            }
        }
    }
    // The tool where it is now.
    if !s.show_tool {
        return;
    }
    let Some((p, _)) = s.timeline.at(s.t) else {
        return;
    };
    let Some(op) = s.op_at(i) else { return };
    let tool = &op.tool;
    let f = &s.frame;
    let axis = v3(f.z);
    let tip = v3(f.to_world(p));
    let iso = Isometry3d::new(tip, Quat::from_rotation_arc(Vec3::Z, axis));
    let r = tool.radius() as f32;
    let color = Color::srgb_u8(0x52, 0x5a, 0x66);
    if tool.is_beam() {
        // A beam: a cone of light down to the kerf.
        g.circle(iso, r.max(0.3), Color::srgb_u8(0xff, 0x8a, 0x00));
        g.line(tip, tip + axis * 20.0, Color::srgb_u8(0xff, 0xb0, 0x40));
        return;
    }
    let flute = tool.cutting_length as f32;
    // The cutter's outline: its tip shape, the flutes and the shank to its overall length.
    let n = 8;
    let mut prev: Option<(f32, f32)> = None;
    for k in 0..=n {
        let rr = r * k as f32 / n as f32;
        let h = tool.height_at(rr as f64).unwrap_or(0.0) as f32;
        g.circle(
            Isometry3d::new(tip + axis * h, iso.rotation),
            rr.max(0.05),
            color,
        );
        if let Some((pr, ph)) = prev {
            for a in [0.0f32, 90.0, 180.0, 270.0] {
                let d = iso.rotation * (Quat::from_rotation_z(a.to_radians()) * Vec3::X);
                g.line(tip + axis * ph + d * pr, tip + axis * h + d * rr, color);
            }
        }
        prev = Some((rr, h));
    }
    let top = tip + axis * flute;
    g.circle(Isometry3d::new(top, iso.rotation), r, color);
    for a in [0.0f32, 90.0, 180.0, 270.0] {
        let d = iso.rotation * (Quat::from_rotation_z(a.to_radians()) * Vec3::X);
        g.line(
            tip + axis * prev.map_or(0.0, |x| x.1) + d * r,
            top + d * r,
            color,
        );
    }
    let shank = tip + axis * (tool.overall_length as f32).max(flute);
    g.line(top, shank, color);
    if s.mode == Mode::Simulate
        && let Some(h) = &tool.holder
    {
        // The holder from the end of the stick-out.
        let hc = Color::srgb_u8(0x9a, 0x6b, 0x2e);
        let mut last: Option<[f64; 2]> = None;
        for q in &h.points {
            let c = shank + axis * q[1] as f32;
            g.circle(Isometry3d::new(c, iso.rotation), q[0] as f32, hc);
            if let Some(lp) = last {
                for a in [0.0f32, 90.0, 180.0, 270.0] {
                    let d = iso.rotation * (Quat::from_rotation_z(a.to_radians()) * Vec3::X);
                    g.line(
                        shank + axis * lp[1] as f32 + d * lp[0] as f32,
                        c + d * q[0] as f32,
                        hc,
                    );
                }
            }
            last = Some(*q);
        }
    }
}

#[derive(Component)]
struct VerifyMesh;

/// Verify's stock: the height map as a surface (cut areas shaded by depth, gouges red), built
/// again at most ten times a second while it changes.
#[allow(clippy::too_many_arguments)]
fn sync_stock_mesh(
    state: Option<ResMut<PreviewState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q: Query<Entity, With<VerifyMesh>>,
    mut mat: Local<Option<Handle<StandardMaterial>>>,
    mut commands: Commands,
) {
    let Some(mut s) = state else {
        for e in &q {
            commands.entity(e).despawn();
        }
        return;
    };
    if s.mode == Mode::BackPlot || s.map.is_none() {
        for e in &q {
            commands.entity(e).despawn();
        }
        s.mesh_dirty = true;
        return;
    }
    if !s.mesh_dirty && !q.is_empty() {
        return;
    }
    if s.mesh_at.is_some_and(|t| t.elapsed().as_secs_f32() < 0.1) && !q.is_empty() {
        return;
    }
    s.mesh_dirty = false;
    s.mesh_at = Some(Instant::now());
    let mesh = height_mesh(s.map.as_ref().unwrap(), s.part.as_ref(), s.top, &s.frame);
    let m = mat
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                // The shading is in the vertex colours (the view has no lights for it).
                unlit: true,
                double_sided: true,
                cull_mode: None,
                ..default()
            })
        })
        .clone();
    let handle = meshes.add(mesh);
    match q.iter().next() {
        Some(e) => {
            commands.entity(e).insert(Mesh3d(handle));
        }
        None => {
            commands.spawn((
                Name::new("cam-verify-stock"),
                VerifyMesh,
                Mesh3d(handle),
                MeshMaterial3d(m),
                Transform::default(),
                DespawnOnExit(AppState::Document),
            ));
        }
    }
}

/// A height map as a surface with walls down to its base, coloured by what happened there.
fn height_mesh(map: &HeightMap, part: Option<&HeightMap>, top: f32, frame: &Frame) -> Mesh {
    let (nx, ny) = (map.nx, map.ny);
    // At most about 400 × 400 vertices: step over cells for big maps.
    let step = ((nx.max(ny) as f32 / 400.0).ceil() as usize).max(1);
    let xs: Vec<usize> = (0..nx).step_by(step).collect();
    let ys: Vec<usize> = (0..ny).step_by(step).collect();
    let base = map
        .z
        .iter()
        .copied()
        .filter(|v| *v != EMPTY)
        .fold(f32::INFINITY, f32::min)
        .min(top)
        - 0.0;
    let stock_c = [0.78f32, 0.80, 0.84, 1.0];
    let mut pos = Vec::with_capacity(xs.len() * ys.len());
    let mut col = Vec::with_capacity(xs.len() * ys.len());
    for &j in &ys {
        for &i in &xs {
            let v = map.at(i, j);
            let z = if v == EMPTY { base } else { v };
            let q = map.point(i, j);
            pos.push(v3(frame.to_world([q[0], q[1], z as f64])).to_array());
            let gouge = part.is_some_and(|p| {
                let pv = p.at(i, j);
                pv != EMPTY && v != EMPTY && v < pv - (map.cell as f32 * 0.5).max(0.01)
            });
            let c = if gouge {
                [0.86, 0.15, 0.12, 1.0]
            } else if v != EMPTY && v < top - 1e-3 {
                // Cut: darker blue-grey the deeper it goes.
                let d = ((top - v) / (top - base).max(1e-3)).clamp(0.0, 1.0);
                [0.62 - 0.25 * d, 0.70 - 0.22 * d, 0.80 - 0.12 * d, 1.0]
            } else {
                stock_c
            };
            col.push(c);
        }
    }
    let w = xs.len();
    let mut idx = Vec::new();
    for j in 0..ys.len().saturating_sub(1) {
        for i in 0..w.saturating_sub(1) {
            let a = (j * w + i) as u32;
            let (b, c, d) = (a + 1, a + w as u32, a + w as u32 + 1);
            let empty = |k: u32| {
                let (ii, jj) = (xs[k as usize % w], ys[k as usize / w]);
                map.at(ii, jj) == EMPTY
            };
            if empty(a) || empty(b) || empty(c) || empty(d) {
                continue;
            }
            idx.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }
    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_indices(bevy::mesh::Indices::U32(idx));
    mesh.compute_smooth_normals();
    // Lit from above right, as the parts are: the colour times a diffuse term.
    let light = Vec3::new(0.35, -0.45, 0.82).normalize();
    if let Some(bevy::mesh::VertexAttributeValues::Float32x3(normals)) =
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
    {
        for (c, n) in col.iter_mut().zip(normals) {
            let k = 0.62 + 0.38 * Vec3::from_array(*n).dot(light).max(0.0);
            for v in c.iter_mut().take(3) {
                *v *= k;
            }
        }
    }
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
    mesh
}
