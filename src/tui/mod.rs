//! TUI module for interactive thought viewer
//!
//! Provides an interactive terminal UI for browsing thoughts with entity
//! highlighting, fuzzy entity filtering, sort toggling, and entity description popups.

pub mod input;
pub mod state;
pub mod ui;

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::{Terminal, backend::Backend};

use crate::errors::ThoughtError;
use crate::models::{Entity, Thought};
use crate::services::entity_parser;

use state::{Mode, SortOrder};

/// Root application state for the TUI viewer.
///
/// Holds all loaded data and UI state. Created once at startup with data
/// loaded from the database, then mutated in response to user input.
pub struct App {
    /// All thoughts loaded from database
    pub thoughts: Vec<Thought>,
    /// All entities loaded from database
    pub entities: Vec<Entity>,
    /// Indices into `thoughts` for the current view (after filtering and sorting)
    pub displayed_thoughts: Vec<usize>,
    /// Ratatui list selection/scroll state
    pub list_state: ratatui::widgets::ListState,
    /// Current interaction mode
    pub mode: Mode,
    /// Current sort direction
    pub sort_order: SortOrder,
    /// Entity name currently filtering by (None = show all)
    pub active_filter: Option<String>,
    /// Exit flag
    pub should_quit: bool,
}

impl App {
    /// Create a new App with loaded data.
    ///
    /// Initializes with ascending sort order (oldest first), no active filter,
    /// Normal mode, and all thoughts displayed.
    pub fn new(thoughts: Vec<Thought>, entities: Vec<Entity>) -> Self {
        let mut app = Self {
            thoughts,
            entities,
            displayed_thoughts: Vec::new(),
            list_state: ratatui::widgets::ListState::default(),
            mode: Mode::Normal,
            sort_order: SortOrder::Ascending,
            active_filter: None,
            should_quit: false,
        };
        app.recompute_displayed_thoughts();
        if !app.displayed_thoughts.is_empty() {
            app.list_state.select(Some(0));
        }
        app
    }

    /// Recompute the displayed thoughts based on current filter and sort order.
    pub fn recompute_displayed_thoughts(&mut self) {
        let mut indices: Vec<usize> = if let Some(ref filter_entity) = self.active_filter {
            let filter_lower = filter_entity.to_lowercase();
            self.thoughts
                .iter()
                .enumerate()
                .filter(|(_, thought)| {
                    let entities = entity_parser::extract_entities(&thought.content);
                    entities.iter().any(|e| e.to_lowercase() == filter_lower)
                })
                .map(|(i, _)| i)
                .collect()
        } else {
            (0..self.thoughts.len()).collect()
        };

        match self.sort_order {
            SortOrder::Ascending => {
                indices.sort_by(|&a, &b| self.thoughts[a].created_at.cmp(&self.thoughts[b].created_at));
            }
            SortOrder::Descending => {
                indices.sort_by(|&a, &b| self.thoughts[b].created_at.cmp(&self.thoughts[a].created_at));
            }
        }

        self.displayed_thoughts = indices;

        // Adjust selection
        if self.displayed_thoughts.is_empty() {
            self.list_state.select(None);
        } else if let Some(selected) = self.list_state.selected() {
            if selected >= self.displayed_thoughts.len() {
                self.list_state.select(Some(self.displayed_thoughts.len() - 1));
            }
        } else {
            self.list_state.select(Some(0));
        }
    }

    /// Get the indices of entities referenced in the currently selected thought.
    pub fn selected_thought_entity_indices(&self) -> Vec<usize> {
        let Some(selected) = self.list_state.selected() else {
            return Vec::new();
        };
        let Some(&thought_idx) = self.displayed_thoughts.get(selected) else {
            return Vec::new();
        };
        let thought = &self.thoughts[thought_idx];
        let unique_entities = entity_parser::extract_unique_entities(&thought.content);

        unique_entities
            .iter()
            .filter_map(|name| {
                let name_lower = name.to_lowercase();
                self.entities.iter().position(|e| e.name == name_lower)
            })
            .collect()
    }

