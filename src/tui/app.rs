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
use std::io::Write;
#[cfg(test)]
use std::vec::IntoIter;
#[cfg(test)]
use chrono::NaiveDate;
use crate::model::entities::Id as EntityId;
use crate::model::thoughts::{Fragment, Thought};

#[derive(Default)]
pub struct Thoughts {
    should_exit: bool,

    view: ThoughtsList,
}

impl Thoughts {}

impl Thoughts {
    pub fn interactive(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?
        }
        Ok(())
    }

    pub fn noninteractive(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.view.interactive = false;
        terminal.draw(|frame| self.draw(frame))?;
        Ok(())
    }

    pub fn raw(&mut self) -> io::Result<()> {
        self.view.interactive = false;
        self.view.raw();
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

    pub fn populated(thoughts: Vec<Thought>) -> Self {
        Self {
            view: ThoughtsList::populated(thoughts),
            should_exit: false,
        }
    }
}

impl Widget for &mut Thoughts {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.view.render(area, buf)
    }
}

fn make_raw_item(thought: &Thought) -> String {
    let added = thought.added.format("%Y %b %d").to_string();

    let mut line = added + " > ";
    for fragment in thought.fragments.iter() {
        match fragment {
            Fragment::Plain { text } => { line = line + text }
            Fragment::EntityRef { under, .. } => { line = line + under }
        };
    }

    line
}

fn make_list_item<'a>(thought: &'a Thought, colorizer: &mut EntityColorizer) -> ListItem<'a> {
    let added = thought.added.format("%Y %b %d").to_string();

    let mut items = vec![
        Span::styled(added, ORANGE.c500),
        Span::from(" > "),
    ];

    for fragment in thought.fragments.iter() {
        let span = match fragment {
            Fragment::Plain { text } => { Span::from(text) }
            Fragment::EntityRef { entity, under, .. } => {
                Span::styled(under, colorizer.assign_color(EntityId::from(entity)))
            }
        };
        items.push(span);
    }
    ListItem::new(Line::from(items))
}

#[cfg(test)]
#[test]
fn plain_fragment_match() {
    let thought = Thought {
        raw: "raw".to_string(),
        added: NaiveDate::parse_from_str("2021-02-03", "%Y-%m-%d").unwrap(),
        fragments: vec![
            Fragment::Plain { text: String::from("raw") }
        ],
    };
    assert_eq!("2021 Feb 03 > raw", make_raw_item(&thought));
}

#[cfg(test)]
#[test]
fn entity_fragment_match() {
    let thought = Thought {
        raw: "[raw]".to_string(),
        added: NaiveDate::parse_from_str("2021-02-03", "%Y-%m-%d").unwrap(),
        fragments: vec![
            Fragment::EntityRef {
                entity: String::from("raw"),
                under: String::from("raw"),
                raw: String::from("[raw]"),
            }
        ],
    };
    assert_eq!("2021 Feb 03 > raw", make_raw_item(&thought));
}

#[cfg(test)]
#[test]
fn aliased_entity_fragment_match() {
    let thought = Thought {
        raw: "[raw](entity)".to_string(),
        added: NaiveDate::parse_from_str("2021-02-03","%Y-%m-%d").unwrap(),
        fragments: vec![
            Fragment::EntityRef {
                entity: String::from("entity"),
                under: String::from("raw"),
                raw: String::from("[raw](entity)"),
            }
        ],
    };
    assert_eq!("2021 Feb 03 > raw", make_raw_item(&thought));
}

#[cfg(test)]
#[test]
fn combined_fragments_match() {
    let thought = Thought {
        raw: "[a](b) c [d]".to_string(),
        added: NaiveDate::parse_from_str("2021-02-03","%Y-%m-%d").unwrap(),
        fragments: vec![
            Fragment::EntityRef { entity: String::from("b"), under: String::from("a"), raw: String::from("[a](b)") },
            Fragment::Plain { text: String::from(" c ") },
            Fragment::EntityRef { entity: String::from("d"), under: String::from("d"), raw: String::from("[d]") }
        ],
    };
    assert_eq!("2021 Feb 03 > a c d", make_raw_item(&thought));
}


#[derive(Default)]
struct ThoughtsList {
    thoughts: Vec<Thought>,
    thoughts_tui: ListState,

    entity_colorizer: EntityColorizer,

    interactive: bool,
}


