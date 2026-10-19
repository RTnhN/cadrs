//! A DRAW-style API on the native kernel: OCCT's test commands (`line`, `mkedge`, `wire`,
//! `box`, `pcylinder`, `bfuse`, `blend`, `fillet2d`, `checkprops`, …) as methods of
//! [`NativeKernel`], so OCCT's test cases read almost line for line as Rust tests
//! (`tests/occt_*.rs`).
//!
//! Arguments follow DRAW's order (points and directions as separate numbers, angles in degrees
//! where DRAW takes degrees). Shapes are [`Shape`] values: vertices, edges, wires and planar
//! faces are held here; solids and sheets are bodies of the kernel.

use std::f64::consts::{PI, TAU};

use super::curve::Curve;
use super::kernel::NativeKernel;
use super::math::*;
use crate::*;

/// An edge: a curve between two parameters.
#[derive(Debug, Clone)]
pub struct DEdge {
    pub curve: Curve,
    pub t0: f64,
    pub t1: f64,
}

impl DEdge {
    pub fn start(&self) -> P3 {
        self.curve.point(self.t0)
    }
    pub fn end(&self) -> P3 {
        self.curve.point(self.t1)
    }
    pub fn length(&self) -> f64 {
        self.curve.length(self.t0.min(self.t1), self.t0.max(self.t1))
    }
    fn reversed(&self) -> DEdge {
        let (curve, t0, t1) = self.curve.reversed(self.t0, self.t1);
        DEdge { curve, t0, t1 }
    }
}

/// A DRAW shape.
#[derive(Debug, Clone)]
pub enum Shape {
    Vertex(P3),
    Edge(DEdge),
    Wire(Vec<DEdge>),
    /// A planar face: its plane and loops (outer first).
    Face { plane: Plane, loops: Vec<Vec<DEdge>> },
    /// A solid or sheet of the kernel.
    Body(BodyId),
    /// An edge of a body (from `explode`).
    BodyEdge(BodyId, EdgeId),
    /// A face of a body (from `explode`).
    BodyFace(BodyId, FaceId),
    /// Nothing (an empty boolean result).
    Empty,
}

/// A placement (DRAW's `plane`): origin, normal and x direction.
#[derive(Debug, Clone, Copy)]
pub struct Ax3 {
    pub origin: P3,
    pub z: V3,
    pub x: V3,
}

impl Ax3 {
    pub fn plane(&self) -> Plane {
        Plane { origin: self.origin, x_dir: nalgebra::Unit::new_normalize(self.x), normal: nalgebra::Unit::new_normalize(self.z) }
    }
    pub fn y(&self) -> V3 {
        self.z.cross(&self.x)
    }
}

/// Counts of sub-shapes (DRAW's `nbshapes`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NbShapes {
    pub vertex: usize,
    pub edge: usize,
    pub wire: usize,
    pub face: usize,
    pub shell: usize,
    pub solid: usize,
}

/// OCCT's order of a primitive's sub-shapes (for `explode`): reference points (edge midpoints,
/// face centres) in OCCT's order, carried along through transforms.
#[derive(Debug, Clone, Default)]
struct OcctOrder {
    edges: Vec<P3>,
    faces: Vec<P3>,
}

thread_local! {
    static ORDERS: std::cell::RefCell<std::collections::HashMap<BodyId, OcctOrder>> = Default::default();
}

fn set_order(b: BodyId, o: OcctOrder) {
    ORDERS.with(|m| m.borrow_mut().insert(b, o));
}

fn order_of(b: BodyId) -> Option<OcctOrder> {
    ORDERS.with(|m| m.borrow().get(&b).cloned())
}

/// OCCT's order for a full cylinder or truncated cone (BRepPrim_OneAxis): faces lateral, top,
/// bottom; edges top circle, seam, bottom circle (the seam at the placement's x).
fn revolved_order(ax: &Ax3, r1: f64, r2: f64, h: f64) -> OcctOrder {
    let w = |x: f64, y: f64, z: f64| ax.origin + ax.x * x + ax.y() * y + ax.z * z;
    let (lo, hi) = (0.0f64.min(h), 0.0f64.max(h));
    let (rlo, rhi) = if h >= 0.0 { (r1, r2) } else { (r2, r1) };
    OcctOrder {
        edges: vec![w(-rhi, 0.0, hi), w((r1 + r2) / 2.0, 0.0, (lo + hi) / 2.0), w(-rlo, 0.0, lo)],
        faces: vec![w(0.0, (r1 + r2) / 2.0, (lo + hi) / 2.0), w(0.0, 0.0, hi), w(0.0, 0.0, lo)],
    }
}

fn err(m: &str) -> KernelError {
    KernelError::OperationFailed(m.to_string())
}

/// OCCT's x direction for a placement with only its main direction (`gp_Ax2(P, V)`).
pub fn ax2_x(v: &V3) -> V3 {
    let (a, b, c) = (v.x, v.y, v.z);
    let (aa, ba, ca) = (a.abs(), b.abs(), c.abs());
    let d = if ba <= aa && ba <= ca {
        if aa > ca { V3::new(-c, 0.0, a) } else { V3::new(c, 0.0, -a) }
    } else if aa <= ba && aa <= ca {
        if ba > ca { V3::new(0.0, -c, b) } else { V3::new(0.0, c, -b) }
    } else if aa > ba {
        V3::new(-b, a, 0.0)
    } else {
        V3::new(b, -a, 0.0)
    };
    d.normalize()
}

/// Checks a measured property against DRAW's `checkprops` rule: within 1% of the expected value
/// (or exactly 0 when 0 is expected).
pub fn check_rel(what: &str, got: f64, want: f64) -> Result<()> {
    let ok = if want == 0.0 { got.abs() < 1e-9 } else { ((got - want) / want).abs() <= 1e-2 };
    if ok { Ok(()) } else { Err(err(&format!("the {what} is {got}, expected {want}"))) }
}

impl NativeKernel {
    // --- Geometry ---------------------------------------------------------------------------

    /// `line name x y z dx dy dz`.
    pub fn line(&self, x: f64, y: f64, z: f64, dx: f64, dy: f64, dz: f64) -> Curve {
        Curve::Line { o: P3::new(x, y, z), d: V3::new(dx, dy, dz).normalize() }
    }

    /// `circle name x y z r` (normal +z).
    pub fn circle(&self, x: f64, y: f64, z: f64, r: f64) -> Curve {
        self.circle_n(x, y, z, 0.0, 0.0, 1.0, r)
    }

    /// `circle name x y z nx ny nz r`.
    pub fn circle_n(&self, x: f64, y: f64, z: f64, nx: f64, ny: f64, nz: f64, r: f64) -> Curve {
        let n = V3::new(nx, ny, nz).normalize();
        let xd = ax2_x(&n);
        Curve::Circle { c: P3::new(x, y, z), x: xd, y: n.cross(&xd), r }
    }

    /// `circle name x y z nx ny nz xx xy xz r`.
    pub fn circle_nx(&self, x: f64, y: f64, z: f64, nx: f64, ny: f64, nz: f64, xx: f64, xy: f64, xz: f64, r: f64) -> Curve {
        let n = V3::new(nx, ny, nz).normalize();
        let xd = V3::new(xx, xy, xz);
        let xd = (xd - n * xd.dot(&n)).normalize();
        Curve::Circle { c: P3::new(x, y, z), x: xd, y: n.cross(&xd), r }
    }

