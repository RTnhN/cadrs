//! Blends from models that failed in the app (their input bodies dumped with `CADRS_DUMP_DIR`).

use cadrs_kernel::native::*;
use cadrs_kernel::*;

/// A 2 mm chamfer along a round cut's rim, which runs over the top face, down two 5 mm fillets
/// and the side face: the contacts cross from the fillets onto the flat faces, and the sections
/// on either side of each crossing must agree (the boolean left an open shell).
#[test]
fn chamfer_over_fillets() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/chamfer_over_fillets.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let edges = [14, 16, 22, 25, 26].map(EdgeId);
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    // About the chain's length (176 mm) times the section (2 mm²), less where it bends.
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((320.0..352.0).contains(&removed), "{removed}");
    // The chamfer ends flush with the round cut's floor (z = 15.5) instead of digging into it:
    // the floor only gains the chamfer's two end sections (under 2 mm² each).
    let floor = |id: BodyId| -> f64 {
        k.faces(id).unwrap().iter().filter(|f| f.kind == SurfaceKind::Plane && (f.center.z - 15.5).abs() < 1e-6).map(|f| f.area).sum()
    };
    let gained = floor(out) - floor(b);
    assert!(gained > 0.0 && gained < 4.0, "floor area {} -> {}", floor(b), floor(out));
}

/// The same model's top arc alone (Tangent propagation off): the chamfer stops square at the
/// arc's ends, where the rim runs on smoothly over the fillets, instead of running on past them.
#[test]
fn chamfer_stops_where_the_chain_runs_on() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/chamfer_over_fillets.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let r = k.chamfer_with(b, &[EdgeId(22)], &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    // The arc (103.5 mm) times the section (2 mm², about 3% more for the section's offset from
    // the round cut's axis).
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((207.0..216.0).contains(&removed), "{removed}");
}

/// A 2 mm chamfer round a block's top: an outline with a notch (two inside corners) and a
/// rectangular hole (four more). Each pair of chamfers meets on the plane bisecting its corner:
/// a mitre at the convex corners, a valley at the inside ones. Exact volume: the section
/// (2 mm²) along the outline offset d/3 into the material (2d/3 shorter at a convex corner,
/// longer at an inside one); and no faces but the 14 the block had and the 12 chamfers.
#[test]
fn chamfer_inside_corners() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/chamfer_inside_corners.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let edges = [2, 6, 9, 12, 15, 18, 21, 23, 26, 30, 33, 35].map(EdgeId);
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((removed - 855.168388).abs() < 1e-4, "{removed}");
    assert_eq!(k.faces(out).unwrap().len(), 14 + 12);
}

/// The same block chamfered on its notch wall all round (its top and bottom edges cut, the
/// vertical inside corner filled), a top edge and two hole edges. Where the fill meets a cut,
/// the cut's plane runs on over the fill's end; a chamfer ending against an unchamfered wall
/// ends flush, its end part of that wall. No faces but the block's 14 and the 7 chamfers.
#[test]
fn chamfer_mixed_corners() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/chamfer_mixed_corners.body")).unwrap());
    let edges = [26, 35, 8, 10, 11, 12, 9].map(EdgeId);
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    assert_eq!(k.faces(out).unwrap().len(), 14 + 7);
    for (one, faces) in [(&[11][..], 15), (&[12, 11][..], 16), (&[10, 11, 12][..], 17), (&[26][..], 15)] {
        let ids: Vec<EdgeId> = one.iter().map(|&i| EdgeId(i)).collect();
        let r = k.chamfer_with(b, &ids, &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
        assert_eq!(k.faces(r.bodies[0]).unwrap().len(), faces, "{one:?}");
    }
}

/// A wheel's five spoke windows chamfered 2 mm on both sides (a user's document): each side
/// face is split in two on its plane by a circle the windows' chamfers cross, and the edge list
/// holds an edge between two tangent cylinders. The tangent edge is passed over (as OCCT does),
/// the rails run on across the split onto the plane's other face, and where a window's outer
/// arc lies wholly past the split its strip starts on that face (the crossings of the split
/// circle are found however long it is).
#[test]
fn wheel_window_chamfer() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/docs/wheel_window_chamfer.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let edges = [1, 2, 28, 29, 30, 31, 32, 33, 67, 68, 69, 70, 71, 72, 101, 102, 34, 35, 36, 37, 38, 39, 73, 74, 75, 76, 77, 78, 107, 108, 10, 11, 12, 13, 14, 15, 49, 50, 51, 52, 53, 54, 84, 85, 16, 17, 18, 19, 20, 21, 55, 56, 57, 58, 59, 60, 92, 93, 22, 23, 24, 25, 26, 27, 61, 62, 63, 64, 65, 66, 98, 99].map(EdgeId);
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(2.0))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    // Ten windows of about 71 mm round, a 2 mm² section.
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((1350.0..1500.0).contains(&removed), "{removed}");
}

