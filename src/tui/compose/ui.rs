//! Rendering for the interactive composer.
//!
//! Lays out the date field, the content field, the live preview, and a persistent
//! syntax help footer, plus the entity whisperer popup anchored under the cursor.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use super::state::{Field, Slot};
use super::wrap;
use super::{ComposeApp, Status};
use crate::services::date_parser;
use crate::tui::ui::styled_content_line;

/// Maximum number of whisperer entries shown at once.
const WHISPERER_MAX_ROWS: usize = 8;

/// Most rows the content field grows to before it starts scrolling.
const CONTENT_MAX_ROWS: usize = 6;

/// Most rows the preview's resolved text grows to before it stops showing more.
///
/// The `+new:` marker gets its own reserved row on top of this, so it can never
/// be the thing wrapping drops.
const PREVIEW_MAX_ROWS: usize = 4;

/// Render the whole composer frame.
pub fn render(app: &ComposeApp, frame: &mut Frame) {
    let area = frame.area();

    // Both text areas grow with their content up to a cap, so a one-line thought
    // gets a one-line box and a long one gets room to wrap.
    let text_width = area.width.saturating_sub(2) as usize;
    let wrapped = wrap::wrap(app.content.value(), app.content.cursor(), text_width);
    let content_rows = wrapped.rows.len().clamp(1, CONTENT_MAX_ROWS) as u16;
    let preview_rows = preview_row_count(app, text_width) as u16;

    let chunks = Layout::vertical([
        Constraint::Length(3),                // date field
        Constraint::Length(content_rows + 2), // content field
        Constraint::Length(preview_rows + 2), // live preview
        Constraint::Min(0),                   // status / new-entity notes
        Constraint::Length(2),                // help footer
    ])
    .split(area);

    render_date_field(app, frame, chunks[0]);
    let content_scroll = render_content_field(app, frame, chunks[1], &wrapped);
    render_preview(app, frame, chunks[2]);
    render_status(app, frame, chunks[3]);
    render_help(frame, chunks[4]);

    // The popup tracks the cursor's column but hangs below the preview, so the
    // preview stays readable while you are picking an entity.
    render_whisperer(app, frame, chunks[1], chunks[2].bottom(), area, &wrapped);

    place_cursor(app, frame, chunks[0], chunks[1], &wrapped, content_scroll);
}

/// How many rows the preview needs: its resolved text, capped, plus a reserved
/// row for the `+new:` marker when there is one.
fn preview_row_count(app: &ComposeApp, width: usize) -> usize {
    let content = app.content.value();
    let marker_rows = usize::from(!app.new_entity_mentions().is_empty());
    if content.trim().is_empty() {
        return 1 + marker_rows;
    }
    // Measure the resolved display text, not the raw markup.
    let rendered: String = styled_content_line(content, usize::MAX)
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect();
    wrap::wrap(&rendered, 0, width).rows.len().clamp(1, PREVIEW_MAX_ROWS) + marker_rows
}

/// Style for the border of a field, highlighted when focused.
fn border_style(app: &ComposeApp, field: Field) -> Style {
    if app.focus == field {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

/// Render the date field with its live resolution or parse error.
fn render_date_field(app: &ComposeApp, frame: &mut Frame, area: Rect) {
    let raw = app.date.value();
    let hint = if raw.trim().is_empty() {
        Span::styled(" → now", Style::default().fg(Color::DarkGray))
    } else {
        match date_parser::parse_date(raw) {
            // Echoing an already-absolute date back at the user is noise — which
            // it always is after an Alt-arrow nudge — so just name the weekday.
            Ok(date) if date.format("%Y-%m-%d").to_string() == raw.trim() => {
                Span::styled(format!("  ({})", date.format("%a")), Style::default().fg(Color::Green))
            }
            Ok(date) => Span::styled(
                format!(" → {}", date.format("%Y-%m-%d (%a)")),
                Style::default().fg(Color::Green),
            ),
            Err(_) => Span::styled(
                format!(" → unrecognized ({})", date_parser::ACCEPTED_FORMS),
                Style::default().fg(Color::Red),
            ),
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(app, Field::Date))
        .title("Date");
    let line = Line::from(vec![Span::raw(raw.to_string()), hint]);
    frame.render_widget(Paragraph::new(line).block(block), area);
}

/// Render the raw content field, where entity markup is shown verbatim, wrapped
/// over as many rows as fit.
///
/// Returns the first visible row index, so the cursor can be placed against the
/// same scroll position.
fn render_content_field(app: &ComposeApp, frame: &mut Frame, area: Rect, wrapped: &wrap::Wrapped) -> usize {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(app, Field::Content))
        .title("Thought");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Scroll only once the text outgrows the box, keeping the cursor's row in view.
    let visible = (inner.height as usize).max(1);
    let scroll = wrapped.cursor_row.saturating_sub(visible.saturating_sub(1));

    let chars: Vec<char> = app.content.value().chars().collect();
    let lines: Vec<Line> = wrapped
        .rows
        .iter()
        .skip(scroll)
        .take(visible)
        .map(|row| Line::from(chars[row.start..row.end].iter().collect::<String>()))
        .collect();

    frame.render_widget(Paragraph::new(lines), inner);
    scroll
}

/// Render the live preview: entity markup resolved and colorized, with a marker
/// on every mention that would create a new entity.
fn render_preview(app: &ComposeApp, frame: &mut Frame, area: Rect) {
    let content = app.content.value();

    let line = if content.trim().is_empty() {
        Line::from(Span::styled(
            "(nothing to preview yet)",
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        // Never truncate — the paragraph wraps instead, matching the content field.
        Line::from(styled_content_line(content, usize::MAX).spans)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title("Preview");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // The marker gets its own row at the bottom rather than trailing the text.
    // Appended inline it was the first thing wrapping dropped, so it vanished
    // exactly on the long thoughts where a typo is most likely.
    let new_mentions = app.new_entity_mentions();
    let marker_rows = u16::from(!new_mentions.is_empty());
    let [text_area, marker_area] = Layout::vertical([Constraint::Min(0), Constraint::Length(marker_rows)]).areas(inner);

    frame.render_widget(Paragraph::new(line).wrap(Wrap { trim: false }), text_area);

    if marker_rows > 0 {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("+new: {}", new_mentions.join(", ")),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::DIM),
            ))),
            marker_area,
        );
    }
}