    /// `plane name x y z [nx ny nz [xx xy xz]]`.
    pub fn plane(&self, x: f64, y: f64, z: f64, nx: f64, ny: f64, nz: f64) -> Ax3 {
        let n = V3::new(nx, ny, nz).normalize();
        Ax3 { origin: P3::new(x, y, z), z: n, x: ax2_x(&n) }
    }

    pub fn plane_x(&self, x: f64, y: f64, z: f64, nx: f64, ny: f64, nz: f64, xx: f64, xy: f64, xz: f64) -> Ax3 {
        let n = V3::new(nx, ny, nz).normalize();
        let xd = V3::new(xx, xy, xz);
        Ax3 { origin: P3::new(x, y, z), z: n, x: (xd - n * xd.dot(&n)).normalize() }
    }

    // --- Topology ---------------------------------------------------------------------------

    /// `vertex name x y z`.
    pub fn vertex(&self, x: f64, y: f64, z: f64) -> Shape {
        Shape::Vertex(P3::new(x, y, z))
    }

    /// `edge name v1 v2`: the segment between two vertices.
    pub fn edge(&self, v1: &Shape, v2: &Shape) -> Result<Shape> {
        let (Shape::Vertex(a), Shape::Vertex(b)) = (v1, v2) else { return Err(err("edge takes two vertices")) };
        let (curve, l) = Curve::line(*a, *b);
        Ok(Shape::Edge(DEdge { curve, t0: 0.0, t1: l }))
    }

    /// `mkedge name curve t0 t1`.
    pub fn mkedge(&self, c: &Curve, t0: f64, t1: f64) -> Shape {
        Shape::Edge(DEdge { curve: c.clone(), t0, t1 })
    }

    /// `mkedge name curve` of a whole circle.
    pub fn mkedge_full(&self, c: &Curve) -> Shape {
        self.mkedge(c, 0.0, TAU)
    }

    /// `wire name e1 e2 …`: the edges chained end to start (each turned round when needed).
    pub fn wire(&self, edges: &[&Shape]) -> Result<Shape> {
        let mut out: Vec<DEdge> = Vec::new();
        for s in edges {
            let list: Vec<DEdge> = match s {
                Shape::Edge(e) => vec![e.clone()],
                Shape::Wire(w) => w.clone(),
                _ => return Err(err("wire takes edges and wires")),
            };
            for e in list {
                out.push(e);
            }
        }
        orient_chain(&mut out)?;
        Ok(Shape::Wire(out))
    }

    /// `polyline name x1 y1 z1 x2 y2 z2 …`.
    pub fn polyline(&self, pts: &[[f64; 3]]) -> Result<Shape> {
        let mut out = Vec::new();
        for w in pts.windows(2) {
            let (a, b) = (P3::from(w[0]), P3::from(w[1]));
            let (curve, l) = Curve::line(a, b);
            if l > 0.0 {
                out.push(DEdge { curve, t0: 0.0, t1: l });
            }
        }
        if out.is_empty() {
            return Err(err("an empty polyline"));
        }
        Ok(Shape::Wire(out))
    }

    /// `mkplane name wire`: the planar face inside a closed wire.
    pub fn mkplane(&self, w: &Shape) -> Result<Shape> {
        let edges = match w {
            Shape::Wire(w) => w.clone(),
            Shape::Edge(e) => vec![e.clone()],
            _ => return Err(err("mkplane takes a wire")),
        };
        let plane = fit_plane(&edges).ok_or_else(|| err("the wire isn't planar"))?;
        Ok(Shape::Face { plane, loops: vec![edges] })
    }

    /// `add wire face`: a hole in a planar face.
    pub fn add_hole(&self, w: &Shape, f: &Shape) -> Result<Shape> {
        let (Shape::Wire(w), Shape::Face { plane, loops }) = (w, f) else { return Err(err("add takes a wire and a face")) };
        let mut loops = loops.clone();
        loops.push(w.clone());
        Ok(Shape::Face { plane: *plane, loops })
    }

    // --- Primitives -------------------------------------------------------------------------

    /// `box name dx dy dz`.
    pub fn box_(&mut self, dx: f64, dy: f64, dz: f64) -> Result<Shape> {
        self.box_at(0.0, 0.0, 0.0, dx, dy, dz)
    }

    /// `box name x y z dx dy dz`.
    pub fn box_at(&mut self, x: f64, y: f64, z: f64, dx: f64, dy: f64, dz: f64) -> Result<Shape> {
        let ax = Ax3 { origin: P3::new(x, y, z), z: V3::z(), x: V3::x() };
        self.box_on(&ax, dx, dy, dz)
    }

