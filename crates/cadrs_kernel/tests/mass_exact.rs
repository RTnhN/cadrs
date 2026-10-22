//! Mass properties against dense independent references: the exact integrals must hold to
//! near rounding, whatever the faces' surfaces.

use cadrs_kernel::native::NativeKernel;
use cadrs_kernel::native::curve::Curve;
use cadrs_kernel::native::draw::Shape;
use cadrs_kernel::native::mass;
use cadrs_kernel::native::surface::Surface;

/// A prism 7 high over a profile bounded by a cubic B-spline of 1000 spans (a wavy top, as an
/// offset curve fitted through many points has) and three lines. Its volume is the profile's area × 7 and its
/// B-spline side's area the curve's length × 7, both taken from 200 000 points of the curve.
/// The side's integral along the curve was cut in four pieces whatever the curve's knots (the
/// side of SMLib's offset_06 inset came out 3e-3 small).
#[test]
fn extrusion_of_a_many_span_spline() {
    let n = 1003;
    let poles: Vec<[f64; 4]> = (0..n)
        .map(|i| {
            [
                i as f64 * 0.02,
                3.0 + 0.2 * (i as f64 * 0.37).sin() + 0.05 * (i as f64 * 1.3).cos(),
                0.0,
                1.0,
            ]
        })
        .collect();
    let spans = n - 3;
    let knots: Vec<f64> = (0..=spans)
        .map(|i| i as f64 + 0.3 * ((i * 7) % 5) as f64 / 5.0)
        .collect();
    let mut mults = vec![1; spans + 1];
    mults[0] = 4;
    mults[spans] = 4;
    let mut k = NativeKernel::new();
    let c = k.bsplinecurve(3, &knots, &mults, &poles, false).unwrap();
    let top = k.mkedge_full(&c);
    let (a, b) = (poles[0], poles[n - 1]);
    let rest = k
        .polyline(&[
            [b[0], b[1], 0.0],
            [b[0], -1.0, 0.0],
            [a[0], -1.0, 0.0],
            [a[0], a[1], 0.0],
        ])
        .unwrap();
    let w = k.wire(&[&top, &rest]).unwrap();
    let f = k.mkplane(&w).unwrap();
    let Shape::Body(id) = k.prism(&f, 0.0, 0.0, 7.0).unwrap() else {
        panic!("not a body")
    };
    let body = k.get(id).unwrap();
    // The references: the polygon through dense samples, its length extrapolated (a chord falls
    // short of its arc by about h²κ²/24 of it: from m and m/2 chords, Richardson).
    let (t0, t1) = (knots[0], knots[spans]);
    let m = 200_000;
    let pts: Vec<(f64, f64)> = (0..=m)
        .map(|i| c.point(t0 + (t1 - t0) * i as f64 / m as f64))
        .map(|p| (p.x, p.y))
        .collect();
    let chords = |step: usize| -> f64 {
        (0..m / step)
            .map(|i| {
                ((pts[(i + 1) * step].0 - pts[i * step].0).powi(2)
                    + (pts[(i + 1) * step].1 - pts[i * step].1).powi(2))
                .sqrt()
            })
            .sum()
    };
    let len = chords(1) + (chords(1) - chords(2)) / 3.0;
    // (The polygon's area falls short like its length: extrapolated the same way.)
    let polygon = |step: usize| -> f64 {
        let mut ring: Vec<(f64, f64)> = (0..=m / step).map(|i| pts[i * step]).collect();
        ring.extend([(b[0], -1.0), (a[0], -1.0)]);
        0.5 * (0..ring.len())
            .map(|i| {
                ring[i].0 * ring[(i + 1) % ring.len()].1 - ring[(i + 1) % ring.len()].0 * ring[i].1
            })
            .sum::<f64>()
            .abs()
    };
    let area = polygon(1) + (polygon(1) - polygon(2)) / 3.0;
    let vol = mass::body_mass(body).volume;
    assert!(
        (vol - area * 7.0).abs() < 1e-9 * area * 7.0,
        "volume {vol} vs {}",
        area * 7.0
    );
    let side: f64 = (0..body.faces.len())
        .filter(|&fi| !matches!(body.faces[fi].surf, Surface::Plane { .. }))
        .map(|fi| mass::face_area(body, fi).0)
        .sum();
    assert!(
        (side - len * 7.0).abs() < 1e-9 * len * 7.0,
        "side {side} vs {}",
        len * 7.0
    );
}

