//! Key event handling for the interactive composer.
//!
//! Two key maps: one while the entity whisperer popup is open, one while it is
//! closed. See [`handle_key_event`].

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_input::backend::crossterm::EventHandler;

use super::state::{Field, Slot, Whisperer};
use super::{ComposeApp, Status};
use crate::tui::fuzzy;

/// Handle a key event and update composer state.
///
/// While the whisperer is open it takes priority, capturing navigation and
/// acceptance keys; everything else falls through to the focused text field.
pub fn handle_key_event(app: &mut ComposeApp, key: KeyEvent) {
    // Ctrl-C always leaves, whatever is on screen.
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        app.should_quit = true;
        return;
    }

    // Alt-arrows nudge the date from wherever you are, so a one-day correction
    // costs no field switch. `tui_input` leaves these unbound (it takes the Ctrl
    // variants for word movement), so nothing is clobbered.
    if key.modifiers.contains(KeyModifiers::ALT) {
        match key.code {
            KeyCode::Left => return app.nudge_date(-1),
            KeyCode::Right => return app.nudge_date(1),
            _ => {}
        }
    }

    if app.whisperer.is_some() {
        handle_whisperer_key(app, key);
    } else {
        handle_field_key(app, key);
    }
}

/// Key handling while the entity completion popup is open.
fn handle_whisperer_key(app: &mut ComposeApp, key: KeyEvent) {
    let has_matches = app.whisperer.as_ref().is_some_and(|w| w.selected_candidate().is_some());

    match key.code {
        KeyCode::Up => {
            if let Some(whisperer) = app.whisperer.as_mut() {
                whisperer.select_previous();
            }
        }
        KeyCode::Down => {
            if let Some(whisperer) = app.whisperer.as_mut() {
                whisperer.select_next();
            }
        }
        KeyCode::Esc => {
            // Dismiss the popup but keep whatever was typed.
            app.whisperer = None;
        }
        KeyCode::Tab | KeyCode::Enter if has_matches => {
            accept_completion(app);
        }
        // With nothing to complete, the popup is just noise: dismiss it and let
        // the key do what it normally would.
        KeyCode::Tab | KeyCode::Enter => {
            app.whisperer = None;
            handle_field_key(app, key);
        }
        _ => {
            app.content.handle_event(&ratatui::crossterm::event::Event::Key(key));
            recompute_whisperer(app, key);
        }
    }
}

/// Key handling while no popup is open.
fn handle_field_key(app: &mut ComposeApp, key: KeyEvent) {
    // Any key other than a second Esc disarms a pending discard.
    if !matches!(key.code, KeyCode::Esc) {
        app.pending_quit = false;
    }

    match key.code {
        // Leaving with an unsaved thought is the one destructive dismissal in the
        // composer, so it takes a confirmation the others do not.
        KeyCode::Esc => {
            if app.content.value().trim().is_empty() || app.pending_quit {
                app.should_quit = true;
            } else {
                app.pending_quit = true;
                app.status = Some(Status::Error(
                    "Unsaved thought. Press Esc again to discard it, or Enter to save.".to_string(),
                ));
            }
        }
        KeyCode::Tab | KeyCode::BackTab => app.focus = app.focus.toggled(),
        KeyCode::Enter => app.save(),
        _ => {
            let field = match app.focus {
                Field::Date => &mut app.date,
                Field::Content => &mut app.content,
            };
            field.handle_event(&ratatui::crossterm::event::Event::Key(key));
            if app.focus == Field::Content {
                recompute_whisperer(app, key);
            }
        }
    }
}

/// Replace the partial reference in the content with the selected candidate's
/// text, and place the cursor just past it.
///
/// A trailing space is appended so typing can continue straight away — see
/// [`needs_trailing_space`] for when that is skipped.
fn accept_completion(app: &mut ComposeApp) {
    let Some(whisperer) = app.whisperer.as_ref() else {
        return;
    };
    let Some(candidate_idx) = whisperer.selected_candidate() else {
        return;
    };
    let mut insertion = app.candidates[candidate_idx].insertion(whisperer.slot);
    let open_at = whisperer.open_at;
    let cursor = app.content.cursor();

    let chars: Vec<char> = app.content.value().chars().collect();
    let prefix: String = chars[..open_at.min(chars.len())].iter().collect();
    let suffix: String = chars[cursor.min(chars.len())..].iter().collect();

    if needs_trailing_space(&suffix) {
        insertion.push(' ');
    }
    let new_cursor = open_at + insertion.chars().count();

    app.content = tui_input::Input::new(format!("{}{}{}", prefix, insertion, suffix)).with_cursor(new_cursor);
    app.whisperer = None;
}

