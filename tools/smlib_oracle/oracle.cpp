// A test oracle for the native kernel on NVIDIA's SMLib (usd-brep, Apache-2.0).
//
//   smlib_oracle props      in
//   smlib_oracle fillet     in out.smb radius x y z [x y z ...]
//   smlib_oracle boolean    a b U|I|D [out.smb]
//   smlib_oracle offset     in dist [keepFace1] [out.smb]          (--tol=1e-4)
//   smlib_oracle shell      in dist face0 [face0 ...] [out.smb]    (--no-extend)
//   smlib_oracle replace    in face0 plane ox oy oz nx ny nz [out.smb]
//                           in face0 cyl ox oy oz ax ay az r [out.smb]
//                           in face0 sms file.sms [out.smb]       (--dom=u0,v0,u1,v1, --xdir=x,y,z)
//   smlib_oracle chamfer    in out.smb dist x y z [x y z ...] | all (--solver=radius|dist, --global-merge)
//   smlib_oracle sweep      in trans dx dy dz [out.smb]
//                           in rot ox oy oz ax ay az deg [out.smb]
//   smlib_oracle stitch     in tol [tol ...]
//   smlib_oracle ssi        file.smp [tx ty tz]                   (--tol=1e-3)
//   smlib_oracle section    in nx ny nz k                          (--tol=1e-2, --move=x,y,z)
//   smlib_oracle silhouette in vx vy vz                            (--persp, --tol=1e-3, --move=x,y,z)
//   smlib_oracle tess       in chord angleDeg [maxEdge [maxAspect]]
//   smlib_oracle cci        file.smp                               (--tol=SM_ZONE_TOL_3D)
//
// An input is a .smb file, `file.smp#i` (the i-th brep of a part file, default 0), or for
// `sweep` a built-in primitive: `sphere:r:cx:cy:cz` (SmPrimitiveCreation::CreateSphere, as
// smSweepTest.cpp case 25/33) or `spherenp:r:cx:cy:cz` (CreateSphereNoPole, cases 27-29), and
// for any subcommand `box:l:w:h:x:y:z`, `cone:h:r0:r1:x:y:z`, `torus:R:r:x:y:z` (my_test_merge).
// `props` and `fillet` read through SM_API with the UV trim curves rebuilt (as before); the other
// subcommands read like SMLib's own tests (SmBrep::ReadFromFile, no rebuild; --rebuild-uv to
// rebuild). The native kernel writes its bodies as .smb with `native::oracle::smb_text`.
// Edges are picked as the ones nearest the given points. Each run prints one line of JSON;
// diagnostics go to stderr with ORACLE_VERBOSE=1.
#include <SmApiBrep.h>
#include <SmApiGeneral.h>
#include <SmApiImportExport.h>
#include <SmApiQueries.h>
#include <SmSmlibAll.h>
#include <SmBrep.h>
#include <SmBrepData.h>
#include <SmBSplineSurface.h>
#include <SmCylinder.h>
#include <SmFilletExecutive.h>
#include <SmFilletSolver.h>
#include <SmFilletStandardSolver.h>
#include <SmGraphicsExtern.h>
#include <SmMerge.h>
#include <SmMessages.h>
#include <SmOffsetExecutive.h>
#include <SmOffsetGeometryCreation.h>
#include <SmPlane.h>
#include <SmPrimitiveCreation.h>
#include <SmRotationalSweepGeometry.h>
#include <SmSolutionArray.h>
#include <SmTess.h>
#include <SmTol.h>
#include <SmTopologySweep.h>
#include <SmTransSweepGeometry.h>
#include <cctype>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <set>
#include <string>
#include <vector>

static bool verbose() { return std::getenv("ORACLE_VERBOSE") != nullptr; }

// Options (`--key` or `--key=value`), taken out of argv before the positional arguments.
static std::map<std::string, std::string> g_opts;
static bool has_opt(const char* k) { return g_opts.count(k) != 0; }
static double opt_num(const char* k, double dflt) {
    auto it = g_opts.find(k);
    return it == g_opts.end() ? dflt : std::atof(it->second.c_str());
}

static SmContext& ctx() { return *SmApiGetOrCreateContext(); }

// A JSON number (null when not finite).
static std::string num(double x) {
    if (!std::isfinite(x)) return "null";
    char b[64];
    std::snprintf(b, sizeof b, "%.12g", x);
    return b;
}
static std::string pt(const SmPoint3d& p) { return "[" + num(p.x) + ", " + num(p.y) + ", " + num(p.z) + "]"; }

[[noreturn]] static void fail(const std::string& status) {
    std::printf("{\"ok\": false, \"status\": \"%s\"}\n", status.c_str());
    std::exit(1);
}

static void report(SmBrep* b, const char* status, const std::string& extra = "") {
    double vol = 0.0, area = 0.0;
    SmApiStatus vs = SmApiBrepComputeVolume(b, 1e-6, vol);
    SmApiBrepComputeArea(b, 1e-6, area);
    if (verbose() && vs != SM_SUCCESS) std::fprintf(stderr, "volume status %d\n", (int)vs);
    SmTArray<SmFace*> faces;
    SmTArray<SmEdge*> edges;
    SmApiGetFaces(b, faces);
    SmApiGetEdges(b, edges);
    if (verbose()) {
        for (ULONG i = 0; i < faces.GetSize(); ++i) {
            double fa = 0.0;
            SmApiFaceComputeArea(faces[i], 1e-6, fa);
            std::fprintf(stderr, "face %lu area %.9g\n", (unsigned long)i, fa);
        }
    }
    SmBoolean solid = FALSE;
    SmApiBrepIsManifoldSolid(b, solid);
    std::printf("{\"ok\": %s, \"status\": \"%s\", \"volume\": %.12g, \"area\": %.12g, \"faces\": %lu, \"edges\": %lu, \"solid\": %s%s}\n",
                std::string(status) == "ok" ? "true" : "false", status, vol, area, (unsigned long)faces.GetSize(),
                (unsigned long)edges.GetSize(), solid ? "true" : "false", extra.c_str());
}

