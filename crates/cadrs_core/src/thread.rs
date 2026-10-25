//! Thread: a modelled screw thread cut into a cylinder (an external thread on a shaft) or into a
//! hole's wall (an internal thread), picked by a circular edge at the end the thread starts from.
//!
//! The thread is the ISO 68-1 basic profile (60°, `H = √3/2 · P`; ASME B1.1's UN profile is the
//! same), its groove swept along a helix and subtracted from the part:
//!
//! - **External**: the groove runs from the major diameter `d` (where it is `7P/8` wide, a little
//!   past the surface so it cuts it cleanly) down to the basic minor diameter `d − 5H/4`, where
//!   it is `P/4` wide.
//! - **Internal**: from the minor diameter `D1 = D − 5H/4` (`3P/4` wide, a little inside the hole)
//!   out to the major diameter `D`, where it is `P/8` wide.
//!
//! The size and pitch come from the hole tables ([`crate::hole::ISO`], [`crate::hole::ANSI`]): a
//! metric "M10" with "M10x1.5", an inch "1/4" with "1/4-20 UNC". The thread starts a pitch
//! outside the picked edge (so it runs out of the end) and runs along the cylinder for its
//! length, or to the cylinder's far end and a pitch past it. Its rebuild is in
//! `rebuild/kernel_ops/thread.rs`.

use serde::{Deserialize, Serialize};

use crate::document::EdgeRef;
use crate::hole::{HoleSpec, HoleStandard, ansi, iso};
use crate::ids::FeatureId;
use crate::solid::Solid;
use cadrs_sketch::Vec3;

/// How far the thread runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThreadEnd {
    /// Its length from the picked edge.
    #[default]
    Blind,
    /// The whole cylinder, running out past its far end.
    Full,
}

impl ThreadEnd {
    pub const ALL: [ThreadEnd; 2] = [ThreadEnd::Blind, ThreadEnd::Full];

    pub fn label(self) -> &'static str {
        match self {
            ThreadEnd::Blind => "Blind",
            ThreadEnd::Full => "Up to end",
        }
    }
}

/// A Thread feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadFeature {
    /// The circular edge at the end of the cylinder or hole the thread starts from.
    #[serde(default)]
    pub edge: Option<EdgeRef>,
    #[serde(default)]
    pub standard: HoleStandard,
    /// "M10" (metric) or "1/4" (inch): a name of the hole tables.
    pub size: String,
    /// The thread: "M10x1.5" (metric) or "1/4-20 UNC" (inch), as [`HoleSpec::pitches`] lists them.
    pub pitch: String,
    #[serde(default)]
    pub end: ThreadEnd,
    /// mm from the picked edge (Blind).
    pub length: f64,
    #[serde(default)]
    pub length_expr: String,
    /// A right-hand thread (else left-hand).
    #[serde(default = "yes")]
    pub clockwise: bool,
    /// A size typed in (the Size dropdown's Custom): `diameter` and `custom_pitch` instead of
    /// the table's.
    #[serde(default)]
    pub custom: bool,
    /// The custom major diameter (mm).
    #[serde(default)]
    pub diameter: f64,
    #[serde(default)]
    pub diameter_expr: String,
    /// The custom pitch (mm).
    #[serde(default)]
    pub custom_pitch: f64,
    #[serde(default)]
    pub custom_pitch_expr: String,
}

fn yes() -> bool {
    true
}

impl Default for ThreadFeature {
    fn default() -> Self {
        Self {
            edge: None,
            standard: HoleStandard::Iso,
            size: "M10".into(),
            pitch: "M10x1.5".into(),
            end: ThreadEnd::Blind,
            length: 10.0,
            length_expr: "10 mm".into(),
            clockwise: true,
            custom: false,
            diameter: 10.0,
            diameter_expr: "10 mm".into(),
            custom_pitch: 1.5,
            custom_pitch_expr: "1.5 mm".into(),
        }
    }
}