/// Whether an accepted completion should be followed by a space.
///
/// Yes at the end of the line, which is the usual case. Skipped when the text
/// that follows already starts with whitespace (it would double up) or with
/// punctuation that should hug the reference — `[Alice].`, `[Alice]'s`, `[Alice],`.
fn needs_trailing_space(suffix: &str) -> bool {
    match suffix.chars().next() {
        None => true,
        Some(next) => {
            !next.is_whitespace() && !matches!(next, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '\'' | '"')
        }
    }
}

/// Open, update, or close the whisperer after a content keystroke.
///
/// Two triggers:
///
/// - `[` opens the popup on the reference's display slot, anchored at that bracket.
/// - `(` typed straight after a `]` opens it on the target slot, so a reference whose
///   wording was written by hand — `[my boss](…` — can still complete its entity.
///
/// While open, the query is the text between the opening bracket and the cursor. The
/// popup closes if the cursor moves back to or past that bracket, if the bracket is
/// edited away, or if the query grows the slot's closing character.
fn recompute_whisperer(app: &mut ComposeApp, key: KeyEvent) {
    let cursor = app.content.cursor();

    if let Some(slot) = opening_slot(app, key, cursor) {
        // The bracket just inserted sits immediately before the cursor.
        app.whisperer = Some(Whisperer {
            open_at: cursor.saturating_sub(1),
            slot,
            matches: matching_candidates(app, "", slot),
            selected: 0,
        });
        return;
    }

    let Some((open_at, slot)) = app.whisperer.as_ref().map(|w| (w.open_at, w.slot)) else {
        return;
    };

    // Cursor moved back to or before the opening bracket - the popup no longer
    // describes what is being typed.
    if cursor <= open_at {
        app.whisperer = None;
        return;
    }

    let chars: Vec<char> = app.content.value().chars().collect();
    // The bracket was edited away (e.g. the whole field was cleared).
    if chars.get(open_at) != Some(&slot.opening_char()) {
        app.whisperer = None;
        return;
    }

    let query: String = chars[open_at + 1..cursor.min(chars.len())].iter().collect();
    if query.contains(slot.closing_char()) {
        app.whisperer = None;
        return;
    }

    let matches = matching_candidates(app, &query, slot);
    if let Some(whisperer) = app.whisperer.as_mut() {
        whisperer.matches = matches;
        whisperer.selected = 0;
    }
}

/// Candidate indices matching `query`, restricted to those valid in `slot`.
fn matching_candidates(app: &ComposeApp, query: &str, slot: Slot) -> Vec<usize> {
    let selectable = app.selectable_candidates(slot);
    let labels: Vec<&str> = selectable.iter().map(|&i| app.candidates[i].label.as_str()).collect();
    fuzzy::match_indices(query, &labels)
        .into_iter()
        .map(|matched| selectable[matched])
        .collect()
}

