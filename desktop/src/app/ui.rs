use iced::widget::canvas::Canvas;
use iced::widget::svg::{Handle, Svg};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, text, text_input, MouseArea,
    Space,
};
use iced::{Element, Length};

use crate::app::diff::DiffView;
use crate::app::events::Message;
use crate::app::{command_palette::COMMANDS, format_log, Language, LogLevel, MulticodeApp};
use crate::modal::Modal;
use crate::visual::canvas::{CanvasMessage, VisualCanvas};
use crate::visual::connections::Connection;
use crate::visual::palette::{BlockPalette, PaletteMessage};
use multicode_core::BlockInfo;

const OPEN_ICON: &[u8] = include_bytes!("../../assets/open.svg");
const SAVE_ICON: &[u8] = include_bytes!("../../assets/save.svg");
const FORMAT_ICON: &[u8] = include_bytes!("../../assets/format.svg");
const AUTOCOMPLETE_ICON: &[u8] = include_bytes!("../../assets/autocomplete.svg");

impl MulticodeApp {
    pub fn search_panel_component(&self) -> Element<Message> {
        if !self.show_search_panel {
            return Space::with_height(Length::Shrink).into();
        }
        row![
            text_input("найти", &self.search_term).on_input(Message::SearchTermChanged),
            button("Найти").on_press(Message::Find),
            button("←").on_press(Message::FindPrev),
            button("→").on_press(Message::FindNext),
            text_input("заменить на", &self.replace_term).on_input(Message::ReplaceTermChanged),
            button("Заменить").on_press(Message::Replace),
            button("Заменить все").on_press(Message::ReplaceAll),
            button("×").on_press(Message::ToggleSearchPanel),
        ]
        .spacing(5)
        .into()
    }

    pub fn lint_panel_component(&self) -> Element<Message> {
        if let Some(file) = self.current_file() {
            if file.diagnostics.is_empty() {
                return Space::with_height(Length::Shrink).into();
            }
            let items = file
                .diagnostics
                .iter()
                .map(|d| text(format!("{}: {}", d.line + 1, d.message)).into())
                .collect::<Vec<Element<Message>>>();
            scrollable(column(items))
                .height(Length::Fixed(100.0))
                .into()
        } else {
            Space::with_height(Length::Shrink).into()
        }
    }

