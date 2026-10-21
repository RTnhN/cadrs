//! What every generated SMLib suite (`tests/smlib_<suite>.rs`, from `tools/smlib_to_rust.py`)
//! runs on: its models (`tests/data/smlib/<suite>`), SMLib's edges and faces found again in
//! our bodies, and the checks of a result against what SMLib's test asserts and, where the
//! generator ran SMLib itself (the oracle), what SMLib produced.
#![allow(dead_code)]

use cadrs_kernel::native::math::P3;
use cadrs_kernel::native::smb;
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::topo::Body;
use cadrs_kernel::{EdgeId, FaceId};

/// A model of a suite, with where SMLib's tests point at its pieces (in SMLib's file order).
pub struct Model {
    pub body: Body,
    pub edge_mids: Vec<P3>,
    pub edge_ends: Vec<(P3, P3)>,
    pub face_centroids: Vec<Option<P3>>,
}

/// The path of a suite's model.
pub fn data(suite: &str, file: &str) -> String {
    format!("{}/tests/data/smlib/{suite}/{file}", env!("CARGO_MANIFEST_DIR"))
}

/// Reads `file` of `suite`: an `.smb`, or B-rep `brep` of an `.smp` (`file#brep`).
pub fn model(suite: &str, file: &str) -> Result<Model, String> {
    let (file, brep) = match file.split_once('#') {
        Some((f, b)) => (f, b.parse::<usize>().map_err(|e| e.to_string())?),
        None => (file, 0),
    };
    let bytes = std::fs::read(data(suite, file)).map_err(|e| format!("{file}: {e}"))?;
    let b = if file.ends_with(".smp") {
        let mut p = smb::read_part(&bytes)?;
        if brep >= p.breps.len() {
            return Err(format!("{file} has {} B-reps", p.breps.len()));
        }
        p.breps.swap_remove(brep)
    } else {
        smb::read_brep(&bytes)?
    };
    Ok(Model { body: b.body, edge_mids: b.edge_mids, edge_ends: b.edge_ends, face_centroids: b.face_centroids })
}

fn size(body: &Body) -> f64 {
    let d = body.bbox().diagonal();
    if d > 0.0 { d } else { 1.0 }
}

/// The body's edges nearest each point (an edge's middle), each once, in order. Panics (it is
/// the test's fault, not the kernel's) when no edge passes near a point or two do.
pub fn pick_edges(body: &Body, pts: &[P3]) -> Vec<EdgeId> {
    let s = size(body);
    let mut ids = Vec::new();
    for (k, m) in pts.iter().enumerate() {
        let mut d: Vec<(f64, usize)> = (0..body.edges.len())
            .filter(|&e| !body.edges[e].degenerate)
            .map(|e| {
                let ed = &body.edges[e];
                let t = ed.curve.closest(m, ed.t[0], ed.t[1]);
                ((ed.point(t) - m).norm(), e)
            })
            .collect();
        d.sort_by(|a, b| a.0.total_cmp(&b.0));
        let Some(&(d1, best)) = d.first() else { panic!("no edges to pick from") };
        assert!(d1 <= 1e-4 * s, "edge pick {k}: no edge passes through {m:?} (nearest {d1:.3e} off)");
        if let Some(&(d2, other)) = d.get(1) {
            assert!(d2 > d1 + 1e-7 * s, "edge pick {k}: edges {best} and {other} both pass through {m:?}");
        }
        if !ids.contains(&EdgeId(best as u64)) {
            ids.push(EdgeId(best as u64));
        }
    }
    ids
}

/// The body's faces whose centroids ([`smb::face_centroids`]) are nearest each point, each once,
/// in order. Panics when no face's centroid is near a point or two are.
pub fn pick_faces(body: &Body, pts: &[P3]) -> Vec<FaceId> {
    let s = size(body);
    let cents = smb::face_centroids(body);
    let mut ids = Vec::new();
    for (k, m) in pts.iter().enumerate() {
        let mut d: Vec<(f64, usize)> = cents.iter().enumerate().map(|(f, c)| ((c - m).norm(), f)).collect();
        d.sort_by(|a, b| a.0.total_cmp(&b.0));
        let Some(&(d1, best)) = d.first() else { panic!("no faces to pick from") };
        assert!(d1 <= 1e-4 * s, "face pick {k}: no face's centroid is at {m:?} (nearest {d1:.3e} off)");
        if let Some(&(d2, other)) = d.get(1) {
            assert!(d2 > d1 + 1e-7 * s, "face pick {k}: faces {best} and {other} both have their centroid at {m:?}");
        }
        if !ids.contains(&FaceId(best as u64)) {
            ids.push(FaceId(best as u64));
        }
    }
    ids
}