// SMLib's errors on stderr when ORACLE_VERBOSE is set (where a read or an operation failed).
static void on_error(SmStatus e, const TCHAR* const file, ULONG line, const TCHAR* const msg) {
    if (verbose()) std::fprintf(stderr, "smlib error %d at %s:%lu %s\n", (int)e, file ? file : "?", (unsigned long)line, msg ? msg : "");
}

static bool ends_with(const std::string& s, const char* suf) {
    size_t n = std::strlen(suf);
    if (s.size() < n) return false;
    for (size_t i = 0; i < n; ++i)
        if (std::tolower((unsigned char)s[s.size() - n + i]) != suf[i]) return false;
    return true;
}
static bool is_int(const char* s) {
    if (!*s) return false;
    for (const char* p = s; *p; ++p)
        if (!std::isdigit((unsigned char)*p)) return false;
    return true;
}

struct Part {
    SmTArray<SmCurve*> curves;
    SmTArray<SmSurface*> surfaces;
    SmTArray<long> trees;
    SmTArray<SmBrep*> breps;
};

// A part file (.smp), as the tests read it (SmBrepData::ReadPartFromFile, ASCII).
static Part* read_part(const std::string& path) {
    Part* p = new Part;
    SmStatus st = SmBrepData::ReadPartFromFile(ctx(), path.c_str(), p->curves, p->surfaces, p->trees, p->breps, SM_ASCII);
    if (st != SM_SUCCESS) fail("can't read part " + path + " (" + std::to_string((int)st) + ")");
    if (verbose())
        std::fprintf(stderr, "part %s: %lu curves, %lu surfaces, %lu breps\n", path.c_str(), (unsigned long)p->curves.GetSize(),
                     (unsigned long)p->surfaces.GetSize(), (unsigned long)p->breps.GetSize());
    return p;
}

static std::vector<double> nums_after(const std::string& spec, size_t from) {
    std::vector<double> v;
    std::string s = spec.substr(from);
    size_t i = 0;
    while (i <= s.size()) {
        size_t j = s.find(':', i);
        if (j == std::string::npos) j = s.size();
        v.push_back(std::atof(s.substr(i, j - i).c_str()));
        i = j + 1;
    }
    return v;
}

// An input brep: file.smb, file.smp#i, or a built-in sphere.
static SmBrep* load(const std::string& spec, bool rebuild_uv) {
    if (spec.rfind("sphere:", 0) == 0 || spec.rfind("spherenp:", 0) == 0) {
        bool np = spec.rfind("spherenp:", 0) == 0;
        std::vector<double> a = nums_after(spec, np ? 9 : 7);
        if (a.size() != 4) fail("bad primitive " + spec);
        SmBrep* b = new (ctx()) SmBrep();
        if (np) {
            // smSweepTest.cpp cases 27-29: tolerance = the new brep's tolerance
            double tol = b->GetTolerance();
            if (SmPrimitiveCreation::CreateSphereNoPole(&ctx(), a[0], SmVector3d(a[1], a[2], a[3]), tol, b) != SM_SUCCESS || !b)
                fail("CreateSphereNoPole failed");
        } else {
            // smSweepTest.cpp cases 25, 33-34
            SmAxis2Placement place(SmPoint3d(a[1], a[2], a[3]), SmVector3d(1, 0, 0), SmVector3d(0, 1, 0));
            SmPrimitiveCreation pc(b->GetInfiniteRegion());
            if (pc.CreateSphere(a[0], 0, 360, place) != SM_SUCCESS) fail("CreateSphere failed");
        }
        return b;
    }
    // my_test_merge's primitive pairs (smMergeTest.cpp): box:l:w:h:x:y:z (CreateBox, min corner
    // at x y z), cone:h:r0:r1:x:y:z (CreateCone, base centre at x y z, axis +z) and
    // torus:R:r:x:y:z (CreateTorus about +z), full turns, tolerance SM_ZONE_TOL_3D as the test sets.
    for (const char* kind : {"box:", "cone:", "torus:"}) {
        size_t n = std::strlen(kind);
        if (spec.compare(0, n, kind) != 0) continue;
        std::vector<double> a = nums_after(spec, n);
        size_t want = kind[0] == 't' ? 5 : 6;
        if (a.size() != want) fail("bad primitive " + spec);
        SmBrep* b = new (ctx()) SmBrep();
        b->SetTolerance(SM_ZONE_TOL_3D);
        SmAxis2Placement place(SmPoint3d(a[want - 3], a[want - 2], a[want - 1]), SmVector3d(1, 0, 0), SmVector3d(0, 1, 0));
        SmPrimitiveCreation pc(b->GetInfiniteRegion());
        SmStatus st = kind[0] == 'b'   ? pc.CreateBox(a[0], a[1], a[2], place)
                      : kind[0] == 'c' ? pc.CreateCone(a[0], a[1], a[2], 0, 360, place)
                                       : pc.CreateTorus(a[0], a[1], 0, 360, place);
        if (st != SM_SUCCESS) fail("can't build " + spec);
        return b;
    }
    std::string path = spec;
    long idx = 0;
    size_t hash = spec.rfind('#');
    if (hash != std::string::npos && ends_with(spec.substr(0, hash), ".smp")) {
        path = spec.substr(0, hash);
        idx = std::atol(spec.c_str() + hash + 1);
    }
    if (ends_with(path, ".smp")) {
        Part* p = read_part(path);
        if (idx < 0 || (ULONG)idx >= p->breps.GetSize())
            fail("no brep " + std::to_string(idx) + " in " + path + " (" + std::to_string(p->breps.GetSize()) + " breps)");
        return p->breps[idx];
    }
    if (rebuild_uv) {
        SmBrep* b = nullptr;
        SmApiStatus st = SmApiReadBrepFromFile(path.c_str(), TRUE, TRUE, b);
        if (st != SM_SUCCESS || !b) fail("can't read " + path + " (" + std::to_string((int)st) + ")");
        return b;
    }
    SmBrep* b = new (ctx()) SmBrep();
    SmStatus st = b->ReadFromFile(ctx(), path.c_str(), SM_ASCII, FALSE);
    if (st != SM_SUCCESS) fail("can't read " + path + " (" + std::to_string((int)st) + ")");
    return b;
}

