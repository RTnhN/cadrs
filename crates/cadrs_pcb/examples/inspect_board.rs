//! Scratch: prints a stored document's PCB Studio boards (not committed).
use cadrs_core::Store;
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args[0] == "copy" { return copy(&args[1..]); }
    if args[0] == "studio" { return studio(&args[1..]); }
    let store = Store::new(Store::default_root().unwrap());
    let id = cadrs_core::DocumentId::from_u128(u128::from_str_radix(&args[0].replace("-", ""), 16).unwrap());
    let f = store.load(id).unwrap();
    for el in &f.document.elements {
        let Some(p) = el.pcb() else { continue };
        println!("STUDIO {} components={}", el.name, p.components.len());
        for b in &p.boards {
            println!(" BOARD {:?} {} source={:?}", b.id, b.name(), b.source);
            let Some(d) = b.design() else { continue };
            let bd = &d.board;
            println!("  thickness {} nets {:?}", cadrs_eda::units::to_mm(bd.thickness), bd.nets);
            for s in bd.shapes.iter().filter(|s| s.layer == cadrs_eda::layer::Layer::Outline) {
                println!("  OUTLINE {:?}", s.shape.geom);
            }
            for fp in &bd.footprints {
                let pl = &fp.placement;
                println!("  FP {} {} at=({:.3},{:.3}) {} {:?} sym={:?} models={}", fp.reference(), fp.footprint.id, cadrs_eda::units::to_mm(pl.at.x), cadrs_eda::units::to_mm(pl.at.y), pl.angle, pl.side, fp.symbol.is_some(), fp.footprint.models.len());
                for pad in &fp.footprint.pads {
                    let c = pl.apply(pad.at);
                    println!("     pad {} ({:.3},{:.3}) net={:?} drill={:?}", pad.number, cadrs_eda::units::to_mm(c.x), cadrs_eda::units::to_mm(c.y), pad.net, pad.drill.as_ref().map(|d| cadrs_eda::units::to_mm(d.size.w)));
                }
            }
            println!("  tracks {} vias {} zones {} texts {}", bd.tracks.len(), bd.vias.len(), bd.zones.len(), bd.texts.len());
            for sh in &d.schematic.sheets { println!("  SHEET {} symbols {} wires {}", sh.name, sh.symbols.len(), sh.wires.len()); for s in &sh.symbols { println!("    SYM {:?}", s.fields.iter().filter(|f| f.name=="Reference"||f.name=="Value"||f.name=="Footprint").map(|f| f.value().to_string()).collect::<Vec<_>>()); } }
        }
    }
}
// `copy <src> <name>`: Store::copy_document.
#[allow(dead_code)]
pub fn copy(args: &[String]) {
    let store = Store::new(Store::default_root().unwrap());
    let src = cadrs_core::DocumentId::from_u128(u128::from_str_radix(&args[0].replace('-', ""), 16).unwrap());
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let e = store.copy_document(src, cadrs_core::DocumentId::new(), &args[1], "rouan", now).unwrap();
    println!("COPIED {} {}", e.id, e.name);
}
// `studio <doc> <tab>`: features, parts (bounds) and round faces' axes.
#[allow(dead_code)]
pub fn studio(args: &[String]) {
    let store = Store::new(Store::default_root().unwrap());
    let id = cadrs_core::DocumentId::from_u128(u128::from_str_radix(&args[0].replace('-', ""), 16).unwrap());
    let f = store.load(id).unwrap();
    let el = f.document.elements.iter().find(|e| e.name == args[1]).expect("tab");
    for (i, ft) in el.features().iter().enumerate() {
        let extra = match &ft.kind {
            cadrs_core::FeatureKind::Sketch(s) => format!("plane={:?}", s.plane),
            cadrs_core::FeatureKind::MateConnector(m) => format!("{:?} off={:?}", m.origin, m.offset),
            _ => String::new(),
        };
        println!("F{i} {} [{}] {extra}", ft.name, cadrs_core::feature_list::type_label(&ft.kind));
    }
    let b = cadrs_core::rebuild::build(el.features());
    for (fid, e) in &b.errors { println!("ERR {:?} {e}", fid); }
    for p in &b.parts {
        let name = cadrs_core::parts::display_name(p, el.part_props());
        let (lo, hi) = p.solid.bounds().unwrap_or(([0.0;3],[0.0;3]));
        println!("PART {name} feat={} lo=({:.2},{:.2},{:.2}) hi=({:.2},{:.2},{:.2})", p.feature.0, lo[0],lo[1],lo[2],hi[0],hi[1],hi[2]);
        if args.len() > 2 && name.to_lowercase().contains(&args[2].to_lowercase()) {
            for fc in &p.solid.faces {
                if let (Some((o, d)), Some(r)) = (fc.axis, fc.radius) {
                    println!("   AXIS o=({:.3},{:.3},{:.3}) d=({:.3},{:.3},{:.3}) r={r:.3}", o[0],o[1],o[2],d[0],d[1],d[2]);
                }
            }
        }
    }
}
