//! The rolling ball's geometry for fillets by local trimming ([`super::fillet`]): where a ball
//! of radius r touching two faces sits, the rails (its contact curves) and the fillet surface
//! between them, for any pair of surfaces.
//!
//! A ball touching faces A and B has its centre on both offset surfaces `O(u, v) = S(u, v) +
//! k·n(u, v)` (n the outward unit normal, k = −r inside a convex edge's material, +r outside a
//! concave one). With a fourth condition the centre is a point: the plane square to the edge
//! (a section), a third face (a corner ball touching three faces) or a point of a face boundary
//! (where a rail leaves its face). Each is solved by Newton's method in the faces' (u, v), as
//! SMLib's fillet solver does (NVIDIA usd-brep, Apache-2.0: `SmConstantRadiusFS::LoadJacobian`,
//! SmFilletStandardSolver.cpp, and `SmFilletSphereSolveENFO::Evaluate`, SmFilletSolver.cpp),
//! with the offset surfaces' derivatives from the surfaces' second derivatives (Weingarten).
//!
//! The fillet surface is exact where it has a closed form: a cylinder along a straight edge
//! between surfaces that don't change along it (planes, parallel cylinders, extrusions), a torus
//! along a circle between surfaces of revolution about its axis. Otherwise it is a rational
//! B-spline: the exact circular sections (rational quadratic arcs) at stations along the edge,
//! interpolated along it, so the rails (its v = 0 and v = 1 iso-curves) pass through the exact
//! contact points at every station; stations are added until the surface between them stays
//! within the tolerance of the true sections.

use super::curve::Curve;
use super::math::*;
use super::nurbs::{hom, NurbsCurve, NurbsSurface, H};
use super::surface::Surface;
use super::topo::Face;
use nalgebra::{Matrix3, Matrix4, SMatrix, SVector, Vector2};
use super::tol;

/// A point where the ball touches a face: (u, v) on the face's surface, the point and the
/// face's outward unit normal there.
#[derive(Debug, Clone, Copy)]
pub struct Contact {
    pub uv: P2,
    pub p: P3,
    pub n: V3,
}

/// A ball touching two faces: its centre and the contacts (on the first face, on the second).
/// A flat ball's section is the straight line between its contacts (a chamfer: SMLib's
/// constant-radius solver with a linear cross section, `SmLinearCrossSectionFSG`).
#[derive(Debug, Clone, Copy)]
pub struct Ball {
    pub c: P3,
    pub q: [Contact; 2],
    pub flat: bool,
}

impl Ball {
    /// The point of the ball's section a fraction `w` of the way from the first contact to
    /// the second (round the arc, the short way, or along the line).
    pub fn arc_point(&self, w: f64) -> P3 {
        if self.flat {
            return self.q[0].p + (self.q[1].p - self.q[0].p) * w;
        }
        let a = self.q[0].p - self.c;
        let b = self.q[1].p - self.c;
        let r = a.norm();
        let th = a.normalize().dot(&b.normalize()).clamp(-1.0, 1.0).acos();
        if th < 1e-12 {
            return self.q[0].p;
        }
        // Slerp.
        let (s0, s1) = (((1.0 - w) * th).sin() / th.sin(), (w * th).sin() / th.sin());
        self.c + (a * s0 + b * s1).normalize() * r
    }

    /// The section arc from the first contact to the second as a circle, with its parameter
    /// range (0 at the first contact).
    pub fn arc(&self) -> Option<(Curve, f64, f64)> {
        if self.flat {
            let (c, l) = Curve::line(self.q[0].p, self.q[1].p);
            return (l > 1e-12).then_some((c, 0.0, l));
        }
        let a = self.q[0].p - self.c;
        let b = self.q[1].p - self.c;
        let r = a.norm();
        let x = a.normalize();
        let yv = b - x * b.dot(&x);
        if yv.norm() < 1e-12 * (1.0 + r) || r < 1e-12 {
            return None;
        }
        let y = yv.normalize();
        let th = b.dot(&y).atan2(b.dot(&x));
        Some((Curve::Circle { c: self.c, x, y, r }, 0.0, th))
    }

