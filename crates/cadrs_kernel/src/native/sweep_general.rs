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
    pub fn new(plane: Plane, segs: &[Seg], control: SweepControl) -> Result<Self> {
        let lens: Vec<f64> = segs.iter().map(|g| g.length()).collect();
        let total: f64 = lens.iter().sum();
        let mut cum = vec![0.0];
        for l in &lens {
            cum.push(cum.last().unwrap() + l / total);
        }
        *cum.last_mut().unwrap() = 1.0;
        let fine = FINE * segs.len();
        let mut me = Self { plane, segs: segs.to_vec(), cum, pts: vec![], rots: vec![], v: vec![], deg: 3 };
        let pts: Vec<P3> = (0..=fine).map(|i| me.at(i as f64 / fine as f64)).collect();
        let tans: Vec<V3> = (0..=fine).map(|i| me.dir(i as f64 / fine as f64)).collect();
        if tans.iter().any(|t| !t.iter().all(|x| x.is_finite())) {
            return Err(KernelError::InvalidParameter("The sweep path has a point with no direction".into()));
        }
        let mut frames: Vec<nalgebra::Rotation3<f64>> = Vec::with_capacity(fine + 1);
        match control {
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
        let n = (8.0 + turn / (PI / 24.0)).ceil().clamp(8.0, 400.0) as usize;
        me.v = (0..=n).map(|k| k as f64 / n as f64).collect();
        me.pts = pts;
        me.rots = rots;
        Ok(me)
    }

    /// The segment and its travel fraction at s.
    fn seg_at(&self, s: f64) -> (usize, f64) {
        let s = s.clamp(0.0, 1.0);
        let k = self.cum.partition_point(|c| *c <= s).saturating_sub(1).min(self.segs.len() - 1);
        let (a, b) = (self.cum[k], self.cum[k + 1]);
        (k, if b > a { ((s - a) / (b - a)).clamp(0.0, 1.0) } else { 0.0 })
    }
    fn at(&self, s: f64) -> P3 {
        let (k, f) = self.seg_at(s);
        self.segs[k].c.point(travel_t(&self.segs[k], f))
    }
    fn dir(&self, s: f64) -> V3 {
        let (k, f) = self.seg_at(s);
        self.segs[k].dir(f)
    }

    /// The rigid motion of the profile from the start of the segment to s.
    fn motion(&self, s: f64) -> Affine {
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
pub fn sweep_seg(faces: &[MergedFace], chains: &[Vec<Piece>], plane: Plane, segs: &[Seg], control: SweepControl, solid: bool) -> Result<(Built, Plane)> {
    let kind = PathSweep::new(plane, segs, control)?;
    if (kind.pts[1] - kind.pts[0]).dot(&plane.normal).abs() < 1e-9 * (kind.pts[1] - kind.pts[0]).norm().max(1e-300) {
        return Err(KernelError::InvalidParameter("The sweep path runs in the profile's plane".into()));
    }
    let b = super::sweep::sweep_full(faces, chains, &kind, 0.0, 1.0, false, solid)?;
    Ok((b, kind.end_plane()))
}
