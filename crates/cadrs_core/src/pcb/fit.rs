//! Fitting a board into the parts around it (`docs/PCB_IN_PART_STUDIO.md` §7.3), on display
//! meshes (components are mesh parts, Q1), in the **board frame** of a board copy (layout x and
//! y in mm on its bottom face, z up through it).
//!
//! - [`section`]: what of some parts lies between two heights above (or below) the board,
//!   projected onto it — the parts in the way of components standing that tall, as a region
//!   of the layout (nm), so the layout can show it, components can snap to it and gaps can be
//!   measured to it.
//! - [`footprint_of`]: a part's own shadow on the board (its projection).
//! - [`collisions`]: components whose shadow overlaps another part's section over the
//!   component's own heights (they run into it), with how far.
//! - [`gaps`]: how far a component can move along ±x and ±y of the board before it meets
//!   something.
//! - [`axes`]: round features standing square to the board (pins, holes): where their axes meet
//!   it, to snap a pad onto.
//! - [`align`]: the move in the board's plane that puts an entity of a component onto a target
//!   (a pin's axis onto a hole's, a face onto a face).

use cadrs_eda::poly::{self, Region};
use cadrs_eda::units::{Pt, mm};
use cadrs_sketch::PlaneFrame;

use crate::solid::Solid;

type V3 = [f64; 3];

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A point of the Part Studio in a board frame (x, y on the layout in mm, z up from the
/// bottom face).
pub fn to_board(frame: &PlaneFrame, p: V3) -> V3 {
    let d = sub(p, frame.origin);
    [dot(d, frame.u), dot(d, frame.v), dot(d, frame.normal())]
}

/// A direction of the Part Studio in a board frame.
pub fn dir_to_board(frame: &PlaneFrame, d: V3) -> V3 {
    [dot(d, frame.u), dot(d, frame.v), dot(d, frame.normal())]
}

/// The part of the triangle `t` (board frame) between heights `z0` and `z1`, if any.
fn clip_slab(t: [V3; 3], z0: f64, z1: f64) -> Vec<V3> {
    let mut poly: Vec<V3> = t.to_vec();
    for (limit, below) in [(z0, false), (z1, true)] {
        let inside = |p: &V3| if below { p[2] <= limit } else { p[2] >= limit };
        let mut out = Vec::with_capacity(poly.len() + 2);
        for i in 0..poly.len() {
            let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
            let (ia, ib) = (inside(&a), inside(&b));
            if ia {
                out.push(a);
            }
            if ia != ib {
                let k = (limit - a[2]) / (b[2] - a[2]);
                out.push([a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, limit]);
            }
        }
        poly = out;
        if poly.len() < 3 {
            return vec![];
        }
    }
    poly
}

fn ring_of(pts: &[V3]) -> Option<Vec<Pt>> {
    let mut ring: Vec<Pt> = pts.iter().map(|p| Pt::mm(p[0], p[1])).collect();
    ring.dedup();
    if ring.len() < 3 {
        return None;
    }
    let a = poly::ring_area(&ring);
    // Edge-on facets (walls seen along) cast no shadow.
    if a.abs() < 1e6 {
        return None;
    }
    if a < 0.0 {
        ring.reverse();
    }
    Some(ring)
}

/// The box of a solid in a board frame.
pub fn bounds_in(frame: &PlaneFrame, s: &Solid) -> Option<(V3, V3)> {
    let mut it = s.positions.iter().map(|p| to_board(frame, *p));
    let first = it.next()?;
    Some(it.fold((first, first), |(lo, hi), p| ([lo[0].min(p[0]), lo[1].min(p[1]), lo[2].min(p[2])], [hi[0].max(p[0]), hi[1].max(p[1]), hi[2].max(p[2])])))
}

/// A closed solid's slice at height `z` of the board frame (its triangles in board coordinates,
/// `p`): the segments where it crosses the plane chained into rings, filled even-odd (layout
/// nm).
fn slice(p: &[V3], indices: &[u32], z: f64) -> Region {
    let key = |q: &V3| ((q[0] * 1e5).round() as i64, (q[1] * 1e5).round() as i64);
    let mut segs: Vec<((i64, i64), (i64, i64))> = Vec::new();
    for t in indices.as_chunks::<3>().0 {
        let v = [p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]];
        let mut cut: Vec<V3> = Vec::with_capacity(2);
        for i in 0..3 {
            let (a, b) = (v[i], v[(i + 1) % 3]);
            if (a[2] < z) != (b[2] < z) {
                let k = (z - a[2]) / (b[2] - a[2]);
                cut.push([a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, z]);
            }
        }
        if let [a, b] = cut[..] {
            let (ka, kb) = (key(&a), key(&b));
            if ka != kb {
                segs.push((ka, kb));
            }
        }
    }
    // Chain them: each point to its segments.
    let mut at: std::collections::HashMap<(i64, i64), Vec<usize>> = Default::default();
    for (i, (a, b)) in segs.iter().enumerate() {
        at.entry(*a).or_default().push(i);
        at.entry(*b).or_default().push(i);
    }
    let mut used = vec![false; segs.len()];
    let mut rings: Region = Vec::new();
    for s0 in 0..segs.len() {
        if used[s0] {
            continue;
        }
        used[s0] = true;
        let start = segs[s0].0;
        let mut ring = vec![start];
        let mut cur = segs[s0].1;
        while cur != start {
            ring.push(cur);
            let Some(&next) = at.get(&cur).and_then(|v| v.iter().find(|i| !used[**i])) else { break };
            used[next] = true;
            cur = if segs[next].0 == cur { segs[next].1 } else { segs[next].0 };
        }
        if ring.len() >= 3 {
            // 1e5 per mm → nm: × 10.
            rings.push(ring.into_iter().map(|(x, y)| Pt::new(x * 10, y * 10)).collect());
        }
    }
    poly::union_even_odd(&rings)
}

