use cadrs_kernel::native::*;
use cadrs_kernel::*;
use nalgebra::{Point2, Point3, Vector3};

fn p(x: f64, y: f64) -> Point2<f64> { Point2::new(x, y) }
fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Loop {
    let l = |a, b| Curve2::Line { a, b, source: None };
    Loop { curves: vec![l(p(x0, y0), p(x1, y0)), l(p(x1, y0), p(x1, y1)), l(p(x1, y1), p(x0, y1)), l(p(x0, y1), p(x0, y0))] }
}
fn reg(l: Loop) -> Region { Region { outer: l, holes: vec![], source: None } }

#[test]
fn debug_box_minus_cylinder() {
    let mut k = NativeKernel::new();
    let a = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 10., 20.))]), Extent::Blind(30.0)).unwrap().bodies[0];
    let c = Loop { curves: vec![Curve2::Circle { center: p(5., 5.), radius: 2.0, source: None }] };
    let t = k.extrude(&Profile::new(Plane::top(), vec![reg(c)]), Extent::Blind(30.0)).unwrap().bodies[0];
    let ba = (**k.get(a).unwrap()).clone();
    let bt = (**k.get(t).unwrap()).clone();
    match boolean::boolean_bodies(&ba, &bt, BoolOp::Subtract) {
        Ok(r) => println!("ok faces {}", r.body.faces.len()),
        Err(e) => println!("err {e}"),
    }
}

#[test]
fn debug_mirror_union() {
    let mut k = NativeKernel::new();
    let half = Region { outer: Loop { curves: vec![Curve2::Arc { center: p(0., 0.), radius: 10.0, start_angle: 0.0, sweep: std::f64::consts::PI, source: None }, Curve2::Line { a: p(-10., 0.), b: p(10., 0.), source: None }] }, holes: vec![], source: None };
    let hb = k.extrude(&Profile::new(Plane::top(), vec![half]), Extent::Blind(5.0)).unwrap().bodies[0];
    let hm = k.transform_motion(hb, &Motion::reflection(Point3::origin(), Vector3::y_axis())).unwrap().bodies[0];
    println!("mirror check {:?} vol {}", k.get(hm).unwrap().check(), k.mass_properties(hm).unwrap().volume);
    let (a, b) = ((**k.get(hb).unwrap()).clone(), (**k.get(hm).unwrap()).clone());
    match boolean::boolean_bodies(&a, &b, BoolOp::Union) {
        Ok(r) => println!("ok faces {} solids {} shells {:?} check {:?}", r.body.faces.len(), r.body.solids.len(), r.body.shells, r.body.check()),
        Err(e) => println!("err {e}"),
    }
}

#[test]
fn debug_revolve_upto() {
    let mut k = NativeKernel::new();
    let front = Plane { origin: Point3::origin(), x_dir: Vector3::x_axis(), normal: -Vector3::y_axis() };
    let prof = Profile::new(front, vec![reg(rect(5., 0., 10., 20.))]);
    let plate = k.extrude(&Profile::new(Plane { origin: Point3::new(0., 0., -5.), ..Plane::top() }, vec![reg(rect(-30., 5., 0., 40.))]), Extent::Blind(35.0)).unwrap().bodies[0];
    let axis = Axis { origin: Point3::origin(), dir: Vector3::z_axis() };
    let spec = RevolveSpec { full: false, end: RevolveEnd::UpToPart { body: plate, offset: 0.0 }, ..RevolveSpec::solid(axis, None) };
    // The almost-full sweep minus the plate.
    let full = k.revolve_with(&prof, &RevolveSpec { full: false, end: RevolveEnd::Angle(std::f64::consts::TAU * (1.0 - 1e-4)), ..RevolveSpec::solid(axis, None) }).unwrap().bodies[0];
    let (a, b) = ((**k.get(full).unwrap()).clone(), (**k.get(plate).unwrap()).clone());
    for op in [BoolOp::Subtract, BoolOp::Intersect] {
        match boolean::boolean_bodies(&a, &b, op) {
            Ok(r) => println!("{op:?} ok faces {} solids {}", r.body.faces.len(), r.body.solids.len()),
            Err(e) => println!("{op:?} err {e}"),
        }
    }
    println!("{:?}", k.revolve_with(&prof, &spec).map(|r| r.bodies.len()));
}