    /// How far `x` (in the section's plane) lies outside the section: past the arc from the
    /// centre, or past the line, away from the centre.
    pub fn sdist(&self, x: &P3) -> f64 {
        if self.flat {
            let chord = self.q[1].p - self.q[0].p;
            let out = (self.q[0].p - self.c) + (self.q[1].p - self.c);
            let n = (out - chord * (out.dot(&chord) / chord.norm_squared().max(tol::TINY))).normalize();
            return (x - self.q[0].p).dot(&n);
        }
        (x - self.c).norm() - (self.q[0].p - self.c).norm()
    }

    /// The fraction of the way across the section from contact `k` that `x` lies.
    pub fn frac(&self, k: usize, x: &P3) -> f64 {
        let (a, o) = (self.q[k].p, self.q[1 - k].p);
        if self.flat {
            let d = o - a;
            return (x - a).dot(&d) / d.norm_squared().max(tol::TINY);
        }
        let (a, o) = (a - self.c, o - self.c);
        let whole = a.angle(&o);
        if whole < 1e-12 { 0.0 } else { a.angle(&(x - self.c)) / whole }
    }
}

/// The offset surface of a face at signed distance `k` along its outward normal, at (u, v):
/// the point and its derivatives along u and v, and the contact there.
struct Offset {
    o: P3,
    du: V3,
    dv: V3,
    contact: Contact,
}

/// The offset point of face `f` at (u, v) for offset `k`, with derivatives. `None` where the
/// surface is singular (a pole, an apex).
fn offset(f: &Face, k: f64, uv: P2) -> Option<Offset> {
    let d = f.surf.d2(uv.x, uv.y);
    let s = P3::from(d[0]);
    let nn = d[1].cross(&d[2]);
    let l = nn.norm();
    if !(l > 1e-14 * (1.0 + d[1].norm() * d[2].norm())) {
        return None;
    }
    let sg = if f.sense { 1.0 } else { -1.0 };
    let n = nn / l;
    // d(N/|N|) = (dN − n (n·dN)) / |N|, N = Su × Sv.
    let dnu = d[3].cross(&d[2]) + d[1].cross(&d[4]);
    let dnv = d[4].cross(&d[2]) + d[1].cross(&d[5]);
    let dn = |x: V3| (x - n * n.dot(&x)) / l;
    let (nu, nv) = (dn(dnu) * sg, dn(dnv) * sg);
    let n = n * sg;
    Some(Offset { o: s + n * k, du: d[1] + nu * k, dv: d[2] + nv * k, contact: Contact { uv, p: s, n } })
}

/// The contact of face `f` at (u, v) (its point and outward normal).
pub fn contact(f: &Face, uv: P2) -> Contact {
    let p = f.surf.eval(uv.x, uv.y);
    let n = f.surf.normal(uv.x, uv.y) * if f.sense { 1.0 } else { -1.0 };
    Contact { uv, p, n }
}

/// The (u, v) of the point of face `f`'s surface nearest `p`, near `hint`.
pub fn foot(f: &Face, p: &P3, hint: Option<P2>) -> P2 {
    let (u, v) = f.surf.project(p, hint.map(|h| (h.x, h.y)));
    P2::new(u, v)
}

/// Keeps a Newton step on a B-spline surface near its parameter box: its end spans' polynomials
/// run on a quarter of the box past it (a fillet's ball may touch a face's surface a little past
/// its boundary at a fillet's ends); other surfaces extend naturally.
fn clamp_uv(f: &Face, uv: P2) -> P2 {
    match &f.surf {
        Surface::Nurbs(n) => {
            let ((u0, u1), (v0, v1)) = n.range();
            let (mu, mv) = (0.25 * (u1 - u0), 0.25 * (v1 - v0));
            P2::new(uv.x.clamp(u0 - mu, u1 + mu), uv.y.clamp(v0 - mv, v1 + mv))
        }
        _ => uv,
    }
}

/// The scale Newton's convergence is measured against.
fn scale(p: &P3, r: f64) -> f64 {
    1.0 + p.coords.abs().max() + r
}

