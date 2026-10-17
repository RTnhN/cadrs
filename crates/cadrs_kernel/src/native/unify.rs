//! Merging faces that lie on the same surface and share edges (OCCT's UnifySameDomain), and
//! edges that continue each other through a vertex only they use.

use super::curve::Curve;
use super::math::*;
use super::pcurve;
use super::surface::Surface;
use super::topo::{co_range, Body, Coedge, Edge, Face};

/// p-curve start point (uv) of a coedge.
fn co_uv(b: &Body, c: &Coedge, end: bool) -> P2 {
    let (t0, t1) = co_range(&b.edges[c.edge], c.fwd);
    let p = c.pc.point(if end { t1 } else { t0 });
    P2::new(p.x, p.y)
}

/// The p-curve of an edge on `s`, shifted by periods so its start (in the coedge's direction)
/// is at `at`.
fn pc_from(s: &Surface, e: &Edge, fwd: bool, at: Option<P2>) -> Curve {
    let pc = pcurve::pcurve(s, &e.curve, e.t[0], e.t[1], None);
    let Some(at) = at else { return pc };
    let (t0, _) = co_range(e, fwd);
    let p = pc.point(t0);
    let du = if s.u_periodic() { ((at.x - p.x) / TAU).round() * TAU } else { 0.0 };
    let dv = if s.v_periodic() { ((at.y - p.y) / TAU).round() * TAU } else { 0.0 };
    if du == 0.0 && dv == 0.0 {
        pc
    } else {
        pc.transformed(&Affine::translation(V3::new(du, dv, 0.0)))
    }
}

struct Dsu(Vec<usize>);
impl Dsu {
    fn find(&mut self, x: usize) -> usize {
        let mut r = x;
        while self.0[r] != r {
            r = self.0[r];
        }
        let mut y = x;
        while self.0[y] != r {
            let n = self.0[y];
            self.0[y] = r;
            y = n;
        }
        r
    }
    fn join(&mut self, a: usize, b: usize) {
        let (a, b) = (self.find(a), self.find(b));
        if a != b {
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            self.0[hi] = lo;
        }
    }
}

/// Splits edge `e` at parameter t (a new vertex), updating every face that uses it. Returns the
/// new vertex.
pub fn split_edge(b: &mut Body, e: usize, t: f64) -> usize {
    let p = b.edges[e].point(t);
    let v = b.add_vertex(p);
    let old = b.edges[e].clone();
    b.edges[e] = Edge { t: [old.t[0], t], v: [old.v[0], v], ..old.clone() };
    b.edges.push(Edge { t: [t, old.t[1]], v: [v, old.v[1]], ..old });
    let e2 = b.edges.len() - 1;
    for f in &mut b.faces {
        for l in &mut f.loops {
            let mut nl = Vec::with_capacity(l.len() + 1);
            for c in l.drain(..) {
                if c.edge == e {
                    let second = Coedge { edge: e2, fwd: c.fwd, pc: c.pc.clone() };
                    if c.fwd {
                        nl.push(c);
                        nl.push(second);
                    } else {
                        nl.push(second);
                        nl.push(c);
                    }
                } else {
                    nl.push(c);
                }
            }
            *l = nl;
        }
    }
    v
}

