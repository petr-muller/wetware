//! Key event handling for the TUI viewer
//!
//! Maps keyboard events to state mutations based on the current interaction mode.

use nucleo_matcher::{Matcher, pattern::Pattern};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use tui_input::backend::crossterm::EventHandler;

use super::App;
use super::state::Mode;

/// Handle a key event and update app state.
///
/// Dispatches to the appropriate mode-specific handler based on the current mode.
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match &app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::EntityPicker { .. } => handle_entity_picker_mode(app, key),
        Mode::EntityDetail { .. } => handle_entity_detail_mode(app, key),
    }
}

/// Handle key events in Normal mode.
fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Esc => {
            if app.active_filter.is_some() {
                app.active_filter = None;
                app.recompute_displayed_thoughts();
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Up => {
            if let Some(selected) = app.list_state.selected()
                && selected > 0
            {
                app.list_state.select(Some(selected - 1));
            }
        }
        KeyCode::Down => {
            if let Some(selected) = app.list_state.selected()
                && selected + 1 < app.displayed_thoughts.len()
            {
                app.list_state.select(Some(selected + 1));
            }
        }
        KeyCode::PageUp => {
            if let Some(selected) = app.list_state.selected() {
                let new_selected = selected.saturating_sub(10);
                app.list_state.select(Some(new_selected));
            }
        }
        KeyCode::PageDown => {
            if let Some(selected) = app.list_state.selected() {
                let max = app.displayed_thoughts.len().saturating_sub(1);
                let new_selected = (selected + 10).min(max);
                app.list_state.select(Some(new_selected));
            }
        }
        KeyCode::Home => {
            if !app.displayed_thoughts.is_empty() {
                app.list_state.select(Some(0));
            }
        }
        KeyCode::End => {
            if !app.displayed_thoughts.is_empty() {
                app.list_state.select(Some(app.displayed_thoughts.len() - 1));
            }
        }
        KeyCode::Char('s') => {
            app.sort_order.toggle();
            app.recompute_displayed_thoughts();
        }
        KeyCode::Char('/') => {
            // Open entity picker
            let all_indices: Vec<usize> = (0..app.entities.len()).collect();
            app.mode = Mode::EntityPicker {
                input: tui_input::Input::default(),
                matches: all_indices,
                selected: 0,
            };
        }
        KeyCode::Enter | KeyCode::Char('d') => {
            // Open entity detail for selected thought
            let entity_indices = app.selected_thought_entity_indices();
            if !entity_indices.is_empty() {
                app.mode = Mode::EntityDetail {
                    entity_indices,
                    scroll_offset: 0,
                };
            }
        }
        _ => {}
    }
}

/// Handle key events in EntityPicker mode.
fn handle_entity_picker_mode(app: &mut App, key: KeyEvent) {
    // We need to extract the mutable fields from Mode
    let Mode::EntityPicker {
        ref mut input,
        ref mut matches,
        ref mut selected,
    } = app.mode
    else {
        return;
    };

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Enter => {
            if let Some(&entity_idx) = matches.get(*selected) {
                let entity_name = app.entities[entity_idx].canonical_name.clone();
                app.active_filter = Some(entity_name);
                app.mode = Mode::Normal;
                app.recompute_displayed_thoughts();
            } else {
                app.mode = Mode::Normal;
            }
        }
        KeyCode::Up => {
            if *selected > 0 {
                *selected -= 1;
            }
        }
        KeyCode::Down => {
            if *selected + 1 < matches.len() {
                *selected += 1;
            }
        }
        _ => {
            // Forward to text input
            input.handle_event(&ratatui::crossterm::event::Event::Key(key));

            // Recompute fuzzy matches
            let query = input.value();
            if query.is_empty() {
                *matches = (0..app.entities.len()).collect();
            } else {
                let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
                let pattern = Pattern::new(
                    query,
                    nucleo_matcher::pattern::CaseMatching::Ignore,
                    nucleo_matcher::pattern::Normalization::Smart,
                    nucleo_matcher::pattern::AtomKind::Fuzzy,
                );

                let mut scored: Vec<(usize, u32)> = app
                    .entities
                    .iter()
                    .enumerate()
                    .filter_map(|(i, entity)| {
                        let mut buf = Vec::new();
                        let haystack = nucleo_matcher::Utf32Str::new(&entity.canonical_name, &mut buf);
                        pattern.score(haystack, &mut matcher).map(|score| (i, score))
                    })
                    .collect();

                scored.sort_by(|a, b| b.1.cmp(&a.1));
                *matches = scored.into_iter().map(|(i, _)| i).collect();
            }
            *selected = 0;
        }
    }
}

