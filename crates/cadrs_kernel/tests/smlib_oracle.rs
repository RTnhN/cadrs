//! Fillets compared with SMLib's (NVIDIA's usd-brep, Apache-2.0) as a second, independent
//! kernel: each case's body goes to the oracle (`tools/smlib_oracle`) as an `.smb`, both kernels
//! fillet the same edges (named by their middle points), and the volumes and areas must agree.
//!
//! Run with the oracle built (see `tools/smlib_oracle/CMakeLists.txt`) and named by
//! `CADRS_SMLIB_ORACLE`; without it the tests say so and pass.
#![allow(clippy::all)]

use cadrs_kernel::native::draw::Shape;
use cadrs_kernel::native::NativeKernel;

struct Oracle {
    ok: bool,
    status: String,
    volume: f64,
    area: f64,
}

fn field(json: &str, key: &str) -> Option<String> {
    let at = json.find(&format!("\"{key}\": "))? + key.len() + 4;
    let rest = &json[at..];
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    Some(rest[..end].trim().trim_matches('"').to_string())
}

fn oracle_bin() -> Option<String> {
    std::env::var("CADRS_SMLIB_ORACLE").ok().filter(|p| std::path::Path::new(p).exists())
}

/// SMLib's fillet of `edges` (by index in `s`'s body) at radius `r`.
fn smlib_fillet(k: &NativeKernel, s: &Shape, edges: &[usize], r: f64, name: &str) -> Option<Oracle> {
    let bin = oracle_bin()?;
    let Shape::Body(id) = s else { return None };
    let b = k.get(*id).ok()?;
    let dir = std::env::temp_dir().join("cadrs_smlib_oracle");
    std::fs::create_dir_all(&dir).ok()?;
    let input = dir.join(format!("{name}.smb"));
    std::fs::write(&input, cadrs_kernel::native::oracle::smb_text(b)).ok()?;
    let mut cmd = std::process::Command::new(bin);
    cmd.arg("fillet").arg(&input).arg(dir.join(format!("{name}_out.smb"))).arg(r.to_string());
    for &e in edges {
        let ed = &b.edges[e];
        let p = ed.point(ed.mid_t());
        cmd.arg(p.x.to_string()).arg(p.y.to_string()).arg(p.z.to_string());
    }
    let out = cmd.output().ok()?;
    let json = String::from_utf8_lossy(&out.stdout).lines().last()?.to_string();
    Some(Oracle {
        ok: field(&json, "ok")? == "true",
        status: field(&json, "status").unwrap_or_default(),
        volume: field(&json, "volume").and_then(|v| v.parse().ok()).unwrap_or(f64::NAN),
        area: field(&json, "area").and_then(|v| v.parse().ok()).unwrap_or(f64::NAN),
    })
}

/// Fillets `edges` of `s` with both kernels and compares; returns a line describing the outcome.
fn compare(k: &mut NativeKernel, s: &Shape, edges: &[usize], r: f64, name: &str) -> Result<String, String> {
    let Some(o) = smlib_fillet(k, s, edges, r, name) else {
        return Ok(format!("{name}: no oracle (set CADRS_SMLIB_ORACLE)"));
    };
    let Shape::Body(id) = s else { unreachable!() };
    let picked: Vec<Shape> = edges.iter().map(|&e| Shape::BodyEdge(*id, cadrs_kernel::EdgeId(e as u64))).collect();
    let refs: Vec<&Shape> = picked.iter().collect();
    let ours = k.blend(s, r, &refs);
    match (ours, o.ok) {
        (Ok(res), true) => {
            let (v, a) = (k.vprops(&res).map_err(|e| e.to_string())?, k.sprops(&res).map_err(|e| e.to_string())?);
            let (dv, da) = ((v - o.volume).abs() / o.volume.abs().max(1e-9), (a - o.area).abs() / o.area.abs().max(1e-9));
            let line = format!("{name}: volume {v:.6} vs SMLib {:.6} ({dv:.1e}), area {a:.6} vs {:.6} ({da:.1e})", o.volume, o.area);
            if dv < 1e-4 && da < 1e-4 { Ok(line) } else { Err(line) }
        }
        (Ok(res), false) => Ok(format!("{name}: ours {:.6}, SMLib fails ({})", k.vprops(&res).unwrap_or(f64::NAN), o.status)),
        (Err(e), true) => Err(format!("{name}: ours fails ({e}), SMLib gives volume {:.6} area {:.6}", o.volume, o.area)),
        (Err(e), false) => Ok(format!("{name}: both fail (ours: {e}; SMLib: {})", o.status)),
    }
}

/// A file of OCCT's dataset (under `$CADRS_OCCT_DATA`, else the checkout's sibling).
fn data(rel: &str) -> String {
    let root = std::env::var("CADRS_OCCT_DATA").unwrap_or_else(|_| format!("{}/../../../occt-dataset/opencascade-dataset-7.8.0", env!("CARGO_MANIFEST_DIR")));
    format!("{root}/{rel}")
}

/// The edges of `s` whose middles are nearest the points.
fn edges_near(k: &NativeKernel, s: &Shape, pts: &[[f64; 3]]) -> Vec<usize> {
    let Shape::Body(id) = s else { return vec![] };
    let b = k.get(*id).unwrap();
    pts.iter()
        .map(|p| {
            let p = cadrs_kernel::native::math::P3::new(p[0], p[1], p[2]);
            (0..b.edges.len()).filter(|&e| !b.edges[e].degenerate).min_by(|&x, &y| {
                let d = |e: usize| (b.edges[e].point(b.edges[e].mid_t()) - p).norm();
                d(x).total_cmp(&d(y))
            }).unwrap()
        })
        .collect()
}

