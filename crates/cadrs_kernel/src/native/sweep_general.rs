//! Sweeps along general path curves (B-splines, ellipses, arcs the profile isn't square to,
//! curves out of a plane) with sampled frames: the profile is carried along the segment by
//! rotation-minimizing frames (double reflection; or no rotation, or a locked direction), each
//! profile piece is placed at sections along the segment and the sections are skinned (cubic
//! interpolation of the homogeneous poles, so conics stay exact in each section). The rails
//! (the paths of the profile's corners) are interpolated with the same parameters and degree,
//! so they are exactly the side surfaces' boundary iso-curves. The generic sweep builder
//! assembles the faces and caps.

use super::build::Built;
use super::curve::Curve;
use super::math::*;
use super::nurbs::{NurbsCurve, NurbsSurface, H};
use super::profile::{MergedFace, Piece};
use super::surface::Surface;
use super::sweep::{SideGeo, SweepKind};
use super::sweep_path::Seg;
use crate::*;
use std::sync::Arc;

/// Rotation-minimizing steps per segment (the frames between them are interpolated).
const FINE: usize = 256;

/// A sweep along a run of path segments meeting tangentially; s ∈ [0, 1] runs along them by
/// length.
pub struct PathSweep {
    plane: Plane,
    segs: Vec<Seg>,
    /// The segments' starts in s (and 1 at the end).
    cum: Vec<f64>,
    /// The path point and frame rotation (relative to the start frame) at the fine steps.
    pts: Vec<P3>,
    rots: Vec<nalgebra::UnitQuaternion<f64>>,
    /// The sections' parameters.
    v: Vec<f64>,
    deg: usize,
    /// How far the profile reaches from the path (for the section count).
    reach: f64,
    /// The helix the path follows (within its approximation), swept by its exact screw motion.
    helix: Option<Helix>,
}

/// A helix: axis through `o` along unit `a`, radius `r`, rising `c` per radian, `e1`/`e2`
/// square to `a`; angle `th0` where the path starts.
#[derive(Debug, Clone, Copy)]
pub struct Helix {
    o: P3,
    a: V3,
    e1: V3,
    e2: V3,
    r: f64,
    c: f64,
    th0: f64,
}

impl Helix {
    fn point(&self, th: f64) -> P3 {
        self.o + (self.e1 * th.cos() + self.e2 * th.sin()) * self.r + self.a * (self.c * th)
    }
    fn tangent(&self, th: f64) -> V3 {
        ((-self.e1 * th.sin() + self.e2 * th.cos()) * self.r + self.a * self.c).normalize()
    }
    /// The angle of a point near the helix, unwrapped next to `near`.
    fn angle(&self, p: &P3, near: f64) -> f64 {
        let w = p - self.o;
        let a = w.dot(&self.e2).atan2(w.dot(&self.e1));
        a + ((near - a) / TAU).round() * TAU
    }
    /// The screw motion that turns the helix by `phi` along itself.
    fn screw(&self, phi: f64) -> Affine {
        let m = nalgebra::Rotation3::from_axis_angle(&nalgebra::Unit::new_normalize(self.a), phi).into_inner();
        Affine { m, t: self.o.coords - m * self.o.coords + self.a * (self.c * phi) }
    }
}

/// The helix a chain of path segments follows (within its approximation), if any.
pub fn fit_helix_segs(segs: &[Seg]) -> Option<Helix> {
    let mut pts = Vec::new();
    let mut tans = Vec::new();
    for g in segs {
        for i in 0..64 {
            let f = i as f64 / 64.0;
            pts.push(g.c.point(travel_t(g, f)));
            tans.push(g.dir(f));
        }
    }
    let last = segs.last()?;
    pts.push(last.end());
    tans.push(last.dir(1.0));
    let size = pts.iter().map(|p| (p - pts[0]).norm()).fold(0.0, f64::max);
    fit_helix(&pts, &tans, 1e-4 * (1.0 + size))
}

