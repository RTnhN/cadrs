//! Body sweeps: points, wires and sheets swept one kind higher (Phase 1b).

use super::{Prism, SideGeo, SweepKind, Turn, side_loop};
use crate::native::build::{Built, MIN_DEPTH, fix_senses};
use crate::native::curve::Curve;
use crate::native::math::*;
use crate::native::profile::{Piece, Src};
use crate::native::tol;
use crate::native::topo::*;
use crate::{BodyClass, KernelError, Origin, Result};

/// How [`sweep_body`] moves a body: s runs from 0 along `Straight`'s unit direction (mm) or
/// about a [`Turn`]'s axis (radians).
pub enum BodyMove<'a> {
    Straight(V3),
    Turn(&'a Turn),
}

impl BodyMove<'_> {
    fn place(&self, s: f64) -> Affine {
        match self {
            BodyMove::Straight(d) => Affine {
                m: nalgebra::Matrix3::identity(),
                t: d * s,
            },
            BodyMove::Turn(t) => t.rot(s),
        }
    }
    fn rail(&self, p: &P3) -> Option<Curve> {
        match self {
            BodyMove::Straight(d) => Some(Curve::Line { o: *p, d: *d }),
            BodyMove::Turn(t) => t.rail_of(p),
        }
    }
    fn side(&self, piece: &Piece, c3: &Curve) -> Option<SideGeo> {
        match self {
            BodyMove::Straight(d) => {
                // A line along the sweep sweeps no area.
                if let Curve::Line { d: ld, .. } = c3
                    && ld.cross(d).norm() <= tol::REL_FINE * ld.norm()
                {
                    return None;
                }
                Prism {
                    plane: crate::Plane::top(),
                    dir: *d,
                }
                .side(piece, c3)
            }
            BodyMove::Turn(t) => t.side_of(piece, c3),
        }
    }
    /// The direction a point starts moving in.
    fn velocity(&self, p: &P3) -> V3 {
        match self {
            BodyMove::Straight(d) => *d,
            BodyMove::Turn(t) => t.f.z.cross(&(p - t.f.o)),
        }
    }
}

