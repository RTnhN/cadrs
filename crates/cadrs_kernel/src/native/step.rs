//! STEP (ISO 10303-21 files, AP214/AP242 B-rep entities) reading and writing.
//!
//! **Parsing** ([`parse`]): the clear-text encoding into records by `#id`: simple records
//! (`#5=LINE('',#6,#7);`) and complex ones (`#8=(BOUNDED_CURVE() B_SPLINE_CURVE(...) ...);`, a
//! list of partial records), in any order, over any number of lines, with comments and string
//! escapes (`''`, `\X2\…\X0\`, `\X\hh`, `\S\c`).
//!
//! **Geometry.** Read: `CARTESIAN_POINT`, `DIRECTION`, `VECTOR`, `AXIS2_PLACEMENT_3D`;
//! curves `LINE`, `CIRCLE`, `ELLIPSE`, `B_SPLINE_CURVE_WITH_KNOTS` (and the rational complex
//! form, `BEZIER_CURVE`, `UNIFORM_CURVE`, `QUASI_UNIFORM_CURVE`), `POLYLINE`, and the wrappers
//! `SURFACE_CURVE`, `SEAM_CURVE`, `INTERSECTION_CURVE`, `TRIMMED_CURVE` (their 3D curve);
//! surfaces `PLANE`, `CYLINDRICAL_SURFACE`, `CONICAL_SURFACE`, `SPHERICAL_SURFACE`,
//! `TOROIDAL_SURFACE` (and `DEGENERATE_TOROIDAL_SURFACE`), `SURFACE_OF_REVOLUTION`,
//! `SURFACE_OF_LINEAR_EXTRUSION`, `B_SPLINE_SURFACE_WITH_KNOTS` (and rational, Bézier, uniform
//! forms), `RECTANGULAR_TRIMMED_SURFACE` (its basis). Lengths are converted to mm and angles
//! to radians from the representation context's units (`SI_UNIT` with a prefix, or a
//! `CONVERSION_BASED_UNIT` such as INCH or DEGREE).
//!
//! **Topology.** `MANIFOLD_SOLID_BREP`, `BREP_WITH_VOIDS`, `FACETED_BREP` (`POLY_LOOP`s),
//! `SHELL_BASED_SURFACE_MODEL` with `CLOSED_SHELL`/`OPEN_SHELL` (and their oriented forms),
//! `ADVANCED_FACE`/`FACE_SURFACE` (and `ORIENTED_FACE`), `FACE_OUTER_BOUND`/`FACE_BOUND`,
//! `EDGE_LOOP` of `ORIENTED_EDGE`s of `EDGE_CURVE`s between `VERTEX_POINT`s (`VERTEX_LOOP`s are
//! the singular points [`super::xbuild`] puts back). The body is rebuilt by
//! [`super::xbuild::XBuilder`]: our own p-curves, pole edges and seams, senses from the loops.
//!
//! **Products.** Parts are `PRODUCT_DEFINITION`s with geometry (through
//! `PRODUCT_DEFINITION_SHAPE` → `SHAPE_DEFINITION_REPRESENTATION` → the representation's items,
//! following plain `SHAPE_REPRESENTATION_RELATIONSHIP`s and `MAPPED_ITEM`s); assemblies are
//! `NEXT_ASSEMBLY_USAGE_OCCURRENCE`s, each placed by the `ITEM_DEFINED_TRANSFORMATION` of its
//! `CONTEXT_DEPENDENT_SHAPE_REPRESENTATION` (the child's placement maps to the parent's: placement
//! = A(item 2) · A(item 1)⁻¹, either order of the two representations accepted).
//!
//! **Writing** ([`write`]): an AP214 file in mm with one product per part (an
//! `ADVANCED_BREP_SHAPE_REPRESENTATION` of `MANIFOLD_SOLID_BREP`s / `BREP_WITH_VOIDS`, sheets as a
//! `MANIFOLD_SURFACE_SHAPE_REPRESENTATION` of a `SHELL_BASED_SURFACE_MODEL`), and with instances
//! one assembly product holding them through NAUOs. Degenerate edges are left out (readers put
//! them back); each face's `same_sense` compares our outward normal with the written surface's.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::sync::Arc;

use super::curve::Curve;
use super::math::*;
use super::nurbs::{hom, NurbsCurve, NurbsSurface, H};
use super::surface::Surface;
use super::topo::*;
use super::xbuild::{expand_knots, knot_mults, XBuilder, XFace};
use crate::Motion;
use super::tol;

// ---------------------------------------------------------------------------------------------
// Parsing

/// A parameter of a record.
#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    Ref(u64),
    Num(f64),
    Str(String),
    /// `.NAME.` (booleans are `.T.` and `.F.`).
    Enum(String),
    List(Vec<Param>),
    /// A typed value such as `LENGTH_MEASURE(1.5)`.
    Typed(String, Vec<Param>),
    /// `$`
    Unset,
    /// `*`
    Derived,
}

impl Param {
    pub fn as_ref(&self) -> Option<u64> {
        match self {
            Param::Ref(r) => Some(*r),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Param::Num(x) => Some(*x),
            Param::Typed(_, v) => v.first()?.as_f64(),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<&[Param]> {
        match self {
            Param::List(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Param::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Param::Enum(e) if e == "T" => Some(true),
            Param::Enum(e) if e == "F" => Some(false),
            _ => None,
        }
    }
}

/// A record: one partial record (simple) or several (complex), each a keyword with parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub parts: Vec<(String, Vec<Param>)>,
}

impl Record {
    pub fn is_complex(&self) -> bool {
        self.parts.len() > 1
    }
    /// The keyword of a simple record (a complex record's first part).
    pub fn kind(&self) -> &str {
        &self.parts[0].0
    }
    pub fn has(&self, kw: &str) -> bool {
        self.parts.iter().any(|p| p.0 == kw)
    }
    pub fn part(&self, kw: &str) -> Option<&[Param]> {
        self.parts.iter().find(|p| p.0 == kw).map(|p| p.1.as_slice())
    }
    /// The parameters of a simple record.
    pub fn params(&self) -> &[Param] {
        &self.parts[0].1
    }
}

/// A parsed exchange file: the header records and the data records by id.
#[derive(Debug, Default)]
pub struct StepFile {
    pub header: Vec<(String, Vec<Param>)>,
    pub records: HashMap<u64, Record>,
    /// Record ids by keyword (every part of complex records), ascending.
    pub by_kind: HashMap<String, Vec<u64>>,
}

struct Lexer<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Lexer<'a> {
    fn err<T>(&self, what: &str) -> Result<T, String> {
        let line = self.s[..self.i.min(self.s.len())].iter().filter(|&&c| c == b'\n').count() + 1;
        Err(format!("STEP syntax error at line {line}: {what}"))
    }

    fn ws(&mut self) {
        loop {
            while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
                self.i += 1;
            }
            if self.s.get(self.i..).is_some_and(|r| r.starts_with(b"/*")) {
                match self.s[self.i + 2..].windows(2).position(|w| w == b"*/") {
                    Some(k) => self.i += k + 4,
                    None => self.i = self.s.len(),
                }
            } else {
                break;
            }
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.ws();
        self.s.get(self.i).copied()
    }

    fn eat(&mut self, c: u8) -> Result<(), String> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            self.err(&format!("expected '{}'", c as char))
        }
    }

    fn keyword(&mut self) -> Result<String, String> {
        self.ws();
        let st = self.i;
        while self.i < self.s.len() && (self.s[self.i].is_ascii_alphanumeric() || self.s[self.i] == b'_' || self.s[self.i] == b'-' || self.s[self.i] == b'!') {
            self.i += 1;
        }
        if st == self.i {
            return self.err("expected a keyword");
        }
        Ok(String::from_utf8_lossy(&self.s[st..self.i]).to_ascii_uppercase())
    }

    fn uint(&mut self) -> Result<u64, String> {
        let st = self.i;
        while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
            self.i += 1;
        }
        std::str::from_utf8(&self.s[st..self.i]).ok().and_then(|t| t.parse().ok()).map_or_else(|| self.err("expected a number"), Ok)
    }

    fn string(&mut self) -> Result<String, String> {
        // At the opening quote.
        self.i += 1;
        let mut raw: Vec<u8> = Vec::new();
        loop {
            match self.s.get(self.i) {
                None => return self.err("unterminated string"),
                Some(b'\'') => {
                    if self.s.get(self.i + 1) == Some(&b'\'') {
                        raw.push(b'\'');
                        self.i += 2;
                    } else {
                        self.i += 1;
                        break;
                    }
                }
                Some(&c) => {
                    if c != b'\n' && c != b'\r' {
                        raw.push(c);
                    }
                    self.i += 1;
                }
            }
        }
        Ok(decode_string(&raw))
    }

    fn number(&mut self) -> Result<f64, String> {
        let st = self.i;
        while self.i < self.s.len() && matches!(self.s[self.i], b'0'..=b'9' | b'+' | b'-' | b'.' | b'e' | b'E') {
            self.i += 1;
        }
        let t = std::str::from_utf8(&self.s[st..self.i]).unwrap_or("");
        parse_real(t).map_or_else(|| self.err(&format!("bad number '{t}'")), Ok)
    }

    fn param(&mut self) -> Result<Param, String> {
        match self.peek() {
            Some(b'#') => {
                self.i += 1;
                Ok(Param::Ref(self.uint()?))
            }
            Some(b'\'') => Ok(Param::Str(self.string()?)),
            Some(b'"') => {
                self.i += 1;
                let st = self.i;
                while self.i < self.s.len() && self.s[self.i] != b'"' {
                    self.i += 1;
                }
                if self.i == self.s.len() {
                    return self.err("unterminated binary");
                }
                let t = String::from_utf8_lossy(&self.s[st..self.i]).into_owned();
                self.i += 1;
                Ok(Param::Str(t))
            }
            Some(b'.') if self.s.get(self.i + 1).is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_') => {
                self.i += 1;
                let st = self.i;
                while self.i < self.s.len() && self.s[self.i] != b'.' {
                    self.i += 1;
                }
                if self.i == self.s.len() {
                    return self.err("unterminated enumeration");
                }
                let t = String::from_utf8_lossy(&self.s[st..self.i]).trim().to_ascii_uppercase();
                self.i += 1;
                Ok(Param::Enum(t))
            }
            Some(b'$') => {
                self.i += 1;
                Ok(Param::Unset)
            }
            Some(b'*') => {
                self.i += 1;
                Ok(Param::Derived)
            }
            Some(b'(') => Ok(Param::List(self.list()?)),
            Some(c) if c.is_ascii_digit() || c == b'-' || c == b'+' || c == b'.' => Ok(Param::Num(self.number()?)),
            Some(c) if c.is_ascii_alphabetic() => {
                let k = self.keyword()?;
                let v = self.list()?;
                Ok(Param::Typed(k, v))
            }
            _ => self.err("expected a parameter"),
        }
    }

    /// `( p, p, ... )`
    fn list(&mut self) -> Result<Vec<Param>, String> {
        self.eat(b'(')?;
        let mut out = Vec::new();
        if self.peek() == Some(b')') {
            self.i += 1;
            return Ok(out);
        }
        loop {
            out.push(self.param()?);
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b')') => {
                    self.i += 1;
                    return Ok(out);
                }
                _ => return self.err("expected ',' or ')'"),
            }
        }
    }
}

/// A real in the file's notation (`1.`, `1.E-07`, `-.5`).
fn parse_real(t: &str) -> Option<f64> {
    if let Ok(x) = t.parse::<f64>() {
        return Some(x);
    }
    let mut s = t.replace(".E", ".0E").replace(".e", ".0e");
    if s.ends_with('.') {
        s.push('0');
    }
    if s.starts_with('.') || s.starts_with("-.") || s.starts_with("+.") {
        s = s.replacen('.', "0.", 1);
    }
    s.parse().ok()
}

/// Decodes a string's escapes (`\\`, `\X2\hhhh…\X0\`, `\X4\hhhhhhhh…\X0\`, `\X\hh`, `\S\c`,
/// `\Px\` page markers dropped).
fn decode_string(raw: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    let hex = |b: &[u8]| -> Option<u32> { u32::from_str_radix(std::str::from_utf8(b).ok()?, 16).ok() };
    while i < raw.len() {
        if raw[i] == b'\\' {
            let rest = &raw[i..];
            if rest.starts_with(b"\\\\") {
                out.push('\\');
                i += 2;
                continue;
            }
            if rest.starts_with(b"\\X2\\") || rest.starts_with(b"\\X4\\") {
                let w = if rest[2] == b'2' { 4 } else { 8 };
                let mut j = i + 4;
                let mut units: Vec<u16> = Vec::new();
                while j + w <= raw.len() && !raw[j..].starts_with(b"\\X0\\") {
                    match hex(&raw[j..j + w]) {
                        // UTF-16 (with surrogate pairs) for \X2\, code points for \X4\.
                        Some(u) if w == 4 => units.push(u as u16),
                        Some(u) => out.extend(char::from_u32(u)),
                        None => {}
                    }
                    j += w;
                }
                out.push_str(&String::from_utf16_lossy(&units));
                i = (j + 4).min(raw.len());
                continue;
            }
            if rest.starts_with(b"\\X\\") && rest.len() >= 5 {
                if let Some(c) = hex(&rest[3..5]).and_then(char::from_u32) {
                    out.push(c);
                }
                i += 5;
                continue;
            }
            if rest.starts_with(b"\\S\\") && rest.len() >= 4 {
                out.push(char::from_u32(rest[3] as u32 + 128).unwrap_or('?'));
                i += 4;
                continue;
            }
            if rest.len() >= 4 && rest[1] == b'P' && rest[3] == b'\\' {
                i += 4;
                continue;
            }
        }
        // Plain bytes (UTF-8 passes through, as some writers emit it).
        let st = i;
        while i < raw.len() && raw[i] != b'\\' {
            i += 1;
        }
        if st == i {
            out.push('\\');
            i += 1;
        } else {
            out.push_str(&String::from_utf8_lossy(&raw[st..i]));
        }
    }
    out
}

