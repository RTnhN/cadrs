//! Height maps (Z-maps): a grid of heights over XY.
//!
//! - **The part's** ([`HeightMap::of_triangles`]): the highest point of the part over each
//!   cell, for the drop-cutter ([`HeightMap::drop_cutter`]): how low a cutter of any shape
//!   ([`crate::tool::Tool::height_at`]) can go at a point without touching the part. The 3D
//!   finishing strategies and the gouge check use it.
//! - **The stock's** ([`HeightMap::block`]): the top of the material over each cell. The
//!   simulation ([`HeightMap::cut`]) lowers it under the cutter along each move: Verify's
//!   in-process stock.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::geom::{Aabb, P2, P3};
use crate::model::{height_on, point_in_tri2};
use crate::tool::Tool;

/// No material over the cell.
pub const EMPTY: f32 = f32::NEG_INFINITY;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeightMap {
    /// The centre of cell (0, 0).
    pub origin: P2,
    /// Cell size (mm).
    pub cell: f64,
    pub nx: usize,
    pub ny: usize,
    /// Row-major heights (`EMPTY` where there's nothing).
    pub z: Vec<f32>,
}

impl HeightMap {
    /// An empty map covering `min`–`max` with cells of `cell` (at most about 4 million cells:
    /// the cell grows for big areas).
    pub fn new(min: P2, max: P2, cell: f64) -> HeightMap {
        let (w, h) = ((max[0] - min[0]).max(cell), (max[1] - min[1]).max(cell));
        let mut cell = cell.max(1e-3);
        while (w / cell).ceil() * (h / cell).ceil() > 4.0e6 {
            cell *= 1.25;
        }
        let nx = (w / cell).ceil() as usize + 1;
        let ny = (h / cell).ceil() as usize + 1;
        HeightMap {
            origin: min,
            cell,
            nx,
            ny,
            z: vec![EMPTY; nx * ny],
        }
    }

    /// A block of stock: `bounds`' top over its footprint.
    pub fn block(bounds: &Aabb, cell: f64) -> HeightMap {
        let mut m = HeightMap::new(
            [bounds.min[0], bounds.min[1]],
            [bounds.max[0], bounds.max[1]],
            cell,
        );
        m.z.fill(bounds.max[2] as f32);
        m
    }

    /// The highest point of the triangles over each cell of a map covering `min`–`max`.
    pub fn of_triangles(tris: &[[P3; 3]], min: P2, max: P2, cell: f64) -> HeightMap {
        let mut m = HeightMap::new(min, max, cell);
        m.rasterize(tris);
        m
    }

    /// Raises cells to the triangles' heights (vertical walls through their edges).
    pub fn rasterize(&mut self, tris: &[[P3; 3]]) {
        let (nx, ny, cell, o) = (self.nx, self.ny, self.cell, self.origin);
        // Rows in parallel: each row takes the triangles whose y range covers it.
        let rows: Vec<Vec<f32>> = (0..ny)
            .into_par_iter()
            .map(|j| {
                let y = o[1] + j as f64 * cell;
                let mut row: Vec<f32> = self.z[j * nx..(j + 1) * nx].to_vec();
                for t in tris {
                    let (ymin, ymax) = (
                        t[0][1].min(t[1][1]).min(t[2][1]),
                        t[0][1].max(t[1][1]).max(t[2][1]),
                    );
                    if y < ymin - cell || y > ymax + cell {
                        continue;
                    }
                    let xmin = t[0][0].min(t[1][0]).min(t[2][0]);
                    let xmax = t[0][0].max(t[1][0]).max(t[2][0]);
                    let i0 = (((xmin - o[0]) / cell).floor().max(0.0)) as usize;
                    let i1 = ((((xmax - o[0]) / cell).ceil()) as isize).min(nx as isize - 1);
                    if i1 < 0 {
                        continue;
                    }
                    for (i, c) in row.iter_mut().enumerate().take(i1 as usize + 1).skip(i0) {
                        let p = [o[0] + i as f64 * cell, y];
                        if point_in_tri2(p, t)
                            && let Some(h) = height_on(t, p)
                        {
                            *c = c.max(h as f32);
                        }
                    }
                    // Edges: walls and slivers thinner than a cell still show.
                    for k in 0..3 {
                        let (a, b) = (t[k], t[(k + 1) % 3]);
                        if (a[1] - y).abs() >= cell
                            && (b[1] - y).abs() >= cell
                            && (a[1] - y).signum() == (b[1] - y).signum()
                        {
                            continue;
                        }
                        let n = (((b[0] - a[0]).abs().max((b[1] - a[1]).abs()) / (cell * 0.5))
                            .ceil() as usize)
                            .max(1);
                        for s in 0..=n {
                            let f = s as f64 / n as f64;
                            let q = [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f];
                            // The cells either side of the edge (on this row and the rows next
                            // to it): an edge never falls between cells, so a cutter dropped
                            // beside it can't slip past (a cell's worth on the safe side).
                            if ((q[1] - y) / cell).abs() >= 1.0 {
                                continue;
                            }
                            let x = (q[0] - o[0]) / cell;
                            let z = (a[2] + (b[2] - a[2]) * f) as f32;
                            for i in [x.floor(), x.ceil()] {
                                if i >= 0.0 && (i as usize) < nx {
                                    let c = &mut row[i as usize];
                                    *c = c.max(z);
                                }
                            }
                        }
                    }
                }
                row
            })
            .collect();
        for (j, r) in rows.into_iter().enumerate() {
            self.z[j * nx..(j + 1) * nx].copy_from_slice(&r);
        }
    }

