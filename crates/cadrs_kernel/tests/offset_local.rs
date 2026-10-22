//! Regressions for offsets, shells and local operations (`native::local`, `native::offset_round`,
//! `native::slab`) through the `Kernel` trait, checked against closed forms.

use std::f64::consts::PI;

use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::math::Frame;
use cadrs_kernel::*;
use nalgebra::{Point2, Point3, Vector3};

fn one(r: OpResult) -> BodyId {
    assert_eq!(r.bodies.len(), 1, "one body");
    r.bodies[0]
}

fn polygon(pts: &[(f64, f64)]) -> Loop {
    let p = |q: (f64, f64)| Point2::new(q.0, q.1);
    Loop { curves: (0..pts.len()).map(|i| Curve2::Line { a: p(pts[i]), b: p(pts[(i + 1) % pts.len()]), source: None }).collect() }
}

fn circle(c: (f64, f64), r: f64) -> Loop {
    Loop { curves: vec![Curve2::Circle { center: Point2::new(c.0, c.1), radius: r, source: None }] }
}

/// The region `outer` less `holes` on the top plane, extruded up by `h`.
fn prism(k: &mut NativeKernel, outer: Loop, holes: Vec<Loop>, h: f64) -> BodyId {
    one(k.extrude(&Profile::new(Plane::top(), vec![Region { outer, holes, source: None }]), Extent::Blind(h)).unwrap())
}

fn volume(k: &NativeKernel, b: BodyId) -> f64 {
    k.mass_properties(b).unwrap().volume
}

fn close(got: f64, want: f64, rel: f64) {
    assert!((got - want).abs() <= rel * want.abs(), "{got} vs {want} ({:.1e} off)", (got - want).abs() / want.abs());
}

/// The planar face with outward normal `n` at `at` along it.
fn face_at(k: &NativeKernel, b: BodyId, n: Vector3<f64>, at: f64) -> FaceId {
    k.faces(b)
        .unwrap()
        .into_iter()
        .find(|f| f.plane.is_some_and(|p| p.normal.dot(&n) > 1.0 - 1e-9) && (f.center.coords.dot(&n) - at).abs() < 1e-9)
        .expect("no such face")
        .id
}

/// A box a × b × c grown by a ball of radius d (Steiner's formula).
fn grown_box(a: f64, b: f64, c: f64, d: f64) -> f64 {
    a * b * c + 2.0 * d * (a * b + b * c + c * a) + PI * d * d * (a + b + c) + 4.0 / 3.0 * PI * d * d * d
}

/// A 10 mm cube with a through hole of radius 1 grown by 2 (more than the hole's radius): the
/// hole's wall (a concave cylinder whose offset would have no radius) fills to its axis, its
/// rims' tubes (wider than the rims) are spindle tori. Inside the hole every point is within 1
/// of the wall; above its mouths a point at radius ρ < 1 and height z is nearest the rim, so
/// it is in only when (1 − ρ)² + z² ≤ 4: the grown cube less a dimple of volume
/// 2π∫₀¹ ρ (2 − √(4 − (1 − ρ)²)) dρ at each mouth.
#[test]
fn rounded_offset_past_a_hole_radius() {
    let mut k = NativeKernel::new();
    let (r, d) = (1.0, 2.0);
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]), vec![circle((5.0, 5.0), r)], 10.0);
    let out = one(k.offset(b, &OffsetSpec { distance: d, faces: vec![], sharp: false }).unwrap());
    // Simpson's rule on the dimple's (smooth) integrand.
    let n = 2000;
    let f = |rho: f64| rho * (d - (d * d - (r - rho) * (r - rho)).sqrt());
    let h = r / n as f64;
    let dimple = 2.0 * PI * h / 3.0 * (0..=n).map(|i| f(i as f64 * h) * if i == 0 || i == n { 1.0 } else if i % 2 == 1 { 4.0 } else { 2.0 }).sum::<f64>();
    close(volume(&k, out), grown_box(10.0, 10.0, 10.0, d) - 2.0 * dimple, 1e-6);
}

