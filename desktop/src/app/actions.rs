use std::collections::HashSet;
use std::path::PathBuf;

use iced::futures::stream;
use iced::{event, subscription, Application, Command, Element, Subscription, Theme};
use tokio::sync::broadcast;

use super::events::Message;
use super::{AppTheme, CreateTarget, EditorMode, MulticodeApp, Screen, UserSettings, ViewMode};

impl Application for MulticodeApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Option<PathBuf>;

    fn new(flags: Option<PathBuf>) -> (Self, Command<Message>) {
        let mut settings = UserSettings::load();
        if let Some(path) = flags {
            settings.add_recent_folder(path);
        }
        let (sender, _) = broadcast::channel(100);

        let mut app = MulticodeApp {
            screen: if let Some(path) = settings.last_folders.first().cloned() {
                match settings.editor_mode {
                    EditorMode::Text => Screen::TextEditor { root: path },
                    EditorMode::Visual => Screen::VisualEditor { root: path },
                }
            } else {
                Screen::ProjectPicker
            },
            view_mode: ViewMode::Code,
            files: Vec::new(),
            tabs: Vec::new(),
            active_tab: None,
            search_term: String::new(),
            replace_term: String::new(),
            search_results: Vec::new(),
            show_search_panel: false,
            current_match: None,
            new_file_name: String::new(),
            new_directory_name: String::new(),
            create_target: CreateTarget::File,
            rename_file_name: String::new(),
            search_query: String::new(),
            favorites: settings.favorites.clone(),
            query: String::new(),
            show_command_palette: false,
            log: Vec::new(),
            project_search_results: Vec::new(),
            goto_line: None,
            show_terminal: false,
            terminal_cmd: String::new(),
            terminal_child: None,
            show_terminal_help: false,
            sender,
            settings,
            expanded_dirs: HashSet::new(),
            context_menu: None,
            show_create_file_confirm: false,
            show_delete_confirm: false,
            pending_action: None,
            hotkey_capture: None,
            shortcut_capture: None,
            settings_warning: None,
            loading: false,
            diff_error: None,
            show_meta_dialog: false,
            meta_tags: String::new(),
            meta_links: String::new(),
            meta_comment: String::new(),
            show_meta_panel: false,
            tab_drag: None,
        };

        let cmd = match &app.screen {
            Screen::TextEditor { root } | Screen::VisualEditor { root } => {
                let mut cmds = vec![app.load_files(root.clone())];
                if let Some(entry) = app.settings.default_entry.clone() {
                    cmds.push(app.handle_message(Message::SelectFile(entry)));
                }
                Command::batch(cmds)
            }
            _ => Command::none(),
        };

        (app, cmd)
    }

    fn title(&self) -> String {
        String::from("Multicode Desktop")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        self.handle_message(message)
    }

    fn subscription(&self) -> Subscription<Message> {
        if matches!(
            self.screen,
            Screen::TextEditor { .. } | Screen::VisualEditor { .. }
        ) {
            let rx = self.sender.subscribe();
            let core = subscription::run_with_id(
                "core-events",
                stream::unfold(rx, |mut rx| async {
                    match rx.recv().await {
                        Ok(ev) => Some((Message::CoreEvent(ev), rx)),
                        Err(_) => None,
                    }
                }),
            );
            let events = event::listen().map(Message::IcedEvent);
            Subscription::batch([core, events])
        } else {
            Subscription::none()
        }
    }

    fn theme(&self) -> Theme {
        match self.settings.theme {
            AppTheme::Light => Theme::Light,
            AppTheme::Dark => Theme::Dark,
        }
    }

    fn view(&self) -> Element<Message> {
        self.render()
    }
}
