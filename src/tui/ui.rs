//! TUI rendering functions
//!
//! All rendering logic for the thought list, status bar, entity picker overlay,
//! and entity description popup.

use owo_colors::AnsiColors;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::services::entity_parser::ENTITY_PATTERN;

use super::App;
use super::state::Mode;

/// Color palette matching the existing EntityStyler colors.
const ENTITY_COLORS: [AnsiColors; 12] = [
    AnsiColors::Cyan,
    AnsiColors::Green,
    AnsiColors::Yellow,
    AnsiColors::Blue,
    AnsiColors::Magenta,
    AnsiColors::Red,
    AnsiColors::BrightCyan,
    AnsiColors::BrightGreen,
    AnsiColors::BrightYellow,
    AnsiColors::BrightBlue,
    AnsiColors::BrightMagenta,
    AnsiColors::BrightRed,
];

/// Map an owo-colors AnsiColor to a ratatui Color.
///
/// Ensures visual consistency between CLI output and TUI display
/// by using the same color mapping.
pub fn ansi_to_ratatui_color(color: AnsiColors) -> Color {
    match color {
        AnsiColors::Black => Color::Black,
        AnsiColors::Red => Color::Red,
        AnsiColors::Green => Color::Green,
        AnsiColors::Yellow => Color::Yellow,
        AnsiColors::Blue => Color::Blue,
        AnsiColors::Magenta => Color::Magenta,
        AnsiColors::Cyan => Color::Cyan,
        AnsiColors::White => Color::White,
        AnsiColors::BrightBlack => Color::DarkGray,
        AnsiColors::BrightRed => Color::LightRed,
        AnsiColors::BrightGreen => Color::LightGreen,
        AnsiColors::BrightYellow => Color::LightYellow,
        AnsiColors::BrightBlue => Color::LightBlue,
        AnsiColors::BrightMagenta => Color::LightMagenta,
        AnsiColors::BrightCyan => Color::LightCyan,
        AnsiColors::BrightWhite => Color::White,
        _ => Color::Reset,
    }
}

/// Get or assign a color index for an entity name (case-insensitive).
///
/// Uses a simple hash to consistently assign colors without needing mutable state.
fn entity_color_index(entity: &str) -> usize {
    let lower = entity.to_lowercase();
    let hash: usize = lower
        .bytes()
        .fold(0usize, |acc, b| acc.wrapping_mul(31).wrapping_add(b as usize));
    hash % ENTITY_COLORS.len()
}

/// Get the ratatui Color for an entity name.
fn entity_color(entity: &str) -> Color {
    ansi_to_ratatui_color(ENTITY_COLORS[entity_color_index(entity)])
}

/// Build a styled Line from thought content, highlighting entity references.
fn styled_content_line(content: &str, max_width: usize) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut last_end = 0;

    for cap in ENTITY_PATTERN.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let display_text = cap[1].trim();
        let target_entity = cap.get(2).map(|m| m.as_str().trim()).unwrap_or(display_text);

        // Plain text before this entity
        if full_match.start() > last_end {
            spans.push(Span::raw(content[last_end..full_match.start()].to_string()));
        }

        // Styled entity reference
        let color = entity_color(target_entity);
        spans.push(Span::styled(
            display_text.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ));

        last_end = full_match.end();
    }

    // Remaining text
    if last_end < content.len() {
        spans.push(Span::raw(content[last_end..].to_string()));
    }

    // Truncate if needed
    let line = Line::from(spans);
    let total_width: usize = line.spans.iter().map(|s| s.content.len()).sum();
    if total_width > max_width && max_width > 3 {
        // Rebuild with truncation — simplified approach: just use raw truncated string
        let truncated = if content.len() > max_width - 1 {
            format!("{}...", &content[..max_width.saturating_sub(3)])
        } else {
            content.to_string()
        };
        // Re-render the truncated content with styling
        let mut spans2: Vec<Span<'static>> = Vec::new();
        let mut last_end2 = 0;
        for cap in ENTITY_PATTERN.captures_iter(&truncated) {
            let full_match = cap.get(0).unwrap();
            let display_text = cap[1].trim();
            let target_entity = cap.get(2).map(|m| m.as_str().trim()).unwrap_or(display_text);
            if full_match.start() > last_end2 {
                spans2.push(Span::raw(truncated[last_end2..full_match.start()].to_string()));
            }
            let color = entity_color(target_entity);
            spans2.push(Span::styled(
                display_text.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
            last_end2 = full_match.end();
        }
        if last_end2 < truncated.len() {
            spans2.push(Span::raw(truncated[last_end2..].to_string()));
        }
        return Line::from(spans2);
    }

    line
}

/// Render the full TUI frame.
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Min(3),    // thought list
        Constraint::Length(1), // status bar
    ])
    .split(area);

    render_thought_list(app, frame, chunks[0]);
    render_status_bar(app, frame, chunks[1]);

    // Render overlays on top
    match &app.mode {
        Mode::ConfirmDelete { .. } => render_confirm_delete(app, frame, area),
        Mode::EntityPicker { .. } => render_entity_picker(app, frame, area),
        Mode::EntityDetail { .. } => render_entity_detail(app, frame, area),
        Mode::Normal => {}
    }
}

