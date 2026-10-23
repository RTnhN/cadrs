//! A bounding-volume hierarchy over boxes (faces, edges, vertices): a static binary tree split
//! at the median of the longest axis, leaves of up to four boxes. Queries return the indices of
//! the boxes overlapping a box, or every overlapping pair between two trees, so the boolean,
//! sewing and classification visit only nearby entities instead of all of them.

use super::math::*;
use super::tol;

#[derive(Debug, Clone)]
enum Node {
    Leaf {
        bb: Bbox,
        items: Vec<usize>,
    },
    Inner {
        bb: Bbox,
        l: Box<Node>,
        r: Box<Node>,
    },
}

impl Node {
    fn bb(&self) -> &Bbox {
        match self {
            Node::Leaf { bb, .. } | Node::Inner { bb, .. } => bb,
        }
    }
}

/// A tree over boxes; an index refers to the box's position in the slice it was built from.
#[derive(Debug, Clone)]
pub struct Bvh {
    root: Option<Node>,
    boxes: Vec<Bbox>,
}

const LEAF: usize = 4;

fn build(boxes: &[Bbox], mut items: Vec<usize>) -> Node {
    let mut bb = Bbox::empty();
    for &i in &items {
        bb.union(&boxes[i]);
    }
    if items.len() <= LEAF {
        return Node::Leaf { bb, items };
    }
    // Split at the median centre along the longest axis of the centres' box.
    let mut cb = Bbox::empty();
    for &i in &items {
        cb.add(&boxes[i].center());
    }
    let ext = cb.max - cb.min;
    let axis = if ext.x >= ext.y && ext.x >= ext.z {
        0
    } else if ext.y >= ext.z {
        1
    } else {
        2
    };
    let key = |i: &usize| boxes[*i].center()[axis];
    let mid = items.len() / 2;
    items.select_nth_unstable_by(mid, |a, b| key(a).total_cmp(&key(b)));
    let right = items.split_off(mid);
    Node::Inner {
        bb,
        l: Box::new(build(boxes, items)),
        r: Box::new(build(boxes, right)),
    }
}

impl Bvh {
    /// The tree over `boxes` (empty boxes are left out of every query).
    pub fn new(boxes: Vec<Bbox>) -> Self {
        let items: Vec<usize> = (0..boxes.len()).filter(|&i| !boxes[i].is_empty()).collect();
        let root = (!items.is_empty()).then(|| build(&boxes, items));
        Bvh { root, boxes }
    }

    pub fn len(&self) -> usize {
        self.boxes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    pub fn bbox(&self, i: usize) -> &Bbox {
        &self.boxes[i]
    }

    /// The boxes overlapping `q`, in no particular order.
    pub fn query(&self, q: &Bbox) -> Vec<usize> {
        let mut out = Vec::new();
        let Some(root) = &self.root else { return out };
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            if !n.bb().overlaps(q) {
                continue;
            }
            match n {
                Node::Leaf { items, .. } => {
                    out.extend(items.iter().copied().filter(|&i| self.boxes[i].overlaps(q)))
                }
                Node::Inner { l, r, .. } => {
                    stack.push(l);
                    stack.push(r);
                }
            }
        }
        out
    }

    /// The boxes a ray from `o` along `d` (for t in [t0, t1]) passes through.
    pub fn ray(&self, o: &P3, d: &V3, t0: f64, t1: f64) -> Vec<usize> {
        let mut out = Vec::new();
        let Some(root) = &self.root else { return out };
        let hits = |bb: &Bbox| ray_box(bb, o, d, t0, t1);
        let mut stack = vec![root];
        while let Some(n) = stack.pop() {
            if !hits(n.bb()) {
                continue;
            }
            match n {
                Node::Leaf { items, .. } => {
                    out.extend(items.iter().copied().filter(|&i| hits(&self.boxes[i])))
                }
                Node::Inner { l, r, .. } => {
                    stack.push(l);
                    stack.push(r);
                }
            }
        }
        out
    }

