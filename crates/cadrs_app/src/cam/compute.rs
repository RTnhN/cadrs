//! Computing toolpaths, off the main thread: an operation's parts are worked out by the kernel
//! in its work plane's coordinates (`cadrs_core::cam::geometry`, with exact sections where its
//! pattern needs them), then its strategy runs on a worker thread. Results are kept by the
//! key of their input ([`cadrs_core::cam::program::op_key`]): the **job cache**, so going
//! back to an earlier state (an undo, a value typed back) shows the toolpath at once, and a
//! toolpath accepted after its live preview is already calculated.
//!
//! An operation's [`Status`] is worked out against its current key: calculated, out of date
//! (it was calculated for other inputs), calculating, failed, or not calculated yet.
//! Regenerate operation queues it; Regenerate all toolpaths queues a setup's out-of-date ones.
//! Rest machining needs the toolpaths above it calculated first (they're queued before it).
//!
//! The toolpath dialog's **live preview** is one more slot: the studio with the draft in place,
//! calculated the same way while the dialog is open.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, mpsc};

use bevy::ecs::change_detection::Tick;
use bevy::prelude::*;
use cadrs_core::cam::cadrs_cam::Frame;
use cadrs_core::cam::cadrs_cam::heightmap::HeightMap;
use cadrs_core::cam::cadrs_cam::model::Model;
use cadrs_core::cam::cadrs_cam::toolpath::{Stats, Toolpath};
use cadrs_core::cam::program::{self, OpInput};
use cadrs_core::cam::{CamId, CamStudio, OpKind, geometry};
use cadrs_core::{ElementId, Part};

/// A calculated toolpath: in its work plane's coordinates, with that plane's frame.
#[derive(Debug, Clone)]
pub struct Done {
    pub toolpath: Arc<Toolpath>,
    pub frame: Frame,
    pub stats: Stats,
}

pub type Outcome = Result<Done, String>;

/// Where an operation stands.
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    NotCalculated,
    OutOfDate,
    Calculating,
    Done(Stats),
    Failed(String),
    /// Its input can't be made (no tool above it, no machine): why.
    Blocked(String),
}

impl Status {
    /// The trailing text of its row.
    pub fn short(&self) -> String {
        match self {
            Status::NotCalculated => "not calculated".into(),
            Status::OutOfDate => "out of date".into(),
            Status::Calculating => "calculating…".into(),
            Status::Done(s) => fmt_minutes(s.minutes),
            Status::Failed(_) => "failed".into(),
            Status::Blocked(_) => "—".into(),
        }
    }

    pub fn problem(&self) -> Option<&str> {
        match self {
            Status::Failed(e) | Status::Blocked(e) => Some(e),
            _ => None,
        }
    }
}

