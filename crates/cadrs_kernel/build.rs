fn main() {
    // OCCT's OSD layer (built statically by the `occt` feature) calls advapi32 (security
    // descriptors, registry, GetUserNameW), which opencascade-sys doesn't link on Windows.
    let occt = std::env::var_os("CARGO_FEATURE_OCCT").is_some();
    let windows = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "windows");
    #[cfg(feature = "occt")]
    {
        cxx_build::bridge("src/backend/step_colors.rs")
            .file("src/backend/step_colors.cc")
            .include("src/backend")
            .include(occt_sys::occt_path().join("include"))
            .flag_if_supported("-std=c++17")
            .compile("cadrs-step-colors");
        println!("cargo:rerun-if-changed=src/backend/step_colors.rs");
        println!("cargo:rerun-if-changed=src/backend/step_colors.cc");
        println!("cargo:rerun-if-changed=src/backend/step_colors.hxx");
    }
    if occt && windows {
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }
}
