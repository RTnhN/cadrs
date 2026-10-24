//! Toolpath strategies.
//!
//! A toolpath picks a [`Type`] (what it machines: edges, faces, bodies or holes) and a
//! [`Pattern`] (how), as the Create toolpath dialog does; the patterns a tool and machine can
//! use are [`Pattern::available`]. Its [`Params`] are the dialog's settings: the Tool path tab
//! (drive selection, offsets, step down and over, sorting, heights, speeds, posting), the Link
//! tab (clearance, entry, leads, gaps) and the Misc tab (smoothing, tolerance), plus the beam
//! settings of laser, plasma and waterjet cuts.
//!
//! [`generate`] makes the toolpath from an [`Input`]: the parts in the setup's coordinates
//! ([`crate::model::Model`]), the stock, the tool and, for rest machining, the in-process
//! stock left by the operations before.
//!
//! - [`mill`]: Face, 2-axis rough (pockets and bosses at levels), 2-axis profile, 2-axis
//!   chamfer, Engrave, Flatlands and the 3D Rough (constant-Z roughing).
//! - [`finish`]: the 3D finishing patterns, Parallel (drop-cutter raster) and Constant Z
//!   (waterlines).
//! - [`holes`]: Drill (drill, peck, chip break and tap cycles) and Bore (helical hole milling).
//! - [`beam`]: Cut (through-cutting with kerf compensation, pierces, leads and microjoints,
//!   holes before outlines), Etch (marking along edges) and Fill (laser hatching).

pub mod adaptive;
pub mod beam;
pub mod collision;
pub mod finish;
pub mod holes;
pub mod link;
pub mod mill;
pub mod path;

use serde::{Deserialize, Serialize};

use crate::geom::{Aabb, P2};
use crate::heightmap::HeightMap;
use crate::machine::MachineKind;
use crate::model::Model;
use crate::tool::{Tool, ToolKind};
use crate::toolpath::{Coolant, CycleKind, Step, Toolpath};

/// What a toolpath machines (the dialog's Type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Wireframe drive edges (no machining faces needed).
    Edges,
    /// Selected faces.
    Faces,
    /// Whole bodies.
    Bodies,
    /// Holes.
    Holes,
}

impl Type {
    pub const ALL: [Type; 4] = [Type::Edges, Type::Faces, Type::Bodies, Type::Holes];

    pub fn label(self) -> &'static str {
        match self {
            Type::Edges => "Edges",
            Type::Faces => "Faces",
            Type::Bodies => "Bodies",
            Type::Holes => "Holes",
        }
    }
}

/// How a toolpath cuts (the dialog's Pattern).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pattern {
    /// Flat passes over the stock's top (or selected flat faces).
    Face,
    /// Clears pockets and around bosses level by level (2.5 axis).
    Rough2D,
    /// Follows drive edges or the part's outline, with depth passes, tabs and leads.
    Profile2D,
    /// Breaks edges with a chamfer mill.
    Chamfer2D,
    /// Follows drive edges at a depth.
    Engrave,
    /// Finishes horizontal floors.
    Flatlands,
    /// Removes the bulk of the stock around any shape, level by level.
    Rough3D,
    /// Roughing at a constant radial engagement: deep, light, fast.
    Adaptive,
    /// Raster passes over the surfaces (drop cutter).
    Parallel,
    /// Waterline passes round the steep walls.
    ConstantZ,
    /// Rings offset across the shallow surfaces: an even step over along them.
    Scallop,
    /// One spiral out from the middle over the surfaces.
    Spiral,
    /// Along the concave corners and fillets tighter than the tool.
    Pencil,
    /// Drilling, peck drilling and tapping cycles.
    Drill,
    /// Helical milling of holes with an end mill.
    Bore,
    /// Threads cut with a thread mill on a helix.
    ThreadMill,
    /// Through-cutting with a beam, kerf compensated.
    Cut,
    /// Marking along edges with a beam (no compensation).
    Etch,
    /// Laser hatching of faces.
    Fill,
}

impl Pattern {
    pub const ALL: [Pattern; 19] = [
        Pattern::Face,
        Pattern::Rough2D,
        Pattern::Profile2D,
        Pattern::Chamfer2D,
        Pattern::Engrave,
        Pattern::Flatlands,
        Pattern::Rough3D,
        Pattern::Adaptive,
        Pattern::Parallel,
        Pattern::ConstantZ,
        Pattern::Scallop,
        Pattern::Spiral,
        Pattern::Pencil,
        Pattern::Drill,
        Pattern::Bore,
        Pattern::ThreadMill,
        Pattern::Cut,
        Pattern::Etch,
        Pattern::Fill,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Pattern::Face => "Face",
            Pattern::Rough2D => "2-axis rough",
            Pattern::Profile2D => "2-axis profile",
            Pattern::Chamfer2D => "2-axis chamfer",
            Pattern::Engrave => "Engrave",
            Pattern::Flatlands => "Flatlands",
            Pattern::Rough3D => "Rough",
            Pattern::Adaptive => "Adaptive clearing",
            Pattern::Parallel => "Parallel",
            Pattern::ConstantZ => "Constant Z",
            Pattern::Scallop => "Scallop",
            Pattern::Spiral => "Spiral",
            Pattern::Pencil => "Pencil",
            Pattern::ThreadMill => "Thread mill",
            Pattern::Drill => "Drill",
            Pattern::Bore => "Bore",
            Pattern::Cut => "Cut",
            Pattern::Etch => "Etch",
            Pattern::Fill => "Fill",
        }
    }

    /// The toolpath's default name ("2-axis profile 1" is the caller's job).
    pub fn type_of(self) -> Type {
        match self {
            Pattern::Profile2D
            | Pattern::Chamfer2D
            | Pattern::Engrave
            | Pattern::Cut
            | Pattern::Etch => Type::Edges,
            Pattern::Face | Pattern::Rough2D | Pattern::Flatlands | Pattern::Fill => Type::Faces,
            Pattern::Rough3D
            | Pattern::Adaptive
            | Pattern::Parallel
            | Pattern::ConstantZ
            | Pattern::Scallop
            | Pattern::Spiral
            | Pattern::Pencil => Type::Bodies,
            Pattern::Drill | Pattern::Bore | Pattern::ThreadMill => Type::Holes,
        }
    }

    /// The icon for the tree (icon-rs).
    pub fn icon(self) -> &'static str {
        match self {
            Pattern::Drill | Pattern::Bore | Pattern::ThreadMill => "cam-drill-cycle",
            Pattern::Cut | Pattern::Etch | Pattern::Fill => "cam-cut-path",
            _ => "cam-toolpath",
        }
    }

    /// Whether `tool` on a `machine` can run it.
    pub fn allowed(self, tool: ToolKind, machine: MachineKind) -> bool {
        if !tool.fits(machine) {
            return false;
        }
        use ToolKind::*;
        match self {
            Pattern::Cut | Pattern::Etch => tool.is_beam(),
            Pattern::Fill => tool == Laser,
            Pattern::Drill => matches!(tool, Drill | Tap | ChamferMill),
            Pattern::Bore => matches!(tool, EndMill | BullMill | BallMill),
            Pattern::Chamfer2D => matches!(tool, ChamferMill),
            Pattern::Engrave => matches!(tool, ChamferMill | BallMill | EndMill | TaperMill),
            Pattern::Face => matches!(tool, FaceMill | EndMill | BullMill),
            Pattern::Rough2D
            | Pattern::Rough3D
            | Pattern::Adaptive
            | Pattern::Profile2D
            | Pattern::Flatlands => {
                matches!(tool, EndMill | BullMill | BallMill | FaceMill | TaperMill)
            }
            Pattern::Parallel | Pattern::ConstantZ | Pattern::Scallop | Pattern::Spiral => {
                matches!(tool, EndMill | BullMill | BallMill | TaperMill)
            }
            Pattern::Pencil => matches!(tool, BallMill | BullMill),
            Pattern::ThreadMill => tool == ThreadMill,
        }
    }

    /// The patterns of `kind` a tool and machine can run, in list order.
    pub fn available(kind: Type, tool: ToolKind, machine: MachineKind) -> Vec<Pattern> {
        Pattern::ALL
            .into_iter()
            .filter(|p| p.type_of() == kind && p.allowed(tool, machine))
            .collect()
    }
}