#[test]
fn debug_thin() {
    let prof = Profile::new(Plane::top(), vec![reg(rect(0., 0., 20., 20.))]);
    let bands = cadrs_kernel::thin::thin_profile(&prof, 2.0, 0.0).unwrap();
    for r in &bands.regions {
        let pts: Vec<_> = r.outer.curves.iter().map(|c| { let s = c.start(); (s.x, s.y) }).collect();
        println!("region {:?} holes {} : {:?}", r.source, r.holes.len(), pts);
    }
    let regions = native::profile::regions(&bands).unwrap();
    let m = native::profile::merge_regions(&regions).unwrap();
    println!("merged {} faces: {:?}", m.len(), m.iter().map(|f| (f.outer.len(), f.holes.len(), native::profile::loop_area(&f.outer))).collect::<Vec<_>>());
}

#[test]
fn debug_thin2() {
    let mut k = NativeKernel::new();
    let a = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 20., 2.))]), Extent::Blind(10.0)).unwrap().bodies[0];
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(18., 0., 20., 20.))]), Extent::Blind(10.0)).unwrap().bodies[0];
    let u = k.boolean(BoolOp::Union, a, &[b]).unwrap().bodies[0];
    println!("union vol {} faces {} solids {}", k.mass_properties(u).unwrap().volume, k.faces(u).unwrap().len(), k.solid_count(u).unwrap());
}

#[test]
fn debug_thin3() {
    let mut k = NativeKernel::new();
    let mut spec = ExtrudeSpec::blind(Vector3::z_axis(), 10.0);
    spec.body = BodyKind::Thin { left: 2.0, right: 0.0 };
    let sq = Profile::new(Plane::top(), vec![reg(rect(0., 0., 20., 20.))]);
    let r = k.extrude_with(&sq, &spec);
    match r {
        Ok(r) => { let b = r.bodies[0]; println!("thin vol {} solids {}", k.mass_properties(b).unwrap().volume, k.solid_count(b).unwrap()); }
        Err(e) => println!("thin err {e}"),
    }
}

#[test]
fn debug_split() {
    let mut k = NativeKernel::new();
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 10., 20.))]), Extent::Blind(30.0)).unwrap().bodies[0];
    let plane = Plane { origin: Point3::new(0.0, 0.0, 10.0), ..Plane::top() };
    let r = k.split(b, &SplitTool::Plane(plane), 42).unwrap();
    let s = r.bodies[0];
    let body = k.get(s).unwrap();
    println!("solids {} shells {:?} faces {}", body.solids.len(), body.shells.len(), body.faces.len());
}

#[test]
fn debug_cube_fillet() {
    let mut k = NativeKernel::new();
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 100., 100.))]), Extent::Blind(100.0)).unwrap().bodies[0];
    let all: Vec<EdgeId> = k.edges(b).unwrap().iter().map(|e| e.id).collect();
    for n in 4..=12usize {
        let r = k.fillet_with(b, &all[..n], &FilletSpec::radius(10.0));
        match r {
            Ok(r) => println!("{n} edges: vol {} faces {}", k.mass_properties(r.bodies[0]).unwrap().volume, k.faces(r.bodies[0]).unwrap().len()),
            Err(e) => println!("{n} edges: err {e}"),
        }
    }
    let infos = k.edges(b).unwrap();
    for e in &infos[..3] { println!("edge {:?} {:?} -> {:?}", e.id, e.start, e.end); }
}

