//! Post processors: toolpaths into the G-code a machine's controller runs.
//!
//! [`PostSettings`] are the machine's post options (as the Create machine dialog's Post
//! settings): output unit, datums, sequence numbers, rapid positioning, arcs and helices,
//! decimals, forced retracts and canned cycles. A [`Dialect`] is the controller family: Fanuc,
//! Haas-style, LinuxCNC, GRBL and Mach3. Beam machines differ in how the beam switches:
//!
//! - **Laser (GRBL)**: `M4` dynamic power (`M3` constant when chosen) with the power as `S`;
//!   `$32` laser mode makes GRBL stop the beam on rapids.
//! - **Plasma (LinuxCNC QtPlasmaC, Mach3 THC)**: `M3`/`M5` fire the torch, the pierce is a
//!   dwell at the pierce height, then the torch drops to its cut height; THC is enabled with
//!   `M65`/`M64` (LinuxCNC) or `M1003`/`M1004`-style codes are left to the user (Mach3 uses the
//!   screen THC by default).
//! - **Waterjet**: `M3`/`M5` open and close the jet, with the pierce dwell.
//!
//! [`post`] writes a whole program: header comments, units, the work offset, each operation
//! (tool change, spindle, coolant, its moves), and the end.

use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::geom::{P2, P3, dist};
use crate::machine::{Machine, MachineKind};
use crate::tool::{Hand, Tool, Unit};
use crate::toolpath::{Coolant, CycleKind, DrillCycle, Step, Toolpath};

/// The controller family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dialect {
    Fanuc,
    Haas,
    LinuxCnc,
    Grbl,
    Mach3,
}

impl Dialect {
    pub const ALL: [Dialect; 5] = [
        Dialect::Fanuc,
        Dialect::Haas,
        Dialect::LinuxCnc,
        Dialect::Grbl,
        Dialect::Mach3,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Dialect::Fanuc => "Fanuc",
            Dialect::Haas => "Haas-style",
            Dialect::LinuxCnc => "LinuxCNC",
            Dialect::Grbl => "GRBL",
            Dialect::Mach3 => "Mach3/Mach4",
        }
    }

    /// The file extension programs for it usually have.
    pub fn extension(self) -> &'static str {
        match self {
            Dialect::Fanuc | Dialect::Haas => "nc",
            Dialect::LinuxCnc => "ngc",
            Dialect::Grbl => "gcode",
            Dialect::Mach3 => "tap",
        }
    }

    /// Supports canned drilling cycles.
    pub fn has_cycles(self) -> bool {
        !matches!(self, Dialect::Grbl)
    }

    /// Wraps the program in `%` and numbers it `O1001`.
    fn program_number(self) -> bool {
        matches!(self, Dialect::Fanuc | Dialect::Haas)
    }

    fn comment(self, text: &str) -> String {
        // Parentheses can't nest in a comment.
        format!("({})", text.replace(['(', ')'], ""))
    }
}

/// How a laser's power follows its speed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LaserMode {
    /// `M4`: the power scales with the actual speed (cleaner corners).
    #[default]
    Dynamic,
    /// `M3`: the power stays constant.
    Constant,
}

/// A machine's post options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostSettings {
    /// Program: the unit the NC code is written in, whatever the job's unit.
    pub output_unit: Unit,
    /// Datums: allow work planes with their own datum (G54.1 P…/G55…); off writes every
    /// operation in the setup's datum.
    pub advanced_work_planes: bool,
    /// NC general: line numbers.
    pub sequence_numbers: bool,
    pub block_start: u32,
    pub block_increment: u32,
    /// Line numbers only at tool changes.
    pub numbers_at_tools_only: bool,
    /// Movement: rapids as G0 (off: G1 at `rapid_rate`).
    pub rapid_positioning: bool,
    /// The feed rapids get without rapid positioning (mm/min).
    pub rapid_rate: f64,
    /// Arcs as G2/G3 (off: short lines).
    pub output_arcs: bool,
    /// Arcs fitted in the XY plane (also from lines that follow a circle).
    pub arcs_xy: bool,
    pub arcs_xz: bool,
    pub arcs_yz: bool,
    /// Helical arcs (an arc that changes Z).
    pub output_helices: bool,
    /// Decimals of coordinates.
    pub decimals: u32,
    /// Retracts: to the machine's Z home (G28/G53) at each tool change.
    pub force_retract_on_tool_change: bool,
    /// At each work plane change.
    pub force_retract_on_plane_change: bool,
    /// At the program's end.
    pub force_retract_on_end: bool,
    /// Fixed cycles: canned cycles for drilling (G81/G82), peck drilling (G83), tapping (G84)
    /// and chip breaking (G73). A cycle that's off is written as moves.
    pub canned_drill: bool,
    pub canned_peck: bool,
    pub canned_tap: bool,
    pub canned_chip_break: bool,
    /// Write the operations' names and the tools as comments.
    pub comments: bool,
    /// Laser power: dynamic (M4) or constant (M3).
    pub laser_mode: LaserMode,
}