impl ThreadFeature {
    pub fn problem(&self) -> Option<&'static str> {
        if self.edge.is_none() {
            return Some("Select a circular edge of a cylinder or a hole");
        }
        if self.major().is_none() || self.pitch_mm().is_none() {
            return Some("Select a thread size and pitch");
        }
        if self.custom && !(self.diameter > 0.0 && self.custom_pitch > 0.0) {
            return Some("The diameter and pitch must be greater than zero");
        }
        if self.end == ThreadEnd::Blind
            && self.length.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)
        {
            return Some("The thread length must be greater than zero");
        }
        None
    }

    pub fn parents(&self) -> Vec<FeatureId> {
        match &self.edge {
            Some(e) => vec![FeatureId(e.edge.faces[0].op), FeatureId(e.edge.faces[1].op)],
            None => Vec::new(),
        }
    }

    /// The nominal (major) diameter in mm.
    pub fn major(&self) -> Option<f64> {
        if self.custom {
            return Some(self.diameter);
        }
        match self.standard {
            HoleStandard::Iso => iso(&self.size).map(|m| m.d),
            HoleStandard::Ansi => ansi(&self.size).map(|a| a.d * 25.4),
        }
    }

    /// The pitch in mm (an inch thread's 25.4 / tpi).
    pub fn pitch_mm(&self) -> Option<f64> {
        if self.custom {
            return Some(self.custom_pitch);
        }
        pitch_mm(self.standard, &self.pitch)
    }

    /// Switches to a typed-in size, starting from the current one's diameter and pitch.
    pub fn set_custom(&mut self) {
        if self.custom {
            return;
        }
        let (d, p) = (self.major(), self.pitch_mm());
        self.custom = true;
        if let (Some(d), Some(p)) = (d, p) {
            self.set_custom_size(d, p);
        }
    }

    fn set_custom_size(&mut self, d: f64, p: f64) {
        let round = |v: f64| (v * 1000.0).round() / 1000.0;
        let mm = |v: f64| format!("{} mm", crate::hole::fmt(v));
        self.diameter = round(d);
        self.diameter_expr = mm(self.diameter);
        self.custom_pitch = round(p);
        self.custom_pitch_expr = mm(self.custom_pitch);
    }

    /// The sizes the Size dropdown offers.
    pub fn sizes(standard: HoleStandard) -> Vec<String> {
        match standard {
            HoleStandard::Iso => crate::hole::ISO.iter().map(|m| m.name.into()).collect(),
            HoleStandard::Ansi => crate::hole::ANSI.iter().map(|a| a.name.into()).collect(),
        }
    }

    /// The threads the Pitch dropdown offers for the size (coarse first).
    pub fn pitches(&self) -> Vec<String> {
        HoleSpec::pitches(self.standard, &self.size)
    }

    /// Sets the standard, keeping the size if it has it, else the first size.
    pub fn set_standard(&mut self, standard: HoleStandard) {
        if standard == self.standard {
            return;
        }
        self.standard = standard;
        let sizes = Self::sizes(standard);
        if !sizes.contains(&self.size) {
            self.size = match standard {
                HoleStandard::Iso => "M10".into(),
                HoleStandard::Ansi => "1/4".into(),
            };
        }
        self.pitch = self.pitches().first().cloned().unwrap_or_default();
    }

    /// Sets the size, with its coarse thread.
    pub fn set_size(&mut self, size: &str) {
        self.custom = false;
        self.size = size.to_string();
        self.pitch = self.pitches().first().cloned().unwrap_or_default();
    }

    /// The standard size that fits a cylinder of `diameter` mm best: an external thread's major
    /// diameter, an internal thread's tap drill (the coarse thread's). Sets it, else (none
    /// matches) a custom size of the cylinder's with the nearest size's coarse pitch. Returns
    /// whether a standard size matches.
    pub fn fit_size(&mut self, diameter: f64, external: bool) -> bool {
        let fit = |d: f64, tap: f64| if external { d } else { tap };
        let best = match self.standard {
            HoleStandard::Iso => crate::hole::ISO
                .iter()
                .map(|m| (m.name, fit(m.d, m.d - m.coarse)))
                .min_by(|a, b| (a.1 - diameter).abs().total_cmp(&(b.1 - diameter).abs())),
            HoleStandard::Ansi => crate::hole::ANSI
                .iter()
                .map(|a| (a.name, fit(a.d, a.tap_unc) * 25.4))
                .min_by(|a, b| (a.1 - diameter).abs().total_cmp(&(b.1 - diameter).abs())),
        };
        let Some((name, _)) = best else {
            return false;
        };
        self.set_size(name);
        if self.matches(diameter, external) {
            return true;
        }
        let p = self.pitch_mm().unwrap_or(1.0);
        // A hole is the custom thread's tap drill, d − P.
        let d = if external { diameter } else { diameter + p };
        self.custom = true;
        self.set_custom_size(d, p);
        false
    }

    /// Whether a cylinder of `diameter` mm is the right size for the thread: an external
    /// thread's major diameter (within 2 %), an internal thread's hole between the minor and
    /// the pitch diameters.
    pub fn matches(&self, diameter: f64, external: bool) -> bool {
        let (Some(d), Some(p)) = (self.major(), self.pitch_mm()) else {
            return false;
        };
        let h = 3f64.sqrt() / 2.0 * p;
        if external {
            (diameter - d).abs() <= 0.02 * d
        } else {
            let minor = d - 1.25 * h;
            let pitch_d = d - 0.75 * h;
            diameter >= minor - 0.1 * p && diameter <= pitch_d
        }
    }

    /// The name the feature list gives a new thread.
    pub fn base_name(external: bool) -> &'static str {
        if external {
            "External thread"
        } else {
            "Internal thread"
        }
    }
}

