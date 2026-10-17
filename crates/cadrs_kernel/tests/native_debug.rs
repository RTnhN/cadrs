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
