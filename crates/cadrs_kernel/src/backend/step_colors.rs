//! Color-aware STEP XDE bridge, kept here until the upstream Rust bindings expose styles.
#[cxx::bridge(namespace = "cadrs_colors")]
pub(super) mod ffi {
    #[derive(Debug, Clone, Copy)]
    struct Color { rgba: [u8; 4], present: bool }
    struct Part { brep: Vec<u8>, name: String, faces: Vec<Color> }
    struct Occurrence { part: usize, matrix: [f64; 12], name: String, color: Color }
    struct Model { name: String, parts: Vec<Part>, occurrences: Vec<Occurrence> }
    unsafe extern "C++" {
        include!("step_colors.hxx");
        fn read_step_colors(path: &str) -> Result<Model>;
        #[allow(dead_code)] // Generates the colored STEP used by the kernel regression test.
        fn step_color_fixture() -> Result<Vec<u8>>;
    }
}