impl PostSettings {
    /// The defaults for a controller and machine kind.
    pub fn for_dialect(d: Dialect, kind: MachineKind) -> PostSettings {
        let cycles = d.has_cycles() && kind.is_milling();
        PostSettings {
            output_unit: Unit::Mm,
            advanced_work_planes: matches!(d, Dialect::Fanuc | Dialect::Haas),
            sequence_numbers: matches!(d, Dialect::Fanuc | Dialect::Haas),
            block_start: 10,
            block_increment: 5,
            numbers_at_tools_only: false,
            rapid_positioning: true,
            rapid_rate: 5_000.0,
            output_arcs: true,
            arcs_xy: true,
            arcs_xz: false,
            arcs_yz: false,
            output_helices: d != Dialect::Mach3,
            decimals: if d == Dialect::Grbl { 3 } else { 4 },
            force_retract_on_tool_change: kind.is_milling() && d != Dialect::Grbl,
            force_retract_on_plane_change: false,
            force_retract_on_end: kind.is_milling() && d != Dialect::Grbl,
            canned_drill: cycles,
            canned_peck: cycles,
            canned_tap: cycles,
            canned_chip_break: cycles,
            comments: true,
            laser_mode: LaserMode::Dynamic,
        }
    }

    fn canned(&self, k: CycleKind) -> bool {
        match k {
            CycleKind::Drill => self.canned_drill,
            CycleKind::Peck => self.canned_peck,
            CycleKind::Tap => self.canned_tap,
            CycleKind::ChipBreak => self.canned_chip_break,
        }
    }
}

/// One operation of a program.
#[derive(Debug, Clone, PartialEq)]
pub struct NcOperation {
    pub name: String,
    pub tool: Tool,
    /// The tool number (T) the job gave it.
    pub tool_number: u32,
    pub toolpath: Toolpath,
    pub coolant: Coolant,
    /// The work offset it's written in: 1 = G54, 2 = G55, … (7+: G54.1 P1…).
    pub datum: u32,
    /// Pause for the operator before it ("M0").
    pub stop_before: bool,
    /// Make the tool change even when the same tool is in.
    pub force_tool_change: bool,
}

/// What to post.
#[derive(Debug, Clone, PartialEq)]
pub struct NcProgram {
    pub name: String,
    pub number: u32,
    pub operations: Vec<NcOperation>,
}

/// Why a program couldn't be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostError {
    Empty,
    /// An operation's tool doesn't fit the machine (a laser on a mill).
    WrongTool(String),
    /// A move leaves the machine's travel.
    OutOfTravel(String),
}

impl std::fmt::Display for PostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostError::Empty => write!(f, "There is nothing to post"),
            PostError::WrongTool(s) => write!(f, "{s}"),
            PostError::OutOfTravel(s) => write!(f, "{s}"),
        }
    }
}

/// The datum code for a work offset number.
pub fn datum_code(n: u32) -> String {
    match n {
        0 | 1 => "G54".into(),
        2..=6 => format!("G{}", 53 + n),
        n => format!("G54.1 P{}", n - 6),
    }
}

struct Writer<'a> {
    s: &'a PostSettings,
    d: Dialect,
    out: String,
    line: u32,
    /// The scale from mm to the output unit.
    k: f64,
    /// The last position written (mm), for modal output.
    at: [Option<f64>; 3],
    feed: Option<f64>,
    /// The motion mode last written (0, 1, 2, 3).
    mode: Option<u8>,
    /// Number the next line even with numbers at tools only.
    number_next: bool,
}

impl<'a> Writer<'a> {
    fn new(s: &'a PostSettings, d: Dialect) -> Self {
        Writer {
            s,
            d,
            out: String::new(),
            line: s.block_start,
            k: 1.0 / s.output_unit.mm(),
            at: [None; 3],
            feed: None,
            mode: None,
            number_next: false,
        }
    }