/// Merges groups of faces; `can_merge(f, g)` says whether two faces may be merged (faces of
/// different inputs). `from` (per face) is concatenated for merged faces, the group's first face
/// first. Returns the new `from`.
pub fn unify_faces(b: &mut Body, from: Vec<Vec<(usize, usize)>>, can_merge: &dyn Fn(usize, usize) -> bool) -> Vec<Vec<(usize, usize)>> {
    let nf = b.faces.len();
    let ef = b.edge_faces();
    let mut dsu = Dsu((0..nf).collect());
    for (_e, fs) in ef.iter().enumerate() {
        if fs.len() != 2 || fs[0] == fs[1] {
            continue;
        }
        let (f, g) = (fs[0], fs[1]);
        if !can_merge(f, g) {
            continue;
        }
        let (a, c) = (&b.faces[f], &b.faces[g]);
        if let Some(same) = a.surf.same_domain(&c.surf, 1e-6) {
            if same == (a.sense == c.sense) {
                dsu.join(f, g);
            }
        }
    }
    let mut groups: std::collections::BTreeMap<usize, Vec<usize>> = Default::default();
    for f in 0..nf {
        let r = dsu.find(f);
        groups.entry(r).or_default().push(f);
    }
    let mut replace: Vec<Option<Face>> = vec![None; nf];
    let mut drop = vec![false; nf];
    for (_, g) in groups {
        if g.len() < 2 {
            continue;
        }
        if let Some(face) = merge_group(b, &g) {
            replace[g[0]] = Some(face);
            for &f in &g[1..] {
                drop[f] = true;
            }
        }
    }
    // Rebuild the face list.
    let mut new_from: Vec<Vec<(usize, usize)>> = Vec::new();
    let mut faces = Vec::new();
    let mut gfrom: std::collections::HashMap<usize, Vec<(usize, usize)>> = Default::default();
    for (r, f) in (0..nf).map(|f| (dsu.find(f), f)) {
        if replace[r].is_some() || drop[r] {
            gfrom.entry(r).or_default().extend(from[f].iter().copied());
        }
    }
    for f in 0..nf {
        if drop[f] {
            continue;
        }
        match replace[f].take() {
            Some(face) => {
                faces.push(face);
                new_from.push(gfrom.remove(&f).unwrap_or_else(|| from[f].clone()));
            }
            None => {
                faces.push(b.faces[f].clone());
                new_from.push(from[f].clone());
            }
        }
    }
    b.faces = faces;
    compact(b);
    new_from
}

/// Drops unused edges and vertices.
pub fn compact(b: &mut Body) {
    let mut used_e = vec![false; b.edges.len()];
    for f in &b.faces {
        for l in &f.loops {
            for c in l {
                used_e[c.edge] = true;
            }
        }
    }
    let mut emap = vec![usize::MAX; b.edges.len()];
    let mut edges = Vec::new();
    for (i, e) in b.edges.iter().enumerate() {
        if used_e[i] {
            emap[i] = edges.len();
            edges.push(e.clone());
        }
    }
    let mut used_v = vec![false; b.verts.len()];
    for e in &edges {
        used_v[e.v[0]] = true;
        used_v[e.v[1]] = true;
    }
    let mut vmap = vec![usize::MAX; b.verts.len()];
    let mut verts = Vec::new();
    for (i, v) in b.verts.iter().enumerate() {
        if used_v[i] {
            vmap[i] = verts.len();
            verts.push(v.clone());
        }
    }
    for e in &mut edges {
        e.v = [vmap[e.v[0]], vmap[e.v[1]]];
    }
    for f in &mut b.faces {
        for l in &mut f.loops {
            for c in l {
                c.edge = emap[c.edge];
            }
        }
    }
    b.edges = edges;
    b.verts = verts;
}