/// What of `solids` lies between heights `z0` and `z1` of the board frame `frame`, projected onto
/// the board (layout nm): their faces' parts between the heights, and their slices at both
/// heights (so a wall standing through them counts). Only the parts within `near` (a layout
/// box, mm: x0, y0, x1, y1) are looked at, when given.
pub fn section(solids: &[&Solid], frame: &PlaneFrame, z0: f64, z1: f64, near: Option<[f64; 4]>) -> Region {
    let mut rings: Region = Vec::new();
    for s in solids {
        let Some((lo, hi)) = bounds_in(frame, s) else { continue };
        if hi[2] < z0 || lo[2] > z1 {
            continue;
        }
        if let Some([x0, y0, x1, y1]) = near
            && (hi[0] < x0 || lo[0] > x1 || hi[1] < y0 || lo[1] > y1)
        {
            continue;
        }
        let p: Vec<V3> = s.positions.iter().map(|q| to_board(frame, *q)).collect();
        for t in s.indices.as_chunks::<3>().0 {
            let tri = [p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]];
            if let Some(r) = ring_of(&clip_slab(tri, z0, z1)) {
                rings.push(r);
            }
        }
        for z in [z0, z1] {
            if z > lo[2] && z < hi[2] {
                rings.extend(slice(&p, &s.indices, z));
            }
        }
    }
    poly::union(&rings)
}

/// A solid's shadow on the board of `frame` (its whole projection).
pub fn footprint_of(s: &Solid, frame: &PlaneFrame) -> Region {
    section(&[s], frame, -1e9, 1e9, None)
}

/// A component running into another part (named by the caller's names).
#[derive(Clone, Debug, PartialEq)]
pub struct Collision {
    pub component: String,
    pub other: String,
    /// How far they overlap (mm): the narrower side of the overlap's box.
    pub depth: f64,
}

/// Components (name, solid) whose shadow overlaps the section of another part (name, solid)
/// over the component's own heights, in the board frame `frame`. Parts that only touch (at a face,
/// or overlapping by less than 0.01 mm) don't count.
pub fn collisions(components: &[(&str, &Solid)], others: &[(&str, &Solid)], frame: &PlaneFrame) -> Vec<Collision> {
    let mut out = vec![];
    for (name, c) in components {
        let Some((lo, hi)) = bounds_in(frame, c) else { continue };
        let shadow = footprint_of(c, frame);
        for (other, o) in others {
            let near = [lo[0], lo[1], hi[0], hi[1]];
            // Inside the component's heights: a part ending where it starts only touches it.
            let sec = section(&[o], frame, lo[2] + 1e-3, hi[2] - 1e-3, Some(near));
            if sec.is_empty() {
                continue;
            }
            let overlap = poly::intersection(&shadow, &sec);
            let Some(bb) = overlap.iter().flatten().fold(None, |acc, p| Some(cadrs_eda::units::Bounds::union(acc, cadrs_eda::units::Bounds::of(*p)))) else { continue };
            let depth = cadrs_eda::units::to_mm(bb.size().w.min(bb.size().h));
            if depth > 0.01 {
                out.push(Collision { component: name.to_string(), other: other.to_string(), depth });
            }
        }
    }
    out
}

