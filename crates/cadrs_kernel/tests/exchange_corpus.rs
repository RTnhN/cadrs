//! The STEP and IGES corpus: every `.stp`/`.step` and `.igs`/`.iges` file of OCCT's public
//! dataset (`step/` and `iges/` of opencascade-dataset-7.8.0, at `$CADRS_OCCT_DATA`, else
//! `../occt-dataset/opencascade-dataset-7.8.0` beside the checkout) read by the native readers
//! and checked as `native::exchange`'s `read_a_corpus` does: valid topology (edges off their
//! faces within the file's own tolerance aside), a positive volume per solid equal to its
//! tessellation's within 1 % and unchanged by a move, and no face dropped.
//!
//! Two-sided, like the generated suites' known failures: a file not in [`KNOWN_BAD`] must read
//! and check clean (a regression otherwise), and a file in it must still fail (stale otherwise).
//! The README's count of files read is this test's. It needs the dataset (the gate and the nightly
//! CI job have it; CI's per-push subset doesn't run this binary).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use cadrs_kernel::native::exchange::split_into_solids;
use cadrs_kernel::native::math::Affine;
use cadrs_kernel::native::{iges, mass, step, tess};

/// The files that don't read clean today, by name, with what they hit (the first problem the
/// check reports). Many are OCCT's own bug-report files, broken on purpose.
const KNOWN_BAD: &[(&str, &str)] = &[
    ("iges/BUC50025.igs", "read: the file holds no B-rep solids, shells or trimmed surfaces"),
    ("iges/CTS17806-38038.igs", "2 check errors (edge 2: end 0 is 6.501e-4 from its vertex)"),
    ("iges/Disc.igs", "read: not an IGES file (no directory or parameter section)"),
    ("iges/OCC131-1.igs", "1 check errors (face 0 loop 0 coedge 2 (edge 2): p-curve off the edge by 1.296e-3)"),
    ("iges/OCC131-2.igs", "9 check errors (face 0 loop 0: p-curve gap after coedge 0: (0.000e0, -9.766e-2))"),
    ("iges/OCC131-3.igs", "4 check errors (face 1 loop 0 coedge 1 (edge 1): p-curve off the edge by 2.066e-3)"),
    ("iges/OCC131-6.igs", "2 check errors (face 0 loop 0 coedge 0 (edge 0): p-curve off the edge by 3.104e-3)"),
    ("iges/OCC17026-1.igs", "read: the file holds no B-rep solids, shells or trimmed surfaces"),
    ("iges/OCC17026-2.igs", "read: the file holds no B-rep solids, shells or trimmed surfaces"),
    ("iges/OCC26138_Torus.igs", "4 check errors (face 0 loop 0 coedge 0 (edge 0): p-curve off the edge by 7.405e-2)"),
    ("iges/PRO19800.igs", "read: the file holds no B-rep we can read (face 0: loops wrap around the surface and no seam could be added)"),
    ("iges/PRO20358-X28.igs", "4 check errors (edge 1: end 0 is 2.276e-3 from its vertex)"),
    ("iges/PRO20359-L1.igs", "1 check errors (edge 7: end 0 is 8.503e-4 from its vertex)"),
    ("iges/PRO20359-L11.igs", "2 check errors (edge 2: end 0 is 1.567e-4 from its vertex)"),
    ("iges/PRO20359-L12.igs", "2 check errors (edge 3: end 0 is 1.701e-4 from its vertex)"),
    ("iges/PRO20359-L13.igs", "1 check errors (edge 4: end 0 is 6.166e-4 from its vertex)"),
    ("iges/PRO20359-L15.igs", "3 check errors (face 0 loop 0 coedge 2 (edge 2): p-curve off the edge by 1.195e-3)"),
    ("iges/PRO20359-L2.igs", "2 check errors (edge 4: end 0 is 1.000e-3 from its vertex)"),
    ("iges/PRO20359-L6.igs", "1 check errors (edge 4: end 0 is 4.151e-6 from its vertex)"),
    ("iges/PRO20360-T1.igs", "1 check errors (edge 2: end 0 is 2.000e-4 from its vertex)"),
    ("iges/PRO20360-T2.igs", "1 check errors (edge 3: end 0 is 1.000e-4 from its vertex)"),
    ("iges/PRO20360-T3.igs", "1 check errors (edge 3: end 0 is 1.000e-4 from its vertex)"),
    ("iges/PRO20360-T4.igs", "1 check errors (edge 2: end 0 is 9.968e-5 from its vertex)"),
    ("iges/PRO20360-T6.igs", "1 check errors (edge 4: end 0 is 1.000e-5 from its vertex)"),
    ("iges/UKI60998.igs", "read: the file holds no B-rep we can read (IGES face 9: directory entry 0 is missing)"),
    ("iges/bad_header.igs", "4 check errors (face 0 loop 0 coedge 0 (edge 0): p-curve off the edge by 8.298e0)"),
    ("iges/bug22869_Spring.igs", "12 check errors (face 0 loop 0: p-curve gap after coedge 0: (0.000e0, -7.930e-5))"),
    ("iges/bug23171_tube_surf.igs", "read: the file holds no B-rep we can read (IGES face 3: unsupported IGES surface type 140)"),
    ("iges/bug23591_VZ-387.igs", "read: the file holds no B-rep we can read (IGES face 1: unsupported IGES surface type 143)"),
    ("iges/bug25837_S.igs", "read: the file holds no B-rep solids, shells or trimmed surfaces"),
    ("iges/bug29356.igs", "4 check errors (edge 5: end 0 is 3.014e-6 from its vertex)"),
    ("iges/bug33327.igs", "8 check errors (edge 0 used (1, 0) times in a closed shell); volume 0.000000 vs mesh 9666666.666667; 4 chec…"),
    ("iges/bug8363_f45.igs", "25 check errors (face 0 loop 0: coedge 0 doesn't meet the next)"),
    ("step/PRO7071.stp", "1 check errors (solid 1 encloses a volume of -5.236e5 (its faces point in)); volume -523598.775598 vs mesh…"),
    ("step/as1_motor.step", "8 check errors (face 1 loop 0: coedge 1 doesn't meet the next)"),
    ("step/bm2_ec_exhaust-A.stp", "8 check errors (face 32 loop 0 coedge 3 (edge 67): p-curve off the edge by 5.706e-2)"),
    ("step/bm2_pe_t4-B.stp", "read: the file holds no shapes we can read (#68: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/bug24595_281390.stp", "8 check errors (face 2 loop 0 coedge 0 (edge 6): p-curve off the edge by 3.159e0); volume 26227.500595 vs m…"),
    ("step/bug24983_111FACE.stp", "9 check errors (edge 1 used (1, 0) times in a closed shell); volume -20.251896 vs mesh -25.521671"),
    ("step/bug25013_25013face.stp", "7 check errors (face 0 loop 0: p-curve gap after coedge 3: (2.133e0, -1.167e-1)); volume 0.051232 vs mesh -…"),
    ("step/bug25013_25013faces.stp", "17 check errors (edge 0 used (0, 1) times in a closed shell); volume 32.359538 vs mesh -1068.287699"),
    ("step/bug25415_part_of_hdrv8w64p254_4x2_1016x502h838p.stp", "50 check errors (face 0 loop 0: p-curve gap after coedge 0: (3.063e-1, -1.531e-1)); volume 3.115555 vs mesh…"),
    ("step/bug27333_simple.stp", "read: the file holds no shapes"),
    ("step/bug27453_165-174_piattello.stp", "3 check errors (face 28 loop 0 coedge 0 (edge 332): p-curve off the edge by 1.006e0)"),
    ("step/bug28366_fuselage.stp", "read: the file holds no shapes we can read (#55: unsupported curve B_SPLINE_SURFACE_WITH_KNOTS)"),
    ("step/bug28414_bug_toroidal_surface.stp", "2 check errors (edge 0 used (0, 1) times in a closed shell); volume -166.399606 vs mesh 4633.052811"),
    ("step/bug28454_directives.stp", "read: STEP syntax error at line 5170: expected ',' or ')'"),
    ("step/bug29478.stp", "2 check errors (face 0 loop 0 coedge 1 (edge 1): p-curve off the edge by 5.141e-3)"),
    ("step/bug30790.step", "4 check errors (edge 0 used (1, 0) times in a closed shell); 3936 faces dropped (face 2: a loop doesn't close)"),
    ("step/bug31292.stp", "8 check errors (edge 0 used (0, 1) times in a closed shell); volume 403.650459 vs mesh 442.967014"),
    ("step/bug32239.STEP", "6 check errors (face 510 loop 0 coedge 0 (edge 7): p-curve off the edge by 6.858e-1); 1 check errors (face…"),
    ("step/bug32314_corrupted.stp", "read: STEP syntax error at line 12: expected '('"),
    ("step/bug32596_abc_0_13.step", "12 check errors (edge 23: end 1 is 4.965e-6 from its vertex)"),
    ("step/bug33100_window.step", "read: the file holds no shapes"),
    ("step/bug33398.step", "1 check errors (face 244 loop 0 coedge 3 (edge 761): p-curve off the edge by 1.487e-3)"),
    ("step/ec_exhaust-A.stp", "10 check errors (face 0 loop 0 coedge 2 (edge 1): p-curve off the edge by 2.641e-3); volume 1536723.001863…"),
    ("step/nist_ftc_08_asme1_ap242-2.stp", "read: the file holds no shapes"),
    ("step/pr1db.stp", "1 check errors (face 6 loop 0 coedge 2 (edge 13): p-curve off the edge by 3.250e1); volume 710635.837392 vs…"),
    ("step/r0901_ec.stp", "1 check errors (face 127 loop 0: p-curve: 50 control points of degree 3 need 54 knots, not 52)"),
    ("step/t4mec.stp", "2 check errors (face 0 loop 0 coedge 15 (edge 15): p-curve off the edge by 1.653e-2)"),
    ("step/tr8_ha1_db.stp", "2 faces dropped (#20: unsupported curve PLANE)"),
    ("step/tr8_ha1_ug.stp", "1 faces dropped (#2420: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/tr9_r1001-ec.stp", "51 check errors (face 1 loop 0: p-curve gap after coedge 8: (-3.657e-1, 1.417e-2)); volume 2146570.057259 v…"),
    ("step/trj1_t4m-ac.stp", "read: the file holds no shapes we can read (#160: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/trj3_s1-ac-214.stp", "volume 68064533.436711 vs mesh 62399427.226574; volume 66779304.584408 vs mesh 61106348.057811; 1 check err…"),
    ("step/trj3_s1-al-214.stp", "48 check errors (face 12 loop 0: coedge 0 doesn't meet the next); volume -7769.279961 vs mesh -7769.279961;…"),
    ("step/trj3_s1-ct-203.stp", "volume 82452.533548 vs mesh 71634.631348; volume 2403.676100 vs mesh 2205.891326; 3 faces dropped (face 15:…"),
    ("step/trj3_s1-ct-214.stp", "33 check errors (face 16 loop 0: p-curve gap after coedge 3: (2.599e-5, 7.055e-5)); volume 82452.533548 vs…"),
    ("step/trj3_s1-db-214.stp", "volume 67.376234 vs mesh 185.505032; volume 426.275660 vs mesh 461.436328; volume 67.376234 vs mesh 185.505…"),
    ("step/trj3_s1-ec-203.stp", "4 check errors (face 0 loop 0: coedge 10 doesn't meet the next); volume 180315.593246 vs mesh 218781.734745"),
    ("step/trj3_s1-ec-214.stp", "4 check errors (face 0 loop 0: coedge 10 doesn't meet the next); volume 180315.593246 vs mesh 218781.734745"),
    ("step/trj3_s1-id-203.stp", "1 check errors (face 0 loop 0 coedge 2 (edge 0): p-curve off the edge by 1.471e1); volume 743526.183770 vs…"),
    ("step/trj3_s1-md-214.stp", "1 check errors (solid 0 encloses a volume of -2.224e4 (its faces point in)); volume -22244.570382 vs mesh -…"),
    ("step/trj3_t2-db-214.stp", "2 check errors (face 7 loop 0 coedge 2 (edge 20): p-curve off the edge by 3.155e0)"),
    ("step/trj4_k1_geo-ac-214.stp", "1 faces dropped (#244: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/trj4_k1_geo-md-203.stp", "read: the file holds no shapes we can read (#544: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/trj4_k1_geo-pe-203.stp", "1 faces dropped (#2055: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/trj4_k1_geo-pe-214.stp", "1 faces dropped (#2059: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/trj4_k1_geo-tc-214.stp", "read: the file holds no shapes we can read (#159: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/trj4_k1_geo-tu-214.stp", "read: the file holds no shapes we can read (#69: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/trj4_k1_top-ec-214.stp", "8 check errors (face 5 loop 0: coedge 4 doesn't meet the next)"),
    ("step/trj4_k1_top-id-214.stp", "1 check errors (face 45 loop 0 coedge 9 (edge 104): p-curve off the edge by 1.357e-2)"),
    ("step/trj4_k1_top-md-203.stp", "3 check errors (edge 53: end 1 is 6.721e-5 from its vertex); 1 faces dropped (face 17: loops wrap around th…"),
    ("step/trj4_k1_top-tu-214.stp", "2 faces dropped (face 0: a loop doesn't close)"),
    ("step/trj4_pm2-ec-214.stp", "9 check errors (face 16 loop 0: p-curve gap after coedge 1: (-1.491e0, 1.307e-1)); 4 check errors (face 129…"),
    ("step/trj4_s1-ac-214.stp", "volume 39389.197591 vs mesh 36112.134546; volume 38645.430894 vs mesh 35361.115040; 1 check errors (solid 0…"),
    ("step/trj4_s1-ai-214.stp", "1 check errors (edge 51: end 0 is 1.589e-4 from its vertex)"),
    ("step/trj4_s1-ec-214.stp", "4 check errors (face 0 loop 0: coedge 10 doesn't meet the next); volume 180315.593246 vs mesh 218781.734745"),
    ("step/trj4_xr1-db-214.stp", "read: the file holds no shapes"),
    ("step/trj4_xr1-ec-214.stp", "read: the file holds no shapes"),
    ("step/trj4_xr1-id-214.stp", "read: the file holds no shapes"),
    ("step/trj4_xr1-pe-214.stp", "read: the file holds no shapes"),
    ("step/trj4_xr1-tc-214.stp", "read: the file holds no shapes"),
    ("step/trj4_xr1-tu-214.stp", "read: the file holds no shapes"),
    ("step/trj5_k1-al-214.stp", "10 check errors (face 36 loop 0 coedge 5 (edge 122): p-curve off the edge by 7.620e-1)"),
    ("step/trj5_k1-ec-214.stp", "8 check errors (face 5 loop 0: coedge 4 doesn't meet the next)"),
    ("step/trj5_k1-id-214.stp", "1 check errors (face 45 loop 0 coedge 9 (edge 104): p-curve off the edge by 1.357e-2)"),
    ("step/trj5_k1-md-203.stp", "3 check errors (edge 53: end 1 is 6.721e-5 from its vertex); 1 faces dropped (face 17: loops wrap around th…"),
    ("step/trj5_k1-sh-203.stp", "1 check errors (edge 23 used (0, 1) times in a closed shell); 1 faces dropped (face 43: a loop doesn't close)"),
    ("step/trj5_s1-ac-214.stp", "1 check errors (solid 0 encloses a volume of -2.261e4 (its faces point in)); volume -22609.829712 vs mesh -…"),
    ("step/trj5_s1-in-214.stp", "1 check errors (solid 0 encloses a volume of -1.828e3 (its faces point in)); volume -1827.979158 vs mesh 33…"),
    ("step/trj6_c1-pe-214.stp", "1 faces dropped (#282: unsupported curve AXIS2_PLACEMENT_3D)"),
    ("step/trj6_k1-al-214.stp", "2 check errors (face 1 loop 0 coedge 13 (edge 33): p-curve off the edge by 1.016e0)"),
    ("step/trj6_k1-ct-203.stp", "4 check errors (face 1 loop 1: coedge 0 doesn't meet the next); 1 faces dropped (face 0: a loop doesn't close)"),
    ("step/trj6_k1-ct-214.stp", "4 check errors (face 1 loop 1: coedge 0 doesn't meet the next); 1 faces dropped (face 0: a loop doesn't close)"),
    ("step/trj6_k1-id-203.stp", "1 check errors (face 45 loop 0 coedge 9 (edge 104): p-curve off the edge by 1.357e-2)"),
    ("step/trj6_k1-md-214.stp", "7 check errors (face 15 loop 0 coedge 1 (edge 43): p-curve off the edge by 1.602e-3); 1 faces dropped (face…"),
    ("step/trj6_k1-sd-214.stp", "1 check errors (face 46 loop 0 crosses itself in (u, v) (segments 15 and 27))"),
    ("step/trj6_k1-td-214.stp", "2 check errors (edge 307: end 1 is 4.331e-6 from its vertex)"),
    ("step/trj6_pm4-ec-203.stp", "8 check errors (face 16 loop 0: p-curve gap after coedge 1: (-1.491e0, 1.307e-1))"),
    ("step/trj7_b1-ec-214.stp", "4 check errors (face 200 loop 0: coedge 43 doesn't meet the next)"),
    ("step/trj7_c2-ac-214.stp", "1 faces dropped (#235: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/trj7_c2-db-214.stp", "1 faces dropped (#132: unsupported curve B_SPLINE_SURFACE_WITH_KNOTS)"),
    ("step/trj7_c2-td-203.stp", "1 faces dropped (#208: unsupported curve B_SPLINE_SURFACE_WITH_KNOTS)"),
    ("step/trj7_k1-ac-203.stp", "44 check errors (face 6 loop 0 coedge 3 (edge 15): p-curve off the edge by 2.919e-3)"),
    ("step/trj7_k1-al-214.stp", "3 check errors (face 4 loop 0: p-curve gap after coedge 5: (-1.007e-1, 5.599e-11))"),
    ("step/trj7_k1-id-214.stp", "1 check errors (face 38 loop 0 coedge 10 (edge 89): p-curve off the edge by 1.357e-2)"),
    ("step/trj7_k1-sd-214.stp", "1 check errors (face 47 loop 0 crosses itself in (u, v) (segments 15 and 27))"),
    ("step/trj8_c2-ac-214.stp", "1 faces dropped (#235: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/trj8_k1-ac-203.stp", "44 check errors (face 6 loop 0 coedge 3 (edge 15): p-curve off the edge by 2.919e-3)"),
    ("step/trj8_k1-db-214.stp", "34 check errors (face 5 loop 0 coedge 3 (edge 8): p-curve off the edge by 2.990e-3)"),
    ("step/trj9_ap209-mp.stp", "volume 39999998799.463844 vs mesh 33333333397.669365; volume 39999998799.463844 vs mesh 33333333397.669365;…"),
    ("step/trj9_c2-pe-203.stp", "1 faces dropped (#67: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/trj9_c2-pe-214.stp", "1 faces dropped (#67: unsupported curve CURVE_BOUNDED_SURFACE)"),
    ("step/xr1-db-214_0.stp", "read: the file holds no shapes"),
    ("step/xr1-db-214_1.stp", "read: the file holds no shapes"),
    ("step/xr1-db-214_2.stp", "read: the file holds no shapes"),
    ("step/xr1-db-214_6.stp", "read: the file holds no shapes"),
];

