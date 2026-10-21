//! Splitting a face by cut edges: a planar arrangement in the face's (u, v), traced into loops
//! with the material on the left, holes nested into the smallest loop around them.

use super::curve::Curve;
use super::math::*;
use super::surface::Surface;

/// A directed piece of an edge in a face: the edge (index in the caller's pool), its direction,
/// its p-curve (parametrized by the edge's t) and the edge's parameter range.
#[derive(Debug, Clone)]
pub struct Arc {
    pub edge: usize,
    pub fwd: bool,
    pub pc: Curve,
    pub t: [f64; 2],
    /// Start and end vertices in the direction of travel.
    pub v: [usize; 2],
    pub cut: bool,
    /// A degenerate edge (a pole): one vertex, a p-curve along the pole line.
    pub deg: bool,
}

impl Arc {
    fn ts(&self) -> f64 {
        if self.fwd { self.t[0] } else { self.t[1] }
    }
    fn te(&self) -> f64 {
        if self.fwd { self.t[1] } else { self.t[0] }
    }
    pub fn uv_at(&self, s: f64) -> P2 {
        let t = self.ts() + (self.te() - self.ts()) * s;
        let p = self.pc.point(t);
        P2::new(p.x, p.y)
    }
    pub fn twin(&self) -> Arc {
        Arc { fwd: !self.fwd, v: [self.v[1], self.v[0]], ..self.clone() }
    }
}

pub struct SplitResult {
    /// Each face: loops of arcs, outer first.
    pub faces: Vec<Vec<Vec<Arc>>>,
}