    /// `box name -pl plane dx dy dz` (a box in a placement).
    pub fn box_on(&mut self, ax: &Ax3, dx: f64, dy: f64, dz: f64) -> Result<Shape> {
        let (x0, x1) = (0.0f64.min(dx), 0.0f64.max(dx));
        let (y0, y1) = (0.0f64.min(dy), 0.0f64.max(dy));
        let mut pl = ax.plane();
        if dz < 0.0 {
            pl.origin += ax.z * dz;
        }
        let rect = |a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)| {
            let p = |q: (f64, f64)| nalgebra::Point2::new(q.0, q.1);
            Loop { curves: vec![l2(p(a), p(b)), l2(p(b), p(c)), l2(p(c), p(d)), l2(p(d), p(a))] }
        };
        let region = Region { outer: rect((x0, y0), (x1, y0), (x1, y1), (x0, y1)), holes: vec![], source: None };
        let r = self.extrude(&Profile::new(pl, vec![region]), Extent::Blind(dz.abs()))?;
        // OCCT's order (BRepPrim_GWedge): faces XMin XMax YMin YMax ZMin ZMax; edges as met
        // along the faces' wires.
        let (z0, z1) = (0.0f64.min(dz), 0.0f64.max(dz));
        let (xm, ym, zm) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0, (z0 + z1) / 2.0);
        let w = |x: f64, y: f64, z: f64| ax.origin + ax.x * x + ax.y() * y + ax.z * z;
        let edges = vec![
            w(x0, y0, zm), w(x0, ym, z1), w(x0, y1, zm), w(x0, ym, z0),
            w(x1, y0, zm), w(x1, ym, z1), w(x1, y1, zm), w(x1, ym, z0),
            w(xm, y0, z0), w(xm, y0, z1), w(xm, y1, z0), w(xm, y1, z1),
        ];
        let faces = vec![w(x0, ym, zm), w(x1, ym, zm), w(xm, y0, zm), w(xm, y1, zm), w(xm, ym, z0), w(xm, ym, z1)];
        set_order(r.bodies[0], OcctOrder { edges, faces });
        Ok(Shape::Body(r.bodies[0]))
    }

    /// `pcylinder name r h`.
    pub fn pcylinder(&mut self, r: f64, h: f64) -> Result<Shape> {
        self.pcylinder_on(&Ax3 { origin: P3::origin(), z: V3::z(), x: V3::x() }, r, h)
    }

    /// `pcylinder name plane r h`.
    pub fn pcylinder_on(&mut self, ax: &Ax3, r: f64, h: f64) -> Result<Shape> {
        let mut pl = ax.plane();
        if h < 0.0 {
            pl.origin += ax.z * h;
        }
        let region = Region { outer: Loop { curves: vec![Curve2::Circle { center: nalgebra::Point2::origin(), radius: r, source: None }] }, holes: vec![], source: None };
        let res = self.extrude(&Profile::new(pl, vec![region]), Extent::Blind(h.abs()))?;
        set_order(res.bodies[0], revolved_order(ax, r, r, h));
        Ok(Shape::Body(res.bodies[0]))
    }

    /// `pcylinder name [plane] r h angle` (degrees): a sector of the cylinder.
    pub fn pcylinder_a(&mut self, ax: Option<&Ax3>, r: f64, h: f64, angle: f64) -> Result<Shape> {
        let ax = ax.copied().unwrap_or(Ax3 { origin: P3::origin(), z: V3::z(), x: V3::x() });
        if angle >= 360.0 - 1e-9 {
            return self.pcylinder_on(&ax, r, h);
        }
        let p = nalgebra::Point2::new;
        self.revolve_section(&ax, &[p(0.0, 0.0), p(r, 0.0), p(r, h), p(0.0, h)], angle.to_radians())
    }

    /// `pcone name [plane] r1 r2 h angle` (degrees).
    pub fn pcone_a(&mut self, ax: Option<&Ax3>, r1: f64, r2: f64, h: f64, angle: f64) -> Result<Shape> {
        let ax = ax.copied().unwrap_or(Ax3 { origin: P3::origin(), z: V3::z(), x: V3::x() });
        if angle >= 360.0 - 1e-9 {
            return self.pcone_on(&ax, r1, r2, h);
        }
        let p = nalgebra::Point2::new;
        let mut pts = vec![p(0.0, 0.0)];
        if r1 > 0.0 {
            pts.push(p(r1, 0.0));
        }
        if r2 > 0.0 {
            pts.push(p(r2, h));
        }
        pts.push(p(0.0, h));
        self.revolve_section(&ax, &pts, angle.to_radians())
    }

    /// `psphere name [plane] r angle` (degrees of longitude).
    pub fn psphere_a(&mut self, ax: Option<&Ax3>, r: f64, angle: f64) -> Result<Shape> {
        let ax = ax.copied().unwrap_or(Ax3 { origin: P3::origin(), z: V3::z(), x: V3::x() });
        let half = Region {
            outer: Loop {
                curves: vec![
                    Curve2::Arc { center: nalgebra::Point2::origin(), radius: r, start_angle: -PI / 2.0, sweep: PI, source: None },
                    l2(nalgebra::Point2::new(0.0, r), nalgebra::Point2::new(0.0, -r)),
                ],
            },
            holes: vec![],
            source: None,
        };
        let res = self.revolve(&Profile::new(self.meridian(&ax), vec![half]), Axis { origin: ax.origin, dir: nalgebra::Unit::new_normalize(ax.z) }, angle.min(360.0).to_radians())?;
        Ok(Shape::Body(res.bodies[0]))
    }

    /// `psphere name [plane] r lat1 lat2 [angle]` (degrees): the zone between two latitudes.
    pub fn psphere_zone(&mut self, ax: Option<&Ax3>, r: f64, lat1: f64, lat2: f64, angle: f64) -> Result<Shape> {
        let ax = ax.copied().unwrap_or(Ax3 { origin: P3::origin(), z: V3::z(), x: V3::x() });
        let (a1, a2) = (lat1.to_radians(), lat2.to_radians());
        let (p1, p2) = (nalgebra::Point2::new(r * a1.cos(), r * a1.sin()), nalgebra::Point2::new(r * a2.cos(), r * a2.sin()));
        let mut curves = vec![Curve2::Arc { center: nalgebra::Point2::origin(), radius: r, start_angle: a1, sweep: a2 - a1, source: None }];
        let (q2, q1) = (nalgebra::Point2::new(0.0, p2.y), nalgebra::Point2::new(0.0, p1.y));
        if p2.x.abs() > 1e-12 {
            curves.push(l2(p2, q2));
        }
        curves.push(l2(q2, q1));
        if p1.x.abs() > 1e-12 {
            curves.push(l2(q1, p1));
        }
        let region = Region { outer: Loop { curves }, holes: vec![], source: None };
        let res = self.revolve(&Profile::new(self.meridian(&ax), vec![region]), Axis { origin: ax.origin, dir: nalgebra::Unit::new_normalize(ax.z) }, angle.min(360.0).to_radians())?;
        Ok(Shape::Body(res.bodies[0]))
    }

    /// `ptorus name [plane] R r angle` (degrees of longitude).
    pub fn ptorus_a(&mut self, ax: Option<&Ax3>, big: f64, r: f64, angle: f64) -> Result<Shape> {
        let ax = ax.copied().unwrap_or(Ax3 { origin: P3::origin(), z: V3::z(), x: V3::x() });
        let region = Region { outer: Loop { curves: vec![Curve2::Circle { center: nalgebra::Point2::new(big, 0.0), radius: r, source: None }] }, holes: vec![], source: None };
        let res = self.revolve(&Profile::new(self.meridian(&ax), vec![region]), Axis { origin: ax.origin, dir: nalgebra::Unit::new_normalize(ax.z) }, angle.min(360.0).to_radians())?;
        Ok(Shape::Body(res.bodies[0]))
    }

    /// `pcone name r1 r2 h`.
    pub fn pcone(&mut self, r1: f64, r2: f64, h: f64) -> Result<Shape> {
        self.pcone_on(&Ax3 { origin: P3::origin(), z: V3::z(), x: V3::x() }, r1, r2, h)
    }

    /// `pcone name plane r1 r2 h`.
    pub fn pcone_on(&mut self, ax: &Ax3, r1: f64, r2: f64, h: f64) -> Result<Shape> {
        let p = nalgebra::Point2::new;
        let mut pts = vec![p(0.0, 0.0)];
        if r1 > 0.0 {
            pts.push(p(r1, 0.0));
        }
        if r2 > 0.0 {
            pts.push(p(r2, h));
        }
        pts.push(p(0.0, h));
        let s = self.revolve_section(ax, &pts, TAU)?;
        if let (Shape::Body(b), true) = (&s, r1 > 0.0 && r2 > 0.0) {
            set_order(*b, revolved_order(ax, r1, r2, h));
        }
        Ok(s)
    }

    /// `psphere name r`.
    pub fn psphere(&mut self, r: f64) -> Result<Shape> {
        self.psphere_on(&Ax3 { origin: P3::origin(), z: V3::z(), x: V3::x() }, r)
    }

    pub fn psphere_on(&mut self, ax: &Ax3, r: f64) -> Result<Shape> {
        let half = Region {
            outer: Loop {
                curves: vec![
                    Curve2::Arc { center: nalgebra::Point2::origin(), radius: r, start_angle: -PI / 2.0, sweep: PI, source: None },
                    l2(nalgebra::Point2::new(0.0, r), nalgebra::Point2::new(0.0, -r)),
                ],
            },
            holes: vec![],
            source: None,
        };
        let res = self.revolve(&Profile::new(self.meridian(ax), vec![half]), Axis { origin: ax.origin, dir: nalgebra::Unit::new_normalize(ax.z) }, TAU)?;
        Ok(Shape::Body(res.bodies[0]))
    }

    /// `ptorus name R r`.
    pub fn ptorus(&mut self, big: f64, r: f64) -> Result<Shape> {
        self.ptorus_on(&Ax3 { origin: P3::origin(), z: V3::z(), x: V3::x() }, big, r)
    }

    pub fn ptorus_on(&mut self, ax: &Ax3, big: f64, r: f64) -> Result<Shape> {
        let region = Region { outer: Loop { curves: vec![Curve2::Circle { center: nalgebra::Point2::new(big, 0.0), radius: r, source: None }] }, holes: vec![], source: None };
        let res = self.revolve(&Profile::new(self.meridian(ax), vec![region]), Axis { origin: ax.origin, dir: nalgebra::Unit::new_normalize(ax.z) }, TAU)?;
        Ok(Shape::Body(res.bodies[0]))
    }

    /// The half plane through a placement's axis, x along its x and y along its axis.
    fn meridian(&self, ax: &Ax3) -> Plane {
        // y_dir = normal × x_dir must be z: normal = x × z.
        Plane { origin: ax.origin, x_dir: nalgebra::Unit::new_normalize(ax.x), normal: nalgebra::Unit::new_normalize(ax.x.cross(&ax.z)) }
    }

    fn revolve_section(&mut self, ax: &Ax3, pts: &[nalgebra::Point2<f64>], angle: f64) -> Result<Shape> {
        let n = pts.len();
        let curves = (0..n).map(|i| l2(pts[i], pts[(i + 1) % n])).collect();
        let region = Region { outer: Loop { curves }, holes: vec![], source: None };
        let res = self.revolve(&Profile::new(self.meridian(ax), vec![region]), Axis { origin: ax.origin, dir: nalgebra::Unit::new_normalize(ax.z) }, angle)?;
        Ok(Shape::Body(res.bodies[0]))
    }

    // --- Sweeps -----------------------------------------------------------------------------

    /// `prism name face dx dy dz` (along the face's normal).
    pub fn prism(&mut self, f: &Shape, dx: f64, dy: f64, dz: f64) -> Result<Shape> {
        let profile = face_profile(f)?;
        let v = V3::new(dx, dy, dz);
        let n = profile.plane.normal.into_inner();
        let along = v.dot(&n);
        if (v - n * along).norm() > 1e-9 * v.norm().max(1.0) {
            return Err(KernelError::Unsupported("prisms off the face's normal"));
        }
        let mut p = profile;
        if along < 0.0 {
            p.plane.origin += n * along;
        }
        let r = self.extrude(&p, Extent::Blind(along.abs()))?;
        Ok(Shape::Body(r.bodies[0]))
    }

    /// `revol name face x y z dx dy dz angle` (degrees).
    pub fn revol(&mut self, f: &Shape, x: f64, y: f64, z: f64, dx: f64, dy: f64, dz: f64, angle: f64) -> Result<Shape> {
        let profile = face_profile(f)?;
        let r = self.revolve(&profile, Axis { origin: P3::new(x, y, z), dir: nalgebra::Unit::new_normalize(V3::new(dx, dy, dz)) }, angle.to_radians())?;
        Ok(Shape::Body(r.bodies[0]))
    }

    /// `restore file name`: a shape from an OCCT BRep file (`explode` follows the file's order).
    pub fn restore(&mut self, path: &str) -> Result<Shape> {
        let bytes = std::fs::read(path).map_err(|e| err(&format!("{path}: {e}")))?;
        let (body, _, edges) = super::brep_occt::read_ordered(&bytes).map_err(|e| err(&format!("{path}: {e}")))?;
        let nf = body.faces.len();
        let b = self.insert(body);
        let infos = self.faces(b)?;
        let body = self.get(b)?;
        if infos.len() == nf {
            // A point on each face, near its middle.
            let faces = (0..nf)
                .map(|i| {
                    let s = &body.faces[i].surf;
                    let (u, v) = s.project(&infos[i].center, None);
                    s.eval(u, v)
                })
                .collect();
            set_order(b, OcctOrder { edges, faces });
        }
        Ok(Shape::Body(b))
    }

    // --- Booleans and transforms ------------------------------------------------------------

    fn body_of(&self, s: &Shape) -> Result<BodyId> {
        match s {
            Shape::Body(b) => Ok(*b),
            _ => Err(KernelError::Unsupported("booleans of shapes other than solids")),
        }
    }

    fn bop(&mut self, op: BoolOp, a: &Shape, b: &Shape) -> Result<Shape> {
        let (a, b) = (self.body_of(a)?, self.body_of(b)?);
        let r = match self.boolean(op, a, &[b]) {
            Err(KernelError::OperationFailed(m)) if m.contains("empty") => return Ok(Shape::Empty),
            r => r?,
        };
        match r.bodies.len() {
            1 => Ok(Shape::Body(r.bodies[0])),
            0 => Ok(Shape::Empty),
            _ => {
                // DRAW keeps a compound: fuse the pieces' properties by summing (one body).
                let mut acc = r.bodies[0];
                for &x in &r.bodies[1..] {
                    acc = self.compound(acc, x)?;
                }
                Ok(Shape::Body(acc))
            }
        }
    }

    /// Two bodies as one (their solids side by side, as DRAW's compounds).
    fn compound(&mut self, a: BodyId, b: BodyId) -> Result<BodyId> {
        let mut body = (**self.get(a)?).clone();
        let other = (**self.get(b)?).clone();
        let (_, _, fo) = body.append(&other);
        let so = body.shells.len();
        for sh in &other.shells {
            body.shells.push(super::topo::Shell { faces: sh.faces.iter().map(|f| f + fo).collect(), closed: sh.closed });
        }
        for s in &other.solids {
            body.solids.push(super::topo::Solid { shells: s.shells.iter().map(|x| x + so).collect() });
        }
        Ok(self.insert(body))
    }

    /// `bfuse result a b`.
    pub fn bfuse(&mut self, a: &Shape, b: &Shape) -> Result<Shape> {
        self.bop(BoolOp::Union, a, b)
    }

    /// `bcut result a b`.
    pub fn bcut(&mut self, a: &Shape, b: &Shape) -> Result<Shape> {
        self.bop(BoolOp::Subtract, a, b)
    }

    /// `bcommon result a b`.
    pub fn bcommon(&mut self, a: &Shape, b: &Shape) -> Result<Shape> {
        self.bop(BoolOp::Intersect, a, b)
    }

    fn motion(&mut self, s: &Shape, m: &Affine) -> Result<Shape> {
        Ok(match s {
            Shape::Vertex(p) => Shape::Vertex(m.point(p)),
            Shape::Edge(e) => Shape::Edge(DEdge { curve: e.curve.transformed(m), t0: e.t0, t1: e.t1 }),
            Shape::Wire(w) => Shape::Wire(w.iter().map(|e| DEdge { curve: e.curve.transformed(m), t0: e.t0, t1: e.t1 }).collect()),
            Shape::Face { plane, loops } => {
                let o = m.point(&plane.origin);
                let x = m.m * plane.x_dir.into_inner();
                let n = (m.m * plane.y_dir().into_inner()).cross(&x);
                Shape::Face {
                    plane: Plane { origin: o, x_dir: nalgebra::Unit::new_normalize(x), normal: nalgebra::Unit::new_normalize(n) },
                    loops: loops.iter().map(|l| l.iter().map(|e| DEdge { curve: e.curve.transformed(m), t0: e.t0, t1: e.t1 }).collect()).collect(),
                }
            }
            Shape::Body(b) => {
                let body = self.get(*b)?.transformed(m);
                let nb = self.insert(body);
                if let Some(o) = order_of(*b) {
                    set_order(nb, OcctOrder { edges: o.edges.iter().map(|p| m.point(p)).collect(), faces: o.faces.iter().map(|p| m.point(p)).collect() });
                }
                Shape::Body(nb)
            }
            _ => return Err(KernelError::Unsupported("moving a sub-shape")),
        })
    }

    /// `ttranslate shape dx dy dz`.
    pub fn ttranslate(&mut self, s: &Shape, dx: f64, dy: f64, dz: f64) -> Result<Shape> {
        self.motion(s, &Affine { m: nalgebra::Matrix3::identity(), t: V3::new(dx, dy, dz) })
    }

    /// `trotate shape x y z dx dy dz angle` (degrees).
    pub fn trotate(&mut self, s: &Shape, x: f64, y: f64, z: f64, dx: f64, dy: f64, dz: f64, angle: f64) -> Result<Shape> {
        let axis = nalgebra::Unit::new_normalize(V3::new(dx, dy, dz));
        let r = nalgebra::Rotation3::from_axis_angle(&axis, angle.to_radians()).into_inner();
        let o = V3::new(x, y, z);
        self.motion(s, &Affine { m: r, t: o - r * o })
    }

    /// `tscale shape x y z factor`.
    pub fn tscale(&mut self, s: &Shape, x: f64, y: f64, z: f64, f: f64) -> Result<Shape> {
        let o = V3::new(x, y, z);
        self.motion(s, &Affine { m: nalgebra::Matrix3::identity() * f, t: o - o * f })
    }

    /// `tmirror shape x y z nx ny nz`.
    pub fn tmirror(&mut self, s: &Shape, x: f64, y: f64, z: f64, nx: f64, ny: f64, nz: f64) -> Result<Shape> {
        let n = V3::new(nx, ny, nz).normalize();
        let m = nalgebra::Matrix3::identity() - n * n.transpose() * 2.0;
        let o = V3::new(x, y, z);
        self.motion(s, &Affine { m, t: o - m * o })
    }

    // --- Sub-shapes, blends -----------------------------------------------------------------

    /// `explode shape e`: the body's edges (in the kernel's order, not OCCT's).
    pub fn explode_edges(&self, s: &Shape) -> Result<Vec<Shape>> {
        let b = self.body_of(s)?;
        let body = self.get(b)?;
        let mut ids: Vec<usize> = (0..body.edges.len()).filter(|&i| !body.edges[i].degenerate).collect();
        // A primitive: OCCT's order, each reference point's nearest edge (by midpoint).
        if let Some(o) = order_of(b) {
            if o.edges.len() == ids.len() {
                let mid = |i: usize| body.edges[i].point(body.edges[i].mid_t());
                ids = o.edges.iter().map(|p| *ids.iter().min_by(|&&x, &&y| (mid(x) - p).norm().total_cmp(&(mid(y) - p).norm())).unwrap()).collect();
            }
        }
        Ok(ids.into_iter().map(|i| Shape::BodyEdge(b, EdgeId(i as u64))).collect())
    }

    /// `nexplode shape e`: the edges sorted by their centres of mass (DRAW's key
    /// x·999 + y·99 + z·0.9; a stable order, the same as OCCT's for the same shape).
    pub fn nexplode_edges(&self, s: &Shape) -> Result<Vec<Shape>> {
        let b = self.body_of(s)?;
        let body = self.get(b)?;
        let mut ids: Vec<(f64, usize)> = Vec::new();
        for (i, e) in body.edges.iter().enumerate() {
            if let Some(c) = linear_centre(&[e]) {
                ids.push((nexplode_key(&c), i));
            }
        }
        ids.sort_by(|x, y| x.0.total_cmp(&y.0));
        Ok(ids.into_iter().map(|(_, i)| Shape::BodyEdge(b, EdgeId(i as u64))).collect())
    }

    /// `nexplode shape f`: the faces sorted by the centres of mass of their boundaries.
    pub fn nexplode_faces(&self, s: &Shape) -> Result<Vec<Shape>> {
        let b = self.body_of(s)?;
        let body = self.get(b)?;
        let infos = self.faces(b)?;
        let mut ids: Vec<(f64, usize)> = Vec::new();
        for (i, f) in body.faces.iter().enumerate() {
            let edges: Vec<&super::topo::Edge> = f.loops.iter().flatten().map(|c| &body.edges[c.edge]).collect();
            let c = linear_centre(&edges).unwrap_or(infos[i].center);
            ids.push((nexplode_key(&c), i));
        }
        ids.sort_by(|x, y| x.0.total_cmp(&y.0));
        Ok(ids.into_iter().map(|(_, i)| Shape::BodyFace(b, infos[i].id)).collect())
    }

    /// `explode shape f`.
    pub fn explode_faces(&self, s: &Shape) -> Result<Vec<Shape>> {
        let b = self.body_of(s)?;
        let infos = self.faces(b)?;
        if let Some(o) = order_of(b) {
            if o.faces.len() == infos.len() {
                // The face nearest each reference point (by distance to its surface and centroid).
                let body = self.get(b)?;
                return Ok(o
                    .faces
                    .iter()
                    .map(|p| {
                        let best = (0..body.faces.len())
                            .min_by(|&x, &y| {
                                let d = |i: usize| body.faces[i].surf.distance(p) + 1e-3 * (infos[i].center - p).norm();
                                d(x).total_cmp(&d(y))
                            })
                            .unwrap();
                        Shape::BodyFace(b, infos[best].id)
                    })
                    .collect());
            }
        }
        Ok(infos.into_iter().map(|f| Shape::BodyFace(b, f.id)).collect())
    }

    /// `blend result shape r e1 e2 …` (one radius).
    pub fn blend(&mut self, s: &Shape, r: f64, edges: &[&Shape]) -> Result<Shape> {
        let b = self.body_of(s)?;
        let ids = edges.iter().map(|e| match e { Shape::BodyEdge(eb, id) if *eb == b => Ok(*id), _ => Err(err("blend takes edges of the shape")) }).collect::<Result<Vec<_>>>()?;
        let res = self.fillet(b, &ids, r)?;
        Ok(Shape::Body(res.bodies[0]))
    }

    /// `chamfer result shape e d`.
    pub fn chamfer_d(&mut self, s: &Shape, edges: &[&Shape], d: f64) -> Result<Shape> {
        let b = self.body_of(s)?;
        let ids = edges.iter().map(|e| match e { Shape::BodyEdge(eb, id) if *eb == b => Ok(*id), _ => Err(err("chamfer takes edges of the shape")) }).collect::<Result<Vec<_>>>()?;
        let res = self.chamfer(b, &ids, ChamferSpec::EqualDistance(d))?;
        Ok(Shape::Body(res.bodies[0]))
    }

    // --- 2D fillets and chamfers ------------------------------------------------------------

    /// `fillet2d result e1 e2 r`: the arc of radius `r` rounding the corner where two coplanar
    /// edges meet.
    pub fn fillet2d(&self, e1: &Shape, e2: &Shape, r: f64) -> Result<Shape> {
        let (Shape::Edge(a), Shape::Edge(b)) = (e1, e2) else { return Err(err("fillet2d takes two edges")) };
        let (a, b) = meet(a, b)?;
        let f = corner_fillet(&a, &b, r)?;
        Ok(Shape::Edge(f.arc))
    }

    /// `fillet2d result wire r`: the wire with each corner between its edges rounded.
    pub fn fillet2d_wire(&self, w: &Shape, r: f64) -> Result<Shape> {
        let Shape::Wire(w) = w else { return Err(err("fillet2d takes a wire")) };
        let mut edges = w.clone();
        let closed = edges.len() > 1 && (edges[0].start() - edges.last().unwrap().end()).norm() < 1e-7;
        let mut out: Vec<DEdge> = Vec::new();
        let n = edges.len();
        let corners = if closed { n } else { n - 1 };
        let mut arcs: Vec<Option<DEdge>> = vec![None; n];
        for i in 0..corners {
            let j = (i + 1) % n;
            if tangent_at(&edges[i]).1.normalize().dot(&tangent_at(&edges[j]).0.normalize()) > 1.0 - 1e-9 {
                continue;
            }
            let f = corner_fillet(&edges[i], &edges[j], r)?;
            edges[i] = f.a;
            edges[j] = f.b;
            arcs[i] = Some(f.arc);
        }
        for i in 0..n {
            out.push(edges[i].clone());
            if let Some(a) = &arcs[i] {
                out.push(a.clone());
            }
        }
        Ok(Shape::Wire(out))
    }

    /// `chamfer2d result e1 e2 d1 d2`: the segment cutting the corner `d1` along the first edge
    /// and `d2` along the second.
    pub fn chamfer2d(&self, e1: &Shape, e2: &Shape, d1: f64, d2: f64) -> Result<Shape> {
        let (Shape::Edge(a), Shape::Edge(b)) = (e1, e2) else { return Err(err("chamfer2d takes two edges")) };
        let (a, b) = meet(a, b)?;
        let pa = point_at_length(&a, a.length() - d1, true)?;
        let pb = point_at_length(&b, d2, false)?;
        let (curve, l) = Curve::line(pa, pb);
        Ok(Shape::Edge(DEdge { curve, t0: 0.0, t1: l }))
    }

    // --- Properties -------------------------------------------------------------------------

    /// `lprops`: the total length of the shape's edges.
    pub fn lprops(&self, s: &Shape) -> Result<f64> {
        Ok(match s {
            Shape::Vertex(_) => 0.0,
            Shape::Edge(e) => e.length(),
            Shape::Wire(w) => w.iter().map(DEdge::length).sum(),
            Shape::Face { loops, .. } => loops.iter().flatten().map(DEdge::length).sum(),
            Shape::Body(b) => self.edges(*b)?.iter().map(|e| e.length).sum(),
            Shape::BodyEdge(b, id) => self.edges(*b)?.iter().find(|e| e.id == *id).map_or(0.0, |e| e.length),
            Shape::BodyFace(..) => return Err(KernelError::Unsupported("lprops of a face of a body")),
            Shape::Empty => 0.0,
        })
    }

    /// `sprops`: the area of the shape's faces.
    pub fn sprops(&mut self, s: &Shape) -> Result<f64> {
        Ok(match s {
            Shape::Face { .. } => {
                // The area as the volume of a unit-thick prism of the face.
                let p = face_profile(s)?;
                let r = self.extrude(&p, Extent::Blind(1.0))?;
                let v = self.mass_properties(r.bodies[0])?.volume;
                self.release(r.bodies[0]);
                v
            }
            Shape::Body(b) => self.mass_properties(*b)?.surface_area,
            Shape::BodyFace(b, id) => self.faces(*b)?.iter().find(|f| f.id == *id).map_or(0.0, |f| f.area),
            _ => 0.0,
        })
    }

    /// `vprops`: the volume of the shape's solids.
    pub fn vprops(&self, s: &Shape) -> Result<f64> {
        Ok(match s {
            Shape::Body(b) => self.mass_properties(*b)?.volume,
            _ => 0.0,
        })
    }

    /// `checkprops shape -s empty` (and -v, -l): nothing there.
    pub fn checkprops_empty(&mut self, s: &Shape) -> Result<()> {
        match s {
            Shape::Empty => Ok(()),
            _ => {
                let (a, v) = (self.sprops(s)?, self.vprops(s)?);
                if a.abs() < 1e-9 && v.abs() < 1e-9 { Ok(()) } else { Err(err(&format!("the result isn't empty (area {a}, volume {v})"))) }
            }
        }
    }

    /// `checkprops shape -l length`.
    pub fn checkprops_l(&self, s: &Shape, want: f64) -> Result<()> {
        check_rel("length", self.lprops(s)?, want)
    }

    /// `checkprops shape -s area`.
    pub fn checkprops_s(&mut self, s: &Shape, want: f64) -> Result<()> {
        let a = self.sprops(s)?;
        check_rel("area", a, want)
    }

    /// `checkprops shape -v volume`.
    pub fn checkprops_v(&self, s: &Shape, want: f64) -> Result<()> {
        check_rel("volume", self.vprops(s)?, want)
    }

    /// `nbshapes`: the counts of the shape's sub-shapes.
    pub fn nbshapes(&self, s: &Shape) -> Result<NbShapes> {
        Ok(match s {
            Shape::Vertex(_) => NbShapes { vertex: 1, ..Default::default() },
            Shape::Edge(e) => NbShapes { vertex: if (e.start() - e.end()).norm() < 1e-9 { 1 } else { 2 }, edge: 1, ..Default::default() },
            Shape::Wire(w) => NbShapes { vertex: wire_vertices(w), edge: w.len(), wire: 1, ..Default::default() },
            Shape::Face { loops, .. } => NbShapes {
                vertex: loops.iter().map(|l| wire_vertices(l)).sum(),
                edge: loops.iter().map(|l| l.len()).sum(),
                wire: loops.len(),
                face: 1,
                ..Default::default()
            },
            Shape::Body(b) => {
                let body = self.get(*b)?;
                NbShapes {
                    vertex: body.verts.len(),
                    edge: body.edges.len(),
                    wire: body.faces.iter().map(|f| f.loops.len()).sum(),
                    face: body.faces.len(),
                    shell: body.shells.len(),
                    solid: body.solids.len(),
                }
            }
            Shape::BodyEdge(..) => NbShapes { vertex: 2, edge: 1, ..Default::default() },
            Shape::BodyFace(..) => NbShapes { face: 1, ..Default::default() },
            Shape::Empty => NbShapes::default(),
        })
    }

    /// `checkshape`: the body is valid.
    pub fn checkshape(&self, s: &Shape) -> Result<()> {
        if let Shape::Body(b) = s {
            let issues = self.get(*b)?.check();
            if !issues.is_empty() {
                return Err(err(&format!("checkshape: {issues:?}")));
            }
        }
        Ok(())
    }
}

