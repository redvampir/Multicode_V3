use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::num::NonZeroUsize;
use std::path::PathBuf;

use iced::futures::stream;
use iced::{event, subscription, Application, Command, Element, Subscription, Theme};
use tokio::sync::broadcast;

use super::command_palette::COMMANDS;
use super::command_translations::command_name;
use super::events::Message;
use super::{
    AppTheme, CreateTarget, EditorMode, Language, LogLevel, MulticodeApp, Screen, UserSettings,
};
use crate::search::{fuzzy, index::SearchIndex};
use crate::sync::{ChangeTracker, SyncEngine};
use crate::visual::palette::{PaletteBlock, DEFAULT_CATEGORY};
use crate::visual::translations::block_synonyms;
use lru::LruCache;
use multicode_core::parse_blocks;
use multicode_core::parser::Lang;

pub(super) fn build_command_index() -> SearchIndex<&'static str> {
    let mut index = SearchIndex::new();
    for cmd in COMMANDS {
        let en = command_name(cmd, Language::English).to_lowercase();
        let ru = command_name(cmd, Language::Russian).to_lowercase();
        for kw in en.split_whitespace().chain(ru.split_whitespace()) {
            index.insert(kw, cmd.id);
        }
    }
    index
}

pub(super) fn build_block_index(palette: &[PaletteBlock]) -> SearchIndex<usize> {
    let mut index = SearchIndex::new();
    for (i, block) in palette.iter().enumerate() {
        if let Some(en) = block.info.translations.get("en") {
            let en = en.to_lowercase();
            for kw in en.split_whitespace() {
                index.insert(kw, i);
            }
        }
        if let Some(ru) = block.info.translations.get("ru") {
            let ru = ru.to_lowercase();
            for kw in ru.split_whitespace() {
                index.insert(kw, i);
            }
        }
        let kind = block.info.kind.to_lowercase();
        for kw in kind.split_whitespace() {
            index.insert(kw, i);
        }
        for tag in &block.info.tags {
            index.insert(&tag.to_lowercase(), i);
        }
        if let Some(syns) = block_synonyms(&block.info.kind) {
            for s in syns {
                index.insert(&s.to_lowercase(), i);
            }
        }
    }
    index
}

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
        let use_index = settings.search.use_index;
        let cache_size = settings.search.cache_size;
        let sync_settings = settings.sync.clone();
        let (sender, _) = broadcast::channel(100);
        let fav_files = settings.favorites.clone();
        let (palette, palette_categories) = load_palette();

        let recent_commands: VecDeque<String> = settings.recent_commands.clone().into();
        let mut command_counts: HashMap<String, usize> = HashMap::new();
        for cmd in &recent_commands {
            *command_counts.entry(cmd.clone()).or_insert(0) += 1;
        }
        let mut command_trigrams: HashMap<&'static str, fuzzy::TrigramSet> = HashMap::new();
        for cmd in COMMANDS {
            let name = command_name(cmd, settings.language);
            command_trigrams.insert(cmd.id, fuzzy::trigram_set(&name));
        }

        let command_index = if use_index {
            Some(build_command_index())
        } else {
            None
        };
        let block_index = if use_index {
            Some(build_block_index(&palette))
        } else {
            None
        };
        let cap = NonZeroUsize::new(cache_size).unwrap_or_else(|| NonZeroUsize::new(1).unwrap());

        let view_mode = settings.last_view_mode;
        let screen = if let Some(path) = settings.last_folders.first().cloned() {
            match settings.editor_mode {
                EditorMode::Text => Screen::TextEditor { root: path },
                EditorMode::Visual => Screen::VisualEditor { root: path },
                EditorMode::Split => Screen::Split { root: path },
            }
        } else {
            Screen::ProjectPicker
        };

        let mut app = MulticodeApp {
            screen,
            view_mode,
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
            favorites: fav_files,
            query: String::new(),
            show_command_palette: false,
            log: Vec::new(),
            min_log_level: LogLevel::Info,
            project_search_results: Vec::new(),
            goto_line: None,
            goto_line_input: String::new(),
            show_goto_line_modal: false,
            show_terminal: false,
            terminal_cmd: String::new(),
            terminal_child: None,
            show_terminal_help: false,
            sender,
            settings,
            expanded_dirs: HashSet::new(),
            context_menu: None,
            selected_path: None,
            show_create_file_confirm: false,
            show_delete_confirm: false,
            pending_action: None,
            shortcut_capture: None,
            settings_warning: None,
            loading: false,
            diff_error: None,
            show_meta_dialog: false,
            meta_tags: String::new(),
            meta_links: String::new(),
            meta_comment: String::new(),
            autocomplete: None,
            show_meta_panel: false,
            tab_drag: None,
            palette,
            palette_categories,
            show_block_palette: false,
            palette_query: String::new(),
            palette_drag: None,
            change_tracker: ChangeTracker::default(),
            sync_engine: SyncEngine::new(Lang::Rust, sync_settings),
            recent_commands,
            command_counts,
            command_trigrams,
            command_index,
            block_index,
            command_cache: RefCell::new(LruCache::new(cap)),
            block_cache: RefCell::new(LruCache::new(cap)),
        };

        let cmd = match &app.screen {
            Screen::TextEditor { root }
            | Screen::VisualEditor { root }
            | Screen::Split { root } => {
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
            Screen::TextEditor { .. } | Screen::VisualEditor { .. } | Screen::Split { .. }
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
            AppTheme::HighContrast => Theme::Dark,
        }
    }

    fn view(&self) -> Element<Message> {
        self.render()
    }
}

fn load_palette() -> (Vec<PaletteBlock>, Vec<(String, Vec<usize>)>) {
    let src = r#"
fn add(a: i32, b: i32) -> i32 { a + b }
fn mul(a: i32, b: i32) -> i32 { a * b }
"#;
    let blocks_raw = parse_blocks(src.to_string(), "rust".into()).unwrap_or_default();
    let blocks: Vec<PaletteBlock> = blocks_raw.into_iter().map(PaletteBlock::new).collect();
    let mut map: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, block) in blocks.iter().enumerate() {
        if block.info.tags.is_empty() {
            map.entry(DEFAULT_CATEGORY.to_string()).or_default().push(i);
        } else {
            for tag in &block.info.tags {
                map.entry(tag.clone()).or_default().push(i);
            }
        }
    }
    (blocks, map.into_iter().collect())
}