/// The helix through points of a path (with their travel fractions), when they all lie on one
/// within `tol` (an approximated helix: Bézier pieces through points of it).
fn fit_helix(pts: &[P3], tans: &[V3], tol: f64) -> Option<Helix> {
    let n = pts.len();
    if n < 8 {
        return None;
    }
    // The axis: the tangents make one angle with it, so it is square to their differences.
    let mean = tans.iter().fold(V3::zeros(), |acc, t| acc + t) / n as f64;
    let mut m = nalgebra::Matrix3::zeros();
    for t in tans {
        let d = t - mean;
        m += d * d.transpose();
    }
    let eig = m.symmetric_eigen();
    let k = (0..3).min_by(|&x, &y| eig.eigenvalues[x].total_cmp(&eig.eigenvalues[y])).unwrap();
    let mut a = eig.eigenvectors.column(k).into_owned().normalize();
    // The tangents must turn round it (not a straight line or a plane curve's single direction).
    if eig.eigenvalues.iter().filter(|e| **e > 1e-6 * n as f64).count() < 2 {
        return None;
    }
    if mean.dot(&a) < 0.0 {
        a = -a;
    }
    // The circle of the points projected square to the axis.
    let fr = Frame::from_z(P3::origin(), a);
    let mut mm = nalgebra::Matrix3::zeros();
    let mut rhs = V3::zeros();
    for p in pts {
        let (x, y) = (p.coords.dot(&fr.x), p.coords.dot(&fr.y));
        let row = V3::new(x, y, 1.0);
        mm += row * row.transpose();
        rhs += row * (x * x + y * y);
    }
    let sol = mm.lu().solve(&rhs)?;
    let (cx, cy) = (sol.x / 2.0, sol.y / 2.0);
    let r = (sol.z + cx * cx + cy * cy).sqrt();
    if !(r.is_finite() && r > 1e-9) {
        return None;
    }
    let o0 = P3::from(fr.x * cx + fr.y * cy);
    let h0 = Helix { o: o0, a, e1: fr.x, e2: fr.y, r, c: 0.0, th0: 0.0 };
    // Angles (unwrapped) and heights: z = c θ + z0 by least squares.
    let mut ths = Vec::with_capacity(n);
    let mut prev = h0.angle(&pts[0], 0.0);
    for p in pts {
        prev = h0.angle(p, prev);
        ths.push(prev);
    }
    let zs: Vec<f64> = pts.iter().map(|p| (p - o0).dot(&a)).collect();
    let (st, sz) = (ths.iter().sum::<f64>() / n as f64, zs.iter().sum::<f64>() / n as f64);
    let (mut num, mut den) = (0.0, 0.0);
    for (t, z) in ths.iter().zip(&zs) {
        num += (t - st) * (z - sz);
        den += (t - st) * (t - st);
    }
    if den < 1e-12 {
        return None;
    }
    let c = num / den;
    let z0 = sz - c * st;
    let o = o0 + a * z0;
    let h = Helix { o, a, e1: fr.x, e2: fr.y, r, c, th0: ths[0] };
    for (p, t) in pts.iter().zip(&ths) {
        if (h.point(*t) - p).norm() > tol {
            return None;
        }
    }
    Some(h)
}

fn frame_of(t: &V3, r: &V3) -> nalgebra::Rotation3<f64> {
    let t = t.normalize();
    let r = (r - t * r.dot(&t)).normalize();
    nalgebra::Rotation3::from_matrix_unchecked(nalgebra::Matrix3::from_columns(&[t, r, t.cross(&r)]))
}

/// Any unit vector square to `t`.
fn square_to(t: &V3) -> V3 {
    let a = if t.x.abs() < 0.9 { V3::x() } else { V3::y() };
    t.cross(&a).normalize()
}

