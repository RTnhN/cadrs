//! The native kernel's side of data exchange (P3F.2): [`crate::Kernel::import_model`] /
//! `export_model` over [`super::step`] and [`super::iges`], the plain `import_step` /
//! `export_step`, `mesh_solid` ([`super::meshsolid`]) and `write_body` / `read_body`
//! ([`super::bodyio`]).
//!
//! - **Export.** Without instances each body is its own part (product), named as given. With
//!   instances, STEP writes one assembly product holding them; IGES has no assemblies, so each
//!   instance is written as its part moved into place (named after the instance).
//! - **Import.** Each distinct part comes once as a body in its own coordinates, with its
//!   occurrences' placements and names; a file without assemblies has one occurrence per part
//!   at the identity. `import_step` instead gives one body per solid, placed (and the loose
//!   shells together as one surface body).

use crate::exchange::{ExchangeFormat, ExportInstance, ImportedModel, ImportedOccurrence, ImportedPart};
use crate::{BodyId, KernelError, Motion, Result};

use super::kernel::NativeKernel;
use super::math::Affine;
use super::step::{motion_affine, ReadModel, WriteInstance};
use super::topo::{Body, Shell, Solid};

/// A file or body that can't be read: the data's fault, not the operation's.
fn failed(what: &str, why: String) -> KernelError {
    KernelError::InvalidInput(format!("{what}: {why}"))
}

fn read(format: ExchangeFormat, bytes: &[u8]) -> Result<ReadModel> {
    let r = match format {
        ExchangeFormat::Step => super::step::read(bytes),
        ExchangeFormat::Iges => super::iges::read(bytes),
    };
    let what = format!("{} import", format.label());
    let m = r.map_err(|e| failed(&what, e))?;
    // Numbers no geometry can have, or a model too large for the kernel's tolerances.
    for p in &m.parts {
        if let Some(why) = p.body.check_refs().into_iter().next().or_else(|| p.body.size_issue(super::tol::SIZE)) {
            return Err(failed(&what, format!("part '{}': {why}", p.name)));
        }
    }
    if std::env::var("CADRS_EXCHANGE_DEBUG").is_ok() {
        for w in &m.warnings {
            eprintln!("{} import: {w}", format.label());
        }
    }
    Ok(m)
}

/// Refuses bodies a writer can't put in a file (non-finite numbers, dangling indices).
fn writable(bodies: &[&Body]) -> Result<()> {
    match bodies.iter().find_map(|b| b.check_refs().into_iter().next()) {
        Some(why) => Err(KernelError::InvalidInput(format!("the body can't be written: {why}"))),
        None => Ok(()),
    }
}

pub fn export_model(k: &NativeKernel, format: ExchangeFormat, name: &str, parts: &[(BodyId, String)], instances: &[ExportInstance]) -> Result<Vec<u8>> {
    let bodies: Vec<&Body> = parts.iter().map(|(b, _)| k.get(*b).map(|a| a.as_ref())).collect::<Result<_>>()?;
    writable(&bodies)?;
    if let Some(i) = instances.iter().find(|i| i.part >= parts.len()) {
        return Err(KernelError::InvalidParameter(format!("instance '{}' refers to part {} of {}", i.name, i.part, parts.len())));
    }
    let finite = |m: &Motion| m.linear.iter().chain(m.translation.iter()).all(|x| x.is_finite());
    if let Some(i) = instances.iter().find(|i| !finite(&i.placement)) {
        return Err(KernelError::InvalidInput(format!("instance '{}' has a placement that isn't finite", i.name)));
    }
    let named: Vec<(&Body, &str)> = bodies.iter().zip(parts).map(|(b, (_, n))| (*b, n.as_str())).collect();
    Ok(match format {
        ExchangeFormat::Step => {
            let insts: Vec<WriteInstance> = instances.iter().map(|i| WriteInstance { part: i.part, placement: motion_affine(&i.placement), name: i.name.clone() }).collect();
            super::step::write(name, &named, &insts)
        }
        ExchangeFormat::Iges => {
            if instances.is_empty() {
                super::iges::write(&named)
            } else {
                let placed: Vec<(Body, String)> = instances.iter().map(|i| (bodies[i.part].transformed(&motion_affine(&i.placement)), i.name.clone())).collect();
                let named: Vec<(&Body, &str)> = placed.iter().map(|(b, n)| (b, n.as_str())).collect();
                super::iges::write(&named)
            }
        }
    })
}