/// Which side of a contour the tool runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Side {
    /// Outside outer outlines, inside holes.
    #[default]
    Auto,
    Outside,
    Inside,
    /// On the line (no compensation).
    On,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::Auto, Side::Outside, Side::Inside, Side::On];
    pub fn label(self) -> &'static str {
        match self {
            Side::Auto => "Automatic",
            Side::Outside => "Outside",
            Side::Inside => "Inside",
            Side::On => "On the line",
        }
    }
}

/// Sub-pattern of the roughing patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SubPattern {
    /// Rings offset from the walls.
    #[default]
    Offset,
    /// Parallel passes, then a ring round the walls.
    Parallel,
}

impl SubPattern {
    pub const ALL: [SubPattern; 2] = [SubPattern::Offset, SubPattern::Parallel];
    pub fn label(self) -> &'static str {
        match self {
            SubPattern::Offset => "Offset",
            SubPattern::Parallel => "Parallel",
        }
    }
}

/// Cutting direction: climb (down) milling keeps the material on the tool's right with a
/// clockwise spindle; conventional on its left. For a beam, "climb" keeps the part on the right
/// (a plasma torch's good side with a clockwise swirl).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Direction {
    #[default]
    Climb,
    Conventional,
}

impl Direction {
    pub const ALL: [Direction; 2] = [Direction::Climb, Direction::Conventional];
    pub fn label(self) -> &'static str {
        match self {
            Direction::Climb => "Climb",
            Direction::Conventional => "Conventional",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CuttingMethod {
    #[default]
    OneWay,
    Zigzag,
}

impl CuttingMethod {
    pub const ALL: [CuttingMethod; 2] = [CuttingMethod::OneWay, CuttingMethod::Zigzag];
    pub fn label(self) -> &'static str {
        match self {
            CuttingMethod::OneWay => "One way",
            CuttingMethod::Zigzag => "Zigzag",
        }
    }
}

/// Machine by levels (every region at one depth before the next) or by regions (each region to
/// the bottom before the next).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum MachineBy {
    #[default]
    Levels,
    Regions,
}

impl MachineBy {
    pub const ALL: [MachineBy; 2] = [MachineBy::Levels, MachineBy::Regions];
    pub fn label(self) -> &'static str {
        match self {
            MachineBy::Levels => "Levels",
            MachineBy::Regions => "Regions",
        }
    }
}

/// What a height is measured from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeightRef {
    StockTop,
    StockBottom,
    ModelTop,
    ModelBottom,
    /// The drive geometry's (selected edges' or faces') height; the model's when none.
    Selection,
    /// Absolute, in the setup's coordinates.
    Origin,
}

impl HeightRef {
    pub const ALL: [HeightRef; 6] = [
        HeightRef::StockTop,
        HeightRef::StockBottom,
        HeightRef::ModelTop,
        HeightRef::ModelBottom,
        HeightRef::Selection,
        HeightRef::Origin,
    ];
    pub fn label(self) -> &'static str {
        match self {
            HeightRef::StockTop => "Stock top",
            HeightRef::StockBottom => "Stock bottom",
            HeightRef::ModelTop => "Model top",
            HeightRef::ModelBottom => "Model bottom",
            HeightRef::Selection => "Selection",
            HeightRef::Origin => "Origin",
        }
    }
}

/// A height: a reference and an offset above it (mm).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Height {
    pub from: HeightRef,
    pub offset: f64,
}

impl Height {
    pub fn new(from: HeightRef, offset: f64) -> Height {
        Height { from, offset }
    }
}

/// The Heights group.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Heights {
    /// Where cutting starts.
    pub top: Height,
    /// Where it ends.
    pub bottom: Height,
}

/// How the tool gets between cuts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkAction {
    /// The shortest move at feed, staying down.
    Direct,
    /// Up to the feed distance above the material, across, down.
    RetractToFeed,
    /// Up to the retract (rapid) distance above the stock.
    RetractToRapid,
    /// Up to the clearance height.
    RetractToClearance,
}

impl LinkAction {
    pub const ALL: [LinkAction; 4] = [
        LinkAction::Direct,
        LinkAction::RetractToFeed,
        LinkAction::RetractToRapid,
        LinkAction::RetractToClearance,
    ];
    pub fn label(self) -> &'static str {
        match self {
            LinkAction::Direct => "Direct",
            LinkAction::RetractToFeed => "Retract to feed distance",
            LinkAction::RetractToRapid => "Retract to rapid distance",
            LinkAction::RetractToClearance => "Retract to clearance area",
        }
    }
}

/// How the tool enters the material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Entry {
    /// Straight down at the plunge feed.
    Plunge,
    /// Down along the path at the ramp angle.
    #[default]
    Ramp,
}

