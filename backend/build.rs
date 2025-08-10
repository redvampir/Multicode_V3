fn main() {
    if std::env::var("SKIP_TAURI_BUILD").is_ok() {
        return;
    }
    tauri_build::build()
}