/// The slot a just-typed character opens a popup for, if any.
///
/// `(` only counts when it directly follows a `]`, so ordinary parenthetical prose
/// never triggers the whisperer.
fn opening_slot(app: &ComposeApp, key: KeyEvent, cursor: usize) -> Option<Slot> {
    match key.code {
        KeyCode::Char('[') => Some(Slot::Display),
        KeyCode::Char('(') => {
            let preceding = app.content.value().chars().nth(cursor.checked_sub(2)?)?;
            (preceding == ']').then_some(Slot::Target)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::Status;
    use super::super::test_support::{seeded_app, type_text};
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    /// Labels of the candidates currently offered by the whisperer.
    fn offered(app: &ComposeApp) -> Vec<String> {
        app.whisperer
            .as_ref()
            .map(|w| w.matches.iter().map(|&i| app.candidates[i].label.clone()).collect())
            .unwrap_or_default()
    }

    #[test]
    fn test_content_open_bracket_opens_whisperer() {
        let mut app = seeded_app();

        type_text(&mut app, "met [");

        let whisperer = app.whisperer.as_ref().expect("whisperer should be open");
        assert_eq!(whisperer.open_at, 4);
        // With an empty query every candidate is offered.
        assert_eq!(whisperer.matches.len(), app.candidates.len());
    }

    #[test]
    fn test_whisperer_typing_narrows_matches() {
        let mut app = seeded_app();

        type_text(&mut app, "[ali");

        let offered = offered(&app);
        assert!(offered.contains(&"Alice".to_string()));
        assert!(offered.contains(&"alicia".to_string()));
        assert!(!offered.contains(&"Bob".to_string()));
    }

    #[test]
    fn test_whisperer_tab_inserts_canonical_reference() {
        let mut app = seeded_app();
        type_text(&mut app, "met [alic");
        // Alice outranks the alias for this query.
        handle_key_event(&mut app, key(KeyCode::Tab));

        assert_eq!(app.content.value(), "met [Alice] ");
        assert_eq!(app.content.cursor(), 12, "cursor sits after the appended space");
        assert!(app.whisperer.is_none());
    }

    #[test]
    fn test_whisperer_enter_accepts_instead_of_saving() {
        let mut app = seeded_app();
        type_text(&mut app, "[bob");

        handle_key_event(&mut app, key(KeyCode::Enter));

        assert_eq!(app.content.value(), "[Bob] ");
        assert_eq!(app.saved_count, 0, "Enter should complete, not save");
    }

    #[test]
    fn test_whisperer_selecting_alias_inserts_aliased_reference() {
        let mut app = seeded_app();
        type_text(&mut app, "[alicia");
        // Exact alias text ranks the alias first.
        handle_key_event(&mut app, key(KeyCode::Tab));

        assert_eq!(app.content.value(), "[alicia](Alice) ");
    }

    #[test]
    fn test_whisperer_down_moves_selection() {
        let mut app = seeded_app();
        type_text(&mut app, "[ali");
        let first = app.whisperer.as_ref().unwrap().selected_candidate();

        handle_key_event(&mut app, key(KeyCode::Down));
        let second = app.whisperer.as_ref().unwrap().selected_candidate();

        assert_ne!(first, second);
    }

    #[test]
    fn test_whisperer_accepts_the_highlighted_candidate() {
        let mut app = seeded_app();
        type_text(&mut app, "[ali");
        handle_key_event(&mut app, key(KeyCode::Down));
        let expected = {
            let idx = app.whisperer.as_ref().unwrap().selected_candidate().unwrap();
            app.candidates[idx].insertion(Slot::Display)
        };

        handle_key_event(&mut app, key(KeyCode::Tab));

        assert_eq!(app.content.value(), format!("{} ", expected));
    }

    #[test]
    fn test_completion_lets_typing_continue_without_a_manual_space() {
        let mut app = seeded_app();
        type_text(&mut app, "met [alic");
        handle_key_event(&mut app, key(KeyCode::Tab));

        type_text(&mut app, "at the standup");

        assert_eq!(app.content.value(), "met [Alice] at the standup");
    }

    #[test]
    fn test_completion_does_not_double_an_existing_space() {
        let mut app = seeded_app();
        type_text(&mut app, "met  today");
        for _ in 0..6 {
            handle_key_event(&mut app, key(KeyCode::Left));
        }
        type_text(&mut app, "[bo");

        handle_key_event(&mut app, key(KeyCode::Tab));

        assert_eq!(app.content.value(), "met [Bob] today");
    }

    #[test]
    fn test_completion_does_not_space_off_following_punctuation() {
        let mut app = seeded_app();
        type_text(&mut app, "met .");
        handle_key_event(&mut app, key(KeyCode::Left));
        type_text(&mut app, "[bo");

        handle_key_event(&mut app, key(KeyCode::Tab));

        assert_eq!(app.content.value(), "met [Bob].");
    }

    #[test]
    fn test_completion_does_not_space_off_a_following_apostrophe() {
        let mut app = seeded_app();
        type_text(&mut app, "met 's laptop");
        for _ in 0..9 {
            handle_key_event(&mut app, key(KeyCode::Left));
        }
        type_text(&mut app, "[bo");

        handle_key_event(&mut app, key(KeyCode::Tab));

        assert_eq!(app.content.value(), "met [Bob]'s laptop");
    }

    #[test]
    fn test_target_completion_also_appends_a_space() {
        let mut app = seeded_app();
        type_text(&mut app, "met [my boss](ali");
        handle_key_event(&mut app, key(KeyCode::Tab));

        type_text(&mut app, "today");

        assert_eq!(app.content.value(), "met [my boss](Alice) today");
    }

    #[test]
    fn test_trailing_space_is_trimmed_before_saving() {
        let mut app = seeded_app();
        type_text(&mut app, "met [alic");
        handle_key_event(&mut app, key(KeyCode::Tab));
        assert_eq!(app.content.value(), "met [Alice] ", "the space is there while editing");

        handle_key_event(&mut app, key(KeyCode::Enter));

        assert_eq!(app.saved_count, 1);
        let saved = crate::storage::thoughts_repository::ThoughtsRepository::list_all(&app.conn).unwrap();
        assert_eq!(saved[0].content, "met [Alice]", "no trailing space reaches storage");
    }

    #[test]
    fn test_whisperer_closing_bracket_dismisses_popup() {
        let mut app = seeded_app();

        type_text(&mut app, "[Carol]");

        assert!(app.whisperer.is_none());
        assert_eq!(app.content.value(), "[Carol]");
    }

    #[test]
    fn test_whisperer_backspacing_past_bracket_dismisses_popup() {
        let mut app = seeded_app();
        type_text(&mut app, "[al");

        handle_key_event(&mut app, key(KeyCode::Backspace));
        assert!(app.whisperer.is_some(), "still inside the reference");
        handle_key_event(&mut app, key(KeyCode::Backspace));
        assert!(app.whisperer.is_some(), "cursor is still just past the bracket");
        handle_key_event(&mut app, key(KeyCode::Backspace));

        assert!(app.whisperer.is_none(), "the bracket itself is gone");
        assert_eq!(app.content.value(), "");
    }

    #[test]
    fn test_whisperer_cursor_left_past_bracket_dismisses_popup() {
        let mut app = seeded_app();
        type_text(&mut app, "[al");

        handle_key_event(&mut app, key(KeyCode::Left));
        handle_key_event(&mut app, key(KeyCode::Left));
        assert!(app.whisperer.is_some());
        handle_key_event(&mut app, key(KeyCode::Left));

        assert!(app.whisperer.is_none());
    }

    #[test]
    fn test_whisperer_esc_dismisses_popup_without_quitting() {
        let mut app = seeded_app();
        type_text(&mut app, "[ali");

        handle_key_event(&mut app, key(KeyCode::Esc));

        assert!(app.whisperer.is_none());
        assert!(!app.should_quit);
        assert_eq!(app.content.value(), "[ali", "typed text is preserved");
    }

    #[test]
    fn test_whisperer_with_no_matches_lets_enter_save() {
        let mut app = seeded_app();
        type_text(&mut app, "[zzzz");
        assert!(app.whisperer.as_ref().unwrap().matches.is_empty());

        handle_key_event(&mut app, key(KeyCode::Enter));

        assert!(app.whisperer.is_none());
        assert_eq!(app.saved_count, 1, "free-form entity names are still allowed");
    }

    #[test]
    fn test_open_paren_after_bracket_opens_target_whisperer() {
        let mut app = seeded_app();

        type_text(&mut app, "met [my boss](");

        let whisperer = app.whisperer.as_ref().expect("whisperer should be open");
        assert_eq!(whisperer.slot, Slot::Target);
        assert_eq!(whisperer.open_at, 13);
        assert_eq!(whisperer.matches.len(), app.candidates.len());
    }

    #[test]
    fn test_target_whisperer_completes_the_link_keeping_the_wording() {
        let mut app = seeded_app();

        type_text(&mut app, "met [my boss](ali");
        handle_key_event(&mut app, key(KeyCode::Tab));

        assert_eq!(app.content.value(), "met [my boss](Alice) ");
        assert_eq!(app.content.cursor(), 21, "cursor lands past the appended space");
        assert!(app.whisperer.is_none());
    }

    #[test]
    fn test_target_whisperer_resolves_an_alias_to_its_canonical_name() {
        let mut app = seeded_app();
        type_text(&mut app, "met [my boss](alicia");

        handle_key_event(&mut app, key(KeyCode::Tab));

        // The display wording is already written, so picking the alias contributes
        // only the entity it points at - never `[my boss](alicia)`.
        assert_eq!(app.content.value(), "met [my boss](Alice) ");
    }

    #[test]
    fn test_target_whisperer_narrows_on_the_target_query() {
        let mut app = seeded_app();

        type_text(&mut app, "[my boss](bo");

        let offered = offered(&app);
        assert_eq!(offered, vec!["Bob".to_string()]);
    }

    #[test]
    fn test_target_whisperer_closes_on_its_own_closing_paren() {
        let mut app = seeded_app();

        type_text(&mut app, "[my boss](Alice)");

        assert!(app.whisperer.is_none());
        assert_eq!(app.content.value(), "[my boss](Alice)");
    }

    #[test]
    fn test_target_whisperer_ignores_a_closing_bracket() {
        let mut app = seeded_app();

        // `]` ends the display slot, not the target slot - it must not close this popup.
        type_text(&mut app, "[my boss](al]");

        assert!(app.whisperer.is_some());
    }

    #[test]
    fn test_paren_in_ordinary_prose_does_not_open_the_whisperer() {
        let mut app = seeded_app();

        type_text(&mut app, "met Alice (by the way");

        assert!(app.whisperer.is_none());
    }

    #[test]
    fn test_paren_at_start_of_content_does_not_open_the_whisperer() {
        let mut app = seeded_app();

        type_text(&mut app, "(");

        assert!(app.whisperer.is_none());
    }

    #[test]
    fn test_target_whisperer_dismissed_by_esc_keeps_typed_text() {
        let mut app = seeded_app();
        type_text(&mut app, "[my boss](ali");

        handle_key_event(&mut app, key(KeyCode::Esc));

        assert!(app.whisperer.is_none());
        assert!(!app.should_quit);
        assert_eq!(app.content.value(), "[my boss](ali");
    }

    #[test]
    fn test_completed_free_form_link_saves_against_the_existing_entity() {
        let mut app = seeded_app();
        type_text(&mut app, "met [my boss](ali");
        handle_key_event(&mut app, key(KeyCode::Tab));

        // The target resolves to a known entity, so nothing new is created.
        assert!(app.new_entity_mentions().is_empty());
    }

    #[test]
    fn test_whisperer_reopens_on_a_second_bracket() {
        let mut app = seeded_app();
        type_text(&mut app, "[Alice] and [bo");

        let whisperer = app.whisperer.as_ref().expect("whisperer should be open");
        assert_eq!(whisperer.open_at, 12);
        assert_eq!(offered(&app), vec!["Bob".to_string()]);
    }

    #[test]
    fn test_field_tab_switches_focus() {
        let mut app = seeded_app();
        assert_eq!(app.focus, Field::Content);

        handle_key_event(&mut app, key(KeyCode::Tab));
        assert_eq!(app.focus, Field::Date);

        handle_key_event(&mut app, key(KeyCode::BackTab));
        assert_eq!(app.focus, Field::Content);
    }

    #[test]
    fn test_field_typing_goes_to_the_focused_field() {
        let mut app = seeded_app();
        handle_key_event(&mut app, key(KeyCode::Tab));

        type_text(&mut app, "yesterday");

        assert_eq!(app.date.value(), "yesterday");
        assert_eq!(app.content.value(), "");
    }

    #[test]
    fn test_bracket_in_date_field_does_not_open_whisperer() {
        let mut app = seeded_app();
        handle_key_event(&mut app, key(KeyCode::Tab));

        type_text(&mut app, "[");

        assert!(app.whisperer.is_none());
    }

    #[test]
    fn test_field_enter_saves_and_clears_content() {
        let mut app = seeded_app();
        type_text(&mut app, "a thought");

        handle_key_event(&mut app, key(KeyCode::Enter));

        assert_eq!(app.saved_count, 1);
        assert_eq!(app.content.value(), "");
    }

    #[test]
    fn test_field_enter_from_date_field_also_saves() {
        let mut app = seeded_app();
        type_text(&mut app, "a thought");
        handle_key_event(&mut app, key(KeyCode::Tab));

        handle_key_event(&mut app, key(KeyCode::Enter));

        assert_eq!(app.saved_count, 1);
    }

    #[test]
    fn test_field_enter_with_invalid_date_does_not_save() {
        let mut app = seeded_app();
        handle_key_event(&mut app, key(KeyCode::Tab));
        type_text(&mut app, "nonsense");
        handle_key_event(&mut app, key(KeyCode::Tab));
        type_text(&mut app, "a thought");

        handle_key_event(&mut app, key(KeyCode::Enter));

        assert_eq!(app.saved_count, 0);
        assert!(matches!(app.status, Some(Status::Error(_))));
        assert_eq!(app.content.value(), "a thought");
    }

    /// Alt-arrow, as the terminal delivers it.
    fn alt(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::ALT)
    }

    fn days_from_today(offset: i64) -> String {
        let today = chrono::Utc::now().date_naive();
        let magnitude = chrono::Days::new(offset.unsigned_abs());
        let date = if offset < 0 {
            today.checked_sub_days(magnitude).unwrap()
        } else {
            today.checked_add_days(magnitude).unwrap()
        };
        date.format("%Y-%m-%d").to_string()
    }

    #[test]
    fn test_alt_left_shifts_the_date_back_without_leaving_content() {
        let mut app = seeded_app();
        type_text(&mut app, "met [Alice]");

        handle_key_event(&mut app, alt(KeyCode::Left));

        assert_eq!(app.date.value(), days_from_today(-1));
        assert_eq!(app.focus, Field::Content, "focus must not move");
        assert_eq!(app.content.value(), "met [Alice]", "content is untouched");
    }

    #[test]
    fn test_alt_right_shifts_the_date_forward() {
        let mut app = seeded_app();

        handle_key_event(&mut app, alt(KeyCode::Right));

        assert_eq!(app.date.value(), days_from_today(1));
    }

    #[test]
    fn test_alt_arrows_accumulate() {
        let mut app = seeded_app();

        for _ in 0..3 {
            handle_key_event(&mut app, alt(KeyCode::Left));
        }

        assert_eq!(app.date.value(), days_from_today(-3));
    }

    #[test]
    fn test_alt_arrow_resolves_a_relative_date_to_an_absolute_one() {
        let mut app = seeded_app();
        app.date = tui_input::Input::new("yesterday".to_string());

        handle_key_event(&mut app, alt(KeyCode::Left));

        assert_eq!(app.date.value(), days_from_today(-2));
    }

    #[test]
    fn test_alt_arrow_shifts_an_absolute_date() {
        let mut app = seeded_app();
        app.date = tui_input::Input::new("2026-03-01".to_string());

        handle_key_event(&mut app, alt(KeyCode::Left));

        assert_eq!(app.date.value(), "2026-02-28");
    }

    #[test]
    fn test_alt_arrow_leaves_an_unparseable_date_alone() {
        let mut app = seeded_app();
        app.date = tui_input::Input::new("nonsen".to_string());

        handle_key_event(&mut app, alt(KeyCode::Left));

        assert_eq!(app.date.value(), "nonsen", "must not overwrite a half-typed date");
    }

    #[test]
    fn test_alt_arrows_work_while_the_whisperer_is_open() {
        let mut app = seeded_app();
        type_text(&mut app, "met [ali");

        handle_key_event(&mut app, alt(KeyCode::Left));

        assert_eq!(app.date.value(), days_from_today(-1));
        assert!(app.whisperer.is_some(), "the popup should survive a date nudge");
    }

    #[test]
    fn test_plain_arrows_still_move_the_cursor() {
        let mut app = seeded_app();
        type_text(&mut app, "abc");

        handle_key_event(&mut app, key(KeyCode::Left));

        assert_eq!(app.content.cursor(), 2);
        assert_eq!(app.date.value(), "", "an unmodified arrow must not touch the date");
    }

    #[test]
    fn test_ctrl_arrows_still_move_by_word() {
        let mut app = seeded_app();
        type_text(&mut app, "one two");

        handle_key_event(&mut app, KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL));

        assert_eq!(app.content.cursor(), 4, "Ctrl+Left is tui-input's word movement");
        assert_eq!(app.date.value(), "");
    }

    #[test]
    fn test_field_esc_quits_when_nothing_is_typed() {
        let mut app = seeded_app();

        handle_key_event(&mut app, key(KeyCode::Esc));

        assert!(app.should_quit);
    }

    #[test]
    fn test_esc_with_unsaved_content_asks_before_discarding() {
        let mut app = seeded_app();
        type_text(&mut app, "a thought worth keeping");

        handle_key_event(&mut app, key(KeyCode::Esc));

        assert!(!app.should_quit, "one Esc must not throw away typed text");
        assert!(app.pending_quit);
        assert!(matches!(app.status, Some(Status::Error(ref m)) if m.contains("Esc again")));
        assert_eq!(app.content.value(), "a thought worth keeping");
    }

    #[test]
    fn test_second_esc_discards_and_quits() {
        let mut app = seeded_app();
        type_text(&mut app, "a thought");

        handle_key_event(&mut app, key(KeyCode::Esc));
        handle_key_event(&mut app, key(KeyCode::Esc));

        assert!(app.should_quit);
    }

    #[test]
    fn test_typing_after_esc_disarms_the_discard() {
        let mut app = seeded_app();
        type_text(&mut app, "a thought");
        handle_key_event(&mut app, key(KeyCode::Esc));

        type_text(&mut app, "!");
        handle_key_event(&mut app, key(KeyCode::Esc));

        assert!(!app.should_quit, "the confirmation must be re-armed after an edit");
        assert!(app.pending_quit);
    }

    #[test]
    fn test_saving_clears_the_pending_discard() {
        let mut app = seeded_app();
        type_text(&mut app, "a thought");
        handle_key_event(&mut app, key(KeyCode::Esc));

        handle_key_event(&mut app, key(KeyCode::Enter));
        handle_key_event(&mut app, key(KeyCode::Esc));

        assert!(app.should_quit, "content is empty after a save, so Esc just leaves");
        assert_eq!(app.saved_count, 1);
    }

    #[test]
    fn test_whisperer_esc_does_not_arm_a_discard() {
        let mut app = seeded_app();
        type_text(&mut app, "met [ali");

        handle_key_event(&mut app, key(KeyCode::Esc));

        assert!(app.whisperer.is_none(), "the popup closes");
        assert!(!app.pending_quit, "closing the popup is not a quit attempt");
        assert!(!app.should_quit);
    }

    #[test]
    fn test_target_slot_hides_entities_whose_name_has_parentheses() {
        let mut app = seeded_app();
        app.candidates.push(super::super::state::Candidate {
            label: "Wetware (project)".to_string(),
            canonical: "Wetware (project)".to_string(),
            is_alias: false,
            description: None,
        });

        // Display slot can express it: `[Wetware (project)]` parses fine.
        type_text(&mut app, "[wetware");
        assert!(offered(&app).contains(&"Wetware (project)".to_string()));

        // Target slot cannot: `](Wetware (project))` misparses, so do not offer it.
        app.content = tui_input::Input::default();
        app.whisperer = None;
        type_text(&mut app, "[my thing](wetware");
        assert!(
            !offered(&app).contains(&"Wetware (project)".to_string()),
            "offered: {:?}",
            offered(&app)
        );
    }

    #[test]
    fn test_target_slot_still_offers_ordinary_entities() {
        let mut app = seeded_app();

        type_text(&mut app, "[my boss](ali");

        assert!(offered(&app).contains(&"Alice".to_string()));
    }

    #[test]
    fn test_ctrl_c_quits_even_with_whisperer_open() {
        let mut app = seeded_app();
        type_text(&mut app, "[ali");

        handle_key_event(&mut app, KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));

        assert!(app.should_quit);
    }

    #[test]
    fn test_completion_inserts_mid_line_keeping_the_suffix() {
        let mut app = seeded_app();
        type_text(&mut app, "met  today");
        // Move the cursor back between "met " and " today".
        for _ in 0..6 {
            handle_key_event(&mut app, key(KeyCode::Left));
        }
        type_text(&mut app, "[bo");
        handle_key_event(&mut app, key(KeyCode::Tab));

        assert_eq!(app.content.value(), "met [Bob] today");
        assert_eq!(app.content.cursor(), 9);
    }
}