/// An L-shaped block (a 10 × 10 square less its 5 × 5 corner, 8 tall) inset by 1, rounded
/// (SMLib's inset, `SetExtendConvexEdges(FALSE)`):
/// the faces move in by 1 and the concave edge, which every point within 1 of keeps away,
/// becomes a quarter round of radius 1 about it: the sharp inset plus the square corner less
/// the quarter disc, (1 − π/4), over the inset height 6.
#[test]
fn rounded_inset_rounds_concave_edges() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (10.0, 0.0), (10.0, 5.0), (5.0, 5.0), (5.0, 10.0), (0.0, 10.0)]), vec![], 8.0);
    let out = one(k.offset(b, &OffsetSpec { distance: -1.0, faces: vec![], sharp: false }).unwrap());
    // The sharp inset: the 8 × 8 square less its 5 × 5 corner beyond (4, 4) …
    let sharp = (64.0 - 25.0) * 6.0;
    close(volume(&k, out), sharp + (1.0 - PI / 4.0) * 6.0, 1e-9);
    // … which the sharp offset gives.
    let s = one(k.offset(b, &OffsetSpec { distance: -1.0, faces: vec![], sharp: true }).unwrap());
    close(volume(&k, s), sharp, 1e-9);
}

/// The same L inset by 1 with its bottom kept: the faces round it move in, the bottom stays.
#[test]
fn rounded_inset_keeps_a_face() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (10.0, 0.0), (10.0, 5.0), (5.0, 5.0), (5.0, 10.0), (0.0, 10.0)]), vec![], 8.0);
    let bottom = face_at(&k, b, -Vector3::z(), 0.0);
    let out = one(k.offset(b, &OffsetSpec { distance: -1.0, faces: vec![(bottom, 0.0)], sharp: false }).unwrap());
    close(volume(&k, out), (64.0 - 25.0) * 7.0 + (1.0 - PI / 4.0) * 7.0, 1e-9);
}

/// An outward shell with sharp corners (SMLib's `bExtendCorners`): the open bottom stays where
/// it was, the other faces move out by 4 and meet where they cross: (108 · 68 · 44) − the body.
#[test]
fn outward_shell_with_sharp_corners() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (100.0, 0.0), (100.0, 60.0), (0.0, 60.0)]), vec![], 40.0);
    let bottom = face_at(&k, b, -Vector3::z(), 0.0);
    let spec = ShellSpec { remove: vec![bottom], thickness: 4.0, outward: true, hollow: false, sharp: true };
    let out = one(k.shell_with(b, &spec).unwrap());
    close(volume(&k, out), 108.0 * 68.0 * 44.0 - 100.0 * 60.0 * 40.0, 1e-9);
}

/// A rounded inward shell of a 20 × 10 block with a fin 2 thick (x 9–11, out to y = 15), all
/// 10 high, bottom open, 1.5 thick: the fin's walls cross (it is thinner than two walls), so it
/// stays solid (OCCT's thick solids keep such ribs; this was refused as walls that intersect
/// themselves). The cavity is the block's 17 × 7 × 8.5 and, under the fin's root, the part of
/// its 2 × 1.5 strip farther than 1.5 from both concave corners (less two quarter discs, which
/// overlap by half a lens), 8.5 high.
#[test]
fn rounded_shell_keeps_a_thin_fin_solid() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (20.0, 0.0), (20.0, 10.0), (11.0, 10.0), (11.0, 15.0), (9.0, 15.0), (9.0, 10.0), (0.0, 10.0)]), vec![], 10.0);
    let bottom = face_at(&k, b, -Vector3::z(), 0.0);
    let spec = ShellSpec { remove: vec![bottom], thickness: 1.5, outward: false, hollow: false, sharp: false };
    let out = one(k.shell_with(b, &spec).unwrap());
    let r: f64 = 1.5;
    let half_lens = r * r * (1.0 / r).acos() - (r * r - 1.0).sqrt();
    let strip = 2.0 * r - (2.0 * PI * r * r / 4.0 - half_lens);
    close(volume(&k, out), 20.0 * 10.0 * 10.0 + 2.0 * 5.0 * 10.0 - (17.0 * 7.0 + strip) * 8.5, 1e-9);
    // Sharp, the walls can't be made: an error.
    let err = k.shell_with(b, &ShellSpec { sharp: true, ..spec }).unwrap_err();
    assert!(err.to_string().contains("reduce the thickness"), "{err}");
}

