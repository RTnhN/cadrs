//! B-splines that are exactly simple geometry, recognised so the exact algorithms (analytic
//! intersections, blend sections, offsets) apply to them: curves that are lines or circular
//! arcs, surfaces that are planes, cylinders, cones, spheres or tori. Files from systems that write
//! everything as NURBS (SMLib's, many STEP exporters') then model like native geometry.
//!
//! Each test samples the B-spline on a grid and accepts the candidate only when every sample
//! lies on it within `1e-7` of the shape's size (rational B-splines represent these shapes
//! exactly, so true matches agree to rounding).

use super::curve::Curve;
use super::math::*;
use super::nurbs::{NurbsCurve, NurbsSurface, dehom};
use super::surface::Surface;
use super::tol;

const REL: f64 = 1e-7;

/// The line or circle a B-spline curve is (over its whole range), if it is one.
pub fn curve(n: &NurbsCurve) -> Option<Curve> {
    let (a, b) = n.range();
    let m = 24.max(4 * n.cps.len());
    let pts: Vec<P3> = (0..=m)
        .map(|i| n.point(a + (b - a) * i as f64 / m as f64))
        .collect();
    let mut bb = Bbox::empty();
    for p in &pts {
        bb.add(p);
    }
    let size = bb.diagonal();
    if size < tol::DEGENERATE {
        return None;
    }
    let tol = size * REL;
    // A line: every sample on the chord's line.
    let (p0, p1) = (pts[0], pts[m]);
    if let Some(d) = (p1 - p0).try_normalize(tol::DEGENERATE)
        && pts
            .iter()
            .all(|p| ((p - p0) - d * (p - p0).dot(&d)).norm() < tol)
        && n.cps
            .iter()
            .all(|h| ((dehom(h) - p0) - d * (dehom(h) - p0).dot(&d)).norm() < tol * 10.0)
    {
        return Some(Curve::Line { o: p0, d });
    }
    // A circle: the circle through three samples holds all of them.
    if n.degree < 2 {
        return None;
    }
    let (c, nrm, r) = circle3(&pts[0], &pts[m / 3], &pts[2 * m / 3])?;
    if pts
        .iter()
        .any(|p| ((p - c).norm() - r).abs() > tol || (p - c).dot(&nrm).abs() > tol)
    {
        return None;
    }
    let x = (pts[0] - c) / r;
    // Turning from the first sample towards the next.
    let along = (pts[1] - pts[0]).dot(&nrm.cross(&x));
    let z = if along >= 0.0 { nrm } else { -nrm };
    Some(Curve::Circle {
        c,
        x,
        y: z.cross(&x),
        r,
    })
}

fn circle3(a: &P3, b: &P3, c: &P3) -> Option<(P3, V3, f64)> {
    let (u, v) = (b - a, c - a);
    let w = u.cross(&v);
    let ww = w.norm_squared();
    if ww < 1e-24 * (u.norm_squared() * v.norm_squared()).max(tol::TINY) {
        return None;
    }
    let cc = a + (v * u.norm_squared() - u * v.norm_squared()).cross(&w) * (0.5 / ww);
    Some((cc, w.normalize(), (a - cc).norm()))
}

/// A B-spline straight in one direction with every cross-section the first one moved by the
/// same vector: the extrusion of that curve (c(u) + d·v).
fn extrusion(n: &NurbsSurface) -> Option<Surface> {
    let rows = |a: &[Vec<super::nurbs::H>]| -> Option<V3> {
        // a[i] = [first, second] per pole: one translation for all, weights equal.
        let mut d: Option<V3> = None;
        for r in a {
            if r.len() != 2 || (r[0].w - r[1].w).abs() > tol::REL_FINE * r[0].w.abs().max(1.0) {
                return None;
            }
            let t = dehom(&r[1]) - dehom(&r[0]);
            match d {
                None => d = Some(t),
                Some(d0) if (t - d0).norm() <= tol::REL * (1.0 + d0.norm()) => {}
                _ => return None,
            }
        }
        d.filter(|d| d.norm() > tol::DEGENERATE)
    };
    if n.dv == 1 && n.cps[0].len() == 2 {
        let d = rows(&n.cps)?;
        let c = NurbsCurve {
            degree: n.du,
            knots: n.ku.clone(),
            cps: n.cps.iter().map(|r| r[0]).collect(),
        };
        return Some(Surface::Extrusion {
            c: Curve::Nurbs(std::sync::Arc::new(c)),
            d,
        });
    }
    if n.du == 1 && n.cps.len() == 2 {
        let cols: Vec<Vec<super::nurbs::H>> = (0..n.cps[0].len())
            .map(|j| vec![n.cps[0][j], n.cps[1][j]])
            .collect();
        let d = rows(&cols)?;
        let c = NurbsCurve {
            degree: n.dv,
            knots: n.kv.clone(),
            cps: n.cps[0].clone(),
        };
        return Some(Surface::Extrusion {
            c: Curve::Nurbs(std::sync::Arc::new(c)),
            d,
        });
    }
    None
}

