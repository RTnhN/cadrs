"""Independent model of OCCT's case_9 2D booleans (edges and wires of lines, arcs and one rational
B-spline in the plane z = 0), with the overlap the line and the B-spline really have.

Reads the dataset's BRep files directly (only what these files use), splits every edge where it
crosses, touches or starts/stops running within TOL of an edge of the other argument, classifies
each piece by its middle point, and applies the DRAW command's semantics:
  common a b: the pieces of a on b; cut a b: the pieces of a off b;
  fuse a b: every piece of a, and b's pieces off a (b's pieces on a too when both are wires).
Counts: vertices are distinct piece ends, edges distinct pieces; length sums the pieces kept."""
import math, sys, re

TOL = 1e-7

def read(path):
    toks = open(path).read().split()
    i = toks.index('Locations')
    nl = int(toks[i + 1]); i += 2
    locs = [(0.0, 0.0)]
    for _ in range(nl):
        if toks[i] == '1':
            v = [float(x) for x in toks[i + 1:i + 13]]
            assert v[0] == v[5] == v[10] == 1.0  # translations only in these files
            locs.append((v[3], v[7])); i += 13
        else:
            assert toks[i] == '2'
            i += 1; x = y = 0.0
            while toks[i] != '0':
                l, pw = int(toks[i]), int(toks[i + 1]); i += 2
                x += pw * locs[l][0]; y += pw * locs[l][1]
            i += 1
            locs.append((x, y))
    i = toks.index('Curves', i)
    nc = int(toks[i + 1]); i += 2
    curves = []
    for _ in range(nc):
        t = toks[i]
        if t == '1':
            v = [float(x) for x in toks[i + 1:i + 7]]; i += 7
            curves.append(('line', v[0], v[1], v[3], v[4]))
        elif t == '2':
            v = [float(x) for x in toks[i + 1:i + 14]]; i += 14
            curves.append(('circle', v[0], v[1], v[6], v[7], v[9], v[10], v[12]))
        elif t == '7':
            rat, per, deg, npl, nk = (int(x) for x in toks[i + 1:i + 6]); i += 6
            poles = []
            for _ in range(npl):
                x, y = float(toks[i]), float(toks[i + 1]); i += 3
                w = 1.0
                if rat:
                    w = float(toks[i]); i += 1
                poles.append((x, y, w))
            knots = []
            for _ in range(nk):
                knots += [float(toks[i])] * int(toks[i + 1]); i += 2
            curves.append(('bspline', deg, poles, knots))
        else:
            raise ValueError(t)
    i = toks.index('TShapes', i)
    edges = []
    j = i
    while j < len(toks):
        if toks[j] == 'Ed':
            # tol, same param, same range, degenerate; then reps to 0
            k = j + 5
            assert toks[k] == '1', toks[j:j + 12]
            c, l, a, b = int(toks[k + 1]), int(toks[k + 2]), float(toks[k + 3]), float(toks[k + 4])
            assert l == 0
            edges.append((c - 1, a, b))
        j += 1
    # the root reference is the last "+n l" before the trailing 0
    m = re.findall(r'[+-]\d+\s+(\d+)\s*\n\s*0\s*$', open(path).read())
    root_loc = int(m[-1]) if m else 0
    dx, dy = locs[root_loc]
    is_wire = ' Wi' in ' ' + ' '.join(toks[i:])
    out = []
    for c, a, b in edges:
        out.append(Edge(curves[c], a, b, dx, dy))
    return out, is_wire

class Edge:
    def __init__(self, c, a, b, dx, dy):
        self.c, self.a, self.b, self.dx, self.dy = c, a, b, dx, dy
    def pt(self, t):
        c = self.c
        if c[0] == 'line':
            x, y = c[1] + c[3] * t, c[2] + c[4] * t
        elif c[0] == 'circle':
            x = c[1] + c[7] * (math.cos(t) * c[3] + math.sin(t) * c[5])
            y = c[2] + c[7] * (math.cos(t) * c[4] + math.sin(t) * c[6])
        else:
            x, y = bspline(c, t)
        return (x + self.dx, y + self.dy)

def bspline(c, t):
    _, p, poles, K = c
    n = len(poles)
    # de Boor on homogeneous points
    k = max(i for i in range(p, n) if K[i] <= t) if t < K[n] else n - 1
    d = [(poles[j][0] * poles[j][2], poles[j][1] * poles[j][2], poles[j][2]) for j in range(k - p, k + 1)]
    for r in range(1, p + 1):
        for j in range(p, r - 1, -1):
            i = j + k - p
            al = (t - K[i]) / (K[i + p + 1 - r] - K[i])
            d[j] = tuple((1 - al) * d[j - 1][q] + al * d[j][q] for q in range(3))
    return d[p][0] / d[p][2], d[p][1] / d[p][2]

def dist_seg(p, e):
    """Distance from p to edge e (sampled finely, then refined by golden section)."""
    n = 400
    ts = [e.a + (e.b - e.a) * i / n for i in range(n + 1)]
    best = min(range(n + 1), key=lambda i: math.dist(p, e.pt(ts[i])))
    lo, hi = ts[max(best - 1, 0)], ts[min(best + 1, n)]
    f = lambda t: math.dist(p, e.pt(t))
    g = (math.sqrt(5) - 1) / 2
    for _ in range(80):
        c, d = hi - g * (hi - lo), lo + g * (hi - lo)
        if f(c) < f(d): hi = d
        else: lo = c
    return f((lo + hi) / 2)