/// Render the thought list in the main area.
fn render_thought_list(app: &App, frame: &mut Frame, area: Rect) {
    if app.displayed_thoughts.is_empty() {
        let message = if app.active_filter.is_some() {
            let filter_name = app.active_filter.as_deref().unwrap_or("");
            format!("No thoughts referencing \"{}\"", filter_name)
        } else {
            "No thoughts recorded yet".to_string()
        };
        let paragraph = Paragraph::new(message)
            .block(Block::default().borders(Borders::ALL).title("Thoughts"))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
        return;
    }

    let inner_width = area.width.saturating_sub(2) as usize; // account for borders

    let items: Vec<ListItem> = app
        .displayed_thoughts
        .iter()
        .map(|&idx| {
            let thought = &app.thoughts[idx];
            let date_str = thought.created_at.format("%Y-%m-%d %H:%M").to_string();
            let date_span = Span::styled(format!("{} ", date_str), Style::default().fg(Color::DarkGray));

            let content_max = inner_width.saturating_sub(date_str.len() + 1);
            let content_line = styled_content_line(&thought.content, content_max);

            let mut spans = vec![date_span];
            spans.extend(content_line.spans);

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if let Some(ref filter) = app.active_filter {
        format!("Thoughts [filtered: {}]", filter)
    } else {
        "Thoughts".to_string()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, area, &mut app.list_state.clone());
}

/// Render the status bar with sort order, active filter, and key hints.
fn render_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    let sort_label = format!("Sort: {}", app.sort_order.label());
    let hints = "q:Quit  /:Filter  s:Sort  x:Delete  Enter:Details  ?:Help";

    let mut spans = vec![
        Span::styled(format!(" {} ", sort_label), Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
    ];

    if let Some(ref filter) = app.active_filter {
        spans.push(Span::styled(
            format!("Filter: {} ", filter),
            Style::default().fg(Color::Yellow),
        ));
        spans.push(Span::raw("(Esc to clear) | "));
    }

    spans.push(Span::styled(hints.to_string(), Style::default().fg(Color::DarkGray)));

    let status = Line::from(spans);
    frame.render_widget(Paragraph::new(status), area);
}

/// Render the delete confirmation overlay.
fn render_confirm_delete(app: &App, frame: &mut Frame, area: Rect) {
    let Mode::ConfirmDelete { thought_index } = app.mode else {
        return;
    };

    let thought = &app.thoughts[thought_index];
    let date_str = thought.created_at.format("%Y-%m-%d %H:%M").to_string();

    let popup_area = centered_rect(60, 30, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Delete Thought")
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let content_max = inner.width.saturating_sub(2) as usize;
    let truncated = if thought.content.len() > content_max {
        format!("{}...", &thought.content[..content_max.saturating_sub(3)])
    } else {
        thought.content.clone()
    };

    let lines = vec![
        Line::raw(""),
        Line::from(vec![
            Span::styled(date_str, Style::default().fg(Color::DarkGray)),
            Span::raw(" "),
            Span::raw(truncated),
        ]),
        Line::raw(""),
        Line::from(Span::styled(
            "Delete this thought? y/n",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

/// Render the fuzzy entity picker overlay.
fn render_entity_picker(app: &App, frame: &mut Frame, area: Rect) {
    let Mode::EntityPicker {
        ref input,
        ref matches,
        selected,
    } = app.mode
    else {
        return;
    };

    let popup_area = centered_rect(60, 70, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Filter by Entity (type to search, Enter to select, Esc to cancel)");

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::vertical([
        Constraint::Length(1), // input field
        Constraint::Length(1), // separator
        Constraint::Min(1),    // match list
    ])
    .split(inner);

    // Input field
    let input_line = Line::from(vec![Span::raw("> "), Span::raw(input.value().to_string())]);
    frame.render_widget(Paragraph::new(input_line), chunks[0]);

    // Match count
    let count_text = format!("{} matches", matches.len());
    frame.render_widget(
        Paragraph::new(count_text).style(Style::default().fg(Color::DarkGray)),
        chunks[1],
    );

    // Match list
    let items: Vec<ListItem> = matches
        .iter()
        .enumerate()
        .map(|(i, &entity_idx)| {
            let entity = &app.entities[entity_idx];
            let style = if i == selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(entity.canonical_name.clone(), style))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, chunks[2]);

    // Set cursor position for input
    let cursor_x = popup_area.x + 1 + 2 + input.value().len() as u16;
    let cursor_y = chunks[0].y;
    frame.set_cursor_position((cursor_x, cursor_y));
}

/// Render the entity description modal popup.
fn render_entity_detail(app: &App, frame: &mut Frame, area: Rect) {
    let Mode::EntityDetail {
        ref entity_indices,
        scroll_offset,
    } = app.mode
    else {
        return;
    };

    let popup_area = centered_rect(70, 70, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Entity Details (Esc to close, ↑↓ to scroll)");

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let mut lines: Vec<Line> = Vec::new();

    for &idx in entity_indices {
        let entity = &app.entities[idx];

        // Entity name header
        let color = entity_color(&entity.name);
        lines.push(Line::from(Span::styled(
            entity.canonical_name.clone(),
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )));

        // Description or placeholder
        if let Some(ref desc) = entity.description {
            // Render description with entity highlighting
            for paragraph in desc.split("\n\n") {
                let content_line = styled_content_line(paragraph, inner.width as usize);
                lines.push(content_line);
                lines.push(Line::raw(""));
            }
        } else {
            lines.push(Line::from(Span::styled(
                "No description available",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            )));
            lines.push(Line::raw(""));
        }
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));

    frame.render_widget(paragraph, inner);
}

/// Create a centered rectangle within the given area.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Entity, Thought};
    use chrono::Utc;
    use ratatui::{Terminal, backend::TestBackend};

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

    /// Render the app into a TestBackend and return the buffer text content.
    fn render_to_string(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(app, frame)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        let mut output = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                output.push(buffer[(x, y)].symbol().chars().next().unwrap_or(' '));
            }
            output.push('\n');
        }
        output
    }

    #[test]
    fn test_ansi_to_ratatui_color_cyan() {
        assert_eq!(ansi_to_ratatui_color(AnsiColors::Cyan), Color::Cyan);
    }

    #[test]
    fn test_ansi_to_ratatui_color_green() {
        assert_eq!(ansi_to_ratatui_color(AnsiColors::Green), Color::Green);
    }

    #[test]
    fn test_ansi_to_ratatui_color_bright_red() {
        assert_eq!(ansi_to_ratatui_color(AnsiColors::BrightRed), Color::LightRed);
    }

    #[test]
    fn test_ansi_to_ratatui_color_bright_cyan() {
        assert_eq!(ansi_to_ratatui_color(AnsiColors::BrightCyan), Color::LightCyan);
    }

    #[test]
    fn test_entity_color_consistent() {
        let color1 = entity_color("Sarah");
        let color2 = entity_color("Sarah");
        assert_eq!(color1, color2);
    }

    #[test]
    fn test_entity_color_case_insensitive() {
        let color1 = entity_color("Sarah");
        let color2 = entity_color("sarah");
        assert_eq!(color1, color2);
    }

    #[test]
    fn test_styled_content_line_plain_text() {
        let line = styled_content_line("plain text", 80);
        assert_eq!(line.spans.len(), 1);
        assert_eq!(line.spans[0].content, "plain text");
    }

    #[test]
    fn test_styled_content_line_with_entity() {
        let line = styled_content_line("hello [Sarah] world", 80);
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "hello ");
        assert_eq!(line.spans[1].content, "Sarah");
        assert_eq!(line.spans[2].content, " world");
        // Entity span should be bold
        assert!(line.spans[1].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_styled_content_line_with_aliased_entity() {
        let line = styled_content_line("the [ML](machine-learning) course", 80);
        assert_eq!(line.spans.len(), 3);
        assert_eq!(line.spans[0].content, "the ");
        assert_eq!(line.spans[1].content, "ML"); // Displays alias
        assert_eq!(line.spans[2].content, " course");
    }

    // === Render tests using TestBackend ===

    #[test]
    fn test_render_empty_thoughts_shows_placeholder() {
        let app = App::new(vec![], vec![]);
        let output = render_to_string(&app, 60, 10);
        assert!(output.contains("No thoughts recorded yet"));
    }

    #[test]
    fn test_render_empty_thoughts_with_filter_shows_filter_message() {
        let mut app = App::new(vec![], vec![]);
        app.active_filter = Some("Sarah".to_string());
        app.recompute_displayed_thoughts();
        let output = render_to_string(&app, 60, 10);
        assert!(output.contains("No thoughts referencing"));
        assert!(output.contains("Sarah"));
    }

    #[test]
    fn test_render_thoughts_shows_content() {
        let thoughts = vec![make_thought("Meeting with team", 0)];
        let app = App::new(thoughts, vec![]);
        let output = render_to_string(&app, 80, 10);
        assert!(output.contains("Meeting with team"));
    }

    #[test]
    fn test_render_status_bar_shows_sort_order() {
        let app = App::new(vec![], vec![]);
        let output = render_to_string(&app, 80, 10);
        assert!(output.contains("Sort: Oldest first"));
    }

    #[test]
    fn test_render_status_bar_shows_key_hints() {
        let app = App::new(vec![], vec![]);
        let output = render_to_string(&app, 80, 10);
        assert!(output.contains("q:Quit"));
    }

    #[test]
    fn test_render_status_bar_shows_active_filter() {
        let thoughts = vec![make_thought("[Sarah] hello", 0)];
        let mut app = App::new(thoughts, vec![]);
        app.active_filter = Some("Sarah".to_string());
        app.recompute_displayed_thoughts();
        let output = render_to_string(&app, 80, 10);
        assert!(output.contains("Filter: Sarah"));
        assert!(output.contains("Esc to clear"));
    }

    #[test]
    fn test_render_filtered_title() {
        let thoughts = vec![make_thought("[Sarah] hello", 0)];
        let mut app = App::new(thoughts, vec![]);
        app.active_filter = Some("Sarah".to_string());
        app.recompute_displayed_thoughts();
        let output = render_to_string(&app, 80, 10);
        assert!(output.contains("filtered: Sarah"));
    }

    #[test]
    fn test_render_descending_sort_label() {
        let mut app = App::new(vec![], vec![]);
        app.sort_order.toggle();
        let output = render_to_string(&app, 80, 10);
        assert!(output.contains("Sort: Newest first"));
    }

    #[test]
    fn test_render_entity_picker_overlay() {
        let entities = vec![make_entity("Sarah", None), make_entity("Project", None)];
        let mut app = App::new(vec![], entities);
        app.mode = Mode::EntityPicker {
            input: tui_input::Input::default(),
            matches: vec![0, 1],
            selected: 0,
        };
        let output = render_to_string(&app, 80, 24);
        assert!(output.contains("Filter by Entity"));
        assert!(output.contains("Sarah"));
        assert!(output.contains("Project"));
        assert!(output.contains("2 matches"));
    }

    #[test]
    fn test_render_entity_detail_with_description() {
        let entities = vec![make_entity("Sarah", Some("A colleague from work"))];
        let mut app = App::new(vec![], entities);
        app.mode = Mode::EntityDetail {
            entity_indices: vec![0],
            scroll_offset: 0,
        };
        let output = render_to_string(&app, 80, 24);
        assert!(output.contains("Entity Details"));
        assert!(output.contains("Sarah"));
        assert!(output.contains("A colleague from work"));
    }

    #[test]
    fn test_render_entity_detail_without_description() {
        let entities = vec![make_entity("Sarah", None)];
        let mut app = App::new(vec![], entities);
        app.mode = Mode::EntityDetail {
            entity_indices: vec![0],
            scroll_offset: 0,
        };
        let output = render_to_string(&app, 80, 24);
        assert!(output.contains("No description available"));
    }

    #[test]
    fn test_render_entity_detail_multiple_entities() {
        let entities = vec![
            make_entity("Sarah", Some("A person")),
            make_entity("Project", Some("A big project")),
        ];
        let mut app = App::new(vec![], entities);
        app.mode = Mode::EntityDetail {
            entity_indices: vec![0, 1],
            scroll_offset: 0,
        };
        let output = render_to_string(&app, 80, 24);
        assert!(output.contains("Sarah"));
        assert!(output.contains("A person"));
        assert!(output.contains("Project"));
        assert!(output.contains("A big project"));
    }

    #[test]
    fn test_render_multiple_thoughts() {
        let thoughts = vec![
            make_thought("First thought", 2),
            make_thought("Second thought", 1),
            make_thought("Third thought", 0),
        ];
        let app = App::new(thoughts, vec![]);
        let output = render_to_string(&app, 80, 10);
        assert!(output.contains("First thought"));
        assert!(output.contains("Second thought"));
        assert!(output.contains("Third thought"));
    }

    #[test]
    fn test_render_thought_shows_date() {
        let thoughts = vec![make_thought("dated thought", 0)];
        let app = App::new(thoughts, vec![]);
        let output = render_to_string(&app, 80, 10);
        let today = Utc::now().format("%Y-%m-%d").to_string();
        assert!(output.contains(&today));
    }

    #[test]
    fn test_centered_rect_produces_smaller_rect() {
        let area = Rect::new(0, 0, 100, 50);
        let result = centered_rect(60, 70, area);
        assert!(result.width < area.width);
        assert!(result.height < area.height);
        assert!(result.x > 0);
        assert!(result.y > 0);
    }

    #[test]
    fn test_render_entity_detail_with_scroll() {
        let long_desc = (0..20)
            .map(|i| format!("Paragraph {}", i))
            .collect::<Vec<_>>()
            .join("\n\n");
        let entities = vec![make_entity("Sarah", Some(&long_desc))];
        let mut app = App::new(vec![], entities);
        app.mode = Mode::EntityDetail {
            entity_indices: vec![0],
            scroll_offset: 5,
        };
        // Should not panic with scroll offset
        let _output = render_to_string(&app, 80, 24);
    }

    #[test]
    fn test_render_confirm_delete_overlay() {
        let thoughts = vec![make_thought("Meeting with team", 0)];
        let mut app = App::new(thoughts, vec![]);
        app.mode = Mode::ConfirmDelete { thought_index: 0 };
        let output = render_to_string(&app, 80, 24);
        assert!(output.contains("Delete Thought"));
        assert!(output.contains("Delete this thought? y/n"));
        assert!(output.contains("Meeting with team"));
    }

    #[test]
    fn test_render_entity_picker_with_selected_highlight() {
        let entities = vec![
            make_entity("Alpha", None),
            make_entity("Beta", None),
            make_entity("Gamma", None),
        ];
        let mut app = App::new(vec![], entities);
        app.mode = Mode::EntityPicker {
            input: tui_input::Input::default(),
            matches: vec![0, 1, 2],
            selected: 1,
        };
        // Should render without panic; the selected item gets REVERSED style
        let output = render_to_string(&app, 80, 24);
        assert!(output.contains("Alpha"));
        assert!(output.contains("Beta"));
        assert!(output.contains("Gamma"));
        assert!(output.contains("3 matches"));
    }
}
