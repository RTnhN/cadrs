//! Where SMLib's tests point at a model's pieces, for `tools/smlib_to_rust.py` (which gives
//! the points to the SMLib oracle): one JSON line per file (`file.smb`, or `file.smp#brep`)
//! with its edges' middle points and ends and its faces' centroids in SMLib's file order, and
//! whether every face is planar.
//!
//!     cargo run -r -p cadrs_kernel --example smlib_points -- a.smb b.smp#1 ...

use cadrs_kernel::native::math::P3;
use cadrs_kernel::native::smb;
use cadrs_kernel::native::surface::Surface;

fn p(q: &P3) -> String {
    format!("[{:.17e}, {:.17e}, {:.17e}]", q.x, q.y, q.z)
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn one(arg: &str) -> Result<String, String> {
    let (file, brep) = match arg.rsplit_once('#') {
        Some((f, b)) => (f, b.parse::<usize>().map_err(|e| e.to_string())?),
        None => (arg, 0),
    };
    let bytes = std::fs::read(file).map_err(|e| e.to_string())?;
    let b = if file.ends_with(".smp") {
        let mut part = smb::read_part(&bytes)?;
        if brep >= part.breps.len() {
            return Err(format!("{} B-reps", part.breps.len()));
        }
        part.breps.swap_remove(brep)
    } else {
        smb::read_brep(&bytes)?
    };
    let edges: Vec<String> = b.edge_mids.iter().map(p).collect();
    let ends: Vec<String> = b
        .edge_ends
        .iter()
        .map(|(a, c)| format!("[{}, {}]", p(a), p(c)))
        .collect();
    let faces: Vec<String> = b
        .face_centroids
        .iter()
        .map(|c| c.as_ref().map_or("null".into(), p))
        .collect();
    let planar = b
        .body
        .faces
        .iter()
        .all(|f| matches!(f.surf, Surface::Plane { .. }));
    Ok(format!(
        "\"edges\": [{}], \"ends\": [{}], \"faces\": [{}], \"planar\": {planar}, \"solid\": {}",
        edges.join(", "),
        ends.join(", "),
        faces.join(", "),
        !b.body.solids.is_empty()
    ))
}

fn main() {
    for arg in std::env::args().skip(1) {
        match one(&arg) {
            Ok(body) => println!("{{\"file\": {}, {body}}}", json_str(&arg)),
            Err(e) => println!(
                "{{\"file\": {}, \"error\": {}}}",
                json_str(&arg),
                json_str(&e)
            ),
        }
    }
}
