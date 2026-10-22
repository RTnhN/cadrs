//! Shared by the OCCT-free gates (metamorphic_booleans.rs, stress_booleans.rs, fingerprint.rs):
//! solids built from parameters, a seeded random source, and the measures the gates compare
//! (volume over all of a result's bodies, validity, the largest tolerance).
//!
//! Environment knobs (the gate runs the defaults; the soak on the build box raises them):
//! `CADRS_META_SEED` (a number, or `random`), `CADRS_META_CASES`, `CADRS_STRESS_ANGLES`,
//! `CADRS_STRESS_UNIONS`, `CADRS_STRESS_DRILLS`, `CADRS_FINGERPRINT_HEAVY`, `CADRS_FINGERPRINT_OUT`.
#![allow(dead_code)]

use std::f64::consts::PI;

use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::native::topo::CheckLevel;
use cadrs_kernel::*;
use nalgebra::{Isometry3, Point2, Point3, Translation3, UnitQuaternion, Vector3};

/// A number from the environment, else `default`.
pub fn env_num<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// The seed: `CADRS_META_SEED` (`random` picks one from the clock and prints it), else `default`.
pub fn seed(default: u64) -> u64 {
    match std::env::var("CADRS_META_SEED").as_deref() {
        Ok("random") => {
            let s = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(1);
            println!("CADRS_META_SEED={s}");
            s
        }
        Ok(v) => v.parse().unwrap_or(default),
        Err(_) => default,
    }
}

/// SplitMix64: small, seedable and the same on every platform.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
    /// On a grid of `step` half the time (coincident faces and edges), anywhere the other half.
    pub fn snapped(&mut self, lo: f64, hi: f64, step: f64) -> f64 {
        let x = self.range(lo, hi);
        if self.below(2) == 0 {
            (x / step).round() * step
        } else {
            x
        }
    }
}

/// A solid by its parameters (printed with `{:?}` as a case's repro).
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    /// From `min`, of sides `size`.
    Box { min: [f64; 3], size: [f64; 3] },
    /// Along axis 0, 1 or 2 (x, y, z) from `base`, of radius `r` and length `len`.
    Cyl {
        axis: usize,
        base: [f64; 3],
        r: f64,
        len: f64,
    },
    /// Centred at `c`.
    Sphere { c: [f64; 3], r: f64 },
    /// A drill bit along −z from `tip + (0, 0, len)`: a cylinder of radius `r` and a 118° point
    /// at `tip`.
    Drill { tip: [f64; 3], r: f64, len: f64 },
    /// A sketch on the plane through `o` normal to axis 0, 1 or 2 (sketch x along the next axis,
    /// y along the one after): the rectangle `outer` ([x0, y0, x1, y1]) less the rectangle
    /// `hole` when given, extruded `depth` along the axis.
    Prism {
        axis: usize,
        o: [f64; 3],
        outer: [f64; 4],
        hole: Option<[f64; 4]>,
        depth: f64,
    },
}

impl Shape {
    /// Whether the point is inside the solid (for a [`Shape::Box`] or [`Shape::Prism`]; open
    /// boundaries: a point on a face is outside).
    pub fn contains(&self, p: [f64; 3]) -> bool {
        let inside = |x: f64, lo: f64, hi: f64| x > lo && x < hi;
        match *self {
            Shape::Box { min, size } => (0..3).all(|i| inside(p[i], min[i], min[i] + size[i])),
            Shape::Prism {
                axis,
                o,
                outer,
                hole,
                depth,
            } => {
                let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
                let (x, y) = (p[a] - o[a], p[b] - o[b]);
                let rect = |r: [f64; 4]| inside(x, r[0], r[2]) && inside(y, r[1], r[3]);
                inside(p[axis], o[axis], o[axis] + depth) && rect(outer) && !hole.is_some_and(rect)
            }
            _ => false,
        }
    }

    /// The coordinates of its faces along each axis (for a [`Shape::Box`] or [`Shape::Prism`]).
    pub fn planes(&self) -> [Vec<f64>; 3] {
        match *self {
            Shape::Box { min, size } => std::array::from_fn(|i| vec![min[i], min[i] + size[i]]),
            Shape::Prism {
                axis,
                o,
                outer,
                hole,
                depth,
            } => {
                let (a, b) = ((axis + 1) % 3, (axis + 2) % 3);
                let mut c: [Vec<f64>; 3] = Default::default();
                c[axis] = vec![o[axis], o[axis] + depth];
                for r in std::iter::once(outer).chain(hole) {
                    c[a].extend([o[a] + r[0], o[a] + r[2]]);
                    c[b].extend([o[b] + r[1], o[b] + r[3]]);
                }
                c
            }
            _ => Default::default(),
        }
    }
}