pub fn import_model(k: &mut NativeKernel, format: ExchangeFormat, bytes: &[u8]) -> Result<ImportedModel> {
    let m = read(format, bytes)?;
    let mut parts = Vec::with_capacity(m.parts.len());
    for p in m.parts {
        let id = k.insert(p.body);
        parts.push(ImportedPart { body: id, name: p.name });
    }
    let occurrences = m.occurrences.into_iter().map(|o| ImportedOccurrence { part: o.part, placement: Motion { linear: o.placement.m, translation: o.placement.t }, name: o.name }).collect();
    Ok(ImportedModel { name: m.name, parts, occurrences })
}

pub fn export_step(k: &NativeKernel, bodies: &[BodyId]) -> Result<Vec<u8>> {
    let bs: Vec<&Body> = bodies.iter().map(|b| k.get(*b).map(|a| a.as_ref())).collect::<Result<_>>()?;
    writable(&bs)?;
    let names: Vec<String> = (1..=bs.len()).map(|i| format!("Body {i}")).collect();
    let named: Vec<(&Body, &str)> = bs.iter().zip(&names).map(|(b, n)| (*b, n.as_str())).collect();
    Ok(super::step::write("cadrs", &named, &[]))
}

/// One body per solid of `b` (with its void shells), and the shells of no solid together as one
/// surface body.
pub fn split_into_solids(b: &Body) -> Vec<Body> {
    let mut out = Vec::new();
    for s in &b.solids {
        let faces: Vec<usize> = s.shells.iter().flat_map(|&sh| b.shells[sh].faces.iter().copied()).collect();
        let mut nb = super::kernel::subset(b, &faces, true);
        nb.shells.clear();
        nb.solids.clear();
        let mut k = 0;
        let mut shells = Vec::new();
        for &sh in &s.shells {
            let n = b.shells[sh].faces.len();
            nb.shells.push(Shell { faces: (k..k + n).collect(), closed: true });
            shells.push(nb.shells.len() - 1);
            k += n;
        }
        nb.solids.push(Solid { shells });
        out.push(nb);
    }
    let in_solid: std::collections::HashSet<usize> = b.solids.iter().flat_map(|s| s.shells.iter().copied()).collect();
    let free: Vec<usize> = (0..b.shells.len()).filter(|s| !in_solid.contains(s)).collect();
    if !free.is_empty() {
        let faces: Vec<usize> = free.iter().flat_map(|&s| b.shells[s].faces.iter().copied()).collect();
        let mut nb = super::kernel::subset(b, &faces, false);
        nb.shells.clear();
        let mut k = 0;
        for &s in &free {
            let n = b.shells[s].faces.len();
            nb.shells.push(Shell { faces: (k..k + n).collect(), closed: b.shells[s].closed });
            k += n;
        }
        out.push(nb);
    }
    out
}

pub fn import_step(k: &mut NativeKernel, bytes: &[u8]) -> Result<Vec<BodyId>> {
    let m = read(ExchangeFormat::Step, bytes)?;
    let mut solids = Vec::new();
    let mut sheets = Body::default();
    for o in &m.occurrences {
        let b = &m.parts[o.part].body;
        let placed = if o.placement == Affine::identity() { b.clone() } else { b.transformed(&o.placement) };
        for piece in split_into_solids(&placed) {
            if piece.solids.is_empty() {
                sheets.append(&piece);
            } else {
                solids.push(piece);
            }
        }
    }
    let mut out: Vec<BodyId> = solids.into_iter().map(|b| k.insert(b)).collect();
    if !sheets.faces.is_empty() {
        out.push(k.insert(sheets));
    }
    if out.is_empty() {
        return Err(failed("STEP import", "the file has no geometry".into()));
    }
    Ok(out)
}

pub fn mesh_solid(k: &mut NativeKernel, triangles: &[[nalgebra::Point3<f64>; 3]], tol: f64) -> Result<BodyId> {
    let b = super::meshsolid::mesh_solid(triangles, tol).map_err(|e| failed("mesh", e))?;
    Ok(k.insert(b))
}