impl Entry {
    pub const ALL: [Entry; 2] = [Entry::Plunge, Entry::Ramp];
    pub fn label(self) -> &'static str {
        match self {
            Entry::Plunge => "Plunge",
            Entry::Ramp => "Ramp",
        }
    }
}

/// A lead onto or off a contour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LeadKind {
    None,
    /// A straight line along the normal.
    Line,
    /// A tangent arc.
    #[default]
    Arc,
}

impl LeadKind {
    pub const ALL: [LeadKind; 3] = [LeadKind::None, LeadKind::Line, LeadKind::Arc];
    pub fn label(self) -> &'static str {
        match self {
            LeadKind::None => "None",
            LeadKind::Line => "Line",
            LeadKind::Arc => "Arc",
        }
    }
}

/// The Link tab.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Links {
    /// Clearance height above the stock top (mm): where the tool travels between features and
    /// starts and ends.
    pub clearance: f64,
    /// Rapid distance above the stock top (mm).
    pub rapid_distance: f64,
    /// Feed distance (mm): rapids stop this far above the material, then feed.
    pub feed_distance: f64,
    /// Gaps up to this long are small (mm).
    pub small_gap: f64,
    pub small_gaps: LinkAction,
    pub large_gaps: LinkAction,
    pub entry: Entry,
    /// Ramp angle (degrees from horizontal).
    pub ramp_angle: f64,
    pub lead_in: LeadKind,
    pub lead_out: LeadKind,
    /// Lead length or arc radius (mm).
    pub lead_radius: f64,
    /// Extra cut past the start of a closed contour (mm).
    pub overlap: f64,
}

impl Default for Links {
    fn default() -> Self {
        Links {
            clearance: 10.0,
            rapid_distance: 3.0,
            feed_distance: 1.0,
            small_gap: 5.0,
            small_gaps: LinkAction::Direct,
            large_gaps: LinkAction::RetractToRapid,
            entry: Entry::Ramp,
            ramp_angle: 3.0,
            lead_in: LeadKind::Arc,
            lead_out: LeadKind::Arc,
            lead_radius: 2.0,
            overlap: 0.0,
        }
    }
}

/// Tabs left to hold a part cut out of its stock.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Tabs {
    pub enabled: bool,
    /// Per contour.
    pub count: usize,
    /// Along the path (mm).
    pub width: f64,
    /// Above the bottom (mm).
    pub height: f64,
}

impl Default for Tabs {
    fn default() -> Self {
        Tabs {
            enabled: false,
            count: 4,
            width: 5.0,
            height: 1.5,
        }
    }
}

/// Which holes a hole toolpath takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum HoleSelect {
    /// The holes the tool's diameter suits (equal for a drill or tap, larger for Bore).
    #[default]
    MatchingTool,
    /// Every hole.
    All,
    /// Only blind holes.
    Blind,
    /// The selected holes' diameters.
    SameDiameter,
}

impl HoleSelect {
    pub const ALL: [HoleSelect; 4] = [
        HoleSelect::MatchingTool,
        HoleSelect::All,
        HoleSelect::Blind,
        HoleSelect::SameDiameter,
    ];
    pub fn label(self) -> &'static str {
        match self {
            HoleSelect::MatchingTool => "Holes the tool fits",
            HoleSelect::All => "All holes",
            HoleSelect::Blind => "Blind holes",
            HoleSelect::SameDiameter => "Same diameter as selected",
        }
    }
}

/// The drive geometry: what was picked. Empty is automatic (the job's bodies).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Drive {
    /// Edge names (chained into contours).
    #[serde(default)]
    pub edges: Vec<String>,
    /// Face names.
    #[serde(default)]
    pub faces: Vec<String>,
    /// Hole centres picked (for Same diameter, and to limit the holes).
    #[serde(default)]
    pub holes: Vec<[f64; 2]>,
    /// Reverse the picked edges' direction.
    #[serde(default)]
    pub reverse: bool,
}

impl Drive {
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty() && self.faces.is_empty() && self.holes.is_empty()
    }
}

/// Speeds and feeds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Speeds {
    pub spindle_rpm: f64,
    pub feed: f64,
    pub plunge: f64,
    /// Ramp feed (also helices).
    pub ramp: f64,
    /// Lead-in and lead-out feed.
    pub lead: f64,
}

/// The Posting group.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Posting {
    pub coolant: Coolant,
    /// Change tools even when the same tool is in.
    pub force_tool_change: bool,
    /// Stop (M0) before the operation.
    pub stop_before: bool,
    /// A note written into the program.
    pub comment: String,
}

impl Default for Posting {
    fn default() -> Self {
        Posting {
            coolant: Coolant::Flood,
            force_tool_change: false,
            stop_before: false,
            comment: String::new(),
        }
    }
}

/// A hole cycle's settings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Cycle {
    pub kind: CycleKind,
    pub peck: f64,
    pub dwell: f64,
    /// How far a drill's full diameter goes past a through hole's bottom (mm).
    pub break_through: f64,
    /// Retract to the clearance height between holes (else to the R plane).
    pub retract_full: bool,
    pub select: HoleSelect,
}

impl Default for Cycle {
    fn default() -> Self {
        Cycle {
            kind: CycleKind::Drill,
            peck: 2.0,
            dwell: 0.0,
            break_through: 1.0,
            retract_full: false,
            select: HoleSelect::MatchingTool,
        }
    }
}

/// A beam cut's settings beyond the tool's.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BeamCut {
    /// Power (0–100 %).
    pub power: f64,
    pub passes: u32,
    /// How far each pass goes down (laser cutting thick material in passes; mm).
    pub pass_depth: f64,
    /// Small gaps left uncut to hold parts in the sheet.
    pub microjoints: usize,
    pub microjoint_width: f64,
    /// Holes smaller than this get a slower feed (plasma; mm; 0 = off).
    pub small_hole: f64,
    /// Feed factor in small holes.
    pub small_hole_feed: f64,
}

impl Default for BeamCut {
    fn default() -> Self {
        BeamCut {
            power: 80.0,
            passes: 1,
            pass_depth: 0.0,
            microjoints: 0,
            microjoint_width: 0.5,
            small_hole: 0.0,
            small_hole_feed: 0.6,
        }
    }
}

/// The Axis tab: the direction the tool cuts along.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum ToolAxis {
    /// The work plane's Z (the setup's, or the work plane above the toolpath's).
    #[default]
    WorkPlane,
    /// A direction of its own in model coordinates (a picked face's normal): the toolpath is
    /// worked out in the work plane turned to it, cut on a 3+2 machine.
    Direction(crate::geom::P3),
}