/// The pitch in mm of a thread designation ("M10x1.5", "1/4-20 UNC").
pub fn pitch_mm(standard: HoleStandard, thread: &str) -> Option<f64> {
    match standard {
        HoleStandard::Iso => thread.split_once('x')?.1.parse().ok(),
        HoleStandard::Ansi => {
            let tpi: f64 = thread.split_once('-')?.1.split(' ').next()?.parse().ok()?;
            Some(25.4 / tpi)
        }
    }
}

/// The groove's half section in the plane through the axis: (radius, along the axis) corners,
/// centred on the helix (along = 0), for a thread of major diameter `d` and pitch `p` cut into
/// a cylinder of radius `r`. `Err` if the cylinder doesn't fit the thread.
pub fn groove(d: f64, p: f64, r: f64, external: bool) -> Result<[[f64; 2]; 4], &'static str> {
    let h = 3f64.sqrt() / 2.0 * p;
    let tan30 = 1.0 / 3f64.sqrt();
    // The groove may not be wider than this (adjacent turns would meet).
    let widest = 0.48 * p;
    if external {
        let root = d / 2.0 - 5.0 * h / 8.0;
        // The root's flat is P/4: half of it each side.
        let half = |rr: f64| p / 8.0 + (rr - root) * tan30;
        let reach = root + (widest - p / 8.0) / tan30;
        let top = (r.max(d / 2.0) + 0.05 * p).min(reach);
        if r <= root + 1e-6 {
            return Err("The cylinder is smaller than the thread's minor diameter");
        }
        if top <= r + 1e-6 {
            return Err("The cylinder is larger than the thread's major diameter");
        }
        let w = half(top);
        Ok([[root, -p / 8.0], [top, -w], [top, w], [root, p / 8.0]])
    } else {
        let root = d / 2.0;
        // The root's flat (at the major diameter) is P/8.
        let half = |rr: f64| p / 16.0 + (root - rr) * tan30;
        let reach = root - (widest - p / 16.0) / tan30;
        let minor = d / 2.0 - 5.0 * h / 8.0;
        let bottom = (r.min(minor) - 0.1 * p).max(reach);
        if r >= root - 1e-6 {
            return Err("The hole is larger than the thread's major diameter");
        }
        if bottom >= r - 1e-6 {
            return Err("The hole is smaller than the thread's minor diameter");
        }
        let w = half(bottom);
        Ok([
            [bottom, -w],
            [root, -p / 16.0],
            [root, p / 16.0],
            [bottom, w],
        ])
    }
}

/// The cylinder a circular edge bounds, as a thread sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cylinder {
    /// The edge's circle centre.
    pub start: Vec3,
    /// Unit, along the axis from the edge into the cylinder.
    pub dir: Vec3,
    pub radius: f64,
    /// Its length along the axis.
    pub height: f64,
    /// A shaft (material inside it), else a hole.
    pub external: bool,
    /// Unit, square to the axis: where its angle 0 is.
    pub x: Vec3,
}