/// Handle key events in EntityDetail mode.
fn handle_entity_detail_mode(app: &mut App, key: KeyEvent) {
    let Mode::EntityDetail {
        ref mut scroll_offset, ..
    } = app.mode
    else {
        return;
    };

    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Up => {
            *scroll_offset = scroll_offset.saturating_sub(1);
        }
        KeyCode::Down => {
            *scroll_offset += 1;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Entity, Thought};
    use chrono::Utc;

    fn make_thought(content: &str, days_ago: i64) -> Thought {
        Thought {
            id: Some(days_ago),
            content: content.to_string(),
            created_at: Utc::now() - chrono::Duration::days(days_ago),
        }
    }

    fn make_entity(name: &str) -> Entity {
        Entity {
            id: Some(1),
            name: name.to_lowercase(),
            canonical_name: name.to_string(),
            description: None,
        }
    }

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, ratatui::crossterm::event::KeyModifiers::NONE)
    }

    #[test]
    fn test_normal_mode_q_quits() {
        let mut app = App::new(vec![], vec![]);
        handle_key_event(&mut app, key_event(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn test_normal_mode_esc_quits_without_filter() {
        let mut app = App::new(vec![], vec![]);
        handle_key_event(&mut app, key_event(KeyCode::Esc));
        assert!(app.should_quit);
    }

    #[test]
    fn test_normal_mode_esc_clears_filter() {
        let thoughts = vec![make_thought("[Sarah] hello", 0)];
        let mut app = App::new(thoughts, vec![]);
        app.active_filter = Some("Sarah".to_string());
        app.recompute_displayed_thoughts();

        handle_key_event(&mut app, key_event(KeyCode::Esc));
        assert!(!app.should_quit);
        assert!(app.active_filter.is_none());
    }

    #[test]
    fn test_normal_mode_up_decreases_selection() {
        let thoughts = vec![make_thought("a", 2), make_thought("b", 1), make_thought("c", 0)];
        let mut app = App::new(thoughts, vec![]);
        app.list_state.select(Some(2));

        handle_key_event(&mut app, key_event(KeyCode::Up));
        assert_eq!(app.list_state.selected(), Some(1));
    }

    #[test]
    fn test_normal_mode_up_stops_at_zero() {
        let thoughts = vec![make_thought("a", 0)];
        let mut app = App::new(thoughts, vec![]);
        app.list_state.select(Some(0));

        handle_key_event(&mut app, key_event(KeyCode::Up));
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_normal_mode_down_increases_selection() {
        let thoughts = vec![make_thought("a", 2), make_thought("b", 1)];
        let mut app = App::new(thoughts, vec![]);
        app.list_state.select(Some(0));

        handle_key_event(&mut app, key_event(KeyCode::Down));
        assert_eq!(app.list_state.selected(), Some(1));
    }

    #[test]
    fn test_normal_mode_down_stops_at_end() {
        let thoughts = vec![make_thought("a", 0)];
        let mut app = App::new(thoughts, vec![]);
        app.list_state.select(Some(0));

        handle_key_event(&mut app, key_event(KeyCode::Down));
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_normal_mode_home_selects_first() {
        let thoughts = vec![make_thought("a", 2), make_thought("b", 1), make_thought("c", 0)];
        let mut app = App::new(thoughts, vec![]);
        app.list_state.select(Some(2));

        handle_key_event(&mut app, key_event(KeyCode::Home));
        assert_eq!(app.list_state.selected(), Some(0));
    }

    #[test]
    fn test_normal_mode_end_selects_last() {
        let thoughts = vec![make_thought("a", 2), make_thought("b", 1), make_thought("c", 0)];
        let mut app = App::new(thoughts, vec![]);
        app.list_state.select(Some(0));

        handle_key_event(&mut app, key_event(KeyCode::End));
        assert_eq!(app.list_state.selected(), Some(2));
    }

    #[test]
    fn test_normal_mode_s_toggles_sort() {
        let thoughts = vec![make_thought("newest", 0), make_thought("oldest", 5)];
        let mut app = App::new(thoughts, vec![]);
        assert_eq!(app.sort_order, crate::tui::state::SortOrder::Ascending);

        handle_key_event(&mut app, key_event(KeyCode::Char('s')));
        assert_eq!(app.sort_order, crate::tui::state::SortOrder::Descending);

        // Verify order changed
        let first_idx = app.displayed_thoughts[0];
        assert_eq!(app.thoughts[first_idx].content, "newest");
    }

    #[test]
    fn test_normal_mode_slash_opens_entity_picker() {
        let entities = vec![make_entity("Sarah"), make_entity("Project")];
        let mut app = App::new(vec![], entities);

        handle_key_event(&mut app, key_event(KeyCode::Char('/')));
        assert!(matches!(app.mode, Mode::EntityPicker { .. }));

        if let Mode::EntityPicker { matches, .. } = &app.mode {
            assert_eq!(matches.len(), 2); // All entities shown initially
        }
    }

    #[test]
    fn test_normal_mode_enter_opens_entity_detail() {
        let thoughts = vec![make_thought("Meeting with [Sarah]", 0)];
        let entities = vec![make_entity("Sarah")];
        let mut app = App::new(thoughts, entities);

        handle_key_event(&mut app, key_event(KeyCode::Enter));
        assert!(matches!(app.mode, Mode::EntityDetail { .. }));

        if let Mode::EntityDetail { entity_indices, .. } = &app.mode {
            assert_eq!(entity_indices.len(), 1);
        }
    }

    #[test]
    fn test_normal_mode_enter_no_entities_stays_normal() {
        let thoughts = vec![make_thought("No entities here", 0)];
        let mut app = App::new(thoughts, vec![]);

        handle_key_event(&mut app, key_event(KeyCode::Enter));
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn test_entity_picker_esc_returns_to_normal() {
        let entities = vec![make_entity("Sarah")];
        let mut app = App::new(vec![], entities);
        app.mode = Mode::EntityPicker {
            input: tui_input::Input::default(),
            matches: vec![0],
            selected: 0,
        };

        handle_key_event(&mut app, key_event(KeyCode::Esc));
        assert!(matches!(app.mode, Mode::Normal));
        assert!(app.active_filter.is_none()); // No filter applied
    }

    #[test]
    fn test_entity_picker_enter_applies_filter() {
        let thoughts = vec![make_thought("[Sarah] hello", 0), make_thought("world", 1)];
        let entities = vec![make_entity("Sarah")];
        let mut app = App::new(thoughts, entities);
        app.mode = Mode::EntityPicker {
            input: tui_input::Input::default(),
            matches: vec![0],
            selected: 0,
        };

        handle_key_event(&mut app, key_event(KeyCode::Enter));
        assert!(matches!(app.mode, Mode::Normal));
        assert_eq!(app.active_filter.as_deref(), Some("Sarah"));
        assert_eq!(app.displayed_thoughts.len(), 1); // Only the thought with Sarah
    }

    #[test]
    fn test_entity_picker_up_down_navigation() {
        let entities = vec![make_entity("Alpha"), make_entity("Beta"), make_entity("Gamma")];
        let mut app = App::new(vec![], entities);
        app.mode = Mode::EntityPicker {
            input: tui_input::Input::default(),
            matches: vec![0, 1, 2],
            selected: 0,
        };

        handle_key_event(&mut app, key_event(KeyCode::Down));
        if let Mode::EntityPicker { selected, .. } = &app.mode {
            assert_eq!(*selected, 1);
        }

        handle_key_event(&mut app, key_event(KeyCode::Up));
        if let Mode::EntityPicker { selected, .. } = &app.mode {
            assert_eq!(*selected, 0);
        }
    }

    #[test]
    fn test_entity_detail_esc_returns_to_normal() {
        let mut app = App::new(vec![], vec![]);
        app.mode = Mode::EntityDetail {
            entity_indices: vec![0],
            scroll_offset: 0,
        };

        handle_key_event(&mut app, key_event(KeyCode::Esc));
        assert!(matches!(app.mode, Mode::Normal));
    }

    #[test]
    fn test_entity_detail_scroll() {
        let mut app = App::new(vec![], vec![]);
        app.mode = Mode::EntityDetail {
            entity_indices: vec![0],
            scroll_offset: 0,
        };

        handle_key_event(&mut app, key_event(KeyCode::Down));
        if let Mode::EntityDetail { scroll_offset, .. } = &app.mode {
            assert_eq!(*scroll_offset, 1);
        }

        handle_key_event(&mut app, key_event(KeyCode::Up));
        if let Mode::EntityDetail { scroll_offset, .. } = &app.mode {
            assert_eq!(*scroll_offset, 0);
        }

        // Up at 0 should stay at 0
        handle_key_event(&mut app, key_event(KeyCode::Up));
        if let Mode::EntityDetail { scroll_offset, .. } = &app.mode {
            assert_eq!(*scroll_offset, 0);
        }
    }
}