/// A shape moved by a rotation of `angle` about `axis` through `about` (no motion for a zero
/// angle).
#[derive(Debug, Clone, Copy)]
pub struct Placed {
    pub shape: Shape,
    pub axis: [f64; 3],
    pub angle: f64,
    pub about: [f64; 3],
}

impl Placed {
    pub fn fixed(shape: Shape) -> Self {
        Placed {
            shape,
            axis: [0.0, 0.0, 1.0],
            angle: 0.0,
            about: [0.0; 3],
        }
    }
}

fn p2(x: f64, y: f64) -> Point2<f64> {
    Point2::new(x, y)
}

fn line(a: Point2<f64>, b: Point2<f64>) -> Curve2 {
    Curve2::Line { a, b, source: None }
}

fn polygon(c: &[Point2<f64>]) -> Loop {
    Loop {
        curves: (0..c.len())
            .map(|i| line(c[i], c[(i + 1) % c.len()]))
            .collect(),
    }
}

fn region(outer: Loop) -> Vec<Region> {
    vec![Region {
        outer,
        holes: vec![],
        source: None,
    }]
}

/// The plane through `o` with normal along `axis` (x, y or z) and its sketch x along the next one.
fn axis_plane(axis: usize, o: [f64; 3]) -> Plane {
    let e = |i: usize| match i % 3 {
        0 => Vector3::x_axis(),
        1 => Vector3::y_axis(),
        _ => Vector3::z_axis(),
    };
    Plane {
        origin: Point3::from(o),
        x_dir: e(axis + 1),
        normal: e(axis),
    }
}

/// Revolves a sketch in the XZ plane (sketch x along X, y along Z) a whole turn about Z.
fn revolve_xz(k: &mut NativeKernel, outer: Loop) -> Result<BodyId> {
    let front = Plane {
        origin: Point3::origin(),
        x_dir: Vector3::x_axis(),
        normal: -Vector3::y_axis(),
    };
    let axis = Axis {
        origin: Point3::origin(),
        dir: Vector3::z_axis(),
    };
    let r = k.revolve(&Profile::new(front, region(outer)), axis, 2.0 * PI)?;
    Ok(r.bodies[0])
}

fn translate(k: &mut NativeKernel, b: BodyId, d: [f64; 3]) -> Result<BodyId> {
    if d == [0.0; 3] {
        return Ok(b);
    }
    let t = Isometry3::from_parts(
        Translation3::new(d[0], d[1], d[2]),
        UnitQuaternion::identity(),
    );
    Ok(k.transform(b, &t)?.bodies[0])
}

/// The rigid motion of `p` (rotation about `p.axis` through `p.about`).
pub fn motion(p: &Placed) -> Isometry3<f64> {
    let axis = Vector3::from(p.axis);
    let q = if axis.norm() > 0.0 {
        UnitQuaternion::from_axis_angle(&nalgebra::Unit::new_normalize(axis), p.angle)
    } else {
        UnitQuaternion::identity()
    };
    let c = Vector3::from(p.about);
    // x ↦ q (x − c) + c
    Isometry3::from_parts(Translation3::from(c - q * c), q)
}

