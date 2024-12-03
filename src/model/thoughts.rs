use crate::model::fragments;
#[cfg(test)]
use crate::model::fragments::Fragment;
use chrono::NaiveDate;
use std::fmt::Formatter;

#[cfg(test)]
mod lexer_tests {
    use crate::model::fragments::lexer;
    use crate::model::fragments::lexer::TokenValue::EntityReference;
    use crate::model::fragments::lexer::{ThoughtLexer, Token, TokenValue};

    // Assert that 'token' has a certain value and optionally a position
    macro_rules! assert_token {
        ($tok:expr, $wantval:expr, $wantpos:expr) => {
            assert_eq!(
                Token {
                    value: $wantval,
                    position: $wantpos
                },
                $tok
            );
        };
        ($tok:expr, $wantval:expr) => {
            let tok = $tok;
            assert_eq!(
                Token {
                    value: $wantval,
                    position: tok.position
                },
                tok,
            );
        };
    }

    #[test]
    fn simple_text() {
        let lex = ThoughtLexer::new("thought lexer");
        let tokens: Vec<Token> = lex.collect();
        assert_eq!(tokens.len(), 1);
        assert_token!(tokens[0], TokenValue::Text("thought lexer"), 0);
    }

    #[test]
    fn simple_entity_reference() {
        let lex = ThoughtLexer::new("[some entity]");
        let tokens: Vec<Token> = lex.collect();
        assert_eq!(tokens.len(), 1);
        let expected = lexer::EntityReference {
            entity: "some entity",
            under: "some entity",
            raw: "[some entity]",
        };
        assert_token!(tokens[0], EntityReference(expected), 0);
    }

    #[test]
    fn aliased_entity_reference() {
        let lex = ThoughtLexer::new("[alias](entity)");
        let tokens: Vec<Token> = lex.collect();
        assert_eq!(tokens.len(), 1);
        let expected = lexer::EntityReference {
            entity: "entity",
            under: "alias",
            raw: "[alias](entity)",
        };
        assert_token!(tokens[0], EntityReference(expected), 0);
    }

    #[test]
    fn thought_with_text_and_entities() {
        let lex =
            ThoughtLexer::new("[entity] acted and [another entity] hates that [entity] did that");
        let tokens: Vec<Token> = lex.collect();
        assert_eq!(tokens.len(), 6);
        let expected = lexer::EntityReference {
            entity: "entity",
            under: "entity",
            raw: "[entity]",
        };
        assert_token!(tokens[0], EntityReference(expected), 0);
        assert_token!(tokens[1], TokenValue::Text(" acted and "), 8);

        let expected = lexer::EntityReference {
            entity: "another entity",
            under: "another entity",
            raw: "[another entity]",
        };
        assert_token!(tokens[2], EntityReference(expected), 19);
        assert_token!(tokens[3], TokenValue::Text(" hates that "), 35);

        let expected = lexer::EntityReference {
            entity: "entity",
            under: "entity",
            raw: "[entity]",
        };
        assert_token!(tokens[4], EntityReference(expected), 47);
        assert_token!(tokens[5], TokenValue::Text(" did that"), 55);
    }

    #[test]
    fn unclosed_entity_is_an_error() {
        let lex = ThoughtLexer::new("this is [wrong");
        let tokens: Vec<Token> = lex.collect();
        assert_eq!(tokens.len(), 2);
        assert_token!(tokens[0], TokenValue::Text("this is "), 0);
        assert_token!(tokens[1], TokenValue::Error, 8);
    }

    #[test]
    fn nested_entity_is_an_error() {
        let lex = ThoughtLexer::new("this is [[broken]]");
        let tokens: Vec<Token> = lex.collect();
        assert_eq!(tokens.len(), 2);
        assert_token!(tokens[0], TokenValue::Text("this is "), 0);
        assert_token!(tokens[1], TokenValue::Error, 9);
    }
}

#[derive(Debug, PartialEq)]
pub struct Thought {
    pub added: NaiveDate,
    pub text: fragments::String,
}

type Result<T> = std::result::Result<T, Error>;

