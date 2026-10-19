//! NVIDIA usd-brep's OCCT BRep corner cases (seams, poles, natural bounds, sparse or
//! inconsistent p-curves, two-turn periodic faces; `tests/data/usd_brep`, Apache-2.0): each
//! reads to a valid body with the faces, solids, volume and area recorded here. Values with a
//! closed form are marked with it; the others were checked by hand once (the two-turn cylinder
//! band: 2 × its mean height 0.19 × 4π) and are regression values.

use cadrs_kernel::native::{brep_occt, mass};

#[test]
fn usd_brep_occt_files_read_right() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/usd_brep");
    // (file, faces, solids, volume, area)
    let cases: &[(&str, usize, usize, f64, f64)] = &[
("bezier_surface", 1, 0, 0.000000, 21.439262),
    ("box", 6, 1, 24.000000, 52.000000), // (2×3×4: V 24, A 52)
    ("box_sparse_pcurves", 6, 1, 24.000000, 52.000000),
    ("bspline_surface", 1, 0, 0.000000, 21.141890),
    ("cone", 3, 1, 21.991149, 45.511728),
    ("cone_full", 2, 1, 12.566371, 35.220717),
    ("cone_u_seam", 1, 0, 0.000000, 8.885766),
    ("cylinder", 3, 1, 28.274334, 51.836279),
    ("cylinder_patch", 1, 0, 0.000000, 9.424778),
    ("cylinder_two_turn", 1, 0, 0.000000, 4.763520),
    ("cylinder_u_seam", 1, 0, 0.000000, 9.424778),
    ("cylinder_u_seam_no_pcurves", 1, 0, 0.000000, 9.424778),
    ("extrusion", 1, 0, 0.000000, 34.505286),
    ("hyperbola", 1, 0, 0.000000, 1.626860),
    ("parabola", 1, 0, 0.000000, 2.666667),
    ("periodic_bspline", 1, 0, 0.000000, 31.349438),
    ("plane", 1, 0, 0.000000, 4.000000),
    ("plane_inconsistent_pcurves", 1, 0, 0.000000, 4.000000),
    ("plate_with_hole", 1, 0, 0.000000, 20.858407),
    ("revolution", 1, 0, 0.000000, 13.108703),
    ("sphere", 1, 1, 33.510322, 50.265482), // (r 2: V 32π/3, A 16π)
    ("sphere_hole", 1, 0, 0.000000, 45.238934),
    ("sphere_natural", 1, 0, 0.000000, 50.265482), // (the whole sphere, r 2: 16π)
    ("sphere_partial_full_uv_extent", 1, 0, 0.000000, 12.566371), // (a quarter of the r 2 sphere: 4π)
    ("sphere_u_seam", 1, 0, 0.000000, 6.283185),
    ("torus", 1, 1, 113.697843, 189.496405), // (R 4, r 1.2: V 2π²Rr², A 4π²Rr)
    ("torus_natural", 1, 0, 0.000000, 189.496405), // (the whole torus: 4π²Rr)
    ("torus_u_seam", 1, 0, 0.000000, 11.440401),
    ("torus_v_seam", 1, 0, 0.000000, 12.091046),
    ];
    let mut bad = Vec::new();
    for &(name, faces, solids, volume, area) in cases {
        let bytes = std::fs::read(format!("{dir}/{name}.brep")).unwrap();
        let b = brep_occt::read(&bytes).map(|(b, _)| b).unwrap_or_else(|e| panic!("{name}: {e}"));
        let m = mass::body_mass(&b);
        let close = |a: f64, b: f64| (a - b).abs() <= 1e-5 * (1.0 + b.abs());
        if b.faces.len() != faces || b.solids.len() != solids || !b.check().is_empty() || !close(m.volume, volume) || !close(m.surface_area, area) {
            bad.push(format!("{name}: faces {} solids {} check {:?} volume {} area {}", b.faces.len(), b.solids.len(), b.check().first(), m.volume, m.surface_area));
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}
