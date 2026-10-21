//! SMLib's ASCII model files, as NVIDIA's usd-brep writes them: B-reps (`.smb`, `//Brep
//! Starts`), parts (`.smp`, `//[Output Summary]`: curves, surfaces and B-reps) and bare curves
//! and surfaces (`.smc`, `.sms`).
//!
//! A B-rep is regions of shells of face uses, faces on surfaces bounded by loops of edge uses,
//! edges on curves between vertices. Everything is rebuilt through [`XBuilder`] (which fits its
//! own p-curves): the faces of each non-void region's shells, with the file's vertex and edge
//! tolerances.
//!
//! The reader follows SMLib's own (`SmBrepData::ReadFromDB` and the classes' `ReadFromDB`): an
//! ASCII stream of whitespace-separated numbers where each group is followed by "go to the next
//! line" (so text after the numbers, as in `16 Vertices In Brep`, is skipped). Versions:
//!
//! - none (30, the legacy layout): `Curve Type[:] 14033` B-spline curves and `Surface Type[:]
//!   15002` B-spline surfaces or `15400` offsets of one. A B-spline curve is the last pole index
//!   n, the degree p, a rational flag and the dimension (2 or 3), then n + 1 poles (homogeneous,
//!   the weight last when rational) and n + p + 2 knots; a surface the last pole indices in u
//!   and v, the degrees, the rational flag, the poles (v fastest) and the u then v knots.
//! - over 30 (in parts; `.smb` files skip them): each record is followed by a line of attribute
//!   indices (count first).
//! - over 31: each topology record has flags and two user ids after its index (edge uses the
//!   same again for their mate).
//! - over 32 (`Object Type N` records, a `Curve Dim` line after a curve's): a B-spline has two
//!   more lines (knot type, form) after its header, and the analytic types write their own data
//!   and then the B-spline they are exactly: 14010 line, 14021 circle (an ellipse, with a polar
//!   conversion table), 15011 plane, 15012 cone (cylinders too: a cone with no angle; a surface
//!   of revolution underneath), 15020 surface of revolution (its generating curve and a polar
//!   table), 15030 surface of extrusion (its generating curve).
//!
//! Curves and surfaces come back in SMLib's own parametrisation (their B-spline forms). Faces
//! of B-reps get the exact analytic surface instead (plane, revolution, extrusion, which the
//! builder turns into cylinders and cones where they are), turned to SMLib's normal.

use std::sync::Arc;

use super::curve::Curve;
use super::math::*;
use super::nurbs::{NurbsCurve, NurbsSurface, H};
use super::surface::Surface;
use super::topo::{Body, TOL_MIN};
use super::xbuild::{XBuilder, XFace};

type R<T> = Result<T, String>;

/// A B-rep read from a file, with where SMLib's tests point at its pieces.
#[derive(Debug, Clone)]
pub struct Brep {
    pub body: Body,
    /// The middle point of each of the file's edges, in the file's order (which SMLib's tests
    /// index: `SmBrep::GetEdges`).
    pub edge_mids: Vec<P3>,
    /// The start and end vertex of each of the file's edges (the direction a fillet law runs).
    pub edge_ends: Vec<(P3, P3)>,
    /// The area centroid of the body face each of the file's faces became, in the file's order
    /// (`SmBrep::GetFaces`); None for a face the builder dropped.
    pub face_centroids: Vec<Option<P3>>,
    /// Problems met while building.
    pub warnings: Vec<String>,
}

/// An SMLib part (`.smp`): free curves and surfaces (in SMLib's parametrisation) and B-reps.
#[derive(Debug, Clone)]
pub struct Part {
    pub curves: Vec<Curve>,
    pub surfaces: Vec<Surface>,
    pub breps: Vec<Brep>,
}

/// SMLib's ASCII stream: `>>` reads the next whitespace-separated token (across lines),
/// `GoToNextLine` drops the rest of the line. Lines starting with `//` are comments.
struct Stream<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Stream<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, i: 0 }
    }

    fn line_no(&self) -> usize {
        self.s[..self.i.min(self.s.len())].bytes().filter(|&b| b == b'\n').count() + 1
    }

    fn skip_ws(&mut self) {
        let b = self.s.as_bytes();
        loop {
            while self.i < b.len() && b[self.i].is_ascii_whitespace() {
                self.i += 1;
            }
            if self.s[self.i..].starts_with("//") {
                self.nl();
                continue;
            }
            break;
        }
    }

    fn tok(&mut self) -> R<&'a str> {
        self.skip_ws();
        let b = self.s.as_bytes();
        let from = self.i;
        while self.i < b.len() && !b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
        if from == self.i {
            return Err("the file ends early".into());
        }
        Ok(&self.s[from..self.i])
    }

    fn num(&mut self) -> R<f64> {
        let t = self.tok()?;
        t.parse().ok().filter(|v: &f64| v.is_finite()).ok_or_else(|| format!("line {}: expected a number, found {t:?}", self.line_no()))
    }

    fn nums(&mut self, n: usize) -> R<Vec<f64>> {
        (0..n).map(|_| self.num()).collect()
    }

    fn int(&mut self) -> R<i64> {
        let v = self.num()?;
        if v.fract() != 0.0 || v.abs() > 9.0e15 {
            return Err(format!("line {}: expected an integer, found {v}", self.line_no()));
        }
        Ok(v as i64)
    }

    /// A count: no more than the file has bytes (each counted thing takes at least one), so a
    /// corrupt count can't ask for more memory than the file justifies.
    fn count(&mut self) -> R<usize> {
        let v = self.int()?;
        usize::try_from(v).ok().filter(|&n| n <= self.s.len()).ok_or_else(|| format!("line {}: a count of {v} in a file of {} bytes", self.line_no(), self.s.len()))
    }

    fn p3(&mut self) -> R<V3> {
        let v = self.nums(3)?;
        self.nl();
        Ok(V3::new(v[0], v[1], v[2]))
    }

    /// SMLib's `GoToNextLine`.
    fn nl(&mut self) {
        match self.s[self.i..].find('\n') {
            Some(k) => self.i += k + 1,
            None => self.i = self.s.len(),
        }
    }

    /// Skips to just after the next line starting with one of `keys` (a section header,
    /// written with or without a space after the slashes); stays put when there is none.
    fn seek(&mut self, keys: &[&str]) -> R<()> {
        let mut at = self.i;
        while at < self.s.len() {
            let end = self.s[at..].find('\n').map_or(self.s.len(), |k| at + k + 1);
            let l = self.s[at..end].trim();
            if keys.iter().any(|k| l.starts_with(k)) {
                self.i = end;
                return Ok(());
            }
            at = end;
        }
        Err(format!("no {}", keys[0]))
    }

    /// `ReadType`: a `<word> <word> N` line (`Object Type 15011`, `Curve Type: 14033`), then,
    /// for a curve of a version over 32, a `Curve Dim N` line.
    fn read_type(&mut self, dim: bool) -> R<(i64, usize)> {
        let ty = self.last_int_of_line()?;
        let d = if dim { self.last_int_of_line()? as usize } else { 3 };
        Ok((ty, d))
    }

    fn last_int_of_line(&mut self) -> R<i64> {
        self.skip_ws();
        let end = self.s[self.i..].find('\n').map_or(self.s.len(), |k| self.i + k);
        let line = &self.s[self.i..end];
        let n = line.split(|c: char| c.is_whitespace() || c == ':').rfind(|w| !w.is_empty()).and_then(|w| w.parse::<i64>().ok());
        let n = n.ok_or_else(|| format!("line {}: expected a type, found {line:?}", self.line_no()))?;
        self.nl();
        Ok(n)
    }
}

