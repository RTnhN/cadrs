//! Times kernel operations on the OCCT and SMLib corpora for `tools/kernel_perf.py`, which
//! runs the same cases through OCCT (`tools/occt_bench`) and compares.
//!
//!     cargo run -r -p cadrs_kernel --example kernel_bench -- run <cases> [--case NAME] [--reps N] [--budget SECS] [--min-time SECS]
//!     cargo run -r -p cadrs_kernel --example kernel_bench -- prep <cases> <cadrs out> <occt out> <step dir>
//!
//! **Cases.** A case file holds blocks `case <name> <category>` … `end`, one command a line,
//! DRAW-like: `var = command args…` (or `command args…` for one that makes nothing). A line
//! starting with `*` is timed; the others set the case up. Arguments are numbers, paths and
//! variables; `a.3` is element 3 (from 0) of a list (`explode_e`), and `sm*-0.005` a number
//! variable (`lprops`) times a number. `tools/occt_bench/occt_bench.cpp` reads the same
//! format; its header lists the commands and the OCCT calls each makes.
//!
//! **run** runs each case on a new kernel each time, at least `--reps` times (default 5) and
//! until the runs have taken `--min-time` (default 1 s; at most 200 runs), but no more once they
//! have taken `--budget` (default 20 s), and prints a line per case: `RESULT <name> <category>
//! ok|err <wall ms> <cpu ms> <runs> [message]`, for the timed lines' sum the median wall time and
//! the mean CPU time (for a case that fails, the failing run's times up to its error). CPU time
//! is the process's (all its threads, from `/proc/self/task/*/schedstat`, less what reading it
//! costs), so under load it stays comparable while the wall time doesn't; the kernel updates it
//! at each tick (1 ms) for a running thread, hence the mean over many runs of a fast case.
//! Threads: rayon's pool, `RAYON_NUM_THREADS=1` for one.
//!
//! **prep** resolves what OCCT can't read: `v = smb file[#n]` (an SMLib model) becomes
//! `v = stepread <step dir>/….step` (the model written as STEP), and `r = smbfillet v rad
//! include a-b …` (SMLib's edges by index ranges, all others when `include` is 0) becomes
//! `r = fillet_at v rad x y z …` (the edges' middle points) in both outputs.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use cadrs_kernel::native::draw::Shape;
use cadrs_kernel::native::math::P3;
use cadrs_kernel::native::smb;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::{EdgeId, Kernel, OffsetSpec, Tessellation};

type R<T> = std::result::Result<T, String>;

/// A variable's value.
enum Val {
    Shape(Shape),
    List(Vec<Shape>),
    Num(f64),
}

/// One command line of a case.
struct Line {
    timed: bool,
    out: Option<String>,
    cmd: String,
    args: Vec<String>,
}

struct Case {
    name: String,
    category: String,
    lines: Vec<Line>,
}

fn parse_cases(text: &str) -> R<Vec<Case>> {
    let mut cases = Vec::new();
    let mut cur: Option<Case> = None;
    for (no, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let toks: Vec<&str> = line.split_whitespace().collect();
        match toks[0] {
            "case" => {
                if toks.len() < 3 {
                    return Err(format!("line {}: case <name> <category>", no + 1));
                }
                cur = Some(Case { name: toks[1].into(), category: toks[2].into(), lines: Vec::new() });
            }
            "end" => cases.extend(cur.take()),
            _ => {
                let c = cur.as_mut().ok_or(format!("line {}: a command outside a case", no + 1))?;
                let (timed, toks) = match toks[0].strip_prefix('*') {
                    Some("") => (true, &toks[1..]),
                    Some(rest) => {
                        let mut t = toks.clone();
                        t[0] = rest;
                        c.lines.push(line_of(true, &t)?);
                        continue;
                    }
                    None => (false, &toks[..]),
                };
                c.lines.push(line_of(timed, toks)?);
            }
        }
    }
    Ok(cases)
}

