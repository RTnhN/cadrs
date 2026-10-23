"""SMLib's surface unit tests (tests/prog_test/src/smSurfaceTest.cpp, run by the `unit` suite's
`my_unit_test_suite`, smAdvancedTest.cpp:4914) on bare B-spline and analytic surfaces:

- `my_test_sp_nurb` (smSurfaceTest.cpp:4042): every point of a 33 × 33 grid of the 9 × 7 cubic
  surface (`my_create_crease_surf`, no creases) found on it again (`my_test_surface_point_intersect`),
  on its natural domain and on [0.1, 0.9]², moved and not; the nearest point of the surface to
  each of 200 points over its box (`my_test_surface_point_extrema` with SM_SO_MINIMIZE); and the
  15 points SMLib finds within 0.03 of the thinly closed `Surface.sms`
  (`my_test_surface_closest_point`). Here the surface is made a sheet face
  (`NativeKernel::face_from_bare_surface`) and the distances are `Kernel::min_distance` from the
  point to the face: within 1e-5 for points on it, no farther than SMLib's nearest (the oracle's
  `ptsolve` and `closest`), and within 0.03 for the 15.
- `my_test_section` (smSurfaceTest.cpp:4429): planes through k points of the surface's box
  diagonal (`my_test_planar_section`) across two 5 × 4 cubic surfaces and `shovel.srf`; and
  `my_test_projection` (smSurfaceTest.cpp:4332): circles about the middle of the box's top
  projected down onto them (`my_test_parallel_projection`). Here the plane, or the vertical
  cylinder through the circle, is intersected with the surface (`ssi::surface_surface`, through
  tests/smlib_common/intersect.rs): every point on both, and per plane or circle the total length
  and SMLib's sample points (8 per curve) against SMLib's own curves (the oracle's `psection` and
  `pproject`), none against some where SMLib's curves have length (SMLib also returns touching
  points as curves of no length: the planes through the box's corners).
- `my_test_drop_curve_near_seam` (smSurfaceTest.cpp:1950): lines along a cylinder's seam (just
  before, on, just past it) and a circle from it, their p-curves lifted back onto them.
- `my_test_analytic_surfaces` (smSurfaceTest.cpp:5298) and the live block of `my_test_sp_analy`
  (smSurfaceTest.cpp:4165, a surface of revolution of a cubic B-spline about a skew axis): an
  11 × 11 grid (101 × 101 for the revolution) of points of each surface found again
  (`Surface::project`) to within 1e-6, as SMLib's `GlobalPointSolveSTEP` round trip.
"""
import math
import sys

from smlib_to_rust import Case, Fixture, rust_f64
from smlib._oracle import oracle_run, points_literal

SOURCE = "smSurfaceTest.cpp"
TITLE = "surface"

NOT_CONVERTED = [
    ("my_test_sp_nurb: the SM_SO_NORMALIZE counts (156, 86)", "every foot of the normal from a point (all critical points of the distance): the kernel finds the nearest and farthest points only"),
    ("my_test_analytic_surfaces: the surface of revolution of a parabola", "the kernel has no parabola curve"),
    ("my_test_analytic_surfaces: `LiftCurve` of each surface's uv diagonal", "lifting a uv curve to a 3D B-spline within a tolerance is not a kernel operation"),
    ("my_test_drop_curve_near_seam: the helix (kind 4) and the swapped cylinder (swap 1)", "the kernel has no helix curve; its cylinder has one parametrisation (the swapped one is the same surface)"),
    ("my_test_sp_analy after its first block", "dead: the function returns after it (smSurfaceTest.cpp:4250)"),
    ("my_test_csi_analy, my_test_csi_tangency, my_test_csi_tol", "in the cci suite (tools/smlib/cci.py)"),
    ("my_test_csi_coincidence, my_test_cs_nurb, my_test_ss_nurb", "overlap ranges of a curve on a surface, and local extrema of curve–surface and surface–surface distance: not kernel operations"),
    ("my_test_drop_curve, my_test_bsplinesurface(_nlib), my_test_surf_methods, my_test_surface_interface", "DropCurve/LiftCurve with trimming, knot and degree editing, Gordon and swung surfaces, approximation: not kernel operations"),
]

