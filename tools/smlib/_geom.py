"""Closed forms and numeric integrals for the SMLib sweeps and primitives suite modules (not a
suite: the driver skips modules starting with `_`).

SMLib's own sweep volume is 0 (it leaves the swept region flagged void), and its curved
primitives are spline approximations, so these suites take their volumes and areas from here:
Pappus for revolutions, zonotopes for boxes swept along a line, and Gauss-Legendre integrals
(to about 1e-10) where there is no closed form.
"""
import math

import numpy as np

# The test B-rep of `my_create_test_brep` (smSweepTest.cpp:192): the face of
# `my_test_tsurf_creation` (smTopoTest.cpp:1259), a 10 x 10 square less its top right quarter,
# with a triangular hole and a round one, split in two by an edge from (0,0) to (5,5); a vertex
# loop at (3.5,8.5) in it, a wire edge from (10,0) to (15,2) and a vertex shell at (-5,0,0).
L_OUTER = [(0, 0), (10, 0), (10, 5), (5, 5), (5, 10), (0, 10)]
TRIANGLE = [(1, 5), (4, 5), (1, 8)]
CIRCLE = ((6.0, 2.5), 1.5)  # starts at (4.5, 2.5) (its frame's x is -x)
# Face 0 after the split (the half with the triangle; the counts of sweeps 21, 23 and 24 tell
# which half it is).
UPPER = [(0, 0), (5, 5), (5, 10), (0, 10)]
DIAGONAL = ((0, 0), (5, 5))
WIRE = ((10, 0), (15, 2))
VERTEX_LOOP = (3.5, 8.5)
ACORN = (-5.0, 0.0)
# Its 13 vertices: the outer loop's 6, the triangle's 3, the circle's, the vertex loop, the wire's
# far end and the vertex shell.
VERTICES = [p + (0.0,) if len(p) == 2 else p for p in
            [(float(x), float(y)) for x, y in L_OUTER] + [(float(x), float(y)) for x, y in TRIANGLE]
            + [(4.5, 2.5), VERTEX_LOOP, (15.0, 2.0), ACORN]]


def poly_area_centroid(pts):
    """Signed area and centroid of a polygon."""
    a = cx = cy = 0.0
    n = len(pts)
    for i in range(n):
        (x0, y0), (x1, y1) = pts[i], pts[(i + 1) % n]
        c = x0 * y1 - x1 * y0
        a += c
        cx += (x0 + x1) * c
        cy += (y0 + y1) * c
    a /= 2
    return a, (cx / (6 * a), cy / (6 * a))


def poly_edges(pts):
    """(length, midpoint) of each side."""
    out = []
    for i in range(len(pts)):
        (x0, y0), (x1, y1) = pts[i], pts[(i + 1) % len(pts)]
        out.append((math.hypot(x1 - x0, y1 - y0), ((x0 + x1) / 2, (y0 + y1) / 2)))
    return out


class Region:
    """A planar region in z = 0: an outer polygon less polygon and circle holes. Everything
    linear over it (areas, first moments) comes from areas and centroids; everything over its
    boundary from lengths and curve centroids."""

    def __init__(self, outer, poly_holes=(), circle_holes=()):
        self.outer, self.poly_holes, self.circle_holes = outer, list(poly_holes), list(circle_holes)

    def pieces(self):
        """[(signed area, centroid)]: the outer polygon positive, the holes negative."""
        a, c = poly_area_centroid(self.outer)
        out = [(abs(a), c)]
        for h in self.poly_holes:
            a, c = poly_area_centroid(h)
            out.append((-abs(a), c))
        for c, r in self.circle_holes:
            out.append((-math.pi * r * r, c))
        return out

    def area(self):
        return sum(a for a, _ in self.pieces())

    def boundary(self):
        """[(length, centroid)] of the boundary curves."""
        out = poly_edges(self.outer)
        for h in self.poly_holes:
            out += poly_edges(h)
        for c, r in self.circle_holes:
            out.append((2 * math.pi * r, c))
        return out

    def perimeter(self):
        return sum(l for l, _ in self.boundary())


def test_region(part="all"):
    if part == "all":
        return Region(L_OUTER, [TRIANGLE], [CIRCLE])
    return Region(UPPER, [TRIANGLE])


def axis_distance(o, d):
    """The signed distance in z = 0 from the in-plane axis through o along d (linear)."""
    n = math.hypot(d[0], d[1])
    ux, uy = d[0] / n, d[1] / n
    return lambda p: (p[0] - o[0]) * uy - (p[1] - o[1]) * ux