/// A Newton step `d` on contact `uv`, cut to `lim` in 3D.
fn step(f: &Face, o: &Offset, uv: P2, d: Vector2<f64>, factor: f64) -> P2 {
    let _ = o;
    clamp_uv(f, uv + d * factor)
}

/// The factor that keeps a step no longer than `lim` in 3D.
fn damp(len: f64, lim: f64) -> f64 {
    if len > lim { lim / len } else { 1.0 }
}

/// The ball touching faces `f[0]` and `f[1]` (offsets `k`) whose centre lies in the plane
/// through `p0` square to `t`, from contacts near `guess`.
pub fn ball_in_plane(f: [&Face; 2], k: [f64; 2], p0: &P3, t: &V3, guess: [P2; 2]) -> Option<Ball> {
    let mut x = guess;
    let sc = scale(p0, k[0].abs());
    // Steps no longer than the ball's size, or (a ball shrinking to nothing) the guess's gap.
    let mut lim = 2.0 * k[0].abs();
    for it in 0..60 {
        let a = offset(f[0], k[0], x[0])?;
        let b = offset(f[1], k[1], x[1])?;
        if it == 0 {
            lim = lim.max(2.0 * (a.contact.p - b.contact.p).norm()).max(tol::REL * sc);
        }
        let r3 = a.o - b.o;
        let rp = (a.o - p0).dot(t);
        let res = SVector::<f64, 4>::new(r3.x, r3.y, r3.z, rp);
        if res.norm() < 1e-11 * sc {
            return Some(Ball { c: P3::from((a.o.coords + b.o.coords) * 0.5), q: [a.contact, b.contact], flat: false });
        }
        let m = Matrix4::new(
            a.du.x, a.dv.x, -b.du.x, -b.dv.x, //
            a.du.y, a.dv.y, -b.du.y, -b.dv.y, //
            a.du.z, a.dv.z, -b.du.z, -b.dv.z, //
            t.dot(&a.du), t.dot(&a.dv), 0.0, 0.0,
        );
        let dx = m.lu().solve(&(-res))?;
        let (da, db) = (Vector2::new(dx[0], dx[1]), Vector2::new(dx[2], dx[3]));
        // Newton overshoots on curved faces.
        let len = (a.du * da.x + a.dv * da.y).norm().max((b.du * db.x + b.dv * db.y).norm());
        let g = damp(len, lim);
        x = [step(f[0], &a, x[0], da, g), step(f[1], &b, x[1], db, g)];
    }
    None
}

/// The ball touching three faces (offsets `k`): its centre and the contacts, from contacts
/// near `guess`.
pub fn ball3(f: [&Face; 3], k: [f64; 3], guess: [P2; 3]) -> Option<(P3, [Contact; 3])> {
    let mut x = guess;
    let mut sc = 0.0;
    for _ in 0..80 {
        let a = offset(f[0], k[0], x[0])?;
        let b = offset(f[1], k[1], x[1])?;
        let c = offset(f[2], k[2], x[2])?;
        if sc == 0.0 {
            sc = scale(&a.o, k[0].abs());
        }
        let (r1, r2) = (a.o - b.o, a.o - c.o);
        let res = SVector::<f64, 6>::new(r1.x, r1.y, r1.z, r2.x, r2.y, r2.z);
        if res.norm() < 1e-11 * sc {
            let cen = P3::from((a.o.coords + b.o.coords + c.o.coords) / 3.0);
            return Some((cen, [a.contact, b.contact, c.contact]));
        }
        let mut m = SMatrix::<f64, 6, 6>::zeros();
        for i in 0..3 {
            m[(i, 0)] = a.du[i];
            m[(i, 1)] = a.dv[i];
            m[(i, 2)] = -b.du[i];
            m[(i, 3)] = -b.dv[i];
            m[(i + 3, 0)] = a.du[i];
            m[(i + 3, 1)] = a.dv[i];
            m[(i + 3, 4)] = -c.du[i];
            m[(i + 3, 5)] = -c.dv[i];
        }
        let dx = m.lu().solve(&(-res))?;
        let d: [Vector2<f64>; 3] = [Vector2::new(dx[0], dx[1]), Vector2::new(dx[2], dx[3]), Vector2::new(dx[4], dx[5])];
        let len = [(&a, d[0]), (&b, d[1]), (&c, d[2])].iter().map(|(o, s)| (o.du * s.x + o.dv * s.y).norm()).fold(0.0, f64::max);
        let g = damp(len, (2.0 * k[0].abs()).max(tol::REL * sc));
        x = [step(f[0], &a, x[0], d[0], g), step(f[1], &b, x[1], d[1], g), step(f[2], &c, x[2], d[2], g)];
    }
    None
}