// --move=x,y,z: translate the input first (SmBrep::Transform), as the section, silhouette and
// SSI tests do before they run.
static SmBrep* moved(SmBrep* b) {
    if (!has_opt("move")) return b;
    double m[3] = {0, 0, 0};
    std::sscanf(g_opts["move"].c_str(), "%lf,%lf,%lf", &m[0], &m[1], &m[2]);
    SmAxis2Placement r;
    r.Translate(SmVector3d(m[0], m[1], m[2]));
    if (b->Transform(r) != SM_SUCCESS) fail("Transform failed");
    return b;
}

static void write_out(SmBrep* b, const std::string& out) {
    if (out.empty()) return;
    if (b->WriteToFile(out.c_str()) != SM_SUCCESS && verbose()) std::fprintf(stderr, "can't write %s\n", out.c_str());
}

// The edges nearest the points in argv[i..] (three numbers per point), each edge once.
static SmTArray<SmEdge*> pick_edges(SmBrep* b, int i, int argc, char** argv) {
    SmTArray<SmEdge*> all, picked;
    b->GetEdges(all);
    std::set<SmEdge*> seen;
    for (; i + 2 < argc; i += 3) {
        SmPoint3d p(std::atof(argv[i]), std::atof(argv[i + 1]), std::atof(argv[i + 2]));
        SmEdge* best = nullptr;
        double bd = 1e300;
        for (ULONG k = 0; k < all.GetSize(); ++k) {
            SmPoint3d q;
            double t, d;
            if (SmApiEdgeClosestPoint(all[k], p, q, t, d) == SM_SUCCESS && d < bd) {
                bd = d;
                best = all[k];
            }
        }
        if (verbose()) std::fprintf(stderr, "point %g %g %g -> edge at distance %g\n", p.x, p.y, p.z, bd);
        if (best && seen.insert(best).second) picked.Add(best);
    }
    return picked;
}

// Length, ends and 8 samples (evenly in the parameter, ends included) of a curve.
static double curve_length(const SmCurve* c) {
    double len = 0.0;
    if (c->Length(c->GetNaturalInterval(), 1e-6, len) != SM_SUCCESS) return NAN;
    return len;
}
static std::string curve_json(const SmCurve* c, const std::string& extra = "") {
    SmExtent1d iv = c->GetNaturalInterval();
    std::string s = "{\"length\": " + num(curve_length(c));
    SmPoint3d a, z;
    c->EvaluatePoint(iv.GetMin(), a);
    c->EvaluatePoint(iv.GetMax(), z);
    s += ", \"start\": " + pt(a) + ", \"end\": " + pt(z) + ", \"samples\": [";
    for (int k = 0; k < 8; ++k) {
        SmPoint3d p;
        c->EvaluatePoint(iv.GetMin() + (iv.GetMax() - iv.GetMin()) * k / 7.0, p);
        s += (k ? ", " : "") + pt(p);
    }
    return s + "]" + extra + "}";
}

