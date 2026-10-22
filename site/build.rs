fn main() {
    println!("cargo:rerun-if-changed=public");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=proa-fonts.lock");
    // Vendors the fonts `google_font!` declares into public/fonts (recorded in proa-fonts.lock)
    // and generates their `proa_fonts::Font` consts.
    let fonts =
        proa_fonts_build::collect::CollectOptions::from_build_env().expect("cargo build env");
    if let Err(error) = proa_fonts_build::collect::run(&fonts) {
        panic!("proa fonts: {error}");
    }
}
