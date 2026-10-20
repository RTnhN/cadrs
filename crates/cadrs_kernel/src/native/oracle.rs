//! A body for the SMLib test oracle (`tools/smlib_oracle`, on NVIDIA's usd-brep SMLib,
//! Apache-2.0), which runs the same operations on it to compare against: as an SMLib `.smb` file
//! ([`smb_text`]).
//!
//! The `.smb` is SMLib's version 42 layout (as `SmApiWriteBrepToFile` writes, its section
//! headers' comment lines included: the reader skips them by count): an infinite void region and
//! one solid region, a shell each holding a use of every face (the outer side facing out), faces
//! on B-spline surfaces (`Object Type 15002`) whose normals point out of the solid, loops of edge
//! uses, edges on B-spline curves (`Object Type 14033`) between vertices. An edge's two uses are
//! linked as SMLib's radial lists are: the first use's next is its own mate and its mate's next
//! the second use's mate; the second's next is the first, its mate's next itself. The UV trim
//! curves are cubics through points along each edge inverted onto the exported surface (a
//! rational arc's parameter isn't our angle; a seam's two uses each on its own side).


use std::fmt::Write;

use super::nurbs::{NurbsCurve, NurbsSurface};
use super::math::P3;
use super::topo::Body;




/// A face's surface over its p-curves' box (grown a little, inside the surface's range) as NURBS,
/// u reversed where the face faces against its surface (so its normal is the face's).
fn face_nurbs(b: &Body, f: &super::topo::Face) -> NurbsSurface {
    let mut uv = super::math::Bbox::empty();
    for c in f.loops.iter().flatten() {
        let e = &b.edges[c.edge];
        uv.union(&super::boolean::curve_bbox(&c.pc, e.t[0], e.t[1]));
    }
    let ((pu0, pu1), (pv0, pv1)) = f.surf.param_box();
    let (gu, gv) = ((uv.max.x - uv.min.x) * 0.01, (uv.max.y - uv.min.y) * 0.01);
    let u = if f.surf.u_periodic() { (uv.min.x, uv.max.x) } else { ((uv.min.x - gu).max(pu0), (uv.max.x + gu).min(pu1)) };
    let v = if f.surf.v_periodic() { (uv.min.y, uv.max.y) } else { ((uv.min.y - gv).max(pv0), (uv.max.y + gv).min(pv1)) };
    let mut ns = f.surf.to_nurbs(u, v);
    if !f.sense {
        let (a, z) = (ns.ku[0], *ns.ku.last().unwrap());
        ns.ku = ns.ku.iter().rev().map(|k| a + z - k).collect();
        ns.cps.reverse();
    }
    ns
}

/// The parameters on `s` of the point `p`, by Newton from `guess` (kept inside the range).
fn invert(s: &NurbsSurface, p: &P3, guess: (f64, f64)) -> (f64, f64) {
    let ((u0, u1), (v0, v1)) = s.range();
    let (mut u, mut v) = (guess.0.clamp(u0, u1), guess.1.clamp(v0, v1));
    for _ in 0..50 {
        let d = s.ders2(u, v);
        let r = d[0] - p.coords;
        let (su, sv) = (d[1], d[2]);
        let (a, b, c) = (su.dot(&su), su.dot(&sv), sv.dot(&sv));
        let det = a * c - b * b;
        if det.abs() < 1e-300 {
            break;
        }
        let (gu, gv) = (r.dot(&su), r.dot(&sv));
        let du = (c * gu - b * gv) / det;
        let dv = (a * gv - b * gu) / det;
        u = (u - du).clamp(u0, u1);
        v = (v - dv).clamp(v0, v1);
        if du.abs() < 1e-15 * (1.0 + u.abs()) && dv.abs() < 1e-15 * (1.0 + v.abs()) {
            break;
        }
    }
    (u, v)
}