/// One merged face for the group, or None when the merge can't be made cleanly.
fn merge_group(b: &mut Body, g: &[usize]) -> Option<Face> {
    let base = g[0];
    let surf = b.faces[base].surf.clone();
    let sense = b.faces[base].sense;
    // Edge uses within the group.
    let mut uses: std::collections::HashMap<usize, Vec<usize>> = Default::default();
    for &f in g {
        for l in &b.faces[f].loops {
            for c in l {
                uses.entry(c.edge).or_default().push(f);
            }
        }
    }
    let internal = |e: usize| -> bool {
        let u = &uses[&e];
        u.len() >= 2 && u.iter().any(|x| *x != u[0])
    };
    let seam = |e: usize| -> bool {
        let u = &uses[&e];
        u.len() == 2 && u[0] == u[1]
    };
    let mut rest: Vec<Coedge> = Vec::new();
    for &f in g {
        for l in &b.faces[f].loops {
            for c in l {
                if internal(c.edge) {
                    continue;
                }
                // A seam: periodic surfaces are re-seamed below; on others it is a slit (a full
                // revolve's planar annulus), which bounds nothing.
                if seam(c.edge) {
                    continue;
                }
                rest.push(c.clone());
            }
        }
    }
    if rest.is_empty() {
        return None;
    }
    // Chain into closed loops by vertices.
    let mut used = vec![false; rest.len()];
    let mut chains: Vec<Vec<Coedge>> = Vec::new();
    for s in 0..rest.len() {
        if used[s] {
            continue;
        }
        used[s] = true;
        let mut ch = vec![rest[s].clone()];
        let start = b.co_start(&rest[s]);
        let mut guard = 0;
        loop {
            let end = b.co_end(ch.last().unwrap());
            if end == start {
                break;
            }
            let nx = (0..rest.len()).find(|&k| !used[k] && b.co_start(&rest[k]) == end)?;
            used[nx] = true;
            ch.push(rest[nx].clone());
            guard += 1;
            if guard > rest.len() {
                return None;
            }
        }
        chains.push(ch);
    }
    // p-curves on the base surface, continuous along each chain.
    let center = {
        let mut bb = Bbox::empty();
        for l in &b.faces[base].loops {
            for c in l {
                let q = co_uv(b, c, false);
                bb.add(&P3::new(q.x, q.y, 0.0));
            }
        }
        let c = bb.center();
        P2::new(c.x, c.y)
    };
    let mut windings = Vec::new();
    for ch in &mut chains {
        let mut at: Option<P2> = None;
        // Start the chain in the base face's chart.
        let first = &ch[0];
        let pc0 = pcurve::pcurve(&surf, &b.edges[first.edge].curve, b.edges[first.edge].t[0], b.edges[first.edge].t[1], Some(center));
        let (t0, _) = co_range(&b.edges[first.edge], first.fwd);
        let p0 = pc0.point(t0);
        let start = P2::new(p0.x, p0.y);
        for c in ch.iter_mut() {
            let e = &b.edges[c.edge];
            let pc = if at.is_none() { pc0.clone() } else { pc_from(&surf, e, c.fwd, at) };
            c.pc = pc;
            at = Some(co_uv(b, c, true));
        }
        let end = at.unwrap();
        windings.push(((end.x - start.x) / TAU).round() as i64);
        if surf.v_periodic() && ((end.y - start.y) / TAU).round() as i64 != 0 {
            return None;
        }
    }
    let flip = if sense { 1.0 } else { -1.0 };
    let (mu, mv) = surf.metric();
    let area = |b: &Body, ch: &[Coedge]| -> f64 {
        let mut pts = Vec::new();
        for c in ch {
            let e = &b.edges[c.edge];
            let (t0, t1) = co_range(e, c.fwd);
            let n = match c.pc {
                Curve::Line { .. } => 1,
                _ => 32,
            };
            for i in 0..n {
                let p = c.pc.point(t0 + (t1 - t0) * i as f64 / n as f64);
                pts.push(P2::new(p.x * mu, p.y * mv * flip));
            }
        }
        polygon_area(&pts)
    };
    let wrapping: Vec<usize> = (0..chains.len()).filter(|&i| windings[i] != 0).collect();
    if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
        eprintln!("merge group {g:?}: {} chains, windings {windings:?}", chains.len());
    }
    let mut loops: Vec<Vec<Coedge>> = Vec::new();
    let mut holes: Vec<Vec<Coedge>> = Vec::new();
    if wrapping.is_empty() {
        for ch in chains {
            if area(b, &ch) > 0.0 {
                loops.push(ch);
            } else {
                holes.push(ch);
            }
        }
        if loops.len() != 1 {
            return None;
        }
    } else {
        if wrapping.len() != 2 || !surf.u_periodic() {
            return None;
        }
        let (i1, i2) = (wrapping[0], wrapping[1]);
        if windings[i1] + windings[i2] != 0 || windings[i1].abs() != 1 {
            return None;
        }
        // Seam at the start vertex of chain 1.
        let s1 = b.co_start(&chains[i1][0]);
        let us = co_uv(b, &chains[i1][0], false).x;
        // A vertex of chain 2 at the same u (mod 2π), else split an edge there.
        let mut s2: Option<(usize, usize)> = None; // (coedge index in chain 2 starting there, vertex)
        for (k, c) in chains[i2].iter().enumerate() {
            let u = co_uv(b, c, false).x;
            if ((u - us) / TAU - ((u - us) / TAU).round()).abs() * TAU * mu < 1e-7 {
                s2 = Some((k, b.co_start(c)));
                break;
            }
        }
        if s2.is_none() {
            // Find the coedge whose u range contains us (mod 2π) and split its edge there.
            let mut hit = None;
            for (k, c) in chains[i2].iter().enumerate() {
                let (a0, a1) = (co_uv(b, c, false).x, co_uv(b, c, true).x);
                let (lo, hi) = if a0 < a1 { (a0, a1) } else { (a1, a0) };
                let target = unwrap_near(us, (lo + hi) * 0.5, TAU);
                if target > lo + 1e-12 && target < hi - 1e-12 {
                    hit = Some((k, target));
                    break;
                }
            }
            let (k, target) = hit?;
            let c = chains[i2][k].clone();
            let e = &b.edges[c.edge];
            // Solve u(t) = target on the p-curve by bisection.
            let (mut lo, mut hi) = (e.t[0], e.t[1]);
            let f = |t: f64| c.pc.point(t).x - target;
            let flo = f(lo);
            if flo.signum() == f(hi).signum() {
                return None;
            }
            for _ in 0..100 {
                let m = 0.5 * (lo + hi);
                if f(m).signum() == flo.signum() {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            let t = 0.5 * (lo + hi);
            let e_id = c.edge;
            let v = split_edge(b, e_id, t);
            // Rebuild chain 2 with the split coedges (the split updated b.faces, not our chain).
            let e2 = b.edges.len() - 1;
            let pc = c.pc.clone();
            let (first, second) = (Coedge { edge: e_id, fwd: c.fwd, pc: pc.clone() }, Coedge { edge: e2, fwd: c.fwd, pc });
            let (pa, pb) = if c.fwd { (first, second) } else { (second, first) };
            chains[i2].splice(k..=k, [pa, pb]);
            s2 = Some((k + 1, v));
        }
        let (k2, s2v) = s2?;
        // Rotate chain 2 to start at s2.
        chains[i2].rotate_left(k2);
        // Re-align p-curves: chain 1 from u_s, chain 2 from where chain 1 ends.
        let realign = |b: &Body, ch: &mut Vec<Coedge>, start: P2| {
            let mut at = Some(start);
            for c in ch.iter_mut() {
                let e = &b.edges[c.edge];
                c.pc = pc_from(&surf, e, c.fwd, at);
                at = Some(co_uv(b, c, true));
            }
        };
        let start1 = co_uv(b, &chains[i1][0], false);
        let mut ch1 = chains[i1].clone();
        realign(b, &mut ch1, start1);
        let end1 = co_uv(b, ch1.last().unwrap(), true);
        // Seam edge along the iso-u line from s1 to s2.
        let iso = surf.iso_u(us)?;
        let p1 = b.verts[s1].p;
        let p2 = b.verts[s2v].p;
        let (v1, v2) = (start1.y, {
            let (_, v) = surf.project(&p2, Some((us, start1.y)));
            v
        });
        let _ = p1;
        let mut ch2 = chains[i2].clone();
        realign(b, &mut ch2, P2::new(end1.x, v2));
        let end2 = co_uv(b, ch2.last().unwrap(), true);
        if (end2.x - us).abs() > 1e-6 && (end2.x - (us + 0.0)).abs() > 1e-6 {
            // chain 2 should come back to u_s
            let _ = end2;
        }
        let (lo, hi, vlo, vhi) = if v1 < v2 { (s1, s2v, v1, v2) } else { (s2v, s1, v2, v1) };
        b.edges.push(Edge { curve: iso, t: [vlo, vhi], v: [lo, hi], degenerate: false });
        let se = b.edges.len() - 1;
        let line_at = |u: f64| Curve::Line { o: P3::new(u, 0.0, 0.0), d: V3::new(0.0, 1.0, 0.0) };
        // From s1 (at u_s + 2πw) to s2, then chain 2, then from s2 (at u_s) back to s1.
        let up_first = v1 < v2; // the seam edge runs s1 → s2
        let mut lp = ch1;
        lp.push(Coedge { edge: se, fwd: up_first, pc: line_at(end1.x) });
        lp.extend(ch2);
        lp.push(Coedge { edge: se, fwd: !up_first, pc: line_at(start1.x) });
        loops.push(lp);
        for (i, ch) in chains.into_iter().enumerate() {
            if i != i1 && i != i2 {
                holes.push(ch);
            }
        }
        // The loop must be counter-clockwise in oriented uv; otherwise give up.
        if area(b, &loops[0]) <= 0.0 {
            return None;
        }
    }
    loops.extend(holes);
    Some(Face { surf, sense, loops })
}

/// Merges pairs of edges meeting at a vertex used by no other edge when they lie on one
/// line or circle and bound the same faces.
pub fn merge_edges(b: &mut Body) {
    loop {
        let mut vedges: Vec<Vec<usize>> = vec![Vec::new(); b.verts.len()];
        for (i, e) in b.edges.iter().enumerate() {
            if e.degenerate {
                vedges[e.v[0]].push(usize::MAX);
                continue;
            }
            vedges[e.v[0]].push(i);
            if e.v[1] != e.v[0] {
                vedges[e.v[1]].push(i);
            }
        }
        let ef = b.edge_faces();
        let mut done = false;
        for v in 0..b.verts.len() {
            let es = &vedges[v];
            if es.len() != 2 || es[0] == es[1] || es.contains(&usize::MAX) {
                continue;
            }
            let (e1, e2) = (es[0], es[1]);
            let (a, c) = (b.edges[e1].clone(), b.edges[e2].clone());
            if a.v[0] == a.v[1] || c.v[0] == c.v[1] {
                continue;
            }
            let mut fa = ef[e1].clone();
            let mut fc = ef[e2].clone();
            fa.sort();
            fc.sort();
            if fa != fc {
                continue;
            }
            // Same carrier.
            let same = match (&a.curve, &c.curve) {
                (Curve::Line { o: o1, d: d1 }, Curve::Line { o: o2, d: d2 }) => {
                    d1.normalize().cross(&d2.normalize()).norm() < 1e-10 && {
                        let w = o2 - o1;
                        w.cross(&d1.normalize()).norm() < 1e-9
                    }
                }
                (Curve::Circle { c: c1, x: x1, y: y1, r: r1 }, Curve::Circle { c: c2, x: x2, y: y2, r: r2 }) => {
                    (c1 - c2).norm() < 1e-9 && (r1 - r2).abs() < 1e-9 && x1.cross(y1).cross(&x2.cross(y2)).norm() < 1e-10
                }
                _ => false,
            };
            if !same {
                continue;
            }
            // The merged edge on a's curve: from a's far end to c's far end through v.
            let far_a = if a.v[0] == v { a.v[1] } else { a.v[0] };
            let far_c = if c.v[0] == v { c.v[1] } else { c.v[0] };
            let curve = a.curve.clone();
            let pa = b.verts[far_a].p;
            let pc_ = b.verts[far_c].p;
            let pv = b.verts[v].p;
            // Parameters on a's curve, continuous through v.
            let tv = if a.v[0] == v { a.t[0] } else { a.t[1] };
            let ta = if a.v[0] == v { a.t[1] } else { a.t[0] };
            let tc = match &curve {
                Curve::Line { o, d } => (pc_ - o).dot(d) / d.norm_squared(),
                _ => {
                    // Angle on the circle unwrapped away from ta through tv.
                    let raw = curve.closest(&pc_, tv - PI, tv + PI);
                    let dir = (tv - ta).signum();
                    let mut t = unwrap_near(raw, tv, TAU);
                    if (t - tv) * dir <= 0.0 {
                        t += TAU * dir;
                    }
                    t
                }
            };
            let _ = (pa, pv);
            let closed = far_a == far_c;
            let (t0, t1, v0, v1) = if ta < tc { (ta, tc, far_a, far_c) } else { (tc, ta, far_c, far_a) };
            if (t1 - t0) > TAU + 1e-9 {
                continue;
            }
            let ne = Edge { curve: curve.clone(), t: [t0, t1], v: [v0, v1], degenerate: false };
            let _ = closed;
            b.edges[e1] = ne.clone();
            // Replace coedges: in each face, the coedges of e1 and e2 become one of e1.
            for fi in 0..b.faces.len() {
                let surf = b.faces[fi].surf.clone();
                for li in 0..b.faces[fi].loops.len() {
                    for _rep in 0..2 {
                    let l = b.faces[fi].loops[li].clone();
                    let n = l.len();
                    // Find e1 and e2 adjacent in this loop.
                    let mut pos = None;
                    for k in 0..n {
                        let (x, y) = (&l[k], &l[(k + 1) % n]);
                        let x_end = {
                            let ed = if x.edge == e1 { &a } else { &c };
                            if x.fwd { ed.v[1] } else { ed.v[0] }
                        };
                        if ((x.edge == e1 && y.edge == e2) || (x.edge == e2 && y.edge == e1)) && x_end == v {
                            pos = Some(k);
                            break;
                        }
                    }
                    let Some(k) = pos else { continue };
                    let first = &l[k];
                    // Direction: the merged coedge starts where `first` starts.
                    let start_v = {
                        let ed = if first.edge == e1 { &a.clone() } else { &c.clone() };
                        if first.fwd { ed.v[0] } else { ed.v[1] }
                    };
                    // p-curve continuous with the old first coedge's start.
                    let old_e = if first.edge == e1 { a.clone() } else { c.clone() };
                    let (ot0, _) = co_range(&old_e, first.fwd);
                    let fwd = if ne.v[0] != ne.v[1] {
                        ne.v[0] == start_v
                    } else {
                        let travel = old_e.curve.tangent(ot0) * if first.fwd { 1.0 } else { -1.0 };
                        ne.curve.tangent(ne.t[0]).dot(&travel) > 0.0
                    };
                    let sp = first.pc.point(ot0);
                    let pc = pc_from(&surf, &ne, fwd, Some(P2::new(sp.x, sp.y)));
                    let mut nl: Vec<Coedge> = Vec::with_capacity(n - 1);
                    for (j, co) in l.iter().enumerate() {
                        if j == k {
                            nl.push(Coedge { edge: e1, fwd, pc: pc.clone() });
                        } else if j == (k + 1) % n {
                            continue;
                        } else {
                            nl.push(co.clone());
                        }
                    }
                    if k + 1 == n {
                        // The pair wrapped: the first element was removed above; fix order.
                    }
                    b.faces[fi].loops[li] = nl;
                    }
                }
            }
            done = true;
            break;
        }
        if !done {
            break;
        }
        compact(b);
    }
}

/// B-spline faces that are flat (all control points in one plane: a loft or skin through
/// coplanar curves) become planes, with their p-curves recomputed, so booleans see them
/// coincide with other planar faces. True when any changed.
pub fn planarize(b: &mut Body) -> bool {
    let mut changed = false;
    for fi in 0..b.faces.len() {
        let Surface::Nurbs(ns) = &b.faces[fi].surf else { continue };
        let pts: Vec<P3> = ns.cps.iter().flatten().map(super::nurbs::dehom).collect();
        let c = P3::from(pts.iter().fold(V3::zeros(), |a, p| a + p.coords) / pts.len() as f64);
        let size = pts.iter().map(|p| (p - c).norm()).fold(0.0, f64::max);
        let ((u0, u1), (v0, v1)) = ns.range();
        let (um, vm) = (0.5 * (u0 + u1), 0.5 * (v0 + v1));
        let s = &b.faces[fi].surf;
        let (o, su, sv) = s.d1(um, vm);
        let n = su.cross(&sv);
        if n.norm() < 1e-12 * (1.0 + size * size) {
            continue;
        }
        let n = n.normalize();
        // Sampled on the surface (exact for a flat skin), and the control net as well.
        let mut dev = 0.0f64;
        for i in 0..=8 {
            for j in 0..=8 {
                let q = s.eval(u0 + (u1 - u0) * i as f64 / 8.0, v0 + (v1 - v0) * j as f64 / 8.0);
                dev = dev.max((q - o).dot(&n).abs());
            }
        }
        if std::env::var("CADRS_BOOL_DEBUG").is_ok() {
            let net = pts.iter().map(|p| (p - o).dot(&n).abs()).fold(0.0, f64::max);
            eprintln!("planarize face {fi}: deviation {dev:e} (net {net:e}, size {size})");
        }
        if dev > 1e-10 * (1.0 + size) {
            continue;
        }
        let x = su.normalize();
        let y = n.cross(&x);
        let plane = Surface::Plane { f: Frame { o, x, y, z: n } };
        let edges = b.edges.clone();
        for l in &mut b.faces[fi].loops {
            for co in l.iter_mut() {
                let e = &edges[co.edge];
                co.pc = pcurve::pcurve(&plane, &e.curve, e.t[0], e.t[1], None);
            }
        }
        b.faces[fi].surf = plane;
        changed = true;
    }
    changed
}