def revolution(region, o, d, deg):
    """(volume, area) of a z = 0 region turned by deg about an in-plane axis it doesn't cross
    (Pappus): V = θ ∫ρ dA, A = θ ∫ρ dl over the boundary, plus the two caps below a whole turn."""
    s = axis_distance(o, d)
    sign = 1 if sum(a * s(c) for a, c in region.pieces()) >= 0 else -1
    th = math.radians(deg)
    vol = th * abs(sum(a * s(c) for a, c in region.pieces()))
    area = th * sum(l * sign * s(c) for l, c in region.boundary())
    if deg < 360:
        area += 2 * region.area()
    return vol, area


def zonotope(gens):
    """(volume, area) of the Minkowski sum of segments (a box along its three sides plus a
    sweep vector): V = Σ|det| over triples, A = 2 Σ|g_i × g_j| over pairs."""
    g = [np.array(x, float) for x in gens]
    vol = area = 0.0
    n = len(g)
    for i in range(n):
        for j in range(i + 1, n):
            area += 2 * np.linalg.norm(np.cross(g[i], g[j]))
            for k in range(j + 1, n):
                vol += abs(np.linalg.det(np.array([g[i], g[j], g[k]])))
    return vol, area


def gauss(f, a, b, panels=200, order=10):
    """∫_a^b f by composite Gauss-Legendre (f takes numpy arrays)."""
    x, w = np.polynomial.legendre.leggauss(order)
    edges = np.linspace(a, b, panels + 1)
    total = 0.0
    for lo, hi in zip(edges[:-1], edges[1:]):
        h = (hi - lo) / 2
        total += h * np.dot(w, f(lo + h * (x + 1)))
    return total


def sphere_turn_overlap(R, r, gap_deg):
    """Volume of a ball of radius r at distance R from an axis turned by 360° − gap about it
    (smSweepTest.cpp case 28: the sweep overlaps itself): the torus less the gap's wedge of
    it, plus the two end balls' parts in the gap. In the meridian plane, at (ρ, z) inside the
    disk, the ball at angle 0 covers |θ| ≤ a with cos a = (ρ² + R² + z² − r²)/(2Rρ), so the
    gap of width w is covered over min(w, 2a)."""
    w = math.radians(gap_deg)
    full = 2 * math.pi * math.pi * R * r * r

    x, wt = np.polynomial.legendre.leggauss(8)

    def nodes(a, b, panels):
        e = np.linspace(a, b, panels + 1)
        h = (e[1:] - e[:-1]) / 2
        pts = ((e[:-1] + h)[:, None] + h[:, None] * x[None, :]).ravel()
        ws = (h[:, None] * wt[None, :]).ravel()
        return pts, ws

    s, ws = nodes(0.0, r, 200)
    phi, wp = nodes(0.0, 2 * math.pi, 200)
    S, P = np.meshgrid(s, phi, indexing="ij")
    rho = R + S * np.cos(P)
    z = S * np.sin(P)
    c = np.clip((rho * rho + R * R + z * z - r * r) / (2 * R * rho), -1.0, 1.0)
    f = rho * (w - np.minimum(w, 2 * np.arccos(c))) * S
    lost_v = float(ws @ f @ wp)
    return full - lost_v


def box_corners(lo, size):
    return [np.array([lo[0] + size[0] * i, lo[1] + size[1] * j, lo[2] + size[2] * k], float)
            for i in (0, 1) for j in (0, 1) for k in (0, 1)]


BOX_EDGES = [(a, b) for a in range(8) for b in range(8) if a < b and bin(a ^ b).count("1") == 1]


def box_turn_volume(lo, size, o, d):
    """Volume swept by a box turned a whole turn about an axis that misses it: π ∫ (ρmax² −
    ρmin²) dh along the axis, the box's slice square to the axis at h being a convex polygon
    (ρmax at a corner of it, ρmin its distance from the axis)."""
    o = np.array(o, float)
    d = np.array(d, float) / np.linalg.norm(d)
    c = box_corners(lo, size)
    hs = sorted({float(np.dot(p - o, d)) for p in c})

    def slice_pts(h):
        pts = []
        for a, b in BOX_EDGES:
            ha, hb = np.dot(c[a] - o, d), np.dot(c[b] - o, d)
            if (ha - h) * (hb - h) <= 0 and ha != hb:
                t = (h - ha) / (hb - ha)
                pts.append(c[a] + t * (c[b] - c[a]))
        return pts

    def radial(p):
        w = p - o
        return w - d * np.dot(w, d)

    def f(hv):
        out = []
        for h in hv:
            pts = [radial(p) for p in slice_pts(h)]
            if len(pts) < 3:
                out.append(0.0)
                continue
            rmax2 = max(np.dot(p, p) for p in pts)
            # The slice as a convex polygon (sorted about its centre); ρmin: the axis (the
            # origin here) to it.
            cen = sum(pts) / len(pts)
            e1 = pts[0] - cen
            e1 /= np.linalg.norm(e1)
            e2 = np.cross(d, e1)
            pts.sort(key=lambda p: math.atan2(np.dot(p - cen, e2), np.dot(p - cen, e1)))
            best = min(np.dot(p, p) for p in pts)
            for i in range(len(pts)):
                a, b = pts[i], pts[(i + 1) % len(pts)]
                ab = b - a
                t = np.clip(-np.dot(a, ab) / max(np.dot(ab, ab), 1e-300), 0.0, 1.0)
                q = a + t * ab
                best = min(best, np.dot(q, q))
            out.append(rmax2 - best)
        return np.array(out)

    return math.pi * sum(gauss(f, a, b, panels=400, order=6) for a, b in zip(hs[:-1], hs[1:]))