/// A boundary curve of a face: its 3D curve on [t0, t1] and its p-curve on the face.
pub struct Boundary<'a> {
    pub curve: &'a Curve,
    pub pc: &'a Curve,
    pub t: [f64; 2],
}

/// Where the rail on face `f[side]` (the ball touching `f[0]` and `f[1]`, offsets `k`) crosses
/// boundary curve `bd` of that face: each crossing's parameter on the boundary and the ball
/// there. Found from the sign changes of the other offset's signed distance along the boundary
/// (sampled), each refined by Newton on (t, u, v of the other face) as SMLib's
/// `SmFilletSolver::RailEdgeuseIntersect` (SmFilletSolver.cpp) does.
pub fn rail_crossings(f: [&Face; 2], k: [f64; 2], side: usize, bd: &Boundary, near: &P3, reach: f64) -> Vec<(f64, Ball)> {
    let other = 1 - side;
    let (fs, fo) = (f[side], f[other]);
    // The ball centre the contact at boundary parameter t would have, and its derivative.
    let centre = |t: f64| -> Option<(P3, V3, Contact)> {
        let (q, dp, _) = bd.pc.d2(t);
        let o = offset(fs, k[side], P2::new(q.x, q.y))?;
        // The contact moves along the p-curve; the centre with it (chain rule through uv).
        Some((o.o, o.du * dp.x + o.dv * dp.y, o.contact))
    };
    let sd = |c: &P3, hint: &mut Option<P2>| -> (f64, P2) {
        let uv = foot(fo, c, *hint);
        *hint = Some(uv);
        let q = contact(fo, uv);
        ((c - q.p).dot(&q.n) - k[other], uv)
    };
    let n = 48;
    let (t0, t1) = (bd.t[0], bd.t[1]);
    let mut hint: Option<P2> = None;
    let mut prev: Option<(f64, f64)> = None;
    let mut brackets: Vec<(f64, f64)> = Vec::new();
    for i in 0..=n {
        let t = t0 + (t1 - t0) * i as f64 / n as f64;
        let Some((c, _, _)) = centre(t) else {
            prev = None;
            continue;
        };
        // Only centres near the edge's balls count (an offset surface folds far away).
        if (c - near).norm() > reach {
            prev = None;
            continue;
        }
        let (g, _) = sd(&c, &mut hint);
        if let Some((tp, gp)) = prev
            && (g == 0.0 || (gp < 0.0) != (g < 0.0)) {
                brackets.push((tp, t));
            }
        prev = Some((t, g));
    }
    let mut out: Vec<(f64, Ball)> = Vec::new();
    for (a, b) in brackets {
        // Bisection on the signed distance to near the root, then Newton on the full system.
        let (mut lo, mut hi) = (a, b);
        let mut h2: Option<P2> = None;
        let g_at = |t: f64, h: &mut Option<P2>| centre(t).map(|(c, _, _)| sd(&c, h).0);
        let (Some(glo), Some(_)) = (g_at(lo, &mut h2), g_at(hi, &mut h2)) else { continue };
        let slo = glo < 0.0;
        for _ in 0..30 {
            let m = 0.5 * (lo + hi);
            let Some(gm) = g_at(m, &mut h2) else { break };
            if (gm < 0.0) == slo { lo = m } else { hi = m }
        }
        let mut t = 0.5 * (lo + hi);
        let Some((c0, _, _)) = centre(t) else { continue };
        let mut uvo = foot(fo, &c0, h2);
        let mut ok = None;
        let sc = scale(&c0, k[0].abs());
        for _ in 0..40 {
            let (Some((c, dc, cs)), Some(o)) = (centre(t), offset(fo, k[other], uvo)) else { break };
            let r = c - o.o;
            if r.norm() < 1e-11 * sc {
                let q = if side == 0 { [cs, o.contact] } else { [o.contact, cs] };
                ok = Some((t, Ball { c: P3::from((c.coords + o.o.coords) * 0.5), q, flat: false }));
                break;
            }
            let m = Matrix3::from_columns(&[dc, -o.du, -o.dv]);
            let Some(dx) = m.lu().solve(&(-r)) else { break };
            t = (t + dx[0]).clamp(t0.min(t1), t0.max(t1));
            uvo = clamp_uv(fo, uvo + Vector2::new(dx[1], dx[2]));
        }
        if let Some(x) = ok
            && !out.iter().any(|(t2, _)| (t2 - x.0).abs() < tol::REL * (1.0 + x.0.abs())) {
                out.push(x);
            }
    }
    out
}

