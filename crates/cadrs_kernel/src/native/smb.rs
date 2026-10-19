//! SMLib's ASCII B-rep files (`.smb`, as NVIDIA's usd-brep writes them: `//Brep Starts`):
//! regions of shells of face uses, faces on NURBS surfaces (or offsets of them) bounded by loops
//! of edge uses, edges on NURBS curves between vertices. Everything is rebuilt through
//! [`XBuilder`] (which fits its own p-curves): the faces of each non-void region's shells, with
//! the file's vertex and edge tolerances.
//!
//! Records: a curve is `Curve Type[:] 14033`, then the last pole index n, the degree p, a
//! rational flag and the dimension (2 or 3), then n + 1 poles (homogeneous, the weight last when
//! rational) and n + p + 2 knots. A surface is `Surface Type: 15002`, then the last pole indices
//! in u and v, the degrees, the rational flag, the poles (v fastest) and the u then v knots;
//! type 15400 is an offset of such a surface (its distance first), approximated here by a
//! surface through offset points.

use std::sync::Arc;

use super::math::*;
use super::nurbs::{NurbsCurve, NurbsSurface, H};
use super::surface::Surface;
use super::topo::{Body, TOL_MIN};
use super::xbuild::{XBuilder, XFace};

type R<T> = Result<T, String>;

struct Lines<'a> {
    ls: Vec<&'a str>,
    i: usize,
    /// Numbers read from a line and not yet taken (records wrap across lines).
    buf: std::collections::VecDeque<f64>,
}

impl<'a> Lines<'a> {
    fn next(&mut self) -> R<&'a str> {
        self.buf.clear();
        while self.i < self.ls.len() {
            let l = self.ls[self.i].trim();
            self.i += 1;
            if !l.is_empty() && !l.starts_with("//") {
                return Ok(l);
            }
        }
        Err("the file ends early".into())
    }
    /// Skips to the line starting with `key` (a section header); its own line is consumed.
    /// Skips to the line starting with one of `keys` (a section header, written with or
    /// without a space after the slashes); its own line is consumed. Stays put when none is
    /// found.
    fn seek(&mut self, keys: &[&str]) -> R<()> {
        self.buf.clear();
        let from = self.i;
        while self.i < self.ls.len() {
            let l = self.ls[self.i].trim();
            self.i += 1;
            if keys.iter().any(|k| l.starts_with(k)) {
                return Ok(());
            }
        }
        self.i = from;
        Err(format!("no {}", keys[0]))
    }
    /// The next `n` numbers, across lines (each line's leading numbers; text after them, as in
    /// `16 Vertices In Brep`, ends the line's).
    fn take(&mut self, n: usize) -> R<Vec<f64>> {
        while self.buf.len() < n {
            let line = loop {
                if self.i >= self.ls.len() {
                    return Err("the file ends early".into());
                }
                let l = self.ls[self.i].trim();
                self.i += 1;
                if !l.is_empty() && !l.starts_with("//") {
                    break l;
                }
            };
            let before = self.buf.len();
            self.buf.extend(line.split_whitespace().map_while(|w| w.parse::<f64>().ok()));
            if self.buf.len() == before {
                return Err(format!("expected numbers, found {line:?}"));
            }
        }
        Ok(self.buf.drain(..n).collect())
    }
    fn count(&mut self) -> R<usize> {
        let n = self.take(1)?[0];
        self.buf.clear();
        Ok(n as usize)
    }
}

