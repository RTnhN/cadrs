//! Property tests: no input panics a reader or a B-spline constructor. Each reader gets valid
//! files with random edits (numbers swapped for extreme ones, bytes flipped, deleted, inserted,
//! the file cut short) and must return a body or an error, never panic. The B-spline
//! constructors get random degrees, knots, weights and points; what they accept must evaluate.
//!
//! `PROPTEST_CASES=<n>` runs more cases (the default keeps the suite to a few seconds);
//! `FUZZ_TRACE=1` prints each case before it runs.

use cadrs_kernel::native::math::P3;
use cadrs_kernel::native::nurbs::{NurbsCurve, NurbsSurface, H};
use cadrs_kernel::native::{bodyio, brep_occt, iges, smb, step, NativeKernel};
use cadrs_kernel::exchange::ExchangeFormat;
use cadrs_kernel::*;
use proptest::prelude::*;

fn data(rel: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data").join(rel)).unwrap()
}

/// An SMLib B-rep to edit (its fixtures moved into per-suite folders).
fn smb_seed() -> Vec<u8> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/smlib");
    ["fillets/shell_fillet.smb", "shell_fillet.smb"].iter().find_map(|f| std::fs::read(dir.join(f)).ok()).expect("an SMLib fixture")
}

/// Numbers that break careless parsers: huge counts, negatives where counts go, non-finite.
const NASTY: &[&str] =
    &["0", "-1", "1e308", "-1e308", "1e-320", "NaN", "inf", "-inf", "4294967296", "18446744073709551615", "99999999", "0.", "1.5", "-0.", "", "#0", "#999999", "''", "$", "*"];

/// One edit of a file.
#[derive(Debug, Clone)]
enum Edit {
    /// The `k`-th number (a run of digits) replaced by a nasty one.
    Number(usize, usize),
    Byte(usize, u8),
    Delete(usize, usize),
    Insert(usize, usize),
    Truncate(usize),
}

fn edit() -> impl Strategy<Value = Edit> {
    prop_oneof![
        4 => (any::<usize>(), 0..NASTY.len()).prop_map(|(k, n)| Edit::Number(k, n)),
        2 => (any::<usize>(), any::<u8>()).prop_map(|(i, b)| Edit::Byte(i, b)),
        1 => (any::<usize>(), 1usize..64).prop_map(|(i, n)| Edit::Delete(i, n)),
        1 => (any::<usize>(), 0..NASTY.len()).prop_map(|(i, n)| Edit::Insert(i, n)),
        1 => any::<usize>().prop_map(Edit::Truncate),
    ]
}

fn apply(mut b: Vec<u8>, edits: &[Edit]) -> Vec<u8> {
    // `FUZZ_TRACE=1`: each case's edits on stderr before it runs (to find a case that hangs).
    if std::env::var_os("FUZZ_TRACE").is_some() {
        eprintln!("case {edits:?}");
    }
    for e in edits {
        if b.is_empty() {
            break;
        }
        match *e {
            Edit::Number(k, n) => {
                // The starts of the digit runs.
                let starts: Vec<usize> = (0..b.len()).filter(|&i| b[i].is_ascii_digit() && (i == 0 || !b[i - 1].is_ascii_digit())).collect();
                if let Some(&s) = starts.get(k % starts.len().max(1)) {
                    let end = (s..b.len()).find(|&i| !b[i].is_ascii_digit()).unwrap_or(b.len());
                    b.splice(s..end, NASTY[n].bytes());
                }
            }
            Edit::Byte(i, v) => {
                let i = i % b.len();
                b[i] = v;
            }
            Edit::Delete(i, n) => {
                let i = i % b.len();
                b.drain(i..(i + n).min(b.len()));
            }
            Edit::Insert(i, n) => {
                let i = i % (b.len() + 1);
                b.splice(i..i, NASTY[n].bytes());
            }
            Edit::Truncate(i) => b.truncate(i % b.len()),
        }
    }
    b
}