/// An edge's curve as the spine sections are taken square to, run on past its ends along its
/// end tangents where the curve itself doesn't extend (B-splines, ellipses).
#[derive(Debug, Clone)]
pub struct Spine {
    pub curve: Curve,
    pub t: [f64; 2],
    /// A closed curve: parameters past its range wrap round.
    pub closed: bool,
}

impl Spine {
    /// The parameter on the curve itself (a closed B-spline's wrapped into its range).
    fn on_curve(&self, t: f64) -> f64 {
        let natural = matches!(self.curve, Curve::Line { .. } | Curve::Circle { .. });
        if self.closed && !natural {
            let (lo, hi) = (self.t[0].min(self.t[1]), self.t[0].max(self.t[1]));
            lo + wrap(t - lo, 0.0, hi - lo)
        } else {
            t
        }
    }

    /// The point and unit tangent at parameter t.
    pub fn at(&self, t: f64) -> (P3, V3) {
        let t = self.on_curve(t);
        let natural = matches!(self.curve, Curve::Line { .. } | Curve::Circle { .. });
        let (lo, hi) = (self.t[0].min(self.t[1]), self.t[0].max(self.t[1]));
        if natural || (t >= lo && t <= hi) {
            let (p, d, _) = self.curve.d2(t);
            return (p, d.normalize());
        }
        let e = if t < lo { lo } else { hi };
        let (p, d, _) = self.curve.d2(e);
        (p + d * (t - e), d.normalize())
    }

    /// The parameter whose section plane holds `c`, starting from `t`.
    pub fn param_of(&self, c: &P3, t: f64) -> f64 {
        // Closed forms: a line's projection, a circle's angle about its axis (near t).
        match &self.curve {
            Curve::Line { o, d } => return (c - o).dot(d) / d.norm_squared(),
            Curve::Circle { c: ctr, x, y, .. } => {
                let v = c - ctr;
                return unwrap_near(v.dot(y).atan2(v.dot(x)), t, TAU);
            }
            _ => {}
        }
        // Start from the guess or, if a sample of the curve lies nearer the point, from that
        // (a curve folding back near the point has other sections through it).
        let (lo, hi) = (self.t[0].min(self.t[1]), self.t[0].max(self.t[1]));
        let mut t = t;
        let mut best = (c - self.at(t).0).norm();
        for i in 0..=64 {
            let s = lo + (hi - lo) * i as f64 / 64.0;
            let d = (c - self.at(s).0).norm();
            if d < 0.5 * best {
                (t, best) = (s, d);
            }
        }
        for _ in 0..50 {
            let (p, d) = self.at(t);
            let (_, d1, d2) = self.curve.d2(self.on_curve(t));
            let speed = d1.norm().max(tol::TINY);
            let g = (c - p).dot(&d);
            // d/dt of (c − p)·T ≈ −|p'| + (c − p)·T' (T' = curvature normal · speed).
            let tp = (d2 - d * d2.dot(&d)) / speed;
            let dg = -speed + (c - p).dot(&tp);
            if dg.abs() < tol::TINY {
                break;
            }
            let dt = -g / dg;
            t += dt;
            if dt.abs() < 1e-14 * (1.0 + t.abs()) {
                break;
            }
        }
        t
    }
}

/// The exact fillet surfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Exact {
    /// Along a straight edge between surfaces unchanged along it.
    Cylinder,
    /// Along a circle between surfaces of revolution about its axis.
    Torus,
}