/// "2:05" (minutes and seconds), "1 h 03 min".
pub fn fmt_minutes(m: f64) -> String {
    let s = (m * 60.0).round() as i64;
    if s >= 3600 {
        format!("{} h {:02} min", s / 3600, (s % 3600) / 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

enum Stage {
    /// Waiting for the kernel's model (`sectioned`: the second request, with sections).
    Model {
        job: cadrs_core::rebuild::PendingJob<Result<Model, String>>,
        sectioned: bool,
    },
    /// The strategy on a worker thread.
    Generate(std::sync::Mutex<mpsc::Receiver<Result<Toolpath, String>>>),
}

struct Running {
    op: CamId,
    key: u64,
    /// The studio it was started from (a preview's has the draft in it).
    cam: Arc<CamStudio>,
    input: Arc<OpInput>,
    stage: Stage,
    /// The in-process stock for rest machining.
    rest: Option<Arc<HeightMap>>,
}

/// The job cache and what's being computed.
#[derive(Resource, Default)]
pub struct Computed {
    /// Each operation's latest result and the key it was computed for.
    pub latest: HashMap<CamId, (u64, Outcome)>,
    /// Results by key (the job cache).
    by_key: HashMap<u64, Outcome>,
    /// Each operation's current key (or why it has none), worked out when the document or the
    /// parts change.
    keys: HashMap<CamId, Result<u64, String>>,
    /// What the keys were worked out for: the tab, the document's change tick and the parts'
    /// generation.
    keys_for: Option<(ElementId, Tick, u64)>,
    queue: VecDeque<CamId>,
    running: Option<Running>,
    /// The live preview: wanted (studio with the draft, operation), running, and its result.
    preview_req: Option<(Arc<CamStudio>, CamId)>,
    preview_running: Option<Running>,
    preview: Option<(CamId, u64, Outcome)>,
    /// Models by request key (operations in one frame share one).
    models: HashMap<u64, Arc<Model>>,
    /// Bumped when a result arrives (the view and tree rebuild).
    pub generation: u64,
}

impl Computed {
    /// Where `op` stands.
    pub fn status(&self, op: CamId) -> Status {
        match self.keys.get(&op) {
            Some(Err(e)) => Status::Blocked(e.clone()),
            Some(Ok(k)) => {
                if self.running.as_ref().is_some_and(|r| r.op == op) || self.queue.contains(&op) {
                    return Status::Calculating;
                }
                match self.latest.get(&op) {
                    Some((lk, r)) if lk == k => match r {
                        Ok(d) => Status::Done(d.stats),
                        Err(e) => Status::Failed(e.clone()),
                    },
                    Some(_) => Status::OutOfDate,
                    None => Status::NotCalculated,
                }
            }
            None => Status::NotCalculated,
        }
    }

    /// The calculated toolpath of `op`, if it is up to date.
    pub fn current(&self, op: CamId) -> Option<&Done> {
        let Some(Ok(k)) = self.keys.get(&op) else {
            return None;
        };
        match self.latest.get(&op) {
            Some((lk, Ok(d))) if lk == k => Some(d),
            _ => None,
        }
    }

    /// The latest toolpath of `op`, up to date or not (the view shows an old one greyed).
    pub fn any(&self, op: CamId) -> Option<&Done> {
        match self.latest.get(&op) {
            Some((_, Ok(d))) => Some(d),
            _ => None,
        }
    }

    /// The live preview's result (while the toolpath dialog is open).
    pub fn preview(&self) -> Option<(CamId, &Outcome)> {
        self.preview.as_ref().map(|(op, _, r)| (*op, r))
    }

    /// Whether the live preview is being calculated.
    pub fn preview_busy(&self) -> bool {
        self.preview_req.is_some() || self.preview_running.is_some()
    }

    /// Forgets the live preview (the dialog closed).
    pub fn clear_preview(&mut self) {
        self.preview_req = None;
        self.preview_running = None;
        self.preview = None;
    }

    /// Queues operations (once each).
    pub fn request(&mut self, ops: impl IntoIterator<Item = CamId>) {
        for op in ops {
            if !self.queue.contains(&op) && self.running.as_ref().is_none_or(|r| r.op != op) {
                self.queue.push_back(op);
            }
        }
    }

    /// Whether anything is queued or running (scenarios wait for it).
    pub fn busy(&self) -> bool {
        self.running.is_some() || !self.queue.is_empty() || self.preview_busy()
    }

    /// The up-to-date toolpaths of every operation (for Verify and posting), in their own
    /// planes' coordinates.
    pub fn paths(&self) -> HashMap<CamId, Toolpath> {
        self.latest
            .iter()
            .filter(|(op, (k, r))| r.is_ok() && matches!(self.keys.get(op), Some(Ok(c)) if c == k))
            .filter_map(|(op, (_, r))| r.as_ref().ok().map(|d| (*op, (*d.toolpath).clone())))
            .collect()
    }

    /// Forgets everything (another document).
    pub fn clear(&mut self) {
        *self = Computed::default();
    }
}

/// Asks for the live preview of `op` in `cam` (the studio with the dialog's draft in place).
pub fn request_preview(world: &mut World, cam: CamStudio, op: CamId) {
    world.resource_mut::<Computed>().preview_req = Some((Arc::new(cam), op));
}

fn hash64(s: &str) -> u64 {
    let b = blake3::hash(s.as_bytes());
    u64::from_le_bytes(b.as_bytes()[..8].try_into().unwrap())
}

/// The studio's built parts when they're the active tab's and settled.
pub fn studio_parts(world: &World, element: ElementId) -> Option<Vec<Part>> {
    let cache = world.get_resource::<crate::parts::PartCache>()?;
    let (id, _, _) = cache.settled()?;
    (id == element).then(|| cache.parts.clone())
}

/// An operation's key in `cam` (or why it has none).
fn key_of(cam: &CamStudio, op: CamId, parts: &[Part], features: u64) -> Result<u64, String> {
    let input = program::op_input(cam, op, parts)?;
    let mut k = program::op_key(&input, features);
    // Rest machining depends on the toolpaths above it.
    if input.params.rest {
        let above = rest_inputs(cam, op, parts, features);
        k ^= hash64(&format!("rest{above:?}"));
    }
    Ok(k)
}

/// Works out every operation's key (the studio or its parts changed); results whose key came
/// back are taken from the cache.
fn refresh_keys(world: &mut World, cam: &CamStudio, parts: &[Part]) {
    let features = program::features_key(cam.part_features());
    let mut keys = HashMap::new();
    for job in &cam.jobs {
        for setup in &job.setups {
            for o in &setup.operations {
                if matches!(o.kind, OpKind::Toolpath(_)) {
                    keys.insert(o.id, key_of(cam, o.id, parts, features));
                }
            }
        }
    }
    let mut c = world.resource_mut::<Computed>();
    let mut adopted = Vec::new();
    for (op, k) in &keys {
        if let Ok(k) = k
            && c.latest.get(op).is_none_or(|(lk, _)| lk != k)
            && let Some(r) = c.by_key.get(k)
        {
            adopted.push((*op, *k, r.clone()));
        }
    }
    for (op, k, r) in adopted {
        c.latest.insert(op, (k, r));
    }
    c.keys = keys;
    c.generation += 1;
}

/// The keys of the toolpaths above `op` in its setup (what its rest stock comes from).
fn rest_inputs(cam: &CamStudio, op: CamId, parts: &[Part], features: u64) -> Vec<u64> {
    let Some((_, setup, _)) = cam.operation(op) else {
        return Vec::new();
    };
    setup
        .operations
        .iter()
        .take_while(|o| o.id != op)
        .filter(|o| !o.suppressed && matches!(o.kind, OpKind::Toolpath(_)))
        .filter_map(|o| program::op_input(cam, o.id, parts).ok())
        .map(|i| program::op_key(&i, features))
        .collect()
}

/// Keeps the keys current, starts what's queued and moves the running work along. Runs every
/// frame; does nothing while nothing changed and nothing is queued.
pub fn drive(world: &mut World) {
    let Some(element) = super::active_cam_id(world) else {
        return;
    };
    let parts_gen = world.resource::<crate::parts::PartCache>().generation;
    let Some(doc_tick) = world
        .get_resource_ref::<crate::ActiveDocument>()
        .map(|d| d.last_changed())
    else {
        return;
    };
    let stamp = Some((element, doc_tick, parts_gen));
    let stale = world.resource::<Computed>().keys_for != stamp;
    if !stale && !world.resource::<Computed>().busy() {
        return;
    }
    let Some(parts) = studio_parts(world, element) else {
        return;
    };
    if stale {
        let Some((_, cam)) = super::active_cam_owned(world) else {
            return;
        };
        refresh_keys(world, &cam, &parts);
        world.resource_mut::<Computed>().keys_for = stamp;
    }
    drive_main(world, &parts);
    drive_preview(world, &parts);
}

fn drive_main(world: &mut World, parts: &[Part]) {
    if let Some(mut r) = world.resource_mut::<Computed>().running.take() {
        match step(world, &mut r, parts) {
            Some(out) => {
                let mut c = world.resource_mut::<Computed>();
                c.by_key.insert(r.key, out.clone());
                c.latest.insert(r.op, (r.key, out));
                c.generation += 1;
            }
            None => world.resource_mut::<Computed>().running = Some(r),
        }
        return;
    }
    let Some(op) = world.resource_mut::<Computed>().queue.pop_front() else {
        return;
    };
    let Some((_, cam)) = super::active_cam_owned(world) else {
        return;
    };
    let key = match world.resource::<Computed>().keys.get(&op).cloned() {
        Some(Ok(k)) => k,
        Some(Err(e)) => {
            let mut c = world.resource_mut::<Computed>();
            c.latest.insert(op, (0, Err(e)));
            c.generation += 1;
            return;
        }
        None => return,
    };
    match start(world, Arc::new(cam), op, key, parts) {
        Started::Running(r) => world.resource_mut::<Computed>().running = Some(*r),
        Started::Waiting => {}
        Started::Known(out) => {
            let mut c = world.resource_mut::<Computed>();
            c.latest.insert(op, (key, out));
            c.generation += 1;
        }
    }
}

fn drive_preview(world: &mut World, parts: &[Part]) {
    // A newer draft supersedes the running preview.
    if world.resource::<Computed>().preview_req.is_some() {
        world.resource_mut::<Computed>().preview_running = None;
    }
    if let Some(mut r) = world.resource_mut::<Computed>().preview_running.take() {
        match step(world, &mut r, parts) {
            Some(out) => {
                let mut c = world.resource_mut::<Computed>();
                c.by_key.insert(r.key, out.clone());
                c.preview = Some((r.op, r.key, out));
                c.generation += 1;
            }
            None => world.resource_mut::<Computed>().preview_running = Some(r),
        }
        return;
    }
    let Some((cam, op)) = world.resource_mut::<Computed>().preview_req.take() else {
        return;
    };
    let features = program::features_key(cam.part_features());
    let key = match key_of(&cam, op, parts, features) {
        Ok(k) => k,
        Err(e) => {
            let mut c = world.resource_mut::<Computed>();
            c.preview = Some((op, 0, Err(e)));
            c.generation += 1;
            return;
        }
    };
    match start(world, cam.clone(), op, key, parts) {
        Started::Running(r) => world.resource_mut::<Computed>().preview_running = Some(*r),
        // Waiting for the toolpaths above it (rest machining): asked again until they're done.
        Started::Waiting => world.resource_mut::<Computed>().preview_req = Some((cam, op)),
        Started::Known(out) => {
            let mut c = world.resource_mut::<Computed>();
            c.preview = Some((op, key, out));
            c.generation += 1;
        }
    }
}

/// What [`start`] did.
enum Started {
    Running(Box<Running>),
    /// It waits for the toolpaths above it (rest machining: they're queued first).
    Waiting,
    /// Its outcome is known at once (from the cache, or why its input can't be made).
    Known(Outcome),
}

/// Starts calculating `op` of `cam` (see [`Started`]).
fn start(world: &mut World, cam: Arc<CamStudio>, op: CamId, key: u64, parts: &[Part]) -> Started {
    if let Some(r) = world.resource::<Computed>().by_key.get(&key).cloned() {
        return Started::Known(r);
    }
    let input = match program::op_input(&cam, op, parts) {
        Ok(i) => Arc::new(i),
        Err(e) => return Started::Known(Err(e)),
    };
    let mut rest = None;
    if input.params.rest {
        let above: Vec<CamId> = cam
            .operation(op)
            .map(|(_, s, _)| {
                s.operations
                    .iter()
                    .take_while(|o| o.id != op)
                    .filter(|o| !o.suppressed && matches!(o.kind, OpKind::Toolpath(_)))
                    .map(|o| o.id)
                    .collect()
            })
            .unwrap_or_default();
        let missing: Vec<CamId> = above
            .iter()
            .copied()
            .filter(|o| world.resource::<Computed>().current(*o).is_none())
            .collect();
        if !missing.is_empty() {
            let mut c = world.resource_mut::<Computed>();
            for m in missing.into_iter().rev() {
                if !c.queue.contains(&m) {
                    c.queue.push_front(m);
                }
            }
            return Started::Waiting;
        }
        rest = rest_stock(world, &cam, op, parts, &input).map(Arc::new);
    }
    let mk = model_key(&cam, &input, &[]);
    let stage = match world.resource::<Computed>().models.get(&mk).cloned() {
        Some(m) => start_generate(world, &cam, &input, m, rest.clone()),
        None => Stage::Model {
            job: geometry::model(program::model_request(&cam, &input, Vec::new())),
            sectioned: false,
        },
    };
    Started::Running(Box::new(Running {
        op,
        key,
        cam,
        input,
        stage,
        rest,
    }))
}

fn model_key(cam: &CamStudio, input: &OpInput, levels: &[f64]) -> u64 {
    hash64(&format!(
        "{}{:?}{:?}{}{:?}",
        program::features_key(cam.part_features()),
        input.parts,
        input.frame,
        input.tolerance,
        levels
    ))
}

/// Starts the strategy (asking for exact sections first when the pattern wants them).
fn start_generate(
    world: &mut World,
    cam: &CamStudio,
    input: &Arc<OpInput>,
    model: Arc<Model>,
    rest: Option<Arc<HeightMap>>,
) -> Stage {
    let levels = program::section_levels(input, &model);
    if levels.iter().any(|z| model.sections.get(*z).is_none()) {
        let key = model_key(cam, input, &levels);
        if let Some(m) = world.resource::<Computed>().models.get(&key).cloned() {
            return spawn_generate(input.clone(), m, rest);
        }
        return Stage::Model {
            job: geometry::model(program::model_request(cam, input, levels)),
            sectioned: true,
        };
    }
    spawn_generate(input.clone(), model, rest)
}

fn spawn_generate(input: Arc<OpInput>, model: Arc<Model>, rest: Option<Arc<HeightMap>>) -> Stage {
    let (tx, rx) = mpsc::channel();
    let spawned = std::thread::Builder::new()
        .name("cam-toolpath".into())
        .spawn(move || {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                program::generate(&input, &model, rest.as_deref())
            }))
            .unwrap_or_else(|_| Err("The strategy failed unexpectedly".into()));
            let _ = tx.send(r);
        });
    if spawned.is_err() {
        warn!("cam: couldn't start a toolpath thread");
    }
    Stage::Generate(std::sync::Mutex::new(rx))
}