/// Builds a shape in `k`.
pub fn build(k: &mut NativeKernel, p: &Placed) -> Result<BodyId> {
    let b = match p.shape {
        Shape::Box { min, size } => {
            let pl = axis_plane(2, min);
            let outer = polygon(&[
                p2(0.0, 0.0),
                p2(size[0], 0.0),
                p2(size[0], size[1]),
                p2(0.0, size[1]),
            ]);
            k.extrude(&Profile::new(pl, region(outer)), Extent::Blind(size[2]))?
                .bodies[0]
        }
        Shape::Cyl { axis, base, r, len } => {
            let pl = axis_plane(axis, base);
            let outer = Loop {
                curves: vec![Curve2::Circle {
                    center: p2(0.0, 0.0),
                    radius: r,
                    source: None,
                }],
            };
            k.extrude(&Profile::new(pl, region(outer)), Extent::Blind(len))?
                .bodies[0]
        }
        Shape::Sphere { c, r } => {
            let half = Loop {
                curves: vec![
                    Curve2::Arc {
                        center: p2(0.0, 0.0),
                        radius: r,
                        start_angle: -PI / 2.0,
                        sweep: PI,
                        source: None,
                    },
                    line(p2(0.0, r), p2(0.0, -r)),
                ],
            };
            let s = revolve_xz(k, half)?;
            translate(k, s, c)?
        }
        Shape::Prism {
            axis,
            o,
            outer,
            hole,
            depth,
        } => {
            let rect = |r: [f64; 4]| {
                polygon(&[
                    p2(r[0], r[1]),
                    p2(r[2], r[1]),
                    p2(r[2], r[3]),
                    p2(r[0], r[3]),
                ])
            };
            let region = Region {
                outer: rect(outer),
                holes: hole.map(rect).into_iter().collect(),
                source: None,
            };
            k.extrude(
                &Profile::new(axis_plane(axis, o), vec![region]),
                Extent::Blind(depth),
            )?
            .bodies[0]
        }
        Shape::Drill { tip, r, len } => {
            // A 118° point: the cone's half angle is 59°.
            let h = r / 59f64.to_radians().tan();
            let outline = polygon(&[p2(0.0, 0.0), p2(r, h), p2(r, len), p2(0.0, len)]);
            let d = revolve_xz(k, outline)?;
            translate(k, d, tip)?
        }
    };
    if p.angle == 0.0 {
        return Ok(b);
    }
    Ok(k.transform(b, &motion(p))?.bodies[0])
}

/// The closed-form volume of a shape (rigid motions keep it).
pub fn exact_volume(s: &Shape) -> f64 {
    match *s {
        Shape::Box { size, .. } => size[0] * size[1] * size[2],
        Shape::Cyl { r, len, .. } => PI * r * r * len,
        Shape::Sphere { r, .. } => 4.0 / 3.0 * PI * r * r * r,
        Shape::Prism {
            outer, hole, depth, ..
        } => {
            let area = |r: [f64; 4]| (r[2] - r[0]) * (r[3] - r[1]);
            (area(outer) - hole.map_or(0.0, area)) * depth
        }
        Shape::Drill { r, len, .. } => {
            let h = r / 59f64.to_radians().tan();
            PI * r * r * (len - h) + PI * r * r * h / 3.0
        }
    }
}

/// The volume of a result: the sum over its bodies (none for an empty result).
pub fn volume(k: &NativeKernel, bodies: &[BodyId]) -> Result<f64> {
    bodies
        .iter()
        .map(|&b| k.mass_properties(b).map(|m| m.volume))
        .sum()
}

/// What is wrong with the bodies: `check_with(Full)` issues, and a body that isn't a solid.
pub fn invalid(k: &NativeKernel, bodies: &[BodyId]) -> Option<String> {
    let mut why = Vec::new();
    for &b in bodies {
        match k.get(b) {
            Ok(body) => {
                let issues = body.check_with(CheckLevel::Full);
                if !issues.is_empty() {
                    why.push(format!("{} issue(s), first: {}", issues.len(), issues[0]));
                }
                if !body.is_solid() {
                    why.push("not a solid".into());
                }
            }
            Err(e) => why.push(format!("{e:?}")),
        }
    }
    (!why.is_empty()).then(|| why.join("; "))
}

/// The largest vertex or edge tolerance of the bodies.
pub fn max_tol(k: &NativeKernel, bodies: &[BodyId]) -> f64 {
    bodies
        .iter()
        .filter_map(|&b| k.get(b).ok())
        .flat_map(|b| {
            let v = b.verts().iter().map(|v| v.tol);
            let e = b.edges().iter().map(|e| e.tol);
            v.chain(e).collect::<Vec<_>>()
        })
        .fold(0.0, f64::max)
}

/// A boolean's bodies, an empty result as no bodies.
pub fn boolean(k: &mut NativeKernel, op: BoolOp, a: BodyId, b: &[BodyId]) -> Result<Vec<BodyId>> {
    match k.boolean(op, a, b) {
        Ok(r) => Ok(r.bodies),
        Err(KernelError::EmptyResult(_)) => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

/// FNV-1a, 64 bits: the fingerprint of a result's bytes.
pub fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}