/// Encodes a string for the file: quotes doubled, backslashes doubled, non-ASCII as `\X2\`.
fn encode_string(s: &str) -> String {
    let mut out = String::from("'");
    let mut wide = String::new();
    let flush = |out: &mut String, wide: &mut String| {
        if !wide.is_empty() {
            out.push_str("\\X2\\");
            out.push_str(wide);
            out.push_str("\\X0\\");
            wide.clear();
        }
    };
    for c in s.chars() {
        if c.is_ascii() && !c.is_ascii_control() {
            flush(&mut out, &mut wide);
            match c {
                '\'' => out.push_str("''"),
                '\\' => out.push_str("\\\\"),
                _ => out.push(c),
            }
        } else if (c as u32) < 0x10000 {
            let _ = write!(wide, "{:04X}", c as u32);
        } else {
            let mut buf = [0u16; 2];
            for u in c.encode_utf16(&mut buf) {
                let _ = write!(wide, "{:04X}", u);
            }
        }
    }
    flush(&mut out, &mut wide);
    out.push('\'');
    out
}

/// Parses a STEP file.
pub fn parse(bytes: &[u8]) -> Result<StepFile, String> {
    let mut lx = Lexer { s: bytes, i: 0 };
    let mut f = StepFile::default();
    let start = lx.peek();
    if !lx.s[lx.i..].starts_with(b"ISO-10303-21") {
        let _ = start;
        return Err("not a STEP file (no ISO-10303-21 header)".into());
    }
    lx.i += "ISO-10303-21".len();
    lx.eat(b';')?;
    loop {
        match lx.peek() {
            None => break,
            Some(b'#') => {
                lx.i += 1;
                let id = lx.uint()?;
                lx.eat(b'=')?;
                let rec = if lx.peek() == Some(b'(') {
                    lx.i += 1;
                    let mut parts = Vec::new();
                    while lx.peek() != Some(b')') {
                        let k = lx.keyword()?;
                        let v = lx.list()?;
                        parts.push((k, v));
                    }
                    lx.i += 1;
                    Record { parts }
                } else {
                    let k = lx.keyword()?;
                    let v = lx.list()?;
                    Record { parts: vec![(k, v)] }
                };
                lx.eat(b';')?;
                for (k, _) in &rec.parts {
                    f.by_kind.entry(k.clone()).or_default().push(id);
                }
                f.records.insert(id, rec);
            }
            Some(_) => {
                let k = lx.keyword()?;
                match k.as_str() {
                    "END-ISO-10303-21" => break,
                    "HEADER" | "DATA" | "ENDSEC" => {
                        // DATA may carry a parameter list in later editions.
                        if lx.peek() == Some(b'(') {
                            lx.list()?;
                        }
                        lx.eat(b';')?;
                    }
                    _ => {
                        let v = lx.list()?;
                        lx.eat(b';')?;
                        f.header.push((k, v));
                    }
                }
            }
        }
    }
    for v in f.by_kind.values_mut() {
        v.sort_unstable();
    }
    Ok(f)
}

// ---------------------------------------------------------------------------------------------
// Reading geometry

type R<T> = Result<T, String>;

/// Reads geometry and topology of one representation context (its units).
struct Geo<'a> {
    f: &'a StepFile,
    /// File length unit → mm, and plane angle unit → radians.
    ls: f64,
    angle: f64,
}

fn at(ps: &[Param], i: usize) -> R<&Param> {
    ps.get(i).ok_or_else(|| "too few parameters".to_string())
}

fn num(ps: &[Param], i: usize) -> R<f64> {
    at(ps, i)?.as_f64().ok_or_else(|| format!("parameter {i} is not a number"))
}

fn rf(ps: &[Param], i: usize) -> R<u64> {
    at(ps, i)?.as_ref().ok_or_else(|| format!("parameter {i} is not a reference"))
}

fn missing(id: u64) -> String {
    format!("#{id} is missing")
}