pub fn write_body(k: &NativeKernel, body: BodyId) -> Result<Vec<u8>> {
    Ok(super::bodyio::write(k.get(body)?))
}

pub fn read_body(k: &mut NativeKernel, bytes: &[u8]) -> Result<BodyId> {
    let b = super::bodyio::read(bytes).map_err(|e| failed("read body", e))?;
    Ok(k.insert(b))
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;
    use std::sync::Arc;

    use nalgebra::{Point2, Point3, Vector3};

    use super::*;
    use crate::native::nurbs::{hom, NurbsSurface};
    use crate::native::surface::Surface;
    use crate::*;

    fn p(x: f64, y: f64) -> Point2<f64> {
        Point2::new(x, y)
    }
    fn line(a: Point2<f64>, b: Point2<f64>) -> Curve2 {
        Curve2::Line { a, b, source: None }
    }
    fn poly(pts: &[Point2<f64>]) -> Loop {
        Loop { curves: (0..pts.len()).map(|i| line(pts[i], pts[(i + 1) % pts.len()])).collect() }
    }
    fn circle(c: Point2<f64>, r: f64) -> Loop {
        Loop { curves: vec![Curve2::Circle { center: c, radius: r, source: None }] }
    }
    fn region(outer: Loop, holes: Vec<Loop>) -> Region {
        Region { outer, holes, source: None }
    }
    /// The XZ plane with in-plane y along +Z, for revolve profiles.
    fn front() -> Plane {
        Plane { origin: Point3::origin(), x_dir: Vector3::x_axis(), normal: -Vector3::y_axis() }
    }
    fn z_axis() -> Axis {
        Axis { origin: Point3::origin(), dir: Vector3::z_axis() }
    }
    fn extrude(k: &mut NativeKernel, regions: Vec<Region>, d: f64) -> BodyId {
        k.extrude(&Profile::new(Plane::top(), regions), Extent::Blind(d)).unwrap().bodies[0]
    }
    fn revolve(k: &mut NativeKernel, regions: Vec<Region>) -> BodyId {
        k.revolve_with(&Profile::new(front(), regions), &RevolveSpec::solid(z_axis(), None)).unwrap().bodies[0]
    }

    /// The test bodies, with their exact volumes.
    fn bodies(k: &mut NativeKernel) -> Vec<(&'static str, BodyId, f64)> {
        let mut out = Vec::new();
        let b = extrude(k, vec![region(poly(&[p(0., 0.), p(100., 0.), p(100., 60.), p(0., 60.)]), vec![])], 25.0);
        out.push(("box", b, 150_000.0));
        let c = extrude(k, vec![region(circle(p(3., 4.), 10.0), vec![circle(p(5., 4.), 4.0)])], 10.0);
        out.push(("cylinder with a hole", c, PI * (100.0 - 16.0) * 10.0));
        let t = revolve(k, vec![region(circle(p(20., 10.), 5.0), vec![])]);
        out.push(("torus", t, 2.0 * PI * PI * 20.0 * 25.0));
        let half = Loop { curves: vec![Curve2::Arc { center: p(0., 0.), radius: 10.0, start_angle: -PI / 2.0, sweep: PI, source: None }, line(p(0., 10.), p(0., -10.))] };
        let s = revolve(k, vec![region(half, vec![])]);
        out.push(("sphere", s, 4.0 / 3.0 * PI * 1000.0));
        let cone = revolve(k, vec![region(poly(&[p(0., 0.), p(10., 0.), p(0., 20.)]), vec![])]);
        out.push(("cone", cone, PI * 100.0 * 20.0 / 3.0));
        let bz = Loop {
            curves: vec![
                line(p(0., 0.), p(20., 0.)),
                Curve2::Bezier { poles: [p(20., 0.), p(30., 10.), p(10., 15.), p(0., 20.)], source: None },
                line(p(0., 20.), p(0., 0.)),
            ],
        };
        let bzb = extrude(k, vec![region(bz, vec![])], 5.0);
        let v = k.mass_properties(bzb).unwrap().volume;
        out.push(("Bézier side", bzb, v));
        // The cylinder with a hole mirrored (left-handed frames, reversed senses).
        let m = k.transform_motion(c, &Motion::reflection(Point3::origin(), Vector3::x_axis())).unwrap().bodies[0];
        out.push(("mirrored cylinder", m, PI * (100.0 - 16.0) * 10.0));
        out
    }

    /// A 10 × 20 × 30 box with its top face on an equivalent bilinear NURBS patch.
    fn nurbs_top_box(k: &mut NativeKernel) -> BodyId {
        let b = extrude(k, vec![region(poly(&[p(0., 0.), p(10., 0.), p(10., 20.), p(0., 20.)]), vec![])], 30.0);
        let mut body = (**k.get(b).unwrap()).clone();
        let top = body.faces.iter().position(|f| matches!(f.surf, Surface::Plane { f: fr } if fr.o.z > 29.0)).unwrap();
        let c = |x: f64, y: f64| hom(&Point3::new(x, y, 30.0), 1.0);
        let n = NurbsSurface { du: 1, dv: 1, ku: vec![0., 0., 1., 1.], kv: vec![0., 0., 1., 1.], cps: vec![vec![c(-5., -5.), c(-5., 25.)], vec![c(15., -5.), c(15., 25.)]] };
        let surf = Surface::Nurbs(Arc::new(n));
        body.faces[top].surf = surf.clone();
        let edges = body.edges.clone();
        for l in &mut body.faces[top].loops {
            for co in l.iter_mut() {
                let e = &edges[co.edge];
                co.pc = crate::native::pcurve::pcurve(&surf, &e.curve, e.t[0], e.t[1], None);
            }
        }
        crate::native::build::fix_senses(&mut body, [top]);
        assert!(body.check().is_empty(), "{:?}", body.check());
        k.insert(body)
    }

    fn check_same(k: &NativeKernel, name: &str, a: BodyId, b: BodyId, vol: f64, tol: f64) {
        let (ma, mb) = (k.mass_properties(a).unwrap(), k.mass_properties(b).unwrap());
        let body = k.get(b).unwrap();
        assert!(body.check().is_empty(), "{name}: {:?}", body.check());
        assert_eq!(k.faces(a).unwrap().len(), k.faces(b).unwrap().len(), "{name}: faces");
        assert!((mb.volume - vol).abs() <= tol * vol, "{name}: volume {} vs {vol}", mb.volume);
        assert!((mb.surface_area - ma.surface_area).abs() <= tol * ma.surface_area, "{name}: area {} vs {}", mb.surface_area, ma.surface_area);
        assert!((mb.center_of_mass - ma.center_of_mass).norm() <= 1e-6 * (1.0 + ma.center_of_mass.coords.norm()), "{name}: centroid");
    }

    #[test]
    fn step_round_trips() {
        let mut k = NativeKernel::new();
        let mut all = bodies(&mut k);
        let nb = nurbs_top_box(&mut k);
        all.push(("NURBS top face", nb, 6000.0));
        for (name, b, vol) in all {
            let bytes = k.export_model(ExchangeFormat::Step, name, &[(b, name.into())], &[]).unwrap();
            let m = k.import_model(ExchangeFormat::Step, &bytes).unwrap();
            assert_eq!(m.parts.len(), 1, "{name}");
            assert_eq!(m.parts[0].name, name);
            assert_eq!(k.solid_count(m.parts[0].body).unwrap(), 1, "{name}");
            check_same(&k, name, b, m.parts[0].body, vol, 1e-9);
            // And again: what we read writes the same geometry.
            let again = k.export_model(ExchangeFormat::Step, name, &[(m.parts[0].body, name.into())], &[]).unwrap();
            let m2 = k.import_model(ExchangeFormat::Step, &again).unwrap();
            check_same(&k, name, b, m2.parts[0].body, vol, 1e-9);
        }
    }

    #[test]
    fn iges_round_trips() {
        let mut k = NativeKernel::new();
        let mut all = bodies(&mut k);
        let nb = nurbs_top_box(&mut k);
        all.push(("NURBS top face", nb, 6000.0));
        for (name, b, vol) in all {
            let bytes = k.export_model(ExchangeFormat::Iges, name, &[(b, name.into())], &[]).unwrap();
            let m = k.import_model(ExchangeFormat::Iges, &bytes).unwrap();
            assert_eq!(m.parts.len(), 1, "{name}");
            assert_eq!(k.solid_count(m.parts[0].body).unwrap(), 1, "{name}");
            check_same(&k, name, b, m.parts[0].body, vol, 1e-6);
        }
    }

    #[test]
    fn bodies_as_bytes_round_trip_exactly() {
        let mut k = NativeKernel::new();
        for (name, b, _) in bodies(&mut k) {
            let bytes = k.write_body(b).unwrap();
            let r = k.read_body(&bytes).unwrap();
            assert_eq!(**k.get(r).unwrap(), **k.get(b).unwrap(), "{name}");
            assert!(k.read_body(&bytes[..bytes.len() - 3]).is_err());
        }
        assert!(k.read_body(b"nonsense").is_err());
    }

    /// Reads the STEP and IGES files listed (one path per line) in the file named by
    /// `CADRS_EXCHANGE_FILES` and checks each part: valid topology, a positive volume per solid,
    /// and the exact volume against its tessellation's. Prints one line per file.
    #[test]
    #[ignore]
    fn read_a_corpus() {
        let Some(list) = std::env::var("CADRS_EXCHANGE_FILES").ok().and_then(|p| std::fs::read_to_string(p).ok()) else { return };
        let (mut ok, mut bad) = (0, 0);
        for path in list.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let Ok(bytes) = std::fs::read(path) else { continue };
            let fmt = if path.to_lowercase().ends_with("igs") || path.to_lowercase().ends_with("iges") { ExchangeFormat::Iges } else { ExchangeFormat::Step };
            let t0 = std::time::Instant::now();
            let m = match fmt {
                ExchangeFormat::Step => super::super::step::read(&bytes),
                ExchangeFormat::Iges => super::super::iges::read(&bytes),
            };
            let dt = t0.elapsed().as_secs_f64();
            let m = match m {
                Ok(m) => m,
                Err(e) => {
                    bad += 1;
                    println!("FAIL {path}: {e}");
                    continue;
                }
            };
            let mut problems = Vec::new();
            let (mut faces, mut solids, mut vol) = (0, 0, 0.0);
            for p in &m.parts {
                faces += p.body.faces.len();
                solids += p.body.solids.len();
                // Edges off their faces within the file's own tolerance are not our doing.
                let size = p.body.bbox().diagonal();
                let errs: Vec<String> = p
                    .body
                    .check()
                    .into_iter()
                    .filter(|e| !e.split("off the edge by ").nth(1).and_then(|d| d.parse::<f64>().ok()).is_some_and(|d| d < 1e-3f64.max(1e-6 * size)))
                    .collect();
                if !errs.is_empty() {
                    problems.push(format!("{} check errors ({})", errs.len(), errs[0]));
                }
                for piece in split_into_solids(&p.body) {
                    if piece.solids.is_empty() {
                        continue;
                    }
                    let v = super::super::mass::body_mass(&piece).volume;
                    vol += v;
                    let defl = (piece.bbox().diagonal() * 2e-4).max(1e-4);
                    let (fm, _) = super::super::tess::mesh_body(&piece, defl, 0.1);
                    let mut mv = 0.0;
                    for f in &fm {
                        for t in &f.indices {
                            let [a, b, c] = t.map(|i| f.positions[i as usize].coords);
                            mv += a.dot(&b.cross(&c)) / 6.0;
                        }
                    }
                    if v <= 0.0 || (mv - v).abs() > 1e-2 * v.abs().max(1e-9) {
                        problems.push(format!("volume {v:.6} vs mesh {mv:.6}"));
                    }
                    // A closed, consistently oriented shell's volume doesn't move with it.
                    let moved = piece.transformed(&Affine::translation(nalgebra::Vector3::new(1000.0, -700.0, 300.0)));
                    let vm = super::super::mass::body_mass(&moved).volume;
                    if (vm - v).abs() > 1e-6 * v.abs().max(1.0) {
                        let bb = piece.bbox();
                        problems.push(format!("volume {v:.6} moved {vm:.6} (box {:.1?})", bb.max - bb.min));
                    }
                }
            }
            let dropped: Vec<&String> = m.warnings.iter().filter(|w| w.contains("no seam") || w.contains("no bounds") || w.contains("doesn't close") || w.contains("unsupported")).collect();
            if !dropped.is_empty() {
                problems.push(format!("{} faces dropped ({})", dropped.len(), dropped[0]));
            }
            let warn = if m.warnings.is_empty() { String::new() } else { format!(" warnings {} ({})", m.warnings.len(), m.warnings[0]) };
            if problems.is_empty() {
                ok += 1;
                println!("ok   {path}: {} parts {} occurrences (model {:?}) {faces} faces {solids} solids V={vol:.4} {dt:.2}s{warn}", m.parts.len(), m.occurrences.len(), m.name);
            } else {
                bad += 1;
                let bb = m.parts[0].body.bbox();
                println!("BAD  {path}: {} parts {faces} faces {solids} solids {dt:.2}s box {:.2?}: {}{warn}", m.parts.len(), bb.max - bb.min, problems.join("; "));
            }
        }
        println!("{ok} ok, {bad} bad");
    }

    /// Writes the test bodies as STEP and IGES files (and a two-part assembly) into the directory
    /// `CADRS_EXPORT_DIR`, with their volumes and areas in `expected.txt`, to check them with
    /// other readers.
    #[test]
    #[ignore]
    fn write_samples() {
        let Ok(dir) = std::env::var("CADRS_EXPORT_DIR") else { return };
        let dir = std::path::PathBuf::from(dir);
        let mut k = NativeKernel::new();
        let mut all = bodies(&mut k);
        let nb = nurbs_top_box(&mut k);
        all.push(("NURBS top face", nb, 6000.0));
        let mut expected = String::new();
        for (i, (name, b, _)) in all.iter().enumerate() {
            let mp = k.mass_properties(*b).unwrap();
            for (fmt, ext) in [(ExchangeFormat::Step, "step"), (ExchangeFormat::Iges, "igs")] {
                let bytes = k.export_model(fmt, name, &[(*b, name.to_string())], &[]).unwrap();
                let file = format!("body{i}.{ext}");
                std::fs::write(dir.join(&file), bytes).unwrap();
                expected.push_str(&format!("{file} {} {} {name}\n", mp.volume, mp.surface_area));
            }
        }
        let turn = Motion { linear: nalgebra::Matrix3::new(0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0), translation: Vector3::new(200.0, 10.0, 5.0) };
        let insts = vec![
            ExportInstance { part: 0, placement: Motion::identity(), name: "Box <1>".into() },
            ExportInstance { part: 0, placement: turn, name: "Box <2>".into() },
            ExportInstance { part: 1, placement: Motion::translation(Vector3::new(0.0, 0.0, 25.0)), name: "Pin <1>".into() },
        ];
        let bytes = k.export_model(ExchangeFormat::Step, "Two parts", &[(all[0].1, "Box".into()), (all[1].1, "Pin".into())], &insts).unwrap();
        std::fs::write(dir.join("assembly.step"), bytes).unwrap();
        std::fs::write(dir.join("expected.txt"), expected).unwrap();
    }

    /// Writers differ in the order of a placement's two representations (child first, as OCCT
    /// and we write, or parent first): both read to the same placement.
    #[test]
    fn assembly_placements_in_either_representation_order() {
        let mut k = NativeKernel::new();
        let a = extrude(&mut k, vec![region(poly(&[p(0., 0.), p(10., 0.), p(10., 10.), p(0., 10.)]), vec![])], 10.0);
        let turn = Motion { linear: nalgebra::Matrix3::new(0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0), translation: Vector3::new(50.0, 7.0, -3.0) };
        let insts = vec![ExportInstance { part: 0, placement: turn, name: "turned".into() }];
        let bytes = k.export_model(ExchangeFormat::Step, "asm", &[(a, "cube".into())], &insts).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        // Swap the representations and the transformation's items.
        let swap = |line: &str, kw: &str| -> String {
            let Some(i) = line.find(kw) else { return line.to_string() };
            let start = i + kw.len();
            let end = start + line[start..].find(')').unwrap();
            let args: Vec<&str> = line[start..end].split(',').collect();
            let (a, b) = (args[2], args[3]);
            format!("{}'','',{b},{a}{}", &line[..start], &line[end..])
        };
        let swapped: String = text
            .lines()
            .map(|l| swap(&swap(l, "(REPRESENTATION_RELATIONSHIP("), "ITEM_DEFINED_TRANSFORMATION("))
            .collect::<Vec<_>>()
            .join("\n");
        assert_ne!(swapped, text);
        for bytes in [text.as_bytes(), swapped.as_bytes()] {
            let m = k.import_model(ExchangeFormat::Step, bytes).unwrap();
            assert_eq!(m.occurrences.len(), 1);
            assert_eq!(m.occurrences[0].name, "turned");
            let pl = &m.occurrences[0].placement;
            assert!((pl.linear - turn.linear).norm() < 1e-12 && (pl.translation - turn.translation).norm() < 1e-12, "{pl:?}");
        }
    }

    #[test]
    fn import_step_splits_solids_and_places_occurrences() {
        let mut k = NativeKernel::new();
        let a = extrude(&mut k, vec![region(poly(&[p(0., 0.), p(10., 0.), p(10., 10.), p(0., 10.)]), vec![])], 10.0);
        let turn = Motion { linear: nalgebra::Matrix3::new(0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0), translation: Vector3::new(50.0, 0.0, 0.0) };
        let mirror = Motion::reflection(Point3::new(0.0, 0.0, 100.0), Vector3::z_axis());
        let insts = vec![
            ExportInstance { part: 0, placement: Motion::identity(), name: "a".into() },
            ExportInstance { part: 0, placement: turn, name: "b".into() },
            ExportInstance { part: 0, placement: mirror, name: "c".into() },
        ];
        let bytes = k.export_model(ExchangeFormat::Step, "asm", &[(a, "cube".into())], &insts).unwrap();
        let bs = k.import_step(&bytes).unwrap();
        assert_eq!(bs.len(), 3);
        let mut zs: Vec<f64> = bs.iter().map(|b| k.mass_properties(*b).unwrap().center_of_mass.z).collect();
        zs.sort_by(f64::total_cmp);
        assert!((zs[0] - 5.0).abs() < 1e-9 && (zs[2] - 195.0).abs() < 1e-9, "{zs:?}");
        for b in bs {
            assert!((k.mass_properties(b).unwrap().volume - 1000.0).abs() < 1e-9);
        }
        // IGES has no assemblies: each instance placed.
        let bytes = k.export_model(ExchangeFormat::Iges, "asm", &[(a, "cube".into())], &insts).unwrap();
        let m = k.import_model(ExchangeFormat::Iges, &bytes).unwrap();
        assert_eq!(m.parts.len(), 3);
        assert_eq!(m.parts[1].name, "b");
        let c = k.mass_properties(m.parts[1].body).unwrap().center_of_mass;
        assert!((c - Point3::new(45.0, 5.0, 5.0)).norm() < 1e-9, "{c}");
    }

    /// Non-finite numbers never reach a file, and never panic the writer (regression:
    /// `step::real` unwrapped the exponent of `{:e}`, and NaN coordinates were written as 0).
    #[test]
    fn non_finite_bodies_are_not_written() {
        for x in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e300, -1e-300, f64::MIN_POSITIVE, 5e-324] {
            let s = crate::native::step::real(x);
            assert!(s.contains('.'), "{x}: {s}");
        }
        let mut k = NativeKernel::new();
        let good = extrude(&mut k, vec![region(poly(&[p(0.0, 0.0), p(10.0, 0.0), p(10.0, 10.0)]), vec![])], 5.0);
        let mut b = Body::clone(k.get(good).unwrap());
        b.verts[0].p.x = f64::NAN;
        let bad = k.insert(b);
        for format in [ExchangeFormat::Step, ExchangeFormat::Iges] {
            assert!(export_model(&k, format, "x", &[(good, "good".into())], &[]).is_ok());
            assert!(matches!(export_model(&k, format, "x", &[(bad, "bad".into())], &[]), Err(KernelError::InvalidInput(_))));
        }
        assert!(matches!(export_step(&k, &[bad]), Err(KernelError::InvalidInput(_))));
        let nan = ExportInstance { part: 0, placement: Motion { linear: nalgebra::Matrix3::identity(), translation: Vector3::new(f64::NAN, 0.0, 0.0) }, name: "i".into() };
        assert!(matches!(export_model(&k, ExchangeFormat::Step, "x", &[(good, "good".into())], &[nan]), Err(KernelError::InvalidInput(_))));
    }
}