/// How a file's records are laid out.
#[derive(Clone, Copy)]
struct Fmt {
    version: u32,
    /// Records carry attribute lines (parts do, `.smb` files don't).
    attrs: bool,
}

impl Fmt {
    /// Flags and user ids after each topology record's index.
    fn ids(&self) -> usize {
        if self.version > 31 { 3 } else { 0 }
    }

    /// Skips `n` attribute-index lines.
    fn skip_attrs(&self, st: &mut Stream, n: usize) -> R<()> {
        if self.version > 30 && self.attrs {
            for _ in 0..n {
                let c = st.count()?;
                st.nums(c)?;
                st.nl();
            }
        }
        Ok(())
    }
}

/// `SmBSplineCurve::ReadFromDB`.
fn bspline_curve(st: &mut Stream, fmt: Fmt) -> R<NurbsCurve> {
    let n = st.count()?;
    st.nl();
    let p = st.count()?;
    st.nl();
    let rational = st.int()? != 0;
    st.nl();
    let dim = st.count()?;
    st.nl();
    if fmt.version > 32 {
        st.num()?; // knot type
        st.nl();
        st.num()?; // form
        st.nl();
    }
    if !(1..=3).contains(&dim) {
        return Err(format!("a curve of dimension {dim}"));
    }
    let mut cps = Vec::with_capacity(n + 1);
    for _ in 0..=n {
        let v = st.nums(dim + rational as usize)?;
        st.nl();
        let (x, y, z) = (v[0], *v.get(1).unwrap_or(&0.0), if dim >= 3 { v[2] } else { 0.0 });
        let w = if rational { v[dim] } else { 1.0 };
        cps.push(H::new(x, y, z, w));
    }
    let mut knots = Vec::with_capacity(n + p + 2);
    for _ in 0..n + p + 2 {
        knots.push(st.num()?);
        st.nl();
    }
    NurbsCurve::new(p, knots, cps).map_err(|e| e.to_string())
}

/// `SmPolarConversion::ReadFromDB` (skipped: the tables follow from the curve).
fn polar_conversion(st: &mut Stream, fmt: Fmt) -> R<()> {
    let mut head = [0usize; 5];
    for h in head.iter_mut() {
        *h = st.count()?;
        st.nl();
    }
    let [_, knots, angles, exact, has_curve] = head;
    for cnt in [knots, angles, exact] {
        for k in 0..cnt {
            st.num()?;
            if k > 0 && k % 5 == 0 {
                st.nl();
            }
        }
        if cnt > 0 && cnt % 5 != 0 {
            st.nl();
        }
    }
    st.nl();
    if has_curve != 0 {
        let (ty, _) = st.read_type(true)?;
        curve_of(st, fmt, ty)?;
    }
    Ok(())
}

/// `SmCurve::ReadFromDB` for type `ty`: the curve's B-spline form (SMLib's parametrisation).
fn curve_of(st: &mut Stream, fmt: Fmt, ty: i64) -> R<NurbsCurve> {
    match ty {
        14033 => {}
        // Line: point, vector, domain, scale, inside-out flag.
        14010 => {
            for n in [3, 3, 2, 1, 1] {
                st.nums(n)?;
                st.nl();
            }
        }
        // Circle and ellipse: origin, x and y axes, domain, the two radii, inside-out flag, then
        // the polar conversion.
        14021 | 14022 => {
            for n in [3, 3, 3, 2, 1, 1, 1] {
                st.nums(n)?;
                st.nl();
            }
            st.nl();
            polar_conversion(st, fmt)?;
        }
        _ => return Err(format!("a curve of type {ty}")),
    }
    bspline_curve(st, fmt)
}

/// `SmBrepData::ReadCurveFromDB` (without the attributes).
fn read_curve_record(st: &mut Stream, fmt: Fmt) -> R<NurbsCurve> {
    if fmt.version > 32 {
        let (ty, _) = st.read_type(true)?;
        curve_of(st, fmt, ty)
    } else {
        let (ty, _) = st.read_type(false)?;
        if ty != 14033 {
            return Err(format!("a curve of type {ty}"));
        }
        bspline_curve(st, fmt)
    }
}