impl<'a> Geo<'a> {
    fn rec(&self, id: u64) -> R<&'a Record> {
        self.f.records.get(&id).ok_or_else(|| missing(id))
    }

    fn num(&self, ps: &[Param], i: usize) -> R<f64> {
        num(ps, i)
    }

    fn rf(&self, ps: &[Param], i: usize) -> R<u64> {
        rf(ps, i)
    }

    fn coords(&self, ps: &[Param], i: usize) -> R<Vec<f64>> {
        let l = at(ps, i)?.as_list().ok_or("expected a list of coordinates")?;
        l.iter().map(|x| x.as_f64().ok_or_else(|| "a coordinate is not a number".to_string())).collect()
    }

    fn point(&self, id: u64) -> R<P3> {
        let r = self.rec(id)?;
        match r.kind() {
            "CARTESIAN_POINT" => {
                let c = self.coords(r.params(), 1)?;
                let g = |k: usize| c.get(k).copied().unwrap_or(0.0) * self.ls;
                Ok(P3::new(g(0), g(1), g(2)))
            }
            "VERTEX_POINT" => self.point(self.rf(r.params(), 1)?),
            k => Err(format!("#{id}: expected a point, found {k}")),
        }
    }

    fn direction(&self, id: u64) -> R<V3> {
        let r = self.rec(id)?;
        if r.kind() != "DIRECTION" {
            return Err(format!("#{id}: expected a DIRECTION, found {}", r.kind()));
        }
        let c = self.coords(r.params(), 1)?;
        let v = V3::new(c.first().copied().unwrap_or(0.0), c.get(1).copied().unwrap_or(0.0), c.get(2).copied().unwrap_or(0.0));
        let n = v.norm();
        if n < tol::TINY {
            return Err(format!("#{id}: a zero direction"));
        }
        Ok(v / n)
    }

    /// `AXIS2_PLACEMENT_3D(name, location, axis, ref_direction)` as a right-handed frame.
    fn axis2(&self, id: u64) -> R<Frame> {
        let r = self.rec(id)?;
        let ps = r.params();
        match r.kind() {
            "AXIS2_PLACEMENT_3D" => {
                let o = self.point(self.rf(ps, 1)?)?;
                let z = match at(ps, 2)? {
                    Param::Ref(d) => self.direction(*d)?,
                    _ => V3::z(),
                };
                let x = match ps.get(3) {
                    Some(Param::Ref(d)) => self.direction(*d)?,
                    _ => any_perpendicular(&z),
                };
                Ok(Frame::from_xz(o, x, z))
            }
            "AXIS2_PLACEMENT_2D" => {
                let o = self.point(self.rf(ps, 1)?)?;
                let x = match ps.get(2) {
                    Some(Param::Ref(d)) => self.direction(*d)?,
                    _ => V3::x(),
                };
                Ok(Frame::from_xz(o, x, V3::z()))
            }
            k => Err(format!("#{id}: expected a placement, found {k}")),
        }
    }

    fn curve(&self, id: u64) -> R<Curve> {
        let r = self.rec(id)?;
        if r.is_complex() {
            if r.has("B_SPLINE_CURVE") {
                return Ok(Curve::Nurbs(Arc::new(self.bspline_curve(r)?)));
            }
            return Err(format!("#{id}: unsupported complex curve"));
        }
        let ps = r.params();
        Ok(match r.kind() {
            "LINE" => {
                let o = self.point(self.rf(ps, 1)?)?;
                let v = self.rec(self.rf(ps, 2)?)?;
                let d = self.direction(self.rf(v.params(), 1)?)?;
                Curve::Line { o, d }
            }
            "CIRCLE" => {
                let f = self.axis2(self.rf(ps, 1)?)?;
                Curve::Circle { c: f.o, x: f.x, y: f.y, r: self.num(ps, 2)? * self.ls }
            }
            "ELLIPSE" => {
                let f = self.axis2(self.rf(ps, 1)?)?;
                Curve::Ellipse { c: f.o, x: f.x, y: f.y, a: self.num(ps, 2)? * self.ls, b: self.num(ps, 3)? * self.ls }
            }
            "B_SPLINE_CURVE_WITH_KNOTS" | "BEZIER_CURVE" | "UNIFORM_CURVE" | "QUASI_UNIFORM_CURVE" => Curve::Nurbs(Arc::new(self.bspline_curve(r)?)),
            "POLYLINE" => {
                let pts: Vec<P3> = at(ps, 1)?.as_list().ok_or("POLYLINE without points")?.iter().filter_map(|p| p.as_ref()).map(|p| self.point(p)).collect::<R<_>>()?;
                if pts.len() < 2 {
                    return Err(format!("#{id}: a POLYLINE needs two points"));
                }
                let mut knots = vec![0.0];
                knots.extend((0..pts.len()).map(|i| i as f64));
                knots.push((pts.len() - 1) as f64);
                Curve::Nurbs(Arc::new(NurbsCurve { degree: 1, knots, cps: pts.iter().map(|p| hom(p, 1.0)).collect() }))
            }
            "SURFACE_CURVE" | "SEAM_CURVE" | "INTERSECTION_CURVE" | "BOUNDED_SURFACE_CURVE" => self.curve(self.rf(ps, 1)?)?,
            "TRIMMED_CURVE" => self.curve(self.rf(ps, 1)?)?,
            k => return Err(format!("#{id}: unsupported curve {k}")),
        })
    }

    /// A hyperbola or parabola (unbounded curves) as the exact quadratic arc between two of its
    /// points (an edge's vertices); None for other curves.
    fn conic_arc(&self, id: u64, p0: &P3, p1: &P3) -> R<Option<Curve>> {
        let mut r = self.rec(id)?;
        // Through SURFACE_CURVE and friends to the 3D curve.
        while matches!(r.kind(), "SURFACE_CURVE" | "SEAM_CURVE" | "INTERSECTION_CURVE" | "BOUNDED_SURFACE_CURVE" | "TRIMMED_CURVE") {
            r = self.rec(self.rf(r.params(), 1)?)?;
        }
        let ps = r.params();
        type F = Box<dyn Fn(f64) -> V3>;
        let (t0, t1, w, pt, d1): (f64, f64, f64, F, F) = match r.kind() {
            "HYPERBOLA" => {
                // c + a cosh t·x + b sinh t·y
                let f = self.axis2(self.rf(ps, 1)?)?;
                let (a, b) = (self.num(ps, 2)? * self.ls, self.num(ps, 3)? * self.ls);
                let t = |p: &P3| ((p - f.o).dot(&f.y) / b).asinh();
                let (t0, t1) = (t(p0), t(p1));
                (
                    t0.min(t1),
                    t0.max(t1),
                    ((t1 - t0) / 2.0).cosh(),
                    Box::new(move |t: f64| f.o.coords + f.x * (a * t.cosh()) + f.y * (b * t.sinh())),
                    Box::new(move |t: f64| f.x * (a * t.sinh()) + f.y * (b * t.cosh())),
                )
            }
            "PARABOLA" => {
                // c + a(t²·x + 2t·y)
                let f = self.axis2(self.rf(ps, 1)?)?;
                let a = self.num(ps, 2)? * self.ls;
                let t = |p: &P3| (p - f.o).dot(&f.y) / (2.0 * a);
                let (t0, t1) = (t(p0), t(p1));
                (
                    t0.min(t1),
                    t0.max(t1),
                    1.0,
                    Box::new(move |t: f64| f.o.coords + (f.x * (t * t) + f.y * (2.0 * t)) * a),
                    Box::new(move |t: f64| (f.x * (2.0 * t) + f.y * 2.0) * a),
                )
            }
            _ => return Ok(None),
        };
        if t1 - t0 < 1e-12 {
            return Ok(None);
        }
        // The middle control point: where the end tangents meet (the weight makes it exact).
        let (a, b) = (P3::from(pt(t0)), P3::from(pt(t1)));
        let (da, db) = (d1(t0), d1(t1));
        let n = da.cross(&db);
        let m = if n.norm() < 1e-14 {
            P3::from((a.coords + b.coords) / 2.0)
        } else {
            let s = (b - a).cross(&db).dot(&n) / n.norm_squared();
            a + da * s
        };
        let c = NurbsCurve { degree: 2, knots: vec![t0, t0, t0, t1, t1, t1], cps: vec![hom(&a, 1.0), hom(&m, w), hom(&b, 1.0)] };
        Ok(Some(Curve::Nurbs(Arc::new(c))))
    }

    /// Control points (with weights) from a list of point references.
    fn cps(&self, l: &Param, w: Option<&[Param]>) -> R<Vec<H>> {
        let l = l.as_list().ok_or("control points are not a list")?;
        let mut out = Vec::with_capacity(l.len());
        for (i, p) in l.iter().enumerate() {
            let q = self.point(p.as_ref().ok_or("a control point is not a reference")?)?;
            let wt = match w {
                Some(w) => w.get(i).and_then(|x| x.as_f64()).ok_or("too few weights")?,
                None => 1.0,
            };
            out.push(hom(&q, wt));
        }
        Ok(out)
    }

    /// Knots from multiplicities and values, or implied by the curve's form.
    fn knots(&self, kind: &str, deg: usize, n: usize, mults: Option<&Param>, vals: Option<&Param>) -> R<Vec<f64>> {
        match kind {
            "WITH_KNOTS" => {
                let m: Vec<usize> = mults.and_then(|m| m.as_list()).ok_or("no knot multiplicities")?.iter().map(|x| x.as_f64().unwrap_or(0.0) as usize).collect();
                let v: Vec<f64> = vals.and_then(|m| m.as_list()).ok_or("no knots")?.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect();
                let k = expand_knots(&v, &m);
                if k.len() != n + deg + 1 {
                    return Err(format!("{} knots for {n} control points of degree {deg}", k.len()));
                }
                Ok(k)
            }
            "UNIFORM" => Ok((0..n + deg + 1).map(|i| i as f64 - deg as f64).collect()),
            "QUASI_UNIFORM" => {
                let spans = n - deg;
                let mut k = vec![0.0; deg + 1];
                k.extend((1..spans).map(|i| i as f64));
                k.extend(vec![spans as f64; deg + 1]);
                Ok(k)
            }
            _ => {
                // Bézier: piecewise, a full-multiplicity knot between pieces.
                let pieces = ((n - 1) / deg.max(1)).max(1);
                let mut k = vec![0.0; deg + 1];
                for i in 1..pieces {
                    k.extend(vec![i as f64; deg]);
                }
                k.extend(vec![pieces as f64; deg + 1]);
                if k.len() != n + deg + 1 {
                    return Err("a Bézier curve with a bad number of control points".into());
                }
                Ok(k)
            }
        }
    }

    fn form_of(r: &Record, base: &str) -> &'static str {
        let has = |s: &str| r.has(&format!("{base}{s}"));
        if has("_WITH_KNOTS") {
            "WITH_KNOTS"
        } else if r.has(&base.replace("B_SPLINE", "UNIFORM")) {
            "UNIFORM"
        } else if r.has(&base.replace("B_SPLINE", "QUASI_UNIFORM")) {
            "QUASI_UNIFORM"
        } else {
            "BEZIER"
        }
    }

    fn bspline_curve(&self, r: &Record) -> R<NurbsCurve> {
        // Simple: (name, degree, points, form, closed, self_intersect, [mults, knots, spec]);
        // complex: B_SPLINE_CURVE(degree, points, ...) plus the knots part and the weights.
        let (deg, pts, mults, vals, weights, form) = if r.is_complex() {
            let b = r.part("B_SPLINE_CURVE").ok_or("no B_SPLINE_CURVE part")?;
            let k = r.part("B_SPLINE_CURVE_WITH_KNOTS");
            let w = r.part("RATIONAL_B_SPLINE_CURVE").and_then(|w| w.first()).and_then(|w| w.as_list());
            (self.num(b, 0)? as usize, at(b, 1)?, k.and_then(|k| k.first()), k.and_then(|k| k.get(1)), w, Self::form_of(r, "B_SPLINE_CURVE"))
        } else {
            let ps = r.params();
            let form = match r.kind() {
                "B_SPLINE_CURVE_WITH_KNOTS" => "WITH_KNOTS",
                "UNIFORM_CURVE" => "UNIFORM",
                "QUASI_UNIFORM_CURVE" => "QUASI_UNIFORM",
                _ => "BEZIER",
            };
            (self.num(ps, 1)? as usize, at(ps, 2)?, ps.get(6), ps.get(7), None, form)
        };
        let cps = self.cps(pts, weights)?;
        if cps.len() < deg + 1 || deg == 0 {
            return Err("a B-spline curve with too few control points".into());
        }
        let knots = self.knots(form, deg, cps.len(), mults, vals)?;
        NurbsCurve::new(deg, knots, cps).map_err(|e| e.message())
    }

    fn bspline_surface(&self, r: &Record) -> R<NurbsSurface> {
        let (du, dv, pts, k, weights, form) = if r.is_complex() {
            let b = r.part("B_SPLINE_SURFACE").ok_or("no B_SPLINE_SURFACE part")?;
            let k = r.part("B_SPLINE_SURFACE_WITH_KNOTS").map(|k| k.to_vec());
            let w = r.part("RATIONAL_B_SPLINE_SURFACE").and_then(|w| w.first()).cloned();
            (self.num(b, 0)? as usize, self.num(b, 1)? as usize, at(b, 2)?.clone(), k, w, Self::form_of(r, "B_SPLINE_SURFACE"))
        } else {
            let ps = r.params();
            let form = match r.kind() {
                "B_SPLINE_SURFACE_WITH_KNOTS" => "WITH_KNOTS",
                "UNIFORM_SURFACE" => "UNIFORM",
                "QUASI_UNIFORM_SURFACE" => "QUASI_UNIFORM",
                _ => "BEZIER",
            };
            (self.num(ps, 1)? as usize, self.num(ps, 2)? as usize, at(ps, 3)?.clone(), ps.get(8..12).map(|k| k.to_vec()), None, form)
        };
        let rows = pts.as_list().ok_or("surface control points are not a list")?;
        let wrows = weights.as_ref().and_then(|w| w.as_list());
        let mut cps = Vec::with_capacity(rows.len());
        for (i, row) in rows.iter().enumerate() {
            let w = wrows.and_then(|w| w.get(i)).and_then(|w| w.as_list());
            cps.push(self.cps(row, w)?);
        }
        let (nu, nv) = (cps.len(), cps.first().map_or(0, |r| r.len()));
        if nu < du + 1 || nv < dv + 1 || du == 0 || dv == 0 || cps.iter().any(|r| r.len() != nv) {
            return Err("a B-spline surface with a bad control net".into());
        }
        let (ku, kv) = match &k {
            Some(k) if form == "WITH_KNOTS" => (self.knots(form, du, nu, k.first(), k.get(2))?, self.knots(form, dv, nv, k.get(1), k.get(3))?),
            _ => (self.knots(form, du, nu, None, None)?, self.knots(form, dv, nv, None, None)?),
        };
        NurbsSurface::new(du, dv, ku, kv, cps).map_err(|e| e.message())
    }

    fn surface(&self, id: u64) -> R<Surface> {
        let r = self.rec(id)?;
        if r.is_complex() {
            if r.has("B_SPLINE_SURFACE") {
                return Ok(Surface::Nurbs(Arc::new(self.bspline_surface(r)?)));
            }
            return Err(format!("#{id}: unsupported complex surface"));
        }
        let ps = r.params();
        Ok(match r.kind() {
            "PLANE" => Surface::Plane { f: self.axis2(self.rf(ps, 1)?)? },
            "CYLINDRICAL_SURFACE" => Surface::Cylinder { f: self.axis2(self.rf(ps, 1)?)?, r: self.num(ps, 2)? * self.ls },
            "CONICAL_SURFACE" => {
                let f = self.axis2(self.rf(ps, 1)?)?;
                let a = self.num(ps, 3)? * self.angle;
                Surface::Cone { f, r: self.num(ps, 2)? * self.ls, a }
            }
            "SPHERICAL_SURFACE" => Surface::Sphere { f: self.axis2(self.rf(ps, 1)?)?, r: self.num(ps, 2)? * self.ls },
            "TOROIDAL_SURFACE" | "DEGENERATE_TOROIDAL_SURFACE" => Surface::Torus { f: self.axis2(self.rf(ps, 1)?)?, rmaj: self.num(ps, 2)? * self.ls, rmin: self.num(ps, 3)? * self.ls },
            "SURFACE_OF_REVOLUTION" => {
                let m = self.curve(self.rf(ps, 1)?)?;
                let ax = self.rec(self.rf(ps, 2)?)?;
                let o = self.point(self.rf(ax.params(), 1)?)?;
                let z = match ax.params().get(2) {
                    Some(Param::Ref(d)) => self.direction(*d)?,
                    _ => V3::z(),
                };
                // x towards the meridian (its middle, off the axis).
                let mid = match m.nurbs_range() {
                    Some((a, b)) => m.point((a + b) / 2.0),
                    None => m.point(0.0),
                };
                let w = mid - o;
                let xh = w - z * w.dot(&z);
                Surface::Revolution { f: Frame::from_xz(o, xh, z), m }
            }
            "SURFACE_OF_LINEAR_EXTRUSION" => {
                let c = self.curve(self.rf(ps, 1)?)?;
                let v = self.rec(self.rf(ps, 2)?)?;
                let d = self.direction(self.rf(v.params(), 1)?)? * self.num(v.params(), 2)? * self.ls;
                Surface::Extrusion { c, d }
            }
            "B_SPLINE_SURFACE_WITH_KNOTS" | "BEZIER_SURFACE" | "UNIFORM_SURFACE" | "QUASI_UNIFORM_SURFACE" => Surface::Nurbs(Arc::new(self.bspline_surface(r)?)),
            "RECTANGULAR_TRIMMED_SURFACE" | "CURVE_BOUNDED_SURFACE" => self.surface(self.rf(ps, 1)?)?,
            k => return Err(format!("#{id}: unsupported surface {k}")),
        })
    }
}

/// A piece of a wire read from a file: a curve from `a` to `b` (along it), or all of a closed
/// curve.
pub struct WireSeg {
    pub c: Curve,
    pub a: P3,
    pub b: P3,
    pub closed: bool,
}

impl Geo<'_> {
    /// The point at a parameter of a basis curve record in the file's terms (a LINE's runs
    /// along its VECTOR's magnitude, a CIRCLE's and ELLIPSE's is an angle).
    fn at_param(&self, basis: u64, c: &Curve, u: f64) -> R<P3> {
        let r = self.rec(basis)?;
        Ok(match (r.kind(), c) {
            ("LINE", Curve::Line { o, d }) => {
                let v = self.rec(self.rf(r.params(), 2)?)?;
                o + d * (u * self.num(v.params(), 2)? * self.ls)
            }
            ("CIRCLE" | "ELLIPSE", _) => c.point(u * self.angle),
            _ => c.point(u),
        })
    }

    /// The curves and points of a wireframe item (`GEOMETRIC_CURVE_SET`, `GEOMETRIC_SET`, and
    /// inside them trimmed curves, composite curves, bounded curves, circles, ellipses and
    /// points). Unbounded lines are skipped.
    fn wire_items(&self, id: u64, segs: &mut Vec<WireSeg>, pts: &mut Vec<P3>, depth: usize) -> R<()> {
        if depth > 8 {
            return Ok(());
        }
        let r = self.rec(id)?;
        let ps = r.params();
        let ends = |c: &Curve, a: P3, b: P3| {
            let closed = (a - b).norm() <= tol::REL * (1.0 + a.coords.norm()) && c.is_periodic();
            WireSeg { c: c.clone(), a, b, closed }
        };
        if r.is_complex() {
            if r.has("B_SPLINE_CURVE") {
                let c = self.curve(id)?;
                let (t0, t1) = c.nurbs_range().unwrap_or((0.0, 1.0));
                let (a, b) = (c.point(t0), c.point(t1));
                segs.push(WireSeg { closed: (a - b).norm() <= tol::REL * (1.0 + a.coords.norm()), c, a, b });
            }
            return Ok(());
        }
        match r.kind() {
            "GEOMETRIC_CURVE_SET" | "GEOMETRIC_SET" => {
                for e in at(ps, 1)?.as_list().ok_or("a set's elements are not a list")? {
                    self.wire_items(e.as_ref().ok_or("a set's element is not a reference")?, segs, pts, depth + 1)?;
                }
            }
            "CARTESIAN_POINT" => pts.push(self.point(id)?),
            "TRIMMED_CURVE" => {
                let basis = self.rf(ps, 1)?;
                let c = self.curve(basis)?;
                let trim = |i: usize| -> R<P3> {
                    let l = at(ps, i)?.as_list().ok_or("a trim is not a list")?;
                    if let Some(p) = l.iter().find_map(|x| x.as_ref()) {
                        return self.point(p);
                    }
                    let u = l.iter().find_map(|x| x.as_f64()).ok_or("a trim has neither a point nor a parameter")?;
                    self.at_param(basis, &c, u)
                };
                let (a, b) = (trim(2)?, trim(3)?);
                let along = ps.get(4).and_then(|p| p.as_bool()).unwrap_or(true);
                let (a, b) = if along { (a, b) } else { (b, a) };
                let curve = match self.conic_arc(basis, &a, &b)? {
                    Some(arc) => arc,
                    None => c,
                };
                segs.push(ends(&curve, a, b));
            }
            "COMPOSITE_CURVE" => {
                for sg in at(ps, 1)?.as_list().ok_or("segments are not a list")? {
                    let sr = self.rec(sg.as_ref().ok_or("a segment is not a reference")?)?;
                    self.wire_items(self.rf(sr.params(), 2)?, segs, pts, depth + 1)?;
                }
            }
            "CIRCLE" | "ELLIPSE" => {
                let c = self.curve(id)?;
                let a = c.point(0.0);
                segs.push(WireSeg { c, a, b: a, closed: true });
            }
            "LINE" => {}
            _ => {
                let c = self.curve(id)?;
                let (t0, t1) = c.nurbs_range().ok_or_else(|| format!("#{id}: an unbounded curve in a curve set"))?;
                let (a, b) = (c.point(t0), c.point(t1));
                segs.push(WireSeg { closed: (a - b).norm() <= tol::REL * (1.0 + a.coords.norm()), c, a, b });
            }
        }
        Ok(())
    }
}