    fn raw(&mut self, text: &str) {
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn block(&mut self, text: &str) {
        let numbered =
            self.s.sequence_numbers && (!self.s.numbers_at_tools_only || self.number_next);
        if numbered {
            let n = self.line;
            self.line += self.s.block_increment.max(1);
            let _ = writeln!(self.out, "N{n} {text}");
        } else {
            self.raw(text);
        }
        self.number_next = false;
    }

    fn comment(&mut self, text: &str) {
        if self.s.comments {
            let c = self.d.comment(text);
            self.raw(&c);
        }
    }

    fn num(&self, v: f64) -> String {
        let d = self.s.decimals.clamp(0, 6) as usize;
        let s = format!("{:.*}", d, v * self.k);
        let s = if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        };
        if s == "-0" { "0".into() } else { s }
    }

    fn feed_word(&mut self, f: f64) -> String {
        if self.feed.is_some_and(|x| (x - f).abs() < 1e-9) {
            String::new()
        } else {
            self.feed = Some(f);
            let d = if self.s.output_unit == Unit::Inch {
                2
            } else {
                0
            };
            let v = f * self.k;
            format!(
                " F{}",
                if d == 0 {
                    format!("{}", v.round())
                } else {
                    format!("{v:.2}")
                }
            )
        }
    }

    /// The axis words of a move to `p` (only the axes that change).
    fn axes(&mut self, p: P3) -> String {
        let mut w = String::new();
        for (i, a) in ["X", "Y", "Z"].iter().enumerate() {
            let v = self.num(p[i]);
            let old = self.at[i].map(|o| self.num(o));
            if old.as_deref() != Some(v.as_str()) {
                let _ = write!(w, " {a}{v}");
            }
            self.at[i] = Some(p[i]);
        }
        w
    }

    fn motion(&mut self, mode: u8) -> String {
        if self.mode == Some(mode) && mode <= 1 {
            String::new()
        } else {
            self.mode = Some(mode);
            format!("G{mode}")
        }
    }

    fn rapid(&mut self, p: P3) {
        let axes = self.axes(p);
        if axes.is_empty() {
            return;
        }
        if self.s.rapid_positioning {
            let m = self.motion(0);
            let line = format!("{m}{axes}").trim().to_string();
            self.block(&line);
        } else {
            let m = self.motion(1);
            let f = self.feed_word(self.s.rapid_rate);
            let line = format!("{m}{axes}{f}").trim().to_string();
            self.block(&line);
        }
    }

    fn linear(&mut self, p: P3, feed: f64) {
        let axes = self.axes(p);
        if axes.is_empty() {
            return;
        }
        let m = self.motion(1);
        let f = self.feed_word(feed);
        let line = format!("{m}{axes}{f}").trim().to_string();
        self.block(&line);
    }

    fn arc(&mut self, from: P3, to: P3, center: P2, cw: bool, feed: f64) {
        let mode = if cw { 2 } else { 3 };
        self.mode = None;
        let i = self.num(center[0] - from[0]);
        let j = self.num(center[1] - from[1]);
        let mut axes = String::new();
        for (k, a) in ["X", "Y", "Z"].iter().enumerate() {
            // An arc writes X and Y always (a full circle ends where it starts).
            if k < 2 || self.at[k].map(|o| self.num(o)) != Some(self.num(to[k])) {
                let _ = write!(axes, " {a}{}", self.num(to[k]));
            }
            self.at[k] = Some(to[k]);
        }
        let f = self.feed_word(feed);
        self.block(&format!("G{mode}{axes} I{i} J{j}{f}"));
        self.mode = Some(mode);
    }
}