impl PathSweep {
    pub fn new(plane: Plane, segs: &[Seg], control: SweepControl, reach: f64, helix: Option<Helix>) -> Result<Self> {
        let lens: Vec<f64> = segs.iter().map(|g| g.length()).collect();
        let total: f64 = lens.iter().sum();
        let mut cum = vec![0.0];
        for l in &lens {
            cum.push(cum.last().unwrap() + l / total);
        }
        *cum.last_mut().unwrap() = 1.0;
        let fine = FINE * segs.len();
        let mut me = Self { plane, segs: segs.to_vec(), cum, pts: vec![], rots: vec![], v: vec![], deg: 3, reach, helix: None };
        let pts: Vec<P3> = (0..=fine).map(|i| me.at(i as f64 / fine as f64)).collect();
        let tans: Vec<V3> = (0..=fine).map(|i| me.dir(i as f64 / fine as f64)).collect();
        if tans.iter().any(|t| !t.iter().all(|x| x.is_finite())) {
            return Err(KernelError::InvalidParameter("The sweep path has a point with no direction".into()));
        }
        let mut frames: Vec<nalgebra::Rotation3<f64>> = Vec::with_capacity(fine + 1);
        // A helix (within its Bézier approximation): the exact screw motion, of the helix the
        // caller fitted to the whole chain (so neighbouring pieces share it), else this run's.
        if control == SweepControl::None {
            me.helix = match helix {
                Some(h) => Some(Helix { th0: h.angle(&pts[0], h.th0), ..h }),
                None => {
                    let size = pts.iter().map(|p| (p - pts[0]).norm()).fold(0.0, f64::max);
                    fit_helix(&pts, &tans, 1e-4 * (1.0 + size))
                }
            };
        }
        let (pts, tans) = match me.helix {
            Some(_) => {
                let p: Vec<P3> = (0..=fine).map(|i| me.at(i as f64 / fine as f64)).collect();
                let t: Vec<V3> = (0..=fine).map(|i| me.dir(i as f64 / fine as f64)).collect();
                (p, t)
            }
            None => (pts, tans),
        };
        match control {
            SweepControl::None if me.helix.is_some() => {
                let h = me.helix.unwrap();
                let mut prev = h.th0;
                for p in &pts {
                    prev = h.angle(p, prev);
                    frames.push(nalgebra::Rotation3::from_axis_angle(&nalgebra::Unit::new_normalize(h.a), prev - h.th0));
                }
            }
            SweepControl::KeepOrientation => {}
            SweepControl::LockDirection(d) => {
                for t in &tans {
                    let r = d.into_inner() - t * d.dot(t);
                    if r.norm() < 1e-9 {
                        return Err(KernelError::InvalidParameter("The sweep path runs along the locked direction".into()));
                    }
                    frames.push(frame_of(t, &r));
                }
            }
            SweepControl::None if me.frenet(&tans).is_some() => {
                // A path that bends everywhere (a helix): its principal normal, so the profile
                // keeps its place relative to the curve (a thread stays on its cylinder).
                for (t, n) in tans.iter().zip(me.frenet(&tans).unwrap()) {
                    frames.push(frame_of(t, &n));
                }
            }
            SweepControl::None => {
                // Double reflection (Wang, Jüttler, Zheng & Liu 2008).
                let mut r = square_to(&tans[0]);
                frames.push(frame_of(&tans[0], &r));
                for i in 0..fine {
                    let v1 = pts[i + 1] - pts[i];
                    let c1 = v1.norm_squared();
                    if c1 < 1e-24 {
                        frames.push(frame_of(&tans[i + 1], &r));
                        continue;
                    }
                    let rl = r - v1 * (2.0 / c1) * v1.dot(&r);
                    let tl = tans[i] - v1 * (2.0 / c1) * v1.dot(&tans[i]);
                    let v2 = tans[i + 1] - tl;
                    let c2 = v2.norm_squared();
                    r = if c2 < 1e-24 { rl } else { rl - v2 * (2.0 / c2) * v2.dot(&rl) };
                    frames.push(frame_of(&tans[i + 1], &r));
                }
            }
        }
        let rots: Vec<nalgebra::UnitQuaternion<f64>> = if frames.is_empty() {
            vec![nalgebra::UnitQuaternion::identity(); fine + 1]
        } else {
            let f0 = frames[0].inverse();
            frames.iter().map(|f| nalgebra::UnitQuaternion::from_rotation_matrix(&(f * f0))).collect()
        };
        // Sections: more where the path turns or twists.
        let mut turn = 0.0;
        for i in 0..fine {
            turn += rots[i].angle_to(&rots[i + 1]) + tans[i].angle(&tans[i + 1]);
        }
        // Enough sections for the cubic interpolation between them to stay within about 1e-8
        // of the true sweep: spacing h with h⁴κ³/384 ≤ tol along rails of curvature κ (the
        // path's, grown by the profile's reach from it).
        let reach = me.profile_reach();
        let mut kmax: f64 = 0.0;
        for i in 0..fine {
            let ds = (pts[i + 1] - pts[i]).norm();
            if ds > 1e-12 {
                kmax = kmax.max(tans[i].angle(&tans[i + 1]) / ds);
            }
        }
        let length: f64 = (0..fine).map(|i| (pts[i + 1] - pts[i]).norm()).sum();
        let k_rail = kmax * (1.0 + reach * kmax);
        let by_err = if k_rail > 1e-12 {
            let h = (384.0 * 1e-8 / k_rail.powi(3)).powf(0.25);
            length / h
        } else {
            0.0
        };
        let n = (8.0 + turn / (PI / 24.0)).max(by_err).ceil().clamp(8.0, 2000.0) as usize;
        me.v = (0..=n).map(|k| k as f64 / n as f64).collect();
        me.pts = pts;
        me.rots = rots;
        Ok(me)
    }

