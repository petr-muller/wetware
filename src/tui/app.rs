use crate::tui::entity_colorizer::EntityColorizer;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
#[allow(unused_imports)]
use ratatui::prelude::{Line, Span, StatefulWidget, Stylize, Widget};
use ratatui::style::palette::tailwind::ORANGE;
use ratatui::widgets::{HighlightSpacing, List, ListItem, ListState};
use ratatui::{DefaultTerminal, Frame};
use std::io;

#[allow(unused_imports)]
use std::ops::Sub;
#[allow(unused_imports)]
use std::vec::IntoIter;
#[allow(unused_imports)]
use chrono::{DateTime, Local};
use crate::model::entities::Id as EntityId;
use crate::model::thoughts::{Fragment, Thought};

#[derive(Default)]
pub struct Thoughts {
    should_exit: bool,

    view: ThoughtsList,
}

impl Thoughts {
    /// runs application main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // Important to check that the event is a key press event as crossterm also emits
            // key release and repeat events on Windows
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }

    fn select_next(&mut self) {
        self.view.select_next()
    }

    fn select_previous(&mut self) {
        self.view.select_previous()
    }

    #[cfg(test)]
    fn populated(thoughts: Vec<Thought>) -> Self {
        Self {
            view: ThoughtsList::populated(thoughts).in_utc(),
            should_exit: false,
        }
    }
}

impl Widget for &mut Thoughts {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.view.render(area, buf)
    }
}

fn make_list_item<'a>(thought: &'a Thought, colorizer: &mut EntityColorizer, keep_utc: bool) -> ListItem<'a> {
    let added = if keep_utc {
        thought.added.format("%Y %b %d %H:%M").to_string()
    } else {
        thought.added.with_timezone(&Local).format("%Y %b %d %H:%M").to_string()
    };
    let mut items = vec![
        Span::styled(added, ORANGE.c500),
        Span::from(" > "),
    ];

    for fragment in thought.fragments.iter() {
        let span = match fragment {
            Fragment::Plain { text } => { Span::from(text) }
            Fragment::EntityRef { entity, raw } => {
                Span::styled(raw, colorizer.assign_color(EntityId::from(entity)))
            }
        };
        items.push(span);
    }
    ListItem::new(Line::from(items))
}


#[derive(Default)]
struct ThoughtsList {
    thoughts: Vec<Thought>,
    thoughts_tui: ListState,

    entity_colorizer: EntityColorizer,

    keep_utc: bool,
}


impl ThoughtsList {
    #[cfg(test)]
    fn populated(thoughts: Vec<Thought>) -> Self {
        Self {
            thoughts,
            thoughts_tui: ListState::default().with_selected(Some(0)),
            entity_colorizer: EntityColorizer::default(),
            keep_utc: false,
        }
    }

    #[cfg(test)]
    fn in_utc(self) -> Self {
        let mut tl = self;
        tl.keep_utc = true;
        tl
    }

    fn select_next(&mut self) {
        self.thoughts_tui.select_next()
    }

    fn select_previous(&mut self) {
        self.thoughts_tui.select_previous()
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .thoughts
            .iter()
            .map(|thought| {
                make_list_item(thought, &mut self.entity_colorizer, self.keep_utc)
            }).collect();

        let list = List::new(items)
            .highlight_symbol("* ")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.thoughts_tui);
    }
}

#[cfg(test)]
fn short_thoughts() -> IntoIter<Thought> {
    let v = vec![
        Thought {
            raw: String::from("[Entity] does [Something] with [Entity]"),
            added: DateTime::parse_from_rfc3339("2024-09-24T00:23:00+02:00").unwrap().to_utc(),
            fragments: vec![
                Fragment::EntityRef { raw: String::from("Entity"), entity: String::from("Entity") },
                Fragment::Plain { text: String::from(" does ") },
                Fragment::EntityRef { raw: String::from("Something"), entity: String::from("Something") },
                Fragment::Plain { text: String::from(" with ") },
                Fragment::EntityRef { raw: String::from("ActuallyEntity"), entity: String::from("Entity") }
            ],
        },
        Thought {
            raw: String::from("[Entity] is not [Another Entity]"),
            added: DateTime::parse_from_rfc3339("2024-09-24T00:25:00+02:00").unwrap().to_utc(),
            fragments: vec![
                Fragment::EntityRef { raw: String::from("Entity"), entity: String::from("Entity") },
                Fragment::Plain { text: String::from(" is not ") },
                Fragment::EntityRef { raw: String::from("another entity"), entity: String::from("Another Entity") },
            ]
        },
    ];
    v.into_iter()
}

#[cfg(test)]
fn no_ref_thoughts() -> IntoIter<Thought> {
    let v = vec![
        Thought {
            raw: String::from("First thought"),
            added: DateTime::parse_from_rfc3339("2024-10-01T00:11:00+02:00").unwrap().to_utc(),
            fragments: vec![Fragment::Plain { text: String::from("First thought") }],
        },
        Thought {
            raw: String::from("Second thought"),
            added: DateTime::parse_from_rfc3339("2024-10-01T00:12:00+02:00").unwrap().to_utc(),
            fragments: vec![Fragment::Plain { text: String::from("Second thought") }],
        },
    ];
    v.into_iter()
}