/// The plane, cylinder, cone, sphere or torus a B-spline surface is, if it is one (checked:
/// every sample lies on the surface returned, by its own distance).
pub fn surface(n: &NurbsSurface) -> Option<Surface> {
    surface_with(n, false)
}

/// [`surface`], also when a side of the B-spline collapses to a point (a cone's apex, a
/// sphere's pole): the normals there are rounding noise and are left out, and the sections
/// taken where they have a radius. (The offsets take it; the readers don't yet, since an
/// analytic cone in place of its patch changes what the SSI finds across its seam.)
pub fn surface_through_poles(n: &NurbsSurface) -> Option<Surface> {
    surface_with(n, true)
}

fn surface_with(n: &NurbsSurface, poles: bool) -> Option<Surface> {
    let cand = candidate(n, poles)
        .or_else(|| torus(n))
        .or_else(|| extrusion(n))?;
    let ((u0, u1), (v0, v1)) = n.range();
    let mut bb = Bbox::empty();
    let m = 10;
    let mut pts = Vec::new();
    for i in 0..=m {
        for j in 0..=m {
            let p = n.point(
                u0 + (u1 - u0) * i as f64 / m as f64,
                v0 + (v1 - v0) * j as f64 / m as f64,
            );
            bb.add(&p);
            pts.push(p);
        }
    }
    let tol = bb.diagonal() * REL * 10.0;
    pts.iter().all(|p| cand.distance(p) < tol).then_some(cand)
}

/// A torus: the iso lines one way are circles about one axis (their normals along it, their
/// centres on it), and an iso line the other way is a circle in a plane through the axis (the
/// meridian), its centre off the axis.
fn torus(n: &NurbsSurface) -> Option<Surface> {
    let ((u0, u1), (v0, v1)) = n.range();
    let m = 12;
    let at = |i: usize, j: usize| {
        n.point(
            u0 + (u1 - u0) * i as f64 / m as f64,
            v0 + (v1 - v0) * j as f64 / m as f64,
        )
    };
    let mut bb = Bbox::empty();
    for i in 0..=m {
        for j in 0..=m {
            bb.add(&at(i, j));
        }
    }
    let tol = bb.diagonal() * REL * 10.0;
    // The circle through a line of samples, when they all lie on it.
    let circle = |ps: &[P3]| -> Option<(P3, V3, f64)> {
        let k = ps.len();
        let (c, nn, r) = circle3(&ps[0], &ps[k / 3], &ps[2 * k / 3])?;
        ps.iter()
            .all(|p| ((p - c).norm() - r).abs() < tol && (p - c).dot(&nn).abs() < tol)
            .then_some((c, nn, r))
    };
    // Rows along v at fixed u (`by_u`), or along u at fixed v.
    let row = |by_u: bool, k: usize| -> Vec<P3> {
        (0..=m)
            .map(|t| if by_u { at(k, t) } else { at(t, k) })
            .collect()
    };
    for by_u in [true, false] {
        let rings: Option<Vec<(P3, V3, f64)>> = [0, m / 2, m]
            .iter()
            .map(|&k| circle(&row(by_u, k)))
            .collect();
        let Some(rings) = rings else { continue };
        let (c0, a, _) = rings[0];
        let on_axis = |c: &P3| ((c - c0) - a * (c - c0).dot(&a)).norm() < tol;
        if !rings
            .iter()
            .all(|(c, nn, _)| nn.cross(&a).norm() < 1e-7 && on_axis(c))
        {
            continue;
        }
        // The meridian: a row the other way.
        let Some((mc, mn, rmin)) = circle(&row(!by_u, 0)) else {
            continue;
        };
        // In a plane through the axis: its normal square to the axis, its centre's plane
        // holding the axis.
        if mn.dot(&a).abs() > 1e-7 || (c0 - mc).dot(&mn).abs() > tol {
            continue;
        }
        let o = c0 + a * (mc - c0).dot(&a);
        let radial = mc - o;
        let rmaj = radial.norm();
        if rmaj < tol {
            continue;
        }
        // The seam (u = 0) towards the patch's first corner.
        let corner = at(0, 0);
        let x = ((corner - o) - a * (corner - o).dot(&a))
            .try_normalize(tol::DEGENERATE)
            .unwrap_or_else(|| radial / rmaj);
        return Some(Surface::Torus {
            f: Frame {
                o,
                x,
                y: a.cross(&x),
                z: a,
            },
            rmaj,
            rmin,
        });
    }
    None
}