/// The points of the model's edges `ranges` name (index ranges, inclusive; all others when
/// `include` is false).
pub fn edge_points(m: &Model, ranges: &[(usize, usize)], include: bool) -> Vec<P3> {
    (0..m.edge_mids.len()).filter(|i| ranges.iter().any(|&(a, b)| (a..=b).contains(i)) == include).map(|i| m.edge_mids[i]).collect()
}

/// The model's edges `idx` names, as body edges.
pub fn edges(m: &Model, idx: &[usize]) -> Result<Vec<EdgeId>, String> {
    let pts = idx.iter().map(|&i| m.edge_mids.get(i).copied().ok_or(format!("no edge {i}"))).collect::<Result<Vec<_>, _>>()?;
    Ok(pick_edges(&m.body, &pts))
}

/// The model's faces `idx` names, as body faces.
pub fn faces(m: &Model, idx: &[usize]) -> Result<Vec<FaceId>, String> {
    let pts = idx.iter().map(|&i| m.face_centroids.get(i).copied().flatten().ok_or(format!("no face {i}"))).collect::<Result<Vec<_>, _>>()?;
    Ok(pick_faces(&m.body, &pts))
}

/// What a result should be. SMLib's test asserts counts; the oracle (SMLib run by the
/// generator) gives volume, area, counts and whether it is a solid.
#[derive(Debug, Clone, Copy, Default)]
pub struct Expect {
    pub volume: Option<f64>,
    pub area: Option<f64>,
    /// Face and edge counts are compared only when every face of the input is planar: on
    /// curved models the two kernels split periodic faces and place seams differently.
    pub faces: Option<usize>,
    pub edges: Option<usize>,
    pub solid: Option<bool>,
}

/// Relative tolerance on volume and area against the oracle.
pub const REL_TOL: f64 = 1e-4;

fn planar(b: &Body) -> bool {
    b.faces.iter().all(|f| matches!(f.surf, Surface::Plane { .. }))
}

/// Non-degenerate edges.
pub fn edge_count(b: &Body) -> usize {
    b.edges.iter().filter(|e| !e.degenerate).count()
}

/// Checks a result: valid (`check()`, which also wants every solid to enclose a positive
/// volume), its shells closed with an Euler characteristic a closed surface can have, and
/// `e`'s values. `input` decides whether face and edge counts are compared (a planar input).
pub fn check_result(input: &Body, out: &Body, e: &Expect) -> Result<(), String> {
    let issues = out.check();
    if let Some(i) = issues.first() {
        return Err(format!("invalid: {i}"));
    }
    let want_solid = e.solid.unwrap_or(true);
    if want_solid {
        if out.solids.is_empty() {
            return Err("not a solid".into());
        }
        if let Some(s) = out.shells.iter().position(|s| !s.closed) {
            return Err(format!("shell {s} is open"));
        }
        // V − E + F − (L − F) = 2(S − G): even, at most 2 per shell.
        let mut used = vec![false; out.verts.len()];
        for ed in out.edges.iter().filter(|e| !e.degenerate) {
            used[ed.v[0]] = true;
            used[ed.v[1]] = true;
        }
        let v = used.iter().filter(|&&u| u).count() as i64;
        let (ne, nf) = (edge_count(out) as i64, out.faces.len() as i64);
        let nl: i64 = out.faces.iter().map(|f| f.loops.len() as i64).sum();
        let chi = v - ne + nf - (nl - nf);
        if chi % 2 != 0 || chi > 2 * out.shells.len() as i64 {
            return Err(format!("Euler characteristic {chi} with {} shells", out.shells.len()));
        }
    } else if !out.solids.is_empty() {
        return Err("a solid, SMLib gives none".into());
    }
    if let Some(n) = e.faces {
        if planar(input) && out.faces.len() != n {
            return Err(format!("{} faces, SMLib gives {n}", out.faces.len()));
        }
    }
    if let Some(n) = e.edges {
        if planar(input) && edge_count(out) != n {
            return Err(format!("{} edges, SMLib gives {n}", edge_count(out)));
        }
    }
    if e.volume.is_some() || e.area.is_some() {
        let m = cadrs_kernel::native::mass::body_mass(out);
        for (what, ours, theirs) in [("volume", m.volume, e.volume), ("area", m.surface_area, e.area)] {
            if let Some(t) = theirs {
                let rel = (ours - t).abs() / t.abs().max(1e-9);
                if !(rel <= REL_TOL) {
                    return Err(format!("{what} {ours:.6}, SMLib gives {t:.6} ({rel:.1e} off)"));
                }
            }
        }
    }
    Ok(())
}

/// A known failure (listed in `tools/smlib_known_failures/<suite>.txt`) must still fail.
pub fn known_failure(r: Result<(), String>, suite: &str, name: &str) {
    assert!(r.is_err(), "a known failure now passes: remove `{name}` from tools/smlib_known_failures/{suite}.txt");
}