fn edits() -> impl Strategy<Value = Vec<Edit>> {
    proptest::collection::vec(edit(), 1..6)
}

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig { cases: std::env::var("PROPTEST_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(cases), ..ProptestConfig::default() }
}

/// A plate with a hole (planes, a cylinder, lines and circles), to be written by our own
/// exporters.
fn model() -> (NativeKernel, Vec<(BodyId, String)>) {
    use nalgebra::Point2;
    let mut k = NativeKernel::new();
    let p = |x: f64, y: f64| Point2::new(x, y);
    let c = [p(0.0, 0.0), p(20.0, 0.0), p(20.0, 10.0), p(0.0, 10.0)];
    let outer = Loop { curves: (0..4).map(|i| Curve2::Line { a: c[i], b: c[(i + 1) % 4], source: None }).collect() };
    let hole = Loop { curves: vec![Curve2::Circle { center: p(5.0, 5.0), radius: 2.0, source: None }] };
    let body = k.extrude(&Profile::new(Plane::top(), vec![Region { outer, holes: vec![hole], source: None }]), Extent::Blind(4.0)).unwrap().bodies[0];
    (k, vec![(body, "plate".into())])
}

fn step_file() -> Vec<u8> {
    let (k, parts) = model();
    k.export_model(ExchangeFormat::Step, "fuzz", &parts, &[]).unwrap()
}

fn iges_file() -> Vec<u8> {
    let (k, parts) = model();
    k.export_model(ExchangeFormat::Iges, "fuzz", &parts, &[]).unwrap()
}

fn body_file() -> Vec<u8> {
    let (k, parts) = model();
    cadrs_kernel::native::exchange::write_body(&k, parts[0].0).unwrap()
}

/// Bytes that read to a body must also take the kernel's queries on it without panicking.
fn use_body(b: native::topo::Body) {
    let _ = b.check();
}

/// The unedited files read, so the edits start from real data.
#[test]
fn seeds_read() {
    assert!(!step::read(&step_file()).unwrap().parts.is_empty());
    assert!(!iges::read(&iges_file()).unwrap().parts.is_empty());
    bodyio::read(&body_file()).unwrap();
    for f in ["occt_box.brep", "usd_brep/bspline_surface.brep", "usd_brep/torus.brep"] {
        brep_occt::read(&data(f)).unwrap();
    }
    smb::read(&smb_seed()).unwrap();
}

proptest! {
    #![proptest_config(config(96))]

    #[test]
    fn step_reader_never_panics(e in edits()) {
        static FILE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        let bytes = apply(FILE.get_or_init(step_file).clone(), &e);
        if let Ok(m) = step::read(&bytes) {
            m.parts.into_iter().for_each(|p| use_body(p.body));
        }
    }

    #[test]
    fn iges_reader_never_panics(e in edits()) {
        static FILE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        let bytes = apply(FILE.get_or_init(iges_file).clone(), &e);
        if let Ok(m) = iges::read(&bytes) {
            m.parts.into_iter().for_each(|p| use_body(p.body));
        }
    }

    #[test]
    fn body_reader_never_panics(e in edits(), raw in proptest::collection::vec((any::<usize>(), any::<u8>()), 0..8)) {
        static FILE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        // A binary format: flip bytes anywhere as well.
        let mut bytes = apply(FILE.get_or_init(body_file).clone(), &e);
        for (i, v) in raw {
            if !bytes.is_empty() {
                let n = bytes.len();
                bytes[i % n] = v;
            }
        }
        if let Ok(b) = bodyio::read(&bytes) {
            use_body(b);
        }
    }

    #[test]
    fn brep_reader_never_panics(e in edits(), which in 0usize..3) {
        let file = ["occt_box.brep", "usd_brep/bspline_surface.brep", "usd_brep/torus.brep"][which];
        if std::env::var_os("FUZZ_TRACE").is_some() {
            eprintln!("file {file}");
        }
        let bytes = apply(data(file), &e);
        if let Ok((b, _)) = brep_occt::read(&bytes) {
            use_body(b);
        }
    }

    #[test]
    fn smb_reader_never_panics(e in edits(), raw in proptest::collection::vec(any::<u8>(), 0..256)) {
        let bytes = apply(smb_seed(), &e);
        if let Ok((b, _)) = smb::read(&bytes) {
            use_body(b);
        }
        let _ = smb::read(&raw);
    }

    #[test]
    fn readers_take_any_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
        let _ = step::read(&bytes);
        let _ = iges::read(&bytes);
        let _ = bodyio::read(&bytes);
        let _ = brep_occt::read(&bytes);
    }
}

fn num() -> impl Strategy<Value = f64> {
    prop_oneof![
        8 => -10.0f64..10.0,
        1 => Just(f64::NAN),
        1 => Just(f64::INFINITY),
        1 => Just(0.0),
        1 => Just(-1.0),
    ]
}

/// A knot vector: mostly sorted (so some are valid), sometimes not.
fn knots() -> impl Strategy<Value = Vec<f64>> {
    (proptest::collection::vec(num(), 0..14), any::<bool>()).prop_map(|(mut k, sort)| {
        if sort {
            k.sort_by(f64::total_cmp);
        }
        k
    })
}

