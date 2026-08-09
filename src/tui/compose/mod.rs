//! Interactive thought composer.
//!
//! A small full-screen editor for entering new thoughts, reached via `wet add -i`
//! (or bare `wet add`). It teaches the entry syntax while you type: the date field
//! accepts relative forms and shows what they resolve to, typing `[` in the content
//! field opens a fuzzy entity "whisperer" over known entity names and aliases, and a
//! live preview colorizes mentions and flags the ones that would create a new entity.
//!
//! The composer stays open after each save so several thoughts can be entered in one
//! sitting.

pub mod input;
pub mod state;
pub mod ui;
pub mod wrap;

use std::collections::HashSet;

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::{Terminal, backend::Backend};
use rusqlite::Connection;

use crate::errors::ThoughtError;
use crate::services::{date_parser, entity_parser, thought_writer};
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::entity_aliases_repository::EntityAliasesRepository;

use state::{Candidate, Field, Slot, Whisperer};

/// Outcome of the last save attempt, shown in the composer's status line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// A thought was saved, with its ID and the number of entities linked
    Saved { id: i64, entities: usize },
    /// Something went wrong; the content field is left untouched
    Error(String),
}

/// Root state for the interactive composer.
///
/// Unlike the browsing [`App`](crate::tui::App), which reopens a connection per
/// mutation, the composer holds one connection open for its lifetime because it
/// writes repeatedly.
pub struct ComposeApp {
    /// Text input for the thought content
    pub content: tui_input::Input,
    /// Text input for the date, parsed by `services::date_parser`
    pub date: tui_input::Input,
    /// Which field currently has focus
    pub focus: Field,
    /// Every completable entity name and alias, rebuilt after each save
    pub candidates: Vec<Candidate>,
    /// Lowercased canonical names and aliases, for the new-entity preview marker
    pub known_lower: HashSet<String>,
    /// The entity completion popup, when open
    pub whisperer: Option<Whisperer>,
    /// Result of the most recent save attempt
    pub status: Option<Status>,
    /// How many thoughts have been saved in this session
    pub saved_count: usize,
    /// Set when the user asks to leave
    pub should_quit: bool,
    /// An `Esc` on a non-empty thought is armed but not yet confirmed
    pub pending_quit: bool,
    conn: Connection,
}

impl ComposeApp {
    /// Build a composer over an open, migrated connection.
    ///
    /// `initial_date` pre-fills the date field, e.g. from `wet add -i --date yesterday`.
    pub fn new(conn: Connection, initial_date: Option<String>) -> Result<Self, ThoughtError> {
        let mut app = Self {
            content: tui_input::Input::default(),
            date: tui_input::Input::new(initial_date.unwrap_or_default()),
            focus: Field::Content,
            candidates: Vec::new(),
            known_lower: HashSet::new(),
            whisperer: None,
            status: None,
            saved_count: 0,
            should_quit: false,
            pending_quit: false,
            conn,
        };
        app.reload_candidates()?;
        Ok(app)
    }

    /// Rebuild the completion candidates and the known-entity set from the database.
    ///
    /// Called at startup and after each save, so entities a save just created become
    /// completable immediately.
    pub fn reload_candidates(&mut self) -> Result<(), ThoughtError> {
        let entities = EntitiesRepository::list_all(&self.conn)?;
        let mut candidates = Vec::new();
        let mut known_lower = HashSet::new();

        for entity in entities {
            known_lower.insert(entity.canonical_name.to_lowercase());
            candidates.push(Candidate {
                label: entity.canonical_name.clone(),
                canonical: entity.canonical_name.clone(),
                is_alias: false,
                description: entity.description.clone(),
            });

            let Some(entity_id) = entity.id else { continue };
            for alias in EntityAliasesRepository::list_for_entity(&self.conn, entity_id)? {
                known_lower.insert(alias.to_lowercase());
                candidates.push(Candidate {
                    label: alias,
                    canonical: entity.canonical_name.clone(),
                    is_alias: true,
                    description: entity.description.clone(),
                });
            }
        }

        self.candidates = candidates;
        self.known_lower = known_lower;
        Ok(())
    }