PRELUDE = r'''
#[path = "smlib_common/intersect.rs"]
mod intersect;
use intersect::*;

use std::sync::Arc;

use cadrs_kernel::native::curve::Curve;
use cadrs_kernel::native::math::{Affine, Frame, P3, V3};
use cadrs_kernel::native::nurbs::{hom, NurbsCurve, NurbsSurface};
use cadrs_kernel::native::pcurve;
use cadrs_kernel::native::smb;
use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::{DistanceTarget, FaceId, Kernel};

const SUITE: &str = "surfaces";

/// SMLib's `SM_ZONE_TOL_3D`.
const TOL: f64 = 1e-5;

type UvBox = ((f64, f64), (f64, f64));

fn p3(a: [f64; 3]) -> P3 {
    P3::new(a[0], a[1], a[2])
}

/// Clamped uniform knots of `n` control points of degree `d`, the inside ones at i/(n − d).
fn knots(n: usize, d: usize) -> Vec<f64> {
    let nk = n - d + 1;
    let mut k = vec![0.0; d + 1];
    k.extend((1..nk - 1).map(|i| i as f64 / (nk - 1) as f64));
    k.extend(vec![1.0; d + 1]);
    k
}

/// `my_create_nurb_surf` (and `my_create_crease_surf` without creases): nu × nv points, v
/// fastest, moved by `tr`.
fn grid(tr: [f64; 3], nu: usize, nv: usize, du: usize, dv: usize, pts: &[[f64; 3]]) -> Surface {
    let cps = (0..nu).map(|i| (0..nv).map(|j| hom(&(p3(pts[i * nv + j]) + p3(tr).coords), 1.0)).collect()).collect();
    Surface::Nurbs(Arc::new(NurbsSurface::new(du, dv, knots(nu, du), knots(nv, dv), cps).expect("a valid B-spline")))
}

/// `my_create_nurb`: a curve's points moved by `tr`, internal knots at i/(n − p + 1).
fn nurb(tr: [f64; 3], pts: &[[f64; 3]], p: usize) -> Curve {
    let nk = pts.len() - p + 1;
    let mut k = vec![0.0; p + 1];
    k.extend((1..nk - 1).map(|i| i as f64 / nk as f64));
    k.extend(vec![1.0; p + 1]);
    let cps = pts.iter().map(|q| hom(&(p3(*q) + p3(tr).coords), 1.0)).collect();
    Curve::nurbs(NurbsCurve::new(p, k, cps).expect("a valid B-spline"))
}

/// A surface file (`my_surface_from_file`).
fn surface_file(file: &str) -> Result<Surface, String> {
    let bytes = std::fs::read(data(SUITE, file)).map_err(|e| format!("{file}: {e}"))?;
    smb::read_surface(&bytes)
}

/// The fractions (a, b) to (c, d) of the surface's [0, 1]² domain.
fn frac(f: [f64; 4]) -> UvBox {
    ((f[0], f[2]), (f[1], f[3]))
}

/// `SmAxis2Placement::SetCanonical(o, +x, +z)` applied (`Transform`): world x, y, z to x, z, −y.
fn placed(s: &Surface, o: [f64; 3]) -> Surface {
    let m = nalgebra::Matrix3::from_columns(&[V3::x(), V3::z(), -V3::y()]);
    s.transformed(&Affine { m, t: p3(o).coords })
}

/// The surface over `dom` as a sheet face in a kernel.
fn sheet(s: &Surface, dom: UvBox) -> Result<(NativeKernel, DistanceTarget), String> {
    let mut k = NativeKernel::new();
    let id = k.face_from_bare_surface(s, Some(dom)).map_err(|e| format!("{e:?}"))?;
    Ok((k, DistanceTarget::Face(id, FaceId(0))))
}

/// The distance from `p` to the face, and the nearest point.
fn nearest(k: &NativeKernel, f: &DistanceTarget, p: P3) -> Result<(f64, P3), String> {
    let r = k.min_distance(&DistanceTarget::Point(p), f).map_err(|e| format!("{e:?}"))?;
    Ok((r.distance, r.b))
}

/// `my_test_surface_point_intersect`: the points of an n × n grid over `dom` found on the
/// surface over `dom` again (within TOL).
fn point_intersect(s: &Surface, dom: UvBox, n: usize) -> Result<(), String> {
    let (k, f) = sheet(s, dom)?;
    let ((u0, u1), (v0, v1)) = dom;
    for i in 0..n {
        for j in 0..n {
            let (u, v) = (u0 + (u1 - u0) * i as f64 / (n - 1) as f64, v0 + (v1 - v0) * j as f64 / (n - 1) as f64);
            let p = s.eval(u, v);
            let (d, _) = nearest(&k, &f, p)?;
            if d > TOL {
                return Err(format!("the point at ({u}, {v}) is found {d:.1e} away"));
            }
        }
    }
    Ok(())
}

/// `my_test_surface_point_extrema` with SM_SO_MINIMIZE: from each target (x, y, z, SMLib's
/// nearest distance) the nearest point of the surface over `dom` is no farther than SMLib's.
fn minimize(s: &Surface, dom: UvBox, targets: &[[f64; 4]]) -> Result<(), String> {
    let (k, f) = sheet(s, dom)?;
    for t in targets {
        let p = P3::new(t[0], t[1], t[2]);
        let (d, q) = nearest(&k, &f, p)?;
        if d > t[3] + 1e-6 {
            return Err(format!("from {p:?}: nearest {d:.9} at {q:?}, SMLib's {:.9}", t[3]));
        }
    }
    Ok(())
}

/// `my_test_surface_closest_point` on `file`: each target (x, y, z, SMLib's distance) within
/// 0.03 of the surface, and no farther than SMLib finds.
fn closest(file: &str, targets: &[[f64; 4]]) -> Result<(), String> {
    let s = surface_file(file)?;
    let ((u0, u1), (v0, v1)) = s.param_box();
    let (k, f) = sheet(&s, ((u0, u1), (v0, v1)))?;
    for t in targets {
        let p = P3::new(t[0], t[1], t[2]);
        let (d, _) = nearest(&k, &f, p)?;
        if d > 0.03 || d > t[3] + 1e-6 {
            return Err(format!("{p:?} is {d:.9} from the surface, SMLib finds {:.9} (at most 0.03)", t[3]));
        }
    }
    Ok(())
}

/// SMLib's curves for one plane or circle: whether any has length, their total length (none
/// when some of SMLib's curves were left out), sample points of those with length.
struct Want3<'a> {
    any: bool,
    length: Option<f64>,
    samples: &'a [[f64; 3]],
}

/// The unit normal of `s` at its point nearest `p`.
fn normal_near(s: &Surface, p: &P3) -> V3 {
    let (u, v) = feet(s, p, None).into_iter().min_by(|a, b| (s.eval(a.0, a.1) - p).norm().total_cmp(&(s.eval(b.0, b.1) - p).norm())).unwrap_or((0.0, 0.0));
    s.normal(u, v)
}

/// Compares our curves with SMLib's: none against some, the total length, and SMLib's samples
/// near ours. SMLib's curves are approximations: a sample's distance from both surfaces (its
/// error e) moves the curve across by up to e / sin θ, θ the angle between the surfaces there,
/// which is allowed twice on top of the usual sample tolerance; the length is compared within
/// the relative tolerance, or 2 % when SMLib's samples are off by more than 1e-4.
fn compare(what: &str, found: &Found, w: &Want3, a: &Surface, b: &Surface) -> Result<(), String> {
    if found.pieces.is_empty() == w.any {
        return Err(format!("{what}: {} curves (length {:.6}), SMLib finds {}", found.pieces.len(), found.length, if w.any { "some" } else { "none" }));
    }
    let mut worst: f64 = 0.0;
    for (k, s) in w.samples.iter().enumerate() {
        let p = p3(*s);
        let err = dist(a, &p) + dist(b, &p);
        worst = worst.max(err);
        let sin = normal_near(a, &p).cross(&normal_near(b, &p)).norm().max(1e-2);
        let d = found.distance(&p);
        if d > SAMPLE_TOL + 2.0 * err / sin {
            return Err(format!("{what}: SMLib's sample {k} {p:?} is {d:.1e} from our curves (it is {err:.1e} off the surfaces)"));
        }
    }
    if let (true, Some(l)) = (w.any, w.length) {
        let tol = if worst > 1e-4 { 2e-2 } else { LEN_TOL };
        if (found.length - l).abs() > tol * l.max(1.0) {
            return Err(format!("{what}: total length {:.6}, SMLib gives {l:.6} (its samples up to {worst:.1e} off the surfaces)", found.length));
        }
    }
    Ok(())
}

/// The surface over `dom` against a big patch of `other`.
fn cut(s: &Surface, dom: UvBox, other: Patch) -> Result<Found, String> {
    intersect_patches(&[Patch::rect(s.clone(), dom)], &[other])
}

/// A frame with z along `n`.
fn frame(o: P3, n: V3) -> Frame {
    let z = n.normalize();
    let w = if z.x.abs() <= z.y.abs() && z.x.abs() <= z.z.abs() { V3::x() } else if z.y.abs() <= z.z.abs() { V3::y() } else { V3::z() };
    let x = (w - z * w.dot(&z)).normalize();
    Frame { o, x, y: z.cross(&x), z }
}

/// `my_test_planar_section`: the planes through `planes`' points square to `n`.
fn section(s: &Surface, dom: UvBox, n: [f64; 3], planes: &[([f64; 3], Want3)]) -> Result<(), String> {
    let size = s.patch_bbox(dom.0, dom.1).diagonal().max(1.0);
    for (i, (p, w)) in planes.iter().enumerate() {
        let f = frame(p3(*p), p3(n).coords);
        let found = cut(s, dom, Patch::rect(Surface::Plane { f }, ((-4.0 * size, 4.0 * size), (-4.0 * size, 4.0 * size))))?;
        compare(&format!("plane {i}"), &found, w, s, &Surface::Plane { f })?;
    }
    Ok(())
}

/// `my_test_parallel_projection`: circles about `centre` (radius, SMLib's curves) projected
/// down −z: the vertical cylinders through them cut with the surface.
fn projection(s: &Surface, dom: UvBox, centre: [f64; 3], circles: &[(f64, Want3)]) -> Result<(), String> {
    let bb = s.patch_bbox(dom.0, dom.1);
    let c = p3(centre);
    let depth = (c.z - bb.min.z) + 1.0;
    for (i, (r, w)) in circles.iter().enumerate() {
        let f = Frame { o: c, x: V3::x(), y: V3::y(), z: V3::z() };
        let cyl = Surface::Cylinder { f, r: *r };
        let found = cut(s, dom, Patch::rect(cyl.clone(), ((0.0, std::f64::consts::TAU), (-depth, 1.0))))?;
        compare(&format!("circle {i} (radius {r})"), &found, w, s, &cyl)?;
    }
    Ok(())
}

/// SMLib's `GlobalPointSolveSTEP` round trip: the points of an n × n grid over `dom` found
/// again (`Surface::project`) within 1e-6 (`SM_EFF_ZERO_SQRT`).
fn round_trip(s: &Surface, dom: UvBox, n: usize) -> Result<(), String> {
    let ((u0, u1), (v0, v1)) = dom;
    for i in 0..n {
        for j in 0..n {
            let (u, v) = (u0 + (u1 - u0) * i as f64 / (n - 1) as f64, v0 + (v1 - v0) * j as f64 / (n - 1) as f64);
            let p = s.eval(u, v);
            let (a, b) = s.project(&p, None);
            let d = (s.eval(a, b) - p).norm();
            if d > 1e-6 {
                return Err(format!("the point at ({u}, {v}) is found at ({a}, {b}), {d:.1e} away"));
            }
        }
    }
    Ok(())
}

/// A frame on (o, x, y) (`SmAxis2Placement::SetCanonical`).
fn axes(o: [f64; 3], x: [f64; 3], y: [f64; 3]) -> Frame {
    let (x, y) = (p3(x).coords.normalize(), p3(y).coords.normalize());
    Frame { o: p3(o), x, y, z: x.cross(&y) }
}

/// `my_test_drop_curve_near_seam` (smSurfaceTest.cpp:1950): a cylinder of radius 0.25 about z
/// (heights 0 to 20) and a curve on it (`kind` 0–2: the vertical line x = 0.25 from z = 1 to 18
/// ending (kind − 1)·2e-5 off its seam; 3: three quarters of a circle at z = 1), maybe reversed:
/// its p-curve (`pcurve::pcurve`, SMLib's `DropCurve`) lifts back onto the curve within 1.01e-5 at
/// the same parameter, at 5 points of each of 64 spans.
fn drop_on_cylinder(kind: usize, reverse: bool) -> Result<(), String> {
    let f = Frame { o: P3::origin(), x: V3::x(), y: V3::y(), z: V3::z() };
    let s = Surface::Cylinder { f, r: 0.25 };
    let (c, t0, t1) = if kind == 3 {
        let c = Curve::Circle { c: P3::new(0.0, 0.0, 1.0), x: V3::x(), y: V3::y(), r: 0.25 };
        if reverse { c.reversed(0.0, 1.5 * std::f64::consts::PI) } else { (c, 0.0, 1.5 * std::f64::consts::PI) }
    } else {
        let (a, b) = (P3::new(0.25, 0.0, 1.0), P3::new(0.25, (kind as f64 - 1.0) * 2.0e-5, 18.0));
        let (c, l) = if reverse { Curve::line(b, a) } else { Curve::line(a, b) };
        (c, 0.0, l)
    };
    let pc = pcurve::pcurve(&s, &c, t0, t1, None);
    let n = 64;
    for i in 0..n {
        for k in 0..=4 {
            let t = t0 + (t1 - t0) * (i as f64 + k as f64 / 4.0) / n as f64;
            let uv = pc.point(t);
            let d = (s.eval(uv.x, uv.y) - c.point(t)).norm();
            if d > 1.01e-5 {
                return Err(format!("at t = {t}: the p-curve's point is {d:.2e} from the curve"));
            }
        }
    }
    Ok(())
}

/// The 7-point cubic of `my_test_analytic_surfaces`' surfaces of revolution and extrusion.
const PROFILE: [[f64; 3]; 7] = [[0.0, 0.1, 0.0], [0.3, 0.5, 0.0], [0.5, 1.0, 0.2], [0.6, 0.3, 0.3], [1.0, 1.0, 0.0], [1.3, 0.3, 0.1], [1.5, 0.5, 0.0]];
'''