impl ThoughtsList {
    fn populated(thoughts: Vec<Thought>) -> Self {
        Self {
            thoughts,
            thoughts_tui: ListState::default().with_selected(Some(0)),
            entity_colorizer: EntityColorizer::default(),
            interactive: true,
        }
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
                make_list_item(thought, &mut self.entity_colorizer)
            }).collect();

        let list = if self.interactive {
            List::new(items)
                .highlight_symbol("* ")
                .highlight_spacing(HighlightSpacing::Always)
        } else {
            List::new(items)
        };

        StatefulWidget::render(list, area, buf, &mut self.thoughts_tui);
    }

    fn raw(&mut self) {
        for item in self.thoughts.iter() {
            println!("{}", make_raw_item(item));
        }
        io::stdout().flush().unwrap();
    }
}

#[cfg(test)]
fn short_thoughts() -> IntoIter<Thought> {
    let v = vec![
        Thought {
            raw: String::from("[Entity] does [Something] with [ActuallyEntity](Entity)"),
            added: NaiveDate::parse_from_str("2023-08-23", "%Y-%m-%d").unwrap(),
            fragments: vec![
                Fragment::EntityRef {
                    raw: String::from("[Entity]"),
                    entity: String::from("Entity"),
                    under: String::from("Entity")
                },
                Fragment::Plain { text: String::from(" does ") },
                Fragment::EntityRef {
                    raw: String::from("[Something]"),
                    entity: String::from("Something"),
                    under: String::from("Something")
                },
                Fragment::Plain { text: String::from(" with ") },
                Fragment::EntityRef {
                    raw: String::from("[ActuallyEntity](Entity)"),
                    entity: String::from("Entity"),
                    under: String::from("ActuallyEntity")
                }
            ],
        },
        Thought {
            raw: String::from("[Entity] is not [another entity](Another Entity)"),
            added: NaiveDate::parse_from_str("2024-09-24", "%Y-%m-%d").unwrap(),
            fragments: vec![
                Fragment::EntityRef {
                    raw: String::from("Entity"),
                    entity: String::from("Entity"),
                    under: String::from("Entity")
                },
                Fragment::Plain { text: String::from(" is not ") },
                Fragment::EntityRef {
                    raw: String::from("[another entity](Another Entity"),
                    entity: String::from("Another Entity"),
                    under: String::from("another entity"),
                },
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
            added: NaiveDate::parse_from_str("2024-10-01", "%Y-%m-%d").unwrap(),
            fragments: vec![Fragment::Plain { text: String::from("First thought") }],
        },
        Thought {
            raw: String::from("Second thought"),
            added: NaiveDate::parse_from_str("2024-10-02", "%Y-%m-%d").unwrap(),
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
        let line1 = "  2024 Oct 01 > First thought";
        let line1_selected = "* 2024 Oct 01 > First thought";

        let line2 = "  2024 Oct 02 > Second thought";
        let line2_selected = "* 2024 Oct 02 > Second thought";

        let mut expected_start = Buffer::with_lines(vec![line1_selected, line2]);
        let mut expected_after_next = Buffer::with_lines(vec![line1, line2_selected]);

        let date_style = Style::from(ORANGE.c500);
        expected_start.set_style(Rect::new(2, 0, 11, 2), date_style);
        expected_after_next.set_style(Rect::new(2, 0, 11, 2), date_style);

        let mut tl = ThoughtsList::populated(no_ref_thoughts().collect());
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 2));

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
        let mut tl = ThoughtsList::populated(short_thoughts().collect());
        let mut buf = Buffer::empty(Rect::new(0, 0, 59, 2));

        tl.render(buf.area, &mut buf);

        let line1 = "* 2023 Aug 23 > Entity does Something with ActuallyEntity  ";
        let line2 = "  2024 Sep 24 > Entity is not another entity               ";

        let mut expected = Buffer::with_lines(vec![line1, line2]);

        let date_style = Style::from(ORANGE.c500);
        expected.set_style(Rect::new(2, 0, 11, 2), date_style);

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
        let line1 = "  2024 Oct 01 > First thought";
        let line1_selected = "* 2024 Oct 01 > First thought";

        let line2 = "  2024 Oct 02 > Second thought";
        let line2_selected = "* 2024 Oct 02 > Second thought";

        let mut expected_start = Buffer::with_lines(vec![line1_selected, line2]);
        let mut expected_after_next = Buffer::with_lines(vec![line1, line2_selected]);

        let date_style = Style::from(ORANGE.c500);
        expected_start.set_style(Rect::new(2, 0, 11, 2), date_style);
        expected_after_next.set_style(Rect::new(2, 0, 11, 2), date_style);

        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 2));

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