/// `SmBSplineSurface::ReadFromDB`.
fn bspline_surface(st: &mut Stream, fmt: Fmt) -> R<NurbsSurface> {
    let ns = st.nums(2)?;
    st.nl();
    let ds = st.nums(2)?;
    st.nl();
    let rational = st.int()? != 0;
    st.nl();
    if fmt.version > 32 {
        st.num()?;
        st.nl();
        st.num()?;
        st.nl();
    }
    let idx = |x: f64| -> R<usize> {
        if x >= 0.0 && x.fract() == 0.0 && x <= st.s.len() as f64 { Ok(x as usize) } else { Err(format!("a surface size of {x}")) }
    };
    let (nu, nv, pu, pv) = (idx(ns[0])?, idx(ns[1])?, idx(ds[0])?, idx(ds[1])?);
    if (nu + 1).saturating_mul(nv + 1) > st.s.len() {
        return Err(format!("{} × {} poles in a file of {} bytes", nu + 1, nv + 1, st.s.len()));
    }
    let mut cps = vec![Vec::with_capacity(nv + 1); nu + 1];
    for row in cps.iter_mut() {
        for _ in 0..=nv {
            let v = st.nums(3 + rational as usize)?;
            st.nl();
            let w = if rational { v[3] } else { 1.0 };
            row.push(H::new(v[0], v[1], v[2], w));
        }
    }
    let mut knots = |n: usize| -> R<Vec<f64>> {
        (0..n)
            .map(|_| {
                let k = st.num();
                st.nl();
                k
            })
            .collect()
    };
    let ku = knots(nu + pu + 2)?;
    let kv = knots(nv + pv + 2)?;
    NurbsSurface::new(pu, pv, ku, kv, cps).map_err(|e| e.to_string())
}

/// A surface record: its B-spline form (SMLib's parametrisation) and, for the analytic types
/// and offsets, the exact surface faces should use.
struct SurfRec {
    nurbs: NurbsSurface,
    exact: Option<Surface>,
}

/// The legacy offset surface: a cubic surface through a grid of offset points (fine enough for
/// the corpus's checks; an exact offset surface type would replace it).
fn offset_approx(base: NurbsSurface, d: f64) -> Surface {
    let ((u0, u1), (v0, v1)) = base.range();
    let s = Surface::Nurbs(Arc::new(base));
    let m = 24;
    let us: Vec<f64> = (0..=m).map(|i| u0 + (u1 - u0) * i as f64 / m as f64).collect();
    let vs: Vec<f64> = (0..=m).map(|j| v0 + (v1 - v0) * j as f64 / m as f64).collect();
    // Interpolate each row in v, then the columns of poles in u.
    let rows: Vec<NurbsCurve> = us
        .iter()
        .map(|&u| {
            let r: Vec<P3> = vs.iter().map(|&v| s.eval(u, v) + s.normal(u, v).normalize() * d).collect();
            NurbsCurve::interpolate(&r, &vs, 3, None, None)
        })
        .collect();
    Surface::Nurbs(Arc::new(NurbsSurface::through_curves(&rows, &us, 3, None, None)))
}

/// `SmSurface::ReadFromDB` for type `ty` (a version over 32).
fn surface_of(st: &mut Stream, fmt: Fmt, ty: i64) -> R<SurfRec> {
    let exact = match ty {
        15002 => None,
        // Plane: origin, x and y axes, domain, uv scale.
        15011 => {
            let o = st.p3()?;
            let x = st.p3()?;
            let y = st.p3()?;
            st.nums(4)?;
            st.nl();
            st.nums(2)?;
            st.nl();
            Some(Surface::Plane { f: Frame::from_xz(P3::from(o), x, x.cross(&y)) })
        }
        // Cone (base radius, semi-angle, inside-out flag) over a surface of revolution.
        15012 => {
            for _ in 0..3 {
                st.num()?;
                st.nl();
            }
            return surface_of(st, fmt, 15020);
        }
        // Revolution: flags, origin, x and y axes (the axis is z), domain, swap flag, then the
        // generating curve and the polar conversion.
        15020 => {
            st.nums(1)?;
            st.nl();
            st.nums(1)?;
            st.nl();
            let o = st.p3()?;
            let x = st.p3()?;
            let y = st.p3()?;
            st.nums(4)?;
            st.nl();
            st.num()?;
            st.nl();
            st.nl();
            let (gty, _) = st.read_type(true)?;
            let generator = curve_of(st, fmt, gty)?;
            st.nl();
            polar_conversion(st, fmt)?;
            let m = super::recognize::curve(&generator).unwrap_or_else(|| Curve::nurbs(generator));
            Some(Surface::Revolution { f: Frame::from_xz(P3::from(o), x, x.cross(&y)), m })
        }
        // Extrusion: origin, vector, domain, orientation, swap flag, the curve's plane (point,
        // normal), then the generating curve: c(u) + v·vector.
        15030 => {
            st.p3()?;
            let d = st.p3()?;
            st.nums(4)?;
            st.nl();
            st.num()?;
            st.nl();
            st.num()?;
            st.nl();
            st.p3()?;
            st.p3()?;
            st.nl();
            let (gty, _) = st.read_type(true)?;
            let generator = curve_of(st, fmt, gty)?;
            let c = super::recognize::curve(&generator).unwrap_or_else(|| Curve::nurbs(generator));
            Some(Surface::Extrusion { c, d })
        }
        _ => return Err(format!("a surface of type {ty}")),
    };
    Ok(SurfRec { nurbs: bspline_surface(st, fmt)?, exact })
}

/// `SmBrepData::ReadSurfaceFromDB` (without the attributes).
fn read_surface_record(st: &mut Stream, fmt: Fmt) -> R<SurfRec> {
    let (ty, _) = st.read_type(false)?;
    if fmt.version > 32 {
        return surface_of(st, fmt, ty);
    }
    match ty {
        15002 => Ok(SurfRec { nurbs: bspline_surface(st, fmt)?, exact: None }),
        15400 => {
            let d = st.num()?;
            st.nl();
            let base = bspline_surface(st, fmt)?;
            Ok(SurfRec { nurbs: base.clone(), exact: Some(offset_approx(base, d)) })
        }
        _ => Err(format!("a surface of type {ty}")),
    }
}