#[test]
fn debug_miter() {
    let mut k = NativeKernel::new();
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 100., 100.))]), Extent::Blind(100.0)).unwrap().bodies[0];
    let infos = k.edges(b).unwrap();
    let v = infos.iter().find(|e| e.start.coords.norm() < 1e-9 && e.end.z > 50.0).unwrap().id;
    let h = infos.iter().find(|e| (e.start.coords.norm() < 1e-9 || e.end.coords.norm() < 1e-9) && (e.start.x - e.end.x).abs() > 50.0).unwrap().id;
    println!("v {v:?} h {h:?}");
    let r = k.fillet_with(b, &[v, h], &FilletSpec::radius(10.0));
    match r {
        Ok(r) => println!("miter: vol {} faces {}", k.mass_properties(r.bodies[0]).unwrap().volume, k.faces(r.bodies[0]).unwrap().len()),
        Err(e) => println!("miter: err {e}"),
    }
}

#[test]
fn debug_corner() {
    let mut k = NativeKernel::new();
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 30., 30.))]), Extent::Blind(30.0)).unwrap().bodies[0];
    let v = k.vertices(b).unwrap().into_iter().find(|x| (x.point - Point3::new(30., 30., 30.)).norm() < 1e-9).unwrap();
    let r = k.fillet_with(b, &v.edges, &FilletSpec::radius(3.0));
    match r {
        Ok(r) => println!("corner: vol {} faces {}", k.mass_properties(r.bodies[0]).unwrap().volume, k.faces(r.bodies[0]).unwrap().len()),
        Err(e) => println!("corner: err {e}"),
    }
}

#[test]
fn debug_block_ball() {
    let c = Point3::new(27., 27., 27.);
    let (a, b) = (27.0, 31.5);
    let block = native::blend::hexahedron(&[
        Point3::new(a, a, a), Point3::new(b, a, a), Point3::new(b, b, a), Point3::new(a, b, a),
        Point3::new(a, a, b), Point3::new(b, a, b), Point3::new(b, b, b), Point3::new(a, b, b),
    ]).unwrap();
    let ball = native::blend::ball_body(c, 3.0).unwrap();
    println!("ball verts {:?}", ball.verts.iter().map(|v| v.p).collect::<Vec<_>>());
    match native::boolean::boolean_bodies(&block, &ball, BoolOp::Subtract) {
        Ok(r) => {
            println!("ok faces {} solids {}", r.body.faces.len(), r.body.solids.len());
            for (i, v) in r.body.verts.iter().enumerate() { println!("v{i} {:?}", v.p); }
        }
        Err(e) => println!("err {e}"),
    }
}

#[test]
fn debug_cube7() {
    let mut k = NativeKernel::new();
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 100., 100.))]), Extent::Blind(100.0)).unwrap().bodies[0];
    let all: Vec<EdgeId> = k.edges(b).unwrap().iter().map(|e| e.id).collect();
    for e in &k.edges(b).unwrap()[..7] { eprintln!("edge {:?} {:?} -> {:?}", e.id, e.start, e.end); }
    // Apply one at a time through the blend to see which step opens.
    for n in [5usize, 6, 7] {
        let r = k.fillet_with(b, &all[4..n], &FilletSpec::radius(10.0));
        match r {
            Ok(r) => eprintln!("bottom edges 4..{n}: vol {} solids {}", k.mass_properties(r.bodies[0]).unwrap().volume, k.solid_count(r.bodies[0]).unwrap()),
            Err(e) => eprintln!("bottom edges 4..{n}: err {e}"),
        }
    }
    let mut set = vec![all[1], all[4], all[5]];
    let r = k.fillet_with(b, &set, &FilletSpec::radius(10.0)).unwrap();
    eprintln!("1,4,5: vol {} solids {}", k.mass_properties(r.bodies[0]).unwrap().volume, k.solid_count(r.bodies[0]).unwrap());
    set.push(all[2]);
    let r = k.fillet_with(b, &set, &FilletSpec::radius(10.0)).unwrap();
    eprintln!("1,2,4,5: vol {} solids {}", k.mass_properties(r.bodies[0]).unwrap().volume, k.solid_count(r.bodies[0]).unwrap());
}

