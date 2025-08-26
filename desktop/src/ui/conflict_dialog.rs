use crate::sync::{ResolutionOption, SyncConflict};
use iced::widget::{button, column, row, text};
use iced::Element;

/// Render a dialog for resolving a synchronization conflict.
///
/// The dialog shows the identifier of the conflicting metadata and
/// allows choosing which version should win.
pub fn view(conflict: &SyncConflict) -> Element<ResolutionOption> {
    column![
        text(format!("Conflict for {}", conflict.id)),
        row![
            button("Text").on_press(ResolutionOption::Text),
            button("Visual").on_press(ResolutionOption::Visual),
            button("Merge").on_press(ResolutionOption::Merge),
        ]
        .spacing(10)
    ]
    .spacing(10)
    .into()
}