/// Sweeps a body one kind higher (Phase 1b): its points into edges (a wire body), its wires
/// into faces (a sheet), or its sheet into a solid (the sheet at both ends and the faces its
/// boundary edges sweep), from s = 0 to `s1`; `whole`: a whole turn (no end copies; the start
/// edges are seams). The input's vertices and edges keep their indices in the result (the
/// sweep's start); side faces are generated `FromEdge` of the edge of body `source` that swept
/// them, a sheet's caps `StartCap`/`EndCap` (region 0).
///
/// A sheet must cross the sweep the same way everywhere (every point of it leaves the same side
/// of it), else the solid would fold through itself: an `InvalidParameter` error.
pub fn sweep_body(b: &Body, mv: &BodyMove, s1: f64, whole: bool, source: u64) -> Result<Built> {
    if !whole && s1 < MIN_DEPTH {
        return Err(KernelError::InvalidParameter(format!(
            "The sweep's length {s1:e} is below the modelling tolerance ({MIN_DEPTH:e} mm)"
        )));
    }
    let class = b.class();
    let m1 = mv.place(s1);
    let mut out = Built::default();
    match class {
        BodyClass::Point => {
            let o = &mut out.body;
            for &v in &b.points {
                let p = b.verts[v].p;
                let rail = mv.rail(&p).ok_or_else(|| {
                    KernelError::InvalidParameter("A point on the axis sweeps nothing".into())
                })?;
                let v0 = o.add_vertex(p);
                let v1 = if whole {
                    v0
                } else {
                    o.add_vertex(m1.point(&p))
                };
                let e = o.add_edge(rail, 0.0, s1, v0, v1);
                o.wires.push(Wire {
                    edges: vec![(e, true)],
                });
            }
            return Ok(out);
        }
        BodyClass::Wire | BodyClass::Sheet => {}
        BodyClass::Solid => {
            return Err(KernelError::InvalidParameter(
                "A solid can't be swept one kind higher (sweep its faces, or unite its copies)"
                    .into(),
            ));
        }
        BodyClass::Empty => {
            return Err(KernelError::InvalidParameter(
                "The body to sweep is empty".into(),
            ));
        }
        BodyClass::Mixed => {
            return Err(KernelError::InvalidParameter(
                "A sweep takes one kind of body: points, wires or a sheet".into(),
            ));
        }
    }
    let sheet = class == BodyClass::Sheet;
    // The edges swept, each with the way its wire (or its sheet's face) runs along it.
    let swept: Vec<(usize, bool)> = if sheet {
        let mut uses: Vec<Vec<bool>> = vec![Vec::new(); b.edges.len()];
        for c in b.faces.iter().flat_map(|f| f.loops.iter().flatten()) {
            uses[c.edge].push(c.fwd);
        }
        (0..b.edges.len())
            .filter(|&e| uses[e].len() == 1 && !b.edges[e].degenerate)
            .map(|e| (e, uses[e][0]))
            .collect()
    } else {
        b.wires
            .iter()
            .flat_map(|w| w.edges.iter().copied())
            .collect()
    };
    if sheet {
        check_transversal(b, mv)?;
    }
    let o = &mut out.body;
    // The start: the body's vertices and edges as they are.
    o.verts = b.verts.clone();
    o.edges = b.edges.clone();
    let nv = b.verts.len();
    let mut on_edge = vec![false; nv];
    for e in &b.edges {
        on_edge[e.v[0]] = true;
        on_edge[e.v[1]] = true;
    }
    let mut swept_v = vec![false; nv];
    for &(e, _) in &swept {
        swept_v[b.edges[e].v[0]] = true;
        swept_v[b.edges[e].v[1]] = true;
    }
    // The end vertices (the same on the axis or for a whole turn) and the rails.
    let mut top_v: Vec<usize> = (0..nv).collect();
    let mut rail_e: Vec<Option<(usize, bool)>> = vec![None; nv];
    for v in 0..nv {
        if !on_edge[v] {
            continue;
        }
        let p = b.verts[v].p;
        let rail = mv.rail(&p);
        if !whole && rail.is_some() {
            top_v[v] = o.add_vertex(m1.point(&p));
            o.verts[top_v[v]].tol = b.verts[v].tol;
        }
        if swept_v[v] {
            rail_e[v] = Some(match rail {
                Some(c) => (o.add_edge(c, 0.0, s1, v, top_v[v]), false),
                None => (o.add_degenerate(p, 0.0, s1, v), true),
            });
        }
    }
    // The end edges: every edge of a sheet (its end copy), the swept edges of wires.
    let ne = b.edges.len();
    let mut top_e: Vec<usize> = (0..ne).collect();
    if !whole {
        let mut need = vec![sheet; ne];
        for &(e, _) in &swept {
            need[e] = true;
        }
        for e in (0..ne).filter(|&e| need[e]) {
            let mut t = b.edges[e].clone();
            t.curve = t.curve.transformed(&m1);
            t.v = [top_v[t.v[0]], top_v[t.v[1]]];
            o.edges.push(t);
            top_e[e] = o.edges.len() - 1;
        }
    }
    let mut faces = Vec::new();
    for &(e, fwd) in &swept {
        let ed = &b.edges[e];
        let piece = Piece {
            c: ed.curve.clone(),
            t0: ed.t[0],
            t1: ed.t[1],
            rev: false,
            src: Src::Edge {
                body: source,
                edge: e as u64,
            },
            more: Vec::new(),
        };
        let Some(side) = mv.side(&piece, &ed.curve) else {
            continue;
        };
        let (vs, ve) = if fwd {
            (ed.v[0], ed.v[1])
        } else {
            (ed.v[1], ed.v[0])
        };
        let (ts, te) = if fwd {
            (ed.t[0], ed.t[1])
        } else {
            (ed.t[1], ed.t[0])
        };
        let (Some(rs), Some(re)) = (rail_e[vs], rail_e[ve]) else {
            continue;
        };
        let sl = side_loop(&side, (e, top_e[e]), fwd, (rs, re), (0.0, s1), (ts, te));
        o.faces.push(Face {
            surf: side.surf,
            sense: true,
            loops: vec![sl],
        });
        let fi = o.faces.len() - 1;
        faces.push(fi);
        out.origins.push((
            fi,
            Origin::FromEdge {
                body: crate::BodyId(source),
                edge: crate::EdgeId(e as u64),
            },
        ));
    }
    fix_senses(o, faces.clone());
    if faces.is_empty() {
        return Err(KernelError::InvalidParameter(
            "The sweep makes no faces (every edge lies along it)".into(),
        ));
    }
    if !sheet {
        // One open shell per piece of edge-connected faces (disjoint wires sweep apart).
        for piece in o.face_components(&faces) {
            o.shells.push(Shell {
                faces: piece,
                closed: false,
            });
        }
        out.origins.sort_by_key(|(f, _)| *f);
        return Ok(out);
    }
    if !whole {
        for f in &b.faces {
            let mut bottom = f.clone();
            Body::reverse_face(&mut bottom);
            o.faces.push(bottom);
            faces.push(o.faces.len() - 1);
            out.origins
                .push((o.faces.len() - 1, Origin::StartCap { region: 0 }));
            let mut top = f.clone();
            top.surf = f.surf.transformed(&m1);
            for c in top.loops.iter_mut().flatten() {
                c.edge = top_e[c.edge];
            }
            o.faces.push(top);
            faces.push(o.faces.len() - 1);
            out.origins
                .push((o.faces.len() - 1, Origin::EndCap { region: 0 }));
        }
    }
    // Built facing the way the sheet faces at the start: turn it out.
    let whole_body = crate::native::kernel::subset(o, &faces, true);
    if crate::native::mass::body_volume(&whole_body) < 0.0 {
        for f in &mut o.faces {
            Body::reverse_face(f);
        }
    }
    assemble_solids(o, &faces)?;
    out.origins.sort_by_key(|(f, _)| *f);
    Ok(out)
}