#[test]
fn debug_cube_combos() {
    let mut k = NativeKernel::new();
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 100., 100.))]), Extent::Blind(100.0)).unwrap().bodies[0];
    let all: Vec<EdgeId> = k.edges(b).unwrap().iter().map(|e| e.id).collect();
    for extra in [None, Some(0usize), Some(2), Some(3), Some(5)] {
        let mut set = vec![all[1], all[4], all[6]];
        if let Some(x) = extra { set.push(all[x]); }
        match k.fillet_with(b, &set, &FilletSpec::radius(10.0)) {
            Ok(r) => eprintln!("1,4,6 + {extra:?}: vol {} solids {}", k.mass_properties(r.bodies[0]).unwrap().volume, k.solid_count(r.bodies[0]).unwrap()),
            Err(e) => eprintln!("1,4,6 + {extra:?}: err {e}"),
        }
    }
}

#[test]
fn debug_corners_all() {
    let mut k = NativeKernel::new();
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 30., 30.))]), Extent::Blind(30.0)).unwrap().bodies[0];
    for v in k.vertices(b).unwrap() {
        let r = k.fillet_with(b, &v.edges, &FilletSpec::radius(3.0));
        match r {
            Ok(r) => eprintln!("corner {:?}: vol {:.4} solids {}", v.point, k.mass_properties(r.bodies[0]).unwrap().volume, k.solid_count(r.bodies[0]).unwrap()),
            Err(e) => eprintln!("corner {:?}: err {e}", v.point),
        }
    }
}

#[test]
fn debug_corner_tools() {
    let mut k = NativeKernel::new();
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 30., 30.))]), Extent::Blind(30.0)).unwrap().bodies[0];
    let body = (**k.get(b).unwrap()).clone();
    for (vi, v) in body.verts.iter().enumerate() {
        if std::env::var("ONLY0").is_ok() && vi != 0 { continue; }
        match native::blend::corner_tool(&body, vi, 3.0, Origin::StartCap { region: 0 }) {
            Ok(Some((t, _))) => {
                let m = native::mass::body_mass(&t);
                // block 4.5³ minus a ball octant
                eprintln!("tool at {:?}: vol {:.9} (want {:.9}) faces {} solids {}", v.p, m.volume, 4.501f64.powi(3) - std::f64::consts::PI * 27.0 / 6.0, t.faces.len(), t.solids.len());
                for (i, f) in t.faces.iter().enumerate() {
                    if matches!(f.surf, native::surface::Surface::Sphere { .. }) {
                        let q = native::mass::face_integrals(&t, f, &Vector3::zeros());
                        eprintln!("   sphere face {i} area {:.12} want {:.12}", q[0], std::f64::consts::PI * 9.0 / 2.0);
                        for l in &f.loops { for c in l { let e = &t.edges[c.edge]; eprintln!("      edge {:?} deg {} t {:?} pc {:?}", std::mem::discriminant(&e.curve), e.degenerate, e.t, c.pc); } }
                    }
                }
            }
            Ok(None) => eprintln!("tool at {:?}: none", v.p),
            Err(e) => eprintln!("tool at {:?}: err {e}", v.p),
        }
    }
}