/// A toolpath repeated (the Misc tab's pattern): the same moves at more places.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum Repeat {
    #[default]
    None,
    /// `count` copies in all, each `step` (mm) on from the one before.
    Linear { count: u32, step: [f64; 2] },
    /// `count` copies in all round `center`, each turned `angle` (degrees) on.
    Circular {
        count: u32,
        center: [f64; 2],
        angle: f64,
    },
}

impl Repeat {
    /// The copies' placements (the first is the toolpath itself): a point's image under each.
    fn placements(&self) -> Vec<Box<dyn Fn(crate::geom::P2) -> crate::geom::P2>> {
        match *self {
            Repeat::None => vec![Box::new(|p| p)],
            Repeat::Linear { count, step } => (0..count.max(1))
                .map(|k| {
                    let d = [step[0] * k as f64, step[1] * k as f64];
                    Box::new(move |p: crate::geom::P2| [p[0] + d[0], p[1] + d[1]])
                        as Box<dyn Fn(crate::geom::P2) -> crate::geom::P2>
                })
                .collect(),
            Repeat::Circular {
                count,
                center: c,
                angle,
            } => (0..count.max(1))
                .map(|k| {
                    let (sn, cs) = (angle * k as f64).to_radians().sin_cos();
                    Box::new(move |p: crate::geom::P2| {
                        let (x, y) = (p[0] - c[0], p[1] - c[1]);
                        [c[0] + x * cs - y * sn, c[1] + x * sn + y * cs]
                    }) as Box<dyn Fn(crate::geom::P2) -> crate::geom::P2>
                })
                .collect(),
        }
    }

    /// The toolpath with its copies, one after another (each starts and ends at the clearance
    /// height, as a toolpath does).
    pub fn apply(&self, tp: Toolpath) -> Toolpath {
        let places = self.placements();
        if places.len() <= 1 {
            return tp;
        }
        let mut steps = Vec::with_capacity(tp.steps.len() * places.len());
        for (k, f) in places.iter().enumerate() {
            let p3 = |q: crate::geom::P3| {
                let xy = f([q[0], q[1]]);
                [xy[0], xy[1], q[2]]
            };
            for s in &tp.steps {
                steps.push(match s {
                    // The spindle and the comment once.
                    Step::Spindle { .. } | Step::Comment(_) if k > 0 => continue,
                    Step::Rapid(q) => Step::Rapid(p3(*q)),
                    Step::Feed { to, feed, kind } => Step::Feed {
                        to: p3(*to),
                        feed: *feed,
                        kind: *kind,
                    },
                    Step::Arc {
                        to,
                        center,
                        cw,
                        feed,
                        kind,
                    } => Step::Arc {
                        to: p3(*to),
                        center: f(*center),
                        cw: *cw,
                        feed: *feed,
                        kind: *kind,
                    },
                    Step::Drill(c) => {
                        let mut c = *c;
                        c.at = f(c.at);
                        Step::Drill(c)
                    }
                    other => other.clone(),
                });
            }
        }
        Toolpath {
            steps,
            warnings: tp.warnings,
        }
    }
}

/// Everything a toolpath's dialog sets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Params {
    pub pattern: Pattern,
    #[serde(default)]
    pub sub_pattern: SubPattern,
    #[serde(default)]
    pub drive: Drive,
    /// Use the job's bodies (off: only the drive geometry).
    #[serde(default = "yes")]
    pub use_job_bodies: bool,
    #[serde(default)]
    pub side: Side,
    /// Material left on walls (mm).
    pub wall_stock: f64,
    /// Material left on floors (mm).
    pub floor_stock: f64,
    pub step_down: f64,
    pub step_over: f64,
    /// A first, deeper or shallower, step down (0 = like the others).
    #[serde(default)]
    pub first_step_down: f64,
    /// Finishing passes on the walls (profiles).
    #[serde(default = "one")]
    pub passes: u32,
    pub direction: Direction,
    pub cutting_method: CuttingMethod,
    pub machine_by: MachineBy,
    /// Raster angle (degrees from X) of parallel passes.
    pub angle: f64,
    pub heights: Heights,
    /// Depth of an engraving or chamfer width (mm).
    pub depth: f64,
    /// A chamfer mill's tip goes this far below the chamfer (mm).
    pub tip_offset: f64,
    /// Steepness limit of Constant Z (degrees from horizontal; walls flatter are left).
    pub steep_angle: f64,
    pub tabs: Tabs,
    pub links: Links,
    pub speeds: Speeds,
    pub posting: Posting,
    pub cycle: Cycle,
    pub beam: BeamCut,
    /// Rest machining: only where the operations before left material.
    #[serde(default)]
    pub rest: bool,
    /// Round sharp corners of the toolpath by this radius (mm; 0 = off).
    #[serde(default)]
    pub smooth_corners: f64,
    /// Chordal accuracy (mm).
    pub tolerance: f64,
    /// The Collision tab.
    #[serde(default)]
    pub collision: collision::Collision,
    /// The Axis tab.
    #[serde(default)]
    pub axis: ToolAxis,
    /// Thread mill: the thread's major diameter (mm; 0 = from the hole).
    #[serde(default)]
    pub thread_diameter: f64,
    /// Thread mill: a left-hand thread (else right-hand).
    #[serde(default)]
    pub left_hand_thread: bool,
    /// Adaptive clearing: the fine levels' spacing between the deep ones (mm; 0 = none).
    #[serde(default)]
    pub fine_step_down: f64,
    /// The Misc tab's pattern: the toolpath repeated.
    #[serde(default)]
    pub repeat: Repeat,
}

fn yes() -> bool {
    true
}
fn one() -> u32 {
    1
}