static SmVector3d unit(SmVector3d v) {
    double l = std::sqrt(v.x * v.x + v.y * v.y + v.z * v.z);
    return l > 0 ? SmVector3d(v.x / l, v.y / l, v.z / l) : v;
}
static SmVector3d cross(const SmVector3d& a, const SmVector3d& b) {
    return SmVector3d(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
}
static double dot(const SmVector3d& a, const SmVector3d& b) { return a.x * b.x + a.y * b.y + a.z * b.z; }
// A unit X axis perpendicular to z: the world axis least along z, made perpendicular.
static SmVector3d perp(const SmVector3d& z) {
    SmVector3d w = std::fabs(z.x) <= std::fabs(z.y) && std::fabs(z.x) <= std::fabs(z.z) ? SmVector3d(1, 0, 0)
                   : std::fabs(z.y) <= std::fabs(z.z)                                  ? SmVector3d(0, 1, 0)
                                                                                       : SmVector3d(0, 0, 1);
    double d = dot(w, z);
    return unit(SmVector3d(w.x - d * z.x, w.y - d * z.y, w.z - d * z.z));
}

// my_test_bbu (smMergeTest.cpp): large parts moved to their joint box centre, then
// SmMerge::ManifoldBoolean.
static SmStatus bbu(SmBrep* b1, SmBrep* b2, SmBooleanOperationType op, SmBrep*& res) {
    SmExtent3d bx, bx2;
    b1->CalculateBoundingBox(bx);
    b2->CalculateBoundingBox(bx2);
    bx.Union(bx2, bx);
    bool trans = false;
    SmPoint3d c = bx.Evaluate(0.5, 0.5, 0.5);
    if (c.LengthSquared() > 1.0e7) {
        trans = true;
        c = -c;
        SmAxis2Placement r;
        r.Translate(c);
        b1->Transform(r);
        b2->Transform(r);
    }
    SmMerge m(*(b1->GetContext()), b1, b2);
    res = nullptr;
    SmStatus st = m.ManifoldBoolean(op, res);
    if (st == SM_SUCCESS && trans && res) {
        c = -c;
        SmAxis2Placement r;
        r.Translate(c);
        res->Transform(r);
    }
    return st;
}

// Triangles from SmTess::OutputPolygons: count, area, the signed volume they enclose, and the
// faces that got any.
class TriSink : public SmPolygonOutputCallback {
public:
    ULONG tris = 0;
    double area = 0.0, vol = 0.0;
    std::set<SmFace*> faces;
    TriSink() : SmPolygonOutputCallback(SM_PO_TRIANGLES) {}
    SmStatus OutputPolygon(ULONG, ULONG n, SmPoint3d* p, SmVector3d*, SmPoint2d*, SmSurface*, SmFace* f, SmGfxArraySet* = NULL) override {
        if (f) faces.insert(f);
        for (ULONG k = 1; k + 1 < n; ++k) {
            SmVector3d a(p[0].x, p[0].y, p[0].z), b(p[k].x, p[k].y, p[k].z), c(p[k + 1].x, p[k + 1].y, p[k + 1].z);
            SmVector3d n3 = cross(SmVector3d(b.x - a.x, b.y - a.y, b.z - a.z), SmVector3d(c.x - a.x, c.y - a.y, c.z - a.z));
            area += 0.5 * std::sqrt(dot(n3, n3));
            vol += dot(a, cross(b, c)) / 6.0;
            ++tris;
        }
        return SM_SUCCESS;
    }
};

[[noreturn]] static void usage() {
    std::fprintf(stderr,
                 "usage: smlib_oracle props|fillet|boolean|offset|shell|replace|chamfer|sweep|stitch|ssi|section|silhouette|tess|cci ...\n"
                 "  (see the comment at the top of oracle.cpp or README.md)\n");
    std::exit(2);
}

int main(int argc0, char** argv0) {
    std::vector<char*> args;
    for (int i = 0; i < argc0; ++i) {
        std::string a = argv0[i];
        if (i > 0 && a.size() > 2 && a[0] == '-' && a[1] == '-' && !std::isdigit((unsigned char)a[2]) && a[2] != '.') {
            size_t eq = a.find('=');
            g_opts[a.substr(2, eq == std::string::npos ? std::string::npos : eq - 2)] = eq == std::string::npos ? "" : a.substr(eq + 1);
        } else {
            args.push_back(argv0[i]);
        }
    }
    int argc = (int)args.size();
    char** argv = args.data();
    if (argc < 3) usage();
    SmApiCreateContext();
    smos_SetErrorCallback(on_error);
    std::string cmd = argv[1];
    bool rebuild = has_opt("rebuild-uv");

    if (cmd == "props") {
        report(load(argv[2], true), "ok");
        return 0;
    }

    if (cmd == "fillet" && argc >= 8 && (argc - 5) % 3 == 0) {
        SmBrep* b = load(argv[2], true);
        double r = std::atof(argv[4]);
        SmTArray<SmEdge*> picked = pick_edges(b, 5, argc, argv);
        SmApiStatus s = SmApiFilletEdges(b, picked, r, 1, 1, 1.0);
        if (s != SM_SUCCESS) fail("fillet failed (" + std::to_string((int)s) + ")");
        SmApiWriteBrepToFile(b, argv[3], TRUE);
        report(b, "ok");
        return 0;
    }

    // boolean a b U|I|D [out]: my_test_bbu.
    if (cmd == "boolean" && (argc == 5 || argc == 6)) {
        SmBrep* a = load(argv[2], rebuild);
        SmBrep* b = load(argv[3], rebuild);
        char op = std::toupper((unsigned char)argv[4][0]);
        if (op != 'U' && op != 'I' && op != 'D') usage();
        SmBooleanOperationType e = op == 'U' ? SM_BO_UNION : op == 'I' ? SM_BO_INTERSECTION : SM_BO_DIFFERENCE;
        SmBrep* res = nullptr;
        SmStatus st = bbu(a, b, e, res);
        if (st != SM_SUCCESS || !res) fail("boolean failed (" + std::to_string((int)st) + ")");
        if (argc == 6) write_out(res, argv[5]);
        report(res, "ok");
        return 0;
    }

    // offset in dist [keepFace1] [out]: my_test_offset (keepFace is 1-based, 0 = none).
    if (cmd == "offset" && argc >= 4 && argc <= 6) {
        SmBrep* b = load(argv[2], rebuild);
        double r = std::atof(argv[3]);
        long keep = 0;
        std::string out;
        for (int i = 4; i < argc; ++i) {
            if (is_int(argv[i])) keep = std::atol(argv[i]);
            else out = argv[i];
        }
        double tol = opt_num("tol", 1.0e-4);
        SmTArray<SmFace*> shell;
        if (keep > 0) {
            SmTArray<SmFace*> faces;
            b->GetFaces(faces);
            if ((ULONG)keep > faces.GetSize()) fail("no face " + std::to_string(keep));
            shell.Add(faces[keep - 1]);
        }
        bool inset = r < 0.0;
        if (inset) r = std::fabs(r);
        SmOffsetGeometryCreation ogc(r, tol);
        SmOffsetExecutive ex(ctx(), SM_OO_SOLID_OFFSET, ogc, b);
        if (inset) ex.SetInset(TRUE);
        if (keep > 0) ex.SetShellFaces(shell);
        ex.SetExtendConvexEdges(FALSE);
        ex.SetMergeResults(TRUE);
        SmBrep* res = nullptr;
        SmStatus st = ex.DoSolidOffset(res);
        if (st != SM_SUCCESS || !res) fail("offset failed (" + std::to_string((int)st) + ")");
        write_out(res, out);
        report(res, "ok");
        return 0;
    }

    // shell in dist face0... [out]: ShellBrep (smOffsetTest.cpp), extend corners as every test does.
    if (cmd == "shell" && argc >= 5) {
        SmBrep* b = load(argv[2], rebuild);
        double dist = std::atof(argv[3]);
        std::string out;
        int last = argc;
        if (!is_int(argv[argc - 1])) {
            out = argv[argc - 1];
            last = argc - 1;
        }
        SmTArray<SmFace*> faces, shell;
        b->GetFaces(faces);
        for (int i = 4; i < last; ++i) {
            ULONG k = std::atol(argv[i]);
            if (k >= faces.GetSize()) fail("no face " + std::to_string(k));
            shell.Add(faces[k]);
        }
        bool extend = !has_opt("no-extend");
        double r = std::fabs(dist);
        double tol = b->GetTolerance();
        SmBrep* off = nullptr;
        SmBrep* offExec = nullptr;
        {
            SmOffsetGeometryCreation ogc(r, tol);
            SmOffsetExecutive ex(ctx(), SM_OO_SOLID_OFFSET, ogc, b);
            if (dist < 0.0) ex.SetInset(TRUE);
            ex.SetShellFaces(shell);
            ex.SetExtendConvexEdges(extend ? TRUE : FALSE);
            ex.SetMergeResults(extend ? FALSE : TRUE);
            SmStatus st = ex.DoSolidOffset(off);
            if (st != SM_SUCCESS || !off) fail("shell offset failed (" + std::to_string((int)st) + ")");
            offExec = ex.GetOffsetBrep();
        }
        if (offExec) delete offExec;
        SmBrep* res = nullptr;
        SmStatus st = dist < 0.0 ? bbu(b, off, SM_BO_DIFFERENCE, res) : bbu(off, b, SM_BO_DIFFERENCE, res);
        if (st != SM_SUCCESS || !res) fail("shell difference failed (" + std::to_string((int)st) + ")");
        if (verbose()) std::fprintf(stderr, "difference result is %s\n", res == b ? "the input" : res == off ? "the offset" : "new");
        write_out(res, out);
        report(res, "ok");
        return 0;
    }

    // replace in face0 plane|cyl|sms ... [out]: SmBrep::LocalOperation (smFilletTest.cpp my_local_op).
    if (cmd == "replace" && argc >= 6) {
        SmBrep* b = load(argv[2], rebuild);
        SmTArray<SmFace*> faces;
        b->GetFaces(faces);
        ULONG fi = std::atol(argv[3]);
        if (fi >= faces.GetSize()) fail("no face " + std::to_string(fi));
        std::string kind = argv[4];
        SmExtent3d box;
        b->CalculateBoundingBox(box);
        SmPoint3d bmin = box.GetMin(), bmax = box.GetMax();
        double pad = 0.1 * std::sqrt((bmax.x - bmin.x) * (bmax.x - bmin.x) + (bmax.y - bmin.y) * (bmax.y - bmin.y) + (bmax.z - bmin.z) * (bmax.z - bmin.z));
        // The box corners' coordinates along a direction from o, padded: a default domain.
        auto span = [&](const SmPoint3d& o, const SmVector3d& d, double& lo, double& hi) {
            lo = 1e300;
            hi = -1e300;
            for (int c = 0; c < 8; ++c) {
                SmVector3d q((c & 1 ? bmax.x : bmin.x) - o.x, (c & 2 ? bmax.y : bmin.y) - o.y, (c & 4 ? bmax.z : bmin.z) - o.z);
                double t = dot(q, d);
                lo = std::min(lo, t);
                hi = std::max(hi, t);
            }
            lo -= pad;
            hi += pad;
        };
        // The X axis (u = 0 direction): --xdir made perpendicular, else the world axis least along the normal/axis.
        auto xaxis = [&](const SmVector3d& z) {
            if (!has_opt("xdir")) return perp(z);
            double w[3] = {0, 0, 0};
            std::sscanf(g_opts["xdir"].c_str(), "%lf,%lf,%lf", &w[0], &w[1], &w[2]);
            double d = w[0] * z.x + w[1] * z.y + w[2] * z.z;
            return unit(SmVector3d(w[0] - d * z.x, w[1] - d * z.y, w[2] - d * z.z));
        };
        bool dom_given = has_opt("dom");
        double dom[4] = {0, 0, 0, 0};
        if (dom_given) std::sscanf(g_opts["dom"].c_str(), "%lf,%lf,%lf,%lf", &dom[0], &dom[1], &dom[2], &dom[3]);
        SmSurface* s = nullptr;
        int next = 0;
        if (kind == "plane" && argc >= 11) {
            SmPoint3d o(std::atof(argv[5]), std::atof(argv[6]), std::atof(argv[7]));
            SmVector3d n = unit(SmVector3d(std::atof(argv[8]), std::atof(argv[9]), std::atof(argv[10])));
            SmVector3d x = xaxis(n), y = cross(n, x);
            SmAxis2Placement pos;
            pos.SetCanonical(o, x, y);
            SmPlane* p = nullptr;
            if (SmPlane::CreateCanonical(ctx(), pos, p) != SM_SUCCESS || !p) fail("CreateCanonical(plane) failed");
            if (!dom_given) {
                span(o, x, dom[0], dom[2]);
                span(o, y, dom[1], dom[3]);
            }
            if (p->AdjustSTEPUVDomain(SmExtent2d(SmPoint2d(dom[0], dom[1]), SmPoint2d(dom[2], dom[3]))) != SM_SUCCESS) fail("AdjustSTEPUVDomain failed");
            s = p;
            next = 11;
        } else if (kind == "cyl" && argc >= 12) {
            SmPoint3d o(std::atof(argv[5]), std::atof(argv[6]), std::atof(argv[7]));
            SmVector3d a = unit(SmVector3d(std::atof(argv[8]), std::atof(argv[9]), std::atof(argv[10])));
            double r = std::atof(argv[11]);
            SmVector3d x = xaxis(a), y = cross(a, x);
            SmAxis2Placement pos;
            pos.SetCanonical(o, x, y);
            SmCylinder* c = nullptr;
            if (SmCylinder::CreateCanonical(ctx(), pos, r, c) != SM_SUCCESS || !c) fail("CreateCanonical(cylinder) failed");
            if (!dom_given) {
                dom[0] = 0.0;
                dom[2] = 360.0;
                span(o, a, dom[1], dom[3]);
            }
            if (c->AdjustSTEPUVDomain(SmExtent2d(SmPoint2d(dom[0], dom[1]), SmPoint2d(dom[2], dom[3]))) != SM_SUCCESS) fail("AdjustSTEPUVDomain failed");
            s = c;
            next = 12;
        } else if (kind == "sms" && argc >= 6) {
            SmBSplineSurface* bs = nullptr;
            if (SmBSplineSurface::ReadFromFile(ctx(), argv[5], bs) != SM_SUCCESS || !bs) fail(std::string("can't read ") + argv[5]);
            bs->Reparameterize(faces[fi]->GetSurface()->GetNaturalUVDomain());
            s = bs;
            next = 6;
        } else {
            usage();
        }
        SmTArray<SmFace*> nf;
        SmTArray<SmSurface*> ns;
        nf.Add(faces[fi]);
        ns.Add(s);
        SmStatus st = b->LocalOperation(nf, ns);
        if (st != SM_SUCCESS) fail("LocalOperation failed (" + std::to_string((int)st) + ")");
        if (next < argc) write_out(b, argv[next]);
        report(b, "ok");
        return 0;
    }

    // chamfer in out dist x y z ... | all: my_test_model (smFilletTest.cpp) with bMakeChamfer.
    if (cmd == "chamfer" && argc >= 6) {
        SmBrep* b = load(argv[2], rebuild);
        double d = std::atof(argv[4]);
        SmTArray<SmEdge*> edges;
        if (std::string(argv[5]) == "all") b->GetEdges(edges);
        else if ((argc - 5) % 3 == 0) edges = pick_edges(b, 5, argc, argv);
        else usage();
        bool dist = g_opts.count("solver") && g_opts["solver"] == "dist";
        SmApproxTol3d tol = SmTol::GetApproxTol3d(b);
        SmLinearCrossSectionFSG lin;
        SmMakeSurfaceBlendSIH sih(1.0);
        SmFilletExecutive* fx = new (ctx()) SmFilletExecutive(ctx(), b);
        fx->SetSelfIntersectionHandler(&sih);
        if (has_opt("global-merge")) fx->SetDoGlobalMerge(TRUE);
        ULONG loaded = 0;
        for (ULONG i = 0; i < edges.GetSize(); ++i) {
            SmEdgeuse* eu = edges[i]->GetBlendEdgeuse();
            if (!eu) continue;
            SmFilletSolver* fs = dist ? (SmFilletSolver*)new (ctx()) SmConstantDistanceFS(ctx(), tol, 30.0 * SM_PI / 180.0, 2.0 * SM_PI / 180.0, d * std::sqrt(2.0), eu)
                                      : (SmFilletSolver*)new (ctx()) SmConstantRadiusFS(ctx(), tol, 30.0 * SM_PI / 180.0, 2.0 * SM_PI / 180.0, d, eu);
            fs->SetFilletSurfaceGenerator(&lin);
            fx->LoadFilletSolver(fs);
            ++loaded;
        }
        if (verbose()) std::fprintf(stderr, "%lu edges, %lu filletable\n", (unsigned long)edges.GetSize(), (unsigned long)loaded);
        SmStatus st = fx->CreateFilletCorners();
        if (st == SM_SUCCESS) st = fx->DoFilleting();
        if (st != SM_SUCCESS) {
            if (verbose()) fx->GetFilletErrorInfo()->Report();
            fail("chamfer failed (" + std::to_string((int)st) + ")");
        }
        write_out(b, argv[3]);
        report(b, "ok", ", \"chamfered\": " + std::to_string(loaded));
        return 0;
    }

    // sweep in trans dx dy dz | rot ox oy oz ax ay az deg [out]: whole-body SmTopologySweep.
    if (cmd == "sweep" && argc >= 7) {
        SmBrep* b = load(argv[2], rebuild);
        std::string kind = argv[3];
        SmSweepGeometryCreation* g = nullptr;
        int next = 0;
        if (kind == "trans" && argc >= 7) {
            g = new SmTranslationalSweepGeometry(SmVector3d(std::atof(argv[4]), std::atof(argv[5]), std::atof(argv[6])));
            next = 7;
        } else if (kind == "rot" && argc >= 11) {
            SmPoint3d o(std::atof(argv[4]), std::atof(argv[5]), std::atof(argv[6]));
            SmVector3d a(std::atof(argv[7]), std::atof(argv[8]), std::atof(argv[9]));
            g = new SmRotationalSweepGeometry(o, a, std::atof(argv[10]), opt_num("eps", b->GetTolerance()));
            next = 11;
        } else {
            usage();
        }
        SmTopologySweep sw(*g);
        sw.SetDoMerge(TRUE);
        sw.SetDoStitching(TRUE);
        // the tests ignore DoSweep's status; it is reported here
        SmStatus st = sw.DoSweep(b, b->GetInfiniteRegion(), NULL, NULL, NULL, NULL, NULL, NULL);
        if (next < argc) write_out(b, argv[next]);
        std::string status = st == SM_SUCCESS ? "ok" : "sweep status " + std::to_string((int)st);
        report(b, status.c_str(), ", \"sweep_status\": " + std::to_string((int)st));
        return 0;
    }

    // stitch in tol [tol ...]: SmBrep::StitchFaces (my_stitching_demo), one run per tolerance.
    if (cmd == "stitch" && argc >= 4) {
        SmBrep* b = load(argv[2], rebuild);
        std::string runs = ", \"runs\": [";
        for (int i = 3; i < argc; ++i) {
            ULONG ns = 0, nl = 0;
            double vg = 0, eg = 0;
            b->m_bEditingEnabled = TRUE;
            SmStatus st = b->StitchFaces(std::atof(argv[i]), ns, nl, vg, eg);
            b->m_bEditingEnabled = FALSE;
            runs += std::string(i > 3 ? ", " : "") + "{\"tol\": " + num(std::atof(argv[i])) + ", \"status\": " + std::to_string((int)st) +
                    ", \"stitched\": " + std::to_string(ns) + ", \"lamina\": " + std::to_string(nl) + ", \"max_vertex_gap\": " + num(vg) +
                    ", \"max_edge_gap\": " + num(eg) + "}";
        }
        report(b, "ok", runs + "]");
        return 0;
    }

    // ssi file.smp [tx ty tz]: GlobalSurfaceIntersect over every face pair of breps 0 and 1
    // (my_test_one_ssi / my_test_ssi), or every surface pair of a surface-only part.
    if (cmd == "ssi" && (argc == 3 || argc == 6)) {
        Part* p = read_part(argv[2]);
        SmApproxTol3d tol = opt_num("tol", 1.0e-3);
        struct Item {
            SmSurface* s;
            SmExtent2d dom;
            ULONG idx;
        };
        std::vector<Item> A, B;
        SmAxis2Placement move;
        if (argc == 6) move.Translate(SmVector3d(std::atof(argv[3]), std::atof(argv[4]), std::atof(argv[5])));
        if (p->breps.GetSize() >= 2) {
            if (argc == 6) {
                p->breps[0]->Transform(move);
                p->breps[1]->Transform(move);
            }
            SmTArray<SmFace*> f1, f2;
            p->breps[0]->GetFaces(f1);
            p->breps[1]->GetFaces(f2);
            for (ULONG i = 0; i < f1.GetSize(); ++i) A.push_back({f1[i]->GetSurface(), f1[i]->GetUVDomain(), i});
            for (ULONG i = 0; i < f2.GetSize(); ++i) B.push_back({f2[i]->GetSurface(), f2[i]->GetUVDomain(), i});
        } else if (p->surfaces.GetSize() >= 2) {
            for (ULONG i = 0; i < p->surfaces.GetSize(); ++i) {
                if (argc == 6) p->surfaces[i]->Transform(move);
                A.push_back({p->surfaces[i], p->surfaces[i]->GetNaturalUVDomain(), i});
            }
        } else {
            fail("ssi needs two breps or two surfaces");
        }
        bool surf_only = B.empty();
        std::string curves;
        ULONG total = 0, pairs = 0, failed = 0;
        for (size_t i = 0; i < A.size(); ++i) {
            for (size_t j = surf_only ? i + 1 : 0; j < (surf_only ? A.size() : B.size()); ++j) {
                const Item& a = A[i];
                const Item& b = surf_only ? A[j] : B[j];
                SmBoolean edges[2] = {TRUE, TRUE};
                SmTArray<SmCurve*> c3, uv1, uv2;
                SmTArray<SmTsectCurveType> types;
                SmTArray<double> dev;
                ++pairs;
                SmStatus st = a.s->GlobalSurfaceIntersect(ctx(), a.dom, *b.s, b.dom, edges, &tol, NULL, &c3, &uv1, &uv2, &types, &dev);
                if (st != SM_SUCCESS) {
                    ++failed;
                    if (verbose()) std::fprintf(stderr, "pair %lu %lu: status %d\n", (unsigned long)a.idx, (unsigned long)b.idx, (int)st);
                    continue;
                }
                for (ULONG k = 0; k < c3.GetSize(); ++k) {
                    if (!c3[k]) continue;
                    std::string extra = ", \"pair\": [" + std::to_string(a.idx) + ", " + std::to_string(b.idx) + "], \"type\": " +
                                        std::to_string(k < types.GetSize() ? (int)types[k] : -1) +
                                        ", \"deviation\": " + num(k < dev.GetSize() ? dev[k] : NAN);
                    curves += (total ? ", " : "") + curve_json(c3[k], extra);
                    ++total;
                }
            }
        }
        std::printf("{\"ok\": true, \"status\": \"ok\", \"pairs\": %lu, \"failed_pairs\": %lu, \"count\": %lu, \"curves\": [%s]}\n",
                    (unsigned long)pairs, (unsigned long)failed, (unsigned long)total, curves.c_str());
        return 0;
    }

    // section in nx ny nz k: CreatePlanarSectionCurves at k planes through the bounding box's
    // diagonal (my_test_tsurf_section).
    if (cmd == "section" && argc == 7) {
        SmBrep* b = moved(load(argv[2], rebuild));
        SmVector3d n = unit(SmVector3d(std::atof(argv[3]), std::atof(argv[4]), std::atof(argv[5])));
        long k = std::atol(argv[6]);
        if (k < 1) usage();
        SmApproxTol3d tol = opt_num("tol", 1.0e-2);
        SmExtent3d box;
        b->CalculateBoundingBox(box);
        std::string planes;
        ULONG total = 0;
        for (long i = 0; i < k; ++i) {
            double t = k == 1 ? 0.5 : i / (k - 1.0);
            SmPoint3d p = box.Evaluate(t, t, t);
            SmTArray<SmCurve*> c3, uv;
            SmStatus st = b->CreatePlanarSectionCurves(ctx(), p, n, &tol, NULL, &c3, &uv, NULL);
            if (verbose()) {
                // per face, the untrimmed surface sections (SmBrep's call skips a face whose call fails)
                SmTArray<SmFace*> faces;
                b->GetFaces(faces);
                for (ULONG f = 0; f < faces.GetSize(); ++f) {
                    SmTArray<SmCurve*> s3, suv;
                    SmStatus fs = faces[f]->GetSurface()->CreatePlanarSectionCurves(ctx(), faces[f]->GetUVDomain(), p, n, &tol, NULL, &s3, &suv);
                    SmTArray<SmCurve*> t3, tuv;
                    SmStatus ts = s3.GetSize() ? faces[f]->CreateCurvesByTrimming(ctx(), &tol, &s3, &suv, &t3, &tuv) : SM_SUCCESS;
                    std::fprintf(stderr, "plane %ld face %lu: surface status %d, %lu curves; trimming status %d, %lu curves\n", i, (unsigned long)f,
                                 (int)fs, (unsigned long)s3.GetSize(), (int)ts, (unsigned long)t3.GetSize());
                }
            }
            double len = 0.0;
            for (ULONG j = 0; j < c3.GetSize(); ++j)
                if (c3[j]) len += curve_length(c3[j]);
            total += c3.GetSize();
            planes += std::string(i ? ", " : "") + "{\"point\": " + pt(p) + ", \"status\": " + std::to_string((int)st) +
                      ", \"count\": " + std::to_string(c3.GetSize()) + ", \"length\": " + num(len) + "}";
        }
        std::printf("{\"ok\": true, \"status\": \"ok\", \"count\": %lu, \"planes\": [%s]}\n", (unsigned long)total, planes.c_str());
        return 0;
    }

    // silhouette in vx vy vz [--persp]: per face SmSurface::CreateSilhouetteCurves then
    // SmFace::CreateCurvesByTrimming (my_test_tsurf_silhouette).
    if (cmd == "silhouette" && argc == 6) {
        SmBrep* b = moved(load(argv[2], rebuild));
        SmPoint3d v(std::atof(argv[3]), std::atof(argv[4]), std::atof(argv[5]));
        SmBoolean persp = has_opt("persp") ? TRUE : FALSE;
        SmApproxTol3d tol = opt_num("tol", 1.0e-3);
        double ang = SM_DEG2RAD(20.0);
        SmTArray<SmFace*> faces;
        b->GetFaces(faces);
        std::string per;
        ULONG total = 0;
        double total_len = 0.0;
        for (ULONG i = 0; i < faces.GetSize(); ++i) {
            SmTArray<SmCurve*> c3, uv, t3, tuv;
            SmStatus st = faces[i]->GetSurface()->CreateSilhouetteCurves(ctx(), faces[i]->GetUVDomain(), v, persp, &tol, &ang, &c3, &uv);
            if (st == SM_SUCCESS) st = faces[i]->CreateCurvesByTrimming(ctx(), &tol, &c3, &uv, &t3, &tuv);
            double len = 0.0;
            for (ULONG j = 0; j < t3.GetSize(); ++j)
                if (t3[j]) len += curve_length(t3[j]);
            total += t3.GetSize();
            total_len += len;
            per += std::string(i ? ", " : "") + "{\"face\": " + std::to_string(i) + ", \"status\": " + std::to_string((int)st) +
                   ", \"untrimmed\": " + std::to_string(c3.GetSize()) + ", \"count\": " + std::to_string(t3.GetSize()) + ", \"length\": " + num(len) + "}";
        }
        std::printf("{\"ok\": true, \"status\": \"ok\", \"count\": %lu, \"length\": %s, \"faces\": [%s]}\n", (unsigned long)total,
                    num(total_len).c_str(), per.c_str());
        return 0;
    }

    // tess in chord angleDeg [maxEdge [maxAspect]]: my_test_one_tess (smTessTest.cpp).
    if (cmd == "tess" && argc >= 5 && argc <= 7) {
        SmBrep* b = load(argv[2], rebuild);
        double ch = std::atof(argv[3]), angd = std::atof(argv[4]);
        double maxe = argc > 5 ? std::atof(argv[5]) : 0.0, maxa = argc > 6 ? std::atof(argv[6]) : 0.0;
        if (maxe < 0.0) {
            SmExtent3d box;
            b->CalculateBoundingBox(box);
            double size = box.GetSize().Length();
            if (size >= 1.0e15 || size <= 0.0) size = 1.0;
            maxe = -0.025 * size * maxe;
        }
        SmCurveTessDriver ct(0, ch, angd);
        SmSurfaceTessDriver stess(ch, angd, maxe, 0.0, 0.001, maxa);
        SmTess tess(*b->GetContext(), ct, stess);
        SmBrep* copy = new (*b->GetContext()) SmBrep(*b);
        SmBoolean failed = FALSE;
        SmStatus st = tess.DoTessellation(copy, failed);
        if (st != SM_SUCCESS) fail("tessellation failed (" + std::to_string((int)st) + ")");
        TriSink sink;
        st = tess.OutputPolygons(sink);
        SmTArray<SmFace*> faces;
        copy->GetFaces(faces);
        ULONG empty = 0;
        for (ULONG i = 0; i < faces.GetSize(); ++i)
            if (!sink.faces.count(faces[i])) ++empty;
        std::printf("{\"ok\": true, \"status\": \"ok\", \"triangles\": %lu, \"area\": %s, \"volume\": %s, \"faces\": %lu, \"failed\": %s, \"faces_without_triangles\": %lu, \"output_status\": %d}\n",
                    (unsigned long)sink.tris, num(sink.area).c_str(), num(sink.vol).c_str(), (unsigned long)faces.GetSize(),
                    failed ? "true" : "false", (unsigned long)empty, (int)st);
        return 0;
    }

    // cci file.smp: GlobalCurveIntersect over every pair of the part's curves
    // (my_test_cci_adv / my_test_GlobalCC_intersect).
    if (cmd == "cci" && argc == 3) {
        Part* p = read_part(argv[2]);
        double tol = opt_num("tol", SM_ZONE_TOL_3D);
        std::string pts;
        ULONG total = 0;
        for (ULONG i = 0; i < p->curves.GetSize(); ++i) {
            for (ULONG j = i + 1; j < p->curves.GetSize(); ++j) {
                SmCurve* c1 = p->curves[i];
                SmCurve* c2 = p->curves[j];
                SmSolutionArray sols;
                SmStatus st = c1->GlobalCurveIntersect(c1->GetNaturalInterval(), *c2, c2->GetNaturalInterval(), tol, sols);
                if (st != SM_SUCCESS) {
                    if (verbose()) std::fprintf(stderr, "curves %lu %lu: status %d\n", (unsigned long)i, (unsigned long)j, (int)st);
                    continue;
                }
                for (ULONG k = 0; k < sols.GetSize(); ++k) {
                    SmSolution& s = sols[k];
                    bool range = s.m_eSolutionType == SM_ST_RANGE_OF_VALUES;
                    double t1 = s.m_vStart.m_adParameters[0], t2 = s.m_vStart.m_adParameters[1];
                    SmPoint3d p1, p2;
                    c1->EvaluatePoint(t1, p1);
                    c2->EvaluatePoint(t2, p2);
                    SmVector3d g(p1.x - p2.x, p1.y - p2.y, p1.z - p2.z);
                    std::string e = "{\"curves\": [" + std::to_string(i) + ", " + std::to_string(j) + "], \"t\": [" + num(t1) + ", " + num(t2) +
                                    "], \"point\": " + pt(p1) + ", \"gap\": " + num(std::sqrt(dot(g, g))) + ", \"range\": " + (range ? "true" : "false");
                    if (range) {
                        SmPoint3d q;
                        c1->EvaluatePoint(s.m_vEnd.m_adParameters[0], q);
                        e += ", \"t_end\": [" + num(s.m_vEnd.m_adParameters[0]) + ", " + num(s.m_vEnd.m_adParameters[1]) + "], \"end\": " + pt(q);
                    }
                    pts += (total ? ", " : "") + e + "}";
                    ++total;
                }
            }
        }
        std::printf("{\"ok\": true, \"status\": \"ok\", \"curves\": %lu, \"count\": %lu, \"points\": [%s]}\n", (unsigned long)p->curves.GetSize(),
                    (unsigned long)total, pts.c_str());
        return 0;
    }

    usage();
}