    pub fn index(&self, i: usize, j: usize) -> usize {
        j * self.nx + i
    }

    pub fn at(&self, i: usize, j: usize) -> f32 {
        self.z[self.index(i, j)]
    }

    pub fn point(&self, i: usize, j: usize) -> P2 {
        [
            self.origin[0] + i as f64 * self.cell,
            self.origin[1] + j as f64 * self.cell,
        ]
    }

    /// The cell nearest to `p`, if it's on the map.
    pub fn cell_of(&self, p: P2) -> Option<(usize, usize)> {
        let i = ((p[0] - self.origin[0]) / self.cell).round();
        let j = ((p[1] - self.origin[1]) / self.cell).round();
        (i >= 0.0 && j >= 0.0 && (i as usize) < self.nx && (j as usize) < self.ny)
            .then_some((i as usize, j as usize))
    }

    /// The height at `p` (nearest cell; `EMPTY` off the map).
    pub fn sample(&self, p: P2) -> f32 {
        self.cell_of(p).map_or(EMPTY, |(i, j)| self.at(i, j))
    }

    /// The cutter's kernel: for each cell offset within its radius, how high its surface is
    /// above its tip there.
    fn kernel(&self, tool: &Tool) -> Vec<(isize, isize, f32)> {
        let r = tool.radius();
        let n = (r / self.cell).ceil() as isize;
        let mut k = Vec::new();
        for dj in -n..=n {
            for di in -n..=n {
                let d = ((di * di + dj * dj) as f64).sqrt() * self.cell;
                if let Some(h) = tool
                    .height_at(d.min(r))
                    .filter(|_| d <= r + 0.5 * self.cell)
                {
                    k.push((di, dj, h as f32));
                }
            }
        }
        k
    }

    /// How low the tool's tip can go at `p` without cutting into the map's surface (the
    /// map's lowest value when nothing is under it: `floor`).
    pub fn drop_cutter(&self, tool: &Tool, p: P2, floor: f64) -> f64 {
        let k = self.kernel(tool);
        self.drop_with(&k, p, floor)
    }

    fn drop_with(&self, k: &[(isize, isize, f32)], p: P2, floor: f64) -> f64 {
        let ci = ((p[0] - self.origin[0]) / self.cell).round() as isize;
        let cj = ((p[1] - self.origin[1]) / self.cell).round() as isize;
        let mut z = floor as f32;
        for &(di, dj, h) in k {
            let (i, j) = (ci + di, cj + dj);
            if i < 0 || j < 0 || i >= self.nx as isize || j >= self.ny as isize {
                continue;
            }
            let v = self.z[j as usize * self.nx + i as usize];
            if v != EMPTY {
                z = z.max(v - h);
            }
        }
        z as f64
    }

    /// [`HeightMap::drop_cutter`] at many points at once (in parallel).
    pub fn drop_many(&self, tool: &Tool, pts: &[P2], floor: f64) -> Vec<f64> {
        let k = self.kernel(tool);
        pts.par_iter()
            .map(|p| self.drop_with(&k, *p, floor))
            .collect()
    }

    /// Lowers the stock under the tool moving from `a` to `b` (tip positions). Returns the
    /// volume removed (mm³).
    pub fn cut(&mut self, tool: &Tool, a: P3, b: P3) -> f64 {
        let k = self.kernel(tool);
        self.cut_with(&k, a, b)
    }

