use axum::{http::{HeaderMap, StatusCode, header}, Json};
use backend::config::ServerConfig;
use backend::server::{plugins_update, PluginToggle, SERVER_CONFIG};
use backend::reload_plugins_state;
use std::path::PathBuf;
use std::fs;

struct SettingsGuard {
    path: PathBuf,
    data: String,
}
impl Drop for SettingsGuard {
    fn drop(&mut self) {
        fs::write(&self.path, &self.data).unwrap();
        reload_plugins_state();
    }
}

#[tokio::test]
async fn plugins_update_toggles_plugin() {
    let settings_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../frontend/settings.json");
    let original = fs::read_to_string(&settings_path).unwrap();
    let _guard = SettingsGuard { path: settings_path.clone(), data: original };

    let _ = SERVER_CONFIG.set(ServerConfig { token: Some("secret".into()), ..Default::default() });
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, "Bearer secret".parse().unwrap());
    let body = PluginToggle { name: "test".into(), enabled: false };

    let _ = plugins_update(headers, Json(body)).await.unwrap();

    let json: serde_json::Value = serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    assert_eq!(json["plugins"]["test"], false);
}

#[tokio::test]
async fn plugins_update_requires_auth() {
    let _ = SERVER_CONFIG.set(ServerConfig { token: Some("secret".into()), ..Default::default() });
    let headers = HeaderMap::new();
    let body = PluginToggle { name: "test".into(), enabled: true };
    let err = plugins_update(headers, Json(body)).await.err().unwrap();
    assert_eq!(err, StatusCode::UNAUTHORIZED);
}