/// Moves a running computation along; its outcome when done.
fn step(world: &mut World, r: &mut Running, parts: &[Part]) -> Option<Outcome> {
    match &mut r.stage {
        Stage::Model { job, sectioned } => {
            let got = job.poll()?;
            let sectioned = *sectioned;
            let model = match got {
                Some(Ok(m)) => Arc::new(m),
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    // The kernel couldn't: the display meshes instead.
                    let bodies: Vec<&Part> = parts
                        .iter()
                        .filter(|p| r.input.parts.contains(&p.id))
                        .collect();
                    Arc::new(geometry::model_from_parts(&bodies, &r.input.frame))
                }
            };
            let levels: Vec<f64> = if sectioned {
                program::section_levels(&r.input, &model)
            } else {
                Vec::new()
            };
            let key = model_key(&r.cam, &r.input, &levels);
            world
                .resource_mut::<Computed>()
                .models
                .insert(key, model.clone());
            r.stage = if sectioned {
                spawn_generate(r.input.clone(), model, r.rest.clone())
            } else {
                let (cam, input, rest) = (r.cam.clone(), r.input.clone(), r.rest.clone());
                start_generate(world, &cam, &input, model, rest)
            };
            None
        }
        Stage::Generate(rx) => match rx
            .get_mut()
            .map_err(|_| mpsc::TryRecvError::Disconnected)
            .and_then(|r| r.try_recv())
        {
            Ok(res) => Some(res.map(|tp| {
                let rapid = r.input.machine.rapid_rate;
                let start = [0.0, 0.0, r.input.stock.bounds.max[2] + 50.0];
                Done {
                    stats: tp.stats(start, rapid),
                    toolpath: Arc::new(tp),
                    frame: r.input.frame,
                }
            })),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err("The strategy stopped unexpectedly".into()))
            }
        },
    }
}