/// `checknbshapes` against the counts given (each `Some` compared).
pub fn check_nb(got: NbShapes, vertex: Option<usize>, edge: Option<usize>, wire: Option<usize>, face: Option<usize>, shell: Option<usize>, solid: Option<usize>) -> Result<()> {
    let mut bad = Vec::new();
    for (name, g, w) in [("vertex", got.vertex, vertex), ("edge", got.edge, edge), ("wire", got.wire, wire), ("face", got.face, face), ("shell", got.shell, shell), ("solid", got.solid, solid)] {
        if let Some(w) = w {
            if g != w {
                bad.push(format!("{name} {g} (expected {w})"));
            }
        }
    }
    if bad.is_empty() { Ok(()) } else { Err(err(&format!("checknbshapes: {}", bad.join(", ")))) }
}

fn nexplode_key(c: &P3) -> f64 {
    c.x * 999.0 + c.y * 99.0 + c.z * 0.9
}

/// The centre of mass of edges by length (`BRepGProp::LinearProperties`).
fn linear_centre(edges: &[&super::topo::Edge]) -> Option<P3> {
    let (mut acc, mut total) = (V3::zeros(), 0.0);
    for e in edges {
        if e.degenerate {
            continue;
        }
        let n = 64;
        let mut prev = e.point(e.t[0]);
        for i in 1..=n {
            let p = e.point(e.t[0] + (e.t[1] - e.t[0]) * i as f64 / n as f64);
            let l = (p - prev).norm();
            acc += (prev.coords + p.coords) * 0.5 * l;
            total += l;
            prev = p;
        }
    }
    (total > 0.0).then(|| P3::from(acc / total))
}