/// A wire body from a file's curves and points: the curves' ends welded within `tol` into
/// vertices, the edges chained into wires ([`Body::chain_wires`]), the points as acorns.
pub fn wire_body(segs: &[WireSeg], pts: &[P3], tol: f64) -> Body {
    let mut b = Body::default();
    let vertex = |b: &mut Body, p: P3| match b.verts.iter().position(|v| (v.p - p).norm() <= tol) {
        Some(i) => i,
        None => b.add_vertex(p),
    };
    for s in segs {
        let va = vertex(&mut b, s.a);
        let vb = if s.closed { va } else { vertex(&mut b, s.b) };
        let p0 = b.verts[va].p;
        let p1 = b.verts[vb].p;
        if let Some((t0, t1)) = super::xbuild::edge_range(&s.c, &p0, &p1, s.closed) {
            b.add_edge(s.c.clone(), t0, t1, va, vb);
        }
    }
    b.chain_wires();
    for p in pts {
        b.add_point(*p);
    }
    b.fit_tolerances(tol.max(1e-6));
    b
}

fn is_wire_item(kind: &str) -> bool {
    matches!(kind, "GEOMETRIC_CURVE_SET" | "GEOMETRIC_SET")
}

/// Builds topology items into one body.
struct TopoReader<'a> {
    g: Geo<'a>,
    xb: XBuilder,
    verts: HashMap<u64, usize>,
    /// EDGE_CURVE → our edge and whether it runs from the file edge's start to its end.
    edges: HashMap<u64, Option<(usize, bool)>>,
    /// POLY_LOOP segments by their (ordered) point records.
    segs: HashMap<(u64, u64), Option<usize>>,
    errors: Vec<String>,
}

impl<'a> TopoReader<'a> {
    fn vertex(&mut self, id: u64) -> R<usize> {
        if let Some(&v) = self.verts.get(&id) {
            return Ok(v);
        }
        let p = self.g.point(id)?;
        let v = self.xb.vertex(p);
        self.verts.insert(id, v);
        Ok(v)
    }

    fn edge(&mut self, id: u64) -> R<Option<(usize, bool)>> {
        if let Some(e) = self.edges.get(&id) {
            return Ok(*e);
        }
        let r = self.g.rec(id)?;
        if r.kind() != "EDGE_CURVE" {
            return Err(format!("#{id}: expected an EDGE_CURVE, found {}", r.kind()));
        }
        let ps = r.params();
        let (a, b) = (self.vertex(self.g.rf(ps, 1)?)?, self.vertex(self.g.rf(ps, 2)?)?);
        let cid = self.g.rf(ps, 3)?;
        let c = match self.g.conic_arc(cid, &self.xb.body.verts[a].p, &self.xb.body.verts[b].p)? {
            Some(c) => c,
            None => self.g.curve(cid)?,
        };
        let same = ps.get(4).and_then(|p| p.as_bool()).unwrap_or(true);
        let e = if same { self.xb.edge(c, a, b).map(|e| (e, true)) } else { self.xb.edge(c, b, a).map(|e| (e, false)) };
        self.edges.insert(id, e);
        Ok(e)
    }

    /// An oriented edge as (our edge, along it).
    fn oriented(&mut self, id: u64) -> R<Option<(usize, bool)>> {
        let r = self.g.rec(id)?;
        match r.kind() {
            "ORIENTED_EDGE" => {
                let ps = r.params();
                let o = ps.get(4).and_then(|p| p.as_bool()).unwrap_or(true);
                Ok(self.oriented(self.g.rf(ps, 3)?)?.map(|(e, f)| (e, f == o)))
            }
            "EDGE_CURVE" => self.edge(id),
            k => Err(format!("#{id}: expected an edge, found {k}")),
        }
    }

    fn poly_loop(&mut self, ps: &[Param]) -> R<Vec<(usize, bool)>> {
        let pts: Vec<u64> = at(ps, 1)?.as_list().ok_or("POLY_LOOP without points")?.iter().filter_map(|p| p.as_ref()).collect();
        let mut out = Vec::new();
        for k in 0..pts.len() {
            let (a, b) = (pts[k], pts[(k + 1) % pts.len()]);
            let key = (a.min(b), a.max(b));
            let e = match self.segs.get(&key) {
                Some(e) => *e,
                None => {
                    let (va, vb) = (self.vertex(key.0)?, self.vertex(key.1)?);
                    let (pa, pb) = (self.xb.body.verts[va].p, self.xb.body.verts[vb].p);
                    let e = if (pb - pa).norm() > 1e-12 { self.xb.edge(Curve::Line { o: pa, d: (pb - pa).normalize() }, va, vb) } else { None };
                    self.segs.insert(key, e);
                    e
                }
            };
            if let Some(e) = e {
                out.push((e, a == key.0));
            }
        }
        Ok(out)
    }