/// The cylindrical face next to a circular edge of `solid` (`edge` indexes `solid.edges`).
pub fn cylinder(solid: &Solid, edge: usize) -> Option<Cylinder> {
    let e = solid.edges.get(edge)?;
    let c = e.circle?;
    for name in &e.name.faces {
        let Some(i) = solid.faces.iter().position(|f| f.name == *name) else {
            continue;
        };
        let f = &solid.faces[i];
        let Some((o, d)) = f.axis else { continue };
        if f.kind != Some(cadrs_kernel::SurfaceKind::Cylinder) {
            continue;
        }
        let d = unit(d);
        // The face's axis must be the circle's.
        if dot(d, unit(c.normal)).abs() < 1.0 - 1e-6 {
            continue;
        }
        let off = sub(c.center, o);
        if norm(sub(off, scale(d, dot(off, d)))) > 1e-4 * (1.0 + c.radius) {
            continue;
        }
        let pts: Vec<Vec3> = f.loops.iter().flatten().copied().collect();
        if pts.is_empty() {
            continue;
        }
        let t = |p: &Vec3| dot(sub(*p, c.center), d);
        let (lo, hi) = pts
            .iter()
            .map(t)
            .fold((f64::MAX, f64::MIN), |(a, b), v| (a.min(v), b.max(v)));
        // The edge is at one end (t = 0): the face runs the other way.
        let (dir, height) = if hi.abs() >= lo.abs() {
            (d, hi)
        } else {
            (scale(d, -1.0), -lo)
        };
        if height < 1e-6 {
            continue;
        }
        let (n, p) = (solid.face_normal(i)?, solid.face_point(i)?);
        let radial = {
            let q = sub(p, c.center);
            sub(q, scale(d, dot(q, d)))
        };
        let x = {
            let q = sub(e.points.first().copied().unwrap_or(p), c.center);
            let q = sub(q, scale(d, dot(q, d)));
            if norm(q) > 1e-9 {
                unit(q)
            } else {
                unit(radial)
            }
        };
        return Some(Cylinder {
            start: c.center,
            dir,
            radius: f.radius.unwrap_or(c.radius),
            height,
            external: dot(n, radial) > 0.0,
            x,
        });
    }
    None
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn scale(a: Vec3, k: f64) -> Vec3 {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn norm(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}
fn unit(a: Vec3) -> Vec3 {
    let l = norm(a);
    if l < 1e-15 { a } else { scale(a, 1.0 / l) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_m10_external_groove_is_the_basic_profile() {
        let g = groove(10.0, 1.5, 5.0, true).unwrap();
        let h = 3f64.sqrt() / 2.0 * 1.5;
        assert!((g[0][0] - (5.0 - 5.0 * h / 8.0)).abs() < 1e-12);
        assert!((g[3][1] - g[0][1] - 1.5 / 4.0).abs() < 1e-12);
        // 7P/8 wide at the major diameter, a little wider past it, under P.
        let w = g[2][1] - g[1][1];
        assert!(w > 7.0 * 1.5 / 8.0 && w < 0.96 * 1.5, "{w}");
        assert!(g[1][0] > 5.0);
    }

    #[test]
    fn an_m10_internal_groove_reaches_the_tap_drill() {
        let g = groove(10.0, 1.5, 8.5 / 2.0, false).unwrap();
        assert!((g[1][0] - 5.0).abs() < 1e-12);
        assert!(g[0][0] < 4.25);
        let w = g[3][1] - g[0][1];
        assert!(w > 0.75 * 1.5 && w < 0.96 * 1.5, "{w}");
    }

    #[test]
    fn a_cylinder_too_big_for_its_thread_is_refused() {
        assert!(groove(10.0, 1.5, 6.0, true).is_err());
        assert!(groove(10.0, 1.5, 5.2, false).is_err());
    }

    #[test]
    fn the_size_fits_the_cylinder() {
        let mut t = ThreadFeature::default();
        assert!(t.fit_size(12.0, true));
        assert_eq!((t.size.as_str(), t.pitch.as_str()), ("M12", "M12x1.75"));
        assert!(t.fit_size(6.8, false));
        assert_eq!(t.size, "M8");
        assert!(!t.fit_size(55.0, true));
        assert!(t.custom && t.matches(55.0, true));
        assert_eq!((t.major(), t.pitch_mm()), (Some(55.0), Some(5.0)));
        assert!(!t.fit_size(60.0, false));
        assert_eq!(t.major(), Some(65.0));
        assert!(t.matches(60.0, false));
        t.set_size("M10");
        assert!(!t.custom);
        t.set_standard(HoleStandard::Ansi);
        assert_eq!(t.pitch, "1/4-20 UNC");
        assert!((t.pitch_mm().unwrap() - 1.27).abs() < 1e-12);
        assert!((t.major().unwrap() - 6.35).abs() < 1e-12);
    }
}