    /// How far the profile reaches from the path's start (an upper bound, from the profile
    /// plane's frame: set by the caller through `reach`).
    fn profile_reach(&self) -> f64 {
        self.reach
    }

    /// The principal normals at the fine steps when the path bends everywhere along it (its
    /// curvature never drops near zero and the normal turns smoothly), else None.
    fn frenet(&self, tans: &[V3]) -> Option<Vec<V3>> {
        let fine = tans.len() - 1;
        let mut size = 0.0f64;
        for g in &self.segs {
            size = size.max((g.end() - g.start()).norm()).max(g.length() / TAU);
        }
        let mut out: Vec<V3> = Vec::with_capacity(fine + 1);
        for i in 0..=fine {
            let (k, f) = self.seg_at(i as f64 / fine as f64);
            let g = &self.segs[k];
            let (_, d1, d2) = g.c.d2(travel_t(g, f));
            let l2 = d1.norm_squared();
            if l2 < 1e-300 {
                return None;
            }
            let kn = (d2 - d1 * (d1.dot(&d2) / l2)) / l2;
            if kn.norm() * size.max(1e-9) < 1e-3 {
                return None;
            }
            let n = kn.normalize();
            if out.last().is_some_and(|m: &V3| m.dot(&n) < 0.5) {
                return None;
            }
            out.push(n);
        }
        Some(out)
    }