/// The in-process stock the toolpaths above `op` leave, in `op`'s coordinates (when they
/// share them).
fn rest_stock(
    world: &World,
    cam: &CamStudio,
    op: CamId,
    parts: &[Part],
    input: &OpInput,
) -> Option<HeightMap> {
    use cadrs_core::cam::cadrs_cam::sim::{SimOp, sim_cell, simulate_mill};
    let (job, setup, _) = cam.operation(op)?;
    let c = world.resource::<Computed>();
    let mut motions = Vec::new();
    for o in setup.operations.iter().take_while(|o| o.id != op) {
        if o.suppressed || !matches!(o.kind, OpKind::Toolpath(_)) {
            continue;
        }
        let d = c.current(o.id)?;
        if d.frame != input.frame {
            // Another work plane: its stock isn't on this one's grid.
            return None;
        }
        let tool = cam.tool_for(o.id)?.tool.clone();
        let start = [0.0, 0.0, input.stock.bounds.max[2] + 50.0];
        motions.push((tool, d.toolpath.motion(start, input.machine.rapid_rate)));
    }
    let bodies = program::job_bodies(cam, job, parts);
    let model = geometry::model_from_parts(&bodies, &input.frame);
    let ops: Vec<SimOp> = motions
        .iter()
        .map(|(t, m)| SimOp {
            tool: t,
            segments: m,
        })
        .collect();
    Some(simulate_mill(&input.stock, &model, &ops, sim_cell(&input.stock, 0.2)).stock)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minutes_read_well() {
        assert_eq!(fmt_minutes(2.0 + 5.0 / 60.0), "2:05");
        assert_eq!(fmt_minutes(63.0), "1 h 03 min");
        assert_eq!(fmt_minutes(0.0), "0:00");
    }
}