    /// The labels of all candidates, in order, for fuzzy matching.
    pub fn candidate_labels(&self) -> Vec<&str> {
        self.candidates.iter().map(|c| c.label.as_str()).collect()
    }

    /// Indices of the candidates that can legally be accepted in `slot`.
    ///
    /// A target is written as `](Name)`, and `entity_parser`'s target group cannot
    /// span a nested paren, so an entity whose canonical name contains `(` or `)`
    /// has no valid target form: accepting it would silently degrade the reference
    /// to the traditional one and create an entity named after the display text.
    /// Such entities are still reachable through the display slot, which tolerates
    /// parens, so they are simply not offered here.
    pub fn selectable_candidates(&self, slot: Slot) -> Vec<usize> {
        (0..self.candidates.len())
            .filter(|&i| match slot {
                Slot::Display => true,
                Slot::Target => !self.candidates[i].canonical.contains(['(', ')']),
            })
            .collect()
    }

    /// Resolve the date field to a calendar date.
    ///
    /// An empty field means "now", represented as `Ok(None)` so the saved thought
    /// keeps a full timestamp rather than being pinned to midnight.
    pub fn resolved_date(&self) -> Result<Option<chrono::NaiveDate>, ThoughtError> {
        let raw = self.date.value().trim();
        if raw.is_empty() {
            return Ok(None);
        }
        date_parser::parse_date(raw).map(Some)
    }

    /// Shift the date field by `days`, rewriting it as an absolute `YYYY-MM-DD`.
    ///
    /// Lets the date be adjusted without leaving the content field. An empty field
    /// counts as today. A field that doesn't parse is left alone — it is mid-edit,
    /// and overwriting what is being typed would be worse than doing nothing.
    pub fn nudge_date(&mut self, days: i64) {
        let base = match self.resolved_date() {
            Ok(Some(date)) => date,
            Ok(None) => chrono::Utc::now().date_naive(),
            Err(_) => return,
        };

        let magnitude = chrono::Days::new(days.unsigned_abs());
        let shifted = if days < 0 {
            base.checked_sub_days(magnitude)
        } else {
            base.checked_add_days(magnitude)
        };

        if let Some(shifted) = shifted {
            self.date = tui_input::Input::new(shifted.format("%Y-%m-%d").to_string());
        }
    }

    /// Entity names mentioned in the current content that don't match any known
    /// entity or alias, and would therefore be created on save.
    pub fn new_entity_mentions(&self) -> Vec<String> {
        entity_parser::extract_unique_entities(self.content.value())
            .into_iter()
            .filter(|name| !self.known_lower.contains(&name.to_lowercase()))
            .collect()
    }

    /// Save the current content as a thought.
    ///
    /// On success the content field is cleared while the date field is kept, so
    /// several thoughts can be entered for the same date in a row. On failure the
    /// content is left intact and the reason is recorded in [`Self::status`].
    pub fn save(&mut self) {
        let date = match self.resolved_date() {
            Ok(date) => date,
            Err(e) => {
                self.status = Some(Status::Error(e.to_string()));
                return;
            }
        };

        // Trimmed so the space appended after a completion never reaches storage.
        let content = self.content.value().trim().to_string();
        match thought_writer::create_thought(&mut self.conn, &content, date) {
            Ok(created) => {
                self.content.reset();
                self.whisperer = None;
                self.pending_quit = false;
                self.saved_count += 1;
                // Ambiguous mentions are reported here rather than printed: this
                // process owns the terminal, and stderr would smear the frame.
                self.status = Some(match created.ambiguous.first() {
                    Some(ambiguous) => {
                        Status::Error(format!("Saved thought #{}, but {}", created.id, ambiguous.describe()))
                    }
                    None => Status::Saved {
                        id: created.id,
                        entities: created.entities.len(),
                    },
                });
                // Entities created by this save should be completable right away.
                if let Err(e) = self.reload_candidates() {
                    self.status = Some(Status::Error(e.to_string()));
                }
            }
            Err(e) => self.status = Some(Status::Error(e.to_string())),
        }
    }

    /// Run the composer's event loop until the user quits.
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
pub(crate) mod test_support {
    use super::*;
    use crate::models::entity::Entity;
    use crate::storage::connection::get_memory_connection;
    use crate::storage::migrations::run_migrations;