    /// The segment and its travel fraction at s.
    fn seg_at(&self, s: f64) -> (usize, f64) {
        let s = s.clamp(0.0, 1.0);
        let k = self.cum.partition_point(|c| *c <= s).saturating_sub(1).min(self.segs.len() - 1);
        let (a, b) = (self.cum[k], self.cum[k + 1]);
        (k, if b > a { ((s - a) / (b - a)).clamp(0.0, 1.0) } else { 0.0 })
    }
    fn path_at(&self, s: f64) -> P3 {
        let (k, f) = self.seg_at(s);
        self.segs[k].c.point(travel_t(&self.segs[k], f))
    }
    /// The helix's angle at s (from the path's point there).
    fn helix_angle(&self, h: &Helix, s: f64) -> f64 {
        // Unwrap along the path from its start, in steps a quarter turn at most apart.
        let n = 64 * self.segs.len();
        let mut th = h.th0;
        let steps = ((s.clamp(0.0, 1.0) * n as f64).ceil() as usize).max(1);
        for i in 1..=steps {
            th = h.angle(&self.path_at(s * i as f64 / steps as f64), th);
        }
        th
    }
    fn at(&self, s: f64) -> P3 {
        match &self.helix {
            Some(h) => h.point(self.helix_angle(h, s)),
            None => self.path_at(s),
        }
    }
    fn dir(&self, s: f64) -> V3 {
        match &self.helix {
            Some(h) => {
                let (k, f) = self.seg_at(s);
                let t = h.tangent(self.helix_angle(h, s));
                // In the direction of travel.
                if t.dot(&self.segs[k].dir(f)) < 0.0 { -t } else { t }
            }
            None => {
                let (k, f) = self.seg_at(s);
                self.segs[k].dir(f)
            }
        }
    }

    /// The rigid motion of the profile from the start of the segment to s.
    fn motion(&self, s: f64) -> Affine {
        if let Some(h) = &self.helix {
            return h.screw(self.helix_angle(h, s) - h.th0);
        }
        let fine = self.pts.len() - 1;
        let x = s.clamp(0.0, 1.0) * fine as f64;
        let i = (x.floor() as usize).min(fine - 1);
        let f = x - i as f64;
        let q = self.rots[i].slerp(&self.rots[i + 1], f);
        let p = self.at(s);
        let m = q.to_rotation_matrix().into_inner();
        Affine { m, t: p.coords - m * self.pts[0].coords }
    }

    /// The profile's plane where the segment ends.
    pub fn end_plane(&self) -> Plane {
        let m = self.motion(1.0);
        Plane { origin: m.point(&self.plane.origin), x_dir: nalgebra::Unit::new_normalize(m.vector(&self.plane.x_dir)), normal: nalgebra::Unit::new_normalize(m.vector(&self.plane.normal)) }
    }

    fn rail_nurbs(&self, q: P2) -> NurbsCurve {
        let pts: Vec<P3> = self.v.iter().map(|&s| self.place(s).point(&P3::new(q.x, q.y, 0.0))).collect();
        NurbsCurve::interpolate(&pts, &self.v, self.deg.min(pts.len() - 1), None, None)
    }

    /// The side surface of a piece: its NURBS placed at the sections, skinned.
    fn skin(&self, piece: &Piece) -> NurbsSurface {
        let n0 = piece.c.to_nurbs(piece.t0, piece.t1);
        let curves: Vec<NurbsCurve> = self.v.iter().map(|&s| n0.transformed(&self.place(s))).collect();
        let q = self.deg.min(curves.len() - 1);
        let ncp = n0.cps.len();
        let mut cols: Vec<Vec<H>> = Vec::with_capacity(ncp);
        let mut kv = Vec::new();
        for i in 0..ncp {
            let xyz: Vec<P3> = curves.iter().map(|c| P3::new(c.cps[i].x, c.cps[i].y, c.cps[i].z)).collect();
            let a = NurbsCurve::interpolate(&xyz, &self.v, q, None, None);
            let w = n0.cps[i].w;
            kv = a.knots.clone();
            cols.push(a.cps.iter().map(|p| H::new(p.x, p.y, p.z, p.w * w)).collect());
        }
        NurbsSurface { du: n0.degree, dv: q, ku: n0.knots.clone(), kv, cps: cols }
    }
}

/// The curve parameter at travel fraction s of a segment.
fn travel_t(seg: &Seg, s: f64) -> f64 {
    if seg.rev { seg.t1 + (seg.t0 - seg.t1) * s } else { seg.t0 + (seg.t1 - seg.t0) * s }
}