/// How a case names its edges: points near their middles, or OCCT `explode` indices.
enum Pick {
    Near(Vec<[f64; 3]>),
    Explode(Vec<usize>),
    NExplode(Vec<usize>),
}

type Case = (&'static str, Box<dyn Fn(&mut NativeKernel) -> (Shape, Pick, f64)>);

fn run(cases: Vec<Case>) {
    if oracle_bin().is_none() {
        eprintln!("SMLib oracle not built: set CADRS_SMLIB_ORACLE to tools/smlib_oracle's binary");
        return;
    }
    let mut bad = Vec::new();
    for (name, build) in cases {
        let mut k = NativeKernel::new();
        let (s, pick, r) = build(&mut k);
        let pick_kind = if matches!(pick, Pick::NExplode(_)) { 2 } else { 1 };
        let edges = match pick {
            Pick::Near(pts) => edges_near(&k, &s, &pts),
            Pick::Explode(ix) | Pick::NExplode(ix) => {
                let all = if matches!(pick_kind, 1) { k.explode_edges(&s).unwrap() } else { k.nexplode_edges(&s).unwrap() };
                ix.iter().map(|&i| match all[i] { Shape::BodyEdge(_, e) => e.0 as usize, _ => unreachable!() }).collect()
            }
        };
        match compare(&mut k, &s, &edges, r, name) {
            Ok(line) => eprintln!("ok   {line}"),
            Err(line) => {
                eprintln!("FAIL {line}");
                bad.push(line);
            }
        }
    }
    assert!(bad.is_empty(), "{} cases disagree with SMLib:\n{}", bad.len(), bad.join("\n"));
}

/// OCCT blend cases (tests/blend/simple) the native kernel fails, on SMLib.
fn occt(name: &'static str, file: &'static str, scale: f64, r: f64, nexplode: bool, edges: Vec<usize>) -> Case {
    (name, Box::new(move |k: &mut NativeKernel| {
        let s = k.restore(&data(file)).unwrap();
        let s = if scale != 1.0 { k.tscale(&s, 0.0, 0.0, 0.0, scale).unwrap() } else { s };
        (s, if nexplode { Pick::NExplode(edges.clone()) } else { Pick::Explode(edges.clone()) }, r)
    }))
}

#[test]
#[ignore = "the native kernel fails these today (blend tool booleans)"]
fn occt_blend_failures_on_smlib() {
    run(vec![
        occt("simple_t6", "brep/CFI_6_f12fhb.rle", 1.0, 6.0, false, vec![6]),
        occt("simple_t7", "brep/CFI_6_f56fhc.rle", 1.0, 6.0, true, vec![7]),
        occt("simple_t9", "brep/CFI_6_g12fhd.rle", 1.0, 10.0, false, vec![9]),
        occt("simple_u3", "brep/CFI_6_h12fhf.rle", 1.0, 5.0, false, vec![2]),
        occt("simple_u4", "brep/CFI_6_h56fhg.rle", 1.0, 5.0, false, vec![1]),
        occt("simple_u5", "brep/CFI_6_i12fhh.rle", 1.0, 2.5, false, vec![5]),
        occt("simple_x4", "brep/CCV_1_h1_gsk.rle", 1.0, 30.0, false, vec![13]),
        occt("simple_m4", "brep/CFI_d1234fii.rle", 10.0, 10.0, false, vec![14]),
        occt("simple_m7", "brep/CFI_d5678fij.rle", 10.0, 10.0, false, vec![13]),
        occt("simple_n6", "brep/CFI_f1234fim.rle", 10.0, 5.0, false, vec![4]),
        occt("simple_o3", "brep/G1234.draw", 10.0, 5.0, false, vec![5]),
        occt("simple_o8", "brep/CFI_h1234fiq.rle", 10.0, 5.0, false, vec![2]),
        occt("simple_p4", "brep/CFI_i1234fis.rle", 10.0, 5.0, false, vec![2]),
    ]);
}

#[test]
fn fillets_agree_with_smlib() {
    run(vec![
        ("box_edge", Box::new(|k| (k.box_(10.0, 20.0, 30.0).unwrap(), Pick::Near(vec![[0.0, 0.0, 15.0]]), 2.0))),
        ("box_two_edges", Box::new(|k| (k.box_(10.0, 20.0, 30.0).unwrap(), Pick::Near(vec![[0.0, 0.0, 15.0], [10.0, 0.0, 15.0]]), 2.0))),
        ("box_corner", Box::new(|k| (k.box_(10.0, 20.0, 30.0).unwrap(), Pick::Near(vec![[0.0, 0.0, 15.0], [5.0, 0.0, 30.0], [0.0, 10.0, 30.0]]), 2.0))),
        ("cylinder_rim", Box::new(|k| (k.pcylinder(5.0, 10.0).unwrap(), Pick::Near(vec![[-5.0, 0.0, 10.0]]), 1.0))),
        ("cone_rim", Box::new(|k| (k.pcone(5.0, 2.0, 8.0).unwrap(), Pick::Near(vec![[-5.0, 0.0, 0.0]]), 1.0))),
        ("wedge_edge", Box::new(|k| (k.pcylinder_a(None, 50.0, 100.0, 90.0).unwrap(), Pick::Near(vec![[35.355, 35.355, 100.0]]), 10.0))),
        ("sphere_zone_rim", Box::new(|k| (k.psphere_zone(None, 15.0, -60.0, 60.0, 360.0).unwrap(), Pick::Near(vec![[-7.5, 0.0, 12.99]]), 2.0))),
    ]);
}