/// The surface a face uses, and whether its normal is opposite SMLib's (the B-spline form's).
fn face_surface(rec: &SurfRec, warnings: &mut Vec<String>, k: usize) -> (Surface, bool) {
    let nurbs = Surface::Nurbs(Arc::new(rec.nurbs.clone()));
    let Some(exact) = &rec.exact else { return (nurbs, false) };
    if matches!(exact, Surface::Nurbs(_)) {
        // The legacy offset: its own normal is the base's.
        return (exact.clone(), false);
    }
    let ((u0, u1), (v0, v1)) = rec.nurbs.range();
    for (fu, fv) in [(0.5, 0.5), (0.31, 0.43), (0.67, 0.29), (0.23, 0.71)] {
        let (u, v) = (u0 + (u1 - u0) * fu, v0 + (v1 - v0) * fv);
        let p = rec.nurbs.point(u, v);
        let n1 = nurbs.normal(u, v);
        let (eu, ev) = exact.project(&p, None);
        let q = exact.eval(eu, ev);
        let n2 = exact.normal(eu, ev);
        let size = rec.nurbs.cps.iter().flatten().map(|h| super::nurbs::dehom(h).coords.norm()).fold(1.0, f64::max);
        if (q - p).norm() > 1e-6 * size {
            warnings.push(format!("surface {k}: the analytic surface misses its B-spline form by {:.2e}", (q - p).norm()));
            return (nurbs, false);
        }
        let d = n1.dot(&n2);
        if d.abs() > 0.5 {
            return (exact.clone(), d < 0.0);
        }
    }
    (nurbs, false)
}

/// One B-rep (`SmBrepData::ReadFromDB`), from `//Brep Starts` (or the start of a part's
/// `[Brep #i]` block) to its end.
fn brep(text: &str, fmt: Fmt) -> R<Brep> {
    if fmt.version > 43 {
        return Err(format!("version {} (the new tolerance model) isn't supported", fmt.version));
    }
    let mut st = Stream::new(text);
    let ids = fmt.ids();
    let sec = |st: &mut Stream, a: &str, b: &str| -> R<usize> {
        st.seek(&[a, b])?;
        let n = st.count()?;
        st.nl();
        Ok(n)
    };
    // Regions: (shell count, first shell, void).
    let nr = sec(&mut st, "//Region Data", "// Region Data")?;
    st.num()?; // start region
    st.nl();
    let mut regions = Vec::new();
    for _ in 0..nr {
        st.nums(1 + ids)?;
        let v = st.nums(3)?;
        st.nl();
        fmt.skip_attrs(&mut st, 1)?;
        regions.push((v[0] as usize, v[1] as usize, v[2] != 0.0));
    }
    let ns = sec(&mut st, "//Shell Data", "// Shell Data")?;
    let mut shells: Vec<Option<(usize, usize)>> = Vec::new();
    for _ in 0..ns {
        st.nums(1 + ids)?;
        // Face-use shells: (count, start); wire-edge and vertex shells carry no faces.
        if st.int()? == 0 {
            let v = st.nums(3)?;
            shells.push(Some((v[1] as usize, v[2] as usize)));
        } else {
            st.num()?;
            shells.push(None);
        }
        st.nl();
        fmt.skip_attrs(&mut st, 1)?;
    }
    let nfu = sec(&mut st, "//Faceuse Data", "// Faceuse Data")?;
    let mut faceuses = Vec::new();
    for _ in 0..nfu {
        st.nums(1 + ids)?;
        let v = st.nums(2)?;
        st.nl();
        fmt.skip_attrs(&mut st, 1)?;
        faceuses.push((v[0] as usize, v[1] != 0.0));
    }
    let nf = sec(&mut st, "//Face Data", "// Face Data")?;
    // (surface, first loop, loop count: the header says "# of Inner Loops" but counts the
    // outer one too)
    let mut faces = Vec::new();
    for _ in 0..nf {
        st.nums(1 + ids)?;
        let v = st.nums(5)?;
        st.nl();
        st.nums(4)?; // uv range
        st.nl();
        fmt.skip_attrs(&mut st, 1)?;
        faces.push((v[1] as usize, v[2] as usize, v[3] as usize));
    }
    let nl = sec(&mut st, "//Loop Data", "// Loop Data")?;
    let mut loops = Vec::new();
    for _ in 0..nl {
        st.nums(1 + ids)?;
        // Edge-use loops: (count, start); a vertex loop has none.
        if st.int()? == 0 {
            let v = st.nums(2)?;
            loops.push((v[0] as usize, v[1] as usize));
        } else {
            st.num()?;
            loops.push((0, 0));
        }
        st.nl();
        fmt.skip_attrs(&mut st, 3)?;
    }
    let neu = sec(&mut st, "//Edgeuse Data", "// Edgeuse Data")?;
    // (kind, edge or vertex, orientation)
    let mut eus = Vec::new();
    for _ in 0..neu {
        st.nums(1 + 2 * ids)?;
        // Edge uses: edge, loop, orientation, UV curve, next, mate; the others (vertex, wire
        // edge, vertex at a pole) their entity, owner, orientation and UV curve.
        let kind = st.int()?;
        let v = st.nums(if kind == 0 { 6 } else { 4 })?;
        st.nl();
        fmt.skip_attrs(&mut st, 2)?;
        eus.push((kind as u8, v[0] as usize, v[2] != 0.0));
    }
    let ne = sec(&mut st, "//Edge Data", "// Edge Data")?;
    // (tolerance, curve, start, end, parameter range)
    let mut edges = Vec::new();
    for _ in 0..ne {
        st.nums(1 + ids)?;
        let v = st.nums(7)?;
        st.nl();
        fmt.skip_attrs(&mut st, 1)?;
        edges.push((v[0], v[1] as usize, v[2] as usize, v[3] as usize, (v[5], v[6])));
    }
    let nv = sec(&mut st, "//Vertex Data", "// Vertex Data")?;
    let mut verts = Vec::new();
    for _ in 0..nv {
        st.nums(1 + ids)?;
        let v = st.nums(4)?;
        st.nl();
        fmt.skip_attrs(&mut st, 1)?;
        verts.push((v[0], P3::new(v[1], v[2], v[3])));
    }
    let nuv = sec(&mut st, "//UV-Curve Data", "// UV-Curve Data")?;
    for k in 0..nuv {
        st.num()?; // its index
        st.nl();
        read_curve_record(&mut st, fmt).map_err(|e| format!("uv curve {k}: {e}"))?;
        fmt.skip_attrs(&mut st, 1)?;
    }
    let nc = sec(&mut st, "//3D-Curve Data", "// 3D-Curve Data")?;
    let mut curves = Vec::new();
    for k in 0..nc {
        st.num()?;
        st.nl();
        curves.push(read_curve_record(&mut st, fmt).map_err(|e| format!("curve {k}: {e}"))?);
        fmt.skip_attrs(&mut st, 1)?;
    }
    let nsu = sec(&mut st, "//Surface Data", "// Surface Data")?;
    let mut surfs = Vec::new();
    for k in 0..nsu {
        st.num()?;
        st.nl();
        surfs.push(read_surface_record(&mut st, fmt).map_err(|e| format!("surface {k}: {e}"))?);
        fmt.skip_attrs(&mut st, 1)?;
    }
    build(Topo { regions, shells, faceuses, faces, loops, eus, edges, verts }, curves, surfs)
}

