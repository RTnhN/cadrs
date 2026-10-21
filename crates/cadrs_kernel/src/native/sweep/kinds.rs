//! The exact motions: straight ([`Prism`]) and turning ([`Revolve`], [`Turn`]).

use super::{SideGeo, SweepKind};
use crate::native::curve::Curve;
use crate::native::math::*;
use crate::native::profile::Piece;
use crate::native::surface::Surface;

/// A straight sweep along the unit `dir`.
pub struct Prism {
    pub plane: crate::Plane,
    pub dir: V3,
}

impl SweepKind for Prism {
    fn place(&self, s: f64) -> Affine {
        crate::native::build::plane_map(&self.plane, &self.dir, s)
    }
    fn rail(&self, q: P2) -> Option<Curve> {
        let p = self.place(0.0).point(&P3::new(q.x, q.y, 0.0));
        Some(Curve::Line { o: p, d: self.dir })
    }
    fn side(&self, _piece: &Piece, c3: &Curve) -> Option<SideGeo> {
        let dir = self.dir;
        Some(match c3 {
            Curve::Line { o, d } => {
                let x = d.normalize();
                let mut y = dir - x * x.dot(&dir);
                y.normalize_mut();
                let z = x.cross(&y);
                let (a, b) = (V2::new(d.norm(), 0.0), V2::new(x.dot(&dir), y.dot(&dir)));
                SideGeo {
                    surf: Surface::Plane { f: Frame { o: *o, x, y, z } },
                    along: Box::new(move |s| line2(P2::origin() + b * s, a)),
                    across: Box::new(move |t| line2(P2::origin() + a * t, b)),
                }
            }
            Curve::Circle { c, x, y, r } if x.cross(y).cross(&dir).norm() < 1e-12 => {
                let z = x.cross(y);
                let k = z.dot(&dir);
                SideGeo {
                    surf: Surface::Cylinder { f: Frame { o: *c, x: *x, y: *y, z }, r: *r },
                    along: Box::new(move |s| line2(P2::new(0.0, k * s), V2::new(1.0, 0.0))),
                    across: Box::new(move |t| line2(P2::new(t, 0.0), V2::new(0.0, k))),
                }
            }
            _ => SideGeo {
                surf: Surface::Extrusion { c: c3.clone(), d: dir },
                along: Box::new(|s| line2(P2::new(0.0, s), V2::new(1.0, 0.0))),
                across: Box::new(|t| line2(P2::new(t, 0.0), V2::new(0.0, 1.0))),
            },
        })
    }
    fn up(&self) -> bool {
        self.dir.dot(&self.plane.normal) > 0.0
    }
}

/// A turn about an axis lying in the profile's plane; s is the angle (radians, counter-clockwise
/// about `f.z`). `f.x` points from the axis towards the profile, so the profile lies in the
/// frame's xz half plane x ≥ 0.
pub struct Revolve {
    pub plane: crate::Plane,
    pub f: Frame,
}

pub fn line2(o: P2, d: V2) -> Curve {
    Curve::Line { o: P3::new(o.x, o.y, 0.0), d: V3::new(d.x, d.y, 0.0) }
}

impl Revolve {
    fn rot(&self, s: f64) -> Affine {
        let r = nalgebra::Rotation3::from_axis_angle(&nalgebra::Unit::new_normalize(self.f.z), s).into_inner();
        let o = self.f.o.coords;
        Affine { m: r, t: o - r * o }
    }
    /// Local (ρ, h) of a model point.
    fn rh(&self, p: &P3) -> (f64, f64) {
        let l = self.f.local(p);
        ((l.x * l.x + l.y * l.y).sqrt(), l.z)
    }
}

