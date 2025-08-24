use crate::modal::Modal;
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable,
    svg::{Handle, Svg},
    text, text_input,
    tooltip::{self, Tooltip},
    Space,
};
use iced::{alignment, theme, Element, Length};

use super::events::Message;
use super::{AppTheme, CreateTarget, Language, MulticodeApp, Screen, ViewMode};
use crate::search::hotkeys::HotkeyContext;
use crate::editor::{CodeEditor, EditorTheme, THEME_SET};
use crate::components::file_manager;

const TERMINAL_HELP: &str = include_str!("../../assets/terminal-help.md");
const CREATE_ICON: &[u8] = include_bytes!("../../assets/create.svg");
const RENAME_ICON: &[u8] = include_bytes!("../../assets/rename.svg");
const DELETE_ICON: &[u8] = include_bytes!("../../assets/delete.svg");

impl MulticodeApp {
    pub fn render(&self) -> Element<Message> {
        let (tabs, content): (Option<Element<_>>, Element<_>) = match &self.screen {
            Screen::ProjectPicker => {
                let settings_label = if self.settings.language == Language::Russian {
                    "Настройки"
                } else {
                    "Settings"
                };
                let mut content = column![
                    text("Выберите папку проекта"),
                    button("Выбрать").on_press(Message::PickFolder),
                    button("Выбрать файл").on_press(Message::PickFile),
                    button(settings_label).on_press(Message::OpenSettings),
                ]
                .align_items(alignment::Alignment::Center)
                .spacing(20);

                if !self.settings.last_folders.is_empty() {
                    let open_label = if self.settings.language == Language::Russian {
                        "Открыть"
                    } else {
                        "Open"
                    };
                    content = content.push(text("Недавние проекты:"));
                    for path in &self.settings.last_folders {
                        let path_str = path.to_string_lossy().to_string();
                        content = content.push(
                            row![
                                text(path_str).width(Length::FillPortion(1)),
                                button(open_label).on_press(Message::OpenRecent(path.clone())),
                            ]
                            .spacing(10),
                        );
                    }
                }

                let picker = container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y();
                let content = column![picker, self.status_bar_component()].spacing(10);
                let content = row![self.sidebar(), content].spacing(10).into();
                (None, content)
            }
            Screen::TextEditor { .. } => {
                let sidebar = self.sidebar();

                let tabs = row(self
                    .tabs
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let name = f.path.file_name().unwrap().to_string_lossy().to_string();
                        row![
                            button(text(name)).on_press(Message::SelectFile(f.path.clone())),
                            button(text("x")).on_press(Message::CloseFile(i))
                        ]
                        .spacing(5)
                        .into()
                    })
                    .collect::<Vec<Element<Message>>>())
                .spacing(5);

                let file_menu = self.file_menu();

                let warning: Element<_> = if self.show_create_file_confirm {
                    row![
                        text("Файл уже существует. Перезаписать?"),
                        button("Да").on_press(Message::ConfirmCreateFile),
                        button("Нет").on_press(Message::CancelCreateFile)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                let dirty_warning: Element<_> = if self.pending_action.is_some() {
                    row![
                        text("Есть несохранённые изменения. Продолжить?"),
                        button("Да").on_press(Message::ConfirmDiscard),
                        button("Нет").on_press(Message::CancelDiscard)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                let editor: Element<_> = CodeEditor::new(self).view();

                let search_panel = self.search_panel_component();

                let content = column![
                    search_panel,
                    editor,
                    self.project_search_component(),
                    self.lint_panel_component(),
                    self.terminal_component(),
                ]
                .spacing(10);

                let body = row![sidebar, content].spacing(10);

                let page = column![
                    file_menu,
                    warning,
                    dirty_warning,
                    body,
                    self.status_bar_component()
                ]
                .spacing(10);

                let mut content: Element<_> = page.into();
                if self.show_delete_confirm {
                    let modal_content = container(
                        column![
                            text("Удалить выбранный файл?"),
                            row![
                                button("Да").on_press(Message::DeleteFile),
                                button("Нет").on_press(Message::CancelDeleteFile)
                            ]
                            .spacing(5)
                        ]
                        .spacing(10),
                    );
                    content = Modal::new(content, modal_content)
                        .on_blur(Message::CancelDeleteFile)
                        .into();
                }
                if self.show_terminal_help {
                    let help = container(scrollable(text(TERMINAL_HELP)))
                        .width(Length::Fixed(400.0))
                        .padding(10);
                    content = Modal::new(content, help)
                        .on_blur(Message::ShowTerminalHelp)
                        .into();
                }
                if self.show_meta_dialog {
                    let modal_content = container(
                        column![
                            text_input("Теги", &self.meta_tags).on_input(Message::MetaTagsChanged),
                            text_input("Связи", &self.meta_links)
                                .on_input(Message::MetaLinksChanged),
                            text_input("Комментарий", &self.meta_comment)
                                .on_input(Message::MetaCommentChanged),
                            row![
                                button("Сохранить").on_press(Message::SaveMeta),
                                button("Отмена").on_press(Message::CloseMetaDialog)
                            ]
                            .spacing(5),
                        ]
                        .spacing(5),
                    )
                    .width(Length::Fixed(400.0))
                    .padding(10);
                    content = Modal::new(content, modal_content)
                        .on_blur(Message::CloseMetaDialog)
                        .into();
                }
                (Some(tabs.into()), content)
            }
            Screen::Split { .. } => {
                let sidebar = self.sidebar();

                let tabs = self.tabs_component();

                let file_menu = self.file_menu();

                let warning: Element<_> = if self.show_create_file_confirm {
                    row![
                        text("Файл уже существует. Перезаписать?"),
                        button("Да").on_press(Message::ConfirmCreateFile),
                        button("Нет").on_press(Message::CancelCreateFile)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                let dirty_warning: Element<_> = if self.pending_action.is_some() {
                    row![
                        text("Есть несохранённые изменения. Продолжить?"),
                        button("Да").on_press(Message::ConfirmDiscard),
                        button("Нет").on_press(Message::CancelDiscard)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                let text_editor = CodeEditor::new(self).view();
                let visual_editor = self.visual_editor_component();
                let editor: Element<_> = row![
                    container(text_editor).width(Length::FillPortion(1)),
                    container(visual_editor).width(Length::FillPortion(1))
                ]
                .spacing(10)
                .into();

                let search_panel = self.search_panel_component();

                let content = column![
                    search_panel,
                    editor,
                    self.project_search_component(),
                    self.lint_panel_component(),
                    self.terminal_component(),
                ]
                .spacing(10);

                let body = row![sidebar, content].spacing(10);

                let page = column![
                    file_menu,
                    warning,
                    dirty_warning,
                    body,
                    self.status_bar_component()
                ]
                .spacing(10);

                let mut content: Element<_> = page.into();
                if self.show_delete_confirm {
                    let modal_content = container(
                        column![
                            text("Удалить выбранный файл?"),
                            row![
                                button("Да").on_press(Message::DeleteFile),
                                button("Нет").on_press(Message::CancelDeleteFile)
                            ]
                            .spacing(5)
                        ]
                        .spacing(10),
                    );
                    content = Modal::new(content, modal_content)
                        .on_blur(Message::CancelDeleteFile)
                        .into();
                }
                if self.show_terminal_help {
                    let help = container(scrollable(text(TERMINAL_HELP)))
                        .width(Length::Fixed(400.0))
                        .padding(10);
                    content = Modal::new(content, help)
                        .on_blur(Message::ShowTerminalHelp)
                        .into();
                }
                if self.show_meta_dialog {
                    let modal_content = container(
                        column![
                            text_input("Теги", &self.meta_tags).on_input(Message::MetaTagsChanged),
                            text_input("Связи", &self.meta_links)
                                .on_input(Message::MetaLinksChanged),
                            text_input("Комментарий", &self.meta_comment)
                                .on_input(Message::MetaCommentChanged),
                            row![
                                button("Сохранить").on_press(Message::SaveMeta),
                                button("Отмена").on_press(Message::CloseMetaDialog)
                            ]
                            .spacing(5),
                        ]
                        .spacing(5),
                    )
                    .width(Length::Fixed(400.0))
                    .padding(10);
                    content = Modal::new(content, modal_content)
                        .on_blur(Message::CloseMetaDialog)
                        .into();
                }

                (Some(tabs.into()), content)
            }
            Screen::VisualEditor { .. } => {
                let sidebar = self.sidebar();

                let tabs = self.tabs_component();

                let file_menu = self.file_menu();

                let warning: Element<_> = if self.show_create_file_confirm {
                    row![
                        text("Файл уже существует. Перезаписать?"),
                        button("Да").on_press(Message::ConfirmCreateFile),
                        button("Нет").on_press(Message::CancelCreateFile)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                let dirty_warning: Element<_> = if self.pending_action.is_some() {
                    row![
                        text("Есть несохранённые изменения. Продолжить?"),
                        button("Да").on_press(Message::ConfirmDiscard),
                        button("Нет").on_press(Message::CancelDiscard)
                    ]
                    .spacing(5)
                    .into()
                } else {
                    Space::with_width(Length::Shrink).into()
                };

                let editor: Element<_> = self.visual_editor_component();

                let content = column![
                    text_input("поиск", &self.query).on_input(Message::QueryChanged),
                    editor,
                    self.project_search_component(),
                    self.terminal_component(),
                ]
                .spacing(10);

                let body = row![sidebar, content].spacing(10);

                let page = column![
                    file_menu,
                    warning,
                    dirty_warning,
                    body,
                    self.status_bar_component()
                ]
                .spacing(10);

                let mut content: Element<_> = page.into();
                if self.show_delete_confirm {
                    let modal_content = container(
                        column![
                            text("Удалить выбранный файл?"),
                            row![
                                button("Да").on_press(Message::DeleteFile),
                                button("Нет").on_press(Message::CancelDeleteFile)
                            ]
                            .spacing(5)
                        ]
                        .spacing(10),
                    );
                    content = Modal::new(content, modal_content)
                        .on_blur(Message::CancelDeleteFile)
                        .into();
                }
                if self.show_terminal_help {
                    let help = container(scrollable(text(TERMINAL_HELP)))
                        .width(Length::Fixed(400.0))
                        .padding(10);
                    content = Modal::new(content, help)
                        .on_blur(Message::ShowTerminalHelp)
                        .into();
                }
                if self.show_meta_dialog {
                    let modal_content = container(
                        column![
                            text_input("Теги", &self.meta_tags).on_input(Message::MetaTagsChanged),
                            text_input("Связи", &self.meta_links)
                                .on_input(Message::MetaLinksChanged),
                            text_input("Комментарий", &self.meta_comment)
                                .on_input(Message::MetaCommentChanged),
                            row![
                                button("Сохранить").on_press(Message::SaveMeta),
                                button("Отмена").on_press(Message::CloseMetaDialog)
                            ]
                            .spacing(5)
                        ]
                        .spacing(5),
                    )
                    .width(Length::Fixed(400.0))
                    .padding(10);
                    content = Modal::new(content, modal_content)
                        .on_blur(Message::CloseMetaDialog)
                        .into();
                }
                (Some(tabs), content)
            }
            Screen::Settings => {
                let hotkeys = &self.settings.hotkeys;
                let create_label = if self.shortcut_capture.as_deref() == Some("create_file") {
                    String::from("...")
                } else {
                    hotkeys
                        .binding(HotkeyContext::Global, "create_file")
                        .map(|h| h.to_string())
                        .unwrap_or_else(|| "-".into())
                };
                let save_label = if self.shortcut_capture.as_deref() == Some("save_file") {
                    String::from("...")
                } else {
                    hotkeys
                        .binding(HotkeyContext::Global, "save_file")
                        .map(|h| h.to_string())
                        .unwrap_or_else(|| "-".into())
                };
                let rename_label = if self.shortcut_capture.as_deref() == Some("rename_file") {
                    String::from("...")
                } else {
                    hotkeys
                        .binding(HotkeyContext::Global, "rename_file")
                        .map(|h| h.to_string())
                        .unwrap_or_else(|| "-".into())
                };
                let delete_label = if self.shortcut_capture.as_deref() == Some("delete_file") {
                    String::from("...")
                } else {
                    hotkeys
                        .binding(HotkeyContext::Global, "delete_file")
                        .map(|h| h.to_string())
                        .unwrap_or_else(|| "-".into())
                };
                let next_diff_label = if self.shortcut_capture.as_deref() == Some("next_diff") {
                    String::from("...")
                } else {
                    hotkeys
                        .binding(HotkeyContext::Diff, "next_diff")
                        .map(|h| h.to_string())
                        .unwrap_or_else(|| "-".into())
                };
                let prev_diff_label = if self.shortcut_capture.as_deref() == Some("prev_diff") {
                    String::from("...")
                } else {
                    hotkeys
                        .binding(HotkeyContext::Diff, "prev_diff")
                        .map(|h| h.to_string())
                        .unwrap_or_else(|| "-".into())
                };
                let syntect_themes: Vec<String> = THEME_SET.themes.keys().cloned().collect();
                let warning: Element<_> = if let Some(w) = &self.settings_warning {
                    text(w.clone()).into()
                } else {
                    Space::with_height(Length::Shrink).into()
                };
                let content = column![
                    text("Settings / Настройки"),
                    row![
                        text("Тема"),
                        pick_list(
                            &AppTheme::ALL[..],
                            Some(self.settings.theme),
                            Message::ThemeSelected
                        )
                    ]
                    .spacing(10),
                    row![
                        text("Тема подсветки"),
                        pick_list(
                            syntect_themes.clone(),
                            Some(self.settings.syntect_theme.clone()),
                            Message::SyntectThemeSelected
                        )
                    ]
                    .spacing(10),
                    row![
                        text("Язык"),
                        pick_list(
                            &Language::ALL[..],
                            Some(self.settings.language),
                            Message::LanguageSelected
                        )
                    ]
                    .spacing(10),
                    row![
                        text("Номера строк"),
                        checkbox("", self.settings.show_line_numbers)
                            .on_toggle(Message::ToggleLineNumbers)
                    ]
                    .spacing(10),
                    row![
                        text("Статус-бар"),
                        checkbox("", self.settings.show_status_bar)
                            .on_toggle(Message::ToggleStatusBar),
                    ]
                    .spacing(10),
                    row![
                        text("Панель инструментов"),
                        checkbox("", self.settings.show_toolbar).on_toggle(Message::ToggleToolbar),
                    ]
                    .spacing(10),
                    row![
                        text("Предпросмотр Markdown"),
                        checkbox("", self.settings.show_markdown_preview)
                            .on_toggle(Message::ToggleMarkdownPreview),
                    ]
                    .spacing(10),
                    row![
                        text("Размер шрифта"),
                        text_input(
                            "",
                            &self.settings.editor.font_size.to_string()
                        )
                        .on_input(Message::FontSizeChanged)
                        .width(Length::Fixed(50.0)),
                    ]
                    .spacing(10),
                    row![
                        text("Ширина табуляции"),
                        text_input(
                            "",
                            &self.settings.editor.tab_width.to_string()
                        )
                        .on_input(Message::TabWidthChanged)
                        .width(Length::Fixed(50.0)),
                    ]
                    .spacing(10),
                    row![
                        text("Автоотступы"),
                        checkbox("", self.settings.editor.auto_indent)
                            .on_toggle(Message::ToggleAutoIndent),
                    ]
                    .spacing(10),
                    row![
                        text("Перенос строк"),
                        checkbox("", self.settings.editor.line_wrapping)
                            .on_toggle(Message::ToggleLineWrapping),
                    ]
                    .spacing(10),
                    row![
                        text("Подсветка строки"),
                        checkbox("", self.settings.editor.highlight_current_line)
                            .on_toggle(Message::ToggleHighlightCurrentLine),
                    ]
                    .spacing(10),
                    row![
                        text("Тема редактора"),
                        pick_list(
                            &EditorTheme::ALL[..],
                            Some(self.settings.editor.theme),
                            Message::EditorThemeSelected
                        ),
                    ]
                    .spacing(10),
                    row![
                        text("Создать файл"),
                        button(text(create_label))
                            .on_press(Message::StartCaptureShortcut("create_file".into()))
                    ]
                    .spacing(10),
                    row![
                        text("Сохранить файл"),
                        button(text(save_label))
                            .on_press(Message::StartCaptureShortcut("save_file".into()))
                    ]
                    .spacing(10),
                    row![
                        text("Переименовать файл"),
                        button(text(rename_label))
                            .on_press(Message::StartCaptureShortcut("rename_file".into()))
                    ]
                    .spacing(10),
                    row![
                        text("Удалить файл"),
                        button(text(delete_label))
                            .on_press(Message::StartCaptureShortcut("delete_file".into()))
                    ]
                    .spacing(10),
                    row![
                        text("Следующее отличие"),
                        button(text(next_diff_label))
                            .on_press(Message::StartCaptureShortcut("next_diff".into()))
                    ]
                    .spacing(10),
                    row![
                        text("Предыдущее отличие"),
                        button(text(prev_diff_label))
                            .on_press(Message::StartCaptureShortcut("prev_diff".into()))
                    ]
                    .spacing(10),
                    self.shortcuts_settings_component(),
                    warning,
                    row![
                        button("Save / Сохранить").on_press(Message::SaveSettings),
                        button("Back / Назад").on_press(Message::CloseSettings)
                    ]
                    .spacing(10)
                ]
                .align_items(alignment::Alignment::Center)
                .spacing(20);

                let settings_page = container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y();
                let content = column![settings_page, self.status_bar_component()].spacing(10);
                let content = row![self.sidebar(), content].spacing(10).into();
                (None, content)
            }
            Screen::Diff(diff) => {
                let diff_view = container(self.diff_component(diff))
                    .width(Length::Fill)
                    .height(Length::Fill);
                let content = column![diff_view, self.status_bar_component()].spacing(10);
                let content = row![self.sidebar(), content].spacing(10).into();
                (None, content)
            }
        };
        let mut page = column![self.main_menu()];
        if let Some(tabs) = tabs {
            page = page.push(tabs);
        }
        let page: Element<_> = page
            .push(self.mode_bar())
            .push(self.toolbar())
            .push(content)
            .spacing(10)
            .into();
        let content = self.loading_overlay(page);
        let content = self.command_palette_modal(content);
        let content = self.goto_line_modal(content);
        let content = self.block_palette_modal(content);
        self.error_modal(content)
    }
}

impl MulticodeApp {
    fn loading_overlay<'a>(&self, content: Element<'a, Message>) -> Element<'a, Message> {
        if self.loading {
            let overlay = container(text("Loading…"))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y();
            Modal::new(content, overlay).into()
        } else {
            content
        }
    }

    fn main_menu(&self) -> Element<Message> {
        let settings_label = if self.settings.language == Language::Russian {
            "Настройки"
        } else {
            "Settings"
        };
        let open_other_label = if self.settings.language == Language::Russian {
            "Открыть другой проект"
        } else {
            "Open another project"
        };
        row![
            button("Разбор").on_press(Message::RunParse),
            button("Поиск").on_press(Message::ProjectSearch(self.query.clone())),
            button("Журнал Git").on_press(Message::RunGitLog),
            button("Экспорт").on_press(Message::RunExport),
            button(open_other_label).on_press(Message::OpenProjectPicker),
            button("Терминал").on_press(Message::ToggleTerminal),
            button(settings_label).on_press(Message::OpenSettings),
        ]
        .spacing(10)
        .into()
    }

    fn mode_bar(&self) -> Element<Message> {
        let code_btn: Element<_> = if self.view_mode == ViewMode::Code {
            button("Код").style(theme::Button::Primary).into()
        } else {
            button("Код")
                .on_press(Message::SwitchViewMode(ViewMode::Code))
                .into()
        };
        let schema_btn: Element<_> = if self.view_mode == ViewMode::Schema {
            button("Схема").style(theme::Button::Primary).into()
        } else {
            button("Схема")
                .on_press(Message::SwitchViewMode(ViewMode::Schema))
                .into()
        };
        let split_btn: Element<_> = if self.view_mode == ViewMode::Split {
            button("Split").style(theme::Button::Primary).into()
        } else {
            button("Split")
                .on_press(Message::SwitchViewMode(ViewMode::Split))
                .into()
        };

        let save_label = if self.is_dirty() {
            "Сохранить*"
        } else {
            "Сохранить"
        };
        let save_btn: Element<_> = if self.active_tab.is_some() {
            button(save_label).on_press(Message::SaveFile).into()
        } else {
            button(save_label).into()
        };

        if matches!(&self.screen, Screen::TextEditor { .. } | Screen::Split { .. }) {
            let autocomplete_btn: Element<_> = if self.active_tab.is_some() {
                button("Автодополнить")
                    .on_press(Message::AutoComplete)
                    .into()
            } else {
                button("Автодополнить").into()
            };
            let format_btn: Element<_> = if self.active_tab.is_some() {
                button("Форматировать").on_press(Message::AutoFormat).into()
            } else {
                button("Форматировать").into()
            };
            row![
                code_btn,
                schema_btn,
                split_btn,
                save_btn,
                autocomplete_btn,
                format_btn
            ]
            .spacing(5)
            .into()
        } else {
            row![code_btn, schema_btn, split_btn, save_btn]
                .spacing(5)
                .into()
        }
    }

    fn file_menu(&self) -> Element<Message> {
        let create_icon = Svg::new(Handle::from_memory(CREATE_ICON))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0));
        let rename_icon = Svg::new(Handle::from_memory(RENAME_ICON))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0));
        let delete_icon = Svg::new(Handle::from_memory(DELETE_ICON))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0));

        let create_select = pick_list(
            &CreateTarget::ALL[..],
            Some(self.create_target),
            Message::CreateTargetChanged,
        );
        let (placeholder, value, on_input_msg, create_msg): (
            &str,
            &String,
            fn(String) -> Message,
            Message,
        ) = match self.create_target {
            CreateTarget::File => (
                "новый файл",
                &self.new_file_name,
                Message::NewFileNameChanged as fn(String) -> Message,
                Message::CreateFile,
            ),
            CreateTarget::Directory => (
                "новый каталог",
                &self.new_directory_name,
                Message::NewDirectoryNameChanged as fn(String) -> Message,
                Message::CreateDirectory,
            ),
        };
        let create_input = text_input(placeholder, value).on_input(on_input_msg);
        let create_button: Element<_> = Tooltip::new(
            button(create_icon).on_press(create_msg),
            "Создать",
            tooltip::Position::Top,
        )
        .into();

        let rename_btn: Element<_> = {
            let btn = button(rename_icon);
            let btn = if self.active_tab.is_some() {
                btn.on_press(Message::RenameFile)
            } else {
                btn
            };
            Tooltip::new(btn, "Переименовать", tooltip::Position::Top).into()
        };

        let delete_btn: Element<_> = {
            let btn = button(delete_icon);
            let btn = if self.active_tab.is_some() {
                btn.on_press(Message::RequestDeleteFile)
            } else {
                btn
            };
            Tooltip::new(btn, "Удалить", tooltip::Position::Top).into()
        };

        row![
            create_select,
            create_input,
            create_button,
            text_input("новое имя", &self.rename_file_name)
                .on_input(Message::RenameFileNameChanged),
            rename_btn,
            delete_btn,
        ]
        .spacing(5)
        .into()
    }

    fn sidebar(&self) -> Element<Message> {
        let search = text_input("поиск", &self.search_query).on_input(Message::SearchChanged);
        let tree = file_manager::file_tree(
            &self.files,
            &self.expanded_dirs,
            &self.search_query,
            &self.favorites,
            &self.selected_path,
        );
        column![search, container(tree).width(200)]
            .spacing(5)
            .into()
    }
}
