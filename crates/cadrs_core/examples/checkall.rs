//! `cargo run -r -p cadrs_core --example checkall -- <data dir> [<document id>]`: rebuilds stored
//! documents headlessly (`cadrs_core::check`) and prints one JSON line per Part Studio with every
//! feature in error or warning. Without an id it lists the store's document ids, one per line.

use cadrs_core::Store;
use cadrs_core::check::Severity;

fn esc(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            '\n' => vec!['\\', 'n'],
            c if (c as u32) < 0x20 => vec![' '],
            c => vec![c],
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let store = Store::new(&args[1]);
    let (lib, _) = store.list();
    let Some(id) = args.get(2) else {
        for e in lib.entries.iter().filter(|e| e.meta.trashed.is_none()) {
            println!("{}", e.id);
        }
        return;
    };
    cadrs_core::derived::set_document_loader(Some(cadrs_core::derived::store_loader(
        store.clone(),
    )));
    let Some(entry) = lib.entries.iter().find(|e| e.id.to_string() == *id) else {
        println!("{{\"doc\":\"{id}\",\"load_error\":\"not found\"}}");
        return;
    };
    let file = match store.load(entry.id) {
        Ok(f) => f,
        Err(e) => {
            println!(
                "{{\"doc\":\"{id}\",\"name\":\"{}\",\"load_error\":\"{}\"}}",
                esc(&entry.name),
                esc(&e.to_string())
            );
            return;
        }
    };
    for s in cadrs_core::check::check(&file.document, None) {
        let issues: Vec<String> = s
            .issues
            .iter()
            .map(|i| {
                let sev = if matches!(i.severity, Severity::Error) {
                    "error"
                } else {
                    "warning"
                };
                format!(
                    "{{\"feature\":\"{}\",\"severity\":\"{sev}\",\"detail\":\"{}\"}}",
                    esc(&i.feature),
                    esc(&i.detail)
                )
            })
            .collect();
        println!(
            "{{\"doc\":\"{id}\",\"name\":\"{}\",\"studio\":\"{}\",\"features\":{},\"parts\":{},\"secs\":{:.2},\"issues\":[{}]}}",
            esc(&entry.name),
            esc(&s.name),
            s.features,
            s.parts,
            s.elapsed.as_secs_f64(),
            issues.join(",")
        );
    }
}