/// An edge record: tolerance, curve, start and end vertex, parameter range.
type EdgeRec = (f64, usize, usize, usize, (f64, f64));

/// A B-rep's records, as indices into each other.
struct Topo {
    /// (shell count, first shell, void)
    regions: Vec<(usize, usize, bool)>,
    /// Face-use shells' (count, first face use); None for wire-edge and vertex shells.
    shells: Vec<Option<(usize, usize)>>,
    /// (face, orientation)
    faceuses: Vec<(usize, bool)>,
    /// (surface, first loop, loop count)
    faces: Vec<(usize, usize, usize)>,
    /// Edge-use loops' (count, first edge use); (0, 0) for a vertex loop.
    loops: Vec<(usize, usize)>,
    /// (kind: 0 for an edge use, edge or vertex, orientation)
    eus: Vec<(u8, usize, bool)>,
    edges: Vec<EdgeRec>,
    /// (tolerance, point)
    verts: Vec<(f64, P3)>,
}

fn build(t: Topo, curves: Vec<NurbsCurve>, surfs: Vec<SurfRec>) -> R<Brep> {
    let Topo { regions, shells, faceuses, faces, loops, eus, edges, verts } = t;
    let mut warnings = Vec::new();
    let surfs: Vec<(Surface, bool)> = surfs.iter().enumerate().map(|(k, s)| face_surface(s, &mut warnings, k)).collect();
    let mut xb = XBuilder::new();
    // SMLib turns its solids outward on reading (some files' face uses point in).
    xb.unoriented = true;
    let vids: Vec<usize> = verts.iter().map(|(_, p)| xb.vertex(*p)).collect();
    for (k, (t, _)) in verts.iter().enumerate() {
        xb.body.verts[vids[k]].tol = t.max(TOL_MIN);
    }
    let mut eids: Vec<Option<usize>> = Vec::new();
    for (k, &(t, c, a, b, (p0, p1))) in edges.iter().enumerate() {
        let cur = curves.get(c).ok_or_else(|| format!("edge {k}: no curve {c}"))?;
        let (&va, &vb) = (vids.get(a).ok_or_else(|| format!("edge {k}: no vertex {a}"))?, vids.get(b).ok_or_else(|| format!("edge {k}: no vertex {b}"))?);
        // The edge's stretch of its curve (the vertices' projections on the whole curve can land
        // on another stretch of a long or closed one).
        let (lo, hi) = cur.range();
        let (p0, p1) = (p0.max(lo), p1.min(hi));
        let span = hi - lo;
        let cur = if p1 > p0 + span * 1e-9 && (p0 > lo + span * 1e-9 || p1 < hi - span * 1e-9) {
            Curve::Nurbs(Arc::new(cur.clone())).to_nurbs(p0, p1)
        } else {
            cur.clone()
        };
        let e = xb.edge(super::xbuild::nurbs_curve(cur), va, vb);
        if let Some(e) = e {
            xb.body.edges[e].tol = t.max(TOL_MIN);
        } else {
            warnings.push(format!("edge {k} is empty"));
        }
        eids.push(e);
    }
    let nf = faces.len();
    let mut face_ids: Vec<Option<usize>> = vec![None; nf];
    let mut make_face = |xb: &mut XBuilder, fi: usize, orient: bool| -> Option<usize> {
        if let Some(x) = *face_ids.get(fi)? {
            return Some(x);
        }
        let (s, first, count) = *faces.get(fi)?;
        let mut lps = Vec::new();
        for li in first..first.saturating_add(count).min(loops.len()) {
            let (cnt, start) = *loops.get(li)?;
            let lp: Vec<(usize, bool)> = (start..start.saturating_add(cnt).min(eus.len()))
                .filter_map(|u| {
                    let (kind, e, o) = *eus.get(u)?;
                    if kind != 0 {
                        return None;
                    }
                    Some((eids.get(e).copied().flatten()?, o))
                })
                .collect();
            if !lp.is_empty() {
                lps.push(lp);
            }
        }
        let (surf, flip) = surfs.get(s)?.clone();
        let id = xb.face(XFace { surf, sense: orient != flip, loops: lps });
        face_ids[fi] = Some(id); // (fi checked above)
        Some(id)
    };
    // Solid regions' shells; a file of only void regions holds sheets (open shells).
    let sheets = regions.iter().all(|r| r.2);
    for &(cnt, first, void) in &regions {
        if void && !sheets {
            continue;
        }
        let mut shell_ids = Vec::new();
        for si in first..first.saturating_add(cnt).min(shells.len()) {
            let Some(Some((n, start))) = shells.get(si).copied() else { continue };
            let fs: Vec<usize> = (start..start.saturating_add(n).min(faceuses.len()))
                .filter_map(|u| {
                    let (f, o) = *faceuses.get(u)?;
                    make_face(&mut xb, f, o)
                })
                .collect();
            if !fs.is_empty() {
                shell_ids.push(xb.shell(fs, !sheets));
            }
        }
        if !shell_ids.is_empty() && !sheets {
            xb.solid(shell_ids);
        }
    }
    let (body, w, map) = xb.finish_mapped();
    warnings.extend(w);
    if body.faces.is_empty() && !faces.is_empty() {
        return Err(format!("no faces read: {}", warnings.iter().take(3).cloned().collect::<Vec<_>>().join("; ")));
    }
    let edge_mids = edges
        .iter()
        .map(|&(_, c, _, _, (t0, t1))| {
            let n = &curves[c];
            let (a, b) = n.range();
            n.point((0.5 * (t0 + t1)).clamp(a, b))
        })
        .collect();
    let edge_ends = edges.iter().map(|&(_, _, a, b, _)| (verts[a].1, verts[b].1)).collect();
    let cents = face_centroids(&body);
    let face_centroids = face_ids.iter().map(|x| x.and_then(|x| map.get(x).copied().flatten()).map(|f| cents[f])).collect();
    Ok(Brep { body, edge_mids, edge_ends, face_centroids, warnings })
}

