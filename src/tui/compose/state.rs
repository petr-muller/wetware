//! State types for the interactive thought composer.

/// Which of the composer's two text fields currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    /// The date field, accepting the forms in `services::date_parser`
    Date,
    /// The thought content field, where the entity whisperer triggers
    Content,
}

impl Field {
    /// The other field, for Tab/Shift-Tab cycling.
    pub fn toggled(self) -> Self {
        match self {
            Field::Date => Field::Content,
            Field::Content => Field::Date,
        }
    }
}

/// A completion target offered by the entity whisperer: either an entity's
/// canonical name or one of its registered aliases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Text shown in the popup and fuzzy-matched against the query
    pub label: String,
    /// Canonical name of the entity this candidate resolves to
    pub canonical: String,
    /// True when `label` is an alias rather than the canonical name
    pub is_alias: bool,
    /// The entity's description, shown as a hint beside the label
    pub description: Option<String>,
}

impl Candidate {
    /// The text to insert when this candidate is accepted in `slot`.
    pub fn insertion(&self, slot: Slot) -> String {
        match slot {
            // A whole reference. Canonical names insert as `[Name]`; registered
            // aliases insert as `[alias](Name)`, displaying the alias but
            // resolving to the entity.
            Slot::Display if self.is_alias => format!("[{}]({})", self.label, self.canonical),
            Slot::Display => format!("[{}]", self.label),
            // Just the target of a reference whose display text the user already
            // wrote, so only the canonical name is useful here.
            Slot::Target => format!("({})", self.canonical),
        }
    }
}

/// Which part of an entity reference the whisperer is completing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// Inside `[…`, opened by typing `[`. Accepting writes a whole reference.
    Display,
    /// Inside `](…`, opened by typing `(` straight after a `]`. Accepting writes
    /// only the target, keeping the display wording the user already typed.
    Target,
}

impl Slot {
    /// The character that closes this slot's reference and dismisses the popup.
    pub fn closing_char(self) -> char {
        match self {
            Slot::Display => ']',
            Slot::Target => ')',
        }
    }

    /// The character that opens this slot, expected to sit at `Whisperer::open_at`.
    pub fn opening_char(self) -> char {
        match self {
            Slot::Display => '[',
            Slot::Target => '(',
        }
    }
}

/// State of the entity completion popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Whisperer {
    /// Character index in the content field of the bracket that opened this popup.
    ///
    /// Counted in codepoints to match `tui_input::Input::cursor`.
    pub open_at: usize,
    /// Which part of the reference is being completed
    pub slot: Slot,
    /// Indices into `ComposeApp::candidates`, best match first
    pub matches: Vec<usize>,
    /// Currently highlighted entry in `matches`
    pub selected: usize,
}

impl Whisperer {
    /// Move the highlight up one entry, stopping at the top.
    pub fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Move the highlight down one entry, stopping at the bottom.
    pub fn select_next(&mut self) {
        if self.selected + 1 < self.matches.len() {
            self.selected += 1;
        }
    }

    /// Index into `ComposeApp::candidates` of the highlighted entry, if any.
    pub fn selected_candidate(&self) -> Option<usize> {
        self.matches.get(self.selected).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(label: &str, canonical: &str, is_alias: bool) -> Candidate {
        Candidate {
            label: label.to_string(),
            canonical: canonical.to_string(),
            is_alias,
            description: None,
        }
    }

    #[test]
    fn test_field_toggles_between_date_and_content() {
        assert_eq!(Field::Date.toggled(), Field::Content);
        assert_eq!(Field::Content.toggled(), Field::Date);
    }

    #[test]
    fn test_canonical_candidate_inserts_plain_reference() {
        assert_eq!(candidate("Alice", "Alice", false).insertion(Slot::Display), "[Alice]");
    }

    #[test]
    fn test_alias_candidate_inserts_aliased_reference() {
        assert_eq!(
            candidate("alicia", "Alice", true).insertion(Slot::Display),
            "[alicia](Alice)"
        );
    }

    #[test]
    fn test_target_slot_inserts_only_the_canonical_name() {
        assert_eq!(candidate("Alice", "Alice", false).insertion(Slot::Target), "(Alice)");
        // The display wording is already written, so an alias contributes only
        // the entity it points at.
        assert_eq!(candidate("alicia", "Alice", true).insertion(Slot::Target), "(Alice)");
    }

    #[test]
    fn test_slot_delimiters() {
        assert_eq!(Slot::Display.opening_char(), '[');
        assert_eq!(Slot::Display.closing_char(), ']');
        assert_eq!(Slot::Target.opening_char(), '(');
        assert_eq!(Slot::Target.closing_char(), ')');
    }

    #[test]
    fn test_whisperer_selection_is_clamped_at_both_ends() {
        let mut whisperer = Whisperer {
            open_at: 0,
            slot: Slot::Display,
            matches: vec![3, 7],
            selected: 0,
        };

        whisperer.select_previous();
        assert_eq!(whisperer.selected, 0);

        whisperer.select_next();
        assert_eq!(whisperer.selected, 1);
        whisperer.select_next();
        assert_eq!(whisperer.selected, 1);

        whisperer.select_previous();
        assert_eq!(whisperer.selected, 0);
    }

    #[test]
    fn test_selected_candidate_maps_through_matches() {
        let whisperer = Whisperer {
            open_at: 0,
            slot: Slot::Display,
            matches: vec![3, 7],
            selected: 1,
        };
        assert_eq!(whisperer.selected_candidate(), Some(7));
    }

    #[test]
    fn test_selected_candidate_is_none_when_no_matches() {
        let whisperer = Whisperer {
            open_at: 0,
            slot: Slot::Display,
            matches: vec![],
            selected: 0,
        };
        assert_eq!(whisperer.selected_candidate(), None);
    }
}
