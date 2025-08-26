use crate::sync::{ResolutionOption, SyncConflict};
use iced::widget::{button, column, row, text};
use iced::Element;

/// Render a dialog for resolving a synchronization conflict.
///
/// The dialog shows details about the conflicting metadata and allows
/// choosing which version should win. A `Cancel` button lets the user
/// dismiss the dialog without applying a resolution.
pub fn view(conflict: &SyncConflict) -> Element<Option<ResolutionOption>> {
    column![
        text(format!("Conflict for {}", conflict.id)),
        text(format!("Type: {:?}", conflict.conflict_type)),
        text(format!("Suggested: {:?}", conflict.resolution)),
        row![
            button("Text").on_press(Some(ResolutionOption::Text)),
            button("Visual").on_press(Some(ResolutionOption::Visual)),
            button("Merge").on_press(Some(ResolutionOption::Merge)),
            button("Cancel").on_press(None),
        ]
        .spacing(10)
    ]
    .spacing(10)
    .into()
}