impl SweepKind for Revolve {
    fn place(&self, s: f64) -> Affine {
        let base = crate::native::build::plane_map(&self.plane, &V3::zeros(), 0.0);
        let r = self.rot(s);
        Affine { m: r.m * base.m, t: r.m * base.t + r.t }
    }
    fn rail(&self, q: P2) -> Option<Curve> {
        let p = self.place(0.0).point(&P3::new(q.x, q.y, 0.0));
        let (rho, h) = self.rh(&p);
        if rho < TOL {
            return None;
        }
        Some(Curve::Circle { c: self.f.o + self.f.z * h, x: self.f.x, y: self.f.y, r: rho })
    }
    fn side(&self, piece: &Piece, c3: &Curve) -> Option<SideGeo> {
        let f = self.f;
        match c3 {
            Curve::Line { o, d } => {
                let (r0, h0) = self.rh(o);
                let dl = f.local(&(o + d)) - f.local(o);
                let (dr, dh) = (dl.x, dl.z); // the line lies in the xz plane: dl.y = 0
                let len = d.norm();
                let (dr, dh) = (dr / len, dh / len);
                // Lies on the axis: sweeps nothing.
                let (ra, rb) = (r0 + dr * piece.t0 * len, r0 + dr * piece.t1 * len);
                if ra.abs() < TOL && rb.abs() < TOL {
                    return None;
                }
                if dr.abs() < 1e-12 {
                    // Cylinder: u = s, v = h0 + dh·len·t.
                    let k = dh * len;
                    Some(SideGeo {
                        surf: Surface::Cylinder { f, r: r0 },
                        along: Box::new(move |s| line2(P2::new(s, h0), V2::new(0.0, k))),
                        across: Box::new(move |t| line2(P2::new(0.0, h0 + k * t), V2::new(1.0, 0.0))),
                    })
                } else if dh.abs() < 1e-12 {
                    // Plane square to the axis at h0: uv = ρ(t)(cos s, sin s).
                    let fr = Frame { o: f.o + f.z * h0, x: f.x, y: f.y, z: f.z };
                    let k = dr * len;
                    Some(SideGeo {
                        surf: Surface::Plane { f: fr },
                        along: Box::new(move |s| {
                            let c = V2::new(s.cos(), s.sin());
                            line2(P2::origin() + c * r0, c * k)
                        }),
                        across: Box::new(move |t| {
                            let r = r0 + k * t;
                            Curve::Circle { c: P3::origin(), x: V3::x(), y: V3::y(), r: r.abs().max(1e-300) }
                        }),
                    })
                } else {
                    // Cone through C(0): r = r0, sin a = dr, cos a = dh, v = len·t.
                    let a = dr.atan2(dh);
                    let fr = Frame { o: f.o + f.z * h0, x: f.x, y: f.y, z: f.z };
                    Some(SideGeo {
                        surf: Surface::Cone { f: fr, r: r0, a },
                        along: Box::new(move |s| line2(P2::new(s, 0.0), V2::new(0.0, len))),
                        across: Box::new(move |t| line2(P2::new(0.0, len * t), V2::new(1.0, 0.0))),
                    })
                }
            }
            Curve::Circle { c, x, y, r } => {
                let (rc, hc) = self.rh(c);
                // Angle φ of the point from the centre in the (ρ, h) plane: φ(t) = σ t + φ0.
                let lx = f.local(&(c + x)) - f.local(c);
                let ly = f.local(&(c + y)) - f.local(c);
                let ax = lx.z.atan2(lx.x);
                let sig = if lx.x * ly.z - lx.z * ly.x > 0.0 { 1.0 } else { -1.0 };
                // Unwrap φ0 so φ at the piece's middle lies in (−π, π].
                let tm = (piece.t0 + piece.t1) * 0.5;
                let phim = wrap(sig * tm + ax, -PI, TAU);
                let phi0 = phim - sig * tm;
                let fr = Frame { o: f.o + f.z * hc, x: f.x, y: f.y, z: f.z };
                let surf = if rc < TOL { Surface::Sphere { f: fr, r: *r } } else { Surface::Torus { f: fr, rmaj: rc, rmin: *r } };
                Some(SideGeo {
                    surf,
                    along: Box::new(move |s| line2(P2::new(s, phi0), V2::new(0.0, sig))),
                    across: Box::new(move |t| line2(P2::new(0.0, sig * t + phi0), V2::new(1.0, 0.0))),
                })
            }
            _ => Some(SideGeo {
                surf: Surface::Revolution { f, m: c3.clone() },
                along: Box::new(|s| line2(P2::new(s, 0.0), V2::new(0.0, 1.0))),
                across: Box::new(|t| line2(P2::new(0.0, t), V2::new(1.0, 0.0))),
            }),
        }
    }
    fn up(&self) -> bool {
        // The sweep leaves along f.y (z × x) at s = 0.
        self.f.y.dot(&self.plane.normal) > 0.0
    }
}

/// A turn about any axis, also one off the profile's plane (SMLib's rotational sweep,
/// `SmRotationalSweepGeometry`, turns whole solids about skew axes); s is the angle (radians,
/// counter-clockwise about `f.z`, the frame orthonormal with `f.o` on the axis).
///
/// A piece lying in a half plane through the axis sweeps the analytic face [`Revolve`] makes,
/// in the frame turned to that half plane; a line square to the axis sweeps a plane (an
/// annulus); any other piece sweeps a surface of revolution of the piece itself (u = s, v = the
/// piece's t: a line skew to the axis makes a hyperboloid). `up`: whether the profile leaves its
/// plane along its normal; the caller checks that every point of it leaves the same way (it
/// lies on one side of the plane through the axis along the normal), which also keeps a whole
/// turn from meeting itself.
pub struct Turn {
    pub plane: crate::Plane,
    pub f: Frame,
    pub up: bool,
}