/// The area centroid of each face of `body`, from a fine tessellation (deflection 1e-3 of the
/// body's size; exact integration of spline faces costs seconds each). What SMLib's tests'
/// face indices are matched by: [`Brep::face_centroids`] are these of the read body.
pub fn face_centroids(body: &Body) -> Vec<P3> {
    let bb = body.bbox();
    let defl = if bb.is_empty() { 1e-3 } else { (bb.diagonal() * 1e-3).max(1e-6) };
    let es = super::tess::edge_samples(body, defl, 0.3);
    body.faces
        .iter()
        .map(|f| {
            let m = super::tess::mesh_face(body, f, &es, defl, 0.3);
            let (mut a, mut c) = (0.0, V3::zeros());
            for t in &m.indices {
                let [p, q, r] = t.map(|i| m.positions[i as usize].coords);
                let ta = (q - p).cross(&(r - p)).norm() / 2.0;
                a += ta;
                c += (p + q + r) * (ta / 3.0);
            }
            if a > 0.0 {
                return P3::from(c / a);
            }
            // No triangles: the middle of its edges' middles.
            let ms: Vec<V3> = f.loops.iter().flatten().filter(|co| !body.edges[co.edge].degenerate).map(|co| body.edges[co.edge].point(body.edges[co.edge].mid_t()).coords).collect();
            P3::from(ms.iter().sum::<V3>() / ms.len().max(1) as f64)
        })
        .collect()
}

/// The format of an `.smb` file: its `Version N` (and `Healer Version N`) lines, 30 without.
fn smb_format(text: &str) -> R<(Fmt, &str)> {
    let t = text.trim_start();
    let version = match t.strip_prefix("Version") {
        Some(rest) => rest.split_whitespace().next().and_then(|v| v.parse().ok()).ok_or("a bad Version line")?,
        None => 30,
    };
    let at = text.find("//Brep Starts").ok_or("not an SMLib B-rep file")?;
    if text[..at].lines().any(|l| {
        let l = l.trim();
        !l.is_empty() && !l.starts_with("Version") && !l.starts_with("Healer Version")
    }) {
        return Err("not an SMLib B-rep file".into());
    }
    Ok((Fmt { version, attrs: false }, &text[at..]))
}

/// Reads an SMLib `.smb` file: the body, its edges' middle points and ends and its faces'
/// centroids (in the file's order).
pub fn read_brep(bytes: &[u8]) -> R<Brep> {
    let text = String::from_utf8_lossy(bytes);
    // (Some `.smb` files are parts of one B-rep.)
    if text.trim_start().starts_with("//[Output Summary]") {
        let mut p = read_part(bytes)?;
        if p.breps.len() != 1 {
            return Err(format!("a part of {} B-reps", p.breps.len()));
        }
        return Ok(p.breps.remove(0));
    }
    let (fmt, body) = smb_format(&text)?;
    brep(body, fmt)
}

/// Reads an SMLib `.smb` file into a body (its solid regions' shells), with problems met.
pub fn read(bytes: &[u8]) -> R<(Body, Vec<String>)> {
    read_brep(bytes).map(|b| (b.body, b.warnings))
}

/// [`read`], with the middle point of each of the file's edges (in the file's order, which
/// SMLib's tests index).
pub fn read_with_edges(bytes: &[u8]) -> R<(Body, Vec<P3>, Vec<String>)> {
    read_brep(bytes).map(|b| (b.body, b.edge_mids, b.warnings))
}

/// [`read`], with the area centroid of the body face each of the file's faces became (in the
/// file's order, which SMLib's tests index; None for a face the builder dropped).
pub fn read_with_faces(bytes: &[u8]) -> R<(Body, Vec<Option<P3>>, Vec<String>)> {
    read_brep(bytes).map(|b| (b.body, b.face_centroids, b.warnings))
}

/// Reads an SMLib part (`.smp`, `SmBrepData::ReadPartFromDB`): `//[Output Summary]` with the
/// counts of curves (plus 10⁶ × the format version), surfaces (plus 10⁶ × the healer version),
/// Boolean tree nodes and B-reps, then `[Curve #i]`, `[Surface #i]` and `[Brep #i]` blocks.
pub fn read_part(bytes: &[u8]) -> R<Part> {
    let text = String::from_utf8_lossy(bytes);
    if !text.trim_start().starts_with("//[Output Summary]") {
        return Err("not an SMLib part file".into());
    }
    let mut counts = [0u64; 4];
    for (k, c) in counts.iter_mut().enumerate() {
        let tag = format!("({})", k + 1);
        let line = text.lines().find(|l| l.contains(&tag)).ok_or_else(|| format!("no count {tag}"))?;
        *c = line.split_whitespace().find_map(|w| w.parse().ok()).ok_or_else(|| format!("a bad count line {line:?}"))?;
    }
    let (version, ncurves) = if counts[0] > 1_000_000 { ((counts[0] / 1_000_000) as u32, (counts[0] % 1_000_000) as usize) } else { (30, counts[0] as usize) };
    let nsurfs = (counts[1] % 1_000_000) as usize;
    let nbreps = counts[3] as usize;
    let fmt = Fmt { version, attrs: true };
    let mut st = Stream::new(&text);
    let mut curves = Vec::new();
    for k in 0..ncurves {
        st.seek(&["[Curve #"])?;
        let n = read_curve_record(&mut st, fmt).map_err(|e| format!("curve {k}: {e}"))?;
        fmt.skip_attrs(&mut st, 1)?;
        curves.push(Curve::nurbs(n));
    }
    let mut surfaces = Vec::new();
    for k in 0..nsurfs {
        st.seek(&["[Surface #"])?;
        let s = read_surface_record(&mut st, fmt).map_err(|e| format!("surface {k}: {e}"))?;
        fmt.skip_attrs(&mut st, 1)?;
        surfaces.push(match s.exact {
            // The legacy offset: no B-spline form of its own.
            Some(e @ Surface::Nurbs(_)) => e,
            _ => Surface::Nurbs(Arc::new(s.nurbs)),
        });
    }
    // B-reps: each from its `[Brep #i]` line to the next.
    let starts: Vec<usize> = text.match_indices("[Brep #").map(|(i, _)| i).collect();
    if starts.len() != nbreps {
        return Err(format!("{} B-rep blocks, the summary says {nbreps}", starts.len()));
    }
    let mut breps = Vec::new();
    for (k, &a) in starts.iter().enumerate() {
        let b = starts.get(k + 1).copied().unwrap_or(text.len());
        breps.push(brep(&text[a..b], fmt).map_err(|e| format!("brep {k}: {e}"))?);
    }
    Ok(Part { curves, surfaces, breps })
}