fn l2(a: nalgebra::Point2<f64>, b: nalgebra::Point2<f64>) -> Curve2 {
    Curve2::Line { a, b, source: None }
}

fn wire_vertices(w: &[DEdge]) -> usize {
    if w.is_empty() {
        return 0;
    }
    let closed = (w[0].start() - w.last().unwrap().end()).norm() < 1e-7;
    if closed { w.len() } else { w.len() + 1 }
}

/// Turns edges round so each starts where the one before ends.
fn orient_chain(es: &mut [DEdge]) -> Result<()> {
    if es.len() < 2 {
        return Ok(());
    }
    // The first edge's direction: the one whose end meets the second edge.
    let near = |p: P3, e: &DEdge| (e.start() - p).norm().min((e.end() - p).norm());
    if near(es[0].start(), &es[1]) < near(es[0].end(), &es[1]) {
        es[0] = es[0].reversed();
    }
    for i in 1..es.len() {
        let p = es[i - 1].end();
        if (es[i].end() - p).norm() < (es[i].start() - p).norm() {
            es[i] = es[i].reversed();
        }
        if (es[i].start() - p).norm() > 1e-6 {
            return Err(err("the wire's edges don't connect"));
        }
    }
    Ok(())
}

/// The two edges as a corner: `a` running into the shared point, `b` out of it.
fn meet(a: &DEdge, b: &DEdge) -> Result<(DEdge, DEdge)> {
    for ra in [false, true] {
        for rb in [false, true] {
            let a2 = if ra { a.reversed() } else { a.clone() };
            let b2 = if rb { b.reversed() } else { b.clone() };
            if (a2.end() - b2.start()).norm() < 1e-6 {
                return Ok((a2, b2));
            }
        }
    }
    Err(err("the edges don't share a vertex"))
}

