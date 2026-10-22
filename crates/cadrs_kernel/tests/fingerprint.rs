//! Cross-run and cross-machine determinism: a hash of each result's body (the body format writes
//! exact doubles, so the hash covers topology and geometry bit for bit) for a sample of booleans
//! and offsets, computed on rayon pools of 1 and of 4 threads, which must agree.
//!
//! The hashes print as `FP <case> <hash> <faces>` lines and, with `CADRS_FINGERPRINT_OUT=<file>`,
//! are written there: the build box's soak keeps one file per commit, and the same file made on
//! another machine must be identical (a difference is CPU- or build-dependent arithmetic).
//! `CADRS_FINGERPRINT_HEAVY=1` adds slow cases (SMLib's bracket offsets, ~10 s each).

mod meta_common;

use cadrs_kernel::native::{NativeKernel, smb};
use cadrs_kernel::*;
use meta_common::*;

/// One case: a name and the operation, returning its result's bytes and face count.
type Case = (
    String,
    Box<dyn Fn() -> std::result::Result<(Vec<u8>, usize), String> + Sync>,
);

fn bytes(k: &NativeKernel, bodies: &[BodyId]) -> std::result::Result<(Vec<u8>, usize), String> {
    let mut out = Vec::new();
    let mut faces = 0;
    for &b in bodies {
        out.extend(k.write_body(b).map_err(|e| format!("{e:?}"))?);
        faces += k.get(b).map_err(|e| format!("{e:?}"))?.faces().len();
    }
    Ok((out, faces))
}

fn smb_offset(file: &'static str, d: f64) -> std::result::Result<(Vec<u8>, usize), String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/smlib/offset")
        .join(file);
    let data = std::fs::read(&path).map_err(|e| format!("{file}: {e}"))?;
    let b = smb::read_brep(&data)?;
    let mut k = NativeKernel::new();
    let id = k.insert(b.body);
    let spec = OffsetSpec {
        distance: d,
        faces: Vec::new(),
        sharp: false,
    };
    let r = k.offset(id, &spec).map_err(|e| format!("offset: {e:?}"))?;
    bytes(&k, &r.bodies)
}

fn cases() -> Vec<Case> {
    let mut v: Vec<Case> = Vec::new();
    // Booleans of generated pairs (the metamorphic gate's shapes, seed 1).
    let mut r = Rng::new(1);
    for i in 0..12 {
        let a = Placed {
            shape: Shape::Cyl {
                axis: i % 3,
                base: [r.range(2.0, 6.0), r.range(2.0, 6.0), r.range(0.0, 2.0)],
                r: r.range(1.0, 3.0),
                len: r.range(4.0, 9.0),
            },
            axis: [r.range(-1.0, 1.0), r.range(-1.0, 1.0), r.range(-1.0, 1.0)],
            angle: r.range(-0.5, 0.5),
            about: [5.0; 3],
        };
        let b = Placed::fixed(Shape::Box {
            min: [r.range(0.0, 4.0), r.range(0.0, 4.0), r.range(0.0, 4.0)],
            size: [6.0, 5.0, 4.0],
        });
        let op = [BoolOp::Union, BoolOp::Subtract, BoolOp::Intersect][i % 3];
        v.push((
            format!("boolean{i}_{op:?}"),
            Box::new(move || {
                let mut k = NativeKernel::new();
                let ia = build(&mut k, &a).map_err(|e| format!("{e:?}"))?;
                let ib = build(&mut k, &b).map_err(|e| format!("{e:?}"))?;
                let out = boolean(&mut k, op, ia, &[ib]).map_err(|e| format!("{e:?}"))?;
                bytes(&k, &out)
            }),
        ));
    }
    v.push((
        "offset_boss".into(),
        Box::new(|| smb_offset("OffsetBoxWithBoss.smb", 1.5)),
    ));
    if env_num("CADRS_FINGERPRINT_HEAVY", 0u32) != 0 {
        v.push((
            "offset_bracket2_3.5".into(),
            Box::new(|| smb_offset("offset_bracket2.smb", 3.5)),
        ));
        v.push((
            "offset_bracket2_5.1".into(),
            Box::new(|| smb_offset("offset_bracket2.smb", 5.1)),
        ));
    }
    v
}

fn on_threads(n: usize, case: &Case) -> std::result::Result<(Vec<u8>, usize), String> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(n)
        .build()
        .map_err(|e| e.to_string())?
        .install(|| (case.1)())
}

#[test]
fn fingerprints() {
    let mut lines = Vec::new();
    let mut fails = Vec::new();
    for c in cases() {
        let one = on_threads(1, &c);
        let four = on_threads(4, &c);
        let line = match (&one, &four) {
            (Ok((a, fa)), Ok((b, _))) => {
                if a != b {
                    fails.push(format!("{}: differs on 1 and 4 threads", c.0));
                }
                format!("FP {} {:016x} {fa}", c.0, fnv(a))
            }
            (Err(a), Err(b)) => {
                if a != b {
                    fails.push(format!(
                        "{}: fails differently on 1 and 4 threads: {a} / {b}",
                        c.0
                    ));
                }
                format!("FP {} error {}", c.0, a.replace(char::is_whitespace, "_"))
            }
            _ => {
                fails.push(
                    format!("{}: {one:?} on 1 thread, {four:?} on 4", c.0)
                        .chars()
                        .take(400)
                        .collect(),
                );
                format!("FP {} mixed", c.0)
            }
        };
        println!("{line}");
        lines.push(line);
    }
    if let Ok(out) = std::env::var("CADRS_FINGERPRINT_OUT") {
        std::fs::write(&out, lines.join("\n") + "\n").unwrap();
    }
    assert!(fails.is_empty(), "{}", fails.join("\n"));
}