impl Params {
    /// The settings a pattern starts with for a tool on a machine.
    pub fn new(pattern: Pattern, tool: &Tool, machine: MachineKind) -> Params {
        let d = tool.diameter;
        let c = &tool.cutting;
        let mut p = Params {
            pattern,
            sub_pattern: SubPattern::Offset,
            drive: Drive::default(),
            use_job_bodies: true,
            side: Side::Auto,
            wall_stock: 0.0,
            floor_stock: 0.0,
            step_down: c.step_down.max(0.05),
            step_over: c.step_over.clamp(d * 0.05, d * 0.95).max(0.01),
            first_step_down: 0.0,
            passes: 1,
            direction: Direction::Climb,
            cutting_method: CuttingMethod::OneWay,
            machine_by: MachineBy::Levels,
            angle: 0.0,
            heights: Heights {
                top: Height::new(HeightRef::StockTop, 0.0),
                bottom: Height::new(HeightRef::ModelBottom, 0.0),
            },
            depth: 0.5,
            tip_offset: 0.5,
            steep_angle: 30.0,
            tabs: Tabs::default(),
            links: Links::default(),
            collision: collision::Collision::default(),
            axis: ToolAxis::WorkPlane,
            thread_diameter: 0.0,
            left_hand_thread: false,
            fine_step_down: 0.0,
            repeat: Repeat::None,
            speeds: Speeds {
                spindle_rpm: c.spindle_rpm,
                feed: c.feed,
                plunge: c.plunge_feed,
                ramp: c.plunge_feed.max(c.feed * 0.5),
                lead: c.feed,
            },
            posting: Posting::default(),
            cycle: Cycle::default(),
            beam: BeamCut {
                power: tool.beam.power,
                passes: tool.beam.passes.max(1),
                ..BeamCut::default()
            },
            rest: false,
            smooth_corners: 0.0,
            tolerance: 0.01,
        };
        match pattern {
            Pattern::Face => {
                p.step_over = d * 0.7;
                p.cutting_method = CuttingMethod::Zigzag;
                p.heights.bottom = Height::new(HeightRef::ModelTop, 0.0);
            }
            Pattern::Rough2D | Pattern::Rough3D => {
                p.wall_stock = if d >= 3.0 { 0.3 } else { 0.1 };
                p.floor_stock = p.wall_stock;
            }
            Pattern::Adaptive => {
                p.wall_stock = if d >= 3.0 { 0.3 } else { 0.1 };
                p.floor_stock = p.wall_stock;
                // Light and deep: a tenth of the diameter across, the flutes down (at most
                // twice the diameter).
                p.step_over = (d * 0.1).max(0.05);
                p.step_down = tool.cutting_length.min(d * 2.0).max(0.1);
                p.fine_step_down = (d * 0.2).max(0.1);
                // At a tenth of the diameter across the chips thin: feed faster.
                p.speeds.feed = c.feed * 2.0;
            }
            Pattern::Profile2D => {
                p.links.lead_radius = (d * 0.5).max(1.0);
            }
            Pattern::Chamfer2D => {
                p.depth = 0.5;
                p.heights.top = Height::new(HeightRef::ModelTop, 0.0);
                p.heights.bottom = Height::new(HeightRef::ModelTop, 0.0);
                p.links.lead_in = LeadKind::None;
                p.links.lead_out = LeadKind::None;
            }
            Pattern::Engrave => {
                p.side = Side::On;
                p.depth = 0.3;
                p.heights.bottom = Height::new(HeightRef::Selection, 0.0);
                p.links.lead_in = LeadKind::None;
                p.links.lead_out = LeadKind::None;
                p.links.entry = Entry::Plunge;
            }
            Pattern::Flatlands => {
                p.step_over = d * 0.5;
            }
            Pattern::Parallel => {
                p.step_over = if tool.kind == ToolKind::BallMill {
                    d * 0.1
                } else {
                    d * 0.2
                };
                p.cutting_method = CuttingMethod::Zigzag;
                p.angle = 45.0;
                p.links.lead_in = LeadKind::None;
                p.links.lead_out = LeadKind::None;
            }
            Pattern::ConstantZ => {
                p.step_down = (d * 0.1).max(0.1);
                p.links.lead_in = LeadKind::Arc;
            }
            Pattern::Scallop | Pattern::Spiral | Pattern::Pencil => {
                p.step_over = if tool.kind == ToolKind::BallMill {
                    d * 0.1
                } else {
                    d * 0.2
                };
                p.links.lead_in = LeadKind::None;
                p.links.lead_out = LeadKind::None;
            }
            Pattern::ThreadMill => {
                p.posting.coolant = Coolant::Flood;
                p.passes = 1;
            }
            Pattern::Drill => {
                p.cycle.kind = if tool.kind == ToolKind::Tap {
                    CycleKind::Tap
                } else {
                    CycleKind::Drill
                };
                p.cycle.peck = d.max(0.5);
                p.posting.coolant = Coolant::Flood;
                p.speeds.feed = c.feed;
                if tool.kind == ToolKind::Tap {
                    p.speeds.feed = tool.pitch * c.spindle_rpm;
                }
            }
            Pattern::Bore => {
                p.step_down = (d * 0.1).max(0.2);
            }
            Pattern::Cut | Pattern::Etch | Pattern::Fill => {
                p.posting.coolant = Coolant::Off;
                p.heights.top = Height::new(HeightRef::StockTop, 0.0);
                p.heights.bottom = Height::new(HeightRef::StockTop, 0.0);
                p.links.lead_in = if pattern == Pattern::Cut {
                    LeadKind::Arc
                } else {
                    LeadKind::None
                };
                p.links.lead_out = LeadKind::None;
                p.links.lead_radius = match machine {
                    MachineKind::Plasma => 4.0,
                    MachineKind::Waterjet => 3.0,
                    _ => 1.0,
                };
                p.links.overlap = if machine == MachineKind::Plasma {
                    1.0
                } else {
                    0.2
                };
                p.links.clearance = match machine {
                    MachineKind::Laser => 0.0,
                    _ => 15.0,
                };
                p.links.small_gaps = LinkAction::RetractToClearance;
                p.links.large_gaps = LinkAction::RetractToClearance;
                p.step_over = d.max(0.05);
                if pattern == Pattern::Etch {
                    p.beam.power = (tool.beam.power * 0.3).max(5.0);
                    p.side = Side::On;
                }
                if pattern == Pattern::Fill {
                    p.beam.power = (tool.beam.power * 0.5).max(5.0);
                    p.step_over = d.max(0.08);
                    p.cutting_method = CuttingMethod::Zigzag;
                }
            }
        }
        if machine == MachineKind::Router || machine == MachineKind::Laser {
            p.posting.coolant = Coolant::Off;
        }
        if !tool.center_cutting {
            p.links.entry = Entry::Ramp;
        }
        p
    }

