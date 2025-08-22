use desktop::app::{events::Message, MulticodeApp, ViewMode};
use iced::Application;
use tempfile::tempdir;

#[test]
fn restores_last_view_mode_after_restart() {
    let dir = tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());

    {
        let (mut app, _) = <MulticodeApp as Application>::new(None);
        let _ = app.handle_message(Message::SwitchViewMode(ViewMode::Schema));
        drop(app);
    }

    {
        let (app, _) = <MulticodeApp as Application>::new(None);
        assert_eq!(app.view_mode(), ViewMode::Schema);
    }
}