def splits(e, others):
    """Parameters of e where it crosses, touches, or starts/stops running along an edge of others."""
    n = 3000
    ts = [e.a + (e.b - e.a) * i / n for i in range(n + 1)]
    out = []
    for o in others:
        # only lines among the other side's edges meet the B-spline in these files; arcs too are
        # handled by the generic distance
        if o.c[0] == 'line':
            ux, uy = o.c[3], o.c[4]
            nrm = math.hypot(ux, uy); ux, uy = ux / nrm, uy / nrm
            o0 = o.pt(o.a)
            o1 = o.pt(o.b)
            lo_s, hi_s = sorted(((o0[0] - o0[0]) * ux + (o0[1] - o0[1]) * uy, (o1[0] - o0[0]) * ux + (o1[1] - o0[1]) * uy))
            def sd(t):
                p = e.pt(t)
                return (p[0] - o0[0]) * (-uy) + (p[1] - o0[1]) * ux
            def along(t):
                p = e.pt(t)
                return (p[0] - o0[0]) * ux + (p[1] - o0[1]) * uy
            def on(t):
                # Distance to the segment: past an end, to that end.
                s = along(t)
                if s < lo_s:
                    return math.hypot(sd(t), lo_s - s) <= TOL
                if s > hi_s:
                    return math.hypot(sd(t), s - hi_s) <= TOL
                return abs(sd(t)) <= TOL
        else:
            def on(t, o=o):
                return dist_seg(e.pt(t), o) <= TOL
            sd = None
        st = [on(t) for t in ts]
        for i in range(1, len(ts)):
            if st[i] != st[i - 1]:
                lo, hi = ts[i - 1], ts[i]
                for _ in range(80):
                    m = (lo + hi) / 2
                    if on(m) == st[i - 1]: lo = m
                    else: hi = m
                out.append((lo + hi) / 2)
        if sd is not None:
            vals = [sd(t) for t in ts]
            for i in range(1, len(ts)):
                if vals[i - 1] * vals[i] < 0 and not st[i - 1] and not st[i]:
                    lo, hi = ts[i - 1], ts[i]
                    for _ in range(80):
                        m = (lo + hi) / 2
                        if sd(m) * vals[i - 1] > 0: lo = m
                        else: hi = m
                    t = (lo + hi) / 2
                    s = along(t)
                    if lo_s - TOL <= s <= hi_s + TOL:
                        out.append(t)
        # the other edge's ends on e
        for q in (o.pt(o.a), o.pt(o.b)):
            i = min(range(len(ts)), key=lambda i: math.dist(q, e.pt(ts[i])))
            lo, hi = ts[max(i - 1, 0)], ts[min(i + 1, n)]
            f = lambda t: math.dist(q, e.pt(t))
            g = (math.sqrt(5) - 1) / 2
            for _ in range(80):
                c, d = hi - g * (hi - lo), lo + g * (hi - lo)
                if f(c) < f(d): hi = d
                else: lo = c
            t = (lo + hi) / 2
            if f(t) <= TOL:
                out.append(t)
    return out

def pieces(edges, others):
    res = []
    for e in edges:
        cuts = sorted(set([e.a, e.b] + [t for t in splits(e, others) if min(e.a, e.b) < t < max(e.a, e.b)]))
        merged = [cuts[0]]
        for t in cuts[1:]:
            if math.dist(e.pt(t), e.pt(merged[-1])) > 1e-6:
                merged.append(t)
        merged[-1] = cuts[-1]
        for t0, t1 in zip(merged, merged[1:]):
            mid = e.pt((t0 + t1) / 2)
            onb = any(dist_seg(mid, o) <= TOL for o in others)
            res.append((e, t0, t1, onb))
    return res

def length(e, t0, t1):
    n = 2000
    return sum(math.dist(e.pt(t0 + (t1 - t0) * i / n), e.pt(t0 + (t1 - t0) * (i + 1) / n)) for i in range(n))

def run(op, A, B):
    (ea, wa), (eb, wb) = A, B
    pa = pieces(ea, eb)
    if op == 'common': kept = [p for p in pa if p[3]]; twice = []
    elif op == 'cut': kept = [p for p in pa if not p[3]]; twice = []
    else:
        pb = pieces(eb, ea)
        kept = pa + [p for p in pb if not p[3]]
        twice = [p for p in pb if p[3]] if (wa and wb) else []
    L = sum(length(*p[:3]) for p in kept + twice)
    pts = []
    for e, t0, t1, _ in kept:
        for q in (e.pt(t0), e.pt(t1)):
            if not any(math.dist(q, r) <= 1e-6 for r in pts): pts.append(q)
    return L, len(pts), len(kept)

if __name__ == '__main__':
    # usage: occt_case9_model.py <dataset brep dir> < cases  (each line: name op a b, with op
    # common|cut|fuse and a, b the case_9_<a>.brep files)
    d = sys.argv[1]
    cache = {}
    def get(n):
        if n not in cache: cache[n] = read(f'{d}/case_9_{n}.brep')
        return cache[n]
    for line in sys.stdin:
        name, op, x, y = line.split()
        L, v, e = run(op, get(x), get(y))
        print(f'{name} {op} {x} {y} len {L:.6f} v {v} e {e}')