/// The program for `machine`, or why it can't be written.
pub fn post(program: &NcProgram, machine: &Machine, s: &PostSettings) -> Result<String, PostError> {
    if program.operations.iter().all(|o| o.toolpath.is_empty()) {
        return Err(PostError::Empty);
    }
    for op in &program.operations {
        if !op.tool.kind.fits(machine.kind) {
            return Err(PostError::WrongTool(format!(
                "{}: a {} can't run on a {}",
                op.name,
                op.tool.kind.label().to_lowercase(),
                machine.kind.label().to_lowercase()
            )));
        }
    }
    let d = machine.dialect;
    let mut w = Writer::new(s, d);
    if d.program_number() {
        w.raw("%");
        let name = program.name.to_uppercase().replace(['(', ')'], "");
        w.raw(&format!("O{:04} ({name})", program.number.min(9999)));
    }
    w.comment(&format!(
        "{} for {} ({})",
        program.name,
        machine.title(),
        d.label()
    ));
    for op in &program.operations {
        w.comment(&format!("T{} {}", op.tool_number, op.tool.name));
    }
    let unit = match s.output_unit {
        Unit::Mm => "G21",
        Unit::Inch => "G20",
    };
    let plane = if machine.kind.is_milling() {
        " G17"
    } else {
        ""
    };
    match d {
        Dialect::Grbl => w.block(&format!("G90 G94{plane} {unit}")),
        Dialect::LinuxCnc => w.block(&format!("G90 G94{plane} G40 G49 {unit} G64 P0.01")),
        _ => w.block(&format!("G90 G94{plane} G40 G49 G80 {unit}")),
    }
    if machine.kind == MachineKind::Plasma && d == Dialect::LinuxCnc {
        // QtPlasmaC: cut mode 1, take the materials from the table.
        w.comment("QtPlasmaC: torch height control from the material table");
    }
    let mut tool_in: Option<u32> = None;
    let mut datum: Option<u32> = None;
    let mut coolant = Coolant::Off;
    let beam = machine.kind.is_cutting();
    for op in &program.operations {
        if op.toolpath.is_empty() {
            continue;
        }
        w.comment(&op.name);
        // Tool change.
        if machine.kind.is_milling() && (tool_in != Some(op.tool_number) || op.force_tool_change) {
            if tool_in.is_some() {
                w.block("M5");
                if coolant != Coolant::Off {
                    w.block("M9");
                    coolant = Coolant::Off;
                }
                if s.force_retract_on_tool_change {
                    retract_home(&mut w, d);
                }
            }
            w.number_next = true;
            if machine.tool_changer {
                w.block(&format!("T{} M6", op.tool_number));
                if d != Dialect::Grbl {
                    w.block(&format!("G43 H{}", op.tool_number));
                }
            } else {
                w.comment(&format!("Change to T{}: {}", op.tool_number, op.tool.name));
                match d {
                    Dialect::Grbl => w.block("M0"),
                    _ => {
                        w.block(&format!("T{} M6", op.tool_number));
                        w.block(&format!("G43 H{}", op.tool_number));
                    }
                }
            }
            tool_in = Some(op.tool_number);
            w.at = [None; 3];
            w.mode = None;
        } else if op.stop_before {
            w.block("M0");
        }
        // Work offset.
        let want = if s.advanced_work_planes {
            op.datum.max(1)
        } else {
            1
        };
        if datum != Some(want) {
            if datum.is_some() && s.force_retract_on_plane_change {
                retract_home(&mut w, d);
            }
            let code = datum_code(want);
            w.block(&code);
            datum = Some(want);
            w.at = [None; 3];
        }
        post_moves(&mut w, op, machine, s, beam, &mut coolant);
    }
    // End.
    if beam {
        w.block("M5");
    } else {
        w.block("M5");
        if coolant != Coolant::Off {
            w.block("M9");
        }
        if s.force_retract_on_end {
            retract_home(&mut w, d);
        }
    }
    w.block(if d == Dialect::Grbl { "M2" } else { "M30" });
    if d.program_number() {
        w.raw("%");
    }
    Ok(w.out)
}

fn retract_home(w: &mut Writer, d: Dialect) {
    match d {
        Dialect::Fanuc | Dialect::Haas => w.block("G91 G28 Z0"),
        Dialect::LinuxCnc | Dialect::Mach3 => w.block("G53 G0 Z0"),
        Dialect::Grbl => w.block("G53 G0 Z-1"),
    }
    if matches!(d, Dialect::Fanuc | Dialect::Haas) {
        w.block("G90");
    }
    w.at[2] = None;
    w.mode = None;
}

fn spindle_word(rpm: f64, cw: bool) -> String {
    format!("S{} {}", rpm.round(), if cw { "M3" } else { "M4" })
}

fn coolant_code(c: Coolant) -> Option<&'static str> {
    match c {
        Coolant::Off => None,
        Coolant::Flood => Some("M8"),
        Coolant::Mist => Some("M7"),
        Coolant::Air => Some("M7"),
    }
}

