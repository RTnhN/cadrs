//! Temporary section caps from the displayed mesh, without a kernel boolean.
use std::collections::{HashMap, HashSet};

use bevy::math::{DVec3, Vec3};
use cadrs_core::{Solid, section::Cap};
use cadrs_sketch::{
    Vec2,
    geom::{point_in_polygon, polygon_area},
    region::Region,
};

/// Stitch only closed, unbranched contours. Open meshes must not acquire invented faces.
fn contours(segments: Vec<(DVec3, DVec3)>) -> Vec<Vec<DVec3>> {
    const EPS: f64 = 1e-5;
    let mut buckets: HashMap<[i64; 3], Vec<usize>> = HashMap::new();
    let mut points: Vec<DVec3> = Vec::new();
    let mut vertex = |p: DVec3| {
        let cell = (p / EPS).floor().as_i64vec3().to_array();
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if let Some(ids) = buckets.get(&[cell[0] + x, cell[1] + y, cell[2] + z]) {
                        if let Some(&id) = ids
                            .iter()
                            .find(|&&id| points[id].distance_squared(p) <= EPS * EPS)
                        {
                            return id;
                        }
                    }
                }
            }
        }
        let id = points.len();
        points.push(p);
        buckets.entry(cell).or_default().push(id);
        id
    };
    let mut edges = HashSet::new();
    for (a, b) in segments {
        let (a, b) = (vertex(a), vertex(b));
        if a != b {
            edges.insert((a.min(b), a.max(b)));
        }
    }
    let mut neighbors = vec![Vec::new(); points.len()];
    for (a, b) in edges {
        neighbors[a].push(b);
        neighbors[b].push(a);
    }
    let mut visited = vec![false; points.len()];
    let mut out = Vec::new();
    for seed in 0..points.len() {
        if visited[seed] {
            continue;
        }
        let mut component = Vec::new();
        let mut stack = vec![seed];
        while let Some(i) = stack.pop() {
            if visited[i] {
                continue;
            }
            visited[i] = true;
            component.push(i);
            stack.extend(neighbors[i].iter().copied().filter(|&n| !visited[n]));
        }
        if component.len() < 3 || component.iter().any(|&i| neighbors[i].len() != 2) {
            continue;
        }
        let mut ring = vec![points[seed]];
        let (mut previous, mut current) = (seed, neighbors[seed][0]);
        while current != seed && ring.len() <= component.len() {
            ring.push(points[current]);
            let next = neighbors[current]
                .iter()
                .copied()
                .find(|&n| n != previous)
                .unwrap();
            previous = current;
            current = next;
        }
        if current == seed && ring.len() == component.len() {
            out.push(ring);
        }
    }
    out
}

