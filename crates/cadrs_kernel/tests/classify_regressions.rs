//! Point classification against bodies that once gave the wrong answer.

use cadrs_kernel::PointClass;
use cadrs_kernel::native::classify;
use cadrs_kernel::native::math::*;
use cadrs_kernel::native::{bodyio, topo::Body};

fn polys(b: &Body) -> Vec<Vec<Vec<P2>>> {
    b.faces
        .iter()
        .map(|f| classify::face_polygons(b, f))
        .collect()
}

/// The blend tool of OCCT's `tolblend_simple/C9` (two B-spline faces, a third surface and two
/// planes): a point 0.29 behind its nearest faces is inside, though the first ray from it
/// counts an even number of crossings (one missed near a face's boundary). Regression: the
/// small piece of the box face it lies on was kept by the blend's subtraction.
#[test]
fn inside_the_tolblend_c9_tool() {
    let b = bodyio::read(include_bytes!("data/blend_tool_tolblend_c9.body")).unwrap();
    let p = P3::new(490.4947963950842, 3.3560290826370736, 26.104651656457563);
    assert_eq!(
        classify::classify_point(&b, &polys(&b), &p, 1e-6),
        PointClass::Inside
    );
}