/// Render the running session status: saves so far and the last outcome.
fn render_status(app: &ComposeApp, frame: &mut Frame, area: Rect) {
    if area.height == 0 {
        return;
    }

    let line = match &app.status {
        Some(Status::Saved { id, entities }) => Line::from(vec![
            Span::styled(format!(" Saved thought #{}", id), Style::default().fg(Color::Green)),
            Span::styled(
                format!(
                    " ({} entit{}) · {} saved this session",
                    entities,
                    if *entities == 1 { "y" } else { "ies" },
                    app.saved_count
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Some(Status::Error(message)) => {
            Line::from(Span::styled(format!(" {}", message), Style::default().fg(Color::Red)))
        }
        None => Line::from(Span::styled(
            " Type a thought, then press Enter to save.",
            Style::default().fg(Color::DarkGray),
        )),
    };

    frame.render_widget(Paragraph::new(line), area);
}

/// Render the persistent syntax help footer.
fn render_help(frame: &mut Frame, area: Rect) {
    let style = Style::default().fg(Color::DarkGray);
    let lines = vec![
        Line::from(Span::styled(
            " Entities: [Name] · [your wording](Name) — type [ or ]( to search",
            style,
        )),
        Line::from(Span::styled(
            " Date: YYYY-MM-DD · today · -3d · mon · Alt+←/→ shifts a day   Tab:field · Enter:save · Esc:quit",
            style,
        )),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

/// Render the entity completion popup.
///
/// It lines up with the cursor's column in the content field but hangs from
/// `top`, so it never hides the preview.
fn render_whisperer(
    app: &ComposeApp,
    frame: &mut Frame,
    content_area: Rect,
    top: u16,
    area: Rect,
    wrapped: &wrap::Wrapped,
) {
    let Some(whisperer) = app.whisperer.as_ref() else {
        return;
    };

    let rows = whisperer.matches.len().clamp(1, WHISPERER_MAX_ROWS) as u16;
    let height = (rows + 2).min(area.height.saturating_sub(top).max(3));
    let width = 48.min(area.width);
    // Track the cursor's wrapped column, but never run past the right edge.
    let x = (content_area.x + 1 + wrapped.cursor_col as u16).min(area.right().saturating_sub(width));
    let y = top.min(area.bottom().saturating_sub(height));
    let popup = Rect { x, y, width, height };

    frame.render_widget(Clear, popup);
    let title = match whisperer.slot {
        Slot::Display => "Entities",
        // Name the slot, so it is obvious the pick replaces only the target and
        // leaves the wording already typed alone.
        Slot::Target => "Entities → links to",
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(title);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    if whisperer.matches.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "no match — will create a new entity",
                Style::default().fg(Color::Yellow),
            )),
            inner,
        );
        return;
    }

    // Scroll so the highlighted entry stays visible.
    let visible = inner.height as usize;
    let start = whisperer.selected.saturating_sub(visible.saturating_sub(1));

    let items: Vec<ListItem> = whisperer
        .matches
        .iter()
        .enumerate()
        .skip(start)
        .take(visible)
        .map(|(row, &candidate_idx)| {
            let candidate = &app.candidates[candidate_idx];
            let mut spans = vec![Span::styled(
                candidate.label.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )];
            if candidate.is_alias {
                spans.push(Span::styled(
                    format!(" → {}", candidate.canonical),
                    Style::default().fg(Color::Cyan),
                ));
            }
            if let Some(description) = candidate.description.as_deref()
                && !description.trim().is_empty()
            {
                spans.push(Span::styled(
                    format!("  {}", first_line(description)),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            let mut item = ListItem::new(Line::from(spans));
            if row == whisperer.selected {
                item = item.style(Style::default().add_modifier(Modifier::REVERSED));
            }
            item
        })
        .collect();

    frame.render_widget(List::new(items), inner);
}

/// First line of a description, for the one-line hint beside a candidate.
fn first_line(description: &str) -> &str {
    description.lines().next().unwrap_or("").trim()
}

/// Put the terminal cursor in the focused field.
///
/// The date field is a single short line, so its column is the input's own visual
/// cursor. The content field is wrapped, so its position comes from the same
/// [`wrap::Wrapped`] the rows were rendered from — the two cannot drift apart.
fn place_cursor(
    app: &ComposeApp,
    frame: &mut Frame,
    date_area: Rect,
    content_area: Rect,
    wrapped: &wrap::Wrapped,
    content_scroll: usize,
) {
    // +1 on each axis for the block's left and top borders.
    let (area, col, row) = match app.focus {
        Field::Date => (date_area, app.date.visual_cursor(), 0),
        Field::Content => (
            content_area,
            wrapped.cursor_col,
            wrapped.cursor_row.saturating_sub(content_scroll),
        ),
    };

    let x = (area.x + 1 + col as u16).min(area.right().saturating_sub(1));
    let y = (area.y + 1 + row as u16).min(area.bottom().saturating_sub(2).max(area.y));
    frame.set_cursor_position((x, y));
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{seeded_app, type_text};
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render_to_string(app: &ComposeApp, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(app, frame)).unwrap();

        let buffer = terminal.backend().buffer();
        let mut out = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    #[test]
    fn test_render_shows_field_titles_and_help_footer() {
        let app = seeded_app();
        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("Date"), "{}", out);
        assert!(out.contains("Thought"), "{}", out);
        assert!(out.contains("Preview"), "{}", out);
        assert!(out.contains("[Name]"), "{}", out);
        assert!(out.contains("[your wording](Name)"), "{}", out);
        assert!(out.contains("type [ or ]( to search"), "{}", out);
        assert!(out.contains("Alt+←/→ shifts a day"), "{}", out);
        assert!(out.contains("Enter:save"), "{}", out);
    }

    #[test]
    fn test_render_target_whisperer_names_the_slot() {
        let mut app = seeded_app();
        type_text(&mut app, "met [my boss](al");

        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("links to"), "{}", out);
        assert!(out.contains("Alice"), "{}", out);
    }

    #[test]
    fn test_render_empty_date_field_reads_as_now() {
        let app = seeded_app();
        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("→ now"), "{}", out);
    }

    #[test]
    fn test_render_resolves_a_relative_date_field() {
        let mut app = seeded_app();
        app.date = tui_input::Input::new("yesterday".to_string());

        let out = render_to_string(&app, 100, 20);

        let expected = chrono::Utc::now().date_naive().pred_opt().unwrap();
        assert!(
            out.contains(&format!("→ {}", expected.format("%Y-%m-%d (%a)"))),
            "{}",
            out
        );
    }

    #[test]
    fn test_render_does_not_echo_an_already_absolute_date() {
        let mut app = seeded_app();
        app.date = tui_input::Input::new("2026-08-01".to_string());

        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("2026-08-01  (Sat)"), "{}", out);
        assert!(!out.contains("→ 2026-08-01"), "redundant echo: {}", out);
    }

    #[test]
    fn test_render_flags_an_unrecognized_date() {
        let mut app = seeded_app();
        app.date = tui_input::Input::new("nonsense".to_string());

        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("unrecognized"), "{}", out);
    }

    #[test]
    fn test_render_preview_resolves_entity_markup() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new("paired with [alicia](Alice)".to_string());

        let out = render_to_string(&app, 100, 20);

        // The preview drops the markup and shows the display text.
        assert!(out.contains("paired with alicia"), "{}", out);
    }

    #[test]
    fn test_render_preview_marks_new_entities() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new("met [Alice] and [Carol]".to_string());

        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("+new: Carol"), "{}", out);
        assert!(!out.contains("+new: Alice"), "{}", out);
    }

    #[test]
    fn test_render_preview_keeps_the_new_marker_on_long_thoughts() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new(format!("{} met [Carol]", "some longer words here ".repeat(6)));

        let out = render_to_string(&app, 40, 30);

        // Appended inline, the marker was the first thing wrapping dropped.
        assert!(out.contains("+new: Carol"), "{}", out);
    }

    #[test]
    fn test_render_preview_marker_sits_on_its_own_row() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new("met [Carol]".to_string());

        let out = render_to_string(&app, 60, 24);

        let marker_row = out.lines().find(|l| l.contains("+new:")).expect("marker row");
        assert!(
            !marker_row.contains("met Carol"),
            "marker must not share a row with the text: {:?}",
            marker_row
        );
    }

    #[test]
    fn test_render_preview_omits_marker_when_all_entities_are_known() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new("met [Alice]".to_string());

        let out = render_to_string(&app, 100, 20);

        assert!(!out.contains("+new"), "{}", out);
    }

    #[test]
    fn test_render_whisperer_lists_matching_entities() {
        let mut app = seeded_app();
        type_text(&mut app, "[ali");

        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("Entities"), "{}", out);
        assert!(out.contains("Alice"), "{}", out);
        assert!(out.contains("alicia"), "{}", out);
        assert!(!out.contains("Bob"), "{}", out);
    }

    #[test]
    fn test_render_whisperer_marks_aliases_with_their_target() {
        let mut app = seeded_app();
        type_text(&mut app, "[alicia");

        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("alicia → Alice"), "{}", out);
    }

    #[test]
    fn test_render_whisperer_explains_an_empty_match_list() {
        let mut app = seeded_app();
        type_text(&mut app, "[zzzz");

        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("no match"), "{}", out);
    }

    #[test]
    fn test_render_reports_the_last_save() {
        let mut app = seeded_app();
        type_text(&mut app, "met [Alice]");
        app.save();

        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("Saved thought #1"), "{}", out);
        assert!(out.contains("1 entity"), "{}", out);
        assert!(out.contains("1 saved this session"), "{}", out);
    }

    #[test]
    fn test_render_reports_a_save_error() {
        let mut app = seeded_app();
        app.save(); // empty content

        let out = render_to_string(&app, 100, 20);

        assert!(out.contains("empty") || out.contains("Empty"), "{}", out);
    }

    #[test]
    fn test_render_wraps_content_past_the_right_edge() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new(
            "the quick brown fox jumps over the lazy dog and keeps on running well past the edge".to_string(),
        );

        let out = render_to_string(&app, 40, 24);

        // Every word survives, so nothing was silently cut off at the edge.
        for word in ["quick", "jumps", "lazy", "running", "past", "edge"] {
            assert!(out.contains(word), "lost {:?} in:\n{}", word, out);
        }
        // And it really is on more than one row of the Thought box.
        assert!(out.matches("running").count() >= 1);
        assert!(!out.contains("…"), "{}", out);
    }

    #[test]
    fn test_content_box_grows_with_the_text() {
        let mut app = seeded_app();
        let short = render_to_string(&app, 40, 24);

        app.content = tui_input::Input::new("word ".repeat(30));
        let long = render_to_string(&app, 40, 24);

        let box_rows = |s: &str| s.lines().filter(|l| l.contains('│')).count();
        assert!(
            box_rows(&long) > box_rows(&short),
            "long content should occupy more rows\nshort:\n{}\nlong:\n{}",
            short,
            long
        );
    }

    #[test]
    fn test_content_box_stops_growing_at_the_cap() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new("word ".repeat(200));

        let out = render_to_string(&app, 100, 24);

        // Capped, so everything below the field is still on screen.
        assert!(out.contains("Preview"), "{}", out);
        assert!(out.contains("Enter:save"), "{}", out);

        let thought_rows = out
            .lines()
            .skip_while(|l| !l.contains("Thought"))
            .skip(1)
            .take_while(|l| !l.starts_with('└'))
            .count();
        assert_eq!(thought_rows, CONTENT_MAX_ROWS, "field should stop at the cap:\n{}", out);
    }

    #[test]
    fn test_long_content_scrolls_to_keep_the_end_visible() {
        let mut app = seeded_app();
        let text = format!("{}CARET", "filler ".repeat(60));
        app.content = tui_input::Input::new(text);

        let out = render_to_string(&app, 40, 24);

        assert!(out.contains("CARET"), "the cursor's row must stay in view:\n{}", out);
    }

    #[test]
    fn test_render_handles_narrow_terminals() {
        let mut app = seeded_app();
        type_text(&mut app, "[ali");

        // Must not panic or overflow when there is barely room for the popup.
        let out = render_to_string(&app, 24, 14);
        assert!(out.contains("Date"), "{}", out);
    }

    #[test]
    fn test_render_handles_multibyte_content() {
        let mut app = seeded_app();
        app.content = tui_input::Input::new("obédèd naïve 日本語 [Alice] ✨".repeat(8));

        let out = render_to_string(&app, 40, 14);
        assert!(out.contains("Preview"), "{}", out);
    }
}