fn curve(l: &mut Lines) -> R<NurbsCurve> {
    let head = l.next()?;
    if !head.starts_with("Curve Type") || !head.ends_with("14033") {
        return Err(format!("a curve of type {head}"));
    }
    let n = l.count()?;
    let p = l.count()?;
    let rational = l.count()? != 0;
    let dim = l.count()?;
    let mut cps = Vec::with_capacity(n + 1);
    if !(1..=3).contains(&dim) {
        return Err(format!("a curve of dimension {dim}"));
    }
    for _ in 0..=n {
        let v = l.take(dim + rational as usize)?;
        let (x, y, z) = (v[0], *v.get(1).unwrap_or(&0.0), if dim >= 3 { v[2] } else { 0.0 });
        let w = if rational { v[dim] } else { 1.0 };
        cps.push(H::new(x, y, z, w));
    }
    let knots = l.take(n + p + 2)?;
    Ok(NurbsCurve { degree: p, knots, cps })
}

fn surface(l: &mut Lines) -> R<Surface> {
    let head = l.next()?;
    let offset = if head.ends_with("15400") {
        Some(l.take(1)?[0])
    } else if head.ends_with("15002") {
        None
    } else {
        return Err(format!("a surface of type {head}"));
    };
    let ns = l.take(2)?;
    let ds = l.take(2)?;
    let (nu, nv, pu, pv) = (ns[0] as usize, ns[1] as usize, ds[0] as usize, ds[1] as usize);
    let rational = l.count()? != 0;
    let mut cps = vec![Vec::with_capacity(nv + 1); nu + 1];
    for row in cps.iter_mut() {
        for _ in 0..=nv {
            let v = l.take(3 + rational as usize)?;
            let w = if rational { v[3] } else { 1.0 };
            row.push(H::new(v[0], v[1], v[2], w));
        }
    }
    let ku = l.take(nu + pu + 2)?;
    let kv = l.take(nv + pv + 2)?;
    let base = NurbsSurface { du: pu, dv: pv, ku, kv, cps };
    let Some(d) = offset else { return Ok(Surface::Nurbs(Arc::new(base))) };
    // The offset: a cubic surface through a grid of offset points (fine enough for the
    // corpus's checks; an exact offset surface type would replace it).
    let s = Surface::Nurbs(Arc::new(base));
    let ((u0, u1), (v0, v1)) = match &s {
        Surface::Nurbs(n) => n.range(),
        _ => unreachable!(),
    };
    let m = 24;
    let pts: Vec<Vec<P3>> = (0..=m)
        .map(|i| {
            let u = u0 + (u1 - u0) * i as f64 / m as f64;
            (0..=m)
                .map(|j| {
                    let v = v0 + (v1 - v0) * j as f64 / m as f64;
                    s.eval(u, v) + s.normal(u, v).normalize() * d
                })
                .collect()
        })
        .collect();
    let us: Vec<f64> = (0..=m).map(|i| u0 + (u1 - u0) * i as f64 / m as f64).collect();
    let vs: Vec<f64> = (0..=m).map(|j| v0 + (v1 - v0) * j as f64 / m as f64).collect();
    // Interpolate each row in v, then the columns of poles in u.
    let rows: Vec<NurbsCurve> = pts.iter().map(|r| NurbsCurve::interpolate(r, &vs, 3, None, None)).collect();
    Ok(Surface::Nurbs(Arc::new(NurbsSurface::through_curves(&rows, &us, 3, None, None))))
}

/// Reads an SMLib `.smb` file into a body (its solid regions' shells), with problems met.
pub fn read(bytes: &[u8]) -> R<(Body, Vec<String>)> {
    read_with_edges(bytes).map(|(b, _, w)| (b, w))
}