def flat(pts):
    return ":".join("%r:%r:%r" % tuple(float(c) for c in p) for p in pts)


def grid_spec(tr, nu, nv, du, dv, pts):
    return "grid:%r:%r:%r:%d:%d:%d:%d:%s" % (float(tr[0]), float(tr[1]), float(tr[2]), nu, nv, du, dv, flat(pts))


def pts_rust(pts):
    return "&[%s]" % ", ".join("[%s, %s, %s]" % tuple(rust_f64(c) for c in p) for p in pts)


def grid_rust(tr, nu, nv, du, dv, pts):
    return "grid([%s, %s, %s], %d, %d, %d, %d, %s)" % (rust_f64(tr[0]), rust_f64(tr[1]), rust_f64(tr[2]), nu, nv, du, dv, pts_rust(pts))


SURF3 = [(0,0,-0.6),(1,0.1,0.4),(2,0.2,0.3),(3,0.3,0),(3.3,0.0,-0.3),(3.7,0.0,-0.7),(4,0.0,-1),
         (0,1,0.2),(1,1,1.5),(2,1,1.6),(3,1,0.2),(3.3,1,-0.3),(3.7,1,-0.7),(4,1,-1),
         (0,2,0.3),(1,2,1.6),(2,2,1.5),(3,2,0.4),(3.3,2,-0.3),(3.7,2,-0.7),(4,2,-1),
         (0,3,0.4),(1,3,1.2),(2,3,1.1),(3,3,0.7),(3.3,3,-0.3),(3.7,3,-0.7),(4,3,-1),
         (0,4,0.4),(1,4,-1.2),(2,4,-1.1),(3,4,0.5),(3.3,4,-0.3),(3.7,4,-0.7),(4,4,-1),
         (0,5,0.4),(1,5,0.2),(2,5,0.1),(3,5,0.5),(3.3,5,-0.3),(3.7,5,-0.7),(4,5,-1),
         (0,5.3,-0.3),(1,5.3,-0.3),(2,5.3,-0.3),(3,5.3,-0.3),(3.3,5.3,-0.5),(3.7,5.3,-0.7),(4,5.3,-1),
         (0,5.7,-0.7),(1,5.7,-0.7),(2,5.7,-0.7),(3,5.7,-0.7),(3.3,5.7,-0.8),(3.7,5.7,-0.9),(4,5.7,-1.0),
         (0,6,-1),(1,6,-1),(2,6,-1),(3,6,-1),(3.3,6,-1),(3.7,6,-1),(4,6,-1)]