impl Turn {
    /// The turn by s about the axis.
    pub fn rot(&self, s: f64) -> Affine {
        Revolve { plane: self.plane, f: self.f }.rot(s)
    }

    /// The circle a model point travels on (parametrized by s), or None on the axis.
    pub fn rail_of(&self, p: &P3) -> Option<Curve> {
        let f = &self.f;
        let l = f.local(p);
        let rho = l.x.hypot(l.y);
        if rho < TOL {
            return None;
        }
        let x = (f.x * l.x + f.y * l.y) / rho;
        Some(Curve::Circle { c: f.o + f.z * l.z, x, y: f.z.cross(&x), r: rho })
    }

    /// The side face a model-space curve (a piece's curve at s = 0) sweeps, or None for a piece
    /// on the axis.
    pub fn side_of(&self, piece: &Piece, c3: &Curve) -> Option<SideGeo> {
        let f = self.f;
        let pts: Vec<V3> = (0..=8).map(|i| f.local(&c3.point(piece.t0 + (piece.t1 - piece.t0) * i as f64 / 8.0))).collect();
        let scale = pts.iter().map(|l| l.norm()).fold(1.0, f64::max);
        let far = pts.iter().max_by(|a, b| a.x.hypot(a.y).total_cmp(&b.x.hypot(b.y)))?;
        if far.x.hypot(far.y) < TOL {
            return None;
        }
        // In the half plane at the azimuth of its point farthest from the axis?
        let phi = far.y.atan2(far.x);
        let (c, s) = (phi.cos(), phi.sin());
        let half = Frame { o: f.o, x: f.x * c + f.y * s, y: f.y * c - f.x * s, z: f.z };
        let in_half = pts.iter().all(|l| {
            let (x, y) = (l.x * c + l.y * s, l.y * c - l.x * s);
            y.abs() <= 1e-9 * scale && x >= -1e-9 * scale
        });
        let in_half = in_half
            && match c3 {
                Curve::Line { d, .. } => half.y.dot(d).abs() <= 1e-12 * d.norm(),
                Curve::Circle { x, y, .. } | Curve::Ellipse { x, y, .. } => half.y.dot(x).abs() <= 1e-12 * x.norm() && half.y.dot(y).abs() <= 1e-12 * y.norm(),
                Curve::Nurbs(_) => true,
            };
        if in_half {
            return Revolve { plane: self.plane, f: half }.side(piece, c3);
        }
        if let Curve::Line { o, d } = c3
            && f.z.dot(d).abs() <= 1e-12 * d.norm()
        {
            // A line square to the axis, off every half plane through it: it turns in the plane
            // at its height; uv = its point's local (x, y) turned by s.
            let l0 = f.local(o);
            let dl = V2::new(f.x.dot(d), f.y.dot(d));
            let q0 = V2::new(l0.x, l0.y);
            let rot = |s: f64, v: V2| V2::new(v.x * s.cos() - v.y * s.sin(), v.x * s.sin() + v.y * s.cos());
            return Some(SideGeo {
                surf: Surface::Plane { f: Frame { o: f.o + f.z * l0.z, x: f.x, y: f.y, z: f.z } },
                along: Box::new(move |s| line2(P2::origin() + rot(s, q0), rot(s, dl))),
                across: Box::new(move |t| {
                    let q = q0 + dl * t;
                    let r = q.norm();
                    let (x, y) = if r > 1e-300 { (V3::new(q.x / r, q.y / r, 0.0), V3::new(-q.y / r, q.x / r, 0.0)) } else { (V3::x(), V3::y()) };
                    Curve::Circle { c: P3::origin(), x, y, r: r.max(1e-300) }
                }),
            });
        }
        Some(SideGeo {
            surf: Surface::Revolution { f, m: c3.clone() },
            along: Box::new(|s| line2(P2::new(s, 0.0), V2::new(0.0, 1.0))),
            across: Box::new(|t| line2(P2::new(0.0, t), V2::new(1.0, 0.0))),
        })
    }
}

impl SweepKind for Turn {
    fn place(&self, s: f64) -> Affine {
        Revolve { plane: self.plane, f: self.f }.place(s)
    }
    fn rail(&self, q: P2) -> Option<Curve> {
        let p = self.place(0.0).point(&P3::new(q.x, q.y, 0.0));
        self.rail_of(&p)
    }
    fn side(&self, piece: &Piece, c3: &Curve) -> Option<SideGeo> {
        self.side_of(piece, c3)
    }
    fn up(&self) -> bool {
        self.up
    }
}
