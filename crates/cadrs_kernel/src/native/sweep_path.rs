//! Sweeps along paths (P3.7, PS19). The path's curves are chained end to end; where the
//! profile's plane crosses the path the sweep starts (both ways when that is part way along an
//! open path; a closed path is restarted there). Each path segment is swept exactly: a line
//! is a prism of the profile as it is placed there, an arc a revolve about the arc's axis; the
//! profile's placement is carried along rigidly, and the pieces are united (the caps at the
//! joints cancel). Other curves (and arcs the profile isn't square to) are swept with
//! sampled frames (see `sweep_general`).

use super::curve::Curve;
use super::features::{face_as_profile, Tagged};
use super::kernel::NativeKernel;
use super::math::*;
use super::profile::{self, MergedFace, Piece};
use super::sweep::{self, Prism, Revolve};
use crate::*;

/// A path segment in the direction the path runs: its curve on [t0, t1], run backwards when
/// `rev`.
#[derive(Debug, Clone)]
pub struct Seg {
    pub c: Curve,
    pub t0: f64,
    pub t1: f64,
    pub rev: bool,
}

impl Seg {
    pub fn start(&self) -> P3 {
        self.c.point(if self.rev { self.t1 } else { self.t0 })
    }
    pub fn end(&self) -> P3 {
        self.c.point(if self.rev { self.t0 } else { self.t1 })
    }
    /// Unit tangent in the direction of travel at fraction s.
    pub fn dir(&self, s: f64) -> V3 {
        let t = if self.rev { self.t1 + (self.t0 - self.t1) * s } else { self.t0 + (self.t1 - self.t0) * s };
        self.c.tangent(t).normalize() * if self.rev { -1.0 } else { 1.0 }
    }
    pub fn length(&self) -> f64 {
        self.c.length(self.t0, self.t1)
    }
    /// The piece of the segment between fractions a and b of its travel.
    pub fn part(&self, a: f64, b: f64) -> Seg {
        let at = |s: f64| if self.rev { self.t1 + (self.t0 - self.t1) * s } else { self.t0 + (self.t1 - self.t0) * s };
        let (x, y) = (at(a), at(b));
        Seg { c: self.c.clone(), t0: x.min(y), t1: x.max(y), rev: self.rev }
    }
    pub fn reversed(&self) -> Seg {
        Seg { rev: !self.rev, ..self.clone() }
    }
    /// The travel fraction of a point on the segment.
    pub fn frac_of(&self, p: &P3) -> f64 {
        let t = self.c.closest(p, self.t0, self.t1);
        let f = (t - self.t0) / (self.t1 - self.t0);
        if self.rev { 1.0 - f } else { f }
    }
}

const PATH_JOIN: f64 = 1e-5;
const MIN_PATH_EDGE: f64 = 1e-4;

/// The path's curves as segments (not yet ordered).
pub fn path_segments(k: &NativeKernel, path: &[PathCurve]) -> Result<Vec<Seg>> {
    let mut out = Vec::new();
    for pc in path {
        match pc {
            PathCurve::Edge { body, edge } => {
                let b = k.get(*body)?;
                let e = b.edges.get(edge.0 as usize).ok_or_else(|| KernelError::InvalidParameter(format!("unknown {edge:?}")))?;
                if e.degenerate {
                    continue;
                }
                out.push(Seg { c: e.curve.clone(), t0: e.t[0], t1: e.t[1], rev: false });
            }
            PathCurve::Sketch { plane, curve } => {
                let p = profile::piece_of(curve, profile::Src::Curve { region: 0, curve: 0 })?;
                let m = super::build::plane_map(plane, &V3::zeros(), 0.0);
                out.push(Seg { c: p.c.transformed(&m), t0: p.t0, t1: p.t1, rev: p.rev });
            }
            PathCurve::Bezier3 { poles, weights } => {
                let w = if weights.len() == poles.len() { weights.clone() } else { vec![1.0; poles.len()] };
                let n = super::nurbs::NurbsCurve::bezier(poles, &w);
                out.push(Seg { c: Curve::Nurbs(std::sync::Arc::new(n)), t0: 0.0, t1: 1.0, rev: false });
            }
        }
    }
    Ok(out)
}