SEC_A = [(0,0,0),(1,0.1,0.4),(2,0.2,0.3),(3,0.3,0),(0,1,0.2),(1,1,1.5),(2,1,1.6),(3,1,0.2),(0,2,0.3),(1,2,1.6),
         (2,2,1.5),(3,2,0.4),(0,3,0.4),(1,3,1.2),(2,3,1.1),(3,3,0.7),(0,4,0.4),(1,4,0.2),(2,4,0.1),(3,4,0.5)]
PROJ_A = [(0,0,0),(1,0.1,0.4),(2,0.2,0.3),(3,0.3,0),(0,1,0.2),(1,1,1.5),(2,1,-1.6),(3,1,0.2),(0,2,0.3),(1,2,-1.6),
          (2,2,1.5),(3,2,0.4),(0,3,0.4),(1,3,1.2),(2,3,1.1),(3,3,-0.7),(0,4,0.4),(1,4,0.2),(2,4,-0.4),(3,4,0.5)]
B2 = [(0,0,0.1),(1,0.1,0.3),(2,0.2,0.4),(3,0.3,0),(0,1,0.3),(1,1,0.6),(2,1,-1.5),(3,1,-1.2),(0,2,0.4),(1,2,0.5),
      (2,2,-1.6),(3,2,-1.3),(0,3,0.7),(1,3,-1.1),(2,3,-1.2),(3,3,0.4),(0,4,0.7),(1,4,0.1),(2,4,0.2),(3,4,0.6)]

