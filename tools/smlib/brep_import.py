"""SMLib's B-rep import regression (tests/prog_test/src/smTopoTest.cpp:5037,
`my_test_brep_import`, the `brep-import` suite): `my_test_MakeFace` reads
`MakeFace/ApproximateSphere.smp` (one surface, four curves, no B-reps), makes a face of the
surface over its natural domain (`SmBrep::CreateFaceFromSurface`) and checks the B-rep's counts
(`ValidateCounts(1, 1, 2, 0, 0, 1, 2, 2)`: one face, one edge (its seam, a manifold edge) and two
vertices (its poles), two regions and two shells).

Here the part's reading (`smb::read_part`) and its counts are checked; making a face of a bare
surface over its whole domain is `missing_face_from_surface`: the kernel has no such operation
(its faces come from profiles, primitives, file readers and operations), so the case is a known
failure until it does.
"""
from smlib_to_rust import Case, Fixture

SOURCE = "smTopoTest.cpp"
TITLE = "B-rep import"

PRELUDE = r'''
use cadrs_kernel::native::smb;
use cadrs_kernel::native::surface::Surface;
use cadrs_kernel::native::topo::Body;

const SUITE: &str = "brep_import";

/// MISSING API: a face of a bare surface over its natural domain (SMLib's
/// `SmBrep::CreateFaceFromSurface`, iso-curve edges, poles as vertices).
fn missing_face_from_surface(_s: &Surface) -> Result<Body, String> {
    Err("missing API: no face from a bare surface over its natural domain (SmBrep::CreateFaceFromSurface)".into())
}

/// `my_test_MakeFace`: `file` holds `surfaces` surfaces, `curves` curves and no B-reps; its
/// surface made a face has `faces` faces, `edges` edges and `verts` vertices.
fn make_face(file: &str, surfaces: usize, curves: usize, faces: usize, edges: usize, verts: usize) -> Result<(), String> {
    let bytes = std::fs::read(data(SUITE, file)).map_err(|e| format!("{file}: {e}"))?;
    let p = smb::read_part(&bytes)?;
    if (p.surfaces.len(), p.curves.len(), p.breps.len()) != (surfaces, curves, 0) {
        return Err(format!("{} surfaces, {} curves, {} B-reps", p.surfaces.len(), p.curves.len(), p.breps.len()));
    }
    let b = missing_face_from_surface(&p.surfaces[0])?;
    if let Some(i) = b.check().first() {
        return Err(format!("invalid: {i}"));
    }
    let ne = b.edges.iter().filter(|e| !e.degenerate).count();
    if (b.faces.len(), ne, b.verts.len()) != (faces, edges, verts) {
        return Err(format!("{} faces, {ne} edges, {} vertices", b.faces.len(), b.verts.len()));
    }
    Ok(())
}
'''


def extract(ctx):
    src = ctx.source(SOURCE)
    assert "ApproximateSphere.smp" in src and "ValidateCounts(1, 1, 2, 0, 0, 1, 2, 2)" in src
    fx = Fixture("TestFiles/pt_TestFiles/MakeFace/ApproximateSphere.smp")
    return [Case("make_face_approximate_sphere",
                 "SMLib B-rep import: `ApproximateSphere.smp` (1 surface, 4 curves, no B-reps), its surface made a face: 1 face, 1 edge (the seam), 2 vertices (the poles).",
                 'make_face("ApproximateSphere.smp", 1, 4, 1, 1, 2)', [fx])]
