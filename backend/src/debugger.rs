use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

fn log_action(action: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/debug.log")
    {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let _ = writeln!(file, "[{}] {}", ts, action);
    }
}

#[cfg_attr(not(test), tauri::command)]
pub fn debug_run() {
    log_action("run");
}

#[cfg_attr(not(test), tauri::command)]
pub fn debug_step() {
    log_action("step");
}

#[cfg_attr(not(test), tauri::command)]
pub fn debug_break() {
    log_action("break");
}