/// [`read`], with the middle point of each of the file's edges (in the file's order, which
/// SMLib's tests index).
pub fn read_with_edges(bytes: &[u8]) -> R<(Body, Vec<P3>, Vec<String>)> {
    let text = String::from_utf8_lossy(bytes);
    // (Newer files put a `Version N` line first.)
    let head = text.trim_start();
    let head = if head.starts_with("Version") { head.split_once('\n').map_or("", |x| x.1) } else { head };
    if !head.trim_start().starts_with("//Brep Starts") {
        return Err("not an SMLib B-rep file".into());
    }
    let mut l = Lines { ls: text.lines().collect(), i: 0, buf: Default::default() };
    // Regions: (shell count, first shell, void).
    l.seek(&["//Region Data", "// Region Data"])?;
    let nr = l.count()?;
    let _start = l.next()?;
    let mut regions = Vec::new();
    for _ in 0..nr {
        let v = l.take(4)?;
        regions.push((v[1] as usize, v[2] as usize, v[3] != 0.0));
    }
    l.seek(&["//Shell Data", "// Shell Data"])?;
    let ns = l.count()?;
    let mut shells: Vec<Option<(usize, usize)>> = Vec::new();
    for _ in 0..ns {
        let h = l.take(2)?;
        // Face-use shells: (count, start); wire-edge and vertex shells carry no faces.
        if h[1] == 0.0 {
            let v = l.take(3)?;
            shells.push(Some((v[1] as usize, v[2] as usize)));
        } else {
            l.take(1)?;
            shells.push(None);
        }
    }
    l.seek(&["//Faceuse Data", "// Faceuse Data"])?;
    let nfu = l.count()?;
    let mut faceuses = Vec::new();
    for _ in 0..nfu {
        let v = l.take(3)?;
        faceuses.push((v[1] as usize, v[2] != 0.0));
    }
    l.seek(&["//Face Data", "// Face Data"])?;
    let nf = l.count()?;
    // (surface, first loop, loop count: the header says "# of Inner Loops" but counts the
    // outer one too)
    let mut faces = Vec::new();
    for _ in 0..nf {
        let v = l.take(6)?;
        let _range = l.take(4)?;
        faces.push((v[2] as usize, v[3] as usize, v[4] as usize));
    }
    l.seek(&["//Loop Data", "// Loop Data"])?;
    let nl = l.count()?;
    let mut loops = Vec::new();
    for _ in 0..nl {
        let h = l.take(2)?;
        // Edge-use loops: (count, start); a vertex loop has none.
        if h[1] == 0.0 {
            let v = l.take(2)?;
            loops.push((v[0] as usize, v[1] as usize));
        } else {
            l.take(1)?;
            loops.push((0, 0));
        }
    }
    l.seek(&["//Edgeuse Data", "// Edgeuse Data"])?;
    let neu = l.count()?;
    // (kind, edge or vertex, orientation)
    let mut eus = Vec::new();
    for _ in 0..neu {
        let h = l.take(2)?;
        // Edge uses: edge, loop, orientation, UV curve, next, mate; the others (vertex, wire
        // edge, vertex at a pole) their entity, owner, orientation and UV curve.
        let v = l.take(if h[1] == 0.0 { 6 } else { 4 })?;
        eus.push((h[1] as u8, v[0] as usize, v[2] != 0.0));
    }
    l.seek(&["//Edge Data", "// Edge Data"])?;
    let ne = l.count()?;
    // (tolerance, curve, start, end, parameter range)
    let mut edges = Vec::new();
    for _ in 0..ne {
        let v = l.take(8)?;
        edges.push((v[1], v[2] as usize, v[3] as usize, v[4] as usize, (*v.get(6).unwrap_or(&0.0), *v.get(7).unwrap_or(&1.0))));
    }
    l.seek(&["//Vertex Data", "// Vertex Data"])?;
    let nv = l.count()?;
    let mut verts = Vec::new();
    for _ in 0..nv {
        let v = l.take(5)?;
        verts.push((v[1], P3::new(v[2], v[3], v[4])));
    }
    l.seek(&["//UV-Curve Data", "// UV-Curve Data"])?;
    let nuv = l.count()?;
    for _ in 0..nuv {
        let _label = l.next()?;
        curve(&mut l)?;
    }
    l.seek(&["//3D-Curve Data", "// 3D-Curve Data"])?;
    let nc = l.count()?;
    let mut curves = Vec::new();
    for _ in 0..nc {
        let _label = l.next()?;
        curves.push(curve(&mut l)?);
    }
    l.seek(&["//Surface Data", "// Surface Data"])?;
    let nsu = l.count()?;
    let mut surfs = Vec::new();
    for _ in 0..nsu {
        let _label = l.next()?;
        surfs.push(surface(&mut l)?);
    }
    // Build.
    let mut xb = XBuilder::new();
    let vids: Vec<usize> = verts.iter().map(|(_, p)| xb.vertex(*p)).collect();
    for (k, (t, _)) in verts.iter().enumerate() {
        xb.body.verts[vids[k]].tol = t.max(TOL_MIN);
    }
    let mut eids: Vec<Option<usize>> = Vec::new();
    let mut warnings = Vec::new();
    for (k, &(t, c, a, b, (p0, p1))) in edges.iter().enumerate() {
        let cur = curves.get(c).ok_or_else(|| format!("edge {k}: no curve {c}"))?;
        // The edge's stretch of its curve (the vertices' projections on the whole curve can land
        // on another stretch of a long or closed one).
        let (lo, hi) = cur.range();
        let (p0, p1) = (p0.max(lo), p1.min(hi));
        let span = hi - lo;
        let cur = if p1 > p0 + span * 1e-9 && (p0 > lo + span * 1e-9 || p1 < hi - span * 1e-9) {
            super::curve::Curve::Nurbs(std::sync::Arc::new(cur.clone())).to_nurbs(p0, p1)
        } else {
            cur.clone()
        };
        let e = xb.edge(super::xbuild::nurbs_curve(cur), vids[a], vids[b]);
        if let Some(e) = e {
            xb.body.edges[e].tol = t.max(TOL_MIN);
        } else {
            warnings.push(format!("edge {k} is empty"));
        }
        eids.push(e);
    }
    let mut face_ids: Vec<Option<usize>> = vec![None; nf];
    let mut make_face = |xb: &mut XBuilder, fi: usize, orient: bool| -> Option<usize> {
        if let Some(x) = face_ids[fi] {
            return Some(x);
        }
        let (s, first, count) = faces[fi];
        let mut lps = Vec::new();
        for li in first..first + count {
            let (cnt, start) = *loops.get(li)?;
            let lp: Vec<(usize, bool)> = (start..start + cnt).filter_map(|u| {
                let (kind, e, o) = *eus.get(u)?;
                if kind != 0 {
                    return None;
                }
                Some((eids.get(e).copied().flatten()?, o))
            }).collect();
            if !lp.is_empty() {
                lps.push(lp);
            }
        }
        let id = xb.face(XFace { surf: surfs.get(s)?.clone(), sense: orient, loops: lps });
        face_ids[fi] = Some(id);
        Some(id)
    };
    // Solid regions' shells; a file of only void regions holds sheets (open shells).
    let sheets = regions.iter().all(|r| r.2);
    for &(cnt, first, void) in &regions {
        if void && !sheets {
            continue;
        }
        let mut shell_ids = Vec::new();
        for si in first..first + cnt {
            let Some(Some((n, start))) = shells.get(si).copied() else { continue };
            let fs: Vec<usize> = (start..start + n).filter_map(|u| {
                let (f, o) = *faceuses.get(u)?;
                make_face(&mut xb, f, o)
            }).collect();
            if !fs.is_empty() {
                shell_ids.push(xb.shell(fs, !sheets));
            }
        }
        if !shell_ids.is_empty() && !sheets {
            xb.solid(shell_ids);
        }
    }
    let (body, w) = xb.finish();
    warnings.extend(w);
    if body.faces.is_empty() {
        return Err("no faces read".into());
    }
    let mids = edges
        .iter()
        .map(|&(_, c, _, _, (t0, t1))| {
            let n = &curves[c];
            let (a, b) = n.range();
            n.point((0.5 * (t0 + t1)).clamp(a, b))
        })
        .collect();
    Ok((body, mids, warnings))
}
