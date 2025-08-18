use ::config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub token: Option<String>,
    pub api_key: Option<String>,
    pub api_base_url: String,
    pub connect_timeout_secs: u64,
    pub request_timeout_secs: u64,
    pub max_body_size: usize,
    pub max_requests_per_second: u64,
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
            api_base_url: "https://api.example.com".into(),
            connect_timeout_secs: 5,
            request_timeout_secs: 30,
            max_body_size: 1024 * 1024,
            max_requests_per_second: 100,
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
