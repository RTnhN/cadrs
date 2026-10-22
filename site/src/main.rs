use std::time::Duration;

use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

mod app;
// Vendored Proa UI source, kept byte for byte as `proa ui add` installed it (see fmt.sh);
// the site uses part of it.
#[allow(unused)]
mod components;
mod config;
mod endpoints;
mod error;
mod github;
mod icons;
mod layouts;
mod pages;
mod routes;
mod static_assets;
mod theme;

mod __proa_fonts_generated {
    include!(concat!(env!("OUT_DIR"), "/proa_fonts.rs"));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let config = config::Config::from_env()?;
    let state = app::AppState;
    let app = routes::router(state, config.base_path);
    let listener = TcpListener::bind(config.address).await?;
    tracing::info!(address = %listener.local_addr()?, "Proa site listening");
    let _ = proa_framework_axum::serve(listener, app, Duration::from_secs(30)).await?;
    Ok(())
}