#[test]
fn debug_cube_precision() {
    use std::f64::consts::PI;
    let mut k = NativeKernel::new();
    let (l, r) = (30.0f64, 3.0f64);
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., l, l))]), Extent::Blind(l)).unwrap().bodies[0];
    let edge = (1.0 - PI / 4.0) * r * r * l;
    let corner = 3.0 * (1.0 - PI / 4.0) * r.powi(3) - (1.0 - PI / 6.0) * r.powi(3);
    let v = k.vertices(b).unwrap().into_iter().find(|x| (x.point - Point3::new(l, l, l)).norm() < 1e-9).unwrap();
    let one = k.fillet_with(b, &v.edges, &FilletSpec::radius(r)).unwrap().bodies[0];
    eprintln!("corner: {:.9} want {:.9}", k.mass_properties(one).unwrap().volume, l.powi(3) - 3.0 * edge + corner);
    let all: Vec<EdgeId> = k.edges(b).unwrap().iter().map(|e| e.id).collect();
    let f = k.fillet_with(b, &all, &FilletSpec::radius(r)).unwrap().bodies[0];
    let m = k.mass_properties(f).unwrap();
    eprintln!("all: {:.9} want {:.9}, faces {}", m.volume, l.powi(3) - 12.0 * edge + 8.0 * corner, k.faces(f).unwrap().len());
    // Area: 6 faces (L−2r)², 12 quarter cylinders (π r/2)(L−2r), 8 octants 4πr²/8.
    eprintln!("area: {:.9} want {:.9}", m.surface_area, 6.0 * (l - 2.0 * r).powi(2) + 12.0 * PI * r / 2.0 * (l - 2.0 * r) + 4.0 * PI * r * r);
    for (i, fi) in k.faces(f).unwrap().iter().enumerate() { if i < 30 { eprintln!("  face {i} {:?} area {:.9}", fi.kind, fi.area); } }
}

#[test]
fn debug_octant() {
    use std::f64::consts::PI;
    let r = 3.0f64;
    let c = Point3::new(0., 0., 0.);
    for (name, ball) in [
        ("aligned", native::blend::ball_frame(c, r, Vector3::z(), -Vector3::new(1.0, 1.0, 0.0).normalize()).unwrap()),
        ("default", native::blend::ball_body(c, r).unwrap()),
    ] {
        let m = native::mass::body_mass(&ball);
        eprintln!("{name} ball: vol err {:.3e} area err {:.3e}", m.volume - 4.0 / 3.0 * PI * r.powi(3), m.surface_area - 4.0 * PI * r * r);
        let bx = native::blend::hexahedron(&[
            Point3::new(0., 0., 0.), Point3::new(5., 0., 0.), Point3::new(5., 5., 0.), Point3::new(0., 5., 0.),
            Point3::new(0., 0., 5.), Point3::new(5., 0., 5.), Point3::new(5., 5., 5.), Point3::new(0., 5., 5.),
        ]).unwrap();
        let o = native::boolean::boolean_bodies(&ball, &bx, BoolOp::Intersect).unwrap().body;
        let m = native::mass::body_mass(&o);
        eprintln!("{name} octant: vol err {:.3e} area err {:.3e} faces {}", m.volume - PI * r.powi(3) / 6.0, m.surface_area - (PI * r * r / 2.0 + 3.0 * PI * r * r / 4.0), o.faces.len());
        for (i, f) in o.faces.iter().enumerate() {
            let q = native::mass::face_integrals(&o, f, &Vector3::zeros());
            eprintln!("   face {i} {:?} area {:.12} loops {:?}", f.surf.kind(), q[0], f.loops.iter().map(|l| l.iter().map(|c| (o.edges[c.edge].degenerate, format!("{:?}", std::mem::discriminant(&c.pc)))).collect::<Vec<_>>()).collect::<Vec<_>>());
        }
    }
}