    /// Problems with the settings themselves.
    pub fn check(&self, tool: &Tool) -> Result<(), String> {
        if self.step_down.is_nan() || self.step_down <= 0.0 {
            return Err("The step down must be more than 0".into());
        }
        if self.step_over.is_nan() || self.step_over <= 0.0 {
            return Err("The step over must be more than 0".into());
        }
        if self.step_over > tool.diameter * 1.0001 && !tool.is_beam() {
            return Err("The step over can't be more than the tool's diameter".into());
        }
        if self.speeds.feed.is_nan() || self.speeds.feed <= 0.0 {
            return Err("The feed rate must be more than 0".into());
        }
        if self.tolerance.is_nan() || self.tolerance <= 0.0 {
            return Err("The tolerance must be more than 0".into());
        }
        if self.tabs.enabled && (self.tabs.width <= 0.0 || self.tabs.height <= 0.0) {
            return Err("Tabs need a width and a height".into());
        }
        Ok(())
    }
}

/// The stock a job starts from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stock {
    pub bounds: Aabb,
    /// A stock body of its own (a casting): its triangles. `None`: the box.
    pub triangles: Option<Vec<[crate::geom::P3; 3]>>,
}

impl Stock {
    pub fn block(bounds: Aabb) -> Stock {
        Stock {
            bounds,
            triangles: None,
        }
    }

    /// The stock's footprint from above.
    pub fn footprint(&self) -> crate::region::Region {
        match &self.triangles {
            Some(t) => Model::from_triangles("stock", t.clone()).silhouette(),
            None => vec![crate::region::rect(
                [self.bounds.min[0], self.bounds.min[1]],
                [self.bounds.max[0], self.bounds.max[1]],
            )],
        }
    }

    /// The stock's top at each point, as a height map.
    pub fn height_map(&self, cell: f64) -> HeightMap {
        match &self.triangles {
            Some(t) => HeightMap::of_triangles(
                t,
                [self.bounds.min[0], self.bounds.min[1]],
                [self.bounds.max[0], self.bounds.max[1]],
                cell,
            ),
            None => HeightMap::block(&self.bounds, cell),
        }
    }
}

/// A rectangle far round the stock and the model: the whole plane as far as a strategy is
/// concerned (the complement of a keep-out is this minus it).
pub fn everywhere(input: &Input) -> crate::region::Ring {
    let mut b = input.stock.bounds;
    let m = input.model.bounds();
    for k in 0..3 {
        b.min[k] = b.min[k].min(m.min[k]);
        b.max[k] = b.max[k].max(m.max[k]);
    }
    crate::region::rect(
        [b.min[0] - 1e4, b.min[1] - 1e4],
        [b.max[0] + 1e4, b.max[1] + 1e4],
    )
}

/// What a strategy works from.
pub struct Input<'a> {
    pub model: &'a Model,
    pub stock: &'a Stock,
    pub tool: &'a Tool,
    pub machine: MachineKind,
    pub params: &'a Params,
    /// The stock as the operations before left it (rest machining, re-machining).
    pub rest: Option<&'a HeightMap>,
    /// The workholding's triangles (vises, clamps, fixture plates): never cut, never touched
    /// by the tool or its holder ([`collision::apply`]).
    pub fixtures: &'a [[crate::geom::P3; 3]],
}

impl Input<'_> {
    /// The z of a height.
    pub fn height(&self, h: Height) -> f64 {
        let mb = self.model.bounds();
        let base = match h.from {
            HeightRef::StockTop => self.stock.bounds.max[2],
            HeightRef::StockBottom => self.stock.bounds.min[2],
            HeightRef::ModelTop => mb.max[2],
            HeightRef::ModelBottom => mb.min[2],
            HeightRef::Selection => self.selection_z().unwrap_or(mb.max[2]),
            HeightRef::Origin => 0.0,
        };
        base + h.offset
    }

    /// The highest material over the disc of radius `r` round `c`: the stock as the operations
    /// before left it, or the stock itself (the top of the box, or of a stock body over the
    /// disc's centre and rim).
    pub fn material_top(&self, c: P2, r: f64) -> f64 {
        if let Some(rest) = self.rest {
            let flat = Tool::new(ToolKind::EndMill, 2.0 * r.max(1e-3));
            return rest.drop_cutter(&flat, c, f64::NEG_INFINITY);
        }
        match &self.stock.triangles {
            None => self.stock.bounds.max[2],
            Some(tris) => {
                let pts = std::iter::once(c).chain((0..8).map(|k| {
                    let a = k as f64 * std::f64::consts::FRAC_PI_4;
                    [c[0] + r * a.cos(), c[1] + r * a.sin()]
                }));
                pts.flat_map(|p| {
                    tris.iter()
                        .filter(move |t| crate::model::point_in_tri2(p, t))
                        .filter_map(move |t| crate::model::height_on(t, p))
                })
                .fold(f64::NEG_INFINITY, f64::max)
            }
        }
    }

    /// The drive geometry's lowest height: picked edges or faces.
    pub fn selection_z(&self) -> Option<f64> {
        let d = &self.params.drive;
        let edges = d
            .edges
            .iter()
            .filter_map(|n| self.model.edge(n))
            .flat_map(|e| e.points.iter().map(|p| p[2]));
        let faces = d
            .faces
            .iter()
            .filter_map(|n| self.model.face(n))
            .flat_map(|f| f.loops.iter().flatten().map(|p| p[2]));
        edges.chain(faces).reduce(f64::min)
    }

    /// Whether the part lies inside a closed contour at height `z`: probed just either side of
    /// its longest segment (a loop round a pocket has the part outside it even though the
    /// part's material is below the pocket's floor).
    pub fn part_inside(&self, c: &path::Contour, z: f64) -> bool {
        let n = c.pts.len();
        let Some(i) = (0..n).max_by(|a, b| {
            let la = crate::geom::dist2(c.pts[*a], c.pts[(*a + 1) % n]);
            let lb = crate::geom::dist2(c.pts[*b], c.pts[(*b + 1) % n]);
            la.total_cmp(&lb)
        }) else {
            return false;
        };
        let (a, b) = (c.pts[i], c.pts[(i + 1) % n]);
        let m = crate::geom::lerp2(a, b, 0.5);
        let t = path::unit(crate::geom::sub2(b, a));
        let e = 0.05;
        let left = [m[0] - t[1] * e, m[1] + t[0] * e];
        let right = [m[0] + t[1] * e, m[1] - t[0] * e];
        let solid = |p: crate::geom::P2| self.model.top_at(p).is_some_and(|top| top >= z - 1e-3);
        let (l, r) = (solid(left), solid(right));
        let left_inside = c.ccw();
        if l != r {
            return if left_inside { l } else { r };
        }
        // Both or neither: whether the material is inside at all.
        let mut ring = c.pts.clone();
        if !left_inside {
            ring.reverse();
        }
        self.model.top_at(path::inner_point(&ring)).is_some()
    }

    pub fn top(&self) -> f64 {
        self.height(self.params.heights.top)
    }

    pub fn bottom(&self) -> f64 {
        self.height(self.params.heights.bottom)
    }

    /// The highest of the stock, the model and the workholding.
    fn highest(&self) -> f64 {
        let fixtures = self
            .fixtures
            .iter()
            .flatten()
            .map(|p| p[2])
            .fold(f64::NEG_INFINITY, f64::max);
        self.stock.bounds.max[2]
            .max(self.model.bounds().max[2])
            .max(fixtures)
    }

    /// The clearance plane (absolute): over the stock, the model and the workholding.
    pub fn clearance(&self) -> f64 {
        self.highest() + self.params.links.clearance
    }

    /// The rapid plane (absolute).
    pub fn rapid_plane(&self) -> f64 {
        self.highest() + self.params.links.rapid_distance
    }

    /// Depth levels from `top` down to `bottom` by the step down (the first one by the first
    /// step down when set), always ending exactly at `bottom`; with the model's flat floors in
    /// between added (so floors are cut at their height).
    pub fn levels(&self, top: f64, bottom: f64, floors: bool) -> Vec<f64> {
        let p = self.params;
        let mut v = Vec::new();
        if bottom >= top - 1e-9 {
            return vec![bottom];
        }
        let mut z = top;
        let mut first = true;
        loop {
            let step = if first && p.first_step_down > 0.0 {
                p.first_step_down
            } else {
                p.step_down
            };
            first = false;
            z -= step.max(1e-3);
            if z <= bottom + 1e-6 {
                v.push(bottom);
                break;
            }
            v.push(z);
            if v.len() > 10_000 {
                break;
            }
        }
        if floors {
            for f in self.model.floor_levels() {
                let fz = f + p.floor_stock;
                if fz < top - 1e-6 && fz > bottom + 1e-6 && !v.iter().any(|z| (z - fz).abs() < 1e-3)
                {
                    v.push(fz);
                }
            }
            v.sort_by(|a, b| b.total_cmp(a));
        }
        v
    }
}