/// Orders and orients the segments into one chain; whether it closes.
pub fn chain(mut segs: Vec<Seg>) -> Result<(Vec<Seg>, bool)> {
    if segs.len() > 1 {
        segs.retain(|s| s.length() >= MIN_PATH_EDGE);
    }
    if segs.is_empty() {
        return Err(KernelError::InvalidParameter("Select a sweep path".into()));
    }
    let n = segs.len();
    let touches = |p: &P3, skip: usize, segs: &[Seg]| -> usize {
        (0..n).filter(|&j| j != skip).map(|j| ((segs[j].start() - p).norm() < PATH_JOIN) as usize + ((segs[j].end() - p).norm() < PATH_JOIN) as usize).sum()
    };
    let mut first = (0usize, false);
    for (i, s) in segs.iter().enumerate() {
        if (s.start() - s.end()).norm() < PATH_JOIN {
            continue;
        }
        if touches(&s.start(), i, &segs) == 0 {
            first = (i, false);
            break;
        }
        if touches(&s.end(), i, &segs) == 0 {
            first = (i, true);
            break;
        }
    }
    let mut used = vec![false; n];
    let mut out = vec![if first.1 { segs[first.0].reversed() } else { segs[first.0].clone() }];
    used[first.0] = true;
    loop {
        let at = out.last().unwrap().end();
        let next = (0..n).filter(|&j| !used[j]).find_map(|j| {
            if (segs[j].start() - at).norm() < PATH_JOIN {
                Some((j, false))
            } else if (segs[j].end() - at).norm() < PATH_JOIN {
                Some((j, true))
            } else {
                None
            }
        });
        let Some((j, rev)) = next else { break };
        used[j] = true;
        out.push(if rev { segs[j].reversed() } else { segs[j].clone() });
    }
    if used.iter().any(|u| !u) {
        return Err(KernelError::InvalidParameter("The sweep path must be one chain of connected curves".into()));
    }
    let closed = (out.last().unwrap().end() - out[0].start()).norm() < PATH_JOIN;
    Ok((out, closed))
}

/// The spines: one (the path from the profile) or two (forwards, and backwards from the
/// profile), each with whether it runs backwards.
fn spines(path: Vec<Seg>, closed: bool, plane: &Plane, center: &P3) -> Vec<(Vec<Seg>, bool)> {
    let o = plane.origin;
    let n = plane.normal.into_inner();
    let pl = super::surface::Surface::Plane { f: super::build::plane_frame(plane, V3::zeros()) };
    // Where the plane crosses the path, nearest the profile's centre.
    let mut best: Option<(f64, usize, f64)> = None;
    for (i, s) in path.iter().enumerate() {
        for t in super::ssi::curve_surface(&s.c, s.t0, s.t1, &pl) {
            let p = s.c.point(t);
            let _ = o;
            let d = (p - center).norm();
            if best.map_or(true, |(bd, ..)| d < bd) {
                let f = (t - s.t0) / (s.t1 - s.t0);
                best = Some((d, i, if s.rev { 1.0 - f } else { f }));
            }
        }
    }
    let _ = n;
    let Some((_, i, f)) = best else { return vec![(path, false)] };
    let tol = 1e-9;
    if i == 0 && f < tol {
        return vec![(path, false)];
    }
    let last = path.len() - 1;
    if !closed && i == last && f > 1.0 - tol {
        let back: Vec<Seg> = path.iter().rev().map(|s| s.reversed()).collect();
        return vec![(back, false)];
    }
    // Split there.
    let mut before: Vec<Seg> = path[..i].to_vec();
    let mut after: Vec<Seg> = Vec::new();
    if f < tol {
        after.extend(path[i..].iter().cloned());
    } else if f > 1.0 - tol {
        before.push(path[i].clone());
        after.extend(path[i + 1..].iter().cloned());
    } else {
        before.push(path[i].part(0.0, f));
        after.push(path[i].part(f, 1.0));
        after.extend(path[i + 1..].iter().cloned());
    }
    if closed {
        let mut all = after;
        all.extend(before);
        return vec![(all, false)];
    }
    let back: Vec<Seg> = before.iter().rev().map(|s| s.reversed()).collect();
    let mut out = Vec::new();
    if !after.is_empty() {
        out.push((after, false));
    }
    if !back.is_empty() {
        out.push((back, true));
    }
    out
}