/// How far the region `of` (a component's shadow, nm) can move along +x, −x, +y and −y before it
/// meets `around` (the parts around, nm), up to `limit` mm; `None` past that.
pub fn gaps(of: &Region, around: &Region, limit: f64) -> [Option<f64>; 4] {
    let Some(bb) = of.iter().flatten().fold(None, |acc, p| Some(cadrs_eda::units::Bounds::union(acc, cadrs_eda::units::Bounds::of(*p)))) else { return [None; 4] };
    // Already touching or overlapping: no room either way.
    if poly::overlaps(of, around) {
        return [Some(0.0); 4];
    }
    let segments: Vec<(Pt, Pt)> = around.iter().flat_map(|r| (0..r.len()).map(move |i| (r[i], r[(i + 1) % r.len()]))).collect();
    // From each side of the box, rays at five places across it.
    let mut out = [None; 4];
    let samples = |a: i64, b: i64| (0..5).map(move |k| a + (b - a) * k / 4);
    let cast = |o: Pt, d: (f64, f64)| -> Option<f64> {
        let mut best: Option<f64> = None;
        for (a, b) in &segments {
            let (ax, ay, bx, by) = (a.x as f64, a.y as f64, b.x as f64, b.y as f64);
            let (ex, ey) = (bx - ax, by - ay);
            let den = d.0 * ey - d.1 * ex;
            if den.abs() < 1e-12 {
                continue;
            }
            let (ox, oy) = (ax - o.x as f64, ay - o.y as f64);
            let t = (ox * ey - oy * ex) / den;
            let u = (ox * d.1 - oy * d.0) / den;
            if t >= 0.0 && (0.0..=1.0).contains(&u) {
                best = Some(best.map_or(t, |x: f64| x.min(t)));
            }
        }
        best.map(|t| t / 1e6)
    };
    let sides: [(Vec<Pt>, (f64, f64)); 4] = [
        (samples(bb.min.y, bb.max.y).map(|y| Pt::new(bb.max.x, y)).collect(), (1.0, 0.0)),
        (samples(bb.min.y, bb.max.y).map(|y| Pt::new(bb.min.x, y)).collect(), (-1.0, 0.0)),
        (samples(bb.min.x, bb.max.x).map(|x| Pt::new(x, bb.max.y)).collect(), (0.0, 1.0)),
        (samples(bb.min.x, bb.max.x).map(|x| Pt::new(x, bb.min.y)).collect(), (0.0, -1.0)),
    ];
    for (i, (starts, d)) in sides.iter().enumerate() {
        out[i] = starts.iter().filter_map(|o| cast(*o, *d)).fold(None, |acc: Option<f64>, t| Some(acc.map_or(t, |a| a.min(t)))).filter(|t| *t <= limit);
    }
    out
}

/// Round features of `solids` standing square to the board of `frame` (a cylinder whose axis
/// is along the board's normal: a pin, a hole) that reach heights `z0`…`z1`: where each axis
/// meets the board (layout, mm) and its radius.
pub fn axes(solids: &[&Solid], frame: &PlaneFrame, z0: f64, z1: f64) -> Vec<([f64; 2], f64)> {
    let mut out: Vec<([f64; 2], f64)> = vec![];
    for s in solids {
        for f in &s.faces {
            let (Some((p, d)), Some(r)) = (f.axis, f.radius) else { continue };
            let d = dir_to_board(frame, d);
            if d[2].abs() < 0.999 {
                continue;
            }
            // The face's own heights.
            let Some(face_z) = face_heights(s, f, frame) else { continue };
            if face_z.1 < z0 || face_z.0 > z1 {
                continue;
            }
            let q = to_board(frame, p);
            if !out.iter().any(|(c, rr)| (c[0] - q[0]).hypot(c[1] - q[1]) < 1e-4 && (rr - r).abs() < 1e-4) {
                out.push(([q[0], q[1]], r));
            }
        }
    }
    out
}