/// The tangents of an edge at its start and end (in its direction of travel).
fn tangent_at(e: &DEdge) -> (V3, V3) {
    let s = if e.t1 >= e.t0 { 1.0 } else { -1.0 };
    (e.curve.tangent(e.t0) * s, e.curve.tangent(e.t1) * s)
}

/// A point `d` along an edge from its start (or its end when `from_end` is false … measured
/// from the start either way: `d` is the length from the start).
fn point_at_length(e: &DEdge, d: f64, _from_end: bool) -> Result<P3> {
    let total = e.length();
    if d < -1e-9 || d > total + 1e-9 {
        return Err(err("the chamfer is longer than its edge"));
    }
    let (mut lo, mut hi) = (e.t0, e.t1);
    for _ in 0..80 {
        let m = 0.5 * (lo + hi);
        let l = e.curve.length(e.t0.min(m), e.t0.max(m));
        if l < d {
            lo = m;
        } else {
            hi = m;
        }
    }
    Ok(e.curve.point(0.5 * (lo + hi)))
}

/// A best-fit plane through a wire's sample points (Newell's normal); `None` if not planar.
fn fit_plane(edges: &[DEdge]) -> Option<Plane> {
    let mut pts: Vec<P3> = Vec::new();
    for e in edges {
        for i in 0..16 {
            pts.push(e.curve.point(e.t0 + (e.t1 - e.t0) * i as f64 / 16.0));
        }
    }
    let n = pts.len();
    let mut nrm = V3::zeros();
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        nrm += V3::new((a.y - b.y) * (a.z + b.z), (a.z - b.z) * (a.x + b.x), (a.x - b.x) * (a.y + b.y));
    }
    // A single circle edge: its own normal.
    if edges.len() == 1 {
        if let Curve::Circle { x, y, .. } = &edges[0].curve {
            nrm = x.cross(y) * if edges[0].t1 >= edges[0].t0 { 1.0 } else { -1.0 };
        }
    }
    let nrm = nrm.try_normalize(1e-12)?;
    let o = pts[0];
    if pts.iter().any(|p| (p - o).dot(&nrm).abs() > 1e-6) {
        return None;
    }
    let xd = ax2_x(&nrm);
    Some(Plane { origin: o, x_dir: nalgebra::Unit::new_normalize(xd), normal: nalgebra::Unit::new_normalize(nrm) })
}