# The 15 points of my_test_sp_nurb's thinly closed surface (smSurfaceTest.cpp:4120).
CLOSEST = [(-1.0616429349692744,-20.167126109553802,20.020823739437560), (-1.1583434957685239,-19.953438722680168,20.005921768970701),
           (-4.5941574701649142,-9.6528635542995485,19.290949275641431), (-0.83908714000092477,-20.645564191951244,20.054070584952139),
           (-0.22169184902399575,-21.752883811327614,20.077084827504891), (0.097876793468328019,-21.633397717952313,18.978461556867217),
           (-0.25257593806641893,-20.10880763638518,19.018298541213202), (-0.32629818190650922,-19.760345509317247,19.026748453308592),
           (-0.66892154856885222,-17.879224383359258,19.072647185802911), (-0.83758573462226327,-20.648619822796935,20.054270506974774),
           (-0.22211826515506564,-21.75240325273451,20.077401022086111), (0.097878601273081345,-21.633405764037899,18.978461275445223),
           (-0.25280760311075551,-20.107797610675622,19.018327728721825), (-0.32651463102618838,-19.7592999915767,19.026773809915298),
           (-0.40219453853645376,-19.384417128948424,19.035877125510886)]


def quads(rows):
    return "&[%s]" % ", ".join("[%s]" % ", ".join(rust_f64(x) for x in r) for r in rows)


