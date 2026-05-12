//! Application state types for the TUI viewer
//!
//! Defines the interaction modes used by the TUI.

/// Interaction mode of the TUI.
///
/// Determines which key bindings are active and what overlays are shown.
pub enum Mode {
    /// Browsing thought list with standard key bindings
    Normal,
    /// Fuzzy entity picker overlay is open
    EntityPicker {
        /// Text input state for the search field
        input: tui_input::Input,
        /// Indices into App::entities matching current input, sorted by fuzzy score
        matches: Vec<usize>,
        /// Currently highlighted match in the picker list
        selected: usize,
    },
    /// Confirmation overlay for deleting a thought
    ConfirmDelete {
        /// Index into App::thoughts of the thought to delete
        thought_index: usize,
    },
    /// Entity description popup is showing
    EntityDetail {
        /// Indices into App::entities for entities referenced in the selected thought
        entity_indices: Vec<usize>,
        /// Scroll position within the description popup
        scroll_offset: usize,
    },
}