fn face_heights(s: &Solid, f: &crate::solid::SolidFace, frame: &PlaneFrame) -> Option<(f64, f64)> {
    let mut lo = f64::MAX;
    let mut hi = f64::MIN;
    for t in f.first_triangle..f.first_triangle + f.triangle_count {
        for k in 0..3 {
            let i = *s.indices.get(3 * t + k)? as usize;
            let z = to_board(frame, s.positions[i])[2];
            lo = lo.min(z);
            hi = hi.max(z);
        }
    }
    (lo <= hi).then_some((lo, hi))
}

/// An entity to align: a point, an axis (a point on it and its direction) or a plane (a point
/// on it and its normal), in Part Studio coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Entity {
    Point(V3),
    Axis(V3, V3),
    Plane(V3, V3),
}

/// The move in the board's plane (layout mm: dx, dy) that puts `from` (on a component) onto
/// `to`: an axis or point onto an axis or point (their places on the board coincide, when the
/// axes stand square to it; a face lying flat counts as its middle), a plane onto a plane
/// (along their normal, when it lies in the board). `None` when the move can't be made in the board's plane (an axis lying in it, planes
/// parallel to the board, or a plane and an axis).
pub fn align(from: Entity, to: Entity, frame: &PlaneFrame) -> Option<[f64; 2]> {
    let place = |e: Entity| -> Option<V3> {
        match e {
            Entity::Point(p) => Some(to_board(frame, p)),
            Entity::Axis(p, d) => {
                let d = dir_to_board(frame, d);
                (d[2].abs() > 0.999).then(|| to_board(frame, p))
            }
            Entity::Plane(..) => None,
        }
    };
    // A face lying flat (parallel to the board: a pin's or a wire's end) is its middle.
    let flat = |e: Entity| match e {
        Entity::Plane(p, n) if dir_to_board(frame, n)[2].abs() > 0.999 => Entity::Point(p),
        e => e,
    };
    let (from, to) = (flat(from), flat(to));
    match (from, to) {
        (Entity::Plane(p, n), Entity::Plane(q, m)) => {
            let (n, m) = (dir_to_board(frame, n), dir_to_board(frame, m));
            // Normals in the board's plane and parallel (facing or not).
            if n[2].abs() > 1e-3 || (n[0] * m[1] - n[1] * m[0]).abs() > 1e-3 {
                return None;
            }
            let len = n[0].hypot(n[1]);
            let (nx, ny) = (n[0] / len, n[1] / len);
            let d = sub(to_board(frame, q), to_board(frame, p));
            let along = d[0] * nx + d[1] * ny;
            Some([nx * along, ny * along])
        }
        (Entity::Plane(..), _) | (_, Entity::Plane(..)) => None,
        (a, b) => {
            let (p, q) = (place(a)?, place(b)?);
            Some([q[0] - p[0], q[1] - p[1]])
        }
    }
}