fn post_moves(
    w: &mut Writer,
    op: &NcOperation,
    machine: &Machine,
    s: &PostSettings,
    beam: bool,
    coolant: &mut Coolant,
) {
    let d = machine.dialect;
    let tool = &op.tool;
    let mut at: P3 = [f64::NAN; 3];
    let max_s = machine.max_spindle.max(1.0);
    // Lines that follow a circle become arcs (when arcs are on); collect feed runs first.
    let steps = if s.output_arcs && s.arcs_xy {
        fit_arcs(&op.toolpath.steps, 0.005)
    } else {
        op.toolpath.steps.clone()
    };
    for st in &steps {
        match st {
            Step::Comment(c) => w.comment(c),
            Step::Spindle { rpm, cw } => {
                if !beam {
                    let cw = *cw == (tool.hand == Hand::Right);
                    let word = spindle_word(rpm.min(machine.max_spindle), cw);
                    w.block(&word);
                    if op.coolant != *coolant {
                        if let Some(c) = coolant_code(op.coolant) {
                            w.block(c);
                        } else {
                            w.block("M9");
                        }
                        *coolant = op.coolant;
                    }
                }
            }
            Step::SpindleOff => {
                if !beam {
                    w.block("M5");
                }
            }
            Step::Coolant(c) => {
                if c != coolant {
                    match coolant_code(*c) {
                        Some(code) => w.block(code),
                        None => w.block("M9"),
                    }
                    *coolant = *c;
                }
            }
            Step::BeamOn { power } => match machine.kind {
                MachineKind::Laser => {
                    let m = match s.laser_mode {
                        LaserMode::Dynamic => "M4",
                        LaserMode::Constant => "M3",
                    };
                    let sv = (power.clamp(0.0, 1.0) * max_s).round();
                    w.block(&format!("{m} S{sv}"));
                    if tool.beam.air_assist {
                        w.block("M8");
                    }
                }
                _ => {
                    if d == Dialect::LinuxCnc && machine.kind == MachineKind::Plasma {
                        w.block("M3 $0 S1");
                    } else {
                        w.block("M3");
                    }
                }
            },
            Step::BeamOff => {
                if machine.kind == MachineKind::Plasma && d == Dialect::LinuxCnc {
                    w.block("M5 $0");
                } else {
                    w.block("M5");
                }
                if machine.kind == MachineKind::Laser && tool.beam.air_assist {
                    w.block("M9");
                }
            }
            Step::Dwell(sec) => {
                let v = match d {
                    // Fanuc-style P is in milliseconds when it has no point; use X seconds.
                    Dialect::Fanuc | Dialect::Haas => format!("G4 X{sec:.3}"),
                    _ => format!("G4 P{sec:.3}"),
                };
                let v = v.trim_end_matches('0').trim_end_matches('.').to_string();
                w.block(&v);
            }
            Step::Rapid(p) => {
                // A laser in GRBL laser mode turns off on G0 itself.
                w.rapid(*p);
                at = *p;
            }
            Step::Feed { to, feed, .. } => {
                w.linear(*to, feed.min(machine.max_feed));
                at = *to;
            }
            Step::Arc {
                to,
                center,
                cw,
                feed,
                ..
            } => {
                let f = feed.min(machine.max_feed);
                let helix = (to[2] - at[2]).abs() > 1e-9;
                let r_ok = !at[0].is_nan();
                if s.output_arcs && s.arcs_xy && r_ok && (!helix || s.output_helices) {
                    if dist(at, *to) < 1e-9 && !helix {
                        // A full circle: two halves (some controls refuse one full circle).
                        let mid = [2.0 * center[0] - at[0], 2.0 * center[1] - at[1], at[2]];
                        w.arc(at, mid, *center, *cw, f);
                        w.arc(mid, *to, *center, *cw, f);
                    } else {
                        w.arc(at, *to, *center, *cw, f);
                    }
                } else {
                    let start = if r_ok { at } else { *to };
                    for p in crate::toolpath::arc_points(start, *to, *center, *cw, 0.005) {
                        w.linear(p, f);
                    }
                }
                at = *to;
            }
            Step::Drill(c) => {
                post_drill(w, c, s, d, tool, &mut at);
            }
        }
    }
    if w.mode.is_some_and(|m| (81..=89).contains(&m)) {
        w.block("G80");
        w.mode = None;
    }
}