    fn face(&mut self, id: u64, flip: bool) -> R<Option<usize>> {
        let r = self.g.rec(id)?;
        let ps = r.params();
        match r.kind() {
            "ORIENTED_FACE" => {
                let o = ps.get(3).and_then(|p| p.as_bool()).unwrap_or(true);
                self.face(self.g.rf(ps, 2)?, flip ^ !o)
            }
            "ADVANCED_FACE" | "FACE_SURFACE" | "FACE" => {
                let surf = match r.kind() {
                    "FACE" => None,
                    _ => Some(self.g.surface(self.g.rf(ps, 2)?)),
                };
                let surf = match surf {
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        self.errors.push(e);
                        return Ok(None);
                    }
                    None => {
                        self.errors.push(format!("#{id}: a face without a surface"));
                        return Ok(None);
                    }
                };
                let same = ps.get(3).and_then(|p| p.as_bool()).unwrap_or(true);
                let mut loops = Vec::new();
                let mut outer_first = None;
                for (bi, b) in at(ps, 1)?.as_list().ok_or("face bounds are not a list")?.iter().enumerate() {
                    let bid = b.as_ref().ok_or("a bound is not a reference")?;
                    let br = self.g.rec(bid)?;
                    let bp = br.params();
                    let o = bp.get(2).and_then(|p| p.as_bool()).unwrap_or(true);
                    let lr = self.g.rec(self.g.rf(bp, 1)?)?;
                    let mut l = match lr.kind() {
                        "EDGE_LOOP" => {
                            let mut l = Vec::new();
                            for oe in at(lr.params(), 1)?.as_list().ok_or("an EDGE_LOOP without edges")? {
                                match self.oriented(oe.as_ref().ok_or("an edge is not a reference")?) {
                                    Ok(Some(c)) => l.push(c),
                                    Ok(None) => {}
                                    Err(e) => {
                                        self.errors.push(e);
                                        return Ok(None);
                                    }
                                }
                            }
                            l
                        }
                        "POLY_LOOP" => self.poly_loop(lr.params())?,
                        // A singular point (a pole): the rebuild puts it back.
                        "VERTEX_LOOP" => continue,
                        k => {
                            self.errors.push(format!("unsupported loop {k}"));
                            continue;
                        }
                    };
                    if !o ^ flip {
                        l.reverse();
                        for c in l.iter_mut() {
                            c.1 = !c.1;
                        }
                    }
                    if br.kind() == "FACE_OUTER_BOUND" && outer_first.is_none() {
                        outer_first = Some(loops.len());
                    }
                    let _ = bi;
                    loops.push(l);
                }
                if let Some(i) = outer_first {
                    let l = loops.remove(i);
                    loops.insert(0, l);
                }
                Ok(Some(self.xb.face(XFace { surf, sense: same ^ flip, loops })))
            }
            k => Err(format!("#{id}: expected a face, found {k}")),
        }
    }

    fn shell(&mut self, id: u64, flip: bool) -> R<usize> {
        let r = self.g.rec(id)?;
        let ps = r.params();
        match r.kind() {
            "ORIENTED_CLOSED_SHELL" | "ORIENTED_OPEN_SHELL" => {
                let o = ps.get(3).and_then(|p| p.as_bool()).unwrap_or(true);
                self.shell(self.g.rf(ps, 2)?, flip ^ !o)
            }
            "CLOSED_SHELL" | "OPEN_SHELL" => {
                let mut faces = Vec::new();
                for f in at(ps, 1)?.as_list().ok_or("shell faces are not a list")? {
                    if let Some(fi) = self.face(f.as_ref().ok_or("a face is not a reference")?, flip)? {
                        faces.push(fi);
                    }
                }
                Ok(self.xb.shell(faces, r.kind() == "CLOSED_SHELL"))
            }
            k => Err(format!("#{id}: expected a shell, found {k}")),
        }
    }

    /// One representation item (a solid or a surface model). Returns whether it held a shape.
    fn item(&mut self, id: u64) -> R<bool> {
        let r = self.g.rec(id)?;
        let ps = r.params();
        match r.kind() {
            "MANIFOLD_SOLID_BREP" | "FACETED_BREP" => {
                let s = self.shell(self.g.rf(ps, 1)?, false)?;
                self.xb.solid(vec![s]);
                Ok(true)
            }
            "BREP_WITH_VOIDS" => {
                let mut ss = vec![self.shell(self.g.rf(ps, 1)?, false)?];
                for v in at(ps, 2)?.as_list().ok_or("voids are not a list")? {
                    // Void shells are written facing out of the void (ORIENTED_CLOSED_SHELL with
                    // .F.); flipped they face into it, as our void shells do.
                    ss.push(self.shell(v.as_ref().ok_or("a void is not a reference")?, false)?);
                }
                self.xb.solid(ss);
                Ok(true)
            }
            "SHELL_BASED_SURFACE_MODEL" => {
                for s in at(ps, 1)?.as_list().ok_or("shells are not a list")? {
                    let si = self.shell(s.as_ref().ok_or("a shell is not a reference")?, false)?;
                    let _ = si;
                }
                Ok(true)
            }
            "CLOSED_SHELL" | "OPEN_SHELL" => {
                let s = self.shell(id, false)?;
                if r.kind() == "CLOSED_SHELL" {
                    self.xb.solid(vec![s]);
                }
                Ok(true)
            }
            "ADVANCED_FACE" | "FACE_SURFACE" => {
                if let Some(f) = self.face(id, false)? {
                    self.xb.shell(vec![f], false);
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

fn is_shape_item(kind: &str) -> bool {
    matches!(kind, "MANIFOLD_SOLID_BREP" | "BREP_WITH_VOIDS" | "FACETED_BREP" | "SHELL_BASED_SURFACE_MODEL" | "CLOSED_SHELL" | "OPEN_SHELL" | "ADVANCED_FACE" | "FACE_SURFACE") || is_wire_item(kind)
}

// ---------------------------------------------------------------------------------------------
// Products

/// A part read from a file: its body (in its own coordinates, mm) and product name.
#[derive(Debug, Clone)]
pub struct ReadPart {
    pub body: Body,
    pub name: String,
}

/// An occurrence: which part, where, and its instance name.
#[derive(Debug, Clone)]
pub struct ReadOccurrence {
    pub part: usize,
    pub placement: Affine,
    pub name: String,
}

/// A file's parts and occurrences (a file without assemblies has each part once at the
/// identity), the top product's name, and problems met.
#[derive(Debug, Clone, Default)]
pub struct ReadModel {
    pub name: String,
    pub parts: Vec<ReadPart>,
    pub occurrences: Vec<ReadOccurrence>,
    pub warnings: Vec<String>,
}

pub fn motion_affine(m: &Motion) -> Affine {
    Affine { m: m.linear, t: m.translation }
}

fn compose(a: &Affine, b: &Affine) -> Affine {
    Affine { m: a.m * b.m, t: a.m * b.t + a.t }
}

fn inverse(a: &Affine) -> Affine {
    let mi = a.m.try_inverse().unwrap_or_else(nalgebra::Matrix3::identity);
    Affine { m: mi, t: -(mi * a.t) }
}

fn frame_affine(f: &Frame) -> Affine {
    Affine { m: nalgebra::Matrix3::from_columns(&[f.x, f.y, f.z]), t: f.o.coords }
}

struct Model<'a> {
    f: &'a StepFile,
    /// Unit factors by context record.
    units: HashMap<u64, (f64, f64)>,
}

impl<'a> Model<'a> {
    fn rec(&self, id: u64) -> Option<&'a Record> {
        self.f.records.get(&id)
    }

    fn ids(&self, kind: &str) -> &'a [u64] {
        self.f.by_kind.get(kind).map_or(&[], |v| v.as_slice())
    }

    /// A unit's factor to mm (lengths) or radians (angles).
    fn unit_factor(&self, id: u64, depth: usize) -> Option<f64> {
        let r = self.rec(id)?;
        if depth > 8 {
            return None;
        }
        if let Some(si) = r.part("SI_UNIT").or_else(|| (r.kind() == "SI_UNIT").then(|| r.params())) {
            // SI_UNIT(prefix, name) (simple records carry the derived dimensions first).
            let (prefix, name) = if si.len() >= 2 { (&si[si.len() - 2], &si[si.len() - 1]) } else { return None };
            let pre = match prefix {
                Param::Enum(p) => match p.as_str() {
                    "EXA" => 1e18,
                    "PETA" => 1e15,
                    "TERA" => 1e12,
                    "GIGA" => 1e9,
                    "MEGA" => 1e6,
                    "KILO" => 1e3,
                    "HECTO" => 1e2,
                    "DECA" => 1e1,
                    "DECI" => 1e-1,
                    "CENTI" => 1e-2,
                    "MILLI" => 1e-3,
                    "MICRO" => 1e-6,
                    "NANO" => 1e-9,
                    "PICO" => 1e-12,
                    _ => 1.0,
                },
                _ => 1.0,
            };
            return match name {
                Param::Enum(n) if n == "METRE" => Some(pre * 1000.0),
                Param::Enum(n) if n == "RADIAN" => Some(pre),
                _ => None,
            };
        }
        if let Some(cb) = r.part("CONVERSION_BASED_UNIT") {
            let m = self.rec(cb.get(1)?.as_ref()?)?;
            // LENGTH_MEASURE_WITH_UNIT(value, unit) or the plain MEASURE_WITH_UNIT.
            let mp = &m.parts.iter().find(|p| p.0.ends_with("MEASURE_WITH_UNIT"))?.1;
            let v = mp.first()?.as_f64()?;
            let base = self.unit_factor(mp.get(1)?.as_ref()?, depth + 1)?;
            return Some(v * base);
        }
        None
    }

    /// (length → mm, angle → radians) for a representation context.
    fn context_units(&mut self, ctx: u64) -> (f64, f64) {
        if let Some(u) = self.units.get(&ctx) {
            return *u;
        }
        let mut out = (1.0, 1.0);
        if let Some(units) = self.rec(ctx).and_then(|r| r.part("GLOBAL_UNIT_ASSIGNED_CONTEXT")).and_then(|p| p.first()).and_then(|l| l.as_list()) {
            for u in units.iter().filter_map(|u| u.as_ref()) {
                let Some(r) = self.rec(u) else { continue };
                if r.has("LENGTH_UNIT") {
                    if let Some(k) = self.unit_factor(u, 0) {
                        out.0 = k;
                    }
                } else if r.has("PLANE_ANGLE_UNIT")
                    && let Some(k) = self.unit_factor(u, 0) {
                        out.1 = k;
                    }
            }
        }
        self.units.insert(ctx, out);
        out
    }

    fn rep_units(&mut self, rep: u64) -> (f64, f64) {
        let ctx = self.rec(rep).and_then(|r| r.parts.iter().find_map(|p| p.1.get(2)?.as_ref()));
        match ctx {
            Some(c) => self.context_units(c),
            None => (1.0, 1.0),
        }
    }

    /// The representation's distance uncertainty (the file's modelling precision), in mm.
    fn rep_uncertainty(&mut self, rep: u64) -> Option<f64> {
        let ctx = self.rec(rep).and_then(|r| r.parts.iter().find_map(|p| p.1.get(2)?.as_ref()))?;
        let ls = self.context_units(ctx).0;
        let list: Vec<u64> = self.rec(ctx)?.part("GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT")?.first()?.as_list()?.iter().filter_map(|x| x.as_ref()).collect();
        list.iter()
            .filter_map(|&u| {
                let r = self.rec(u)?;
                r.has("UNCERTAINTY_MEASURE_WITH_UNIT").then(|| r.part("UNCERTAINTY_MEASURE_WITH_UNIT").and_then(|ps| ps.first()?.as_f64()))?
            })
            .map(|x| x * ls)
            .filter(|x| x.is_finite() && *x > 0.0)
            .reduce(f64::max)
    }

    fn geo(&mut self, rep: u64) -> Geo<'a> {
        let (ls, angle) = self.rep_units(rep);
        Geo { f: self.f, ls, angle }
    }

    /// A representation's items.
    fn items(&self, rep: u64) -> Vec<u64> {
        let Some(r) = self.rec(rep) else { return Vec::new() };
        r.parts.iter().find_map(|p| p.1.get(1)?.as_list()).map(|l| l.iter().filter_map(|x| x.as_ref()).collect()).unwrap_or_default()
    }

    /// The shape items reachable from a representation, each with its placement in the
    /// representation's coordinates and the representation that holds it (for its units).
    fn rep_shapes(&mut self, rep: u64, seen: &mut HashSet<u64>, out: &mut Vec<(u64, u64, Affine)>, at: Affine) {
        if !seen.insert(rep) {
            return;
        }
        for it in self.items(rep) {
            let Some(r) = self.rec(it) else { continue };
            if is_shape_item(r.kind()) {
                out.push((it, rep, at));
            } else if r.kind() == "MAPPED_ITEM" {
                // MAPPED_ITEM(name, REPRESENTATION_MAP(origin, rep), target).
                let ps = r.params();
                let (Some(map), Some(target)) = (ps.get(1).and_then(|p| p.as_ref()), ps.get(2).and_then(|p| p.as_ref())) else { continue };
                let Some(mr) = self.rec(map) else { continue };
                let (Some(origin), Some(mrep)) = (mr.params().first().and_then(|p| p.as_ref()), mr.params().get(1).and_then(|p| p.as_ref())) else { continue };
                let (go, gt) = (self.geo(mrep), self.geo(rep));
                let (Ok(fo), Ok(ft)) = (go.axis2(origin), gt.axis2(target)) else { continue };
                let t = compose(&frame_affine(&ft), &inverse(&frame_affine(&fo)));
                self.rep_shapes(mrep, seen, out, compose(&at, &t));
            }
        }
        // Plain relationships between representations of the same shape.
        for &rr in self.ids("SHAPE_REPRESENTATION_RELATIONSHIP") {
            let Some(r) = self.rec(rr) else { continue };
            if r.has("REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION") {
                continue;
            }
            let ps = r.part("REPRESENTATION_RELATIONSHIP").or_else(|| r.part("SHAPE_REPRESENTATION_RELATIONSHIP")).unwrap_or(&[]);
            let (Some(a), Some(b)) = (ps.get(2).and_then(|p| p.as_ref()), ps.get(3).and_then(|p| p.as_ref())) else { continue };
            if a == rep {
                self.rep_shapes(b, seen, out, at);
            } else if b == rep {
                self.rep_shapes(a, seen, out, at);
            }
        }
    }

    /// The representations of a product definition (or of a NAUO's shape).
    fn reps_of(&self, def: u64) -> Vec<u64> {
        let mut out = Vec::new();
        for &pds in self.ids("PRODUCT_DEFINITION_SHAPE") {
            if self.rec(pds).and_then(|r| r.params().get(2)?.as_ref()) != Some(def) {
                continue;
            }
            for &sdr in self.ids("SHAPE_DEFINITION_REPRESENTATION") {
                let Some(r) = self.rec(sdr) else { continue };
                if r.params().first().and_then(|p| p.as_ref()) == Some(pds)
                    && let Some(rep) = r.params().get(1).and_then(|p| p.as_ref()) {
                        out.push(rep);
                    }
            }
        }
        out
    }

    fn product_name(&self, pd: u64) -> String {
        let name = (|| {
            let pdf = self.rec(pd)?.params().get(2)?.as_ref()?;
            let prod = self.rec(pdf)?.params().get(2)?.as_ref()?;
            let ps = self.rec(prod)?.params();
            let n = ps.get(1)?.as_str().filter(|s| !s.is_empty()).or_else(|| ps.first()?.as_str())?;
            Some(n.to_string())
        })();
        name.unwrap_or_default()
    }

    /// A NAUO's placement of its child in its parent, by its context-dependent representation.
    fn nauo_placement(&mut self, nauo: u64, child_reps: &[u64]) -> Affine {
        let mut out = Affine::identity();
        for &pds in self.ids("PRODUCT_DEFINITION_SHAPE") {
            if self.rec(pds).and_then(|r| r.params().get(2)?.as_ref()) != Some(nauo) {
                continue;
            }
            for &cdsr in self.ids("CONTEXT_DEPENDENT_SHAPE_REPRESENTATION") {
                let Some(r) = self.rec(cdsr) else { continue };
                if r.params().get(1).and_then(|p| p.as_ref()) != Some(pds) {
                    continue;
                }
                let Some(rr) = r.params().first().and_then(|p| p.as_ref()).and_then(|i| self.rec(i)) else { continue };
                let Some(base) = rr.part("REPRESENTATION_RELATIONSHIP") else { continue };
                let (Some(r1), Some(r2)) = (base.get(2).and_then(|p| p.as_ref()), base.get(3).and_then(|p| p.as_ref())) else { continue };
                let Some(tr) = rr.part("REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION").and_then(|p| p.first()?.as_ref()) else { continue };
                let Some(t) = self.rec(tr) else { continue };
                let swapped = child_reps.contains(&r2) && !child_reps.contains(&r1);
                let m = match t.kind() {
                    "ITEM_DEFINED_TRANSFORMATION" => {
                        let ps = t.params();
                        let (Some(i1), Some(i2)) = (ps.get(2).and_then(|p| p.as_ref()), ps.get(3).and_then(|p| p.as_ref())) else { continue };
                        let (g1, g2) = (self.geo(r1), self.geo(r2));
                        let (Ok(f1), Ok(f2)) = (g1.axis2(i1), g2.axis2(i2)) else { continue };
                        let (a1, a2) = (frame_affine(&f1), frame_affine(&f2));
                        if swapped { compose(&a1, &inverse(&a2)) } else { compose(&a2, &inverse(&a1)) }
                    }
                    "CARTESIAN_TRANSFORMATION_OPERATOR_3D" => {
                        let g = self.geo(if swapped { r1 } else { r2 });
                        let ps = t.params();
                        let d = |i: usize, def: V3| ps.get(i).and_then(|p| p.as_ref()).and_then(|r| g.direction(r).ok()).unwrap_or(def);
                        let (x, y) = (d(3, V3::x()), d(4, V3::y()));
                        let o = ps.get(5).and_then(|p| p.as_ref()).and_then(|r| g.point(r).ok()).unwrap_or(P3::origin());
                        let z = d(7, x.cross(&y));
                        let a = Affine { m: nalgebra::Matrix3::from_columns(&[x, y, z]), t: o.coords };
                        if swapped { inverse(&a) } else { a }
                    }
                    _ => continue,
                };
                out = m;
                return out;
            }
        }
        out
    }

    fn build_part(&mut self, reps: &[u64], warnings: &mut Vec<String>) -> Option<Body> {
        let mut shapes = Vec::new();
        let mut seen = HashSet::new();
        for &rep in reps {
            self.rep_shapes(rep, &mut seen, &mut shapes, Affine::identity());
        }
        build_shapes(self, &shapes, warnings)
    }
}

/// Builds shape items (each with the representation holding it and a placement) into one body.
fn build_shapes(m: &mut Model, shapes: &[(u64, u64, Affine)], warnings: &mut Vec<String>) -> Option<Body> {
    if shapes.is_empty() {
        return None;
    }
    let mut body = Body::default();
    // Curves and points (wireframes), read only for a part with nothing else.
    let mut wires = Body::default();
    let is_wire: Vec<bool> = shapes.iter().map(|(it, ..)| m.rec(*it).is_some_and(|r| is_wire_item(r.kind()))).collect();
    for (it, rep, at) in shapes.iter().zip(&is_wire).filter(|(_, w)| **w).map(|(s, _)| s) {
        let g = m.geo(*rep);
        let (mut segs, mut pts) = (Vec::new(), Vec::new());
        if let Err(e) = g.wire_items(*it, &mut segs, &mut pts, 0) {
            warnings.push(e);
        }
        let size = segs.iter().flat_map(|s| [s.a, s.b]).chain(pts.iter().copied()).map(|p| p.coords.amax().max(-p.coords.min())).fold(1.0, f64::max);
        let tol = m.rep_uncertainty(*rep).unwrap_or(1e-7 * size).max(tol::REL * size);
        let w = wire_body(&segs, &pts, tol);
        wires.append(&if *at == Affine::identity() { w } else { w.transformed(at) });
    }
    // Each item gets its own vertices and edges (solids that share edge records in the file,
    // touching along them, stay separate solids).
    let groups: Vec<(u64, Affine, Vec<u64>)> = shapes.iter().zip(&is_wire).filter(|(_, w)| !**w).map(|((it, rep, at), _)| (*rep, *at, vec![*it])).collect();
    for (rep, at, items) in groups {
        let g = m.geo(rep);
        let mut tr = TopoReader { g, xb: XBuilder::new(), verts: HashMap::new(), edges: HashMap::new(), segs: HashMap::new(), errors: Vec::new() };
        for it in items {
            if let Err(e) = tr.item(it) {
                tr.errors.push(e);
            }
        }
        let errors = std::mem::take(&mut tr.errors);
        warnings.extend(errors);
        let (mut b, w) = tr.xb.finish();
        warnings.extend(w);
        // The file's precision is its entities' tolerance (OCCT's reader does the same).
        if let Some(u) = m.rep_uncertainty(rep).filter(|u| *u > super::topo::TOL_MIN) {
            for v in &mut b.verts {
                v.tol = v.tol.max(u);
            }
            for e in &mut b.edges {
                e.tol = e.tol.max(u);
            }
        }
        let b = if at == Affine::identity() { b } else { b.transformed(&at) };
        body.append(&b);
    }
    if body.faces.is_empty() {
        body.append(&wires);
    }
    if body.faces.is_empty() && body.wires.is_empty() && body.points.is_empty() { None } else { Some(body) }
}