    pub fn toolbar(&self) -> Element<Message> {
        if self.settings.show_toolbar {
            let open_icon = Svg::new(Handle::from_memory(OPEN_ICON))
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0));
            let save_icon = Svg::new(Handle::from_memory(SAVE_ICON))
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0));
            let format_icon = Svg::new(Handle::from_memory(FORMAT_ICON))
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0));
            let auto_icon = Svg::new(Handle::from_memory(AUTOCOMPLETE_ICON))
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0));
            let lint_btn = button("Lint").on_press(Message::RunLint);
            let new_btn = button("Новый").on_press(Message::NewFile);
            let palette_btn = button("Командная палитра").on_press(Message::ToggleCommandPalette);
            let settings_btn = button("Настройки").on_press(Message::OpenSettings);
            row![
                new_btn,
                palette_btn,
                settings_btn,
                button(open_icon).on_press(Message::PickFile),
                button(save_icon).on_press(Message::SaveFile),
                button(format_icon).on_press(Message::AutoFormat),
                button(auto_icon).on_press(Message::AutoComplete),
                lint_btn,
                button("Meta").on_press(Message::ToggleMetaPanel)
            ]
            .spacing(5)
            .into()
        } else {
            Space::with_height(Length::Shrink).into()
        }
    }

    pub fn project_search_component(&self) -> Element<Message> {
        if self.project_search_results.is_empty() {
            return Space::with_height(Length::Shrink).into();
        }
        let items = self
            .project_search_results
            .iter()
            .map(|(path, line, snippet)| {
                let label = format!("{}:{}: {}", path.display(), line + 1, snippet);
                button(text(label))
                    .on_press(Message::OpenSearchResult(path.clone(), *line))
                    .into()
            })
            .collect::<Vec<Element<Message>>>();
        scrollable(column(items))
            .height(Length::Fixed(150.0))
            .into()
    }

    pub fn tabs_component(&self) -> Element<Message> {
        let tabs = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let name = f
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let tab = row![
                    text(name),
                    button(text("x")).on_press(Message::CloseFile(i))
                ]
                .spacing(5);
                MouseArea::new(tab)
                    .on_press(Message::StartTabDrag(i))
                    .on_move(|p| Message::UpdateTabDrag(p.x))
                    .on_release(Message::EndTabDrag)
                    .into()
            })
            .collect::<Vec<Element<Message>>>();
        row(tabs).spacing(5).into()
    }

    pub fn diff_component<'a>(&self, diff: &'a DiffView) -> Element<'a, Message> {
        let toggle = checkbox("Игнорировать пробелы", diff.ignore_whitespace)
            .on_toggle(Message::ToggleDiffIgnoreWhitespace);
        column![toggle, diff.view()].spacing(5).into()
    }

    pub fn meta_panel_component(&self) -> Element<Message> {
        if let Some(file) = self.current_file() {
            if let Some(meta) = &file.meta {
                let tags = if meta.tags.is_empty() {
                    "-".into()
                } else {
                    meta.tags.join(", ")
                };
                let links = if meta.links.is_empty() {
                    "-".into()
                } else {
                    meta.links.join(", ")
                };
                let comment = meta
                    .extras
                    .as_ref()
                    .and_then(|e| e.get("comment"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                column![
                    text("Мета"),
                    text(format!("Теги: {}", tags)),
                    text(format!("Связи: {}", links)),
                    text(format!(
                        "Комментарий: {}",
                        if comment.is_empty() {
                            "-".into()
                        } else {
                            comment
                        }
                    )),
                    button("Редактировать").on_press(Message::ShowMetaDialog)
                ]
                .spacing(5)
                .width(Length::Fixed(200.0))
                .into()
            } else {
                column![
                    text("Мета отсутствует"),
                    button("Создать").on_press(Message::ShowMetaDialog)
                ]
                .spacing(5)
                .width(Length::Fixed(200.0))
                .into()
            }
        } else {
            Space::with_width(Length::Shrink).into()
        }
    }

    pub fn visual_editor_component(&self) -> Element<Message> {
        let blocks: &[BlockInfo] = self
            .current_file()
            .map(|f| f.blocks.as_slice())
            .unwrap_or(&[]);
        let connections: &[Connection] = self
            .current_file()
            .map(|f| f.connections.as_slice())
            .unwrap_or(&[]);
        let canvas_widget = Canvas::new(VisualCanvas::new(
            blocks,
            connections,
            self.settings.language,
        ))
        .width(Length::Fill)
        .height(Length::Fill);
        let canvas: Element<CanvasMessage> = canvas_widget.into();
        let canvas = canvas.map(Message::CanvasEvent);
        if self.show_meta_panel {
            row![
                container(canvas).width(Length::FillPortion(3)),
                self.meta_panel_component()
            ]
            .spacing(5)
            .into()
        } else {
            canvas
        }
    }

    pub fn status_bar_component(&self) -> Element<Message> {
        if !self.settings.show_status_bar {
            return Space::with_height(Length::Shrink).into();
        }
        if let Some(file) = self.current_file() {
            let (line, column) = file.editor.cursor_position();
            let path = file.path.to_string_lossy().to_string();
            let dirty = if file.dirty { "*" } else { "" };
            let info = format!("{}:{} | blocks {}", line + 1, column + 1, file.blocks.len());
            container(row![text(path).width(Length::Fill), text(info), text(dirty)].spacing(10))
                .width(Length::Fill)
                .padding(5)
                .into()
        } else {
            let root = self.current_root();
            container(row![text(root).width(Length::Fill)].spacing(10))
                .width(Length::Fill)
                .padding(5)
                .into()
        }
    }

    pub fn terminal_component(&self) -> Element<Message> {
        if !self.show_terminal {
            return Space::with_height(Length::Shrink).into();
        }
        let output = scrollable(column(
            self.log
                .iter()
                .filter(|e| e.level >= self.min_log_level)
                .map(|e| text(format_log(e, self.settings.language)).into())
                .collect::<Vec<Element<Message>>>(),
        ))
        .height(Length::Fixed(150.0));
        let input = text_input("cmd", &self.terminal_cmd)
            .on_input(Message::TerminalCmdChanged)
            .on_submit(Message::RunTerminalCmd(self.terminal_cmd.clone()));
        let clear_btn = button("Очистить").on_press(Message::RunTerminalCmd(":clear".into()));
        let stop_btn = button("Stop").on_press(Message::RunTerminalCmd(":stop".into()));
        let help_btn = button("Справка").on_press(Message::ShowTerminalHelp);
        let lang_pick = pick_list(
            &Language::ALL[..],
            Some(self.settings.language),
            Message::LanguageSelected,
        );
        let save_log_btn = button("Сохранить лог").on_press(Message::SaveLog);
        let level_pick = pick_list(
            &LogLevel::ALL[..],
            Some(self.min_log_level),
            Message::LogLevelSelected,
        );
        column![
            output,
            row![
                input,
                clear_btn,
                stop_btn,
                help_btn,
                lang_pick,
                save_log_btn,
                level_pick
            ]
            .spacing(5)
        ]
        .spacing(5)
        .into()
    }

    pub fn error_modal<'a>(&self, content: Element<'a, Message>) -> Element<'a, Message> {
        if let Some(err) = &self.diff_error {
            let modal_content = container(
                column![
                    text(err.clone()),
                    button("OK").on_press(Message::ClearDiffError)
                ]
                .spacing(10),
            )
            .padding(10);
            Modal::new(content, modal_content)
                .on_blur(Message::ClearDiffError)
                .into()
        } else {
            content
        }
    }

    pub fn command_palette_modal<'a>(&self, content: Element<'a, Message>) -> Element<'a, Message> {
        if !self.show_command_palette {
            return content;
        }
        let query_input = text_input("команда", &self.query).on_input(Message::QueryChanged);
        let items = COMMANDS
            .iter()
            .filter(|c| c.title.to_lowercase().contains(&self.query.to_lowercase()))
            .fold(column![], |col, cmd| {
                col.push(
                    button(text(cmd.title)).on_press(Message::ExecuteCommand(cmd.id.to_string())),
                )
            })
            .spacing(5);
        let modal_content = container(column![query_input, items]).padding(10);
        Modal::new(content, modal_content)
            .on_blur(Message::ToggleCommandPalette)
            .into()
    }

    pub fn goto_line_modal<'a>(&self, content: Element<'a, Message>) -> Element<'a, Message> {
        if !self.show_goto_line_modal {
            return content;
        }
        let placeholder = if self.settings.language == Language::Russian {
            "номер строки"
        } else {
            "line number"
        };
        let input = text_input(placeholder, &self.goto_line_input)
            .on_input(Message::GotoLineInputChanged)
            .on_submit(Message::ConfirmGotoLine);
        let modal_content = container(
            column![
                input,
                row![
                    button("OK").on_press(Message::ConfirmGotoLine),
                    button("Cancel").on_press(Message::CancelGotoLine)
                ]
                .spacing(10)
            ]
            .spacing(10),
        )
        .padding(10);
        Modal::new(content, modal_content)
            .on_blur(Message::CancelGotoLine)
            .into()
    }

    pub fn block_palette_modal<'a>(
        &'a self,
        content: Element<'a, Message>,
    ) -> Element<'a, Message> {
        if !self.show_block_palette {
            return content;
        }
        let pal: Element<_> = BlockPalette::new(
            &self.palette,
            &self.palette_categories,
            &self.settings.block_favorites,
            &self.palette_query,
            self.settings.language,
        )
        .view()
        .map(Message::PaletteEvent);
        Modal::new(content, pal)
            .on_blur(Message::PaletteEvent(PaletteMessage::Close))
            .into()
    }

    pub fn shortcuts_settings_component(&self) -> Element<Message> {
        let items = COMMANDS
            .iter()
            .map(|cmd| {
                let label = if self.shortcut_capture.as_deref() == Some(cmd.id) {
                    String::from("...")
                } else {
                    self.settings
                        .shortcuts
                        .get(cmd.id)
                        .map(|h| h.to_string())
                        .unwrap_or_else(|| String::from("-"))
                };
                row![
                    text(cmd.title),
                    button(text(label)).on_press(Message::StartCaptureShortcut(cmd.id.to_string()))
                ]
                .spacing(10)
                .into()
            })
            .collect::<Vec<Element<Message>>>();
        column(items).spacing(10).into()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CreateTarget, LogLevel, MulticodeApp, Screen, UserSettings, ViewMode};
    use crate::components::file_manager::ContextMenu;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tokio::sync::broadcast;

    #[test]
    fn context_menu_creation() {
        let cm = ContextMenu::new(PathBuf::from("test"));
        assert!(cm.hovered.borrow().is_none());
    }

    fn build_app(screen: Screen) -> MulticodeApp {
        let (sender, _) = broadcast::channel(1);
        let view_mode = match screen {
            Screen::VisualEditor { .. } => ViewMode::Schema,
            Screen::Split { .. } => ViewMode::Split,
            _ => ViewMode::Code,
        };
        MulticodeApp {
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
            favorites: Vec::new(),
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
            settings: UserSettings::default(),
            expanded_dirs: HashSet::new(),
            context_menu: None,
            selected_path: None,
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
            autocomplete: None,
            show_meta_panel: false,
            tab_drag: None,
            palette: Vec::new(),
            palette_categories: Vec::new(),
            show_block_palette: false,
            palette_query: String::new(),
            palette_drag: None,
        }
    }

    #[test]
    fn visual_mode_check() {
        let app = build_app(Screen::VisualEditor {
            root: PathBuf::new(),
        });
        assert!(app.is_visual_mode());

        let app = build_app(Screen::Split {
            root: PathBuf::new(),
        });
        assert!(app.is_visual_mode());

        let app = build_app(Screen::TextEditor {
            root: PathBuf::new(),
        });
        assert!(!app.is_visual_mode());
    }

    #[test]
    fn goto_line_modal_flow() {
        let mut app = build_app(Screen::TextEditor { root: PathBuf::new() });
        assert!(!app.show_goto_line_modal);
        let _ = app.handle_message(crate::app::events::Message::OpenGotoLine);
        assert!(app.show_goto_line_modal);
        let _ = app.handle_message(crate::app::events::Message::GotoLineInputChanged("3".into()));
        let _ = app.handle_message(crate::app::events::Message::ConfirmGotoLine);
        assert!(!app.show_goto_line_modal);
        assert!(app.goto_line_input.is_empty());
    }
}
