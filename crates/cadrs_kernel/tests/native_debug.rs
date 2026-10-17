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
