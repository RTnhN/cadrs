//! Rounds and a chamfer round the opening of an outward shell (a user's document,
//! `fixtures/shell_opening_chamfer.cadrs`): a 30 mm cube with every edge rounded 5, shelled 6.5
//! outward with its back face, two top rounds and their corner taken away; Fillet 2 rounds the
//! opening's seven rims, Chamfer 1 (tangent propagation) chamfers the rim round the back wall's
//! cut end. The document rebuilds at a range of fillet radii and chamfer widths: every feature
//! builds, and the chamfer takes material off, more the wider it is.
#![cfg(feature = "brep")]

use cadrs_core::document::Document;
use cadrs_core::{
    ElementId, FeatureId, FeatureKind, History, Store, commands::SetFeature, rebuild,
};

/// The fillet radii the sweep tries (the document's own 2.8 among them).
const RADII: [f64; 6] = [1.0, 2.0, 2.8, 3.25, 3.5, 4.0];
/// The chamfer widths it tries at each (the document's own 1).
const WIDTHS: [f64; 4] = [0.5, 1.0, 1.5, 2.0];

struct Studio {
    d: Document,
    h: History,
    el: ElementId,
    fillet: FeatureId,
    chamfer: FeatureId,
}

impl Studio {
    fn new() -> Self {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/shell_opening_chamfer.cadrs");
        let d = Store::load_path(&path).expect("the fixture loads").document;
        let el = d.elements[0].id;
        let id = |name: &str| {
            d.element(el)
                .unwrap()
                .features()
                .iter()
                .find(|f| f.name == name)
                .unwrap_or_else(|| panic!("no feature {name}"))
                .id
        };
        let (fillet, chamfer) = (id("Fillet 2"), id("Chamfer 1"));
        Self {
            d,
            h: History::default(),
            el,
            fillet,
            chamfer,
        }
    }

    /// Fillet 2's radius `r` and Chamfer 1's width `w`.
    fn set(&mut self, r: f64, w: f64) {
        let el = self.d.element(self.el).unwrap();
        let mut fillet = el.feature(self.fillet).unwrap().kind.clone();
        let FeatureKind::Fillet(x) = &mut fillet else {
            panic!("Fillet 2 isn't a fillet")
        };
        x.size = r;
        x.size_expr = format!("{r} mm");
        let mut chamfer = el.feature(self.chamfer).unwrap().kind.clone();
        let FeatureKind::Chamfer(x) = &mut chamfer else {
            panic!("Chamfer 1 isn't a chamfer")
        };
        x.distance = w;
        x.distance_expr = format!("{w} mm");
        for (feature, kind) in [(self.fillet, fillet), (self.chamfer, chamfer)] {
            self.h
                .execute(
                    &mut self.d,
                    &SetFeature {
                        element: self.el,
                        feature,
                        kind,
                        label: "Edit".into(),
                    },
                )
                .expect("the edit applies");
        }
    }

    /// The part's volume built up to and with the chamfer (`with_chamfer`) or without it; the
    /// features that failed, by name and why, otherwise.
    fn volume(&self, with_chamfer: bool) -> Result<f64, String> {
        let el = self.d.element(self.el).unwrap();
        let mut features = el.active_features();
        if !with_chamfer {
            features.retain(|f| f.id != self.chamfer);
        }
        let b = rebuild::build(&features);
        if !b.errors.is_empty() {
            let name = |id: &FeatureId| {
                el.feature(*id)
                    .map_or_else(|| format!("{id:?}"), |f| f.name.clone())
            };
            return Err(b
                .errors
                .iter()
                .map(|(id, e)| format!("{}: {e}", name(id)))
                .collect::<Vec<_>>()
                .join("; "));
        }
        match b.parts.as_slice() {
            [p] => Ok(p.mass.as_ref().expect("the part's mass").volume),
            ps => Err(format!("{} parts", ps.len())),
        }
    }
}

/// Every radius and width: what failed (the fillet alone, or the chamfer after it) and where
/// the chamfer didn't take more off the wider it was.
fn sweep(radii: &[f64], widths: &[f64]) -> Vec<String> {
    let mut s = Studio::new();
    let mut bad = Vec::new();
    for &r in radii {
        s.set(r, widths[0]);
        let filleted = match s.volume(false) {
            Ok(v) => v,
            Err(e) => {
                bad.push(format!("fillet {r}: {e}"));
                continue;
            }
        };
        let mut last = 0.0;
        for &w in widths {
            s.set(r, w);
            match s.volume(true) {
                Ok(v) => {
                    let removed = filleted - v;
                    if removed <= last {
                        bad.push(format!(
                            "fillet {r}, chamfer {w}: takes off {removed:.4} mm³, no more than {last:.4} narrower"
                        ));
                    }
                    last = removed;
                }
                Err(e) => bad.push(format!("fillet {r}, chamfer {w}: {e}")),
            }
        }
    }
    bad
}

#[test]
fn the_documents_own_fillet_and_chamfer() {
    let bad = sweep(&[2.8], &[1.0]);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// Slow (a rebuild of the shell, fillets and chamfer each time): run with `--ignored`.
#[test]
#[ignore]
fn fillet_radii_and_chamfer_widths() {
    let bad = sweep(&RADII, &WIDTHS);
    assert!(bad.is_empty(), "{} failed:\n{}", bad.len(), bad.join("\n"));
}

/// One radius and width from `CADRS_CASE=r,w` (with `CADRS_DUMP_DIR` for the kernel's inputs).
#[test]
#[ignore]
fn one_case() {
    let Ok(case) = std::env::var("CADRS_CASE") else {
        return;
    };
    let v: Vec<f64> = case.split(',').map(|x| x.parse().unwrap()).collect();
    let bad = sweep(&[v[0]], &[v[1]]);
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}