/// Whether a fillet between `f` along `spine` has a closed form.
pub fn exact_kind(f: [&Face; 2], spine: &Spine) -> Option<Exact> {
    match &spine.curve {
        Curve::Line { d, .. } => {
            let d = d.normalize();
            let along = |s: &Surface| match s {
                Surface::Plane { f } => f.x.cross(&f.y).normalize().dot(&d).abs() < 1e-10,
                Surface::Cylinder { f, .. } => f.z.cross(&d).norm() < 1e-10,
                Surface::Extrusion { d: e, .. } => e.normalize().cross(&d).norm() < 1e-10,
                _ => false,
            };
            (along(&f[0].surf) && along(&f[1].surf)).then_some(Exact::Cylinder)
        }
        Curve::Circle { c, x, y, .. } => {
            let z = x.cross(y).normalize();
            let coaxial = |s: &Surface| match s {
                Surface::Plane { f } => f.x.cross(&f.y).cross(&z).norm() < tol::ANGULAR,
                _ => s.axis().is_some_and(|(o, a)| a.cross(&z).norm() < tol::ANGULAR && (o - c).cross(&z).norm() < 1e-7),
            };
            (coaxial(&f[0].surf) && coaxial(&f[1].surf)).then_some(Exact::Torus)
        }
        _ => None,
    }
}

/// A fillet surface between two faces with its rails (the contact curves on the first and
/// second face) and how a ball maps to the rails' parameters.
#[derive(Debug, Clone)]
pub struct Strip {
    pub surf: Surface,
    pub rails: [Curve; 2],
    pub kind: StripKind,
}

#[derive(Debug, Clone)]
pub enum StripKind {
    Exact(Exact),
    /// u is the spine's parameter, v ∈ [0, 1] across (0 on the first face).
    Nurbs,
}

impl Strip {
    /// The parameter on rail `k` of the contact of a ball at spine parameter `t`.
    pub fn rail_param(&self, k: usize, p: &P3, t: f64, near: f64) -> f64 {
        match (&self.kind, &self.rails[k]) {
            (StripKind::Nurbs, _) => t,
            (_, Curve::Line { o, d }) => (p - o).dot(d) / d.norm_squared(),
            (_, Curve::Circle { c, x, y, .. }) => {
                let v = p - c;
                unwrap_near(v.dot(y).atan2(v.dot(x)), near, TAU)
            }
            (_, rail) => rail.closest(p, near - PI, near + PI),
        }
    }
}

/// The exact strip through ball `b` (all its balls are this one moved along the spine).
pub fn exact_strip(kind: Exact, b: &Ball, spine: &Spine) -> Option<Strip> {
    let r = (b.q[0].p - b.c).norm();
    if b.flat {
        return exact_flat(kind, b, spine);
    }
    match (kind, &spine.curve) {
        (Exact::Cylinder, Curve::Line { d, .. }) => {
            let d = d.normalize();
            let x = (b.q[0].p - b.c).normalize();
            let surf = Surface::Cylinder { f: Frame { o: b.c, x, y: d.cross(&x), z: d }, r };
            let rails = [Curve::Line { o: b.q[0].p, d }, Curve::Line { o: b.q[1].p, d }];
            Some(Strip { surf, rails, kind: StripKind::Exact(kind) })
        }
        (Exact::Torus, Curve::Circle { c, x, y, .. }) => {
            let z = x.cross(y).normalize();
            let h = (b.c - c).dot(&z);
            let axis_pt = c + z * h;
            let rho = b.c - axis_pt;
            let rmaj = rho.norm();
            if rmaj <= r * (1.0 + tol::REL) {
                return None;
            }
            // The torus's frame and rails share the spine's x and y so their angles agree.
            let surf = Surface::Torus { f: Frame { o: axis_pt, x: *x, y: z.cross(x), z }, rmaj, rmin: r };
            let rail = |q: &P3| {
                let hq = (q - c).dot(&z);
                let cq = c + z * hq;
                let rr = (q - cq).norm();
                Curve::Circle { c: cq, x: *x, y: z.cross(x), r: rr }
            };
            Some(Strip { surf, rails: [rail(&b.q[0].p), rail(&b.q[1].p)], kind: StripKind::Exact(kind) })
        }
        _ => None,
    }
}

