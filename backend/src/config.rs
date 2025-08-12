use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub token: Option<String>,
    pub api_key: Option<String>,
    /// Disable authentication (debug builds only)
    pub disable_auth: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 3001,
            token: None,
            api_key: None,
            disable_auth: false,
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        Config::builder()
            .add_source(Environment::with_prefix("SERVER").separator("_"))
            .build()
            .ok()
            .and_then(|c| c.try_deserialize().ok())
            .unwrap_or_default()
    }
}