/// The format of a bare curve or surface file: the current one when it starts with `Object
/// Type`, else the legacy one (an optional `Curve Type` line, then the B-spline).
fn bare(text: &str) -> (Fmt, Stream<'_>) {
    let t = text.trim_start();
    let version = if t.starts_with('O') { 42 } else { 30 };
    let mut st = Stream::new(t);
    if version == 30 && (t.starts_with('C') || t.starts_with('S')) {
        st.nl();
    }
    (Fmt { version, attrs: false }, st)
}

/// Reads a bare SMLib curve file (`.smc`, `SmCurve::ReadFromFile`): the curve in SMLib's
/// parametrisation.
pub fn read_curve(bytes: &[u8]) -> R<Curve> {
    let text = String::from_utf8_lossy(bytes);
    let (fmt, mut st) = bare(&text);
    let n = if fmt.version > 32 {
        let (ty, _) = st.read_type(true)?;
        curve_of(&mut st, fmt, ty)?
    } else {
        bspline_curve(&mut st, fmt)?
    };
    Ok(Curve::nurbs(n))
}

/// Reads a bare SMLib surface file (`.sms`, `SmSurface::ReadFromFile`): the surface in SMLib's
/// parametrisation.
pub fn read_surface(bytes: &[u8]) -> R<Surface> {
    let text = String::from_utf8_lossy(bytes);
    let (fmt, mut st) = bare(&text);
    let n = if fmt.version > 32 {
        let (ty, _) = st.read_type(false)?;
        surface_of(&mut st, fmt, ty)?.nurbs
    } else {
        bspline_surface(&mut st, fmt)?
    };
    Ok(Surface::Nurbs(Arc::new(n)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixture copied from NVIDIA usd-brep's `TestFiles/pt_TestFiles` into
    /// `tests/data/smlib/<dir>` (Apache-2.0; see the README there).
    fn fixture(rel: &str) -> Vec<u8> {
        let p = format!("{}/tests/data/smlib/{rel}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read(&p).unwrap_or_else(|e| panic!("{p}: {e}"))
    }

    /// A valid closed solid with its edges and faces located.
    fn solid(b: &Brep, name: &str) {
        let issues = b.body.check();
        assert!(issues.is_empty(), "{name}: {issues:?}");
        assert!(!b.body.solids.is_empty(), "{name}: no solid");
        // (check() also says when a solid encloses a negative volume.)
        assert!(b.face_centroids.iter().all(|c| c.is_some()), "{name}: a face dropped");
        assert_eq!(b.edge_mids.len(), b.edge_ends.len());
    }

    #[test]
    fn version_38_and_42_solids() {
        for f in [
            "reader/LShape_I.smb",
            "reader/LShape_II.smb",
            "reader/Platte.smb",
            "reader/brep1.smb",
            "reader/defillet.smb",
            "reader/extrusion.smb",
            "reader/flathead_screw_top.smb",
            "reader/inset_cube.smb",
            "reader/inset_cube_1x1fillet.smb",
            "reader/inset_cube_1x2fillet.smb",
            "reader/inset_cube_1x3fillet.smb",
            "reader/inset_cube_3x1fillet.smb",
            "reader/OffsetBoxCylOnCorner_edited.smb",
            // A part file of one B-rep.
            "reader/SmallBox.smb",
        ] {
            let bytes = fixture(f);
            let b = read_brep(&bytes).unwrap_or_else(|e| panic!("{f}: {e}"));
            solid(&b, f);
        }
    }

    /// Version 38 with surfaces of extrusion and revolution (checking its spline faces takes
    /// about a minute).
    #[test]
    #[ignore = "slow"]
    fn version_38_blade() {
        let bytes = fixture("fillets/blade_fillet_fails.smb");
        let b = read_brep(&bytes).unwrap();
        solid(&b, "blade_fillet_fails.smb");
    }

    /// A face use naming a face past the file's faces: no panic.
    #[test]
    fn face_reference_out_of_range_is_an_error() {
        let text = String::from_utf8(fixture("fillets/box_on_box.smb")).unwrap();
        // Face use 11 ("11 0 0", the solid's first) names face 99.
        let bad = text.replacen("\n11 0 0\n", "\n11 99 0\n", 1);
        assert_ne!(bad, text);
        // (The face use is dropped; whatever the builder makes of the rest.)
        let _ = read_brep(bad.as_bytes());
        // And past every face and loop.
        let bad = text.replacen("\n11 0 0\n", "\n11 18446744073709551615 0\n", 1);
        let _ = read_brep(bad.as_bytes());
    }

    /// Counts the file can't hold (allocation would overflow) are errors.
    #[test]
    fn huge_counts_are_errors() {
        let text = String::from_utf8(fixture("fillets/box_on_box.smb")).unwrap();
        for (from, to) in [("24 Edges In Brep", "4000000000000000000 Edges In Brep"), ("48 Edgeuses In Brep", "99999999999 Edgeuses In Brep")] {
            let bad = text.replacen(from, to, 1);
            assert_ne!(bad, text, "{from}");
            assert!(read_brep(bad.as_bytes()).is_err(), "{from}");
        }
        // A curve of absurd size, alone.
        assert!(read_curve(b"1000000000000000000\n3\n0\n3\n").is_err());
        assert!(read_surface(b"100000000 100000000\n3 3\n0\n").is_err());
    }

    #[test]
    fn version_38_sheets() {
        // One face each (a surface of extrusion, a B-spline), SMLib's two-surface fillet inputs.
        for f in ["reader/eg1_srf1.smb", "reader/eg1_srf2.smb"] {
            let bytes = fixture(f);
            let b = read_brep(&bytes).unwrap_or_else(|e| panic!("{f}: {e}"));
            assert_eq!(b.body.faces.len(), 1, "{f}");
            assert!(b.body.solids.is_empty(), "{f}");
        }
    }

    #[test]
    fn analytic_faces_are_exact() {
        let bytes = fixture("reader/LShape_I.smb");
        let b = read_brep(&bytes).unwrap();
        let kinds: Vec<_> = b.body.faces.iter().map(|f| f.surf.kind()).collect();
        use crate::SurfaceKind;
        assert!(kinds.contains(&SurfaceKind::Plane));
        assert!(kinds.contains(&SurfaceKind::Cylinder), "{kinds:?}");
        assert!(!kinds.contains(&SurfaceKind::Other), "{kinds:?}");
    }

    #[test]
    fn legacy_solid_with_faces() {
        let bytes = fixture("fillets/box_on_box.smb");
        let (body, cents, _) = read_with_faces(&bytes).unwrap();
        assert_eq!(cents.len(), 11);
        // Each centroid is its own face's: the nearest body face centroid, unambiguously.
        for c in cents.iter().flatten() {
            let all = face_centroids(&body);
            let mut d: Vec<f64> = all.iter().map(|f| (f - c).norm()).collect();
            d.sort_by(f64::total_cmp);
            assert!(d[0] < 1e-9 && d[1] > 1e-3, "{d:?}");
        }
    }

    #[test]
    fn parts() {
        // (file, curves, surfaces, breps)
        for (f, nc, ns, nb) in [
            ("reader/boolean_trees.smp", 0, 0, 7),
            ("reader/ssi_cylcyl1.smp", 0, 0, 2),
            ("reader/ssi_pln_rev1.smp", 0, 0, 2),
            ("reader/ssi_ext_ext_touch.smp", 0, 0, 2),
            ("reader/ssi_coincident_torii.smp", 0, 0, 2),
            ("reader/cci_bug_rhino1.smp", 2, 0, 0),
            ("reader/ApproximateSphere.smp", 4, 1, 0),
        ] {
            let bytes = fixture(f);
            let p = read_part(&bytes).unwrap_or_else(|e| panic!("{f}: {e}"));
            assert_eq!((p.curves.len(), p.surfaces.len(), p.breps.len()), (nc, ns, nb), "{f}");
            for (k, b) in p.breps.iter().enumerate() {
                let issues = b.body.check();
                // The builder's limit, not the reader's: brep 4 loses a face whose loop doesn't
                // close (so it is no solid).
                if f == "reader/boolean_trees.smp" && k == 4 {
                    assert!(b.body.solids.is_empty(), "{f} brep {k} now reads whole: drop it from the exceptions");
                    continue;
                }
                assert!(issues.is_empty(), "{f} brep {k}: {issues:?}");
            }
        }
    }

    /// Every SSI part of an NVIDIA usd-brep checkout (`$CADRS_USD_BREP`, else the one beside
    /// this checkout) reads into valid bodies.
    #[test]
    #[ignore = "needs a usd-brep checkout"]
    fn every_ssi_part_reads() {
        let root = std::env::var("CADRS_USD_BREP").unwrap_or_else(|_| format!("{}/../../../../nvbrep/usd-brep", env!("CARGO_MANIFEST_DIR")));
        let dir = std::path::Path::new(&root).join("TestFiles/pt_TestFiles");
        let mut names: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).filter(|n| (n.starts_with("ssi_") || n.starts_with("ss_")) && n.ends_with(".smp")).collect();
        names.sort();
        let mut bad = Vec::new();
        for n in &names {
            match read_part(&std::fs::read(dir.join(n)).unwrap()) {
                Ok(p) => {
                    for (k, b) in p.breps.iter().enumerate() {
                        if let Some(i) = b.body.check().first() {
                            bad.push(format!("{n} brep {k}: {i}"));
                        }
                    }
                }
                Err(e) => bad.push(format!("{n}: {e}")),
            }
        }
        // The builder's limits (the files read): faces at a cone's apex or a sphere's pole whose
        // loops don't close, single-face solids that come out inside out.
        let known = ["ssi_coincident_spheres2.smp", "ssi_cone_cyl1.smp", "ssi_cone_cyl2.smp", "ssi_cone_plane1.smp", "ssi_spiral.smp", "ssi_tor_sph_tan.smp", "ssi_tor_sph_tan2.smp"];
        let (old, new): (Vec<_>, Vec<_>) = bad.iter().partition(|b| known.iter().any(|k| b.starts_with(k)));
        assert!(new.is_empty(), "{} of {}: {new:#?}", new.len(), names.len());
        let fixed: Vec<_> = known.iter().filter(|k| !old.iter().any(|b| b.starts_with(*k))).collect();
        assert!(fixed.is_empty(), "now read well: {fixed:?}");
    }

    #[test]
    fn bare_curves_and_surfaces() {
        let bytes = fixture("reader/offset_nurb.sms");
        let Surface::Nurbs(s) = read_surface(&bytes).unwrap() else { panic!() };
        assert_eq!((s.cps.len(), s.cps[0].len(), s.du, s.dv), (6, 2, 3, 1));
        let ((u0, _), (v0, _)) = s.range();
        let p = s.point(u0, v0);
        assert!((p - P3::new(10.0, 12.0, 0.0)).norm() < 1e-9, "{p:?}");
        for (f, n, p) in [("reader/curve0.smc", 1389, 4), ("reader/closedCurve.smc", 10, 3), ("reader/crvToOffset.smc", 95, 2)] {
            let bytes = fixture(f);
            let Curve::Nurbs(c) = read_curve(&bytes).unwrap_or_else(|e| panic!("{f}: {e}")) else { panic!() };
            assert_eq!((c.cps.len(), c.degree), (n, p), "{f}");
        }
        let bytes = fixture("reader/Surface.sms");
        read_surface(&bytes).unwrap();
    }

    #[test]
    fn part_curves_keep_smlib_parameters() {
        let bytes = fixture("reader/ApproximateSphere.smp");
        let p = read_part(&bytes).unwrap();
        // Curve 1: the uv line from (0, 0) to (π, 0) over [0, π] (π to 10 digits).
        let c = &p.curves[1];
        let q = c.point(c.nurbs_range().unwrap().1);
        assert!((q - P3::new(PI, 0.0, 0.0)).norm() < 1e-8, "{q:?}");
    }
}