fn dataset() -> PathBuf {
    let root = std::env::var("CADRS_OCCT_DATA").unwrap_or_else(|_| format!("{}/../../../occt-dataset/opencascade-dataset-7.8.0", env!("CARGO_MANIFEST_DIR")));
    let p = PathBuf::from(root);
    assert!(p.join("step").is_dir(), "OCCT's dataset isn't at {}: set CADRS_OCCT_DATA to the dataset's folder", p.display());
    p
}

/// The corpus: the dataset's STEP and IGES files, sorted.
fn files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in ["step", "iges"] {
        let Ok(rd) = std::fs::read_dir(root.join(dir)) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
            if matches!(ext.as_str(), "stp" | "step" | "igs" | "iges") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

/// What a file read to: `Ok(solids)` when it reads and checks clean, else what is wrong.
fn check_file(path: &Path) -> Result<usize, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("unreadable: {e}"))?;
    let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("").to_lowercase();
    let m = if ext.starts_with("ig") { iges::read(&bytes) } else { step::read(&bytes) }.map_err(|e| format!("read: {e}"))?;
    let mut problems = Vec::new();
    let mut solids = 0;
    for p in &m.parts {
        solids += p.body.solids.len();
        // Edges off their faces within the file's own tolerance are not our doing.
        let size = p.body.bbox().diagonal();
        let errs: Vec<String> = p
            .body
            .check()
            .into_iter()
            .filter(|e| !e.split("off the edge by ").nth(1).and_then(|d| d.parse::<f64>().ok()).is_some_and(|d| d < 1e-3f64.max(1e-6 * size)))
            .collect();
        if let Some(e) = errs.first() {
            problems.push(format!("{} check errors ({e})", errs.len()));
        }
        for piece in split_into_solids(&p.body) {
            if piece.solids.is_empty() {
                continue;
            }
            let v = mass::body_mass(&piece).volume;
            let defl = (piece.bbox().diagonal() * 2e-4).max(1e-4);
            let (fm, _) = tess::mesh_body(&piece, defl, 0.1);
            let mv: f64 = fm
                .iter()
                .flat_map(|f| f.indices.iter().map(|t| t.map(|i| f.positions[i as usize].coords)))
                .map(|[a, b, c]| a.dot(&b.cross(&c)) / 6.0)
                .sum();
            if v <= 0.0 || (mv - v).abs() > 1e-2 * v.abs().max(1e-9) {
                problems.push(format!("volume {v:.6} vs mesh {mv:.6}"));
            }
            // A closed, consistently oriented shell's volume doesn't move with it.
            let moved = piece.transformed(&Affine::translation(nalgebra::Vector3::new(1000.0, -700.0, 300.0)));
            let vm = mass::body_mass(&moved).volume;
            if (vm - v).abs() > 1e-6 * v.abs().max(1.0) {
                problems.push(format!("volume {v:.6}, moved {vm:.6}"));
            }
        }
    }
    let dropped: Vec<&String> = m.warnings.iter().filter(|w| w.contains("no seam") || w.contains("no bounds") || w.contains("doesn't close") || w.contains("unsupported")).collect();
    if let Some(w) = dropped.first() {
        problems.push(format!("{} faces dropped ({w})", dropped.len()));
    }
    if problems.is_empty() { Ok(solids) } else { Err(problems.join("; ")) }
}