fn line_of(timed: bool, toks: &[&str]) -> R<Line> {
    let (out, rest) = if toks.len() >= 3 && toks[1] == "=" { (Some(toks[0].to_string()), &toks[2..]) } else { (None, toks) };
    let (cmd, args) = rest.split_first().ok_or("an empty command")?;
    Ok(Line { timed, out, cmd: cmd.to_string(), args: args.iter().map(|s| s.to_string()).collect() })
}

/// On-CPU time of all the process's threads (ns).
fn cpu_ns() -> u64 {
    let Ok(dir) = std::fs::read_dir("/proc/self/task") else { return 0 };
    dir.flatten()
        .filter_map(|e| std::fs::read_to_string(e.path().join("schedstat")).ok())
        .filter_map(|s| s.split_whitespace().next().and_then(|x| x.parse::<u64>().ok()))
        .sum()
}

/// What reading [`cpu_ns`] twice costs (ns), with rayon's pool started: taken off each timed
/// line's CPU time.
fn cpu_overhead() -> u64 {
    rayon::join(|| (), || ());
    let mut d: Vec<u64> = (0..41)
        .map(|_| {
            let a = cpu_ns();
            cpu_ns().saturating_sub(a)
        })
        .collect();
    d.sort_unstable();
    d[d.len() / 2]
}

/// A case being run: the kernel, the variables and the time spent in timed lines.
struct Ctx {
    k: NativeKernel,
    vars: HashMap<String, Val>,
    timing: bool,
    wall: Duration,
    cpu: u64,
    overhead: u64,
}

impl Ctx {
    fn new(overhead: u64) -> Self {
        Self { k: NativeKernel::new(), vars: HashMap::new(), timing: false, wall: Duration::ZERO, cpu: 0, overhead }
    }

    /// Runs `f`, adding its time to the case's when the line is timed.
    fn time<T>(&mut self, f: impl FnOnce(&mut NativeKernel) -> T) -> T {
        if !self.timing {
            return f(&mut self.k);
        }
        let (c0, t0) = (cpu_ns(), Instant::now());
        let r = f(&mut self.k);
        self.wall += t0.elapsed();
        self.cpu += cpu_ns().saturating_sub(c0).saturating_sub(self.overhead);
        r
    }

    fn shape(&self, tok: &str) -> R<Shape> {
        if let Some((v, i)) = tok.rsplit_once('.')
            && let Ok(i) = i.parse::<usize>()
        {
            return match self.vars.get(v) {
                Some(Val::List(l)) => l.get(i).cloned().ok_or(format!("{v} has {} elements", l.len())),
                _ => Err(format!("{v} isn't a list")),
            };
        }
        match self.vars.get(tok) {
            Some(Val::Shape(s)) => Ok(s.clone()),
            _ => Err(format!("{tok} isn't a shape")),
        }
    }

    fn num(&self, tok: &str) -> R<f64> {
        if let Ok(x) = tok.parse::<f64>() {
            return Ok(x);
        }
        let (v, c) = tok.split_once('*').ok_or(format!("{tok} isn't a number"))?;
        let c: f64 = c.parse().map_err(|_| format!("{tok} isn't a number"))?;
        match self.vars.get(v) {
            Some(Val::Num(x)) => Ok(x * c),
            _ => Err(format!("{v} isn't a number")),
        }
    }

    fn nums<const N: usize>(&self, a: &[String]) -> R<[f64; N]> {
        if a.len() < N {
            return Err(format!("{N} numbers expected"));
        }
        let mut out = [0.0; N];
        for (o, t) in out.iter_mut().zip(a) {
            *o = self.num(t)?;
        }
        Ok(out)
    }

    fn body(&self, tok: &str) -> R<cadrs_kernel::BodyId> {
        match self.shape(tok)? {
            Shape::Body(b) => Ok(b),
            _ => Err(format!("{tok} isn't a solid or sheet")),
        }
    }