    /// Run the TUI event loop on the given terminal.
    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<(), ThoughtError> {
        loop {
            terminal
                .draw(|frame| ui::render(self, frame))
                .map_err(|e| ThoughtError::TuiError(e.to_string()))?;

            let event = event::read().map_err(|e| ThoughtError::TuiError(e.to_string()))?;

            if let Event::Key(key) = event
                && key.kind == KeyEventKind::Press
            {
                input::handle_key_event(self, key);
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_thought(content: &str, days_ago: i64) -> Thought {
        Thought {
            id: Some(days_ago),
            content: content.to_string(),
            created_at: Utc::now() - chrono::Duration::days(days_ago),
        }
    }

    fn make_entity(name: &str, description: Option<&str>) -> Entity {
        Entity {
            id: Some(1),
            name: name.to_lowercase(),
            canonical_name: name.to_string(),
            description: description.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_new_initializes_with_all_thoughts_displayed() {
        let thoughts = vec![
            make_thought("thought 1", 2),
            make_thought("thought 2", 1),
            make_thought("thought 3", 0),
        ];
        let app = App::new(thoughts, vec![]);
        assert_eq!(app.displayed_thoughts.len(), 3);
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_new_with_empty_thoughts() {
        let app = App::new(vec![], vec![]);
        assert!(app.displayed_thoughts.is_empty());
        assert_eq!(app.list_state.selected(), None);
    }

    #[test]
    fn test_new_defaults_to_ascending_sort() {
        let app = App::new(vec![], vec![]);
        assert_eq!(app.sort_order, SortOrder::Ascending);
    }

    #[test]
    fn test_new_defaults_to_normal_mode() {
        let app = App::new(vec![], vec![]);
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn test_new_defaults_to_no_filter() {
        let app = App::new(vec![], vec![]);
        assert!(app.active_filter.is_none());
    }

    #[test]
    fn test_ascending_sort_orders_oldest_first() {
        let thoughts = vec![
            make_thought("newest", 0),
            make_thought("oldest", 5),
            make_thought("middle", 2),
        ];
        let app = App::new(thoughts, vec![]);
        // Ascending = oldest first, so thought at index 1 (5 days ago) should be first
        let first_thought_idx = app.displayed_thoughts[0];
        assert_eq!(app.thoughts[first_thought_idx].content, "oldest");
    }

    #[test]
    fn test_descending_sort_orders_newest_first() {
        let thoughts = vec![
            make_thought("newest", 0),
            make_thought("oldest", 5),
            make_thought("middle", 2),
        ];
        let mut app = App::new(thoughts, vec![]);
        app.sort_order = SortOrder::Descending;
        app.recompute_displayed_thoughts();
        let first_thought_idx = app.displayed_thoughts[0];
        assert_eq!(app.thoughts[first_thought_idx].content, "newest");
    }

    #[test]
    fn test_filter_by_entity() {
        let thoughts = vec![
            make_thought("Meeting with [Sarah]", 2),
            make_thought("No entities here", 1),
            make_thought("Called [Sarah] about [project]", 0),
        ];
        let mut app = App::new(thoughts, vec![]);
        app.active_filter = Some("Sarah".to_string());
        app.recompute_displayed_thoughts();
        assert_eq!(app.displayed_thoughts.len(), 2);
    }

    #[test]
    fn test_filter_clears_to_show_all() {
        let thoughts = vec![
            make_thought("Meeting with [Sarah]", 2),
            make_thought("No entities here", 1),
        ];
        let mut app = App::new(thoughts, vec![]);
        app.active_filter = Some("Sarah".to_string());
        app.recompute_displayed_thoughts();
        assert_eq!(app.displayed_thoughts.len(), 1);

        app.active_filter = None;
        app.recompute_displayed_thoughts();
        assert_eq!(app.displayed_thoughts.len(), 2);
    }

    #[test]
    fn test_selected_thought_entity_indices() {
        let thoughts = vec![make_thought("Meeting with [Sarah] about [Project]", 0)];
        let entities = vec![
            make_entity("Sarah", Some("A person")),
            make_entity("Project", None),
            make_entity("Unrelated", None),
        ];
        let app = App::new(thoughts, entities);
        let indices = app.selected_thought_entity_indices();
        assert_eq!(indices.len(), 2);
        assert!(indices.contains(&0)); // Sarah
        assert!(indices.contains(&1)); // Project
    }

    #[test]
    fn test_selected_thought_entity_indices_no_selection() {
        let app = App::new(vec![], vec![]);
        assert!(app.selected_thought_entity_indices().is_empty());
    }

    #[test]
    fn test_quit_flag() {
        let mut app = App::new(vec![], vec![]);
        assert!(!app.should_quit);
        app.should_quit = true;
        assert!(app.should_quit);
    }
}
