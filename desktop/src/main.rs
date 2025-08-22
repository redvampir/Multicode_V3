use iced::{Sandbox, Settings};
use desktop::ui::MainUI;

pub fn main() -> iced::Result {
    MainUI::run(Settings::default())
}