/// Generates the toolpath.
pub fn generate(input: &Input) -> Result<Toolpath, String> {
    let p = input.params;
    p.check(input.tool)?;
    input.tool.check()?;
    if !p.pattern.allowed(input.tool.kind, input.machine) {
        return Err(format!(
            "{} can't be cut with a {} on a {}",
            p.pattern.label(),
            input.tool.kind.label().to_lowercase(),
            input.machine.label().to_lowercase()
        ));
    }
    if input.model.triangles.is_empty() && p.drive.is_empty() {
        return Err("The job has no bodies to machine".into());
    }
    let tp = match p.pattern {
        Pattern::Face => mill::face(input),
        Pattern::Rough2D => mill::rough(input, false),
        Pattern::Rough3D => mill::rough(input, true),
        Pattern::Adaptive => adaptive::adaptive(input),
        Pattern::Profile2D => mill::profile(input),
        Pattern::Chamfer2D => mill::chamfer(input),
        Pattern::Engrave => mill::engrave(input),
        Pattern::Flatlands => mill::flatlands(input),
        Pattern::Parallel => finish::parallel(input),
        Pattern::ConstantZ => finish::constant_z(input),
        Pattern::Scallop => finish::scallop(input),
        Pattern::Spiral => finish::spiral(input),
        Pattern::Pencil => finish::pencil(input),
        Pattern::ThreadMill => holes::thread_mill(input),
        Pattern::Drill => holes::drill(input),
        Pattern::Bore => holes::bore(input),
        Pattern::Cut => beam::cut(input),
        Pattern::Etch => beam::etch(input),
        Pattern::Fill => beam::fill(input),
    }?;
    // Repeated, then every copy checked.
    let tp = p.repeat.apply(tp);
    let mut tp = if input.machine.is_milling() {
        collision::apply(input, tp)
    } else {
        tp
    };
    // Through the bottom of the stock (a hole's break-through, a profile's cut-off): whatever
    // the part sits on is there.
    if input.machine.is_milling() {
        let lowest = tp
            .steps
            .iter()
            .filter_map(|s| match s {
                Step::Feed { to, .. } | Step::Arc { to, .. } => Some(to[2]),
                Step::Drill(c) => Some(c.bottom),
                _ => None,
            })
            .fold(f64::INFINITY, f64::min);
        let under = input.stock.bounds.min[2] - lowest;
        if under > 1e-3 {
            tp.warnings.push(format!(
                "It cuts {} mm below the stock's bottom: what the part sits on must be clear \
                 there (parallels, a spoil board)",
                crate::tool::fmt_mm(under)
            ));
        }
    }
    if tp.is_empty() {
        return Err(format!(
            "{} found nothing to cut{}",
            p.pattern.label(),
            tp.warnings
                .first()
                .map(|w| format!(": {w}"))
                .unwrap_or_default()
        ));
    }
    Ok(tp)
}