#[test]
fn occt_dataset_step_and_iges() {
    let root = dataset();
    let all = files(&root);
    assert!(!all.is_empty(), "no STEP or IGES files under {}", root.display());
    // The files on a few threads (the test's own: the gate runs its binaries' tests in parallel).
    let next = AtomicUsize::new(0);
    let got: Mutex<Vec<(usize, Result<usize, String>)>> = Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(2, |n| n.get().min(4));
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(p) = all.get(i) else { break };
                    let r = std::panic::catch_unwind(|| check_file(p)).unwrap_or_else(|_| Err("panicked".into()));
                    got.lock().unwrap_or_else(|e| e.into_inner()).push((i, r));
                }
            });
        }
    });
    let mut got = got.into_inner().unwrap_or_else(|e| e.into_inner());
    got.sort_by_key(|x| x.0);
    let (mut ok, mut with_solids, mut wrong) = (0, 0, Vec::new());
    for (i, r) in &got {
        let name = all[*i].strip_prefix(&root).unwrap_or(&all[*i]).display().to_string();
        let known = KNOWN_BAD.iter().find(|k| k.0 == name);
        match (r, known) {
            (Ok(solids), None) => {
                ok += 1;
                with_solids += usize::from(*solids > 0);
                println!("ok    {name}: {solids} solids");
            }
            (Ok(_), Some(_)) => wrong.push(format!("{name}: a known bad file now reads clean: take it off KNOWN_BAD")),
            (Err(e), Some(_)) => println!("known {name}: {e}"),
            (Err(e), None) => wrong.push(format!("{name}: {e}")),
        }
    }
    println!("{ok} of {} files read and check clean ({with_solids} with solids); {} known bad", all.len(), KNOWN_BAD.len());
    assert!(wrong.is_empty(), "{} files changed:\n  {}", wrong.len(), wrong.join("\n  "));
}
