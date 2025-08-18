use iced::futures::stream;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{alignment, subscription, Application, Command, Element, Length, Settings, Subscription, Theme};
use multicode_core::{export, git, search};
use tokio::sync::broadcast;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub fn main() -> iced::Result {
    MulticodeApp::run(Settings::default())
}

#[derive(Debug)]
struct MulticodeApp {
    screen: Screen,
    files: Vec<PathBuf>,
    query: String,
    log: Vec<String>,
    sender: broadcast::Sender<String>,
    settings: UserSettings,
}

#[derive(Debug, Clone)]
enum Screen {
    ProjectPicker,
    Workspace { root: PathBuf },
}

#[derive(Debug, Clone)]
enum Message {
    PickFolder,
    FolderPicked(Option<PathBuf>),
    FilesLoaded(Vec<PathBuf>),
    QueryChanged(String),
    RunSearch,
    SearchFinished(Result<Vec<String>, String>),
    RunParse,
    ParseFinished(Result<(), String>),
    RunGitLog,
    GitFinished(Result<Vec<String>, String>),
    RunExport,
    ExportFinished(Result<(), String>),
    CoreEvent(String),
    SaveSettings,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct UserSettings {
    last_folder: Option<PathBuf>,
}

impl UserSettings {
    fn load() -> Self {
        if let Some(proj) = ProjectDirs::from("com", "multicode", "multicode") {
            let path = proj.config_dir().join("settings.json");
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(s) = serde_json::from_str(&data) {
                    return s;
                }
            }
        }
        Self::default()
    }

    async fn save(self) {
        if let Some(proj) = ProjectDirs::from("com", "multicode", "multicode") {
            let path = proj.config_dir().join("settings.json");
            let _ = fs::create_dir_all(path.parent().unwrap());
            if let Ok(json) = serde_json::to_string_pretty(&self) {
                let _ = fs::write(path, json);
            }
        }
    }
}

impl Application for MulticodeApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let settings = UserSettings::load();
        let (sender, _) = broadcast::channel(100);

        let app = MulticodeApp {
            screen: if let Some(path) = settings.last_folder.clone() {
                Screen::Workspace { root: path }
            } else {
                Screen::ProjectPicker
            },
            files: Vec::new(),
            query: String::new(),
            log: Vec::new(),
            sender,
            settings,
        };

        let cmd = match &app.screen {
            Screen::Workspace { root } => app.load_files(root.clone()),
            _ => Command::none(),
        };

        (app, cmd)
    }

    fn title(&self) -> String {
        String::from("Multicode Desktop")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PickFolder => Command::perform(pick_folder(), Message::FolderPicked),
            Message::FolderPicked(path) => {
                if let Some(root) = path {
                    self.settings.last_folder = Some(root.clone());
                    self.screen = Screen::Workspace { root: root.clone() };
                    multicode_core::meta::watch::spawn(self.sender.clone());
                    return Command::batch([
                        self.load_files(root),
                        Command::perform(self.settings.clone().save(), |_| Message::SaveSettings),
                    ]);
                }
                Command::none()
            }
            Message::FilesLoaded(list) => {
                self.files = list;
                Command::none()
            }
            Message::QueryChanged(q) => {
                self.query = q;
                Command::none()
            }
            Message::RunSearch => {
                let root = self.current_root();
                let query = self.query.clone();
                Command::perform(async move {
                    let results = search::search_metadata(Path::new(&root), &query);
                    Ok::<_, String>(
                        results
                            .into_iter()
                            .map(|r| r.file.display().to_string())
                            .collect(),
                    )
                }, |r| Message::SearchFinished(r))
            }
            Message::SearchFinished(Ok(list)) => {
                for item in list {
                    self.log.push(format!("found {item}"));
                }
                Command::none()
            }
            Message::SearchFinished(Err(e)) => {
                self.log.push(format!("search error: {e}"));
                Command::none()
            }
            Message::RunParse => {
                let sender = self.sender.clone();
                Command::perform(async move {
                    sender.send("parsing".into()).ok();
                    Ok::<_, String>(())
                }, Message::ParseFinished)
            }
            Message::ParseFinished(Ok(())) => Command::none(),
            Message::ParseFinished(Err(e)) => {
                self.log.push(format!("parse error: {e}"));
                Command::none()
            }
            Message::RunGitLog => {
                Command::perform(async move { git::log().map_err(|e| e.to_string()) }, Message::GitFinished)
            }
            Message::GitFinished(Ok(lines)) => {
                self.log.extend(lines);
                Command::none()
            }
            Message::GitFinished(Err(e)) => {
                self.log.push(format!("git error: {e}"));
                Command::none()
            }
            Message::RunExport => {
                Command::perform(async move {
                    export::serialize_viz_document("{}").ok_or_else(|| "export".to_string()).map(|_| ())
                }, Message::ExportFinished)
            }
            Message::ExportFinished(Ok(())) => Command::none(),
            Message::ExportFinished(Err(e)) => {
                self.log.push(format!("export error: {e}"));
                Command::none()
            }
            Message::CoreEvent(ev) => {
                self.log.push(ev);
                Command::none()
            }
            Message::SaveSettings => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if matches!(self.screen, Screen::Workspace { .. }) {
            let rx = self.sender.subscribe();
            subscription::run_with_id(
                "core-events",
                stream::unfold(rx, |mut rx| async {
                    match rx.recv().await {
                        Ok(ev) => Some((Message::CoreEvent(ev), rx)),
                        Err(_) => None,
                    }
                }),
            )
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::ProjectPicker => {
                let content = column![
                    text("Select a project folder"),
                    button("Pick folder").on_press(Message::PickFolder),
                ]
                .align_items(alignment::Alignment::Center)
                .spacing(20);

                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }
            Screen::Workspace { .. } => {
                let menu = row![
                    button("Parse").on_press(Message::RunParse),
                    button("Search").on_press(Message::RunSearch),
                    button("Git Log").on_press(Message::RunGitLog),
                    button("Export").on_press(Message::RunExport),
                ]
                .spacing(10);

                let sidebar = container(
                    scrollable(column(
                        self.files
                            .iter()
                            .map(|p| {
                                button(text(p.file_name().unwrap().to_string_lossy().to_string()))
                                    .on_press(Message::CoreEvent(format!("open {:?}", p)))
                                    .into()
                            })
                            .collect::<Vec<Element<Message>>>()
                    )),
                )
                .width(200);

                let content = column![
                    text_input("search", &self.query).on_input(Message::QueryChanged),
                    scrollable(column(
                        self.log
                            .iter()
                            .cloned()
                            .map(|l| text(l).into())
                            .collect::<Vec<Element<Message>>>()
                    )),
                ];

                let body = row![sidebar, content].spacing(10);

                column![menu, body, text("Ready")]
                    .spacing(10)
                    .into()
            }
        }
    }
}

fn pick_folder() -> impl std::future::Future<Output = Option<PathBuf>> {
    async { std::env::current_dir().ok() }
}

impl MulticodeApp {
    fn current_root(&self) -> String {
        match &self.screen {
            Screen::Workspace { root } => root.to_string_lossy().to_string(),
            Screen::ProjectPicker => String::new(),
        }
    }

    fn load_files(&self, root: PathBuf) -> Command<Message> {
        Command::perform(async move {
            let mut files = Vec::new();
            if let Ok(entries) = fs::read_dir(&root) {
                for entry in entries.flatten() {
                    if let Ok(ft) = entry.file_type() {
                        if ft.is_file() {
                            files.push(entry.path());
                        }
                    }
                }
            }
            files
        }, Message::FilesLoaded)
    }
}