# SMLib curves with a sample farther than this off the plane or cylinder (its own stray pieces:
# shovel.srf's plane 12 has a 0.04 stub starting 2.6e-3 off the plane) aren't compared.
OFF = 1e-3


def want3(curves, off):
    """A Want3 literal from the oracle's curves of one plane or circle (`off`: a point's
    distance from the plane or cylinder), and the number of SMLib's curves left out."""
    good = [c for c in curves if max(off(q) for q in c["samples"] + [c["start"], c["end"]]) <= OFF]
    long_ = [c for c in good if c["length"] > 1e-9]
    samples = [s for c in long_ for s in c["samples"]]
    length = "Some(%s)" % rust_f64(sum(c["length"] for c in long_)) if len(good) == len(curves) else "None"
    return "Want3 { any: %s, length: %s, samples: %s }" % ("true" if long_ else "false", length, points_literal(samples)), len(curves) - len(good)


def need(r, what):
    """The oracle's report, or None without the oracle (those cases are left out, said on
    stderr); an oracle that can't run the command (built before it had it) stops generation."""
    if r is None:
        print("surfaces: no oracle (CADRS_SMLIB_ORACLE): %s left out" % what, file=sys.stderr)
        return None
    if not r.get("ok"):
        raise SystemExit("surfaces: the oracle fails on %s (%s): rebuild it from tools/smlib_oracle" % (what, r.get("status")))
    return r


