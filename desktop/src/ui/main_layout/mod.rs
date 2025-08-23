pub mod state;
pub mod update;
pub mod view;

pub use state::{Dragging, MainUI};
pub use update::{update, MainMessage};
pub use view::view;

use iced::{Element, Sandbox};

impl Sandbox for MainUI {
    type Message = MainMessage;

    fn new() -> Self {
        MainUI::default()
    }

    fn title(&self) -> String {
        String::from("Multicode")
    }

    fn update(&mut self, message: Self::Message) {
        update(self, message);
    }

    fn view(&self) -> Element<Self::Message> {
        view(self)
    }
}
