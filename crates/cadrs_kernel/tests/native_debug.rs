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