/// A cube's top replaced by a cylinder across it whose axis lies above the top: the side
/// planes cut the cylinder twice, and the domain (the upper half) picks the arcs. The result
/// is the 10 × 10 × 12 block under a half round of radius 5 (SMLib's local operation 3).
#[test]
fn replace_face_takes_the_arc_in_the_domain() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]), vec![], 10.0);
    let top = face_at(&k, b, Vector3::z(), 10.0);
    // u from −x round to +z: the upper half for u in [0, π].
    let f = Frame::from_xz(Point3::new(5.0, 0.0, 12.0), -Vector3::x(), Vector3::y());
    let upper = FaceReplacement { face: top, surface: Surface::Cylinder { f, r: 5.0 }, domain: Some(((0.0, PI), (0.0, 10.0))) };
    let out = one(k.replace_faces(b, std::slice::from_ref(&upper)).unwrap());
    close(volume(&k, out), 1200.0 + 125.0 * PI, 1e-9);
    // Without a domain, the crossings nearest the old edges: the lower half.
    let lower = one(k.replace_faces(b, &[FaceReplacement { domain: None, ..upper }]).unwrap());
    close(volume(&k, lower), 1200.0 - 125.0 * PI, 1e-9);
}

/// A cylinder whose side is given its exact B-spline form (closed round, not periodic: its seam
/// on the spline's boundary), then offset sharp: the seam moves as the spline's own iso line
/// (an exact iso curve of the surface, not an iso line of an analytic one).
#[test]
fn seam_on_a_closed_spline_moves() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, circle((0.0, 0.0), 5.0), vec![], 10.0);
    let side = k.faces(b).unwrap().into_iter().find(|f| f.kind == SurfaceKind::Cylinder).expect("no side").id;
    let surf = {
        let body = k.get(b).unwrap();
        let f = &body.faces[side.0 as usize];
        Surface::Nurbs(std::sync::Arc::new(f.surf.to_nurbs((0.0, 2.0 * PI), (-1.0, 11.0))))
    };
    let spline = one(k.replace_faces(b, &[FaceReplacement { face: side, surface: surf, domain: None }]).unwrap());
    close(volume(&k, spline), PI * 25.0 * 10.0, 1e-6);
    let out = one(k.offset(spline, &OffsetSpec { distance: 1.0, faces: vec![], sharp: true }).unwrap());
    close(volume(&k, out), PI * 36.0 * 12.0, 1e-5);
}

/// OCCT's offset of an L-shaped prism (a 2 × 2 square less its 1 × 1 corner, 1 deep) by 1
/// with the inner riser moved 3: the riser passes the outer side, so the step between them
/// vanishes and so does the outer side (the riser moved past it): the 5 × 4 rectangle, 3 deep.
#[test]
fn sharp_offset_where_faces_vanish() {
    let mut k = NativeKernel::new();
    let b = prism(&mut k, polygon(&[(0.0, 0.0), (2.0, 0.0), (2.0, 1.0), (1.0, 1.0), (1.0, 2.0), (0.0, 2.0)]), vec![], 1.0);
    let riser = k.faces(b).unwrap().into_iter().find(|f| f.plane.is_some_and(|p| p.normal.x > 0.5) && (f.center.x - 1.0).abs() < 1e-9).expect("no riser").id;
    let out = one(k.offset(b, &OffsetSpec { distance: 1.0, faces: vec![(riser, 3.0)], sharp: true }).unwrap());
    close(volume(&k, out), 5.0 * 4.0 * 3.0, 1e-9);
}

/// Every hole of every face lies inside its face's outer loop in (u, v).
fn holes_inside(b: &cadrs_kernel::native::topo::Body) -> std::result::Result<(), String> {
    use cadrs_kernel::native::topo::co_range;
    for (fi, f) in b.faces.iter().enumerate() {
        let polys: Vec<Vec<(f64, f64)>> = f
            .loops
            .iter()
            .map(|l| {
                l.iter()
                    .flat_map(|c| {
                        let e = &b.edges[c.edge];
                        let (t0, t1) = co_range(e, c.fwd);
                        (0..16).map(move |i| {
                            let q = c.pc.point(t0 + (t1 - t0) * i as f64 / 16.0);
                            (q.x, q.y)
                        })
                    })
                    .collect()
            })
            .collect();
        let inside = |p: (f64, f64), poly: &[(f64, f64)]| {
            let mut w = false;
            for i in 0..poly.len() {
                let (a, c) = (poly[i], poly[(i + 1) % poly.len()]);
                if (a.1 > p.1) != (c.1 > p.1) && p.0 < a.0 + (p.1 - a.1) * (c.0 - a.0) / (c.1 - a.1) {
                    w = !w;
                }
            }
            w
        };
        for (li, h) in polys.iter().enumerate().skip(1) {
            if h.iter().any(|p| !inside(*p, &polys[0])) {
                return Err(format!("face {fi}: hole {li} isn't inside the outer loop"));
            }
        }
    }
    Ok(())
}

