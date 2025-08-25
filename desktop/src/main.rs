use desktop::ui::MainUI;
use iced::{Sandbox, Settings};
use tracing_subscriber::EnvFilter;

pub fn main() -> iced::Result {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("desktop=debug"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
    MainUI::run(Settings::default())
}
