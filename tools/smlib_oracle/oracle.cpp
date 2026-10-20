// A test oracle for the native kernel on NVIDIA's SMLib (usd-brep, Apache-2.0).
//
//   smlib_oracle props  in.smb
//   smlib_oracle fillet in.smb out.smb radius x y z [x y z ...]
//
// The native kernel writes its bodies as .smb with `native::oracle::smb_text`.
// Edges are picked as the ones nearest the given points. Prints one line of JSON: ok, volume,
// area, the face, edge and vertex counts and whether the result is a manifold solid.
#include <SmApiBrep.h>
#include <SmApiGeneral.h>
#include <SmApiImportExport.h>
#include <SmApiQueries.h>
#include <SmBrep.h>
#include <SmMessages.h>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>

static void report(SmBrep* b, const char* status) {
    double vol = 0.0, area = 0.0;
    SmApiBrepComputeVolume(b, 1e-6, vol);
    SmApiBrepComputeArea(b, 1e-6, area);
    SmTArray<SmFace*> faces;
    SmTArray<SmEdge*> edges;
    SmApiGetFaces(b, faces);
    SmApiGetEdges(b, edges);
    if (std::getenv("ORACLE_VERBOSE")) {
        for (ULONG i = 0; i < faces.GetSize(); ++i) {
            double fa = 0.0;
            SmApiFaceComputeArea(faces[i], 1e-6, fa);
            std::fprintf(stderr, "face %lu area %.9g\n", (unsigned long)i, fa);
        }
    }
    SmBoolean solid = FALSE;
    SmApiBrepIsManifoldSolid(b, solid);
    std::printf("{\"ok\": %s, \"status\": \"%s\", \"volume\": %.12g, \"area\": %.12g, \"faces\": %lu, \"edges\": %lu, \"solid\": %s}\n",
                std::string(status) == "ok" ? "true" : "false", status, vol, area, (unsigned long)faces.GetSize(),
                (unsigned long)edges.GetSize(), solid ? "true" : "false");
}

static SmBrep* read(const char* path) {
    SmBrep* b = nullptr;
    SmApiStatus st = SmApiReadBrepFromFile(path, TRUE, TRUE, b);
    if (st != SM_SUCCESS || !b) {
        std::printf("{\"ok\": false, \"status\": \"can't read %s (%d)\"}\n", path, (int)st);
        std::exit(1);
    }
    return b;
}

// SMLib's errors on stderr when ORACLE_VERBOSE is set (where a read or an operation failed).
static void on_error(SmStatus e, const TCHAR* const file, ULONG line, const TCHAR* const msg) {
    if (std::getenv("ORACLE_VERBOSE")) std::fprintf(stderr, "smlib error %d at %s:%lu %s\n", (int)e, file ? file : "?", (unsigned long)line, msg ? msg : "");
}

int main(int argc, char** argv) {
    if (argc < 3) {
        std::fprintf(stderr, "usage: smlib_oracle props|fillet ...\n");
        return 2;
    }
    SmApiCreateContext();
    smos_SetErrorCallback(on_error);
    std::string cmd = argv[1];
    SmBrep* b = read(argv[2]);
    if (cmd == "props") {
        report(b, "ok");
        return 0;
    }
    if (cmd == "fillet" && argc >= 8 && (argc - 5) % 3 == 0) {
        double r = std::atof(argv[4]);
        SmTArray<SmEdge*> all, picked;
        SmApiGetEdges(b, all);
        for (int i = 5; i + 2 < argc; i += 3) {
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
            if (best) picked.Add(best);
        }
        SmApiStatus s = SmApiFilletEdges(b, picked, r, 1, 1, 1.0);
        if (s != SM_SUCCESS) {
            std::printf("{\"ok\": false, \"status\": \"fillet failed (%d)\"}\n", (int)s);
            return 1;
        }
        SmApiWriteBrepToFile(b, argv[3], TRUE);
        report(b, "ok");
        return 0;
    }
    std::fprintf(stderr, "bad arguments\n");
    return 2;
}