/// The exact chamfer strips: a plane along a straight edge between surfaces unchanged along
/// it; along a circle between surfaces of revolution about its axis, the cone (or cylinder, or
/// plane) through the two rail circles.
fn exact_flat(kind: Exact, b: &Ball, spine: &Spine) -> Option<Strip> {
    let (q0, q1) = (b.q[0].p, b.q[1].p);
    match (kind, &spine.curve) {
        (Exact::Cylinder, Curve::Line { d, .. }) => {
            let d = d.normalize();
            let x = (q1 - q0).normalize();
            let surf = Surface::Plane { f: Frame { o: q0, x, y: d, z: x.cross(&d) } };
            let rails = [Curve::Line { o: q0, d }, Curve::Line { o: q1, d }];
            Some(Strip { surf, rails, kind: StripKind::Exact(kind) })
        }
        (Exact::Torus, Curve::Circle { c, x, y, .. }) => {
            let z = x.cross(y).normalize();
            let yy = z.cross(x);
            let ring = |q: &P3| -> (f64, f64) {
                let h = (q - c).dot(&z);
                (h, (q - (c + z * h)).norm())
            };
            let ((h0, r0), (h1, r1)) = (ring(&q0), ring(&q1));
            let rail = |h: f64, rr: f64| Curve::Circle { c: c + z * h, x: *x, y: yy, r: rr };
            let rails = [rail(h0, r0), rail(h1, r1)];
            let tol = 1e-12 * (1.0 + r0.max(r1));
            let surf = if (h1 - h0).abs() < tol {
                Surface::Plane { f: Frame { o: c + z * h0, x: *x, y: yy, z } }
            } else if (r1 - r0).abs() < tol {
                Surface::Cylinder { f: Frame { o: c + z * h0, x: *x, y: yy, z }, r: r0 }
            } else {
                let a = (r1 - r0).atan2(h1 - h0);
                Surface::Cone { f: Frame { o: c + z * h0, x: *x, y: yy, z }, r: r0, a }
            };
            Some(Strip { surf, rails, kind: StripKind::Exact(kind) })
        }
        _ => None,
    }
}

/// The rational quadratic arc of a ball's section, as three homogeneous control points.
fn section_cps(b: &Ball) -> Option<[H; 3]> {
    if b.flat {
        let mid = P3::from((b.q[0].p.coords + b.q[1].p.coords) * 0.5);
        return Some([hom(&b.q[0].p, 1.0), hom(&mid, 1.0), hom(&b.q[1].p, 1.0)]);
    }
    let a = b.q[0].p - b.c;
    let e = b.q[1].p - b.c;
    let r = 0.5 * (a.norm() + e.norm());
    // A ball of no size (a radius law reaching 0): the section is a point.
    if r < 1e-12 * (1.0 + b.c.coords.abs().max()) {
        return Some([hom(&b.c, 1.0); 3]);
    }
    let half = 0.5 * a.normalize().dot(&e.normalize()).clamp(-1.0, 1.0).acos();
    // Arcs near a half turn have their middle control point far out: no single segment.
    if half > 85f64.to_radians() {
        return None;
    }
    let w = half.cos();
    let m = (a.normalize() + e.normalize()).normalize();
    let mid = b.c + m * (r / w);
    Some([hom(&b.q[0].p, 1.0), hom(&mid, w), hom(&b.q[1].p, 1.0)])
}