/// The face as a kernel profile (lines and circular arcs in its plane).
fn face_profile(f: &Shape) -> Result<Profile> {
    let Shape::Face { plane, loops } = f else { return Err(err("not a planar face")) };
    let to2 = |p: &P3| {
        let d = p - plane.origin;
        nalgebra::Point2::new(d.dot(&plane.x_dir), d.dot(&plane.y_dir()))
    };
    let n = plane.normal.into_inner();
    let conv = |l: &Vec<DEdge>| -> Result<Loop> {
        let mut curves = Vec::new();
        for e in l {
            match &e.curve {
                Curve::Line { .. } => curves.push(l2(to2(&e.start()), to2(&e.end()))),
                Curve::Circle { c, x, y, r } => {
                    let flip = x.cross(y).dot(&n) < 0.0;
                    let cc = to2(c);
                    let full = (e.t1 - e.t0).abs() >= TAU - 1e-9;
                    if full && l.len() == 1 {
                        curves.push(Curve2::Circle { center: cc, radius: *r, source: None });
                        continue;
                    }
                    let s = to2(&e.start()) - cc;
                    let start = s.y.atan2(s.x);
                    let sweep = (e.t1 - e.t0) * if flip { -1.0 } else { 1.0 };
                    curves.push(Curve2::Arc { center: cc, radius: *r, start_angle: start, sweep, source: None });
                }
                _ => return Err(KernelError::Unsupported("faces bounded by curves other than lines and circles")),
            }
        }
        Ok(Loop { curves })
    };
    let outer = conv(&loops[0])?;
    let holes = loops[1..].iter().map(conv).collect::<Result<Vec<_>>>()?;
    Ok(Profile::new(*plane, vec![Region { outer, holes, source: None }]))
}

