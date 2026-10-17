//! Building bodies from profiles: planar faces, prisms (extrudes).

use super::math::*;
use super::topo::*;
use crate::Origin;

/// A body and the origins of its faces (a face may have several: a merged cap).
#[derive(Debug, Clone, Default)]
pub struct Built {
    pub body: Body,
    pub origins: Vec<(usize, Origin)>,
}

/// The extrude depth below which the kernel refuses to build (mm).
pub const MIN_DEPTH: f64 = 1e-6;

/// The map from sketch coordinates (z = 0) to model space at `s` along `dir`.
pub fn plane_map(plane: &crate::Plane, dir: &V3, s: f64) -> Affine {
    let (x, y, n) = (plane.x_dir.into_inner(), plane.y_dir().into_inner(), plane.normal.into_inner());
    Affine { m: nalgebra::Matrix3::from_columns(&[x, y, n]), t: plane.origin.coords + dir * s }
}

pub fn plane_frame(plane: &crate::Plane, offset: V3) -> Frame {
    Frame { o: plane.origin + offset, x: plane.x_dir.into_inner(), y: plane.y_dir().into_inner(), z: plane.normal.into_inner() }
}

/// Signed area in uv of a face's first loop, by sampling its p-curves.
pub fn loop_uv_area(body: &Body, l: &[Coedge]) -> f64 {
    let mut pts: Vec<P2> = Vec::new();
    for c in l {
        let e = &body.edges[c.edge];
        let (a, b) = co_range(e, c.fwd);
        let n = 32;
        for i in 0..n {
            let t = a + (b - a) * i as f64 / n as f64;
            let p = c.pc.point(t);
            pts.push(P2::new(p.x, p.y));
        }
    }
    polygon_area(&pts)
}

/// Sets each face's sense from the uv orientation of its outer loop (loops run CCW about the
/// outward normal, so an outer loop that is CCW in uv means outward = Su × Sv).
pub fn fix_senses(body: &mut Body, faces: impl IntoIterator<Item = usize>) {
    for fi in faces {
        let a = loop_uv_area(body, &body.faces[fi].loops[0]);
        body.faces[fi].sense = a > 0.0;
    }
}