impl Thought {
    pub fn from_input(raw: String, added: NaiveDate) -> Result<Self> {
        let raw_thought = RawThought { raw, added };
        let thought = raw_thought.as_thought()?;

        Ok(thought)
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Thought error: {}", self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod thought_tests {
    use crate::model::fragments;
    use crate::model::thoughts::Fragment::Plain;
    use crate::model::thoughts::{Error, Fragment, RawThought, Thought};

    #[test]
    fn from_input_simple() -> Result<(), Error> {
        let added = chrono::Local::now().date_naive();
        let thought = Thought::from_input("This is a thought".to_string(), added)?;

        assert_eq!(
            Thought {
                text: fragments::String {
                    raw: String::from("This is a thought"),
                    fragments: vec![Plain {
                        text: String::from("This is a thought")
                    }],
                },
                added
            },
            thought
        );
        Ok(())
    }

    //noinspection DuplicatedCode
    #[test]
    fn from_input_with_entities() -> Result<(), Error> {
        let added = chrono::Local::now().date_naive();
        let thought = Thought::from_input(
            "This is a [thought] with [entity] about [thought]".to_string(),
            added,
        )?;

        assert_eq!(
            Thought {
                text: fragments::String {
                    raw: String::from("This is a [thought] with [entity] about [thought]"),
                    fragments: vec![
                        Plain {
                            text: String::from("This is a ")
                        },
                        Fragment::EntityRef {
                            entity: String::from("thought"),
                            under: String::from("thought"),
                            raw: String::from("[thought]")
                        },
                        Plain {
                            text: String::from(" with ")
                        },
                        Fragment::EntityRef {
                            entity: String::from("entity"),
                            under: String::from("entity"),
                            raw: String::from("[entity]")
                        },
                        Plain {
                            text: String::from(" about ")
                        },
                        Fragment::EntityRef {
                            entity: String::from("thought"),
                            under: String::from("thought"),
                            raw: String::from("[thought]")
                        },
                    ]
                },
                added,
            },
            thought
        );
        Ok(())
    }

    #[test]
    fn from_store() -> Result<(), Error> {
        let added = chrono::Local::now().date_naive();
        let simple = RawThought::from_store("This is a thought".to_string(), added);
        assert_eq!(
            RawThought {
                raw: "This is a thought".to_string(),
                added,
            },
            simple
        );

        let with_entities = RawThought::from_store(
            "This is a [thought] with [entity] about [thought]".to_string(),
            added,
        );
        assert_eq!(
            RawThought {
                raw: "This is a [thought] with [entity] about [thought]".to_string(),
                added,
            },
            with_entities,
        );
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawThought {
    raw: String,
    added: NaiveDate,
}

impl RawThought {
    pub fn from_store(raw: String, added: NaiveDate) -> RawThought {
        RawThought { raw, added }
    }

    pub fn as_thought(&self) -> Result<Thought> {
        let thought = Thought {
            added: self.added,
            text: fragments::String::parse(&self.raw.to_string())?,
        };
        Ok(thought)
    }
}

impl std::fmt::Display for RawThought {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

#[cfg(test)]
mod raw_thought_tests {
    use crate::model::fragments;
    use crate::model::thoughts::Fragment::Plain;
    use crate::model::thoughts::{Error, Fragment, RawThought, Thought};

    #[test]
    fn as_thought_simple() -> Result<(), Error> {
        let added = chrono::Local::now().date_naive();
        let raw = RawThought {
            raw: "This is a thought".to_string(),
            added,
        };
        let thought = raw.as_thought()?;
        assert_eq!(
            Thought {
                text: fragments::String {
                    raw: "This is a thought".to_string(),
                    fragments: vec![Plain {
                        text: String::from("This is a thought")
                    }],
                },
                added,
            },
            thought
        );
        Ok(())
    }

    #[test]
    fn as_thought_with_entities() -> Result<(), Error> {
        let added = chrono::Local::now().date_naive();
        let raw = RawThought {
            raw: String::from("This is a [thought] with [entity] about [thought]"),
            added,
        };
        let thought = raw.as_thought()?;
        assert_eq!(
            Thought {
                text: fragments::String {
                    raw: "This is a [thought] with [entity] about [thought]".to_string(),
                    fragments: vec![
                        Plain {
                            text: String::from("This is a ")
                        },
                        Fragment::EntityRef {
                            entity: String::from("thought"),
                            under: String::from("thought"),
                            raw: String::from("[thought]")
                        },
                        Plain {
                            text: String::from(" with ")
                        },
                        Fragment::EntityRef {
                            entity: String::from("entity"),
                            under: String::from("entity"),
                            raw: String::from("[entity]")
                        },
                        Plain {
                            text: String::from(" about ")
                        },
                        Fragment::EntityRef {
                            entity: String::from("thought"),
                            under: String::from("thought"),
                            raw: String::from("[thought]")
                        },
                    ],
                },
                added,
            },
            thought
        );
        Ok(())
    }
}