fn post_drill(
    w: &mut Writer,
    c: &DrillCycle,
    s: &PostSettings,
    d: Dialect,
    tool: &Tool,
    at: &mut P3,
) {
    if s.canned(c.kind) && d.has_cycles() {
        let code = match c.kind {
            CycleKind::Drill if c.dwell > 0.0 => "G82",
            CycleKind::Drill => "G81",
            CycleKind::Peck => "G83",
            CycleKind::ChipBreak => "G73",
            CycleKind::Tap => {
                if tool.hand == Hand::Left {
                    "G74"
                } else {
                    "G84"
                }
            }
        };
        // Over the hole at the retract height first.
        if !at[0].is_nan() && at[2] < c.retract {
            w.rapid([at[0], at[1], c.retract]);
        }
        w.rapid([c.at[0], c.at[1], c.retract]);
        let mut words = format!(
            "G98 {code} X{} Y{} Z{} R{}",
            w.num(c.at[0]),
            w.num(c.at[1]),
            w.num(c.bottom),
            w.num(c.r_plane)
        );
        if matches!(c.kind, CycleKind::Peck | CycleKind::ChipBreak) && c.peck > 0.0 {
            let _ = write!(words, " Q{}", w.num(c.peck));
        }
        if c.dwell > 0.0 {
            let _ = write!(words, " P{}", (c.dwell * 1000.0).round());
        }
        // (A tap's feed is its pitch × rpm already.)
        w.feed = None;
        let fw = w.feed_word(c.feed);
        words.push_str(&fw);
        w.block(&words);
        w.block("G80");
        w.mode = None;
        w.at = [Some(c.at[0]), Some(c.at[1]), Some(c.retract)];
        *at = [c.at[0], c.at[1], c.retract];
    } else {
        // As moves.
        let tp = Toolpath {
            steps: vec![Step::Drill(*c)],
            warnings: vec![],
        };
        let start = if at[0].is_nan() {
            [c.at[0], c.at[1], c.retract]
        } else {
            *at
        };
        for seg in tp.motion(start, 1.0) {
            match seg.feed {
                None => w.rapid(seg.to),
                Some(f) => {
                    if c.dwell > 0.0
                        && (seg.from[2] - c.bottom).abs() < 1e-9
                        && seg.to[2] > seg.from[2]
                    {
                        let line = format!("G4 P{:.3}", c.dwell);
                        w.block(&line);
                    }
                    w.linear(seg.to, f)
                }
            }
        }
        *at = [c.at[0], c.at[1], c.retract];
    }
}

/// Runs of feed moves at one feed and height that follow a circle within `tol`, replaced by
/// arcs (the strategies make polylines; G2/G3 are shorter and smoother on the machine).
pub fn fit_arcs(steps: &[Step], tol: f64) -> Vec<Step> {
    let mut out: Vec<Step> = Vec::with_capacity(steps.len());
    let mut i = 0;
    // The position before step i.
    let mut at: Option<P3> = None;
    while i < steps.len() {
        if let (Some(start), Step::Feed { feed, kind, .. }) = (at, &steps[i]) {
            // The longest run from i that fits one arc (at least 4 segments).
            let mut pts = vec![start];
            let mut j = i;
            while j < steps.len() {
                match &steps[j] {
                    Step::Feed {
                        to,
                        feed: f,
                        kind: k,
                    } if (f - feed).abs() < 1e-9
                        && k == kind
                        && (to[2] - start[2]).abs() < 1e-9 =>
                    {
                        pts.push(*to);
                        j += 1;
                    }
                    _ => break,
                }
            }
            let mut best: Option<(usize, P2, bool)> = None;
            let mut n = pts.len();
            while n >= 5 {
                if let Some((c, cw)) = fits_circle(&pts[..n], tol) {
                    best = Some((n, c, cw));
                    break;
                }
                n -= 1;
            }
            if let Some((n, c, cw)) = best {
                out.push(Step::Arc {
                    to: pts[n - 1],
                    center: c,
                    cw,
                    feed: *feed,
                    kind: *kind,
                });
                at = Some(pts[n - 1]);
                i += n - 1;
                continue;
            }
        }
        match &steps[i] {
            Step::Rapid(p) | Step::Feed { to: p, .. } | Step::Arc { to: p, .. } => at = Some(*p),
            Step::Drill(c) => at = Some([c.at[0], c.at[1], c.retract]),
            _ => {}
        }
        out.push(steps[i].clone());
        i += 1;
    }
    out
}

/// Whether the points lie on one circle turning one way (within `tol`, sweeping less than a
/// full turn, chords short against the radius); its centre and direction.
fn fits_circle(pts: &[P3], tol: f64) -> Option<(P2, bool)> {
    let n = pts.len();
    let (a, b, c) = (pts[0], pts[n / 2], pts[n - 1]);
    let center = circumcenter([a[0], a[1]], [b[0], b[1]], [c[0], c[1]])?;
    let r = (a[0] - center[0]).hypot(a[1] - center[1]);
    if r.is_nan() || r <= tol * 10.0 || r > 5_000.0 {
        return None;
    }
    let mut turn = 0.0;
    let mut sign = 0.0;
    for w in pts.windows(2) {
        let (p, q) = (w[0], w[1]);
        let rq = (q[0] - center[0]).hypot(q[1] - center[1]);
        if (rq - r).abs() > tol {
            return None;
        }
        // The chord's middle stays within tol of the circle too.
        let m = [(p[0] + q[0]) / 2.0, (p[1] + q[1]) / 2.0];
        if r - (m[0] - center[0]).hypot(m[1] - center[1]) > tol {
            return None;
        }
        let a0 = (p[1] - center[1]).atan2(p[0] - center[0]);
        let a1 = (q[1] - center[1]).atan2(q[0] - center[0]);
        let mut da = a1 - a0;
        while da > std::f64::consts::PI {
            da -= std::f64::consts::TAU;
        }
        while da < -std::f64::consts::PI {
            da += std::f64::consts::TAU;
        }
        if da.abs() < 1e-9 {
            return None;
        }
        if sign == 0.0 {
            sign = da.signum();
        } else if da.signum() != sign {
            return None;
        }
        turn += da;
    }
    (turn.abs() < std::f64::consts::TAU - 0.05).then_some((center, sign < 0.0))
}