#[cfg(test)]
mod thoughts_list_tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::palette::tailwind::ORANGE;
    use super::{no_ref_thoughts, short_thoughts, ThoughtsList};

    use ratatui::style::Style;

    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use crate::model::entities::Id;

    #[test]
    fn render_simple_thoughts() {
        let line1 = "  2024 Sep 30 22:11 > First thought";
        let line1_selected = "* 2024 Sep 30 22:11 > First thought";

        let line2 = "  2024 Sep 30 22:12 > Second thought";
        let line2_selected = "* 2024 Sep 30 22:12 > Second thought";

        let mut expected_start = Buffer::with_lines(vec![line1_selected, line2]);
        let mut expected_after_next = Buffer::with_lines(vec![line1, line2_selected]);

        let date_style = Style::from(ORANGE.c500);
        expected_start.set_style(Rect::new(2, 0, 17, 2), date_style);
        expected_after_next.set_style(Rect::new(2, 0, 17, 2), date_style);

        let mut tl = ThoughtsList::populated(no_ref_thoughts().collect()).in_utc();
        let mut buf = Buffer::empty(Rect::new(0, 0, 36, 2));

        tl.render(buf.area, &mut buf);
        assert_eq!(expected_start, buf);

        tl.select_previous();
        tl.render(buf.area, &mut buf);
        assert_eq!(expected_start, buf);

        tl.select_next();
        tl.render(buf.area, &mut buf);
        assert_eq!(expected_after_next, buf);

        tl.select_next();
        tl.render(buf.area, &mut buf);
        assert_eq!(expected_after_next, buf);

        tl.select_previous();
        tl.render(buf.area, &mut buf);
        assert_eq!(expected_start, buf);
    }

    #[test]
    fn render_thoughts_with_entities() {
        let mut tl = ThoughtsList::populated(short_thoughts().collect()).in_utc();
        let mut buf = Buffer::empty(Rect::new(0, 0, 65, 2));

        tl.render(buf.area, &mut buf);

        let line1 = "* 2024 Sep 23 22:23 > Entity does Something with ActuallyEntity  ";
        let line2 = "  2024 Sep 23 22:25 > Entity is not another entity               ";

        let mut expected = Buffer::with_lines(vec![line1, line2]);

        let date_style = Style::from(ORANGE.c500);
        expected.set_style(Rect::new(2, 0, 17, 2), date_style);

        let entity_style = Style::from(tl.entity_colorizer.assign_color(Id::from("Entity")));

        expected.set_style(Rect::new(line1.find("Entity").unwrap() as u16, 0, "Entity".len() as u16, 1), entity_style);
        expected.set_style(Rect::new(line1.find("ActuallyEntity").unwrap() as u16, 0, "ActuallyEntity".len() as u16, 1), entity_style);
        expected.set_style(Rect::new(line2.find("Entity").unwrap() as u16, 1, "Entity".len() as u16, 1), entity_style);

        let something_style = Style::from(tl.entity_colorizer.assign_color(Id::from("Something")));
        expected.set_style(Rect::new(line1.find("Something").unwrap() as u16, 0, "Something".len() as u16, 1), something_style);

        let another_entity_style = Style::from(tl.entity_colorizer.assign_color(Id::from("Another Entity")));
        expected.set_style(Rect::new(line2.find("another entity").unwrap() as u16, 1, "another entity".len() as u16, 1), another_entity_style);

        assert_eq!(expected, buf);
    }
}

#[cfg(test)]
mod thoughts_tests {
    use std::io;
    use crossterm::event::KeyCode;
    use pretty_assertions::assert_eq;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::prelude::Style;
    use ratatui::style::palette::tailwind::ORANGE;
    use ratatui::widgets::Widget;
    use crate::tui::app::{no_ref_thoughts, Thoughts};

    #[test]
    fn render_simple_thoughts() -> io::Result<()> {
        let mut tui = Thoughts::populated(no_ref_thoughts().collect());

        let line1 = "  2024 Sep 30 22:11 > First thought";
        let line1_selected = "* 2024 Sep 30 22:11 > First thought";

        let line2 = "  2024 Sep 30 22:12 > Second thought";
        let line2_selected = "* 2024 Sep 30 22:12 > Second thought";

        let mut expected_start = Buffer::with_lines(vec![line1_selected, line2]);
        let mut expected_after_next = Buffer::with_lines(vec![line1, line2_selected]);

        let date_style = Style::from(ORANGE.c500);
        expected_start.set_style(Rect::new(2, 0, 17, 2), date_style);
        expected_after_next.set_style(Rect::new(2, 0, 17, 2), date_style);

        let mut buf = Buffer::empty(Rect::new(0, 0, 36, 2));

        tui.render(buf.area, &mut buf);
        assert_eq!(expected_start, buf);

        tui.handle_key_event(KeyCode::Char('k').into());
        tui.render(buf.area, &mut buf);
        assert_eq!(expected_start, buf);

        tui.handle_key_event(KeyCode::Char('j').into());
        tui.render(buf.area, &mut buf);
        assert_eq!(expected_after_next, buf);

        tui.handle_key_event(KeyCode::Char('j').into());
        tui.render(buf.area, &mut buf);
        assert_eq!(expected_after_next, buf);

        tui.handle_key_event(KeyCode::Char('k').into());
        tui.render(buf.area, &mut buf);
        assert_eq!(expected_start, buf);

        Ok(())
    }

    #[test]
    fn handle_key_event_quit() -> io::Result<()> {
        let mut tui = Thoughts::default();
        tui.handle_key_event(KeyCode::Char('q').into());
        assert!(tui.should_exit);

        Ok(())
    }
}