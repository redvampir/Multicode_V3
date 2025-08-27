use crate::sync::{ResolutionOption, SyncConflict};
use iced::widget::{button, column, row, text};
use iced::Element;

/// Possible user interactions within the conflict dialog.
#[derive(Debug, Clone)]
pub enum ConflictDialogMessage {
    /// User selected a resolution option or cancelled the dialog.
    Resolve(Option<ResolutionOption>),
    /// Move to the next conflict.
    Next,
    /// Move to the previous conflict.
    Prev,
}

/// Render a dialog for resolving a synchronization conflict.
///
/// The dialog shows details about the conflicting metadata and allows
/// choosing which version should win. A `Cancel` button lets the user
/// dismiss the dialog without applying a resolution.
pub fn view(conflict: &SyncConflict) -> Element<ConflictDialogMessage> {
    column![
        text(format!("Conflict for {}", conflict.id)),
        text(format!("Type: {:?}", conflict.conflict_type)),
        text(format!("Suggested: {:?}", conflict.resolution)),
        row![
            button("Text").on_press(ConflictDialogMessage::Resolve(Some(ResolutionOption::Text))),
            button("Visual").on_press(ConflictDialogMessage::Resolve(Some(
                ResolutionOption::Visual
            ))),
            button("Merge").on_press(ConflictDialogMessage::Resolve(Some(
                ResolutionOption::Merge
            ))),
            button("Cancel").on_press(ConflictDialogMessage::Resolve(None)),
        ]
        .spacing(10),
        row![
            button("Prev").on_press(ConflictDialogMessage::Prev),
            button("Next").on_press(ConflictDialogMessage::Next),
        ]
        .spacing(10)
    ]
    .spacing(10)
    .into()
}