/// The levels a toolpath sections the model at (the studio asks the kernel for exact sections
/// there before generating).
pub fn section_levels(input: &Input) -> Vec<f64> {
    match input.params.pattern {
        Pattern::ConstantZ => input.levels(
            input.top().min(input.model.bounds().max[2]),
            input.bottom(),
            true,
        ),
        _ => Vec::new(),
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use super::*;
    use crate::geom::Aabb;
    use crate::model::{Face, Surface, box_triangles};

    /// A 60 × 40 × 20 block with a 30 × 20 pocket 8 deep in its top (the pocket's floor an exact
    /// face), at the origin.
    pub fn pocket_block() -> Model {
        let mut tris = Vec::new();
        // The block as four walls around the pocket plus the floor slab: boxes that overlap
        // make a valid shadow for the strategies (they union the upward triangles).
        tris.extend(box_triangles(&Aabb::new(
            [0.0, 0.0, 0.0],
            [60.0, 40.0, 12.0],
        )));
        tris.extend(box_triangles(&Aabb::new(
            [0.0, 0.0, 12.0],
            [15.0, 40.0, 20.0],
        )));
        tris.extend(box_triangles(&Aabb::new(
            [45.0, 0.0, 12.0],
            [60.0, 40.0, 20.0],
        )));
        tris.extend(box_triangles(&Aabb::new(
            [15.0, 0.0, 12.0],
            [45.0, 10.0, 20.0],
        )));
        tris.extend(box_triangles(&Aabb::new(
            [15.0, 30.0, 12.0],
            [45.0, 40.0, 20.0],
        )));
        let mut m = Model::from_triangles("block", tris);
        let sq = |x0: f64, y0: f64, x1: f64, y1: f64, z: f64| {
            vec![[x0, y0, z], [x1, y0, z], [x1, y1, z], [x0, y1, z]]
        };
        m.faces.push(Face {
            name: "floor".into(),
            body: 0,
            surface: Surface::Plane {
                normal: [0.0, 0.0, 1.0],
            },
            loops: vec![sq(15.0, 10.0, 45.0, 30.0, 12.0)],
            triangles: vec![],
        });
        m.faces.push(Face {
            name: "top".into(),
            body: 0,
            surface: Surface::Plane {
                normal: [0.0, 0.0, 1.0],
            },
            loops: vec![
                sq(0.0, 0.0, 60.0, 40.0, 20.0),
                sq(15.0, 10.0, 45.0, 30.0, 20.0),
            ],
            triangles: vec![],
        });
        m
    }

    /// How far the toolpath's worst move (rapids, links, leads and cuts alike, sampled every
    /// 0.05 mm) takes the tool into the model (exact drop-cutter), and where: 0 if never.
    pub fn max_gouge(m: &Model, tool: &Tool, tp: &Toolpath) -> (f64, crate::geom::P3) {
        max_gouge_within(m, tool, tp, 0.001)
    }

    /// [`max_gouge`] allowing `tol` into the surface along its normal (a finishing
    /// toolpath's own tolerance): the cutter is shrunk by it all round, so a wall (where a
    /// hair sideways is a long way down) counts by how far into it the cutter goes.
    pub fn max_gouge_within(
        m: &Model,
        tool: &Tool,
        tp: &Toolpath,
        tol: f64,
    ) -> (f64, crate::geom::P3) {
        let idx = crate::dropcutter::TriIndex::new(&m.triangles);
        let mut scratch = Vec::new();
        let mut tool = tool.clone();
        tool.diameter -= 2.0 * tol;
        if tool.kind == ToolKind::BallMill {
            tool.corner_radius = tool.diameter / 2.0;
        }
        let tool = &tool;
        let start = tp
            .steps
            .iter()
            .find_map(|s| match s {
                crate::toolpath::Step::Rapid(p) => Some(*p),
                _ => None,
            })
            .expect("a rapid");
        let mut worst = (0.0, start);
        for seg in tp.motion(start, 5000.0) {
            let n = (crate::geom::dist(seg.from, seg.to) / 0.05).ceil().max(1.0) as usize;
            for i in 0..=n {
                let q = crate::geom::lerp(seg.from, seg.to, i as f64 / n as f64);
                let low = crate::dropcutter::drop_point(
                    tool,
                    &idx,
                    [q[0], q[1]],
                    f64::NEG_INFINITY,
                    &mut scratch,
                );
                if low - tol - q[2] > worst.0 {
                    worst = (low - tol - q[2], q);
                }
            }
        }
        worst
    }

    pub fn stock_for(m: &Model, margin: f64, top: f64) -> Stock {
        let b = m.bounds();
        Stock::block(Aabb::new(
            [b.min[0] - margin, b.min[1] - margin, b.min[2]],
            [b.max[0] + margin, b.max[1] + margin, b.max[2] + top],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::MachineKind;

    #[test]
    fn patterns_by_type_and_tool() {
        let mill = Pattern::available(Type::Edges, ToolKind::EndMill, MachineKind::Mill);
        assert!(mill.contains(&Pattern::Profile2D));
        assert!(!mill.contains(&Pattern::Cut));
        let laser = Pattern::available(Type::Edges, ToolKind::Laser, MachineKind::Laser);
        assert_eq!(laser, vec![Pattern::Cut, Pattern::Etch]);
        assert!(
            Pattern::available(Type::Holes, ToolKind::Drill, MachineKind::Mill)
                .contains(&Pattern::Drill)
        );
        assert!(Pattern::available(Type::Edges, ToolKind::EndMill, MachineKind::Laser).is_empty());
        // Every pattern has a tool and machine it runs with.
        for p in Pattern::ALL {
            assert!(
                ToolKind::ALL
                    .iter()
                    .any(|t| MachineKind::ALL.iter().any(|m| p.allowed(*t, *m))),
                "{p:?}"
            );
        }
    }

    #[test]
    fn levels_end_at_bottom_with_floors() {
        let m = testing::pocket_block();
        let stock = testing::stock_for(&m, 5.0, 1.0);
        let tool = Tool::new(ToolKind::EndMill, 6.0);
        let mut p = Params::new(Pattern::Rough2D, &tool, MachineKind::Mill);
        p.step_down = 5.0;
        p.floor_stock = 0.0;
        let input = Input {
            model: &m,
            stock: &stock,
            tool: &tool,
            machine: MachineKind::Mill,
            params: &p,
            rest: None,
            fixtures: &[],
        };
        let l = input.levels(21.0, 0.0, true);
        assert_eq!(*l.last().unwrap(), 0.0);
        assert!(l.iter().any(|z| (z - 12.0).abs() < 1e-9), "{l:?}");
        assert!(l.windows(2).all(|w| w[0] > w[1]));
    }

    #[test]
    fn repeats_place_copies() {
        let tp = Toolpath {
            steps: vec![
                Step::Spindle {
                    rpm: 1000.0,
                    cw: true,
                },
                Step::Rapid([10.0, 0.0, 5.0]),
                Step::Arc {
                    to: [10.0, 0.0, -1.0],
                    center: [5.0, 0.0],
                    cw: false,
                    feed: 100.0,
                    kind: crate::toolpath::FeedKind::Cut,
                },
            ],
            warnings: vec![],
        };
        let lin = Repeat::Linear {
            count: 3,
            step: [20.0, 0.0],
        }
        .apply(tp.clone());
        assert_eq!(lin.steps.len(), 1 + 2 * 3, "the spindle once");
        assert!(matches!(lin.steps[5], Step::Rapid(p) if (p[0] - 50.0).abs() < 1e-9));
        let circ = Repeat::Circular {
            count: 4,
            center: [0.0, 0.0],
            angle: 90.0,
        }
        .apply(tp);
        // The second copy turned 90°: (10, 0) to (0, 10), its arc's centre with it.
        assert!(
            matches!(circ.steps[3], Step::Rapid(p) if p[0].abs() < 1e-9 && (p[1] - 10.0).abs() < 1e-9)
        );
        assert!(
            matches!(circ.steps[4], Step::Arc { center, .. } if center[0].abs() < 1e-9 && (center[1] - 5.0).abs() < 1e-9)
        );
    }
}