fn circumcenter(a: P2, b: P2, c: P2) -> Option<P2> {
    let d = 2.0 * (a[0] * (b[1] - c[1]) + b[0] * (c[1] - a[1]) + c[0] * (a[1] - b[1]));
    if d.abs() < 1e-12 {
        return None;
    }
    let (a2, b2, c2) = (
        a[0] * a[0] + a[1] * a[1],
        b[0] * b[0] + b[1] * b[1],
        c[0] * c[0] + c[1] * c[1],
    );
    Some([
        (a2 * (b[1] - c[1]) + b2 * (c[1] - a[1]) + c2 * (a[1] - b[1])) / d,
        (a2 * (c[0] - b[0]) + b2 * (a[0] - c[0]) + c2 * (b[0] - a[0])) / d,
    ])
}

/// A file name for a program: its name with characters files can't have replaced, and the
/// dialect's extension.
pub fn file_name(name: &str, d: Dialect) -> String {
    let stem: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stem = stem.trim_matches('_');
    format!(
        "{}.{}",
        if stem.is_empty() { "program" } else { stem },
        d.extension()
    )
}

/// A short summary line of a program for the export dialog: lines and unit.
pub fn summary(nc: &str, s: &PostSettings) -> String {
    format!(
        "{} lines, {}",
        nc.lines().count(),
        match s.output_unit {
            Unit::Mm => "millimetres",
            Unit::Inch => "inches",
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::catalog;
    use crate::tool::ToolKind;
    use crate::toolpath::FeedKind;

    fn machine(kind: MachineKind, d: Dialect) -> Machine {
        catalog()
            .into_iter()
            .find(|m| m.kind == kind && m.dialect == d)
            .unwrap()
    }

    fn square_op(tool: Tool, beam: bool) -> NcOperation {
        let mut steps = vec![Step::Rapid([0.0, 0.0, 5.0])];
        if beam {
            steps.push(Step::BeamOn { power: 0.5 });
        } else {
            steps.push(Step::Spindle {
                rpm: 10_000.0,
                cw: true,
            });
            steps.push(Step::Feed {
                to: [0.0, 0.0, -1.0],
                feed: 200.0,
                kind: FeedKind::Plunge,
            });
        }
        for p in [[10.0, 0.0], [10.0, 10.0], [0.0, 10.0], [0.0, 0.0]] {
            steps.push(Step::Feed {
                to: [p[0], p[1], if beam { 0.0 } else { -1.0 }],
                feed: 800.0,
                kind: FeedKind::Cut,
            });
        }
        if beam {
            steps.push(Step::BeamOff);
        }
        steps.push(Step::Rapid([0.0, 0.0, 5.0]));
        NcOperation {
            name: "Profile 1".into(),
            tool,
            tool_number: 1,
            toolpath: Toolpath {
                steps,
                warnings: vec![],
            },
            coolant: Coolant::Flood,
            datum: 1,
            stop_before: false,
            force_tool_change: false,
        }
    }

    #[test]
    fn fanuc_program_shape() {
        let m = machine(MachineKind::Mill, Dialect::Fanuc);
        let p = NcProgram {
            name: "Bracket".into(),
            number: 1001,
            operations: vec![square_op(Tool::new(ToolKind::EndMill, 6.0), false)],
        };
        let nc = post(&p, &m, &m.post).unwrap();
        assert!(nc.starts_with("%\nO1001 (BRACKET)"));
        assert!(nc.contains("T1 M6"));
        assert!(nc.contains("G43 H1"));
        assert!(nc.contains("S10000 M3"));
        assert!(nc.contains("M8"));
        assert!(nc.contains("G54"));
        assert!(nc.contains("X10"));
        assert!(nc.trim_end().ends_with("M30\n%") || nc.trim_end().ends_with('%'));
        assert!(nc.contains("N10 "));
    }

    #[test]
    fn grbl_laser_uses_dynamic_power() {
        let m = machine(MachineKind::Laser, Dialect::Grbl);
        let p = NcProgram {
            name: "Sign".into(),
            number: 1,
            operations: vec![square_op(Tool::new(ToolKind::Laser, 0.1), true)],
        };
        let nc = post(&p, &m, &m.post).unwrap();
        assert!(nc.contains("M4 S500"), "{nc}");
        assert!(!nc.contains("M6"));
        assert!(nc.trim_end().ends_with("M2"));
        assert!(!nc.contains('%'));
    }

    #[test]
    fn plasma_linuxcnc_torch_codes() {
        let m = machine(MachineKind::Plasma, Dialect::LinuxCnc);
        let p = NcProgram {
            name: "Bracket".into(),
            number: 1,
            operations: vec![square_op(Tool::new(ToolKind::Plasma, 1.5), true)],
        };
        let nc = post(&p, &m, &m.post).unwrap();
        assert!(nc.contains("M3 $0 S1"));
        assert!(nc.contains("M5 $0"));
    }

    #[test]
    fn wrong_tool_is_refused() {
        let m = machine(MachineKind::Laser, Dialect::Grbl);
        let p = NcProgram {
            name: "x".into(),
            number: 1,
            operations: vec![square_op(Tool::new(ToolKind::EndMill, 6.0), false)],
        };
        assert!(matches!(
            post(&p, &m, &m.post),
            Err(PostError::WrongTool(_))
        ));
    }

    #[test]
    fn inch_output_scales() {
        let m = machine(MachineKind::Mill, Dialect::LinuxCnc);
        let mut s = m.post.clone();
        s.output_unit = Unit::Inch;
        let p = NcProgram {
            name: "x".into(),
            number: 1,
            operations: vec![square_op(Tool::new(ToolKind::EndMill, 6.0), false)],
        };
        let nc = post(&p, &m, &s).unwrap();
        assert!(nc.contains("G20"));
        // 10 mm = 0.3937 in.
        assert!(nc.contains("X0.3937"), "{nc}");
    }

    #[test]
    fn canned_cycles_and_expansion() {
        let m = machine(MachineKind::Mill, Dialect::Fanuc);
        let cycle = DrillCycle {
            at: [10.0, 20.0],
            r_plane: 2.0,
            bottom: -12.0,
            retract: 10.0,
            feed: 120.0,
            kind: CycleKind::Peck,
            peck: 3.0,
            dwell: 0.0,
            pitch: 0.0,
        };
        let op = NcOperation {
            name: "Drill".into(),
            tool: Tool::new(ToolKind::Drill, 5.0),
            tool_number: 3,
            toolpath: Toolpath {
                steps: vec![
                    Step::Spindle {
                        rpm: 3000.0,
                        cw: true,
                    },
                    Step::Rapid([10.0, 20.0, 10.0]),
                    Step::Drill(cycle),
                ],
                warnings: vec![],
            },
            coolant: Coolant::Off,
            datum: 1,
            stop_before: false,
            force_tool_change: false,
        };
        let p = NcProgram {
            name: "Holes".into(),
            number: 2,
            operations: vec![op],
        };
        let nc = post(&p, &m, &m.post).unwrap();
        assert!(nc.contains("G98 G83 X10 Y20 Z-12 R2 Q3 F120"), "{nc}");
        let mut s = m.post.clone();
        s.canned_peck = false;
        let nc = post(&p, &m, &s).unwrap();
        assert!(!nc.contains("G83"));
        assert!(nc.contains("Z-12"));
    }

    #[test]
    fn arcs_are_fitted() {
        let mut steps = vec![Step::Rapid([10.0, 0.0, -1.0])];
        for i in 1..=16 {
            let a = std::f64::consts::PI * i as f64 / 16.0;
            steps.push(Step::Feed {
                to: [10.0 * a.cos(), 10.0 * a.sin(), -1.0],
                feed: 500.0,
                kind: FeedKind::Cut,
            });
        }
        let out = fit_arcs(&steps, 0.05);
        assert_eq!(out.len(), 2, "{out:?}");
        match &out[1] {
            Step::Arc { center, cw, .. } => {
                assert!(center[0].abs() < 1e-6 && center[1].abs() < 1e-6);
                assert!(!cw);
            }
            s => panic!("{s:?}"),
        }
    }

    #[test]
    fn datum_codes() {
        assert_eq!(datum_code(1), "G54");
        assert_eq!(datum_code(6), "G59");
        assert_eq!(datum_code(7), "G54.1 P1");
        assert_eq!(
            file_name("My part (v2)", Dialect::LinuxCnc),
            "My_part__v2.ngc"
        );
    }
}