    /// A composer over an in-memory database seeded with `Alice` (alias `alicia`),
    /// `Bob`, and `Machine Learning`.
    pub fn seeded_app() -> ComposeApp {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let alice = EntitiesRepository::find_or_create(&conn, &Entity::new("Alice".to_string())).unwrap();
        EntityAliasesRepository::add_alias(&conn, alice, "alicia").unwrap();
        EntitiesRepository::find_or_create(&conn, &Entity::new("Bob".to_string())).unwrap();
        EntitiesRepository::find_or_create(&conn, &Entity::new("Machine Learning".to_string())).unwrap();

        ComposeApp::new(conn, None).unwrap()
    }

    /// Type `text` into the composer one key at a time, exercising the real
    /// key-handling path including whisperer recomputation.
    pub fn type_text(app: &mut ComposeApp, text: &str) {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        for ch in text.chars() {
            super::input::handle_key_event(app, KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{seeded_app, type_text};
    use super::*;

    #[test]
    fn test_candidates_include_canonical_names_and_aliases() {
        let app = seeded_app();
        let labels = app.candidate_labels();

        assert!(labels.contains(&"Alice"));
        assert!(labels.contains(&"alicia"));
        assert!(labels.contains(&"Bob"));
        assert!(labels.contains(&"Machine Learning"));
    }

    #[test]
    fn test_alias_candidate_points_at_its_canonical_entity() {
        let app = seeded_app();
        let alias = app.candidates.iter().find(|c| c.label == "alicia").unwrap();

        assert!(alias.is_alias);
        assert_eq!(alias.canonical, "Alice");
    }

    #[test]
    fn test_empty_date_field_resolves_to_none() {
        let app = seeded_app();
        assert_eq!(app.resolved_date().unwrap(), None);
    }

    #[test]
    fn test_date_field_accepts_relative_forms() {
        let mut app = seeded_app();
        app.date = tui_input::Input::new("2026-08-01".to_string());

        assert_eq!(
            app.resolved_date().unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2026, 8, 1).unwrap())
        );
    }

    #[test]
    fn test_invalid_date_field_reports_an_error() {
        let mut app = seeded_app();
        app.date = tui_input::Input::new("nonsense".to_string());

        assert!(app.resolved_date().is_err());
    }

    #[test]
    fn test_new_entity_mentions_flags_only_unknown_names() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new("[Alice] met [Carol] about [alicia]".to_string());

        assert_eq!(app.new_entity_mentions(), vec!["Carol".to_string()]);
    }

    #[test]
    fn test_save_persists_and_clears_content_but_keeps_date() {
        let mut app = seeded_app();
        app.date = tui_input::Input::new("2026-08-01".to_string());
        type_text(&mut app, "talked to Alice");

        app.save();

        assert!(matches!(app.status, Some(Status::Saved { .. })));
        assert_eq!(app.saved_count, 1);
        assert_eq!(app.content.value(), "");
        assert_eq!(app.date.value(), "2026-08-01");
    }

    #[test]
    fn test_save_reports_linked_entity_count() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new("[Alice] and [Bob]".to_string());

        app.save();

        assert_eq!(app.status, Some(Status::Saved { id: 1, entities: 2 }));
    }

    #[test]
    fn test_save_makes_newly_created_entities_completable() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new("met [Carol]".to_string());
        assert!(!app.candidate_labels().contains(&"Carol"));

        app.save();

        assert!(app.candidate_labels().contains(&"Carol"));
        assert!(app.known_lower.contains("carol"));
    }

    #[test]
    fn test_save_with_invalid_date_keeps_content_and_reports_error() {
        let mut app = seeded_app();
        app.date = tui_input::Input::new("nonsense".to_string());
        app.content = tui_input::Input::new("a thought".to_string());

        app.save();

        assert!(matches!(app.status, Some(Status::Error(_))));
        assert_eq!(app.saved_count, 0);
        assert_eq!(app.content.value(), "a thought");
    }

    #[test]
    fn test_save_with_empty_content_keeps_composer_open() {
        let mut app = seeded_app();

        app.save();

        assert!(matches!(app.status, Some(Status::Error(_))));
        assert_eq!(app.saved_count, 0);
        assert!(!app.should_quit);
    }
}