    /// Every pair (i in self, j in other) whose boxes overlap.
    pub fn pairs(&self, other: &Bvh) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let (Some(a), Some(b)) = (&self.root, &other.root) else {
            return out;
        };
        let mut stack = vec![(a, b)];
        while let Some((x, y)) = stack.pop() {
            if !x.bb().overlaps(y.bb()) {
                continue;
            }
            match (x, y) {
                (Node::Leaf { items: ia, .. }, Node::Leaf { items: ib, .. }) => {
                    for &i in ia {
                        for &j in ib {
                            if self.boxes[i].overlaps(&other.boxes[j]) {
                                out.push((i, j));
                            }
                        }
                    }
                }
                (Node::Inner { l, r, .. }, Node::Leaf { .. }) => {
                    stack.push((l, y));
                    stack.push((r, y));
                }
                (Node::Leaf { .. }, Node::Inner { l, r, .. }) => {
                    stack.push((x, l));
                    stack.push((x, r));
                }
                (
                    Node::Inner {
                        l: xl,
                        r: xr,
                        bb: xb,
                    },
                    Node::Inner {
                        l: yl,
                        r: yr,
                        bb: yb,
                    },
                ) => {
                    // Descend the bigger box first.
                    if xb.diagonal() >= yb.diagonal() {
                        stack.push((xl, y));
                        stack.push((xr, y));
                    } else {
                        stack.push((x, yl));
                        stack.push((x, yr));
                    }
                }
            }
        }
        out
    }
}

/// Whether the segment of the ray o + t·d, t in [t0, t1], meets the box (slab test).
pub fn ray_box(bb: &Bbox, o: &P3, d: &V3, t0: f64, t1: f64) -> bool {
    let (mut lo, mut hi) = (t0, t1);
    for k in 0..3 {
        if d[k].abs() < tol::TINY {
            if o[k] < bb.min[k] || o[k] > bb.max[k] {
                return false;
            }
            continue;
        }
        let (a, b) = ((bb.min[k] - o[k]) / d[k], (bb.max[k] - o[k]) / d[k]);
        let (a, b) = if a < b { (a, b) } else { (b, a) };
        lo = lo.max(a);
        hi = hi.min(b);
        if lo > hi {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube(x: f64, y: f64, z: f64, s: f64) -> Bbox {
        let mut b = Bbox::empty();
        b.add(&P3::new(x, y, z));
        b.add(&P3::new(x + s, y + s, z + s));
        b
    }

    #[test]
    fn queries_match_brute_force() {
        let boxes: Vec<Bbox> = (0..300)
            .map(|i| {
                let f = i as f64;
                cube(
                    (f * 7.3) % 50.0,
                    (f * 3.1) % 40.0,
                    (f * 1.7) % 30.0,
                    1.0 + (f % 5.0),
                )
            })
            .collect();
        let t = Bvh::new(boxes.clone());
        for q in [
            cube(10.0, 10.0, 10.0, 5.0),
            cube(-3.0, 0.0, 0.0, 2.0),
            cube(0.0, 0.0, 0.0, 60.0),
        ] {
            let mut got = t.query(&q);
            got.sort();
            let want: Vec<usize> = (0..boxes.len())
                .filter(|&i| boxes[i].overlaps(&q))
                .collect();
            assert_eq!(got, want);
        }
        let other: Vec<Bbox> = (0..120)
            .map(|i| cube((i as f64 * 5.9) % 45.0, (i as f64 * 2.3) % 35.0, 5.0, 2.0))
            .collect();
        let u = Bvh::new(other.clone());
        let mut got = t.pairs(&u);
        got.sort();
        let mut want = Vec::new();
        for (i, a) in boxes.iter().enumerate() {
            for (j, b) in other.iter().enumerate() {
                if a.overlaps(b) {
                    want.push((i, j));
                }
            }
        }
        assert_eq!(got, want);
        let (o, d) = (P3::new(-5.0, 20.0, 15.0), V3::new(1.0, 0.0, 0.0));
        let mut got = t.ray(&o, &d, 0.0, 100.0);
        got.sort();
        let want: Vec<usize> = (0..boxes.len())
            .filter(|&i| ray_box(&boxes[i], &o, &d, 0.0, 100.0))
            .collect();
        assert_eq!(got, want);
    }
}
