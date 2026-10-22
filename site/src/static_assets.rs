use tower_http::services::ServeDir;

pub fn service(base_path: proa_core::BasePath) -> axum::Router {
    proa_framework_axum::base_path_assets(
        axum::Router::new()
            .fallback_service(ServeDir::new("public").append_index_html_on_directories(false)),
        base_path,
    )
}

/// A directory of the cadrs repository, served as is: the screenshots the README uses and
/// the app icon, so the site keeps no copies of them.
pub fn repository(path: &str) -> ServeDir {
    ServeDir::new(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(path),
    )
    .append_index_html_on_directories(false)
}

/// The fonts `google_font!` vendors into public/fonts, at the URLs it renders.
pub fn fonts() -> ServeDir {
    ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/public/fonts"))
        .append_index_html_on_directories(false)
}