def extract(ctx):
    src = ctx.source(SOURCE)
    for f in ("my_test_sp_nurb", "my_test_section", "my_test_projection", "my_test_analytic_surfaces", "my_test_sp_analy"):
        assert "SmStatus %s" % f in src, f
    cases = []
    crease = grid_rust((0, 0, 0), 9, 7, 3, 3, SURF3)
    crease8 = grid_rust((-8, 0, 0), 9, 7, 3, 3, SURF3)
    nat, trim, trim2 = "frac([0.0, 0.0, 1.0, 1.0])", "frac([0.1, 0.1, 0.9, 0.9])", "frac([0.1, 0.1, 0.82, 0.82])"
    for name, surf, dom, doc in [
        ("sp_nurb_point_intersect", crease, nat, "the 9 × 7 cubic surface over its natural domain"),
        ("sp_nurb_point_intersect_moved", "placed(&%s, [0.0, 7.0, -7.0])" % crease, nat, "the surface turned onto the xz plane and moved by (0, 7, −7)"),
        ("sp_nurb_point_intersect_trimmed", crease8, trim, "the surface (moved by (−8, 0, 0)) over [0.1, 0.9]² of its domain"),
        ("sp_nurb_point_intersect_trimmed_moved", "placed(&%s, [0.0, 7.0, -7.0])" % crease8, trim2, "that surface turned and moved, over [0.1, 0.82]² (SMLib's second domain)"),
    ]:
        cases.append(Case(name, "SMLib `my_test_sp_nurb`: every point of a 33 × 33 grid of %s found on it again." % doc,
                          "point_intersect(&%s, %s, 33)" % (surf, dom)))
    for name, tr, dom, frac_s, doc in [("sp_nurb_minimize", (0, 0, 0), nat, None, "its natural domain"),
                                       ("sp_nurb_minimize_trimmed", (-8, 0, 0), trim, "0.1,0.1,0.9,0.9", "[0.1, 0.9]² of its domain")]:
        args = ["ptsolve", grid_spec(tr, 9, 7, 3, 3, SURF3), "min", 10, 10, 2] + (["--dom=" + frac_s] if frac_s else [])
        r = need(oracle_run(args), name)
        if not r:
            continue
        rows = []
        for t in r["targets"]:
            if t["solutions"]:
                rows.append(list(t["target"]) + [min(s["distance"] for s in t["solutions"])])
        cases.append(Case(name, "SMLib `my_test_sp_nurb`: from %d points of a 10 × 10 × 2 grid over its box, the nearest point of the 9 × 7 cubic surface over %s no farther than SMLib's (%d solutions)." % (len(rows), doc, r["count"]),
                          "minimize(&%s, %s, %s)" % (grid_rust(tr, 9, 7, 3, 3, SURF3), dom, quads(rows))))
    sms = Fixture("TestFiles/pt_TestFiles/Surface.sms")
    r = need(oracle_run(["closest", ctx.path(sms.rel)] + [c for p in CLOSEST for c in p]), "sp_nurb_closest_thin_surface")
    if r:
        rows = [list(p) + [q.get("distance", 0.03)] for p, q in zip(CLOSEST, r["points"])]
        cases.append(Case("sp_nurb_closest_thin_surface", "SMLib `my_test_sp_nurb`: 15 points within 0.03 of the thinly closed `Surface.sms`, as close as SMLib finds them.",
                          'closest("Surface.sms", %s)' % quads(rows), [sms]))
    shovel = Fixture("TestFiles/pt_TestFiles/shovel.srf")
    sections = [
        ("section_shovel", shovel, None, (0, 0, 1), 40, None, "`shovel.srf`, 40 planes square to z"),
        ("section_cubic_z", None, ((0, -5, 0), SEC_A), (0, 0, 1), 10, 10, "a 5 × 4 cubic surface, 10 planes square to z"),
        ("section_cubic_xy", None, ((0, -5, 0), SEC_A), (1, 1, 0), 10, 10, "the same surface, 10 planes square to (1, 1, 0)"),
        ("section_cubic2_z", None, ((5, -5, 0), B2), (0, 0, 1), 20, 21, "a second 5 × 4 cubic surface, 20 planes square to z"),
        ("section_cubic2_xyz", None, ((5, -5, 0), B2), (1, 1, 1), 10, 7, "the second surface, 10 planes square to (1, 1, 1)"),
    ]
    for name, fx, g, n, k, asserted, doc in sections:
        spec = ctx.path(fx.rel) if fx else grid_spec(g[0], 5, 4, 3, 3, g[1])
        surf = 'surface_file("%s")?' % fx.name if fx else grid_rust(g[0], 5, 4, 3, 3, g[1])
        r = need(oracle_run(["psection", spec] + list(n) + [k]), name)
        if not r:
            continue
        nn = math.sqrt(sum(c * c for c in n))
        rows, stray = [], 0
        for pl in r["planes"]:
            off = lambda q, o=pl["point"]: abs(sum((q[i] - o[i]) * n[i] for i in range(3))) / nn
            w, k_ = want3(pl["curves"], off)
            stray += k_
            rows.append("([%s], %s)" % (", ".join(rust_f64(c) for c in pl["point"]), w))
        planes = ", ".join(rows)
        doc2 = "SMLib `my_test_section`: %s (SMLib: %d curves%s%s)." % (doc, r["count"], "" if asserted is None else ", its test asserts %d" % asserted,
                                                                        "" if not stray else "; %d of SMLib's, off the plane by more than %g, left out, with their plane's length" % (stray, OFF))
        call = "(|| -> Result<(), String> { let s = %s; let d = s.param_box(); section(&s, d, [%s], &[%s]) })()" % (
            surf, ", ".join(rust_f64(c) for c in n), planes)
        cases.append(Case(name, doc2, call, [fx] if fx else []))
    projections = [
        ("projection_shovel", shovel, None, 20, None, "`shovel.srf`, 20 circles"),
        ("projection_cubic", None, ((0, -5, 0), PROJ_A), 10, 15, "a 5 × 4 cubic surface, 10 circles"),
        ("projection_cubic2", None, ((5, 5, 0), B2), 20, 29, "a second 5 × 4 cubic surface, 20 circles"),
    ]
    for name, fx, g, k, asserted, doc in projections:
        spec = ctx.path(fx.rel) if fx else grid_spec(g[0], 5, 4, 3, 3, g[1])
        surf = 'surface_file("%s")?' % fx.name if fx else grid_rust(g[0], 5, 4, 3, 3, g[1])
        r = need(oracle_run(["pproject", spec, k]), name)
        if not r:
            continue
        centre = r["circles"][0]["centre"]
        rows, stray = [], 0
        for c in r["circles"]:
            off = lambda q, rr=c["radius"]: abs(math.hypot(q[0] - centre[0], q[1] - centre[1]) - rr)
            w, k_ = want3(c["curves"], off)
            stray += k_
            rows.append("(%s, %s)" % (rust_f64(c["radius"]), w))
        circles = ", ".join(rows)
        doc2 = "SMLib `my_test_projection`: %s about the middle of its box's top projected down (SMLib: %d curves%s%s)." % (doc, r["count"], "" if asserted is None else ", its test asserts %d" % asserted,
                                                                                                                        "" if not stray else "; %d of SMLib's, off the cylinder by more than %g, left out, with their circle's length" % (stray, OFF))
        call = "(|| -> Result<(), String> { let s = %s; let d = s.param_box(); projection(&s, d, [%s], &[%s]) })()" % (
            surf, ", ".join(rust_f64(c) for c in centre), circles)
        cases.append(Case(name, doc2, call, [fx] if fx else []))
    d = math.radians
    analytic = [
        ("plane", "Surface::Plane { f: axes([0.0; 3], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]) }", ((-1, 1), (-1, 1)), "a plane x = 0 over [−1, 1]²"),
        ("cylinder", "Surface::Cylinder { f: axes([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), r: 1.0 }", ((0, d(360)), (-1, 1)), "a unit cylinder about z through (−1, 0, 0), heights −1 to 1"),
        ("cone", "Surface::Cone { f: axes([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]), r: 1.0, a: std::f64::consts::FRAC_PI_4 }", ((0, d(360)), (-1, 1)), "a 45° cone of radius 1 about −z through (0, 1, 0)"),
        ("cone_apex", "Surface::Cone { f: axes([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), r: 0.0, a: std::f64::consts::FRAC_PI_4 }", ((0, d(360)), (0, 2)), "a 45° cone from its apex at the origin"),
        ("torus", "Surface::Torus { f: axes([2.0, 2.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), rmaj: 1.0, rmin: 0.25 }", ((0, d(360)), (0, d(360))), "a torus R 1, r 0.25"),
        ("sphere", "Surface::Sphere { f: axes([-2.0, 2.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), r: 1.0 }", ((0, d(90)), (d(-90), d(90))), "a quarter of a unit sphere, pole to pole"),
        ("revolution", "Surface::Revolution { f: axes([0.0; 3], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]), m: nurb([0.0; 3], &PROFILE, 3) }", ((0, d(180)), (0, 1)), "half the surface of revolution of a cubic about x"),
        ("extrusion", "Surface::Extrusion { c: nurb([0.0; 3], &[[0.0, 0.1, 0.0], [0.3, 0.5, 0.0], [0.5, 0.6, 0.0], [0.6, 0.3, 0.0], [1.0, 0.5, 0.0], [1.3, 0.3, 0.0], [1.5, 0.5, 0.0]], 3), d: V3::new(0.0, 1.0, 0.8) }", ((0, 1), (-1, 3)), "a planar cubic extruded along (0, 1, 0.8)"),
        ("extrusion_z", "Surface::Extrusion { c: nurb([0.0; 3], &PROFILE, 3), d: V3::z() }", ((0, 1), (-1, 3)), "a space cubic extruded along z"),
    ]
    for name, make, ((u0, u1), (v0, v1)), doc in analytic:
        cases.append(Case("analytic_" + name, "SMLib `my_test_analytic_surfaces`: an 11 × 11 grid of %s found again." % doc,
                          "round_trip(&%s, ((%s, %s), (%s, %s)), 11)" % (make, rust_f64(u0), rust_f64(u1), rust_f64(v0), rust_f64(v1))))
    assert "SmStatus my_test_drop_curve_near_seam()" in src
    for kind, what in [(0, "a vertical line ending 2e-5 before the seam"), (1, "a vertical line on the seam"), (2, "a vertical line ending 2e-5 past the seam"), (3, "three quarters of a circle from the seam")]:
        for rev in (False, True):
            cases.append(Case("drop_near_seam_%d%s" % (kind, "_reversed" if rev else ""), "SMLib `my_test_drop_curve_near_seam`: %s of a cylinder of radius 0.25%s, its p-curve lifted back within 1.01e-5." % (what, ", reversed" if rev else ""),
                              "drop_on_cylinder(%d, %s)" % (kind, "true" if rev else "false")))
    cases.append(Case("sp_analy_revolution", "SMLib `my_test_sp_analy`: a 101 × 101 grid of the full surface of revolution of a cubic about the axis through (0, 2, 0) along (1, 1, 0) found again.",
                      "round_trip(&Surface::Revolution { f: frame(P3::new(0.0, 2.0, 0.0), V3::new(1.0, 1.0, 0.0)), m: nurb([0.0; 3], &[[0.0, 0.1, 0.0], [0.3, 0.5, 0.0], [0.5, 1.0, 0.0], [0.6, 0.3, 0.0], [1.0, 1.0, 0.0], [1.3, 0.3, 0.0], [1.5, 0.5, 0.0]], 3) }, ((0.0, std::f64::consts::TAU), (0.0, 1.0)), 101)"))
    return cases