/// Interpolates homogeneous points at increasing parameters (degree 3, or less for few
/// points): each coordinate as a curve, all on the same knots.
fn interp_h(hs: &[H], params: &[f64], closed: bool) -> Option<(Vec<f64>, Vec<H>, usize)> {
    let xyz: Vec<P3> = hs.iter().map(|h| P3::new(h.x, h.y, h.z)).collect();
    let ww: Vec<P3> = hs.iter().map(|h| P3::new(h.w, 0.0, 0.0)).collect();
    let p = 3.min(hs.len() - 1);
    let (a, b) = if closed {
        // The last point repeats the first: periodic over the m distinct ones.
        let m = hs.len() - 1;
        (NurbsCurve::interpolate_periodic(&xyz[..m], params, p)?, NurbsCurve::interpolate_periodic(&ww[..m], params, p)?)
    } else {
        (NurbsCurve::interpolate(&xyz, params, p, None, None), NurbsCurve::interpolate(&ww, params, p, None, None))
    };
    if a.knots != b.knots || a.cps.len() != b.cps.len() {
        return None;
    }
    let cps: Vec<H> = a.cps.iter().zip(&b.cps).map(|(x, w)| H::new(x.x / x.w, x.y / x.w, x.z / x.w, w.x / w.w)).collect();
    if cps.iter().any(|c| !(c.w > 0.0)) {
        return None;
    }
    Some((a.knots, cps, a.degree))
}

/// The B-spline fillet through the sections of `balls` (at increasing spine parameters `ts`;
/// for a closed strip the last repeats the first a period on). At the stations `kinks` (indices
/// into `ts`) it is only continuous: the faces' curvature jumps there (a closed strip with
/// kinks is then only continuous where it closes too).
pub fn nurbs_strip(balls: &[Ball], ts: &[f64], closed: bool, kinks: &[usize]) -> Option<Strip> {
    if balls.len() < 2 || balls.len() != ts.len() {
        return None;
    }
    let secs: Vec<[H; 3]> = balls.iter().map(section_cps).collect::<Option<_>>()?;
    // The pieces between kinks, each interpolated on its own, joined end to end.
    let mut cuts: Vec<usize> = vec![0];
    cuts.extend(kinks.iter().copied().filter(|&k| k > 0 && k + 1 < ts.len()));
    cuts.push(ts.len() - 1);
    cuts.dedup();
    let mut rows3: [Option<NurbsCurve>; 3] = [None, None, None];
    for w in cuts.windows(2) {
        let (a, z) = (w[0], w[1]);
        for (j, row) in rows3.iter_mut().enumerate() {
            let col: Vec<H> = secs[a..=z].iter().map(|s| s[j]).collect();
            let (k, cps, d) = interp_h(&col, &ts[a..=z], closed && cuts.len() == 2)?;
            let mut c = NurbsCurve { degree: d, knots: k, cps };
            // A short piece interpolates with a lower degree: raise it to the others'.
            while c.degree < 3 && cuts.len() > 2 {
                c = c.elevated();
            }
            *row = Some(match row.take() {
                None => c,
                Some(prev) if prev.degree == c.degree => prev.joined(&c),
                Some(_) => return None,
            });
        }
    }
    let [Some(r0), Some(r1), Some(r2)] = rows3 else { return None };
    if r0.knots != r1.knots || r0.knots != r2.knots {
        return None;
    }
    let (du, ku) = (r0.degree, r0.knots.clone());
    let rows: Vec<Vec<H>> = (0..r0.cps.len()).map(|i| vec![r0.cps[i], r1.cps[i], r2.cps[i]]).collect();
    let rail = |c: &NurbsCurve| -> Option<Curve> { Some(Curve::nurbs(NurbsCurve::new(c.degree, c.knots.clone(), c.cps.clone()).ok()?)) };
    let rails = [rail(&r0)?, rail(&r2)?];
    let surf = NurbsSurface::new(du, 2, ku, vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0], rows).ok()?;
    Some(Strip { surf: Surface::Nurbs(std::sync::Arc::new(surf)), rails, kind: StripKind::Nurbs })
}

/// How far the B-spline strip strays from the true sections `truth` at the parameters `ts`
/// (between its stations): the largest distance of its rails and its middle line.
pub fn strip_error(s: &Strip, truth: &[(f64, Ball)]) -> Vec<f64> {
    let Surface::Nurbs(n) = &s.surf else { return vec![0.0; truth.len()] };
    truth
        .iter()
        .map(|(t, b)| {
            let mut e: f64 = 0.0;
            for (v, want) in [(0.0, b.q[0].p), (1.0, b.q[1].p), (0.5, b.arc_point(0.5))] {
                e = e.max((n.point(*t, v) - want).norm());
            }
            e
        })
        .collect()
}
