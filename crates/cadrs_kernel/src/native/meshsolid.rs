//! A solid from a closed triangle mesh (STL import, [`crate::Kernel::mesh_solid`]).
//!
//! 1. Corners within `tol` are welded into vertices; triangles that collapse are dropped.
//! 2. The mesh must be closed and consistently oriented: every directed edge used by exactly one
//!    triangle and its reverse by exactly one other (an open or non-manifold mesh is an error).
//! 3. Coplanar neighbouring triangles are flooded into planar regions (normals within ~1e-5 rad
//!    and corners within `tol` of the region's first triangle's plane, so curved meshes don't
//!    drift into one face).
//! 4. Each region's boundary half-edges are chained into polygon loops; a boundary vertex used by
//!    only two boundary segments in line with each other is dropped, so straight edges are one
//!    edge. Loops run counter-clockwise about the outward normal, holes clockwise.
//! 5. A mesh oriented inside out (negative volume) has every face reversed.

use std::collections::HashMap;

use super::curve::Curve;
use super::math::*;
use super::pcurve;
use super::surface::Surface;
use super::topo::*;
use super::tol;

/// Welds points within `tol` (looking in neighbouring grid cells).
struct Weld {
    tol: f64,
    cells: HashMap<[i64; 3], Vec<usize>>,
    pts: Vec<P3>,
}

impl Weld {
    fn key(&self, p: &P3) -> [i64; 3] {
        [p.x, p.y, p.z].map(|c| (c / self.tol).floor() as i64)
    }
    fn add(&mut self, p: P3) -> usize {
        let k = self.key(&p);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(v) = self.cells.get(&[k[0] + dx, k[1] + dy, k[2] + dz]) {
                        for &i in v {
                            if (self.pts[i] - p).norm() <= self.tol {
                                return i;
                            }
                        }
                    }
                }
            }
        }
        self.pts.push(p);
        let i = self.pts.len() - 1;
        self.cells.entry(k).or_default().push(i);
        i
    }
}