/// The side of that B-spline as an extrusion (the surface an offset of one makes: offset_06's
/// inset side, 1025 spans), a sheet 7 high: its area is the curve's length × 7.
#[test]
fn extruded_many_span_spline_face() {
    use cadrs_kernel::native::math::{Affine, P3, V3};
    use cadrs_kernel::native::xbuild::{XBuilder, XFace};
    let n = 1003;
    let poles: Vec<[f64; 4]> = (0..n)
        .map(|i| {
            [
                i as f64 * 0.02,
                3.0 + 0.2 * (i as f64 * 0.37).sin() + 0.05 * (i as f64 * 1.3).cos(),
                0.0,
                1.0,
            ]
        })
        .collect();
    let spans = n - 3;
    let knots: Vec<f64> = (0..=spans)
        .map(|i| i as f64 + 0.3 * ((i * 7) % 5) as f64 / 5.0)
        .collect();
    let mut mults = vec![1; spans + 1];
    mults[0] = 4;
    mults[spans] = 4;
    let k = NativeKernel::new();
    let c = k.bsplinecurve(3, &knots, &mults, &poles, false).unwrap();
    let (t0, t1) = (knots[0], knots[spans]);
    let up = Affine {
        m: nalgebra::Matrix3::identity(),
        t: V3::new(0.0, 0.0, 7.0),
    };
    let mut xb = XBuilder::new();
    let (a, b) = (c.point(t0), c.point(t1));
    let v = [
        xb.vertex(a),
        xb.vertex(b),
        xb.vertex(b + V3::z() * 7.0),
        xb.vertex(a + V3::z() * 7.0),
    ];
    let bottom = xb.edge(c.clone(), v[0], v[1]).unwrap();
    let right = xb
        .edge(Curve::line(b, b + V3::z() * 7.0).0, v[1], v[2])
        .unwrap();
    let top = xb.edge(c.transformed(&up), v[3], v[2]).unwrap();
    let left = xb
        .edge(Curve::line(a, a + V3::z() * 7.0).0, v[0], v[3])
        .unwrap();
    let f = xb.face(XFace {
        surf: Surface::Extrusion {
            c: c.clone(),
            d: V3::z(),
        },
        sense: true,
        loops: vec![vec![
            (bottom, true),
            (right, true),
            (top, false),
            (left, false),
        ]],
    });
    let sh = xb.shell(vec![f], false);
    let _ = sh;
    let (mut body, _) = xb.finish();
    assert!(
        matches!(body.faces[0].surf, Surface::Extrusion { .. }),
        "{:?}",
        body.faces[0].surf.kind()
    );
    // The spline edges' p-curves as the straight lines they are (u is the curve's parameter, as
    // the edges'), as a boolean leaves them: the integral along them is then cut only where the
    // surface needs it.
    let (bottom_e, top_e) = (
        body.faces[0].loops[0][0].edge,
        body.faces[0].loops[0][2].edge,
    );
    for co in body.faces[0].loops[0].iter_mut() {
        if co.edge == bottom_e || co.edge == top_e {
            let v = if co.edge == top_e { 7.0 } else { 0.0 };
            co.pc = Curve::Line {
                o: P3::new(0.0, v, 0.0),
                d: V3::x(),
            };
        }
    }
    assert!(body.check().is_empty(), "{:?}", body.check());
    let m = 200_000;
    let pts: Vec<P3> = (0..=m)
        .map(|i| c.point(t0 + (t1 - t0) * i as f64 / m as f64))
        .collect();
    let chords = |step: usize| -> f64 {
        (0..m / step)
            .map(|i| (pts[(i + 1) * step] - pts[i * step]).norm())
            .sum()
    };
    let len = chords(1) + (chords(1) - chords(2)) / 3.0;
    let area = mass::face_area(&body, 0).0.abs();
    assert!(
        (area - len * 7.0).abs() < 1e-9 * len * 7.0,
        "area {area} vs {}",
        len * 7.0
    );
}

/// `body_volume` (the volume's integrand alone, which booleans and offsets weigh solids by, on
/// the fewer points it needs: exact on polynomial B-spline spans) is `body_mass`'s volume to
/// rounding, over every kind of face: analytic, B-spline (trimmed and iso-rectangles), extrusions
/// and revolutions, on SMLib's models.
#[test]
fn volume_alone_matches_mass_properties() {
    use cadrs_kernel::native::smb;
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/smlib");
    let mut seen = 0;
    for sub in ["fillets", "offset", "booleans"] {
        let mut files: Vec<_> = std::fs::read_dir(format!("{dir}/{sub}"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "smb"))
            .collect();
        files.sort();
        for p in files {
            let Ok(b) = smb::read_brep(&std::fs::read(&p).unwrap()) else {
                continue;
            };
            let (full, alone) = (mass::body_mass(&b.body).volume, mass::body_volume(&b.body));
            assert!(
                (full - alone).abs() <= 1e-11 * full.abs().max(1.0),
                "{}: {full} vs {alone}",
                p.display()
            );
            seen += 1;
        }
    }
    assert!(seen > 50, "only {seen} models read");
}