/// A pipe snap's edges chamfered 0.5 mm (a user's document), one of them where a cross hole
/// meets the bore: the faces' angle varies along it, so its chamfer is a law of radii (the
/// boolean tool's result wasn't valid).
#[test]
fn pipe_snap_chamfer() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/docs/pipe_snap_chamfer.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let edges = [4, 45, 46, 26, 2, 7, 10, 13, 16, 19, 32, 35, 38, 41, 43, 22, 25, 29, 0, 5, 8, 11, 14, 17, 30, 33, 36, 39, 42, 20, 23, 27, 24, 28, 31, 15, 18, 21].map(EdgeId);
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(0.5))).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    // OCCT takes 62.45 mm³ off.
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((60.0..65.0).contains(&removed), "{removed}");
}

/// The rims of a cross hole through a D-shaped block filleted 5 mm (a user's document): the
/// hole's axis lies where the block's half cylinder meets its flat sides, so the ball rolls
/// from the cylinder onto the planes. A crossing on the cylinder's far side (its surface goes
/// on round past the face) isn't a roll, nor one whose spine parameter isn't a number (the
/// ball rolled back and forth for ever).
#[test]
fn cross_hole_fillet() {
    let mut k = NativeKernel::new();
    let b = k.insert(bodyio::read(include_bytes!("data/docs/cross_hole_fillet.body")).unwrap());
    let v0 = k.mass_properties(b).unwrap().volume;
    let spec = FilletSpec { allow_overflow: true, propagate: true, ..FilletSpec::radius(5.0) };
    let r = k.fillet_with(b, &[11, 12, 2, 6].map(EdgeId), &spec).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty());
    // OCCT takes 1986 mm³ off.
    let removed = v0 - k.mass_properties(out).unwrap().volume;
    assert!((1880.0..2040.0).contains(&removed), "{removed}");
}

/// A degenerate edge among the picked ones (OCCT's `explode` lists a cone's apex; blend
/// encoderegularity_a5 picks two pole edges) is passed over, as OCCT's blend does.
#[test]
fn degenerate_picked_edge_is_passed_over() {
    let mut k = NativeKernel::new();
    let s = k.pcone_a(None, 50.0, 0.0, 120.0, 90.0).unwrap();
    let es = k.explode_edges(&s).unwrap();
    let (draw::Shape::Body(b), draw::Shape::BodyEdge(_, apex), draw::Shape::BodyEdge(_, arc)) = (&s, &es[0], &es[2]) else { panic!("not a body's edges") };
    assert!(k.get(*b).unwrap().edges()[apex.0 as usize].degenerate);
    let area = |k: &mut NativeKernel, edges: &[EdgeId]| {
        let r = k.fillet(*b, edges, 10.0).unwrap();
        k.mass_properties(r.bodies[0]).unwrap().surface_area
    };
    let both = area(&mut k, &[*apex, *arc]);
    let one = area(&mut k, &[*arc]);
    assert!((both - one).abs() < 1e-9 * one, "{both} {one}");
    assert!(k.fillet(*b, &[*apex], 10.0).is_err());
}