    fn exec(&mut self, l: &Line) -> R<Option<Val>> {
        self.timing = l.timed;
        let a = &l.args;
        let arg = |i: usize| a.get(i).map(String::as_str).ok_or(format!("{}: argument {} is missing", l.cmd, i + 1));
        let e = |x: cadrs_kernel::KernelError| format!("{x:?}");
        let s = |v: Shape| Ok(Some(Val::Shape(v)));
        match l.cmd.as_str() {
            "restore" => {
                let p = arg(0)?.to_string();
                s(self.time(|k| k.restore(&p)).map_err(e)?)
            }
            "box" => {
                let [dx, dy, dz] = self.nums(a)?;
                s(self.time(|k| k.box_(dx, dy, dz)).map_err(e)?)
            }
            "boxat" => {
                let [x, y, z, dx, dy, dz] = self.nums(a)?;
                s(self.time(|k| k.box_at(x, y, z, dx, dy, dz)).map_err(e)?)
            }
            "pcylinder" => {
                let [r, h] = self.nums(a)?;
                s(self.time(|k| k.pcylinder(r, h)).map_err(e)?)
            }
            "pcone" => {
                let [r1, r2, h] = self.nums(a)?;
                s(self.time(|k| k.pcone(r1, r2, h)).map_err(e)?)
            }
            "psphere" => {
                let [r] = self.nums(a)?;
                s(self.time(|k| k.psphere(r)).map_err(e)?)
            }
            "ptorus" => {
                let [big, r] = self.nums(a)?;
                s(self.time(|k| k.ptorus(big, r)).map_err(e)?)
            }
            "ttranslate" => {
                let sh = self.shape(arg(0)?)?;
                let [dx, dy, dz] = self.nums(&a[1..])?;
                s(self.time(|k| k.ttranslate(&sh, dx, dy, dz)).map_err(e)?)
            }
            "trotate" => {
                let sh = self.shape(arg(0)?)?;
                let [x, y, z, dx, dy, dz, ang] = self.nums(&a[1..])?;
                s(self.time(|k| k.trotate(&sh, x, y, z, dx, dy, dz, ang)).map_err(e)?)
            }
            "tscale" => {
                let sh = self.shape(arg(0)?)?;
                let [x, y, z, f] = self.nums(&a[1..])?;
                s(self.time(|k| k.tscale(&sh, x, y, z, f)).map_err(e)?)
            }
            "explode_e" | "nexplode_e" | "explode_f" | "nexplode_f" => {
                let sh = self.shape(arg(0)?)?;
                let c = l.cmd.clone();
                let r = self.time(|k| match c.as_str() {
                    "explode_e" => k.explode_edges(&sh),
                    "nexplode_e" => k.nexplode_edges(&sh),
                    "explode_f" => k.explode_faces(&sh),
                    _ => k.nexplode_faces(&sh),
                });
                Ok(Some(Val::List(r.map_err(e)?)))
            }
            "bcut" | "bfuse" | "bcommon" | "bsection" => {
                let (x, y) = (self.shape(arg(0)?)?, self.shape(arg(1)?)?);
                let c = l.cmd.clone();
                s(self
                    .time(|k| match c.as_str() {
                        "bcut" => k.bcut(&x, &y),
                        "bfuse" => k.bfuse(&x, &y),
                        "bcommon" => k.bcommon(&x, &y),
                        _ => k.bsection(&x, &y),
                    })
                    .map_err(e)?)
            }
            "blend" => {
                let sh = self.shape(arg(0)?)?;
                let r = self.num(arg(1)?)?;
                let edges = a[2..].iter().map(|t| self.shape(t)).collect::<R<Vec<_>>>()?;
                let refs: Vec<&Shape> = edges.iter().collect();
                s(self.time(|k| k.blend(&sh, r, &refs)).map_err(e)?)
            }
            "wire" => {
                let parts = a.iter().map(|t| self.shape(t)).collect::<R<Vec<_>>>()?;
                let refs: Vec<&Shape> = parts.iter().collect();
                s(self.time(|k| k.wire(&refs)).map_err(e)?)
            }
            "polyline" => {
                let pts = a.chunks(3).map(|c| self.nums::<3>(c)).collect::<R<Vec<_>>>()?;
                s(self.time(|k| k.polyline(&pts)).map_err(e)?)
            }
            "prism" => {
                let sh = self.shape(arg(0)?)?;
                let [dx, dy, dz] = self.nums(&a[1..])?;
                s(self.time(|k| k.prism(&sh, dx, dy, dz)).map_err(e)?)
            }
            "revol" => {
                let sh = self.shape(arg(0)?)?;
                let [x, y, z, dx, dy, dz, ang] = self.nums(&a[1..])?;
                s(self.time(|k| k.revol(&sh, x, y, z, dx, dy, dz, ang)).map_err(e)?)
            }
            "lprops" => {
                let sh = self.shape(arg(0)?)?;
                Ok(Some(Val::Num(self.time(|k| k.lprops(&sh)).map_err(e)?)))
            }
            "mkoffset" => {
                let sh = self.shape(arg(0)?)?;
                let d = self.num(arg(1)?)?;
                s(self.time(|k| k.mkoffset(&sh, d)).map_err(e)?)
            }
            "mkplane" => {
                let sh = self.shape(arg(0)?)?;
                s(self.time(|k| k.mkplane(&sh)).map_err(e)?)
            }
            "offsetperform" => {
                let sh = self.shape(arg(0)?)?;
                let d = self.num(arg(1)?)?;
                let arc = arg(2)? == "1";
                let faces = a[3..].iter().map(|t| self.shape(t)).collect::<R<Vec<_>>>()?;
                let refs: Vec<&Shape> = faces.iter().collect();
                s(self.time(|k| k.offsetperform(&sh, d, &refs, arc)).map_err(e)?)
            }
            "pipe" => {
                let (sp, pr) = (self.shape(arg(0)?)?, self.shape(arg(1)?)?);
                s(self.time(|k| k.pipe(&sp, &pr)).map_err(e)?)
            }
            "buildsweep" => {
                // spine mode dx dy dz transition solid (section contact correction)…
                let sp = self.shape(arg(0)?)?;
                let mode = arg(1)?.to_string();
                let dir = self.nums::<3>(&a[2..])?;
                let tr = arg(5)?.chars().next().unwrap_or('M');
                let solid = arg(6)? == "1";
                let secs = a[7..]
                    .chunks(3)
                    .map(|c| match c {
                        [v, t, r] => Ok((self.shape(v)?, t == "1", r == "1")),
                        _ => Err("buildsweep: section contact correction".to_string()),
                    })
                    .collect::<R<Vec<_>>>()?;
                let refs: Vec<(&Shape, bool, bool)> = secs.iter().map(|(v, t, r)| (v, *t, *r)).collect();
                s(self.time(|k| k.buildsweep(&sp, &refs, &mode, dir, tr, solid)).map_err(e)?)
            }
            "depouille" => {
                // shape dx dy dz (face angle px py pz nx ny nz)…
                let sh = self.shape(arg(0)?)?;
                let dir = self.nums::<3>(&a[1..])?;
                let groups = a[4..]
                    .chunks(8)
                    .map(|c| {
                        if c.len() != 8 {
                            return Err("depouille: face angle px py pz nx ny nz".to_string());
                        }
                        let n = self.nums::<7>(&c[1..])?;
                        Ok((self.shape(&c[0])?, n[0], [n[1], n[2], n[3]], [n[4], n[5], n[6]]))
                    })
                    .collect::<R<Vec<_>>>()?;
                let refs: Vec<_> = groups.iter().map(|(f, ang, p, n)| (f, *ang, *p, *n)).collect();
                s(self.time(|k| k.depouille(&sh, dir, &refs)).map_err(e)?)
            }
            "smb" => {
                let m = read_smb(arg(0)?)?;
                s(Shape::Body(self.time(|k| k.insert(m.body))))
            }
            "stepread" => {
                let bytes = std::fs::read(arg(0)?).map_err(|x| format!("{}: {x}", arg(0).unwrap_or("")))?;
                let ids = self.time(|k| k.import_step(&bytes)).map_err(e)?;
                s(match ids.as_slice() {
                    [one] => Shape::Body(*one),
                    _ => Shape::Compound(ids.into_iter().map(Shape::Body).collect()),
                })
            }
            "stepwrite" => {
                let ids = match self.shape(arg(0)?)? {
                    Shape::Body(b) => vec![b],
                    Shape::Compound(v) => v.into_iter().filter_map(|x| if let Shape::Body(b) = x { Some(b) } else { None }).collect(),
                    _ => return Err("stepwrite takes bodies".into()),
                };
                let bytes = self.time(|k| k.export_step(&ids)).map_err(e)?;
                std::fs::write(arg(1)?, bytes).map_err(|x| x.to_string())?;
                Ok(None)
            }
            "fillet_at" => {
                // shape radius (x y z)…: the edges through those points.
                let b = self.body(arg(0)?)?;
                let r = self.num(arg(1)?)?;
                let pts = a[2..].chunks(3).map(|c| self.nums::<3>(c).map(|p| P3::new(p[0], p[1], p[2]))).collect::<R<Vec<_>>>()?;
                let ids = pick_edges(self.k.get(b).map_err(e)?, &pts)?;
                let res = self.time(|k| k.fillet(b, &ids, r)).map_err(e)?;
                s(Shape::Body(res.bodies.first().copied().ok_or("fillet made nothing")?))
            }
            "offset" => {
                // shape distance: every face moved, edges rounded (SMLib's offset).
                let b = self.body(arg(0)?)?;
                let d = self.num(arg(1)?)?;
                let res = self.time(|k| k.offset(b, &OffsetSpec { distance: d, faces: Vec::new(), sharp: false })).map_err(e)?;
                s(Shape::Body(res.bodies.first().copied().ok_or("offset made nothing")?))
            }
            "tess" => {
                // shape chord angle: the chord a fraction of the bounding box's diagonal.
                let b = self.body(arg(0)?)?;
                let [rel, angle] = self.nums(&a[1..])?;
                let chord = rel * self.k.get(b).map_err(e)?.bbox().diagonal();
                let m = self.time(|k| k.tessellate(b, Tessellation { deflection: chord, angle })).map_err(e)?;
                if m.indices.is_empty() {
                    return Err("an empty mesh".into());
                }
                Ok(None)
            }
            "mass" => {
                // (OCCT's side takes a precision too; the kernel has one, its Gauss rules'.)
                let b = self.body(arg(0)?)?;
                self.time(|k| k.mass_properties(b)).map_err(e)?;
                Ok(None)
            }
            c => Err(format!("unknown command {c}")),
        }
    }
}