/// The profile's centre (where the path should meet it).
fn profile_center(faces: &[MergedFace], chains: &[Vec<Piece>], plane: &Plane) -> P3 {
    let m = super::build::plane_map(plane, &V3::zeros(), 0.0);
    let mut acc = V3::zeros();
    let mut n = 0.0;
    for p in faces.iter().flat_map(|f| std::iter::once(&f.outer).chain(f.holes.iter())).flatten().chain(chains.iter().flatten()) {
        for k in 0..8 {
            let q = p.at(k as f64 / 8.0);
            acc += m.point(&P3::new(q.x, q.y, 0.0)).coords;
            n += 1.0;
        }
    }
    if n == 0.0 { plane.origin } else { P3::from(acc / n) }
}

/// A rigid placement of the profile plane.
fn move_plane(p: &Plane, m: &Affine) -> Plane {
    Plane {
        origin: m.point(&p.origin),
        x_dir: nalgebra::Unit::new_normalize(m.vector(&p.x_dir)),
        normal: nalgebra::Unit::new_normalize(m.vector(&p.normal)),
    }
}

/// Sweeps the profile (faces and chains in `plane` coordinates) along one spine.
fn sweep_spine(faces: &[MergedFace], chains: &[Vec<Piece>], plane: Plane, spine: &[Seg], control: SweepControl, solid: bool) -> Result<Vec<Tagged>> {
    let mut place = plane;
    let mut out = Vec::new();
    // Segments swept with sampled frames, gathered while they meet tangentially (one surface
    // per profile curve along the run).
    let mut run: Vec<Seg> = Vec::new();
    let flush = |run: &mut Vec<Seg>, place: &mut Plane, out: &mut Vec<Tagged>| -> Result<()> {
        if run.is_empty() {
            return Ok(());
        }
        let (b, end) = super::sweep_general::sweep_seg(faces, chains, *place, run, control, solid)?;
        out.push(Tagged::from_built(b));
        *place = end;
        run.clear();
        Ok(())
    };
    for seg in spine {
        let exact = match (&seg.c, control) {
            (Curve::Line { .. }, SweepControl::None | SweepControl::KeepOrientation | SweepControl::LockDirection(_)) => true,
            (Curve::Circle { c, x, y, .. }, SweepControl::None) => {
                let axis = x.cross(y).normalize();
                // The axis must lie in the profile's plane (where it will be when the run ends:
                // only checked once the run is flushed, so be conservative).
                run.is_empty() && axis.dot(&place.normal).abs() <= 1e-7 && (c - place.origin).dot(&place.normal).abs() <= 1e-6
            }
            _ => false,
        };
        if !exact {
            if let Some(last) = run.last() {
                // A break in tangent, or a quarter turn gathered: a new piece.
                let turned: f64 = run.iter().map(|g| g.dir(0.0).angle(&g.dir(1.0))).sum::<f64>() + seg.dir(0.0).angle(&seg.dir(1.0));
                if last.dir(1.0).dot(&seg.dir(0.0)) < 1.0 - 1e-9 || turned > PI / 2.0 + 1e-9 {
                    flush(&mut run, &mut place, &mut out)?;
                }
            }
            run.push(seg.clone());
            continue;
        }
        flush(&mut run, &mut place, &mut out)?;
        match &seg.c {
            Curve::Line { d, .. } => {
                let dir = d.normalize() * if seg.rev { -1.0 } else { 1.0 };
                let len = seg.length();
                let kind = Prism { plane: place, dir };
                // The profile plane may sit anywhere along the line: sweep from it.
                let b = sweep::sweep_full(faces, chains, &kind, 0.0, len, false, solid)?;
                out.push(Tagged::from_built(b));
                place = move_plane(&place, &Affine::translation(dir * len));
            }
            Curve::Circle { c, x, y, .. } => {
                // Turn about the arc's axis, counter-clockwise about z.
                let axis = x.cross(y).normalize() * if seg.rev { -1.0 } else { 1.0 };
                let span = seg.t1 - seg.t0;
                let start = seg.start();
                let radial = (start - c).normalize();
                let f = Frame { o: *c, x: radial, y: axis.cross(&radial), z: axis };
                let kind = Revolve { plane: place, f };
                let whole = span >= TAU - 1e-9;
                let b = sweep::sweep_full(faces, chains, &kind, 0.0, if whole { TAU } else { span }, whole, solid)?;
                out.push(Tagged::from_built(b));
                let r = nalgebra::Rotation3::from_axis_angle(&nalgebra::Unit::new_normalize(axis), span).into_inner();
                place = move_plane(&place, &Affine { m: r, t: c.coords - r * c.coords });
            }
            _ => unreachable!(),
        }
    }
    flush(&mut run, &mut place, &mut out)?;
    Ok(out)
}