/// Splits the region bounded by `boundary` (loops of arcs, as the face has them) by `cuts`
/// (each used both ways). `flip`: the face's sense is reversed (its loops run clockwise in uv).
pub fn split_face(surf: &Surface, flip: bool, boundary: &[Vec<Arc>], cuts: &[Arc]) -> SplitResult {
    let (mu, mv) = surf.metric();
    let sgn = if flip { -1.0 } else { 1.0 };
    // Oriented, scaled uv (material on the left for every loop).
    let o = |p: P2| P2::new(p.x * mu, p.y * mv * sgn);
    let mut arcs: Vec<Arc> = Vec::new();
    // Degenerate boundary arcs are split where cuts arrive on them (a cut ending at a pole).
    let cut_ends: Vec<(usize, P2)> = cuts.iter().flat_map(|c| [(c.v[0], c.uv_at(0.0)), (c.v[1], c.uv_at(1.0))]).collect();
    for a in boundary.iter().flatten() {
        if !a.deg {
            arcs.push(a.clone());
            continue;
        }
        let (u0, u1) = (a.uv_at(0.0), a.uv_at(1.0));
        let span = (u1 - u0).norm();
        let mut fr: Vec<f64> = cut_ends
            .iter()
            .filter(|(v, _)| *v == a.v[0])
            .filter_map(|(_, q)| {
                if span < 1e-300 {
                    return None;
                }
                let s = (q - u0).dot(&(u1 - u0)) / (span * span);
                let off = (u0 + (u1 - u0) * s - q).norm();
                (s > 1e-9 && s < 1.0 - 1e-9 && off < 1e-6 * (1.0 + span)).then_some(s)
            })
            .collect();
        fr.sort_by(f64::total_cmp);
        fr.dedup_by(|x, y| (*x - *y).abs() < 1e-9);
        if fr.is_empty() {
            arcs.push(a.clone());
            continue;
        }
        let (ts, te) = if a.fwd { (a.t[0], a.t[1]) } else { (a.t[1], a.t[0]) };
        let mut bounds = vec![ts];
        bounds.extend(fr.iter().map(|s| ts + (te - ts) * s));
        bounds.push(te);
        for w in bounds.windows(2) {
            let (lo, hi) = if a.fwd { (w[0], w[1]) } else { (w[1], w[0]) };
            arcs.push(Arc { t: [lo, hi], ..a.clone() });
        }
    }
    for c in cuts {
        arcs.push(Arc { cut: true, ..c.clone() });
        arcs.push(Arc { cut: true, ..c.twin() });
    }
    // Nodes: (vertex, uv).
    let size = {
        let mut bb = Bbox::empty();
        for a in &arcs {
            let p = o(a.uv_at(0.0));
            bb.add(&P3::new(p.x, p.y, 0.0));
        }
        bb.diagonal().max(1e-9)
    };
    let ntol = (size * 1e-7).max(1e-7);
    let mut nodes: Vec<(usize, P2)> = Vec::new();
    let node_of = |v: usize, p: P2, nodes: &mut Vec<(usize, P2)>| -> usize {
        if let Some(i) = nodes.iter().position(|(nv, np)| *nv == v && (np - p).norm() < ntol * 100.0) {
            return i;
        }
        nodes.push((v, p));
        nodes.len() - 1
    };
    let mut ends: Vec<[usize; 2]> = Vec::with_capacity(arcs.len());
    for a in &arcs {
        let s = node_of(a.v[0], o(a.uv_at(0.0)), &mut nodes);
        let e = node_of(a.v[1], o(a.uv_at(1.0)), &mut nodes);
        ends.push([s, e]);
    }
    // Prune dangling cut arcs (a cut end touching nothing else) repeatedly.
    let mut alive = vec![true; arcs.len()];
    loop {
        let mut deg = vec![0usize; nodes.len()];
        for (k, e) in ends.iter().enumerate() {
            if alive[k] {
                deg[e[0]] += 1;
                deg[e[1]] += 1;
            }
        }
        let mut changed = false;
        for k in 0..arcs.len() {
            if !alive[k] || !arcs[k].cut {
                continue;
            }
            // A cut pair contributes 2 to each end's degree; a dangling end has only those.
            let (s, e) = (ends[k][0], ends[k][1]);
            if (deg[s] <= 2 && s != e) || (deg[e] <= 2 && s != e) {
                alive[k] = false;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    if std::env::var("CADRS_SPLIT_DEBUG").is_ok() {
        for (k, a) in arcs.iter().enumerate() {
            eprintln!("arc {k}: edge {} fwd {} cut {} alive {} nodes {:?} uv {:?} -> {:?}", a.edge, a.fwd, a.cut, alive[k], ends[k], a.uv_at(0.0), a.uv_at(1.0));
        }
    }
    // Departure direction of each arc at its start, and arrival direction at its end.
    let dir_at = |a: &Arc, s: f64| -> V2 {
        let p0 = o(a.uv_at(s));
        let e = if s < 0.5 { 1e-4 } else { -1e-4 };
        let p1 = o(a.uv_at(s + e));
        let d = (p1 - p0) * if s < 0.5 { 1.0 } else { -1.0 };
        let l = d.norm();
        if l > 0.0 { d / l } else { d }
    };
    let dep: Vec<V2> = arcs.iter().map(|a| dir_at(a, 0.0)).collect();
    let arr: Vec<V2> = arcs.iter().map(|a| dir_at(a, 1.0)).collect();
    // Secondary directions (further along) for ties.
    let dep2: Vec<V2> = arcs.iter().map(|a| {
        let p0 = o(a.uv_at(0.0));
        let p1 = o(a.uv_at(0.05));
        let d = p1 - p0;
        if d.norm() > 0.0 { d.normalize() } else { d }
    }).collect();
    let mut out_at: Vec<Vec<usize>> = vec![Vec::new(); nodes.len()];
    for k in 0..arcs.len() {
        if alive[k] {
            out_at[ends[k][0]].push(k);
        }
    }
    let next = |k: usize| -> Option<usize> {
        let n = ends[k][1];
        let din = arr[k];
        let mut best: Option<(f64, f64, usize)> = None;
        for &c in &out_at[n] {
            let d = dep[c];
            // Turning angle from the incoming direction, in (−π, π]; the reverse is π.
            let mut ang = cross2(&din, &d).atan2(din.dot(&d));
            let is_twin = arcs[c].edge == arcs[k].edge && arcs[c].fwd != arcs[k].fwd && ends[c][1] == ends[k][0];
            if is_twin {
                ang = -10.0; // only if nothing else
            }
            let d2 = dep2[c];
            let ang2 = cross2(&din, &d2).atan2(din.dot(&d2));
            let key = (ang, ang2);
            if best.is_none_or(|(a, b, _)| key.0 > a + 1e-9 || ((key.0 - a).abs() <= 1e-9 && key.1 > b)) {
                best = Some((key.0, key.1, c));
            }
        }
        best.map(|(_, _, c)| c)
    };
    let mut used = vec![false; arcs.len()];
    let mut loops: Vec<Vec<usize>> = Vec::new();
    for s in 0..arcs.len() {
        if used[s] || !alive[s] {
            continue;
        }
        let mut lp = vec![s];
        used[s] = true;
        let mut cur = s;
        let mut ok = true;
        loop {
            let Some(nx) = next(cur) else {
                ok = false;
                break;
            };
            if nx == s {
                break;
            }
            if used[nx] {
                ok = false;
                break;
            }
            used[nx] = true;
            lp.push(nx);
            cur = nx;
            if lp.len() > arcs.len() {
                ok = false;
                break;
            }
        }
        if ok {
            loops.push(lp);
        }
    }
    // Areas and polygons in oriented uv.
    let poly = |lp: &[usize]| -> Vec<P2> {
        let mut pts = Vec::new();
        for &k in lp {
            let n = match arcs[k].pc {
                Curve::Line { .. } => 1,
                _ => 48,
            };
            for i in 0..n {
                pts.push(o(arcs[k].uv_at(i as f64 / n as f64)));
            }
        }
        pts
    };
    let mut outers: Vec<(Vec<usize>, Vec<P2>, f64)> = Vec::new();
    let mut holes: Vec<(Vec<usize>, Vec<P2>)> = Vec::new();
    for lp in loops {
        // Drop loops made only of a cut and its twin (slivers).
        let pts = poly(&lp);
        let a = polygon_area(&pts);
        if a.abs() < size * size * 1e-14 {
            continue;
        }
        if a > 0.0 {
            outers.push((lp, pts, a));
        } else {
            holes.push((lp, pts));
        }
    }
    let mut faces: Vec<Vec<Vec<Arc>>> = outers.iter().map(|(lp, _, _)| vec![lp.iter().map(|&k| arcs[k].clone()).collect()]).collect();
    for (lp, pts) in holes {
        // A point just beside the hole, on its material side (left of its direction).
        let a0 = &arcs[lp[0]];
        let pm = o(a0.uv_at(0.5));
        let pn = o(a0.uv_at(0.5 + 1e-3));
        let d = pn - pm;
        let d = if d.norm() > 0.0 { d.normalize() } else { d };
        let q = pm + V2::new(-d.y, d.x) * (size * 1e-6);
        let _ = &pts;
        let mut best: Option<(f64, usize)> = None;
        for (i, (_, op, a)) in outers.iter().enumerate() {
            if point_in_polygon(&q, op) && best.is_none_or(|(ba, _)| *a < ba) {
                best = Some((*a, i));
            }
        }
        if let Some((_, i)) = best {
            faces[i].push(lp.iter().map(|&k| arcs[k].clone()).collect());
        }
    }
    SplitResult { faces }
}