/// A coedge's trim curve on the face's exported surface, on the edge's parameters (in the edge's
/// direction): points along the edge inverted onto the surface from the p-curve's (u, v) (u
/// reversed with the surface), a cubic through them.
fn uv_curve(b: &Body, f: &super::topo::Face, c: &super::topo::Coedge, s: &NurbsSurface) -> NurbsCurve {
    let e = &b.edges[c.edge];
    let (ua, uz) = (s.ku[0], *s.ku.last().unwrap());
    let n = 24;
    let ts: Vec<f64> = (0..=n).map(|i| e.t[0] + (e.t[1] - e.t[0]) * i as f64 / n as f64).collect();
    let pts: Vec<P3> = ts
        .iter()
        .map(|&t| {
            let q = c.pc.point(t);
            let guess = if f.sense { (q.x, q.y) } else { (ua + uz - q.x, q.y) };
            let (u, v) = invert(s, &e.point(t), guess);
            P3::new(u, v, 0.0)
        })
        .collect();
    NurbsCurve::interpolate(&pts, &ts, 3, None, None)
}

fn f64s(out: &mut String, xs: &[f64]) {
    for x in xs {
        let _ = writeln!(out, "{x:.17e}");
    }
}

/// The body as an SMLib `.smb` file (a closed solid; see the module notes).
pub fn smb_text(b: &Body) -> String {
    let tol = 1e-5f64;
    let mut out = String::new();
    let nf = b.faces.len();
    let _ = writeln!(out, "Version 42\nHealer Version 10\n//Brep Starts\n1.0 0.0 0.0 - Color\n{tol:.17e} - Brep ZoneTol3d\n1 - is Manifold:[1 = is, 0 = isn't]\n0 - has Composites:[1 = does, 0 = doesn't]\n");
    let _ = writeln!(out, "//Region Data\n2 Regions In Brep\n0 - Start Region\n//Index, Flags, UserId1, UserId2, Number Of Shells, First Shell, Is void Region\n0 0 0 0 1 0 1\n1 0 0 0 1 1 0\n");
    let _ = writeln!(out, "//Shell Data\n2 Shells In Brep\n//Index, Flags, UserId1, UserId2, 0(Faceuse), Orientation, Number Of Faceuses, Start Faceuse\n//Index, Flags, UserId1, UserId2, 1(WireEdge), Edge\n//Index, Flags, UserId1, UserId2, 2(Vertex), Vertex\n0 0 0 0 0 1 {nf} 0\n1 0 0 0 0 1 {nf} {nf}\n");
    let _ = writeln!(out, "//Composite Face Data\n0 Composite Faces In Brep\n");
    let _ = writeln!(out, "//Faceuse Data\n{} Faceuses In Brep\n//Index, Flags, UserId1, UserId2, Face, Orientation", 2 * nf);
    for i in 0..nf {
        let _ = writeln!(out, "{i} 0 0 0 {i} 1");
    }
    for i in 0..nf {
        let _ = writeln!(out, "{} 0 0 0 {i} 0", nf + i);
    }
    // Loops and edge uses (degenerate edges left out), each edge's uses in the order met.
    let mut loops: Vec<(usize, usize)> = Vec::new(); // (count, start)
    let mut uses: Vec<(usize, usize, bool)> = Vec::new(); // (edge, loop, along the edge)
    let mut uvs: Vec<NurbsCurve> = Vec::new(); // each use's trim curve
    let mut face_loops: Vec<(usize, usize)> = Vec::new(); // (first loop, count)
    let surfs: Vec<NurbsSurface> = b.faces.iter().map(|f| face_nurbs(b, f)).collect();
    for (fi, f) in b.faces.iter().enumerate() {
        face_loops.push((loops.len(), f.loops.len()));
        for l in &f.loops {
            let start = uses.len();
            for c in l {
                if !b.edges[c.edge].degenerate {
                    uses.push((c.edge, loops.len(), c.fwd));
                    uvs.push(uv_curve(b, f, c, &surfs[fi]));
                }
            }
            loops.push((uses.len() - start, start));
        }
    }
    let mut by_edge: Vec<Vec<usize>> = vec![Vec::new(); b.edges.len()];
    for (i, u) in uses.iter().enumerate() {
        by_edge[u.0].push(i);
    }
    let _ = writeln!(out, "\n//Face Data\n{nf} Faces In Brep\n//Index, Flags, UserId1, UserId2, Tol, Surface, Outer Loop, # of Loops, Naturally Trimmed\n//UMin, UMax, VMin, VMax");
    for (i, s) in surfs.iter().enumerate() {
        let (l0, nl) = face_loops[i];
        let _ = writeln!(out, "{i} 0 0 0 {tol:.17e} {i} {l0} {nl} 0");
        let _ = writeln!(out, "{:.17e} {:.17e} {:.17e} {:.17e}", s.ku[0], s.ku.last().unwrap(), s.kv[0], s.kv.last().unwrap());
    }
    let _ = writeln!(out, "\n//Loop Data\n{} Loops In Brep\n//Index, Flags, UserId1, UserId2, 0(Edgeuse), # Of Edgeuses, Start Edgeuse\n//Index, Flags, UserId1, UserId2, 1(Vertex), Vertex", loops.len());
    for (i, (n, s0)) in loops.iter().enumerate() {
        let _ = writeln!(out, "{i} 0 0 0 0 {n} {s0}");
    }
    let _ = writeln!(out, "\n//Composite Edge Data\n0 Composite Edges In Brep\n");
    let _ = writeln!(out, "//Edgeuse Data\n{} Edgeuses In Brep\n//Index, Flags, UserId1, UserId2, MateFlags, MateUserId1, MateUserId2, 0(Edge), Edge, Loop, Orientation, UV Curve, Next EU, Mate Next EU\n//Index, Flags, UserId1, UserId2, MateFlags, MateUserId1, MateUserId2, 1(Vertex), Vertex, Loop, Orientation, UV Curve\n//Index, Flags, UserId1, UserId2, MateFlags, MateUserId1, MateUserId2, 2(Wire Edge), Edge, Shell, Orientation, UV Curve\n//Index, Flags, UserId1, UserId2, MateFlags, MateUserId1, MateUserId2, 3(Vertex At Pole), Vertex, Loop, Orientation, UV Curve", uses.len());
    // Edges kept (not degenerate, used), renumbered.
    let kept: Vec<usize> = (0..b.edges.len()).filter(|&e| !b.edges[e].degenerate && !by_edge[e].is_empty()).collect();
    let mut edge_no = vec![usize::MAX; b.edges.len()];
    for (k, &e) in kept.iter().enumerate() {
        edge_no[e] = k;
    }
    for (i, &(e, l, fwd)) in uses.iter().enumerate() {
        let us = &by_edge[e];
        let first = us[0];
        let (next, mate_next): (i64, i64) = if us.len() == 2 {
            let second = us[1];
            if i == first { (-(i as i64) - 1, -(second as i64) - 1) } else { (first as i64, i as i64) }
        } else {
            // A lone use (an open sheet's boundary): its own radial list.
            (-(i as i64) - 1, i as i64)
        };
        let _ = writeln!(out, "{i} 0 0 0 0 0 0 0 {} {l} {} {i} {next} {mate_next}", edge_no[e], fwd as u8);
    }
    // Vertices used by kept edges, renumbered.
    let mut vert_no = vec![usize::MAX; b.verts.len()];
    let mut verts: Vec<usize> = Vec::new();
    for &e in &kept {
        for v in b.edges[e].v {
            if vert_no[v] == usize::MAX {
                vert_no[v] = verts.len();
                verts.push(v);
            }
        }
    }
    let _ = writeln!(out, "\n//Edge Data\n{} Edges In Brep\n//Index, Flags, UserId1, UserId2, Tol, Curve, Start Vertex, End Vertex, Primary Edgeuse, Param Min, Param Max", kept.len());
    for (k, &e) in kept.iter().enumerate() {
        let ed = &b.edges[e];
        let _ = writeln!(out, "{k} 0 0 0 {:.17e} {k} {} {} {} {:.17e} {:.17e}", ed.tol.max(tol), vert_no[ed.v[0]], vert_no[ed.v[1]], by_edge[e][0], ed.t[0], ed.t[1]);
    }
    let _ = writeln!(out, "\n//Vertex Data\n{} Vertices In Brep\n//Index, Flags, UserId1, UserId2, Tol, X, Y, Z", verts.len());
    for (k, &v) in verts.iter().enumerate() {
        let p = b.verts[v].p;
        let _ = writeln!(out, "{k} 0 0 0 {:.17e} {:.17e} {:.17e} {:.17e}", b.verts[v].tol.max(tol), p.x, p.y, p.z);
    }
    let _ = writeln!(out, "\n// UV-Curve Data\n{} UV-Curves In Brep", uvs.len());
    for (i, c) in uvs.iter().enumerate() {
        let _ = writeln!(out, "{i} - UV-Curve:\nObject Type 14033\nCurve Dim 2\n{} Number of Control Points \n{} Degree \n0 RationalFlag \n2 Dimension \n1 Knot Type Enum: 1=2d NonRational, 2=3d NonRational, 3=2d Rational, 4=3d Rational\n0 BSplineCurveForm Enum", c.cps.len() - 1, c.degree);
        for h in &c.cps {
            let _ = writeln!(out, "{:.17e} {:.17e}", h.x / h.w, h.y / h.w);
        }
        f64s(&mut out, &c.knots);
    }
    let _ = writeln!(out, "\n// 3D-Curve Data\n{} 3D-Curves In Brep", kept.len());
    for (k, &e) in kept.iter().enumerate() {
        let ed = &b.edges[e];
        let c = ed.curve.to_nurbs(ed.t[0], ed.t[1]);
        let rational = c.cps.iter().any(|h| (h.w - 1.0).abs() > 1e-15);
        let _ = writeln!(out, "{k} - 3D-Curve\nObject Type 14033\nCurve Dim 3\n{} Number of Control Points \n{} Degree \n{} RationalFlag \n3 Dimension \n1 Knot Type Enum: 1=2d NonRational, 2=3d NonRational, 3=2d Rational, 4=3d Rational\n0 BSplineCurveForm Enum", c.cps.len() - 1, c.degree, rational as u8);
        for h in &c.cps {
            if rational {
                let _ = writeln!(out, "{:.17e} {:.17e} {:.17e} {:.17e}", h.x, h.y, h.z, h.w);
            } else {
                let _ = writeln!(out, "{:.17e} {:.17e} {:.17e}", h.x, h.y, h.z);
            }
        }
        f64s(&mut out, &c.knots);
    }
    let _ = writeln!(out, "\n//Surface Data\n{nf} Surfaces In Brep");
    for (i, s) in surfs.iter().enumerate() {
        let rational = s.cps.iter().flatten().any(|h| (h.w - 1.0).abs() > 1e-15);
        let _ = writeln!(out, "{i} - Surface\nObject Type 15002\n{} {}\n{} {}\n{}\n1 BSplineSurface Knot Type Flag \n9 BSplineSurfaceForm Flag ", s.cps.len() - 1, s.cps[0].len() - 1, s.du, s.dv, rational as u8);
        for row in &s.cps {
            for h in row {
                if rational {
                    let _ = writeln!(out, "{:.17e} {:.17e} {:.17e} {:.17e}", h.x, h.y, h.z, h.w);
                } else {
                    let _ = writeln!(out, "{:.17e} {:.17e} {:.17e}", h.x, h.y, h.z);
                }
            }
        }
        f64s(&mut out, &s.ku);
        f64s(&mut out, &s.kv);
    }
    out
}