impl SweepKind for PathSweep {
    fn place(&self, s: f64) -> Affine {
        let base = super::build::plane_map(&self.plane, &V3::zeros(), 0.0);
        let m = self.motion(s);
        Affine { m: m.m * base.m, t: m.m * base.t + m.t }
    }
    fn rail(&self, q: P2) -> Option<Curve> {
        Some(Curve::Nurbs(Arc::new(self.rail_nurbs(q))))
    }
    fn side(&self, piece: &Piece, _c3: &Curve) -> Option<SideGeo> {
        let sk = self.skin(piece);
        let (ua, ub) = (sk.ku[0], sk.ku[sk.ku.len() - 1]);
        // The surface's u runs from the piece's t0 to its t1.
        let n0 = piece.c.to_nurbs(piece.t0, piece.t1);
        let (ua, ub) = if (n0.point(n0.range().0) - piece.c.point(piece.t0)).norm() <= (n0.point(n0.range().0) - piece.c.point(piece.t1)).norm() { (ua, ub) } else { (ub, ua) };
        let linear = n0.degree == 1 && !n0.is_rational() && n0.cps.len() == 2;
        let surf = Surface::Nurbs(Arc::new(sk));
        let s_a = surf.clone();
        let c = piece.c.clone();
        let (t0, t1) = (piece.t0, piece.t1);
        let ends: Vec<Affine> = vec![self.place(0.0), self.place(1.0)];
        Some(SideGeo {
            surf,
            along: Box::new(move |s| {
                let v = if s < 0.5 { 0.0 } else { 1.0 };
                if linear {
                    let k = (ub - ua) / (t1 - t0);
                    return super::sweep::line2(P2::new(ua - t0 * k, v), V2::new(k, 0.0));
                }
                let m = if s < 0.5 { &ends[0] } else { &ends[1] };
                super::pcurve::pcurve(&s_a, &c.transformed(m), t0, t1, None)
            }),
            across: Box::new(move |t| {
                let u = if (t - t0).abs() <= (t - t1).abs() { ua } else { ub };
                super::sweep::line2(P2::new(u, 0.0), V2::new(0.0, 1.0))
            }),
        })
    }
    fn up(&self) -> bool {
        // Which way the profile leaves its plane at the start.
        let d = self.pts[1] - self.pts[0];
        d.dot(&self.plane.normal) > 0.0
    }
}

/// Sweeps the profile along a run of tangent-continuous segments; the profile's plane where it
/// ends.
pub fn sweep_seg(faces: &[MergedFace], chains: &[Vec<Piece>], plane: Plane, segs: &[Seg], control: SweepControl, solid: bool, helix: Option<Helix>) -> Result<(Built, Plane)> {
    // The profile's reach from the path's start.
    let m = super::build::plane_map(&plane, &V3::zeros(), 0.0);
    let start = segs[0].start();
    let mut reach: f64 = 0.0;
    for p in faces.iter().flat_map(|f| std::iter::once(&f.outer).chain(f.holes.iter())).flatten().chain(chains.iter().flatten()) {
        for k in 0..=4 {
            let q = p.at(k as f64 / 4.0);
            reach = reach.max((m.point(&P3::new(q.x, q.y, 0.0)) - start).norm());
        }
    }
    let kind = PathSweep::new(plane, segs, control, reach, helix)?;
    if (kind.pts[1] - kind.pts[0]).dot(&plane.normal).abs() < 1e-9 * (kind.pts[1] - kind.pts[0]).norm().max(1e-300) {
        return Err(KernelError::InvalidParameter("The sweep path runs in the profile's plane".into()));
    }
    let mut b = super::sweep::sweep_full(faces, chains, &kind, 0.0, 1.0, false, solid)?;
    // Sides that came out flat or on a cylinder (a profile edge along a helix's axis) as those
    // surfaces, so booleans see them coincide with faces of the same surface.
    super::unify::planarize(&mut b.body);
    super::unify::cylindrize(&mut b.body);
    Ok((b, kind.end_plane()))
}