/// Reads a STEP file's parts and assembly structure.
pub fn read(bytes: &[u8]) -> Result<ReadModel, String> {
    let f = parse(bytes)?;
    let mut m = Model { f: &f, units: HashMap::new() };
    let mut out = ReadModel::default();
    let mut warnings = Vec::new();
    let mut pds: Vec<u64> = Vec::new();
    for k in ["PRODUCT_DEFINITION", "PRODUCT_DEFINITION_WITH_ASSOCIATED_DOCUMENTS"] {
        pds.extend(m.ids(k));
    }
    pds.sort_unstable();
    // Assembly links: parent → (NAUO, child), in file order.
    let mut children: HashMap<u64, Vec<(u64, u64)>> = HashMap::new();
    let mut is_child: HashSet<u64> = HashSet::new();
    for &n in m.ids("NEXT_ASSEMBLY_USAGE_OCCURRENCE") {
        let ps = m.rec(n).map(|r| r.params()).unwrap_or(&[]);
        if let (Some(p), Some(c)) = (ps.get(3).and_then(|x| x.as_ref()), ps.get(4).and_then(|x| x.as_ref())) {
            children.entry(p).or_default().push((n, c));
            is_child.insert(c);
        }
    }
    let mut part_of: HashMap<u64, Option<usize>> = HashMap::new();
    let roots: Vec<u64> = pds.iter().copied().filter(|p| !is_child.contains(p)).collect();
    // Depth-first from the roots.
    let mut stack: Vec<(u64, Affine, String, usize)> = roots.iter().rev().map(|&r| (r, Affine::identity(), m.product_name(r), 0)).collect();
    let mut useful_roots = Vec::new();
    while let Some((pd, at, name, depth)) = stack.pop() {
        if depth > 32 {
            continue;
        }
        let reps = m.reps_of(pd);
        let part = match part_of.get(&pd) {
            Some(p) => *p,
            None => {
                let p = m.build_part(&reps, &mut warnings).map(|body| {
                    out.parts.push(ReadPart { body, name: m.product_name(pd) });
                    out.parts.len() - 1
                });
                part_of.insert(pd, p);
                p
            }
        };
        if depth == 0 && (part.is_some() || children.contains_key(&pd)) {
            useful_roots.push(pd);
        }
        if let Some(p) = part {
            out.occurrences.push(ReadOccurrence { part: p, placement: at, name: name.clone() });
        }
        if let Some(ch) = children.get(&pd).cloned() {
            for &(nauo, c) in ch.iter().rev() {
                let creps = m.reps_of(c);
                let pl = m.nauo_placement(nauo, &creps);
                let ps = m.rec(nauo).map(|r| r.params()).unwrap_or(&[]);
                let nm = ps.get(1).and_then(|x| x.as_str()).filter(|s| !s.is_empty()).or_else(|| ps.first().and_then(|x| x.as_str())).unwrap_or("").to_string();
                stack.push((c, compose(&at, &pl), nm, depth + 1));
            }
        }
    }
    if useful_roots.len() == 1 {
        out.name = m.product_name(useful_roots[0]);
    }
    if out.parts.is_empty() {
        // No product structure: every shape item in the file, as one part.
        let reps: Vec<u64> = ["ADVANCED_BREP_SHAPE_REPRESENTATION", "MANIFOLD_SURFACE_SHAPE_REPRESENTATION", "SHAPE_REPRESENTATION", "FACETED_BREP_SHAPE_REPRESENTATION", "GEOMETRICALLY_BOUNDED_WIREFRAME_SHAPE_REPRESENTATION"].iter().flat_map(|k| m.ids(k).iter().copied()).collect();
        let mut shapes = Vec::new();
        let mut seen = HashSet::new();
        for rep in reps {
            m.rep_shapes(rep, &mut seen, &mut shapes, Affine::identity());
        }
        if shapes.is_empty() {
            // Bare solids with no representation at all (mm).
            for k in ["MANIFOLD_SOLID_BREP", "BREP_WITH_VOIDS", "SHELL_BASED_SURFACE_MODEL"] {
                for &id in m.ids(k) {
                    shapes.push((id, 0, Affine::identity()));
                }
            }
        }
        if let Some(body) = build_shapes(&mut m, &shapes, &mut warnings) {
            out.parts.push(ReadPart { body, name: String::new() });
            out.occurrences.push(ReadOccurrence { part: 0, placement: Affine::identity(), name: String::new() });
        }
    }
    if out.parts.is_empty() {
        return Err(if warnings.is_empty() { "the file holds no shapes".into() } else { format!("the file holds no shapes we can read ({})", warnings[0]) });
    }
    out.warnings = warnings;
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Writing

/// A real in the file's notation: always with a decimal point, shortest round-trip digits. A
/// non-finite value (which no file can hold; the exporters refuse such bodies) is written as 0.
pub fn real(x: f64) -> String {
    if !x.is_finite() {
        return "0.".into();
    }
    if x == 0.0 {
        return "0.".into();
    }
    let a = x.abs();
    if (1e-4..1e15).contains(&a) {
        let s = format!("{x}");
        if s.contains('.') { s } else { s + "." }
    } else {
        // `{:e}` of a finite number always has its exponent after an `e`.
        let s = format!("{x:e}");
        let (m, e) = s.split_once('e').unwrap_or((s.as_str(), "0"));
        let m = if m.contains('.') { m.to_string() } else { format!("{m}.") };
        format!("{m}E{e}")
    }
}

/// The right-handed form in which writers put a surface (frames from an origin, an axis and a
/// reference direction; a cone opening along its axis with a semi-angle in (0, π/2) and a
/// radius ≥ 0). Readers rebuild exactly this surface, so normals can be compared.
pub fn canonical(s: &Surface) -> Surface {
    let rh = |o: P3, z: V3, x: V3| Frame::from_xz(o, x, z);
    match s {
        Surface::Plane { f } => Surface::Plane { f: rh(f.o, f.x.cross(&f.y), f.x) },
        Surface::Cylinder { f, r } => Surface::Cylinder { f: rh(f.o, f.z, f.x), r: *r },
        Surface::Cone { f, r, a } => {
            // r + h·tan a at height h along f.z: open along ±z; move to the apex if r < 0.
            let (sa, ca) = a.sin_cos();
            let tan = sa / ca;
            let z = if tan >= 0.0 { f.z } else { -f.z };
            let (o, rr) = if *r >= 0.0 { (f.o, *r) } else { (f.o + f.z * (-r / tan), 0.0) };
            Surface::Cone { f: rh(o, z, f.x), r: rr, a: tan.abs().atan() }
        }
        Surface::Sphere { f, r } => Surface::Sphere { f: rh(f.o, f.z, f.x), r: *r },
        Surface::Torus { f, rmaj, rmin } => Surface::Torus { f: rh(f.o, f.z, f.x), rmaj: *rmaj, rmin: *rmin },
        Surface::Revolution { f, m } => Surface::Revolution { f: rh(f.o, f.z, f.x), m: m.clone() },
        Surface::Extrusion { .. } | Surface::Nurbs(_) => s.clone(),
    }
}

/// Whether a face's outward normal agrees with the natural normal of `written` (the face's
/// surface as written), at the middle of its first real coedge (off any pole).
pub fn same_sense(b: &Body, f: &Face, written: &Surface) -> bool {
    let Some(c) = f.loops.iter().flatten().find(|c| !b.edges[c.edge].degenerate) else { return f.sense };
    let e = &b.edges[c.edge];
    let uv = c.pc.point(e.mid_t());
    let n_out = f.surf.normal(uv.x, uv.y) * if f.sense { 1.0 } else { -1.0 };
    let p = f.surf.eval(uv.x, uv.y);
    let (u2, v2) = written.project(&p, None);
    written.normal(u2, v2).dot(&n_out) > 0.0
}

struct Writer {
    out: String,
    next: u64,
}

impl Writer {
    fn add(&mut self, s: &str) -> u64 {
        let id = self.next;
        self.next += 1;
        let _ = writeln!(self.out, "#{id}={s};");
        id
    }
    fn point(&mut self, p: &P3) -> u64 {
        self.add(&format!("CARTESIAN_POINT('',({},{},{}))", real(p.x), real(p.y), real(p.z)))
    }
    fn dir(&mut self, d: &V3) -> u64 {
        let d = d.normalize();
        self.add(&format!("DIRECTION('',({},{},{}))", real(d.x), real(d.y), real(d.z)))
    }
    /// A placement at `o` with axis `z` and reference `x` (made square to z).
    fn axis2(&mut self, o: &P3, z: &V3, x: &V3) -> u64 {
        let z = z.normalize();
        let x = (x - z * x.dot(&z)).normalize();
        let (p, a, r) = (self.point(o), self.dir(&z), self.dir(&x));
        self.add(&format!("AXIS2_PLACEMENT_3D('',#{p},#{a},#{r})"))
    }
    fn refs(ids: &[u64]) -> String {
        ids.iter().map(|i| format!("#{i}")).collect::<Vec<_>>().join(",")
    }
    fn reals(xs: &[f64]) -> String {
        xs.iter().map(|x| real(*x)).collect::<Vec<_>>().join(",")
    }
    fn ints(xs: &[usize]) -> String {
        xs.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
    }

    fn bspline_curve(&mut self, n: &NurbsCurve) -> u64 {
        let pts: Vec<u64> = n.cps.iter().map(|h| self.point(&super::nurbs::dehom(h))).collect();
        let (k, m) = knot_mults(&n.knots);
        let (deg, cp) = (n.degree, Self::refs(&pts));
        let (ms, ks) = (Self::ints(&m), Self::reals(&k));
        if n.is_rational() {
            let w: Vec<f64> = n.cps.iter().map(|h| h.w).collect();
            self.add(&format!(
                "(BOUNDED_CURVE() B_SPLINE_CURVE({deg},({cp}),.UNSPECIFIED.,.F.,.F.) B_SPLINE_CURVE_WITH_KNOTS(({ms}),({ks}),.UNSPECIFIED.) CURVE() GEOMETRIC_REPRESENTATION_ITEM() RATIONAL_B_SPLINE_CURVE(({})) REPRESENTATION_ITEM(''))",
                Self::reals(&w)
            ))
        } else {
            self.add(&format!("B_SPLINE_CURVE_WITH_KNOTS('',{deg},({cp}),.UNSPECIFIED.,.F.,.F.,({ms}),({ks}),.UNSPECIFIED.)"))
        }
    }

    fn curve(&mut self, c: &Curve, t0: f64) -> u64 {
        match c {
            Curve::Line { o, d } => {
                let p = self.point(&(o + d * t0));
                let dd = self.dir(d);
                let v = self.add(&format!("VECTOR('',#{dd},{})", real(d.norm())));
                self.add(&format!("LINE('',#{p},#{v})"))
            }
            Curve::Circle { c, x, y, r } => {
                let a = self.axis2(c, &x.cross(y), x);
                self.add(&format!("CIRCLE('',#{a},{})", real(*r)))
            }
            Curve::Ellipse { c, x, y, a, b } => {
                // The first semi-axis is the major one.
                let (x2, y2, a2, b2) = if a >= b { (*x, *y, *a, *b) } else { (*y, -*x, *b, *a) };
                let ax = self.axis2(c, &x2.cross(&y2), &x2);
                self.add(&format!("ELLIPSE('',#{ax},{},{})", real(a2), real(b2)))
            }
            Curve::Nurbs(n) => self.bspline_curve(n),
        }
    }

    /// Writes a surface (in its [`canonical`] form); returns its record and that form.
    fn surface(&mut self, s: &Surface) -> (u64, Surface) {
        let c = canonical(s);
        let id = match &c {
            Surface::Plane { f } => {
                let a = self.axis2(&f.o, &f.z, &f.x);
                self.add(&format!("PLANE('',#{a})"))
            }
            Surface::Cylinder { f, r } => {
                let a = self.axis2(&f.o, &f.z, &f.x);
                self.add(&format!("CYLINDRICAL_SURFACE('',#{a},{})", real(*r)))
            }
            Surface::Cone { f, r, a } => {
                let ax = self.axis2(&f.o, &f.z, &f.x);
                self.add(&format!("CONICAL_SURFACE('',#{ax},{},{})", real(*r), real(*a)))
            }
            Surface::Sphere { f, r } => {
                let a = self.axis2(&f.o, &f.z, &f.x);
                self.add(&format!("SPHERICAL_SURFACE('',#{a},{})", real(*r)))
            }
            Surface::Torus { f, rmaj, rmin } => {
                let a = self.axis2(&f.o, &f.z, &f.x);
                self.add(&format!("TOROIDAL_SURFACE('',#{a},{},{})", real(*rmaj), real(*rmin)))
            }
            Surface::Revolution { f, m } => {
                let c = self.curve(m, 0.0);
                let (p, d) = (self.point(&f.o), self.dir(&f.z));
                let ax = self.add(&format!("AXIS1_PLACEMENT('',#{p},#{d})"));
                self.add(&format!("SURFACE_OF_REVOLUTION('',#{c},#{ax})"))
            }
            Surface::Extrusion { c, d } => {
                let ci = self.curve(c, 0.0);
                let dd = self.dir(d);
                let v = self.add(&format!("VECTOR('',#{dd},{})", real(d.norm())));
                self.add(&format!("SURFACE_OF_LINEAR_EXTRUSION('',#{ci},#{v})"))
            }
            Surface::Nurbs(n) => self.bspline_surface(n),
        };
        (id, c)
    }

    fn bspline_surface(&mut self, n: &NurbsSurface) -> u64 {
        let rows: Vec<String> = n.cps.iter().map(|row| format!("({})", Self::refs(&row.iter().map(|h| self.point(&super::nurbs::dehom(h))).collect::<Vec<_>>()))).collect();
        let (ku, mu) = knot_mults(&n.ku);
        let (kv, mv) = knot_mults(&n.kv);
        let rational = {
            let w0 = n.cps[0][0].w;
            n.cps.iter().flatten().any(|h| (h.w - w0).abs() > 1e-14)
        };
        let (du, dv, net) = (n.du, n.dv, rows.join(","));
        let kn = format!("({}),({}),({}),({})", Self::ints(&mu), Self::ints(&mv), Self::reals(&ku), Self::reals(&kv));
        if rational {
            let w: Vec<String> = n.cps.iter().map(|row| format!("({})", Self::reals(&row.iter().map(|h| h.w).collect::<Vec<_>>()))).collect();
            self.add(&format!(
                "(BOUNDED_SURFACE() B_SPLINE_SURFACE({du},{dv},({net}),.UNSPECIFIED.,.F.,.F.,.F.) B_SPLINE_SURFACE_WITH_KNOTS({kn},.UNSPECIFIED.) GEOMETRIC_REPRESENTATION_ITEM() RATIONAL_B_SPLINE_SURFACE(({})) REPRESENTATION_ITEM('') SURFACE())",
                w.join(",")
            ))
        } else {
            self.add(&format!("B_SPLINE_SURFACE_WITH_KNOTS('',{du},{dv},({net}),.UNSPECIFIED.,.F.,.F.,.F.,{kn},.UNSPECIFIED.)"))
        }
    }

    /// Writes a body's wires and points as one `GEOMETRIC_CURVE_SET` of trimmed curves (each
    /// trimmed by its end points and its parameters in the file's terms) and points.
    fn wireframe(&mut self, b: &Body) -> Vec<u64> {
        let mut items = Vec::new();
        for &(ei, _) in b.wires.iter().flat_map(|w| w.edges.iter()) {
            let e = &b.edges[ei];
            let basis = self.curve(&e.curve, e.t[0]);
            // The file's parameter of our t (see `curve`: a line starts at t0, an ellipse with
            // its axes swapped a quarter turn on).
            let u = |t: f64| match &e.curve {
                Curve::Line { .. } => t - e.t[0],
                Curve::Ellipse { a, b, .. } if a < b => t - std::f64::consts::FRAC_PI_2,
                _ => t,
            };
            let (p0, p1) = (self.point(&b.verts[e.v[0]].p), self.point(&b.verts[e.v[1]].p));
            items.push(self.add(&format!(
                "TRIMMED_CURVE('',#{basis},(#{p0},PARAMETER_VALUE({})),(#{p1},PARAMETER_VALUE({})),.T.,.PARAMETER.)",
                real(u(e.t[0])),
                real(u(e.t[1]))
            )));
        }
        for &v in &b.points {
            items.push(self.point(&b.verts[v].p));
        }
        if items.is_empty() {
            return Vec::new();
        }
        vec![self.add(&format!("GEOMETRIC_CURVE_SET('',({}))", Self::refs(&items)))]
    }

    /// Writes a body's shape items: solids, and free shells as one surface model.
    fn body(&mut self, b: &Body) -> (Vec<u64>, Vec<u64>) {
        let mut vid = vec![0u64; b.verts.len()];
        let mut used = vec![false; b.verts.len()];
        let wire_edge: HashSet<usize> = b.wires.iter().flat_map(|w| w.edges.iter().map(|e| e.0)).collect();
        for (_, e) in b.edges.iter().enumerate().filter(|(i, e)| !e.degenerate && !wire_edge.contains(i)) {
            used[e.v[0]] = true;
            used[e.v[1]] = true;
        }
        for (i, v) in b.verts.iter().enumerate() {
            if used[i] {
                let p = self.point(&v.p);
                vid[i] = self.add(&format!("VERTEX_POINT('',#{p})"));
            }
        }
        let mut eid = vec![0u64; b.edges.len()];
        let in_wire: HashSet<usize> = b.wires.iter().flat_map(|w| w.edges.iter().map(|e| e.0)).collect();
        for (i, e) in b.edges.iter().enumerate() {
            if e.degenerate || in_wire.contains(&i) {
                continue;
            }
            let c = self.curve(&e.curve, e.t[0]);
            eid[i] = self.add(&format!("EDGE_CURVE('',#{},#{},#{c},.T.)", vid[e.v[0]], vid[e.v[1]]));
        }
        // Faces, by shell (a void shell's faces are written reversed).
        let in_solid: HashSet<usize> = b.solids.iter().flat_map(|s| s.shells.iter().copied()).collect();
        let voids: HashSet<usize> = b.solids.iter().flat_map(|s| s.shells.iter().skip(1).copied()).collect();
        let mut shell_ids = vec![0u64; b.shells.len()];
        for (si, sh) in b.shells.iter().enumerate() {
            let rev = voids.contains(&si);
            let faces: Vec<u64> = sh.faces.iter().filter_map(|&fi| self.face(b, &b.faces[fi], &eid, rev)).collect();
            let kind = if sh.closed && in_solid.contains(&si) { "CLOSED_SHELL" } else { "OPEN_SHELL" };
            shell_ids[si] = self.add(&format!("{kind}('',({}))", Self::refs(&faces)));
        }
        let mut solids = Vec::new();
        for s in &b.solids {
            let outer = shell_ids[s.shells[0]];
            if s.shells.len() == 1 {
                solids.push(self.add(&format!("MANIFOLD_SOLID_BREP('',#{outer})")));
            } else {
                let vs: Vec<u64> = s.shells[1..].iter().map(|&v| self.add(&format!("ORIENTED_CLOSED_SHELL('',*,#{},.F.)", shell_ids[v]))).collect();
                solids.push(self.add(&format!("BREP_WITH_VOIDS('',#{outer},({}))", Self::refs(&vs))));
            }
        }
        let free: Vec<u64> = (0..b.shells.len()).filter(|s| !in_solid.contains(s)).map(|s| shell_ids[s]).collect();
        let mut sheets = Vec::new();
        if !free.is_empty() {
            sheets.push(self.add(&format!("SHELL_BASED_SURFACE_MODEL('',({}))", Self::refs(&free))));
        }
        (solids, sheets)
    }

    fn face(&mut self, b: &Body, f: &Face, eid: &[u64], rev: bool) -> Option<u64> {
        let (sid, written) = self.surface(&f.surf);
        let same = same_sense(b, f, &written);
        let mut bounds = Vec::new();
        for (li, l) in f.loops.iter().enumerate() {
            let oes: Vec<u64> = l.iter().filter(|c| !b.edges[c.edge].degenerate).map(|c| self.add(&format!("ORIENTED_EDGE('',*,*,#{},{})", eid[c.edge], if c.fwd { ".T." } else { ".F." }))).collect();
            if oes.is_empty() {
                continue;
            }
            let lp = self.add(&format!("EDGE_LOOP('',({}))", Self::refs(&oes)));
            let kind = if li == 0 { "FACE_OUTER_BOUND" } else { "FACE_BOUND" };
            bounds.push(self.add(&format!("{kind}('',#{lp},{})", if rev { ".F." } else { ".T." })));
        }
        if bounds.is_empty() && !matches!(f.surf, Surface::Sphere { .. } | Surface::Torus { .. }) {
            return None;
        }
        Some(self.add(&format!("ADVANCED_FACE('',({}),#{sid},{})", Self::refs(&bounds), if same ^ rev { ".T." } else { ".F." })))
    }
}

/// The date and time now as `YYYY-MM-DDThh:mm:ss` (UTC).
pub fn timestamp() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    // Civil from days (H. Hinnant).
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}", rem / 3600, rem / 60 % 60, rem % 60)
}