/// A planar primitive for the fillet: a line or a circle in the corner's plane.
#[derive(Clone, Copy)]
enum Prim2 {
    Line { p: nalgebra::Point2<f64>, d: nalgebra::Vector2<f64> },
    Circle { c: nalgebra::Point2<f64>, r: f64 },
}

struct Fillet {
    a: DEdge,
    b: DEdge,
    arc: DEdge,
}

/// The fillet of radius `r` at the corner where `a` ends and `b` starts (lines and arcs in one
/// plane): the circle tangent to both, nearest the corner, with both edges trimmed to it.
fn corner_fillet(a: &DEdge, b: &DEdge, r: f64) -> Result<Fillet> {
    let v = a.end();
    let (_, ta) = tangent_at(a);
    let (tb, _) = tangent_at(b);
    // The plane: from a circle's axis, or the two tangents.
    let mut n = ta.cross(&tb);
    for e in [a, b] {
        if let Curve::Circle { x, y, .. } = &e.curve {
            n = x.cross(y);
        }
    }
    let n = n.try_normalize(1e-12).ok_or_else(|| err("the edges are tangent at the corner"))?;
    let xd = ta.normalize();
    let yd = n.cross(&xd);
    let to2 = |p: &P3| nalgebra::Point2::new((p - v).dot(&xd), (p - v).dot(&yd));
    let to3 = |q: &nalgebra::Point2<f64>| v + xd * q.x + yd * q.y;
    let prim = |e: &DEdge| -> Result<Prim2> {
        match &e.curve {
            Curve::Line { o, d } => Ok(Prim2::Line { p: to2(o), d: nalgebra::Vector2::new(d.dot(&xd), d.dot(&yd)) }),
            Curve::Circle { c, r, .. } => Ok(Prim2::Circle { c: to2(c), r: *r }),
            _ => Err(KernelError::Unsupported("2D fillets of curves other than lines and circles")),
        }
    };
    let (pa, pb) = (prim(a)?, prim(b)?);
    // Centres: on both curves' offsets by r (each side).
    let mut best: Option<(f64, nalgebra::Point2<f64>, P3, P3, f64, f64)> = None;
    for sa in [-1.0, 1.0] {
        for sb in [-1.0, 1.0] {
            for c in intersect(&offset(&pa, sa * r), &offset(&pb, sb * r)) {
                let c3 = to3(&c);
                // Feet on each edge, within the edges.
                let (Some(ta_), Some(tb_)) = (foot_param(a, &c3), foot_param(b, &c3)) else { continue };
                let (fa, fb) = (a.curve.point(ta_), b.curve.point(tb_));
                if ((fa - c3).norm() - r).abs() > 1e-6 * r.max(1.0) || ((fb - c3).norm() - r).abs() > 1e-6 * r.max(1.0) {
                    continue;
                }
                // Not a point fillet (where the curves touch, at a cusp).
                if (fa - fb).norm() < 1e-9 * r.max(1.0) {
                    continue;
                }
                let score = (fa - v).norm() + (fb - v).norm();
                if best.as_ref().is_none_or(|x| score < x.0) {
                    best = Some((score, c, fa, fb, ta_, tb_));
                }
            }
        }
    }
    let (_, c, fa, fb, ta_, tb_) = best.ok_or_else(|| err("no fillet of that radius fits the corner"))?;
    let c3 = to3(&c);
    let ua = (fa - c3) / r;
    let ub = (fb - c3) / r;
    // The arc from fa to fb (a fillet turns by less than a half turn).
    let y = (ub - ua * ua.dot(&ub)).try_normalize(1e-12).unwrap_or_else(|| n.cross(&ua));
    let x = ua;
    let ang = ub.dot(&y).atan2(ub.dot(&x));
    let arc = DEdge { curve: Curve::Circle { c: c3, x, y, r }, t0: 0.0, t1: ang };
    Ok(Fillet { a: DEdge { curve: a.curve.clone(), t0: a.t0, t1: ta_ }, b: DEdge { curve: b.curve.clone(), t0: tb_, t1: b.t1 }, arc })
}

fn offset(p: &Prim2, s: f64) -> Prim2 {
    match *p {
        Prim2::Line { p, d } => {
            let nrm = nalgebra::Vector2::new(-d.y, d.x).normalize();
            Prim2::Line { p: p + nrm * s, d }
        }
        Prim2::Circle { c, r } => Prim2::Circle { c, r: r + s },
    }
}

fn intersect(a: &Prim2, b: &Prim2) -> Vec<nalgebra::Point2<f64>> {
    use nalgebra::Vector2 as V2_;
    match (*a, *b) {
        (Prim2::Line { p: p1, d: d1 }, Prim2::Line { p: p2, d: d2 }) => {
            let den = d1.x * d2.y - d1.y * d2.x;
            if den.abs() < 1e-14 {
                return vec![];
            }
            let w = p2 - p1;
            let t = (w.x * d2.y - w.y * d2.x) / den;
            vec![p1 + d1 * t]
        }
        (Prim2::Line { p, d }, Prim2::Circle { c, r }) | (Prim2::Circle { c, r }, Prim2::Line { p, d }) => {
            if r <= 0.0 {
                return vec![];
            }
            let d = d.normalize();
            let w = p - c;
            let bq = w.dot(&d);
            let cq = w.dot(&w) - r * r;
            let disc = bq * bq - cq;
            if disc < 0.0 {
                return vec![];
            }
            let s = disc.sqrt();
            vec![p + d * (-bq - s), p + d * (-bq + s)]
        }
        (Prim2::Circle { c: c1, r: r1 }, Prim2::Circle { c: c2, r: r2 }) => {
            if r1 <= 0.0 || r2 <= 0.0 {
                return vec![];
            }
            let dv = c2 - c1;
            let dd = dv.norm();
            if dd < 1e-14 || dd > r1 + r2 || dd < (r1 - r2).abs() {
                return vec![];
            }
            let a = (r1 * r1 - r2 * r2 + dd * dd) / (2.0 * dd);
            let h = (r1 * r1 - a * a).max(0.0).sqrt();
            let m = c1 + dv * (a / dd);
            let perp = V2_::new(-dv.y, dv.x) / dd;
            vec![m + perp * h, m - perp * h]
        }
    }
}

/// The parameter of the point of an edge nearest `p`, if it lies within the edge.
fn foot_param(e: &DEdge, p: &P3) -> Option<f64> {
    let (lo, hi) = (e.t0.min(e.t1), e.t0.max(e.t1));
    let t = match &e.curve {
        Curve::Line { o, d } => (p - o).dot(d),
        Curve::Circle { c, x, y, .. } => {
            let w = p - c;
            let a = w.dot(y).atan2(w.dot(x));
            // Into the edge's range.
            let mut a = a;
            while a < lo - 1e-9 {
                a += TAU;
            }
            while a > hi + 1e-9 {
                a -= TAU;
            }
            a
        }
        _ => return None,
    };
    (t >= lo - 1e-9 && t <= hi + 1e-9).then_some(t.clamp(lo, hi))
}
