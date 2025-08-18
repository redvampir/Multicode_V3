use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

fn log_action(action: &str) {
    if let Some(mut path) = dirs::data_dir() {
        path.push("multicode");
        path.push("logs");
        if fs::create_dir_all(&path).is_ok() {
            path.push("debug.log");
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let _ = writeln!(file, "[{}] {}", ts, action);
            }
        }
    }
}

pub fn debug_run() {
    log_action("запуск");
}

pub fn debug_step() {
    log_action("шаг");
}

pub fn debug_break() {
    log_action("остановка");
}