pub fn mesh_solid(triangles: &[[P3; 3]], tol: f64) -> Result<Body, String> {
    let tol = if tol.is_finite() && tol > 0.0 { tol } else { 1e-6 };
    let mut weld = Weld { tol, cells: HashMap::new(), pts: Vec::new() };
    let mut tris: Vec<[usize; 3]> = Vec::with_capacity(triangles.len());
    for t in triangles {
        let ix = [weld.add(t[0]), weld.add(t[1]), weld.add(t[2])];
        if ix[0] == ix[1] || ix[1] == ix[2] || ix[2] == ix[0] {
            continue;
        }
        let (a, b, c) = (weld.pts[ix[0]], weld.pts[ix[1]], weld.pts[ix[2]]);
        if (b - a).cross(&(c - a)).norm() <= tol::TINY {
            continue;
        }
        tris.push(ix);
    }
    if tris.len() < 4 {
        return Err("fewer than four usable triangles".into());
    }
    let pts = &weld.pts;
    // Directed edges.
    let mut half: HashMap<(usize, usize), usize> = HashMap::with_capacity(tris.len() * 3);
    for (ti, t) in tris.iter().enumerate() {
        for k in 0..3 {
            if half.insert((t[k], t[(k + 1) % 3]), ti).is_some() {
                return Err("the mesh is not consistently oriented or not manifold (an edge is used twice the same way)".into());
            }
        }
    }
    for &(a, b) in half.keys() {
        if !half.contains_key(&(b, a)) {
            return Err("the mesh is not closed (an edge is used by one triangle only)".into());
        }
    }
    let normal = |t: &[usize; 3]| (pts[t[1]] - pts[t[0]]).cross(&(pts[t[2]] - pts[t[0]])).normalize();
    let normals: Vec<V3> = tris.iter().map(normal).collect();
    // Planar regions.
    let mut region = vec![usize::MAX; tris.len()];
    let mut regions: Vec<Vec<usize>> = Vec::new();
    for seed in 0..tris.len() {
        if region[seed] != usize::MAX {
            continue;
        }
        let r = regions.len();
        let (n0, p0) = (normals[seed], pts[tris[seed][0]]);
        let mut members = vec![seed];
        region[seed] = r;
        let mut stack = vec![seed];
        while let Some(t) = stack.pop() {
            for k in 0..3 {
                let (a, b) = (tris[t][k], tris[t][(k + 1) % 3]);
                let o = half[&(b, a)];
                if region[o] != usize::MAX || normals[o].dot(&n0) < 1.0 - 1e-10 {
                    continue;
                }
                if tris[o].iter().any(|&v| (pts[v] - p0).dot(&n0).abs() > tol) {
                    continue;
                }
                region[o] = r;
                members.push(o);
                stack.push(o);
            }
        }
        regions.push(members);
    }
    // Boundary half-edges per region, and each vertex's boundary valence.
    let mut bnd: Vec<Vec<(usize, usize)>> = vec![Vec::new(); regions.len()];
    let mut valence: HashMap<usize, usize> = HashMap::new();
    for (ti, t) in tris.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            if region[half[&(b, a)]] != region[ti] {
                bnd[region[ti]].push((a, b));
                if a < b {
                    *valence.entry(a).or_default() += 1;
                    *valence.entry(b).or_default() += 1;
                }
            }
        }
    }
    let mut body = Body::default();
    let mut vmap: HashMap<usize, usize> = HashMap::new();
    let mut emap: HashMap<(usize, usize), usize> = HashMap::new();
    for (ri, hs) in bnd.iter().enumerate() {
        // Chain the half-edges into loops.
        let mut from: HashMap<usize, Vec<usize>> = HashMap::new();
        for (i, &(a, _)) in hs.iter().enumerate() {
            from.entry(a).or_default().push(i);
        }
        let mut used = vec![false; hs.len()];
        let mut loops: Vec<Vec<usize>> = Vec::new();
        for s in 0..hs.len() {
            if used[s] {
                continue;
            }
            let mut l = Vec::new();
            let mut cur = s;
            loop {
                used[cur] = true;
                l.push(hs[cur].0);
                let end = hs[cur].1;
                if end == hs[s].0 {
                    break;
                }
                match from.get(&end).and_then(|v| v.iter().copied().find(|&i| !used[i])) {
                    Some(n) => cur = n,
                    None => return Err("a face boundary doesn't close".into()),
                }
            }
            // Drop vertices in line with their neighbours and on no other boundary.
            let mut changed = true;
            while changed && l.len() > 3 {
                changed = false;
                for k in 0..l.len() {
                    let n = l.len();
                    let (a, v, b) = (l[(k + n - 1) % n], l[k], l[(k + 1) % n]);
                    if valence.get(&v) != Some(&2) {
                        continue;
                    }
                    let (pa, pv, pb) = (pts[a], pts[v], pts[b]);
                    let ab = pb - pa;
                    if (pv - pa).cross(&ab).norm() <= tol * ab.norm() && (pv - pa).dot(&ab) > 0.0 && (pb - pv).dot(&ab) > 0.0 {
                        l.remove(k);
                        changed = true;
                        break;
                    }
                }
            }
            loops.push(l);
        }
        // The face: a plane through the region with its outward normal (Newell's, of all loops).
        let mut n = V3::zeros();
        for l in &loops {
            for k in 0..l.len() {
                let (p, q) = (pts[l[k]], pts[l[(k + 1) % l.len()]]);
                n += p.coords.cross(&q.coords);
            }
        }
        let n = if n.norm() > tol::TINY { n.normalize() } else { normals[regions[ri][0]] };
        let l0 = &loops[0];
        let o = pts[l0[0]];
        let f = Frame::from_xz(o, pts[l0[1]] - o, n);
        let surf = Surface::Plane { f };
        let area = |l: &[usize]| -> f64 {
            let q: Vec<P2> = l.iter().map(|&v| {
                let lp = f.local(&pts[v]);
                P2::new(lp.x, lp.y)
            }).collect();
            polygon_area(&q)
        };
        let mut ls: Vec<(f64, Vec<usize>)> = loops.into_iter().map(|l| (area(&l), l)).collect();
        ls.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut face_loops = Vec::new();
        for (_, l) in ls {
            let mut co = Vec::with_capacity(l.len());
            for k in 0..l.len() {
                let (a, b) = (l[k], l[(k + 1) % l.len()]);
                let va = *vmap.entry(a).or_insert_with(|| body.add_vertex(pts[a]));
                let vb = *vmap.entry(b).or_insert_with(|| body.add_vertex(pts[b]));
                let key = (a.min(b), a.max(b));
                let e = *emap.entry(key).or_insert_with(|| {
                    let (p, q) = (pts[key.0], pts[key.1]);
                    let (c, len) = Curve::line(p, q);
                    let (v0, v1) = if key.0 == a { (va, vb) } else { (vb, va) };
                    body.add_edge(c, 0.0, len, v0, v1)
                });
                let ed = &body.edges[e];
                let pc = pcurve::pcurve(&surf, &ed.curve, ed.t[0], ed.t[1], None);
                co.push(Coedge { edge: e, fwd: key.0 == a, pc });
            }
            face_loops.push(co);
        }
        body.faces.push(Face { surf, sense: true, loops: face_loops });
    }
    let nf = body.faces.len();
    body.shells.push(Shell { faces: (0..nf).collect(), closed: true });
    body.solids.push(Solid { shells: vec![0] });
    if super::mass::body_mass(&body).volume < 0.0 {
        for f in &mut body.faces {
            Body::reverse_face(f);
        }
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cube_of_triangles_with_split_sides() {
        let c = |x: f64, y: f64, z: f64| P3::new(x * 10.0, y * 10.0, z * 10.0);
        let quads = [
            [c(0., 0., 0.), c(0., 1., 0.), c(1., 1., 0.), c(1., 0., 0.)],
            [c(0., 0., 1.), c(1., 0., 1.), c(1., 1., 1.), c(0., 1., 1.)],
            [c(0., 0., 0.), c(1., 0., 0.), c(1., 0., 1.), c(0., 0., 1.)],
            [c(0., 1., 0.), c(0., 1., 1.), c(1., 1., 1.), c(1., 1., 0.)],
            [c(0., 0., 0.), c(0., 0., 1.), c(0., 1., 1.), c(0., 1., 0.)],
            [c(1., 0., 0.), c(1., 1., 0.), c(1., 1., 1.), c(1., 0., 1.)],
        ];
        // Each side as four triangles about its middle (the middles are interior vertices).
        let mut tris = Vec::new();
        for q in &quads {
            let m = P3::from((q[0].coords + q[1].coords + q[2].coords + q[3].coords) / 4.0);
            for k in 0..4 {
                tris.push([q[k], q[(k + 1) % 4], m]);
            }
        }
        let b = mesh_solid(&tris, 1e-6).unwrap();
        assert!(b.check().is_empty(), "{:?}", b.check());
        assert_eq!(b.faces.len(), 6);
        assert_eq!(b.edges.len(), 12);
        assert!((super::super::mass::body_mass(&b).volume - 1000.0).abs() < 1e-9);
        // Inside out: reversed back.
        let inv: Vec<[P3; 3]> = tris.iter().map(|t| [t[0], t[2], t[1]]).collect();
        assert!((super::super::mass::body_mass(&mesh_solid(&inv, 1e-6).unwrap()).volume - 1000.0).abs() < 1e-9);
        assert!(mesh_solid(&tris[..20], 1e-6).is_err());
    }
}