pub(super) fn cap(solid: &Solid, origin: Vec3, normal: Vec3) -> Cap {
    let (o, n) = (origin.as_dvec3(), normal.as_dvec3().normalize());
    let distances: Vec<f64> = solid
        .positions
        .iter()
        .map(|p| {
            let d = (DVec3::from_array(*p) - o).dot(n);
            if d.abs() < 1e-7 { 0.0 } else { d }
        })
        .collect();
    if !distances.iter().any(|&d| d >= 0.0) || !distances.iter().any(|&d| d <= 0.0) {
        return Cap::default();
    }
    let mut segments = Vec::new();
    for triangle in solid.indices.chunks_exact(3) {
        let ids = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        let d = ids.map(|i| distances[i]);
        if d.iter().all(|&d| d == 0.0) {
            continue;
        }
        let p = ids.map(|i| DVec3::from_array(solid.positions[i]));
        let mut hits: Vec<DVec3> = Vec::with_capacity(3);
        for i in 0..3 {
            let j = (i + 1) % 3;
            if d[i] == 0.0 {
                hits.push(p[i]);
            }
            if d[i] * d[j] < 0.0 {
                hits.push(p[i].lerp(p[j], d[i] / (d[i] - d[j])));
            }
        }
        hits.dedup_by(|a, b| a.distance_squared(*b) < 1e-16);
        if hits.len() == 2 && hits[0].distance_squared(hits[1]) > 1e-16 {
            segments.push((hits[0], hits[1]));
        }
    }
    let u = n
        .cross(if n.x.abs() < 0.9 { DVec3::X } else { DVec3::Y })
        .normalize();
    let v = n.cross(u);
    let rings: Vec<Vec<Vec2>> = contours(segments)
        .into_iter()
        .map(|ring| {
            ring.into_iter()
                .map(|p| {
                    let q = p - o;
                    Vec2::new(q.dot(u), q.dot(v))
                })
                .collect()
        })
        .filter(|ring: &Vec<Vec2>| polygon_area(ring).abs() > 1e-10)
        .collect();
    let areas: Vec<f64> = rings.iter().map(|r| polygon_area(r).abs()).collect();
    let parents: Vec<Option<usize>> = rings
        .iter()
        .enumerate()
        .map(|(i, r)| {
            (0..rings.len())
                .filter(|&j| areas[j] > areas[i] && point_in_polygon(r[0], &rings[j]))
                .min_by(|&a, &b| areas[a].total_cmp(&areas[b]))
        })
        .collect();
    let depth = |i: usize| {
        let mut d = 0;
        let mut p = parents[i];
        while let Some(i) = p {
            d += 1;
            p = parents[i];
        }
        d
    };
    let to3 = |p: Vec2| (o + u * p.x + v * p.y).as_vec3().to_array();
    let mut cap = Cap::default();
    for (i, outer) in rings.iter().enumerate().filter(|(i, _)| depth(*i) % 2 == 0) {
        let holes: Vec<Vec<Vec2>> = rings
            .iter()
            .enumerate()
            .filter(|(j, _)| parents[*j] == Some(i))
            .map(|(_, r)| r.clone())
            .collect();
        let region = Region {
            outer: outer.clone(),
            holes,
            curves: Vec::new(),
            outer_curves: Vec::new(),
            hole_curves: Vec::new(),
            outer_pieces: Vec::new(),
            hole_pieces: Vec::new(),
            hole_piece_curves: Vec::new(),
        };
        let (vertices, indices) = region.triangulate();
        if indices.is_empty() {
            continue;
        }
        for ids in indices.chunks_exact(3) {
            let mut t = [
                to3(vertices[ids[0] as usize]),
                to3(vertices[ids[1] as usize]),
                to3(vertices[ids[2] as usize]),
            ];
            let [a, b, c] = t.map(|p| Vec3::from_array(p).as_dvec3());
            let cross = (b - a).cross(c - a);
            cap.area += cross.length() * 0.5;
            if cross.dot(n) < 0.0 {
                t.swap(1, 2);
            }
            cap.triangles.push(t);
        }
        cap.loops.push(outer.iter().copied().map(to3).collect());
        cap.loops.extend(
            region
                .holes
                .iter()
                .map(|h| h.iter().copied().map(to3).collect::<Vec<_>>()),
        );
    }
    cap
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    fn walls(s: &mut Solid, ring: &[[f64; 2]]) {
        for i in 0..ring.len() {
            let [x, y] = ring[i];
            let [u, v] = ring[(i + 1) % ring.len()];
            let q = [[x, y, -1.0], [u, v, -1.0], [u, v, 1.0], [x, y, 1.0]];
            // Independent vertices, as at tessellation seams.
            for tri in [[q[0], q[1], q[2]], [q[0], q[2], q[3]]] {
                let base = s.positions.len() as u32;
                s.positions.extend(tri);
                s.indices.extend([base, base + 1, base + 2]);
            }
        }
    }
    pub(crate) fn tube() -> Solid {
        let mut solid = Solid::default();
        walls(
            &mut solid,
            &[[-2.0, -2.0], [2.0, -2.0], [2.0, 2.0], [-2.0, 2.0]],
        );
        walls(
            &mut solid,
            &[[-1.0, -1.0], [-1.0, 1.0], [1.0, 1.0], [1.0, -1.0]],
        );
        solid
    }

    #[test]
    fn caps_preserve_holes_islands_and_triangle_winding() {
        let mut solid = tube();
        walls(
            &mut solid,
            &[[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]],
        );
        walls(
            &mut solid,
            &[[3.0, -1.0], [5.0, -1.0], [5.0, 1.0], [3.0, 1.0]],
        );
        for normal in [Vec3::Z, -Vec3::Z] {
            let result = cap(&solid, Vec3::ZERO, normal);
            assert!((result.area - 17.0).abs() < 1e-6, "{}", result.area);
            assert_eq!(result.loops.len(), 4);
            for t in result.triangles {
                let [a, b, c] = t.map(Vec3::from_array);
                assert!((b - a).cross(c - a).dot(normal) > 0.0);
                let center = (a + b + c) / 3.0;
                assert!(
                    !(center.x.abs() < 1.0 && center.y.abs() < 1.0)
                        || (center.x.abs() <= 0.5 && center.y.abs() <= 0.5),
                    "filled the hole: {center}"
                );
            }
        }
    }

    #[test]
    fn misses_open_contours_and_planes_through_mesh_vertices() {
        let mut solid = tube();
        assert!(cap(&solid, Vec3::Z * 2.0, Vec3::Z).triangles.is_empty());
        solid.indices.truncate(6 * 3); // Three walls: an open contour cannot make a face.
        assert!(cap(&solid, Vec3::ZERO, Vec3::Z).triangles.is_empty());
        // The tilted cut passes exactly through existing top/bottom vertices.
        let solid = tube();
        let n = Vec3::new(0.5, 0.0, 1.0).normalize();
        let result = cap(&solid, Vec3::ZERO, n);
        assert!(
            (result.area - 12.0 * 1.25f64.sqrt()).abs() < 1e-5,
            "{}",
            result.area
        );
        for p in result.triangles.iter().flatten() {
            assert!(Vec3::from_array(*p).dot(n).abs() < 1e-6);
        }
    }

    #[test]
    fn welding_searches_neighbor_buckets_and_rejects_branches() {
        let a = DVec3::ZERO;
        let b = DVec3::X;
        let c = DVec3::Y;
        let epsilon = DVec3::splat(1e-7);
        let triangle = vec![(a, b), (b + epsilon, c), (c - epsilon, a - epsilon)];
        assert_eq!(contours(triangle.clone()).len(), 1);
        let mut branch = triangle;
        branch.push((b, DVec3::Z));
        assert!(contours(branch).is_empty());
    }
}
