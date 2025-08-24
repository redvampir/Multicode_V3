use iced::{Sandbox, Settings};
use desktop::ui::MainUI;
use tracing_subscriber::EnvFilter;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    MainUI::run(Settings::default())
}
