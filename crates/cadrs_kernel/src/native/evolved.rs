//! Evolved solids (OCCT's `BRepOffsetAPI_MakeEvolved` in volume mode, DRAW's `evolved -v`): a
//! profile region swept along a planar spine, held square to it, with its x measured outwards
//! from the spine in the spine's plane and its y up from it.
//!
//! Each spine edge sweeps the profile exactly: a line a prism of it, a circular arc a revolution
//! about the arc's axis. At a corner where the profile's side of the spine turns away (a left
//! turn of a counter-clockwise spine for a profile outside it, x ≥ 0; a right turn for one
//! inside, x ≤ 0) the gap is filled by the profile turned about the corner's vertical axis, so
//! offsets of the spine round off as OCCT's do (arc joins); where it turns in, the neighbouring
//! sweeps overlap. The pieces are united. A closed spine is taken counter-clockwise about its
//! plane's normal, which is "up".

use super::curve::Curve;
use super::kernel::NativeKernel;
use super::math::*;
use crate::*;

/// A spine edge in the direction of travel.
struct SpineEdge {
    c: Curve,
    t0: f64,
    t1: f64,
}

impl SpineEdge {
    fn point(&self, s: f64) -> P3 {
        self.c.point(self.t0 + (self.t1 - self.t0) * s)
    }
    fn dir(&self, s: f64) -> V3 {
        (self.c.tangent(self.t0 + (self.t1 - self.t0) * s) * (self.t1 - self.t0).signum()).normalize()
    }
}

/// The evolved solid of `profile` (closed regions in the profile's (x, y): outwards, up) along
/// `spine` (lines and circular arcs, chained in order, in one plane). The profile must lie on
/// one side of x = 0. Returns the solid's body.
pub fn evolved(k: &mut NativeKernel, spine: &[(Curve, f64, f64)], profile: &[Region]) -> Result<BodyId> {
    if spine.is_empty() || profile.is_empty() {
        return Err(KernelError::InvalidParameter("an evolved solid needs a spine and a profile".into()));
    }
    let edges: Vec<SpineEdge> = spine.iter().map(|(c, t0, t1)| SpineEdge { c: c.clone(), t0: *t0, t1: *t1 }).collect();
    for e in &edges {
        if !matches!(e.c, Curve::Line { .. } | Curve::Circle { .. }) {
            return Err(KernelError::Unsupported("evolved solids along spines of curves other than lines and arcs"));
        }
    }
    let n = edges.len();
    let closed = (edges[n - 1].point(1.0) - edges[0].point(0.0)).norm() < 1e-7 * (1.0 + edges[0].point(0.0).coords.norm());
    // The spine's plane normal ("up"): counter-clockwise for a closed spine.
    let pts: Vec<P3> = edges.iter().flat_map(|e| (0..16).map(move |i| e.point(i as f64 / 16.0))).collect();
    let mut up = V3::zeros();
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        up += (a - pts[0]).cross(&(b - pts[0]));
    }
    if up.norm() < 1e-12 {
        // An open straight spine: any normal square to it.
        up = super::axes::ax2_x(&edges[0].dir(0.0));
    }
    let up = up.normalize();
    let size = pts.iter().map(|p| (p - pts[0]).norm()).fold(0.0, f64::max).max(1e-9);
    if pts.iter().any(|p| (p - pts[0]).dot(&up).abs() > 1e-6 * size) {
        return Err(KernelError::InvalidParameter("an evolved solid's spine must be planar".into()));
    }
    // Which side of the spine the profile lies on.
    let mut xs = (f64::INFINITY, f64::NEG_INFINITY);
    for r in profile {
        for c in r.outer.curves.iter().chain(r.holes.iter().flat_map(|h| h.curves.iter())) {
            for i in 0..=8 {
                let p = c.point_at(i as f64 / 8.0);
                xs = (xs.0.min(p.x), xs.1.max(p.x));
            }
        }
    }
    let outside = if xs.0 >= -1e-9 {
        true
    } else if xs.1 <= 1e-9 {
        false
    } else {
        return Err(KernelError::Unsupported("evolved solids of profiles on both sides of the spine"));
    };
    // The profile placed at a point of the spine with travel direction t: x outwards (t × up).
    let place = |p: P3, t: V3| -> Profile {
        let x = t.cross(&up).normalize();
        Profile::new(Plane { origin: p, x_dir: nalgebra::Unit::new_normalize(x), normal: nalgebra::Unit::new_normalize(x.cross(&up)) }, profile.to_vec())
    };
    let mut pieces: Vec<BodyId> = Vec::new();
    for (i, e) in edges.iter().enumerate() {
        let p0 = e.point(0.0);
        let prof = place(p0, e.dir(0.0));
        let body = match &e.c {
            Curve::Line { .. } => super::features::prism_along(k, &prof, e.point(1.0) - p0)?.bodies[0],
            Curve::Circle { c, x, y, .. } => {
                // About the arc's axis, the way it runs.
                let ax = x.cross(y) * (e.t1 - e.t0).signum();
                let sweep = (e.t1 - e.t0).abs();
                crate::Kernel::revolve(k, &prof, Axis { origin: *c, dir: nalgebra::Unit::new_normalize(ax) }, sweep)?.bodies[0]
            }
            _ => unreachable!(),
        };
        pieces.push(body);
        // The corner at the edge's end.
        if i + 1 == n && !closed {
            continue;
        }
        let next = &edges[(i + 1) % n];
        let (ta, tb) = (e.dir(1.0), next.dir(0.0));
        let turn = ta.cross(&tb).dot(&up).atan2(ta.dot(&tb));
        if turn.abs() < 1e-9 || (turn > 0.0) != outside {
            continue;
        }
        let v = e.point(1.0);
        let prof = place(v, ta);
        let axis = Axis { origin: v, dir: nalgebra::Unit::new_normalize(up * turn.signum()) };
        pieces.push(crate::Kernel::revolve(k, &prof, axis, turn.abs())?.bodies[0]);
    }
    let mut acc = pieces[0];
    for &p in &pieces[1..] {
        acc = crate::Kernel::boolean(k, BoolOp::Union, acc, &[p])?.bodies[0];
    }
    Ok(acc)
}
