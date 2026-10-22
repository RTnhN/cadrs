use std::net::{IpAddr, SocketAddr};

pub struct Config {
    pub address: SocketAddr,
    pub base_path: proa_core::BasePath,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let base_path =
            proa_core::BasePath::new(&std::env::var("PROA_BASE_PATH").unwrap_or_default())?;
        let host: IpAddr = std::env::var("PROA_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string())
            .parse()?;
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()?;
        Ok(Self {
            address: SocketAddr::new(host, port),
            base_path,
        })
    }
}