#[test]
fn debug_corner_face() {
    let mut k = NativeKernel::new();
    let b = k.extrude(&Profile::new(Plane::top(), vec![reg(rect(0., 0., 30., 30.))]), Extent::Blind(30.0)).unwrap().bodies[0];
    let v = k.vertices(b).unwrap().into_iter().find(|x| (x.point - Point3::new(30., 30., 30.)).norm() < 1e-9).unwrap();
    let r = k.fillet_with(b, &v.edges, &FilletSpec::radius(3.0)).unwrap().bodies[0];
    let body = k.get(r).unwrap();
    for (i, v) in body.verts.iter().enumerate() {
        let off = |x: f64| (x - (x * 2.0).round() / 2.0).abs();
        if off(v.p.x).max(off(v.p.y)).max(off(v.p.z)) > 1e-12 { eprintln!("inexact v{i} {:?}", v.p); }
    }
    for (i, f) in body.faces.iter().enumerate() {
        let q = native::mass::face_integrals(body, f, &Vector3::zeros());
        eprintln!("face {i} {:?} area {:.12}", f.surf.kind(), q[0]);
        if matches!(f.surf, native::surface::Surface::Sphere { .. } | native::surface::Surface::Cylinder { .. }) {
            for l in &f.loops { for c in l { let e = &body.edges[c.edge]; eprintln!("      edge {:?} deg {} t {:?} pc {:?}", e.curve, e.degenerate, e.t, c.pc); } }
        }
    }
}

#[test]
fn debug_concave_conic() {
    let mut k = NativeKernel::new();
    let l = Loop { curves: {
        let pts = [p(0.,0.), p(20.,0.), p(20.,5.), p(5.,5.), p(5.,20.), p(0.,20.)];
        (0..6).map(|i| Curve2::Line { a: pts[i], b: pts[(i+1)%6], source: None }).collect()
    } };
    let lb = k.extrude(&Profile::new(Plane::top(), vec![reg(l)]), Extent::Blind(10.0)).unwrap().bodies[0];
    let inner = k.edges(lb).unwrap().into_iter().find(|e| (e.mid - Point3::new(5.0, 5.0, 5.0)).norm() < 1e-9).unwrap().id;
    for prof in [FilletProfile::Circular, FilletProfile::Conic { rho: 0.5 }] {
        let spec = FilletSpec { profile: prof, ..FilletSpec::radius(2.0) };
        let f = k.fillet_with(lb, &[inner], &spec).unwrap().bodies[0];
        eprintln!("{prof:?}: added {:.9} per mm (circle {:.9}) solids {} faces {}", (k.mass_properties(f).unwrap().volume - 1750.0) / 10.0, 1.0 - std::f64::consts::PI / 4.0 * 4.0 / 4.0, k.solid_count(f).unwrap(), k.faces(f).unwrap().len());
    }
}

#[test]
fn debug_sweep() {
    use std::f64::consts::PI;
    let mut k = NativeKernel::new();
    let prof = Profile::new(Plane { origin: Point3::origin(), x_dir: Vector3::y_axis(), normal: Vector3::x_axis() }, vec![Region { outer: Loop { curves: vec![Curve2::Circle { center: p(0., 0.), radius: 5.0, source: None }] }, holes: vec![], source: Some(7) }]);
    let plane = Plane::top();
    let l = |a, b| PathCurve::Sketch { plane, curve: Curve2::Line { a, b, source: None } };
    let paths = vec![
        ("line", vec![l(p(0., 0.), p(100., 0.))]),
        ("arc", vec![PathCurve::Sketch { plane, curve: Curve2::Arc { center: p(0., 50.), radius: 50.0, start_angle: -PI / 2.0, sweep: PI / 2.0, source: None } }]),
        ("L", vec![l(p(0., 0.), p(100., 0.)), PathCurve::Sketch { plane, curve: Curve2::Arc { center: p(100., 50.), radius: 50.0, start_angle: -PI / 2.0, sweep: PI / 2.0, source: None } }, l(p(150., 50.), p(150., 130.))]),
    ];
    for (name, path) in paths {
        let spec = SweepSpec { body: BodyKind::Solid, path, control: SweepControl::None, faces: vec![] };
        match k.sweep_with(&prof, &spec) {
            Ok(r) => { let b = r.bodies[0]; eprintln!("{name}: vol {:.6} solids {} faces {} check {:?}", k.mass_properties(b).unwrap().volume, k.solid_count(b).unwrap(), k.faces(b).unwrap().len(), k.get(b).unwrap().check()); }
            Err(e) => eprintln!("{name}: err {e}"),
        }
    }
}