/// An instance to write: which part, where (a rigid motion, or a reflection which is baked into
/// a mirrored copy of the part), and its name.
#[derive(Debug, Clone)]
pub struct WriteInstance {
    pub part: usize,
    pub placement: Affine,
    pub name: String,
}

/// Writes parts (bodies with product names) as a STEP file, as one assembly `name` of
/// `instances` when there are any.
pub fn write(name: &str, parts: &[(&Body, &str)], instances: &[WriteInstance]) -> Vec<u8> {
    let mut w = Writer { out: String::new(), next: 1 };
    let mut s = String::new();
    s.push_str("ISO-10303-21;\nHEADER;\n");
    let _ = writeln!(s, "FILE_DESCRIPTION(('cadrs model'),'2;1');");
    let _ = writeln!(s, "FILE_NAME({},'{}',(''),(''),'cadrs native kernel','cadrs','');", encode_string(name), timestamp());
    s.push_str("FILE_SCHEMA(('AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }'));\nENDSEC;\nDATA;\n");
    let app = w.add("APPLICATION_CONTEXT('core data for automotive mechanical design processes')");
    w.add(&format!("APPLICATION_PROTOCOL_DEFINITION('international standard','automotive_design',2000,#{app})"));
    let pctx = w.add(&format!("PRODUCT_CONTEXT('',#{app},'mechanical')"));
    let pdctx = w.add(&format!("PRODUCT_DEFINITION_CONTEXT('part definition',#{app},'design')"));
    let mm = w.add("(LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.))");
    let rad = w.add("(NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.))");
    let sr = w.add("(NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT())");
    // The parts' largest entity tolerance (1e-7 for exact geometry).
    let tol = parts.iter().flat_map(|(b, _)| b.verts.iter().map(|v| v.tol).chain(b.edges.iter().map(|e| e.tol))).fold(super::topo::TOL_MIN, f64::max);
    let unc = w.add(&format!("UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE({}),#{mm},'distance_accuracy_value','confusion accuracy')", real(tol)));
    let ctx = w.add(&format!(
        "(GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{unc})) GLOBAL_UNIT_ASSIGNED_CONTEXT((#{mm},#{rad},#{sr})) REPRESENTATION_CONTEXT('Context #1','3D Context with UNIT and UNCERTAINTY'))"
    ));
    let origin = |w: &mut Writer| w.axis2(&P3::origin(), &V3::z(), &V3::x());
    // Product records: (product definition, representation, origin placement).
    let product = |w: &mut Writer, pname: &str, items: &[u64], kind: &str, ax: u64| -> (u64, u64) {
        let n = encode_string(pname);
        let p = w.add(&format!("PRODUCT({n},{n},'',(#{pctx}))"));
        w.add(&format!("PRODUCT_RELATED_PRODUCT_CATEGORY('part',$,(#{p}))"));
        let pf = w.add(&format!("PRODUCT_DEFINITION_FORMATION('','',#{p})"));
        let pd = w.add(&format!("PRODUCT_DEFINITION('design','',#{pf},#{pdctx})"));
        let pds = w.add(&format!("PRODUCT_DEFINITION_SHAPE('','',#{pd})"));
        let mut all = vec![ax];
        all.extend_from_slice(items);
        let rep = w.add(&format!("{kind}({n},({}),#{ctx})", Writer::refs(&all)));
        w.add(&format!("SHAPE_DEFINITION_REPRESENTATION(#{pds},#{rep})"));
        (pd, rep)
    };
    // Mirrored instances get a mirrored copy of their part.
    let mut bodies: Vec<(Body, String)> = Vec::new();
    let mut insts: Vec<(usize, Affine, String)> = Vec::new();
    let mut mirrored: HashMap<usize, usize> = HashMap::new();
    for i in instances {
        if i.placement.det() < 0.0 {
            let k = *mirrored.entry(i.part).or_insert_with(|| {
                let refl = Affine { m: nalgebra::Matrix3::from_diagonal(&V3::new(1.0, 1.0, -1.0)), t: V3::zeros() };
                bodies.push((parts[i.part].0.transformed(&refl), format!("{} (mirrored)", parts[i.part].1)));
                parts.len() + bodies.len() - 1
            });
            let refl = Affine { m: nalgebra::Matrix3::from_diagonal(&V3::new(1.0, 1.0, -1.0)), t: V3::zeros() };
            insts.push((k, compose(&i.placement, &refl), i.name.clone()));
        } else {
            insts.push((i.part, i.placement, i.name.clone()));
        }
    }
    let all_parts: Vec<(&Body, &str)> = parts.iter().copied().chain(bodies.iter().map(|(b, n)| (b, n.as_str()))).collect();
    let mut written: Vec<(u64, u64, u64)> = Vec::new();
    for (b, pname) in &all_parts {
        let (solids, sheets) = w.body(b);
        let wires = w.wireframe(b);
        let ax = origin(&mut w);
        // The first kind present is the part's representation; the others are related to it.
        let kinds: Vec<(&str, &Vec<u64>)> = [("ADVANCED_BREP_SHAPE_REPRESENTATION", &solids), ("MANIFOLD_SURFACE_SHAPE_REPRESENTATION", &sheets), ("GEOMETRICALLY_BOUNDED_WIREFRAME_SHAPE_REPRESENTATION", &wires)]
            .into_iter()
            .filter(|(_, items)| !items.is_empty())
            .collect();
        let (pd, rep) = match kinds.split_first() {
            None => product(&mut w, pname, &[], "SHAPE_REPRESENTATION", ax),
            Some(((kind, items), rest)) => {
                let (pd, rep) = product(&mut w, pname, items, kind, ax);
                for (kind, items) in rest {
                    let ax2 = origin(&mut w);
                    let n = encode_string(pname);
                    let srep = w.add(&format!("{kind}({n},(#{ax2},{}),#{ctx})", Writer::refs(items)));
                    w.add(&format!("SHAPE_REPRESENTATION_RELATIONSHIP('','',#{rep},#{srep})"));
                }
                (pd, rep)
            }
        };
        written.push((pd, rep, ax));
    }
    if !insts.is_empty() {
        let ax0 = origin(&mut w);
        let places: Vec<u64> = insts
            .iter()
            .map(|(_, a, _)| {
                let o = P3::from(a.t);
                w.axis2(&o, &(a.m * V3::z()), &(a.m * V3::x()))
            })
            .collect();
        let mut items = vec![ax0];
        items.extend(&places);
        // The assembly: its representation holds the instances' placements.
        let n = encode_string(name);
        let p = w.add(&format!("PRODUCT({n},{n},'',(#{pctx}))"));
        w.add(&format!("PRODUCT_RELATED_PRODUCT_CATEGORY('part',$,(#{p}))"));
        let pf = w.add(&format!("PRODUCT_DEFINITION_FORMATION('','',#{p})"));
        let apd = w.add(&format!("PRODUCT_DEFINITION('design','',#{pf},#{pdctx})"));
        let pds = w.add(&format!("PRODUCT_DEFINITION_SHAPE('','',#{apd})"));
        let arep = w.add(&format!("SHAPE_REPRESENTATION({n},({}),#{ctx})", Writer::refs(&items)));
        w.add(&format!("SHAPE_DEFINITION_REPRESENTATION(#{pds},#{arep})"));
        for (k, (part, _, iname)) in insts.iter().enumerate() {
            let (ppd, prep, pax) = written[*part];
            let nm = encode_string(iname);
            let nauo = w.add(&format!("NEXT_ASSEMBLY_USAGE_OCCURRENCE({nm},{nm},'',#{apd},#{ppd},$)"));
            let npds = w.add(&format!("PRODUCT_DEFINITION_SHAPE({nm},'',#{nauo})"));
            let idt = w.add(&format!("ITEM_DEFINED_TRANSFORMATION('','',#{pax},#{})", places[k]));
            let rr = w.add(&format!("(REPRESENTATION_RELATIONSHIP('','',#{prep},#{arep}) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(#{idt}) SHAPE_REPRESENTATION_RELATIONSHIP())"));
            w.add(&format!("CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(#{rr},#{npds})"));
        }
    }
    s.push_str(&w.out);
    s.push_str("ENDSEC;\nEND-ISO-10303-21;\n");
    s.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_complex_records_comments_and_strings() {
        let src = br"ISO-10303-21;
HEADER; FILE_NAME('a''b','',(''),(''),'','','');
ENDSEC;
DATA;
/* a comment */ #10 = ( BOUNDED_CURVE() B_SPLINE_CURVE(2,(#1,#2,
  #3),.UNSPECIFIED.,.F.,.F.) B_SPLINE_CURVE_WITH_KNOTS((3,3),(0.,1.),.UNSPECIFIED.) CURVE()
  GEOMETRIC_REPRESENTATION_ITEM() RATIONAL_B_SPLINE_CURVE((1.,0.70710678118654757,1.))
  REPRESENTATION_ITEM('') );
#3=CARTESIAN_POINT('',(0.,1.,0.));
#1=CARTESIAN_POINT('\X2\00E9\X0\t\\e',(1.,0.,0.));
#2 = CARTESIAN_POINT ( 'p' , ( 1.E+00, 1., -0.E0 ) ) ;
#4=LENGTH_MEASURE_WITH_UNIT(LENGTH_MEASURE(25.4),$);
ENDSEC;
END-ISO-10303-21;
";
        let f = parse(src).unwrap();
        assert_eq!(f.header[0].1[0], Param::Str("a'b".into()));
        assert_eq!(f.records[&1].params()[0], Param::Str("\u{e9}t\\e".into()));
        assert!(f.records[&10].is_complex());
        assert_eq!(f.by_kind["RATIONAL_B_SPLINE_CURVE"], vec![10]);
        assert_eq!(f.records[&4].params()[0], Param::Typed("LENGTH_MEASURE".into(), vec![Param::Num(25.4)]));
        let g = Geo { f: &f, ls: 1.0, angle: 1.0 };
        let c = g.curve(10).unwrap();
        // A quarter circle of radius 1.
        for i in 0..=10 {
            let p = c.point(i as f64 / 10.0);
            assert!((p.coords.norm() - 1.0).abs() < 1e-12);
        }
        assert!(parse(b"not a step file").is_err());
        assert_eq!(encode_string("it's \u{e9}"), "'it''s \\X2\\00E9\\X0\\'");
        assert_eq!(decode_string(encode_string("a\\b \u{1F600}").trim_matches('\'').as_bytes()), "a\\b \u{1F600}");
        for x in [1.0, 0.1, -2.5e-9, 1e20, 123456.789, 1e-4] {
            assert_eq!(parse_real(&real(x)), Some(x), "{}", real(x));
        }
    }

    /// A unit cube by hand, as other systems write it: inch units, edges with same_sense .F.,
    /// a FACE_BOUND reversed, a complex unit, records out of order.
    #[test]
    fn reads_a_hand_written_cube() {
        let mut s = String::from("ISO-10303-21;\nHEADER;\nFILE_SCHEMA(('CONFIG_CONTROL_DESIGN'));\nENDSEC;\nDATA;\n");
        let pts = [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.], [0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]];
        for (i, p) in pts.iter().enumerate() {
            let _ = writeln!(s, "#{}=CARTESIAN_POINT('',({},{},{}));", 100 + i, real(p[0]), real(p[1]), real(p[2]));
            let _ = writeln!(s, "#{}=VERTEX_POINT('',#{});", 200 + i, 100 + i);
        }
        // Edges (a, b), some written against their line.
        let edges = [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4), (0, 4), (1, 5), (2, 6), (3, 7)];
        for (k, (a, b)) in edges.iter().enumerate() {
            let (pa, pb) = (pts[*a], pts[*b]);
            let flip = k % 3 == 1;
            let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let (o, dd) = if flip { (pb, [-d[0], -d[1], -d[2]]) } else { (pa, d) };
            let _ = writeln!(s, "#{}=CARTESIAN_POINT('',({},{},{}));", 300 + k, real(o[0]), real(o[1]), real(o[2]));
            let _ = writeln!(s, "#{}=DIRECTION('',({},{},{}));", 320 + k, real(dd[0]), real(dd[1]), real(dd[2]));
            let _ = writeln!(s, "#{}=VECTOR('',#{},1.);", 340 + k, 320 + k);
            let _ = writeln!(s, "#{}=LINE('',#{},#{});", 360 + k, 300 + k, 340 + k);
            let _ = writeln!(s, "#{}=EDGE_CURVE('',#{},#{},#{},{});", 400 + k, 200 + a, 200 + b, 360 + k, if flip { ".F." } else { ".T." });
        }
        // Faces as vertex cycles, counter-clockwise about the outward normal.
        let faces: [([usize; 4], [f64; 3], [f64; 3]); 6] = [
            ([0, 3, 2, 1], [0., 0., 0.], [0., 0., -1.]),
            ([4, 5, 6, 7], [0., 0., 1.], [0., 0., 1.]),
            ([0, 1, 5, 4], [0., 0., 0.], [0., -1., 0.]),
            ([1, 2, 6, 5], [1., 0., 0.], [1., 0., 0.]),
            ([2, 3, 7, 6], [0., 1., 0.], [0., 1., 0.]),
            ([3, 0, 4, 7], [0., 0., 0.], [-1., 0., 0.]),
        ];
        let mut face_ids = Vec::new();
        for (fi, (cyc, o, n)) in faces.iter().enumerate() {
            let mut oes = Vec::new();
            // The second face's bound is written reversed (orientation .F.).
            let rev = fi == 1;
            let order: Vec<usize> = if rev { (0..4).rev().collect() } else { (0..4).collect() };
            for k in order {
                let (a, b) = if rev { (cyc[(k + 1) % 4], cyc[k]) } else { (cyc[k], cyc[(k + 1) % 4]) };
                let (ei, along) = edges.iter().enumerate().find_map(|(i, e)| if *e == (a, b) { Some((i, true)) } else if *e == (b, a) { Some((i, false)) } else { None }).unwrap();
                let id = 1000 + fi * 10 + oes.len();
                let _ = writeln!(s, "#{id}=ORIENTED_EDGE('',*,*,#{},{});", 400 + ei, if along { ".T." } else { ".F." });
                oes.push(format!("#{id}"));
            }
            let b = 2000 + fi * 10;
            let _ = writeln!(s, "#{}=EDGE_LOOP('',({}));", b, oes.join(","));
            let _ = writeln!(s, "#{}=FACE_OUTER_BOUND('',#{},{});", b + 1, b, if rev { ".F." } else { ".T." });
            let _ = writeln!(s, "#{}=CARTESIAN_POINT('',({},{},{}));", b + 2, real(o[0]), real(o[1]), real(o[2]));
            let _ = writeln!(s, "#{}=DIRECTION('',({},{},{}));", b + 3, real(n[0]), real(n[1]), real(n[2]));
            let _ = writeln!(s, "#{}=AXIS2_PLACEMENT_3D('',#{},#{},$);", b + 4, b + 2, b + 3);
            let _ = writeln!(s, "#{}=PLANE('',#{});", b + 5, b + 4);
            let _ = writeln!(s, "#{}=ADVANCED_FACE('',(#{}),#{},.T.);", b + 6, b + 1, b + 5);
            face_ids.push(format!("#{}", b + 6));
        }
        let _ = writeln!(s, "#3000=CLOSED_SHELL('',({}));", face_ids.join(","));
        let _ = writeln!(s, "#3001=MANIFOLD_SOLID_BREP('cube',#3000);");
        s.push_str("#3002=ADVANCED_BREP_SHAPE_REPRESENTATION('',(#3001),#3003);\n");
        s.push_str("#3003=(GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNIT_ASSIGNED_CONTEXT((#3004,#3007)) REPRESENTATION_CONTEXT('',''));\n");
        s.push_str("#3004=(CONVERSION_BASED_UNIT('INCH',#3005) LENGTH_UNIT() NAMED_UNIT(#3006));\n");
        s.push_str("#3005=LENGTH_MEASURE_WITH_UNIT(LENGTH_MEASURE(25.4),#3008);\n#3006=DIMENSIONAL_EXPONENTS(1.,0.,0.,0.,0.,0.,0.);\n");
        s.push_str("#3007=(NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.));\n#3008=(LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));\n");
        s.push_str("ENDSEC;\nEND-ISO-10303-21;\n");
        let m = read(s.as_bytes()).unwrap();
        assert_eq!(m.parts.len(), 1);
        let b = &m.parts[0].body;
        assert!(b.check().is_empty(), "{:?}", b.check());
        assert_eq!(b.solids.len(), 1);
        let mp = super::super::mass::body_mass(b);
        assert!((mp.volume - 25.4f64.powi(3)).abs() < 1e-6, "{}", mp.volume);
    }
}