fn candidate(n: &NurbsSurface, poles: bool) -> Option<Surface> {
    let ((u0, u1), (v0, v1)) = n.range();
    let m = 12;
    let mut pts = Vec::new();
    let mut nrms = Vec::new();
    for i in 0..=m {
        for j in 0..=m {
            let (u, v) = (
                u0 + (u1 - u0) * i as f64 / m as f64,
                v0 + (v1 - v0) * j as f64 / m as f64,
            );
            let d = n.ders2(u, v);
            pts.push(P3::from(d[0]));
            // (Not at a pole or apex, where a derivative vanishes and the cross product is
            // rounding noise.)
            let c = d[1].cross(&d[2]);
            if (!poles || c.norm() > tol::REL * d[1].norm_squared().max(d[2].norm_squared()))
                && let Some(nn) = c.try_normalize(tol::DEGENERATE)
            {
                nrms.push((P3::from(d[0]), nn));
            }
        }
    }
    let mut bb = Bbox::empty();
    for p in &pts {
        bb.add(p);
    }
    let size = bb.diagonal();
    if size < tol::DEGENERATE || nrms.len() < 4 {
        return None;
    }
    let tol = size * REL;
    let sample_u = n.ders2(0.5 * (u0 + u1), 0.5 * (v0 + v1));
    // A plane: all samples on the plane through one with its normal.
    let (o, n0) = nrms[nrms.len() / 2];
    if pts.iter().all(|p| (p - o).dot(&n0).abs() < tol) {
        let x = (sample_u[1] - n0 * sample_u[1].dot(&n0))
            .try_normalize(tol::DEGENERATE)
            .unwrap_or_else(|| any_perpendicular(&n0));
        return Some(Surface::Plane {
            f: Frame {
                o,
                x,
                y: n0.cross(&x),
                z: n0,
            },
        });
    }
    // A sphere: every normal line through one centre at one distance.
    let centre_of = |(p, nn): (P3, V3), (q, mm): (P3, V3)| -> Option<P3> {
        // Closest points of the two normal lines.
        let w = p - q;
        let (a, b, c, d, e) = (
            nn.dot(&nn),
            nn.dot(&mm),
            mm.dot(&mm),
            nn.dot(&w),
            mm.dot(&w),
        );
        let den = a * c - b * b;
        if den.abs() < tol::DEGENERATE {
            return None;
        }
        let s = (b * e - c * d) / den;
        Some(p + nn * s)
    };
    let k = nrms.len();
    if let Some(c) = centre_of(nrms[0], nrms[k / 2]) {
        let r = (pts[0] - c).norm();
        if r > tol && pts.iter().all(|p| ((p - c).norm() - r).abs() < tol) {
            let x = (pts[0] - c)
                .try_normalize(tol::DEGENERATE)
                .map(|d| {
                    (d - V3::z() * d.z)
                        .try_normalize(tol::ANGULAR)
                        .unwrap_or(V3::x())
                })
                .unwrap_or(V3::x());
            return Some(Surface::Sphere {
                f: Frame {
                    o: c,
                    x,
                    y: V3::z().cross(&x),
                    z: V3::z(),
                },
                r,
            });
        }
    }
    // A cylinder or cone: an axis direction making one angle with every normal (n·a = s).
    // The smallest eigenvector of Σ [n, −1][n, −1]ᵀ.
    let mut mtx = nalgebra::Matrix4::<f64>::zeros();
    for (_, nn) in &nrms {
        let row = nalgebra::Vector4::new(nn.x, nn.y, nn.z, -1.0);
        mtx += row * row.transpose();
    }
    let eig = mtx.symmetric_eigen();
    let (imin, _) = eig
        .eigenvalues
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(b.1))?;
    let ev = eig.eigenvectors.column(imin);
    let ax = V3::new(ev[0], ev[1], ev[2]);
    let axn = ax.norm();
    if axn < tol::ROUNDOFF {
        return None;
    }
    let a = ax / axn;
    let s = ev[3] / axn;
    if nrms.iter().any(|(_, nn)| (nn.dot(&a) - s).abs() > 1e-7) {
        return None;
    }
    // The axis through the centres of two cross-sections (circles square to a).
    let frame = Frame::from_z(P3::origin(), a);
    let to2 = |p: &P3| {
        (
            p.coords.dot(&frame.x),
            p.coords.dot(&frame.y),
            p.coords.dot(&a),
        )
    };
    let section = |pick: &dyn Fn(usize) -> bool| -> Option<(P3, f64, f64)> {
        let ps: Vec<P3> = pts
            .iter()
            .enumerate()
            .filter(|(i, _)| pick(*i))
            .map(|(_, p)| *p)
            .collect();
        if ps.len() < 3 {
            return None;
        }
        // Circle through three of them in the plane square to the axis.
        let q = |p: &P3| {
            let (x, y, _) = to2(p);
            P3::new(x, y, 0.0)
        };
        // (Not the last point: on a whole turn it is the first.)
        let (c, _, r) = circle3(&q(&ps[0]), &q(&ps[ps.len() / 3]), &q(&ps[2 * ps.len() / 3]))?;
        let h = to2(&ps[0]).2;
        Some((c, r, h))
    };
    let width = m + 1;
    let row_u = |i: usize| move |k: usize| k / width == i;
    let row_v = |j: usize| move |k: usize| k % width == j;
    // Sections at fixed u or fixed v, whichever lies square to the axis.
    let flat = |pick: &dyn Fn(usize) -> bool| {
        let hs: Vec<f64> = pts
            .iter()
            .enumerate()
            .filter(|(i, _)| pick(*i))
            .map(|(_, p)| to2(p).2)
            .collect();
        let (lo, hi) = hs
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &h| {
                (a.min(h), b.max(h))
            });
        hi - lo < tol
    };
    // (Through poles, two whose circles have a radius: a cone's row at its apex is a point.)
    type Section = (P3, f64, f64);
    let sections = |by_u: bool| -> Option<(Section, Section)> {
        let rows: &[usize] = if poles { &[0, m / 2, m] } else { &[0, m] };
        if !rows.iter().all(|&i| {
            if by_u {
                flat(&row_u(i))
            } else {
                flat(&row_v(i))
            }
        }) {
            return None;
        }
        let secs: Vec<Option<Section>> = rows
            .iter()
            .map(|&i| {
                if by_u {
                    section(&row_u(i))
                } else {
                    section(&row_v(i))
                }
            })
            .collect();
        if !poles {
            return Some((secs[0]?, secs[1]?));
        }
        let secs: Vec<Section> = secs.into_iter().flatten().filter(|s| s.1 > tol).collect();
        (secs.len() >= 2).then(|| (secs[0], secs[secs.len() - 1]))
    };
    let (s0, s1) = match (poles, flat(&row_u(0)) && flat(&row_u(m))) {
        (true, _) => sections(true).or_else(|| sections(false))?,
        (false, true) => sections(true)?,
        (false, false) => sections(false)?,
    };
    let world = |c: &P3, h: f64| P3::from(frame.x * c.x + frame.y * c.y + a * h);
    let (c0, c1) = (world(&s0.0, s0.2), world(&s1.0, s1.2));
    // Both centres on one axis line along a.
    if ((c1 - c0) - a * (c1 - c0).dot(&a)).norm() > tol {
        return None;
    }
    let dist_axis = |p: &P3| ((p - c0) - a * (p - c0).dot(&a)).norm();
    // The seam (u = 0) at the patch's corner, so a face on it keeps its parameters in one
    // piece.
    let corner = pts[0];
    let radial = ((corner - c0) - a * (corner - c0).dot(&a))
        .try_normalize(tol::DEGENERATE)
        .unwrap_or_else(|| any_perpendicular(&a));
    let seam_frame = Frame {
        o: c0,
        x: radial,
        y: a.cross(&radial),
        z: a,
    };
    if s.abs() < tol::ROUNDOFF {
        let r = s0.1;
        if pts.iter().all(|p| (dist_axis(p) - r).abs() < tol) {
            return Some(Surface::Cylinder { f: seam_frame, r });
        }
        return None;
    }
    // A cone: the radius changing linearly along the axis, heights measured from the first
    // section's centre (the seam frame's origin, where the radius is s0's).
    let h1 = (c1 - c0).dot(&a);
    if h1.abs() < tol {
        return None;
    }
    let slope = (s1.1 - s0.1) / h1;
    if pts
        .iter()
        .any(|p| (dist_axis(p) - (s0.1 + slope * (p - c0).dot(&a))).abs() > tol)
    {
        return None;
    }
    // Surface::Cone { f, r, a }: radius r at the frame's origin, half-angle a (radius growing
    // along +z for a > 0).
    Some(Surface::Cone {
        f: seam_frame,
        r: s0.1,
        a: slope.atan(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::nurbs::H;

    /// A whole circle in five unequal rational pieces (as SMLib writes them) is a circle.
    #[test]
    #[allow(
        clippy::approx_constant,
        reason = "SMLib's weights, as written in its file"
    )]
    fn uneven_rational_circle_is_a_circle() {
        let w = [
            1.0,
            0.7071067811865476,
            1.0,
            0.8535533905932737,
            0.8535533905932737,
            1.0,
            0.7071067811865475,
            1.0,
            0.7071067811865475,
            1.0,
        ];
        let p = [
            [3.618_033_988_749_895, 2.5, 5.0],
            [
                2.5583363680084634,
                0.977_197_537_924_274,
                3.5355339059327378,
            ],
            [2.5, 1.381_966_011_250_105, 5.0],
            [
                1.7385987689621367,
                1.1795817745871895,
                4.267_766_952_966_369,
            ],
            [
                1.1795817745871895,
                1.7385987689621367,
                4.267_766_952_966_369,
            ],
            [1.381_966_011_250_105, 2.5, 5.0],
            [0.9771975379242739, 2.5583363680084634, 3.5355339059327373],
            [2.5, 3.618_033_988_749_895, 5.0],
            [
                2.558_336_368_008_463,
                2.5583363680084634,
                3.5355339059327373,
            ],
            [3.618_033_988_749_895, 2.5, 5.0],
        ];
        let cps = p
            .iter()
            .zip(w)
            .map(|(q, w)| H::new(q[0], q[1], q[2], w))
            .collect();
        let knots = vec![
            0.0,
            0.0,
            0.0,
            1.711412337262567,
            1.711412337262567,
            2.567118505893851,
            3.422824674525135,
            3.422824674525135,
            5.134237011787703,
            5.134237011787703,
            6.84564934905027,
            6.84564934905027,
            6.84564934905027,
        ];
        let n = NurbsCurve {
            degree: 2,
            knots,
            cps,
        };
        match curve(&n) {
            Some(Curve::Circle { c, r, .. }) => assert!(
                (c - P3::new(2.5, 2.5, 5.0)).norm() < 1e-9 && (r - 1.118033988749895).abs() < 1e-9
            ),
            other => panic!("{other:?}"),
        }
    }

    /// A cone's NURBS form is recognised as that cone (radius at the seam frame's origin,
    /// half-angle with its sign), whichever way the radius grows and wherever the cone sits
    /// along its axis (regression: the first section's height was written `(c0 - c0)·a`).
    #[test]
    fn nurbs_cone_is_the_cone() {
        for (o, z, r, half) in [
            (P3::new(0.0, 0.0, 0.0), V3::z(), 5.0, 0.3),
            (
                P3::new(1.0, -2.0, 7.0),
                V3::new(1.0, 2.0, 2.0).normalize(),
                8.0,
                -0.4,
            ),
            (P3::new(0.0, 0.0, 40.0), -V3::y(), 2.0, 0.6),
        ] {
            let cone = Surface::Cone {
                f: Frame::from_z(o, z),
                r,
                a: half,
            };
            let n = cone.to_nurbs((0.0, 2.0), (3.0, 9.0));
            let Some(Surface::Cone { f, r: r2, a: a2 }) = surface(&n) else {
                panic!("not a cone: {:?}", surface(&n))
            };
            // The recognised cone has the same points (a's sign follows its frame's z).
            for (u, v) in [(0.0, 3.0), (1.0, 6.0), (2.0, 9.0), (0.5, 4.5)] {
                let p = n.point(u, v);
                assert!(
                    cone.distance(&p) < 1e-9
                        && Surface::Cone { f, r: r2, a: a2 }.distance(&p) < 1e-9
                );
            }
            assert!(
                (f.z.dot(&z).abs() - 1.0).abs() < 1e-9 && (a2.abs() - half.abs()).abs() < 1e-9,
                "{a2} {half}"
            );
            // The frame's origin lies on the axis, the radius r2 there.
            let (h, d) = ((f.o - o).dot(&z), f.o - o);
            assert!((d - z * h).norm() < 1e-9);
            assert!((r2 - (r + h * half.tan())).abs() < 1e-9, "{r2} at {h}");
        }
    }
}
