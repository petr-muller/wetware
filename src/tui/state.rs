//! Application state types for the TUI viewer
//!
//! Defines the interaction modes and sort order used by the TUI.

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

/// Sort order for the thought list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Oldest thoughts first
    Ascending,
    /// Newest thoughts first
    Descending,
}

impl SortOrder {
    /// Toggle between ascending and descending order.
    pub fn toggle(&mut self) {
        *self = match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
    }

    /// Human-readable label for the current sort order.
    pub fn label(&self) -> &str {
        match self {
            SortOrder::Ascending => "Oldest first",
            SortOrder::Descending => "Newest first",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_order_toggle_ascending_to_descending() {
        let mut order = SortOrder::Ascending;
        order.toggle();
        assert_eq!(order, SortOrder::Descending);
    }

    #[test]
    fn test_sort_order_toggle_descending_to_ascending() {
        let mut order = SortOrder::Descending;
        order.toggle();
        assert_eq!(order, SortOrder::Ascending);
    }

    #[test]
    fn test_sort_order_label_ascending() {
        assert_eq!(SortOrder::Ascending.label(), "Oldest first");
    }

    #[test]
    fn test_sort_order_label_descending() {
        assert_eq!(SortOrder::Descending.label(), "Newest first");
    }

    #[test]
    fn test_sort_order_double_toggle_returns_to_original() {
        let mut order = SortOrder::Ascending;
        order.toggle();
        order.toggle();
        assert_eq!(order, SortOrder::Ascending);
    }
}