/// Dr Browns cap, Fillet 1: R4 rounds along a window cut into a dome (a spindle torus's band)
/// across the dome's seam, whose input came with the window's loop crossing the seam while the
/// seam edge, not split there, ran on through the window. The local fillet couldn't place its
/// rails in that face (and the blend tool's boolean left an open shell); the hole is opened
/// into the outer loop first. Built here: the dome less a bar across its seam, the window's
/// seam pieces then joined back into one edge running through it, as the document had them.
#[test]
fn fillet_round_a_window_across_a_seam() {
    use nalgebra::{Point2, Point3, Vector3};
    use std::f64::consts::TAU;
    let mut k = NativeKernel::new();
    let (r, cx, cz) = (80.0_f64, -56.0_f64, 1.0_f64);
    let top = (-cx / r).acos();
    let (x1, z1) = (cx + r * top.cos(), cz + r * top.sin());
    let curves = vec![
        Curve2::Line { a: Point2::new(0.0, cz), b: Point2::new(cx + r, cz), source: None },
        Curve2::Arc { center: Point2::new(cx, cz), radius: r, start_angle: 0.0, sweep: top, source: None },
        Curve2::Line { a: Point2::new(x1, z1), b: Point2::new(0.0, cz), source: None },
    ];
    let xz = Plane { origin: Point3::origin(), x_dir: Vector3::x_axis(), normal: -Vector3::y_axis() };
    let profile = Profile::new(xz, vec![Region { outer: Loop { curves }, holes: vec![], source: None }]);
    let dome = k.revolve(&profile, Axis { origin: Point3::origin(), dir: Vector3::z_axis() }, TAU).unwrap().bodies[0];
    let side = Plane { origin: Point3::new(0.0, 40.0, 0.0), x_dir: Vector3::x_axis(), normal: -Vector3::y_axis() };
    let c = Curve2::Circle { center: Point2::new(24.68, 31.92), radius: 11.25, source: None };
    let bar = k.extrude(&Profile::new(side, vec![Region { outer: Loop { curves: vec![c] }, holes: vec![], source: None }]), Extent::Blind(80.0)).unwrap().bodies[0];
    let cut = k.boolean(BoolOp::Subtract, dome, &[bar]).unwrap().bodies[0];
    // The torus face: its loop runs up the seam, round one half of the window, up the seam
    // again, … and back down. Join the seam's pieces and give the window a loop of its own.
    let mut b = (**k.get(cut).unwrap()).clone();
    let fi = b.faces.iter().position(|f| matches!(f.surf, surface::Surface::Torus { .. })).unwrap();
    let l = b.faces[fi].loops[0].clone();
    let seam = |e: usize| l.iter().filter(|c| c.edge == e).count() == 2;
    let n = l.len();
    // The two window halves: the coedges between two different seam pieces.
    let halves: Vec<usize> = (0..n)
        .filter(|&i| {
            let (p, q) = (l[(i + n - 1) % n].edge, l[(i + 1) % n].edge);
            !seam(l[i].edge) && seam(p) && seam(q) && p != q
        })
        .collect();
    let halves: [usize; 2] = halves.clone().try_into().unwrap_or_else(|_| panic!("{halves:?}"));
    assert_eq!(halves.len(), 2, "{:?}", l.iter().map(|c| (c.edge, c.fwd, seam(c.edge))).collect::<Vec<_>>());
    let (below, above) = (l[(halves[0] + n - 1) % n].edge, l[(halves[0] + 1) % n].edge);
    let (lo, hi) = if b.edges[below].t[0] < b.edges[above].t[0] { (below, above) } else { (above, below) };
    let joined = topo::Edge { t: [b.edges[lo].t[0], b.edges[hi].t[1]], v: [b.edges[lo].v[0], b.edges[hi].v[1]], ..b.edges[lo].clone() };
    b.edges[lo] = joined;
    let mut outer = Vec::new();
    let mut hole = Vec::new();
    for (i, c) in l.iter().enumerate() {
        if halves.contains(&i) {
            hole.push(c.clone());
        } else if c.edge == hi {
            continue;
        } else {
            outer.push(c.clone());
        }
    }
    // The hole's second half in the first's period (its p-curves running on through the seam).
    let du = hole[0].pc.point(b.edges[hole[0].edge].t[0]).x - hole[1].pc.point(b.edges[hole[1].edge].t[1]).x;
    let du = (du / TAU).round() * TAU;
    hole[1].pc = hole[1].pc.transformed(&math::Affine::translation(math::V3::new(du, 0.0, 0.0)));
    b.faces[fi].loops = vec![outer, hole];
    // The seam's upper piece, unused now, as a point at the apex (so edge indices stay).
    b.edges[hi].degenerate = true;
    let id = k.insert(b);
    let window = halves.map(|i| EdgeId(l[i].edge as u64));
    let v0 = k.mass_properties(id).unwrap().volume;
    let spec = FilletSpec { size: FilletSize::Radius(4.0), allow_overflow: true, profile: FilletProfile::Circular, propagate: true };
    let out = k.fillet_with(id, &window, &spec).unwrap().bodies[0];
    assert!(k.get(out).unwrap().check().is_empty(), "{:?}", k.get(out).unwrap().check());
    // A convex round removes a little material all round the window.
    let v = k.mass_properties(out).unwrap().volume;
    assert!(v < v0 && v > v0 - 200.0, "{v0} -> {v}");
}

