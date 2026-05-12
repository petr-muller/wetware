//! TUI module for interactive thought viewer
//!
//! Provides an interactive terminal UI for browsing thoughts with entity
//! highlighting, fuzzy entity filtering, sort toggling, and entity description popups.

pub mod input;
pub mod state;
pub mod ui;

use std::path::PathBuf;

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::{Terminal, backend::Backend};

use crate::errors::ThoughtError;
use crate::models::{Entity, Thought};
use crate::services::entity_parser;
use crate::storage::connection::get_connection;
use crate::storage::migrations::run_migrations;
use crate::storage::thoughts_repository::ThoughtsRepository;

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
    /// Path to the database for mutation operations
    pub db_path: Option<PathBuf>,
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
            db_path: None,
        };
        app.recompute_displayed_thoughts();
        if !app.displayed_thoughts.is_empty() {
            app.list_state.select(Some(0));
        }
        app
    }

    /// Set the database path for mutation operations (delete).
    pub fn with_db_path(mut self, db_path: PathBuf) -> Self {
        self.db_path = Some(db_path);
        self
    }

    /// Delete the thought currently pending confirmation.
    ///
    /// Must be called while in `ConfirmDelete` mode. Opens a database connection,
    /// deletes the thought, removes it from in-memory state, and returns to Normal mode.
    pub fn delete_selected_thought(&mut self) -> Result<(), ThoughtError> {
        let Mode::ConfirmDelete { thought_index } = self.mode else {
            return Ok(());
        };

        let thought_id = self.thoughts[thought_index]
            .id
            .ok_or(ThoughtError::InvalidInput("Thought has no ID".into()))?;

        let db_path = self
            .db_path
            .as_ref()
            .ok_or(ThoughtError::InvalidInput("No database path configured".into()))?;

        let conn = get_connection(db_path)?;
        run_migrations(&conn)?;
        ThoughtsRepository::delete(&conn, thought_id)?;

        self.thoughts.remove(thought_index);
        self.mode = Mode::Normal;
        self.recompute_displayed_thoughts();

        Ok(())
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

    #[test]
    fn test_with_db_path() {
        let app = App::new(vec![], vec![]).with_db_path(std::path::PathBuf::from("/tmp/test.db"));
        assert_eq!(app.db_path, Some(std::path::PathBuf::from("/tmp/test.db")));
    }

    #[test]
    fn test_new_has_no_db_path() {
        let app = App::new(vec![], vec![]);
        assert!(app.db_path.is_none());
    }

    #[test]
    fn test_delete_selected_thought_not_in_confirm_mode() {
        let mut app = App::new(vec![make_thought("test", 0)], vec![]);
        // Not in ConfirmDelete mode — should be a no-op
        let result = app.delete_selected_thought();
        assert!(result.is_ok());
        assert_eq!(app.thoughts.len(), 1);
    }

    #[test]
    fn test_delete_selected_thought_no_db_path() {
        let mut app = App::new(vec![make_thought("test", 0)], vec![]);
        app.mode = Mode::ConfirmDelete { thought_index: 0 };
        let result = app.delete_selected_thought();
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_selected_thought_success() {
        use crate::storage::connection::get_connection;
        use crate::storage::migrations::run_migrations;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Set up DB with a thought
        let conn = get_connection(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        let thought = Thought::new("to delete".to_string()).unwrap();
        let id = ThoughtsRepository::save(&conn, &thought).unwrap();
        drop(conn);

        // Load thoughts with the saved ID
        let thought_with_id = Thought {
            id: Some(id),
            content: "to delete".to_string(),
            created_at: thought.created_at,
        };

        let mut app = App::new(vec![thought_with_id], vec![]).with_db_path(db_path.clone());
        app.mode = Mode::ConfirmDelete { thought_index: 0 };

        let result = app.delete_selected_thought();
        assert!(result.is_ok());
        assert!(app.thoughts.is_empty());
        assert!(matches!(app.mode, Mode::Normal));
        assert_eq!(app.list_state.selected(), None);

        // Verify in DB
        let conn = get_connection(&db_path).unwrap();
        let remaining = ThoughtsRepository::list_all(&conn).unwrap();
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_delete_selected_thought_adjusts_selection() {
        use crate::storage::connection::get_connection;
        use crate::storage::migrations::run_migrations;

        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection(&db_path).unwrap();
        run_migrations(&conn).unwrap();

        let t1 = Thought::new("first".to_string()).unwrap();
        let t2 = Thought::new("second".to_string()).unwrap();
        let id1 = ThoughtsRepository::save(&conn, &t1).unwrap();
        let id2 = ThoughtsRepository::save(&conn, &t2).unwrap();
        drop(conn);

        let thoughts = vec![
            Thought {
                id: Some(id1),
                content: "first".to_string(),
                created_at: t1.created_at,
            },
            Thought {
                id: Some(id2),
                content: "second".to_string(),
                created_at: t2.created_at,
            },
        ];

        let mut app = App::new(thoughts, vec![]).with_db_path(db_path);
        app.list_state.select(Some(1));
        // Delete the last thought (index 1 in thoughts vec)
        app.mode = Mode::ConfirmDelete { thought_index: 1 };

        app.delete_selected_thought().unwrap();
        assert_eq!(app.thoughts.len(), 1);
        assert_eq!(app.list_state.selected(), Some(0));
    }
}