fn read_smb(spec: &str) -> R<smb::Brep> {
    let (file, n) = match spec.rsplit_once('#') {
        Some((f, n)) => (f, n.parse::<usize>().map_err(|x| x.to_string())?),
        None => (spec, 0),
    };
    let bytes = std::fs::read(file).map_err(|x| format!("{file}: {x}"))?;
    if file.ends_with(".smp") {
        let mut p = smb::read_part(&bytes)?;
        if n >= p.breps.len() {
            return Err(format!("{file} has {} B-reps", p.breps.len()));
        }
        Ok(p.breps.swap_remove(n))
    } else {
        smb::read_brep(&bytes)
    }
}

/// The body's edges nearest each point, each once (as the SMLib suites pick them).
fn pick_edges(body: &cadrs_kernel::native::topo::Body, pts: &[P3]) -> R<Vec<EdgeId>> {
    let mut ids = Vec::new();
    for p in pts {
        let best = (0..body.edges.len())
            .filter(|&i| !body.edges[i].degenerate)
            .map(|i| {
                let ed = &body.edges[i];
                let t = ed.curve.closest(p, ed.t[0], ed.t[1]);
                ((ed.point(t) - p).norm(), i)
            })
            .min_by(|x, y| x.0.total_cmp(&y.0))
            .ok_or("no edges")?;
        let id = EdgeId(best.1 as u64);
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    Ok(ids)
}

/// One run of a case: the timed lines' wall and CPU time.
/// One run of a case: the timed lines' wall and CPU time, up to and including a line that fails,
/// and that line's error.
fn run_once(c: &Case, overhead: u64) -> (Duration, u64, Option<String>) {
    let mut ctx = Ctx::new(overhead);
    for l in &c.lines {
        match ctx.exec(l) {
            Ok(v) => {
                if let (Some(name), Some(v)) = (&l.out, v) {
                    ctx.vars.insert(name.clone(), v);
                }
            }
            Err(m) => return (ctx.wall, ctx.cpu, Some(format!("{}: {m}", l.cmd))),
        }
    }
    (ctx.wall, ctx.cpu, None)
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { 0.5 * (v[n / 2 - 1] + v[n / 2]) }
}

/// The most runs of a fast case.
const MAX_RUNS: usize = 200;

fn run(cases: &[Case], only: Option<&str>, reps: usize, budget: f64, min_time: f64) {
    let overhead = cpu_overhead();
    for c in cases.iter().filter(|c| only.is_none_or(|o| o == c.name)) {
        let (mut walls, mut cpus) = (Vec::new(), Vec::new());
        let mut err = None;
        let start = Instant::now();
        for r in 0..MAX_RUNS {
            let t = start.elapsed().as_secs_f64();
            if (r >= reps.max(1) && t >= min_time) || (r >= 1 && t > budget) {
                break;
            }
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_once(c, overhead))) {
                Ok((w, cpu, failed)) => {
                    walls.push(w.as_secs_f64() * 1e3);
                    cpus.push(cpu as f64 * 1e-6);
                    err = failed;
                }
                Err(_) => {
                    (walls, cpus) = (Vec::new(), Vec::new());
                    err = Some("panic".into());
                }
            }
            if err.is_some() {
                break;
            }
        }
        match err {
            // The time to the error, of the one run that failed (0 after a panic).
            Some(m) => {
                let (w, cpu) = (walls.last().copied().unwrap_or(0.0), cpus.last().copied().unwrap_or(0.0));
                println!("RESULT\t{}\t{}\terr\t{w:.3}\t{cpu:.3}\t1\t{}", c.name, c.category, m.replace(['\t', '\n'], " "));
            }
            None => println!("RESULT\t{}\t{}\tok\t{:.3}\t{:.3}\t{}", c.name, c.category, median(&mut walls), cpus.iter().sum::<f64>() / cpus.len() as f64, walls.len()),
        }
    }
}