/// Makes the closed shells and solids of `faces` (outward-facing): one closed shell per piece of
/// edge-connected faces; a piece enclosing a positive volume bounds a solid, one enclosing a
/// negative volume is a void of the smallest such solid around it (a profile's hole turned a
/// whole turn about an axis outside it sweeps a tunnel, not a hole).
fn assemble_solids(o: &mut Body, faces: &[usize]) -> Result<()> {
    let pieces = o.face_components(faces);
    let vols: Vec<f64> = pieces
        .iter()
        .map(|p| crate::native::mass::body_volume(&crate::native::kernel::subset(o, p, true)))
        .collect();
    let mut outers: Vec<usize> = (0..pieces.len()).filter(|&k| vols[k] > 0.0).collect();
    outers.sort_by(|&a, &b| vols[a].total_cmp(&vols[b]));
    let mut solids: Vec<Vec<usize>> = outers.iter().map(|&k| vec![k]).collect();
    for k in (0..pieces.len()).filter(|&k| !(vols[k] > 0.0)) {
        let Some(p) = pieces[k]
            .iter()
            .flat_map(|&f| o.faces[f].loops.iter().flatten())
            .next()
            .map(|c| o.verts[o.co_start(c)].p)
        else {
            continue;
        };
        // The smallest solid holding it.
        let home = outers.iter().position(|&s| {
            let sub = crate::native::kernel::subset(o, &pieces[s], true);
            let polys: Vec<_> = sub
                .faces
                .iter()
                .map(|f| crate::native::classify::face_polygons(&sub, f))
                .collect();
            crate::native::classify::parity_inside(&sub, &polys, &p)
        });
        let Some(h) = home else {
            return Err(KernelError::InvalidGeometry {
                op: "sweep",
                why: "The sweep makes a cavity outside its solid".into(),
            });
        };
        solids[h].push(k);
    }
    for sol in solids {
        let first = o.shells.len();
        for &k in &sol {
            o.shells.push(Shell {
                faces: pieces[k].clone(),
                closed: true,
            });
        }
        o.solids.push(Solid {
            shells: (first..first + sol.len()).collect(),
        });
    }
    Ok(())
}

/// Refuses a sheet that doesn't cross the sweep the same way everywhere (sampled on its
/// tessellation): its solid would fold through itself, or (a sheet along the sweep) be flat.
fn check_transversal(b: &Body, mv: &BodyMove) -> Result<()> {
    let size = b.bbox().diagonal().max(tol::ROUNDOFF);
    let (meshes, _) = crate::native::tess::mesh_body(b, size * 1e-2, 0.5);
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for m in &meshes {
        for (p, n) in m.positions.iter().zip(&m.normals) {
            let v = mv.velocity(p);
            if v.norm() < tol::REL_FINE * size {
                continue;
            }
            let d = v.normalize().dot(&n.normalize());
            lo = lo.min(d);
            hi = hi.max(d);
        }
    }
    const EPS: f64 = 1e-6;
    if hi > EPS && lo < -EPS {
        return Err(KernelError::InvalidParameter(
            "The sheet folds back across the sweep: its solid would cross itself".into(),
        ));
    }
    if !(hi > EPS || lo < -EPS) {
        return Err(KernelError::InvalidParameter(
            "The sheet lies along the sweep: it sweeps no volume".into(),
        ));
    }
    Ok(())
}