/// A cabinet side (garbage_cabinet, Chamfer 1): an L-shaped plate 20 deep whose front 5 mm
/// is cut on a diagonal, chamfered 5 mm along both inside edges of the L's step (fills) and
/// four outside edges at the diagonal's ends (cuts). Where the fills' tools run on past their
/// corners and the cuts nearly meet, the tools left islands of 0.02 to 0.04 mm³ as separate
/// solids (seven parts where there are three); a blend never splits a solid, so the result is
/// one solid, as the input was.
#[test]
fn chamfer_leaves_no_islands() {
    let mut k = NativeKernel::new();
    let p = |x: f64, z: f64| nalgebra::Point2::new(x, z);
    let polygon = |pts: &[nalgebra::Point2<f64>]| {
        let n = pts.len();
        Loop { curves: (0..n).map(|i| Curve2::Line { a: pts[i], b: pts[(i + 1) % n], source: None }).collect() }
    };
    // Sketched on the XZ plane (in-plane y along +Z), extruded along −Y.
    let at = |y: f64| Plane { origin: nalgebra::Point3::new(0.0, y, 0.0), x_dir: nalgebra::Vector3::x_axis(), normal: -nalgebra::Vector3::y_axis() };
    let prism = |k: &mut NativeKernel, y: f64, depth: f64, pts: &[nalgebra::Point2<f64>]| {
        let region = Region { outer: polygon(pts), holes: vec![], source: None };
        k.extrude(&Profile::new(at(y), vec![region]), Extent::Blind(depth)).unwrap().bodies[0]
    };
    let front = prism(&mut k, 0.0, 5.0, &[p(0.0, 0.0), p(5.0, 0.0), p(55.0, 50.0), p(55.0, 55.0), p(0.0, 55.0)]);
    let back = prism(&mut k, -5.0, 15.0, &[p(0.0, 0.0), p(5.0, 0.0), p(5.0, 50.0), p(55.0, 50.0), p(55.0, 55.0), p(0.0, 55.0)]);
    let b = k.boolean(BoolOp::Union, front, &[back]).unwrap().bodies[0];
    let v0 = k.mass_properties(b).unwrap().volume;
    // The edges by their midpoints: the step's two inside edges, the outside edges at x = 5,
    // z = 0 and x = 55, z = 50 either side of the step.
    let mids = [(5.0, -5.0, 25.0), (30.0, -5.0, 50.0), (5.0, -12.5, 0.0), (5.0, -2.5, 0.0), (5.0, -12.5, 50.0), (55.0, -12.5, 50.0), (55.0, -2.5, 50.0)];
    let infos = k.edges(b).unwrap();
    let edges: Vec<EdgeId> = mids
        .iter()
        .map(|&(x, y, z)| {
            let m = nalgebra::Point3::new(x, y, z);
            infos.iter().find(|e| (nalgebra::center(&e.start, &e.end) - m).norm() < 1e-9).expect("the edge").id
        })
        .collect();
    let r = k.chamfer_with(b, &edges, &ChamferOpts::new(ChamferSpec::EqualDistance(5.0))).unwrap();
    let out = k.get(r.bodies[0]).unwrap();
    assert!(out.check().is_empty(), "{:?}", out.check());
    assert_eq!(out.solids.len(), 1);
    // The fills (two inside edges, 50 mm each, 12.5 mm² of section) outweigh the cuts.
    let v = k.mass_properties(r.bodies[0]).unwrap().volume;
    assert!(v > v0, "{v0} -> {v}");
}