fn swap_caps(t: &mut Tagged) {
    for ts in &mut t.tags {
        for x in ts.iter_mut() {
            *x = match *x {
                Origin::StartCap { region } => Origin::EndCap { region },
                Origin::EndCap { region } => Origin::StartCap { region },
                o => o,
            };
        }
    }
}

pub fn sweep_with(k: &mut NativeKernel, profile: &Profile, spec: &SweepSpec) -> Result<OpResult> {
    let segs = path_segments(k, &spec.path)?;
    let (path, closed) = chain(segs)?;
    let path_len = path.len();
    let (used, solid) = match spec.body {
        BodyKind::Thin { left, right } => (crate::thin::thin_profile(profile, left, right)?, true),
        BodyKind::Solid => (profile.clone(), true),
        BodyKind::Surface => (profile.clone(), false),
    };
    let regions = profile::regions(&used)?;
    let mut faces = if regions.is_empty() { Vec::new() } else { profile::merge_regions(&regions)? };
    let mut plane = profile.plane;
    if !spec.faces.is_empty() {
        if !solid {
            return Err(KernelError::InvalidParameter("Surface sweeps take sketch curves, not faces".into()));
        }
        if profile.regions.is_empty() {
            // The first face's own plane.
            let info = k.faces(spec.faces[0].body)?.into_iter().find(|i| i.id == spec.faces[0].face);
            if let Some(pl) = info.and_then(|i| i.plane) {
                plane = pl;
            }
        }
        let p2 = Profile::new(plane, vec![]);
        for f in &spec.faces {
            faces.push(face_as_profile(k, &p2, f)?);
        }
    }
    let chains: Vec<Vec<Piece>> = if solid {
        Vec::new()
    } else {
        let mut out = Vec::new();
        for (i, ch) in used.chains.iter().enumerate() {
            let region = ch.source.unwrap_or((used.regions.len() + i) as u64);
            let mut ps = Vec::new();
            for (ci, c) in ch.curves.iter().enumerate() {
                ps.push(profile::piece_of(c, profile::Src::Curve { region, curve: c.source().unwrap_or(unsourced_curve(0, ci)) })?);
            }
            out.push(profile::chain_loop_open(ps)?);
        }
        out
    };
    if faces.is_empty() && chains.is_empty() {
        return Err(KernelError::InvalidProfile("the profile has nothing to sweep".into()));
    }
    let center = profile_center(&faces, &chains, &plane);
    let prof = std::env::var("CADRS_PROFILE").is_ok();
    let t0 = std::time::Instant::now();
    let mut pieces: Vec<Tagged> = Vec::new();
    for (spine, backward) in spines(path, closed, &plane, &center) {
        let mut these = sweep_spine(&faces, &chains, plane, &spine, spec.control, solid)?;
        if backward {
            these.iter_mut().for_each(swap_caps);
        }
        pieces.extend(these);
    }
    if prof {
        eprintln!("sweep: {} pieces swept in {:?} (path {} segments)", pieces.len(), t0.elapsed(), path_len);
    }
    let t1 = std::time::Instant::now();
    // Join the pieces: the caps at the joints are glued away (pieces that meet anywhere else,
    // a two-way sweep's halves, are united).
    let mut acc = pieces.remove(0);
    for p in pieces {
        acc = if solid {
            match glue(&acc, &p) {
                Some(g) => g,
                None => super::features::tagged_union(&acc, &p)?,
            }
        } else {
            compound(acc, p)
        };
    }
    // A closed path: the last piece's end cap meets the first piece's start cap.
    if solid {
        if let Some(g) = glue_self(&acc) {
            acc = g;
        }
    }
    if prof {
        eprintln!("sweep: joined in {:?}", t1.elapsed());
    }
    Ok(acc.into_result(k))
}

/// Faces of `t` that are caps (`end`: end caps, else start caps), with their area centroids.
fn caps(t: &Tagged, end: bool) -> Vec<(usize, P3, f64)> {
    (0..t.body.faces.len())
        .filter(|&f| t.tags[f].iter().any(|o| if end { matches!(o, Origin::EndCap { .. }) } else { matches!(o, Origin::StartCap { .. }) }))
        .map(|f| {
            let (a, c) = super::mass::face_area(&t.body, f);
            (f, c, a)
        })
        .collect()
}