/// OCCT's draft angle J4: a box fused onto a cylinder's side, its face drafted; the hole the
/// box leaves in the cylinder grows across the cylinder's seam, which has to move round out
/// of it (the hole may not straddle the seam).
#[test]
fn draft_moves_a_seam_out_of_a_hole() {
    use cadrs_kernel::native::draw;
    let mut k = NativeKernel::new();
    let p = k.plane(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let p = k.pcylinder_on(&p, 10.0, 20.0).unwrap();
    let bx = k.box_at(5.0, 0.5, 0.0, 10.0, 5.0, 20.0).unwrap();
    let f = k.bfuse(&p, &bx).unwrap();
    let faces = k.nexplode_faces(&f).unwrap();
    let draw::Shape::Body(r) = k.depouille(&f, [0.0, 0.0, 1.0], &[(&faces[3], 5.0, [0.0, 0.0, 20.0], [0.0, 0.0, 1.0])]).unwrap() else { panic!("not a body") };
    holes_inside(k.get(r).unwrap()).unwrap();
}

/// The loop of face `fi` of SMLib's off_extrusion (its end faces: two B-splines, an arc and a
/// line) as a wire offset takes it: each edge in the loop's direction, a reversed one as a
/// decreasing parameter range, in the face's plane (or as it is in space).
fn extrusion_loop(fi: usize, in_plane: bool) -> Vec<(cadrs_kernel::native::curve::Curve, f64, f64)> {
    use cadrs_kernel::native::math::Affine;
    let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/smlib/offset/off_extrusion.smb")).unwrap();
    let b = cadrs_kernel::native::smb::read_brep(&bytes).unwrap().body;
    let f = &b.faces[fi];
    let Surface::Plane { f: fr } = &f.surf else { panic!("face {fi} isn't planar") };
    let fr = Frame::from_z(fr.o, fr.x.cross(&fr.y));
    let m = nalgebra::Matrix3::from_rows(&[fr.x.transpose(), fr.y.transpose(), fr.z.transpose()]);
    let to_plane = Affine { m, t: -(m * fr.o.coords) };
    f.loops[0]
        .iter()
        .map(|c| {
            let e = &b.edges[c.edge];
            let curve = if in_plane { e.curve.transformed(&to_plane) } else { e.curve.clone() };
            if c.fwd { (curve, e.t[0], e.t[1]) } else { (curve, e.t[1], e.t[0]) }
        })
        .collect()
}

/// The planar wire offset of off_extrusion's end loop with its reversed edges given as
/// decreasing parameter ranges: offset outward and inward (it panicked in a closest-point
/// search over a backwards range). A side face's loop, square to the plane z = 0 the offset
/// works in, flattens to edges of no length: an error, not a panic.
#[test]
fn wire_offset_takes_reversed_edges() {
    use cadrs_kernel::native::curve::Curve;
    use cadrs_kernel::native::offset2d::offset_wire;
    let wire = extrusion_loop(0, true);
    assert!(wire.iter().any(|e| matches!(e.0, Curve::Nurbs(_))) && wire.iter().any(|e| e.1 > e.2), "a spline and a reversed edge");
    for d in [0.3, -0.3, 1.0] {
        let loops = offset_wire(&wire, d).unwrap_or_else(|e| panic!("offset by {d}: {e:?}"));
        assert_eq!(loops.len(), 1, "offset by {d}");
    }
    // Faces 3 and up are the extrusion's flat sides.
    assert!(offset_wire(&extrusion_loop(3, false), 0.3).is_err());
}

/// Offsets of off_extrusion's end loop past the radius of its concave B-spline (about 1.1):
/// that edge's offset folds into a swallowtail, whose two cuts meet at one point. They stayed
/// two cuts (they were merged as one, and the loop came out empty from 1.8 to 2.3 outward): one
/// loop at every distance, its area growing with the distance.
#[test]
fn wire_offset_trims_a_swallowtail() {
    use cadrs_kernel::native::offset2d::offset_wire;
    let wire = extrusion_loop(0, true);
    let area = |d: f64| -> f64 {
        let loops = offset_wire(&wire, d).unwrap_or_else(|e| panic!("offset by {d}: {e:?}"));
        assert_eq!(loops.len(), 1, "offset by {d}");
        let mut a = 0.0;
        for p in &loops[0] {
            for k in 0..200 {
                let (x, y) = (p.c.point(p.t0 + (p.t1 - p.t0) * k as f64 / 200.0), p.c.point(p.t0 + (p.t1 - p.t0) * (k + 1) as f64 / 200.0));
                a += 0.5 * (x.x * y.y - y.x * x.y);
            }
        }
        a.abs()
    };
    let ds = [-3.0, -2.5, -2.0, -1.5, 1.5, 1.8, 2.0, 2.2, 2.3, 2.5];
    let areas: Vec<f64> = ds.iter().map(|&d| area(d)).collect();
    assert!(areas.windows(2).all(|w| w[0] < w[1]), "{areas:?}");
}

/// OCCT bug 26233: a whole sphere's face thickened by 10 outward: the solid between radii 100
/// and 110, its outer sphere one shell and the inner one a void (the face has no boundary to
/// join them across).
#[test]
fn thickened_sphere_face_has_a_void() {
    use cadrs_kernel::native::draw;
    use cadrs_kernel::native::topo::CheckLevel;
    let mut k = NativeKernel::new();
    let s = k.psphere(100.0).unwrap();
    let faces = k.explode_faces(&s).unwrap();
    let draw::Shape::Body(r) = k.thickshell(&faces[0], 10.0).unwrap() else { panic!("not a body") };
    let b = k.get(r).unwrap();
    assert_eq!((b.solids.len(), b.shells.len(), b.faces.len()), (1, 2, 2));
    assert_eq!(b.check_with(CheckLevel::Full), Vec::<String>::new());
    close(volume(&k, r), 4.0 / 3.0 * PI * (110f64.powi(3) - 100f64.powi(3)), 1e-6);
}

/// The slab of a B-spline face closing to a cone-like apex (SMLib's boolean_trees_result, face
/// 11: its normals at the apex turn round a cone about −y): the apex offsets to a closed curve,
/// the offset face's edge there, with the fan of normals between it and the apex a side. The
/// slab left the point out, and the offset face's loop didn't close.
#[test]
fn slab_of_a_face_with_a_cone_like_apex() {
    use cadrs_kernel::native::topo::CheckLevel;
    let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/smlib/offset/boolean_trees_result.smb")).unwrap();
    let b = cadrs_kernel::native::smb::read_brep(&bytes).unwrap().body;
    let fi = 11;
    let apex = cadrs_kernel::native::math::P3::new(12.0, -8.0, 0.0);
    assert!(b.faces[fi].loops.iter().flatten().any(|c| b.edges[c.edge].degenerate && (b.verts[b.edges[c.edge].v[0]].p - apex).norm() < 1e-9), "face {fi} has no pole at the apex");
    let slab = cadrs_kernel::native::slab::face_slab(&b, fi, 0.3).unwrap();
    assert_eq!(slab.check_with(CheckLevel::Full), Vec::<String>::new());
    // The fan: a closed edge 0.3 from the apex all round.
    let fan = slab.edges.iter().any(|e| !e.degenerate && e.v[0] == e.v[1] && (0..8).all(|i| ((e.point(e.t[0] + (e.t[1] - e.t[0]) * i as f64 / 8.0) - apex).norm() - 0.3).abs() < 1e-6));
    assert!(fan, "no offset curve round the apex");
}

/// The inward slab of OffsetBoxSphOnCorner's corner spline (a sphere of radius about 37.3 as a
/// B-spline closed in v, poles at u = 0 and 1, its loop running round the pole at u = 1 along
/// its seam): the seam's offset is the offset surface's own seam, so the offset face's loop goes
/// round that pole too, and its area is the face's scaled by ((R − t)/R)². A fitted seam strayed
/// off the offset surface's seam near the pole, and the offset face left out the ring round it.
#[test]
fn slab_of_a_face_round_a_pole_along_its_seam() {
    use cadrs_kernel::native::mass;
    use cadrs_kernel::native::topo::CheckLevel;
    let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/smlib/offset/OffsetBoxSphOnCorner.smb")).unwrap();
    let b = cadrs_kernel::native::smb::read_brep(&bytes).unwrap().body;
    let fi = 10;
    // The sphere (read as one, or as the B-spline: its poles at u = 0 and 1 a diameter apart).
    let r = match &b.faces[fi].surf {
        Surface::Sphere { r, .. } => *r,
        s => {
            let (p0, p1) = (s.eval(0.0, 0.3), s.eval(1.0, 0.3));
            let (c, r) = (p0 + (p1 - p0) * 0.5, (p1 - p0).norm() * 0.5);
            assert!((0..=8).all(|i| ((s.eval(0.1 * i as f64 + 0.05, 0.37) - c).norm() - r).abs() < 1e-6 * r), "not a sphere");
            r
        }
    };
    let t = 2.54;
    let slab = cadrs_kernel::native::slab::face_slab(&b, fi, -t).unwrap();
    assert_eq!(slab.check_with(CheckLevel::Full), Vec::<String>::new());
    let want = mass::face_area(&b, fi).0 * ((r - t) / r).powi(2);
    let got = (0..slab.faces.len()).map(|f| mass::face_area(&slab, f).0).fold(f64::NAN, |best, a| if (a - want).abs() < (best - want).abs() || best.is_nan() { a } else { best });
    assert!((got - want).abs() < 1e-4 * want, "offset face {got}, want {want}");
}

/// A sourced rectangle (each side a curve with its own id), for naming.
fn named_rect(w: f64, h: f64) -> Loop {
    let c = [Point2::new(0.0, 0.0), Point2::new(w, 0.0), Point2::new(w, h), Point2::new(0.0, h)];
    Loop { curves: (0..4).map(|i| Curve2::Line { a: c[i], b: c[(i + 1) % 4], source: Some(i as u64 + 1) }).collect() }
}

/// Persistent names through offsets and shells: a 10 × 20 × 30 box offset by 2 rounded keeps
/// each face's name once (its offset), and names its 12 edge tubes and 8 corner balls from the
/// edges and vertices; offset sharp, the same six names and nothing new. Shelled 1 thick with
/// its top open, the top's name is on the walls' rim (what is left of the top) and each other
/// face's name on two faces (its outer side and the inner wall offset from it). (The offset's history used to say face i came from
/// face i, whatever the union of its pieces had numbered them.)
#[test]
fn names_survive_offsets_and_shells() {
    use cadrs_kernel::naming::{self, FaceName, FaceOrigin};
    let op = |n: u128| uuid::Uuid::from_u128(n);
    let mut k = NativeKernel::new();
    let r = k.extrude(&Profile::new(Plane::top(), vec![Region { outer: named_rect(10.0, 20.0), holes: vec![], source: None }]), Extent::Blind(30.0)).unwrap();
    let b = one(r.clone());
    let names = naming::name_body(&k, b, op(1), &r.history, &[]).unwrap();
    assert!(names.faces.iter().all(|n| n.is_stable()));
    let check = |k: &NativeKernel, r: &OpResult, edges: usize, verts: usize, twice: bool, rim: Option<FaceName>| {
        let out = r.bodies[0];
        let got = naming::name_body(k, out, op(2), &r.history, &[(b, &names)]).unwrap();
        assert!(got.faces.iter().all(|n| n.is_stable()), "{:?}", got.faces);
        for n in &names.faces {
            // (The open face's name stays on the walls' rim, what is left of it.)
            let want = if Some(*n) == rim { 1 } else if twice { 2 } else { 1 };
            // (Two faces from one are its pieces: split 1 and 2.)
            assert_eq!(got.faces.iter().filter(|g| g.base() == n.base()).count(), want, "{n:?} in {:?}", got.faces);
        }
        let count = |f: &dyn Fn(&FaceOrigin) -> bool| got.faces.iter().filter(|n| f(&n.origin)).count();
        assert_eq!(count(&|o| matches!(o, FaceOrigin::FromEdge { .. })), edges);
        assert_eq!(count(&|o| matches!(o, FaceOrigin::FromVertex { .. })), verts);
    };
    let rounded = k.offset(b, &OffsetSpec { distance: 2.0, faces: vec![], sharp: false }).unwrap();
    check(&k, &rounded, 12, 8, false, None);
    let sharp = k.offset(b, &OffsetSpec { distance: 2.0, faces: vec![], sharp: true }).unwrap();
    check(&k, &sharp, 0, 0, false, None);
    let top = face_at(&k, b, Vector3::z(), 30.0);
    let shelled = k.shell_with(b, &ShellSpec { remove: vec![top], thickness: 1.0, outward: false, hollow: false, sharp: false }).unwrap();
    let top_name = names.faces[top.0 as usize];
    check(&k, &shelled, 0, 0, true, Some(top_name));
}