/// [`mm`] for whole layout moves (`dx`, `dy` mm).
pub fn delta(d: [f64; 2]) -> Pt {
    Pt::new(mm(d[0]), mm(d[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube(min: [f64; 3], max: [f64; 3]) -> Solid {
        let mut b = cadrs_eda::model3d::Body::default();
        b.cuboid(min, max, [1, 1, 1]);
        crate::pcb::mesh_solid::mesh_solid(&cadrs_eda::model3d::mesh(&b), uuid::Uuid::nil(), 1.0)
    }

    const TOP: PlaneFrame = PlaneFrame { origin: [0.0, 0.0, 10.0], u: [1.0, 0.0, 0.0], v: [0.0, 1.0, 0.0] };

    #[test]
    fn a_section_is_what_lies_between_the_heights() {
        // A 10 × 10 block from 12 to 20 above Top, the board's bottom at z 10: between board
        // heights 0 and 5 (z 10–15) it is there, 10 × 10; between 15 and 20 (z 25–30) it isn't.
        let block = cube([0.0, 0.0, 12.0], [10.0, 10.0, 20.0]);
        let r = section(&[&block], &TOP, 0.0, 5.0, None);
        assert!((poly::area(&r) / 1e12 - 100.0).abs() < 1e-6, "{}", poly::area(&r) / 1e12);
        assert!(section(&[&block], &TOP, 15.0, 20.0, None).is_empty());
    }

    #[test]
    fn a_wall_standing_through_the_heights_is_in_them() {
        // A 2 × 10 wall from z 0 to 30: between board heights 1 and 3 (z 11–13) it is its
        // cross-section, though none of its faces lie flat there.
        let wall = cube([4.0, 0.0, 0.0], [6.0, 10.0, 30.0]);
        let r = section(&[&wall], &TOP, 1.0, 3.0, None);
        assert!((poly::area(&r) / 1e12 - 20.0).abs() < 1e-6, "{}", poly::area(&r) / 1e12);
    }

    #[test]
    fn gaps_are_the_free_distances_each_way() {
        // A 2 × 2 component at (0..2, 0..2); a wall from x 7 to 8 across; nothing elsewhere.
        let comp = vec![vec![Pt::mm(0.0, 0.0), Pt::mm(2.0, 0.0), Pt::mm(2.0, 2.0), Pt::mm(0.0, 2.0)]];
        let wall = vec![vec![Pt::mm(7.0, -10.0), Pt::mm(8.0, -10.0), Pt::mm(8.0, 10.0), Pt::mm(7.0, 10.0)]];
        let g = gaps(&comp, &wall, 50.0);
        assert!((g[0].unwrap() - 5.0).abs() < 1e-6);
        assert_eq!(g[1], None);
        assert_eq!(g[2], None);
    }

    #[test]
    fn a_component_in_a_wall_collides_by_its_overlap() {
        let comp = cube([0.0, 0.0, 11.6], [4.0, 4.0, 14.0]);
        let wall = cube([3.0, -5.0, 0.0], [10.0, 10.0, 30.0]);
        let far = cube([30.0, 0.0, 0.0], [31.0, 1.0, 30.0]);
        let c = collisions(&[("J1", &comp)], &[("Wall", &wall), ("Far", &far)], &TOP);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].other, "Wall");
        assert!((c[0].depth - 1.0).abs() < 1e-6, "{}", c[0].depth);
    }

    #[test]
    fn a_pin_square_to_the_board_aligns_onto_a_hole() {
        // A pin along z at (3, 4); a hole's axis along z at (10, 2): move (7, −2).
        let d = align(Entity::Axis([3.0, 4.0, 11.0], [0.0, 0.0, 1.0]), Entity::Axis([10.0, 2.0, 0.0], [0.0, 0.0, -1.0]), &TOP).unwrap();
        assert!((d[0] - 7.0).abs() < 1e-9 && (d[1] + 2.0).abs() < 1e-9);
        // A face onto a face along x: only the x part of the offset moves.
        let d = align(Entity::Plane([2.0, 5.0, 11.0], [1.0, 0.0, 0.0]), Entity::Plane([9.0, -3.0, 0.0], [-1.0, 0.0, 0.0]), &TOP).unwrap();
        assert!((d[0] - 7.0).abs() < 1e-9 && d[1].abs() < 1e-9);
        // A wire's flat end (its middle) onto a pin's axis.
        let d = align(Entity::Plane([1.0, 1.0, 15.0], [0.0, 0.0, 1.0]), Entity::Axis([4.0, -1.0, 0.0], [0.0, 0.0, 1.0]), &TOP).unwrap();
        assert!((d[0] - 3.0).abs() < 1e-9 && (d[1] + 2.0).abs() < 1e-9);
        // An axis lying in the board can't be met by a move in it.
        assert!(align(Entity::Axis([0.0; 3], [1.0, 0.0, 0.0]), Entity::Point([1.0, 1.0, 1.0]), &TOP).is_none());
    }

    #[test]
    fn pins_standing_up_are_found_with_their_place_and_radius() {
        let mut b = cadrs_eda::model3d::Body::default();
        b.cylinder([5.0, -2.0, 0.0], cadrs_eda::model3d::Axis::Z, 1.2, 20.0, [1, 1, 1]);
        let pin = crate::pcb::mesh_solid::mesh_solid(&cadrs_eda::model3d::mesh(&b), uuid::Uuid::nil(), 1.0);
        let a = axes(&[&pin], &TOP, -1.0, 3.0);
        assert_eq!(a.len(), 1);
        assert!((a[0].0[0] - 5.0).abs() < 1e-3 && (a[0].0[1] + 2.0).abs() < 1e-3 && (a[0].1 - 1.2).abs() < 1e-2, "{a:?}");
        // Not reaching those heights: none.
        assert!(axes(&[&pin], &TOP, 15.0, 20.0).is_empty());
    }
}