/// Glues `b` onto `a` where an end cap of `a` and a start cap of `b` coincide: both caps go,
/// their edges become one.
fn glue(a: &Tagged, b: &Tagged) -> Option<Tagged> {
    let ea = caps(a, true);
    let sb = caps(b, false);
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for (fa, ca, aa) in &ea {
        for (fb, cb, ab) in &sb {
            if (ca - cb).norm() < 1e-6 && (aa - ab).abs() < 1e-6 * (1.0 + aa) {
                pairs.push((*fa, *fb));
            }
        }
    }
    if pairs.is_empty() {
        return None;
    }
    let mut body = a.body.clone();
    let (vo, eo, fo) = body.append(&b.body);
    let _ = vo;
    let drop: Vec<usize> = pairs.iter().flat_map(|&(fa, fb)| [fa, fb + fo]).collect();
    // Identify b's cap edges with a's.
    let mut emap: std::collections::HashMap<usize, (usize, bool)> = Default::default();
    let mut vmap: std::collections::HashMap<usize, usize> = Default::default();
    for &(fa, fb) in &pairs {
        let ea_list: Vec<usize> = body.faces[fa].loops.iter().flatten().map(|c| c.edge).collect();
        for c in body.faces[fb + fo].loops.iter().flatten() {
            let e = &body.edges[c.edge];
            let m = e.point(e.mid_t());
            let found = ea_list.iter().copied().find(|&x| {
                let o = &body.edges[x];
                let om = o.point(o.mid_t());
                (om - m).norm() < 1e-6
                    && ((body.verts[o.v[0]].p - body.verts[e.v[0]].p).norm() < 1e-6 || (body.verts[o.v[0]].p - body.verts[e.v[1]].p).norm() < 1e-6)
            });
            let x = found?;
            let o = &body.edges[x];
            let same = (body.verts[o.v[0]].p - body.verts[e.v[0]].p).norm() < 1e-6 && (o.v[0] != o.v[1] || o.curve.tangent(o.mid_t()).dot(&e.curve.tangent(e.mid_t())) > 0.0);
            emap.insert(c.edge, (x, same));
            for k in 0..2 {
                let ov = if same { o.v[k] } else { o.v[1 - k] };
                vmap.insert(e.v[k], ov);
            }
        }
    }
    let _ = eo;
    // Repoint b's faces (re-parametrizing the p-curves onto a's edges).
    let nf = body.faces.len();
    for fi in fo..nf {
        if drop.contains(&fi) {
            continue;
        }
        let surf = body.faces[fi].surf.clone();
        for li in 0..body.faces[fi].loops.len() {
            for ci in 0..body.faces[fi].loops[li].len() {
                let c = body.faces[fi].loops[li][ci].clone();
                if let Some(&(x, same)) = emap.get(&c.edge) {
                    let old = body.edges[c.edge].clone();
                    let new = body.edges[x].clone();
                    let fwd = if same { c.fwd } else { !c.fwd };
                    // Keep the p-curve's chart: align its start with the old coedge's start.
                    let (ot0, _) = super::topo::co_range(&old, c.fwd);
                    let sp = c.pc.point(ot0);
                    let mut pc = super::pcurve::pcurve(&surf, &new.curve, new.t[0], new.t[1], None);
                    let (nt0, _) = super::topo::co_range(&new, fwd);
                    let q = pc.point(nt0);
                    let du = if surf.u_periodic() { ((sp.x - q.x) / TAU).round() * TAU } else { 0.0 };
                    let dv = if surf.v_periodic() { ((sp.y - q.y) / TAU).round() * TAU } else { 0.0 };
                    if du != 0.0 || dv != 0.0 {
                        pc = pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)));
                    }
                    body.faces[fi].loops[li][ci] = super::topo::Coedge { edge: x, fwd, pc };
                }
            }
        }
    }
    // Merge the vertices of b's other edges.
    for e in body.edges.iter_mut() {
        for k in 0..2 {
            if let Some(&v) = vmap.get(&e.v[k]) {
                e.v[k] = v;
            }
        }
    }
    let mut tags: Vec<Vec<Origin>> = a.tags.clone();
    tags.extend(b.tags.iter().cloned());
    let keep: Vec<usize> = (0..nf).filter(|f| !drop.contains(f)).collect();
    body.faces = keep.iter().map(|&f| body.faces[f].clone()).collect();
    let tags = keep.iter().map(|&f| tags[f].clone()).collect();
    super::unify::compact(&mut body);
    super::boolean::assemble_shells(&mut body).ok()?;
    if body.solids.len() != 1 {
        return None;
    }
    Some(Tagged { body, tags })
}

