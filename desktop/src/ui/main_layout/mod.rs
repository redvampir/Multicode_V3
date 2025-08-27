pub mod state;
pub mod update;
pub mod view;

pub use state::{Dragging, MainUI};
pub use update::{update, MainMessage};
pub use view::view;

use iced::{executor, time, Application, Command, Element, Subscription, Theme};
use std::time::Duration;

const SYNC_INTERVAL: Duration = Duration::from_millis(500);

impl Application for MainUI {
    type Executor = executor::Default;
    type Message = MainMessage;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (MainUI::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Multicode")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        update(self, message);
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(SYNC_INTERVAL).map(|_| MainMessage::SyncTick)
    }

    fn view(&self) -> Element<Self::Message> {
        view(self)
    }
}