/// `prep`: the case files for each side (see the module docs).
fn prep(cases: &[Case], step_dir: &str) -> R<(String, String)> {
    std::fs::create_dir_all(step_dir).map_err(|x| x.to_string())?;
    let (mut ours, mut occt) = (String::new(), String::new());
    let mut written: HashMap<String, String> = HashMap::new();
    for c in cases {
        let (mut o, mut t) = (format!("case {} {}\n", c.name, c.category), format!("case {} {}\n", c.name, c.category));
        let mut models: HashMap<String, smb::Brep> = HashMap::new();
        let mut ok = true;
        for l in &c.lines {
            let star = if l.timed { "*" } else { "" };
            let lhs = l.out.as_ref().map(|v| format!("{v} = ")).unwrap_or_default();
            match l.cmd.as_str() {
                "smb" => {
                    let spec = &l.args[0];
                    let m = read_smb(spec)?;
                    let path = match written.get(spec) {
                        Some(p) => p.clone(),
                        None => {
                            let base = spec.rsplit('/').next().unwrap_or(spec).replace(['#', '.'], "_");
                            let path = format!("{step_dir}/{base}.step");
                            let mut k = NativeKernel::new();
                            let id = k.insert(m.body.clone());
                            match k.export_step(&[id]) {
                                Ok(bytes) => std::fs::write(&path, bytes).map_err(|x| x.to_string())?,
                                Err(x) => {
                                    eprintln!("{}: {spec} can't be written as STEP: {x:?}", c.name);
                                    ok = false;
                                }
                            }
                            written.insert(spec.clone(), path.clone());
                            path
                        }
                    };
                    o += &format!("{star}{lhs}smb {spec}\n");
                    t += &format!("{star}{lhs}stepread {path}\n");
                    if let Some(v) = &l.out {
                        models.insert(v.clone(), m);
                    }
                }
                "smbfillet" => {
                    // r = smbfillet v rad include a-b …
                    let m = models.get(&l.args[0]).ok_or(format!("{}: smbfillet of an unknown model", c.name))?;
                    let include = l.args[2] == "1";
                    let ranges = l.args[3..]
                        .iter()
                        .map(|r| {
                            let (x, y) = r.split_once('-').unwrap_or((r, r));
                            Ok((x.parse::<usize>().map_err(|e| e.to_string())?, y.parse::<usize>().map_err(|e| e.to_string())?))
                        })
                        .collect::<R<Vec<_>>>()?;
                    let pts: Vec<String> = (0..m.edge_mids.len())
                        .filter(|i| ranges.iter().any(|&(x, y)| (x..=y).contains(i)) == include)
                        .map(|i| format!("{:.17e} {:.17e} {:.17e}", m.edge_mids[i].x, m.edge_mids[i].y, m.edge_mids[i].z))
                        .collect();
                    let line = format!("{star}{lhs}fillet_at {} {} {}\n", l.args[0], l.args[1], pts.join(" "));
                    o += &line;
                    t += &line;
                }
                _ => {
                    let line = format!("{star}{lhs}{} {}\n", l.cmd, l.args.join(" "));
                    o += &line;
                    t += &line;
                }
            }
        }
        if ok {
            ours += &o;
            ours += "end\n";
            occt += &t;
            occt += "end\n";
        }
    }
    Ok((ours, occt))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let usage = "kernel_bench run <cases> [--case NAME] [--reps N] [--budget SECS] | prep <cases> <cadrs out> <occt out> <step dir>";
    let (Some(mode), Some(file)) = (args.first(), args.get(1)) else {
        eprintln!("{usage}");
        std::process::exit(2);
    };
    let cases = match std::fs::read_to_string(file).map_err(|e| e.to_string()).and_then(|t| parse_cases(&t)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{file}: {e}");
            std::process::exit(2);
        }
    };
    match (mode.as_str(), &args[2..]) {
        ("run", _) => {
            let reps = flag("--reps").and_then(|s| s.parse().ok()).unwrap_or(5);
            let budget = flag("--budget").and_then(|s| s.parse().ok()).unwrap_or(20.0);
            let min_time = flag("--min-time").and_then(|s| s.parse().ok()).unwrap_or(1.0);
            run(&cases, flag("--case").as_deref(), reps, budget, min_time);
        }
        ("prep", [ours, occt, dir, ..]) => match prep(&cases, dir) {
            Ok((a, b)) => {
                if let Err(e) = std::fs::write(ours, a).and_then(|_| std::fs::write(occt, b)) {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
        _ => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    }
}