def interval_sub(iv, cut):
    """Intervals iv less the interval cut."""
    a, b = cut
    out = []
    for lo, hi in iv:
        if b <= lo or a >= hi:
            out.append((lo, hi))
            continue
        if a > lo:
            out.append((lo, a))
        if b < hi:
            out.append((b, hi))
    return out


def tsurf_intervals(y, dx=0.0, dy=0.0):
    """The test face (with its holes), moved by (dx, dy), on the line at height y."""
    y = y - dy
    if y < 0 or y > 10:
        return []
    iv = [(0.0, 10.0)] if y < 5 else [(0.0, 5.0)]
    if 5 <= y <= 8:
        iv = interval_sub(iv, (1.0, 1.0 + (8 - y)))
    (cx, cy), r = CIRCLE
    if abs(y - cy) < r:
        h = math.sqrt(r * r - (y - cy) ** 2)
        iv = interval_sub(iv, (cx - h, cx + h))
    return [(lo + dx, hi + dx) for lo, hi in iv]


def tsurf_union_area(dx, dy):
    """Area of the test face ∪ itself moved by (dx, dy) (`Boolean2D` SM_2D_UNION in
    my_primsol_demo case 2): 2A − area of the common part, the latter integrated along y."""
    def common(yv):
        out = []
        for y in yv:
            a, b = tsurf_intervals(y), tsurf_intervals(y, dx, dy)
            out.append(sum(max(0.0, min(h1, h2) - max(l1, l2)) for l1, h1 in a for l2, h2 in b))
        return np.array(out)
    (cx, cy), r = CIRCLE
    cuts = sorted({0, 5, 8, 10, cy - r, cy, cy + r} | {dy + v for v in (0, 5, 8, 10, cy - r, cy, cy + r)})
    inter = sum(gauss(common, a, b, panels=100, order=10) for a, b in zip(cuts[:-1], cuts[1:]))
    return 2 * test_region().area() - inter


def oblique_prism(region, v):
    """(volume, area) of a z = 0 region swept along v: V = A v_z, sides |t × v| per unit
    length of boundary (a circle's side integrated round it)."""
    v = np.array(v, float)
    vol = abs(region.area() * v[2])
    side = 0.0
    pts_list = [region.outer] + region.poly_holes
    for pts in pts_list:
        for i in range(len(pts)):
            p0 = np.array([pts[i][0], pts[i][1], 0.0])
            p1 = np.array([pts[(i + 1) % len(pts)][0], pts[(i + 1) % len(pts)][1], 0.0])
            side += np.linalg.norm(np.cross(p1 - p0, v))
    for c, r in region.circle_holes:
        side += gauss(lambda th: r * np.linalg.norm(np.cross(np.stack([-np.sin(th), np.cos(th), 0 * th], 1), v), axis=1),
                      0.0, 2 * math.pi, panels=50, order=10)
    return vol, side + 2 * region.area()


def bspline_to_bezier(degree, knots, pts):
    """The Bézier pieces (lists of poles) of a clamped B-spline, by knot insertion."""
    knots = list(knots)
    pts = [np.array(p, float) for p in pts]
    inner = sorted(set(knots[degree + 1:-degree - 1]))
    for u in inner:
        while knots.count(u) < degree:
            k = max(i for i in range(len(knots) - 1) if knots[i] <= u < knots[i + 1])
            new = []
            for i in range(len(pts) + 1):
                if i <= k - degree:
                    new.append(pts[i])
                elif i > k:
                    new.append(pts[i - 1])
                else:
                    a = (u - knots[i]) / (knots[i + degree] - knots[i])
                    new.append((1 - a) * pts[i - 1] + a * pts[i])
            pts = new
            knots.insert(k + 1, u)
    n = len(set(knots)) - 1
    return [pts[i * degree:i * degree + degree + 1] for i in range(n)]


def bezier_length(poles):
    def speed(t):
        n = len(poles) - 1
        out = []
        for s in t:
            d = sum(n * math.comb(n - 1, i) * s ** i * (1 - s) ** (n - 1 - i) * (poles[i + 1] - poles[i]) for i in range(n))
            out.append(np.linalg.norm(d))
        return np.array(out)
    return gauss(speed, 0.0, 1.0, panels=50, order=10)