fn pole() -> impl Strategy<Value = H> {
    (num(), num(), num(), prop_oneof![4 => 0.1f64..4.0, 1 => num()]).prop_map(|(x, y, z, w)| H::new(x * w, y * w, z * w, w))
}

/// Everything a valid curve offers must work on it.
fn exercise(c: &NurbsCurve) {
    let (a, b) = c.range();
    for t in [a, b, 0.5 * (a + b), a - 1.0, b + 1.0] {
        let _ = c.point(t);
        let _ = c.ders(t, 2);
    }
    let _ = c.reversed();
    let _ = c.breaks();
    let m = 0.5 * (a + b);
    if m > a && m < b {
        let _ = c.split(m);
        let _ = c.insert_knot(m);
    }
    let _ = c.elevated();
}

proptest! {
    #![proptest_config(config(512))]

    #[test]
    fn nurbs_curve_constructor_never_panics(degree in 0usize..6, knots in knots(), cps in proptest::collection::vec(pole(), 0..10)) {
        if let Ok(c) = NurbsCurve::new(degree, knots, cps) {
            exercise(&c);
        }
    }

    /// Valid knots from multiplicities: the constructor accepts them, and they evaluate.
    #[test]
    fn valid_nurbs_curves_are_accepted(degree in 1usize..5, inner in proptest::collection::vec((0.01f64..0.99, 1usize..3), 0..5), poles in proptest::collection::vec((num(), num(), num(), 0.1f64..4.0), 12)) {
        let mut inner = inner;
        inner.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut k = vec![0.0; degree + 1];
        for (u, m) in inner {
            k.extend(std::iter::repeat_n(u, m.min(degree)));
        }
        k.extend(vec![1.0; degree + 1]);
        let n = k.len() - degree - 1;
        let cps: Vec<H> = poles.iter().cycle().take(n).map(|&(x, y, z, w)| H::new(x * w, y * w, z * w, w)).collect();
        let finite = cps.iter().all(|c| c.iter().all(|x| x.is_finite()));
        let c = NurbsCurve::new(degree, k, cps);
        prop_assert_eq!(c.is_ok(), finite);
        if let Ok(c) = c {
            exercise(&c);
            prop_assert!(c.point(0.5).iter().all(|x| x.is_finite()));
        }
    }

    #[test]
    fn bezier_constructor_never_panics(poles in proptest::collection::vec((num(), num(), num()), 0..6), weights in proptest::collection::vec(num(), 0..6)) {
        let poles: Vec<P3> = poles.into_iter().map(|(x, y, z)| P3::new(x, y, z)).collect();
        if let Ok(c) = NurbsCurve::try_bezier(&poles, &weights) {
            exercise(&c);
        }
    }

    #[test]
    fn nurbs_surface_constructor_never_panics(du in 0usize..4, dv in 0usize..4, ku in knots(), kv in knots(), rows in proptest::collection::vec(proptest::collection::vec(pole(), 0..6), 0..6)) {
        if let Ok(s) = NurbsSurface::new(du, dv, ku, kv, rows) {
            let ((u0, u1), (v0, v1)) = s.range();
            for (u, v) in [(u0, v0), (u1, v1), (0.5 * (u0 + u1), 0.5 * (v0 + v1))] {
                let _ = s.ders2(u, v);
            }
            let _ = s.breaks();
        }
    }
}

/// Reads the file `FUZZ_REPRO` names with every reader (to replay a case that failed).
#[test]
#[ignore = "replays FUZZ_REPRO"]
fn replay() {
    let Some(path) = std::env::var_os("FUZZ_REPRO") else { return };
    let bytes = std::fs::read(path).unwrap();
    eprintln!("brep: {:?}", brep_occt::read(&bytes).map(|(b, w)| (b.faces.len(), w)));
    eprintln!("step: {:?}", step::read(&bytes).map(|m| m.parts.len()));
    eprintln!("iges: {:?}", iges::read(&bytes).map(|m| m.parts.len()));
    eprintln!("smb: {:?}", smb::read(&bytes).map(|(b, w)| (b.faces.len(), w)));
    eprintln!("body: {:?}", bodyio::read(&bytes).map(|b| b.faces.len()));
}

/// Inputs the fuzzing found that once panicked, kept as plain cases.
#[test]
fn smb_reader_regressions() {
    // "1e-320" put into the file: a closed spline's restart parameter is not a number, which
    // NurbsCurve::split unwrapped (now a typed error, and the edge is dropped).
    let cases: &[&[Edit]] = &[&[Edit::Insert(1562873202180017185, 4)]];
    for edits in cases {
        let bytes = apply(smb_seed(), edits);
        if let Ok((b, _)) = smb::read(&bytes) {
            use_body(b);
        }
    }
}