/// A closed sweep: glues its own end cap to its start cap.
fn glue_self(t: &Tagged) -> Option<Tagged> {
    let e = caps(t, true);
    let s = caps(t, false);
    for (fe, ce, ae) in &e {
        for (fs, cs, as_) in &s {
            if (ce - cs).norm() < 1e-6 && (ae - as_).abs() < 1e-6 * (1.0 + ae) {
                // Split into a body without the two caps and glue through a copy.
                let _ = (fe, fs);
                let mut a = t.clone();
                let mut b = Tagged { body: super::topo::Body::default(), tags: vec![] };
                std::mem::swap(&mut a, &mut b);
                let _ = a;
                return glue_within(&b, *fe, *fs);
            }
        }
    }
    None
}

/// Removes two coincident faces of one body and identifies their edges.
fn glue_within(t: &Tagged, fa: usize, fb: usize) -> Option<Tagged> {
    let mut body = t.body.clone();
    let ea_list: Vec<usize> = body.faces[fa].loops.iter().flatten().map(|c| c.edge).collect();
    let mut emap: std::collections::HashMap<usize, (usize, bool)> = Default::default();
    let mut vmap: std::collections::HashMap<usize, usize> = Default::default();
    for c in body.faces[fb].loops.iter().flatten() {
        let e = &body.edges[c.edge];
        let m = e.point(e.mid_t());
        let x = ea_list.iter().copied().find(|&x| {
            let o = &body.edges[x];
            (o.point(o.mid_t()) - m).norm() < 1e-6
        })?;
        let o = &body.edges[x];
        let same = (body.verts[o.v[0]].p - body.verts[e.v[0]].p).norm() < 1e-6 && (o.v[0] != o.v[1] || o.curve.tangent(o.mid_t()).dot(&e.curve.tangent(e.mid_t())) > 0.0);
        emap.insert(c.edge, (x, same));
        for k in 0..2 {
            vmap.insert(e.v[k], if same { o.v[k] } else { o.v[1 - k] });
        }
    }
    let nf = body.faces.len();
    for fi in 0..nf {
        if fi == fa || fi == fb {
            continue;
        }
        let surf = body.faces[fi].surf.clone();
        for li in 0..body.faces[fi].loops.len() {
            for ci in 0..body.faces[fi].loops[li].len() {
                let c = body.faces[fi].loops[li][ci].clone();
                if let Some(&(x, same)) = emap.get(&c.edge) {
                    if x == c.edge {
                        continue;
                    }
                    let old = body.edges[c.edge].clone();
                    let new = body.edges[x].clone();
                    let fwd = if same { c.fwd } else { !c.fwd };
                    let (ot0, _) = super::topo::co_range(&old, c.fwd);
                    let sp = c.pc.point(ot0);
                    let mut pc = super::pcurve::pcurve(&surf, &new.curve, new.t[0], new.t[1], None);
                    let (nt0, _) = super::topo::co_range(&new, fwd);
                    let q = pc.point(nt0);
                    let du = if surf.u_periodic() { ((sp.x - q.x) / TAU).round() * TAU } else { 0.0 };
                    let dv = if surf.v_periodic() { ((sp.y - q.y) / TAU).round() * TAU } else { 0.0 };
                    if du != 0.0 || dv != 0.0 {
                        pc = pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)));
                    }
                    body.faces[fi].loops[li][ci] = super::topo::Coedge { edge: x, fwd, pc };
                }
            }
        }
    }
    for e in body.edges.iter_mut() {
        for k in 0..2 {
            if let Some(&v) = vmap.get(&e.v[k]) {
                e.v[k] = v;
            }
        }
    }
    let keep: Vec<usize> = (0..nf).filter(|&f| f != fa && f != fb).collect();
    body.faces = keep.iter().map(|&f| body.faces[f].clone()).collect();
    let tags = keep.iter().map(|&f| t.tags[f].clone()).collect();
    super::unify::compact(&mut body);
    super::boolean::assemble_shells(&mut body).ok()?;
    if body.solids.len() != 1 {
        return None;
    }
    Some(Tagged { body, tags })
}

fn compound(mut a: Tagged, b: Tagged) -> Tagged {
    a.body.append(&b.body);
    a.tags.extend(b.tags);
    a
}