    fn cut_with(&mut self, k: &[(isize, isize, f32)], a: P3, b: P3) -> f64 {
        let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let n = ((len / (self.cell * 0.5)).ceil() as usize).max(1);
        let area = (self.cell * self.cell) as f32;
        let mut removed = 0.0f32;
        for s in 0..=n {
            let f = s as f64 / n as f64;
            let p = [
                a[0] + (b[0] - a[0]) * f,
                a[1] + (b[1] - a[1]) * f,
                a[2] + (b[2] - a[2]) * f,
            ];
            let ci = ((p[0] - self.origin[0]) / self.cell).round() as isize;
            let cj = ((p[1] - self.origin[1]) / self.cell).round() as isize;
            for &(di, dj, h) in k {
                let (i, j) = (ci + di, cj + dj);
                if i < 0 || j < 0 || i >= self.nx as isize || j >= self.ny as isize {
                    continue;
                }
                let idx = j as usize * self.nx + i as usize;
                let surface = p[2] as f32 + h;
                let v = &mut self.z[idx];
                if *v != EMPTY && *v > surface {
                    removed += (*v - surface) * area;
                    *v = surface;
                }
            }
        }
        removed as f64
    }

    /// Lowers the stock along a whole motion (feed moves only; rapids are checked by
    /// [`crate::sim`]). Returns the volume removed.
    pub fn cut_motion(&mut self, tool: &Tool, segs: &[crate::toolpath::Segment]) -> f64 {
        let k = self.kernel(tool);
        segs.iter()
            .filter(|s| !s.is_rapid())
            .map(|s| self.cut_with(&k, s.from, s.to))
            .sum()
    }

    /// The material's volume over the map (mm³) above `base`.
    pub fn volume_above(&self, base: f64) -> f64 {
        let a = self.cell * self.cell;
        self.z
            .iter()
            .filter(|v| **v != EMPTY)
            .map(|v| (*v as f64 - base).max(0.0) * a)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::box_triangles;
    use crate::tool::ToolKind;

    #[test]
    fn rasterize_box_top() {
        let tris = box_triangles(&Aabb::new([2.0, 2.0, 0.0], [8.0, 8.0, 5.0]));
        let m = HeightMap::of_triangles(&tris, [0.0, 0.0], [10.0, 10.0], 0.25);
        assert_eq!(m.sample([5.0, 5.0]), 5.0);
        assert_eq!(m.sample([1.0, 1.0]), EMPTY);
    }

    #[test]
    fn drop_flat_and_ball() {
        let tris = box_triangles(&Aabb::new([0.0, 0.0, 0.0], [10.0, 10.0, 5.0]));
        let m = HeightMap::of_triangles(&tris, [-10.0, -10.0], [20.0, 20.0], 0.1);
        let flat = Tool::new(ToolKind::EndMill, 6.0);
        // Over the top, and overhanging the edge by less than the radius: on the top.
        assert!((m.drop_cutter(&flat, [5.0, 5.0], -1.0) - 5.0).abs() < 1e-6);
        assert!((m.drop_cutter(&flat, [12.0, 5.0], -1.0) - 5.0).abs() < 1e-6);
        // Clear of the box: down to the floor.
        assert_eq!(m.drop_cutter(&flat, [14.0, 5.0], -1.0), -1.0);
        let ball = Tool::new(ToolKind::BallMill, 6.0);
        // 2 mm past the edge, a 3 mm ball touches the edge 3 − √5 up its side.
        let z = m.drop_cutter(&ball, [12.0, 5.0], -1.0);
        assert!((z - (5.0 - (3.0 - 5f64.sqrt()))).abs() < 0.06, "{z}");
    }

    #[test]
    fn cut_removes_slot_volume() {
        let stock = Aabb::new([0.0, 0.0, 0.0], [40.0, 20.0, 10.0]);
        let mut m = HeightMap::block(&stock, 0.1);
        let before = m.volume_above(0.0);
        let t = Tool::new(ToolKind::EndMill, 6.0);
        let removed = m.cut(&t, [5.0, 10.0, 8.0], [35.0, 10.0, 8.0]);
        // A 30 mm slot 6 wide with round ends, 2 deep.
        let expect = (30.0 * 6.0 + std::f64::consts::PI * 9.0) * 2.0;
        assert!(
            (removed - expect).abs() / expect < 0.03,
            "{removed} vs {expect}"
        );
        assert!((before - m.volume_above(0.0) - removed).abs() < 1.0);
        assert!((m.sample([20.0, 10.0]) - 8.0).abs() < 1e-6);
    }
}