/// Edges of body `b` picked by `pick` filleted 5 mm by local trimming alone (no tool and
/// boolean): the result checked, and its volume and area against OCCT's.
fn local_fillet_matches(k: &mut NativeKernel, b: BodyId, pick: &dyn Fn(&[EdgeInfo]) -> Vec<EdgeId>, volume: f64, area: f64) {
    let edges = pick(&k.edges(b).unwrap());
    let laws: Vec<FilletLaw> = edges.iter().map(|&edge| FilletLaw { edge, radii: vec![(0.0, 5.0), (1.0, 5.0)] }).collect();
    let r = blend::fillet_laws_local(k, b, &laws).unwrap();
    let out = r.bodies[0];
    assert!(k.get(out).unwrap().check().is_empty(), "{:?}", k.get(out).unwrap().check());
    let m = k.mass_properties(out).unwrap();
    assert!((m.volume - volume).abs() < 1e-4 * volume, "volume {} (OCCT {volume})", m.volume);
    assert!((m.surface_area - area).abs() < 1e-4 * area, "area {} (OCCT {area})", m.surface_area);
}

/// A convex fillet round one of two fused cylinders' top meeting the concave one up the valley
/// between them, the top's edge round the other cylinder left sharp (OCCT blend simple P2:
/// cylinders of radius 50 and height 150, their axes 70 apart): the strips stop where their
/// rails cross on the cylinder they share; the convex one at its section there, the concave
/// one running on to the top's edge round the other cylinder; and a three-sided face fills the
/// corner, its side on the top running out over the valley the concave fillet fills (the top
/// grows there).
#[test]
fn fillet_convex_meets_concave() {
    use nalgebra::{Point2, Point3, Vector3};
    use std::f64::consts::PI;
    let mut k = NativeKernel::new();
    // The two circles' outline: round the first from the upper crossing to the lower, then
    // round the second back.
    let a = (35.0_f64 / 50.0).acos();
    let curves = vec![
        Curve2::Arc { center: Point2::new(50.0, 50.0), radius: 50.0, start_angle: a, sweep: 2.0 * (PI - a), source: None },
        Curve2::Arc { center: Point2::new(120.0, 50.0), radius: 50.0, start_angle: PI + a, sweep: 2.0 * (PI - a), source: None },
    ];
    let xy = Plane { origin: Point3::origin(), x_dir: Vector3::x_axis(), normal: Vector3::z_axis() };
    let profile = Profile::new(xy, vec![Region { outer: Loop { curves }, holes: vec![], source: None }]);
    let b = k.extrude(&profile, Extent::Blind(150.0)).unwrap().bodies[0];
    let pick = |es: &[EdgeInfo]| -> Vec<EdgeId> {
        // The valley's edge at y < 50, and the first cylinder's top arc.
        let valley = es.iter().find(|e| e.curve == CurveKind::Line && e.mid.y < 50.0).expect("the valley");
        let top = es.iter().find(|e| e.mid.z > 149.0 && e.circle.is_some_and(|c| (c.center.x - 50.0).abs() < 1e-6)).expect("the top arc");
        vec![valley.id, top.id]
    };
    local_fillet_matches(&mut k, b, &pick, 2133929.39, 98102.78);
}

/// The reverse: a short cylinder standing out of a tall one, filleted round its top (convex)
/// and where its top meets the tall cylinder (concave), and up one valley between them (OCCT
/// blend simple P7: radius 50, heights 200 and 120, the short one's base at 30 and the axes 50
/// apart). At the valley's top all three meet (a patched corner); at the other end of the
/// arcs the valley is left sharp: the concave fillet stops at its section through the rails'
/// crossing on the short cylinder's top, the convex one runs on down to the valley.
#[test]
fn fillet_concave_meets_convex() {
    let mut k = NativeKernel::new();
    let c1 = k.pcylinder(50.0, 200.0).unwrap();
    let c1 = k.ttranslate(&c1, 50.0, 50.0, 0.0).unwrap();
    let c2 = k.pcylinder(50.0, 120.0).unwrap();
    let c2 = k.ttranslate(&c2, 100.0, 50.0, 30.0).unwrap();
    let draw::Shape::Body(b) = k.bfuse(&c1, &c2).unwrap() else { panic!("not a body") };
    let pick = |es: &[EdgeInfo]| -> Vec<EdgeId> {
        let mut out: Vec<EdgeId> = es.iter().filter(|e| (e.mid.z - 150.0).abs() < 1e-6 && e.circle.is_some()).map(|e| e.id).collect();
        out.push(es.iter().find(|e| e.curve == CurveKind::Line && e.mid.y < 50.0 && e.mid.z > 30.0).expect("the valley").id);
        out
    };
    local_fillet_matches(&mut k, b, &pick, 2144319.08, 99976.90);
}
