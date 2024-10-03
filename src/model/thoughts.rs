use std::fmt::Formatter;
use chrono::{DateTime, Utc};
use crate::model::thoughts::Fragment::{EntityRef, Plain};
use crate::model::thoughts::lexer::{ThoughtLexer, TokenValue};

// TODO(muller): As an exercise, I will implement a lexer manually using Eli Bendersky's blog post:
//               https://eli.thegreenplace.net/2022/rewriting-the-lexer-benchmark-in-rust/
//               Eventually I may want to use [Logos](https://github.com/maciejhirsz/logos)
mod lexer {
    #[derive(Debug, PartialEq)]
    pub enum TokenValue<'source> {
        EOF,
        Error,

        Text(&'source str),
        EntityReference(&'source str),
    }

    // Token is defined as a value at a byte offset in the input string
    #[derive(Debug, PartialEq)]
    pub struct Token<'source> {
        pub value: TokenValue<'source>,
        pub position: usize,
    }

    use core::str::CharIndices;

    pub(crate) struct ThoughtLexer<'source> {
        input: &'source str,
        iter: CharIndices<'source>,
        // c is the last char taken from iter
        c: char,
        // ci is the offset of c in the input
        ci: usize,
        // error is true iff lexer encountered an error
        error: bool,
    }

    impl<'source> ThoughtLexer<'source> {
        // Consumes the next char from source and sets `c` and `ci`. `c` is set to `\x00` as a sentinel value at the
        // end of input
        fn scan_char(&mut self) {
            if let Some((index, chr)) = self.iter.next() {
                self.ci = index;
                self.c = chr;
            } else {
                self.ci = self.input.len();
                self.c = '\x00';
            }
        }

        pub fn new(input: &'source str) -> Self {
            let mut lex = Self {
                input,
                iter: input.char_indices(),
                c: '\x00',
                ci: 0,
                error: false,
            };
            lex.scan_char();
            lex
        }

        fn is_at_end(&self) -> bool {
            self.ci >= self.input.len()
        }

        fn error_token(&mut self, position: usize) -> Token<'source> {
            self.error = true;
            Token {
                value: TokenValue::Error,
                position,
            }
        }

        fn scan_entity_reference(&mut self) -> Token<'source> {
            let start = self.ci;
            self.scan_char();
            while !(self.is_at_end() || self.c == ']' || self.c == '[') {
                self.scan_char();
            }
            match self.c {
                '[' => {
                    self.error_token(self.ci)
                }
                ']' => {
                    self.scan_char();
                    Token {
                        value: TokenValue::EntityReference(&self.input[start..self.ci]),
                        position: start,
                    }
                }
                _ => {
                    self.error_token(start)
                }
            }
        }

        fn scan_text(&mut self) -> Token<'source> {
            let start = self.ci;
            while !(self.is_at_end() || self.c == '[') {
                self.scan_char();
            }

            Token {
                value: TokenValue::Text(&self.input[start..self.ci]),
                position: start,
            }
        }

        // next_token is the "raw" API for Lexers. It yields the next token in the
        // input until it encounters the end, at which point it starts yielding
        // TokenValue::EOF. If it encounters an error, it will return
        // TokenValue::error and will continue returning it for subsequent calls.
        // See also the next() method for an Iterator-like interface.
        fn next_token(&mut self) -> Token<'source> {
            if self.is_at_end() {
                return Token {
                    value: TokenValue::EOF,
                    position: self.ci,
                };
            }

            if self.c == '[' {
                self.scan_entity_reference()
            } else {
                self.scan_text()
            }
        }
    }

    impl<'source> Iterator for ThoughtLexer<'source> {
        type Item = Token<'source>;
        fn next(&mut self) -> Option<Self::Item> {
            if self.error {
                // If an error was already been set before we invoke next_token, it means we have already returned
                // TokenValue::Error once, and now we should terminate the iteration.
                return None;
            }

            let token = self.next_token();
            if token.value == TokenValue::EOF {
                None
            } else {
                Some(token)
            }
        }
    }
}

#[cfg(test)]
mod lexer_tests {
    use crate::model::thoughts::lexer::{ThoughtLexer, Token, TokenValue};

    // Assert that 'token' has a certain value and optionally a position
    macro_rules! assert_token {
            ($tok:expr, $wantval:expr, $wantpos:expr) => {
                assert_eq!(
                    $tok,
                    Token {
                        value: $wantval,
                        position: $wantpos
                    }
                );
            };
            ($tok:expr, $wantval:expr) => {
                let tok = $tok;
                assert_eq!(
                    tok,
                    Token {
                        value: $wantval,
                        position: tok.position
                    }
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
        assert_token!(tokens[0], TokenValue::EntityReference("[some entity]"), 0);
    }

    #[test]
    fn thought_with_text_and_entities() {
        let lex = ThoughtLexer::new("[entity] acted and [another entity] hates that [entity] did that");
        let tokens: Vec<Token> = lex.collect();
        assert_eq!(tokens.len(), 6);
        assert_token!(tokens[0], TokenValue::EntityReference("[entity]"), 0);
        assert_token!(tokens[1], TokenValue::Text(" acted and "), 8);
        assert_token!(tokens[2], TokenValue::EntityReference("[another entity]"), 19);
        assert_token!(tokens[3], TokenValue::Text(" hates that "), 35);
        assert_token!(tokens[4], TokenValue::EntityReference("[entity]"), 47);
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
pub enum Fragment {
    Plain { text: String },
    EntityRef { entity: crate::model::entities::Id, raw: String },
}

#[derive(Debug, PartialEq)]
pub struct Thought {
    pub raw: String,
    pub added: DateTime<Utc>,

    pub fragments: Vec<Fragment>,
}

type Result<T> = std::result::Result<T, Error>;

impl Thought {
    pub fn from_input(raw: String, added: DateTime<Utc>) -> Result<Self> {
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
        write!(f, "SQLite store error: {}", self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}


#[cfg(test)]
mod thought_tests {
    use crate::model::thoughts::{Error, Fragment, RawThought, Thought};
    use crate::model::thoughts::Fragment::Plain;

    #[test]
    fn from_input_simple() -> Result<(), Error> {
        let added = chrono::offset::Utc::now();
        let thought = Thought::from_input("This is a thought".to_string(), added)?;

        assert_eq!(
            Thought {
                raw: "This is a thought".to_string(),
                fragments: vec![Plain { text: String::from("This is a thought") }
                ],
                added
            },
            thought
        );
        Ok(())
    }

    //noinspection DuplicatedCode
    #[test]
    fn from_input_with_entities() -> Result<(), Error> {
        let added = chrono::offset::Utc::now();
        let thought = Thought::from_input("This is a [thought] with [entity] about [thought]".to_string(), added)?;

        assert_eq!(
            Thought {
                raw: String::from("This is a [thought] with [entity] about [thought]"),
                fragments: vec![
                    Plain { text: String::from("This is a ") },
                    Fragment::EntityRef { entity: String::from("thought"), raw: String::from("thought") },
                    Plain { text: String::from(" with ") },
                    Fragment::EntityRef { entity: String::from("entity"), raw: String::from("entity") },
                    Plain { text: String::from(" about ") },
                    Fragment::EntityRef { entity: String::from("thought"), raw: String::from("thought") },
                ],
                added,
            },
            thought);
        Ok(())
    }

    #[test]
    fn from_store() -> Result<(), Error> {
        let added = chrono::offset::Utc::now();
        let simple = RawThought::from_store("This is a thought".to_string(), added);
        assert_eq!(
            RawThought {
                raw: "This is a thought".to_string(),
                added,
            },
            simple
        );

        let with_entities = RawThought::from_store("This is a [thought] with [entity] about [thought]".to_string(), added);
        assert_eq!(
                RawThought{
                    raw: "This is a [thought] with [entity] about [thought]".to_string(),
                    added,
                },
                with_entities,
            );
        Ok(())
    }
}


#[derive(Debug, PartialEq)]
pub struct RawThought {
    raw: String,
    added: DateTime<Utc>,
}

impl RawThought {
    pub fn from_store(raw: String, added: DateTime<Utc>) -> RawThought {
        RawThought { raw, added }
    }

    pub fn as_thought(&self) -> Result<Thought> {
        let lex = ThoughtLexer::new(self.raw.as_str());
        let mut fragments = vec![];
        for token in lex {
            match token.value {
                TokenValue::Error => {
                    return Err(Error { message: format!("invalid token at position {}", token.position) });
                }
                TokenValue::EntityReference(entity) => {
                    let entity_name = &entity[1..entity.len() - 1];
                    fragments.push(EntityRef { entity: entity_name.into(), raw: entity_name.into() });
                }
                TokenValue::Text(text) => {
                    fragments.push(Plain { text: text.into() })
                }
                _ => {}
            }
        }
        let thought = Thought {
            raw: self.raw.to_string(),
            added: self.added,
            fragments,
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
    use crate::model::thoughts::{Error, Fragment, RawThought, Thought};
    use crate::model::thoughts::Fragment::Plain;

    #[test]
    fn as_thought_simple() -> Result<(), Error> {
        let added = chrono::offset::Utc::now();
        let raw = RawThought {
            raw: "This is a thought".to_string(),
            added,
        };
        let thought = raw.as_thought()?;
        assert_eq!(
            Thought {
                raw: "This is a thought".to_string(),
                fragments: vec![
                    Plain { text: String::from("This is a thought") }
                ],
                added,
            },
            thought);
        Ok(())
    }

    #[test]
    fn as_thought_with_entities() -> Result<(), Error> {
        let added = chrono::offset::Utc::now();
        let raw = RawThought {
            raw: "This is a [thought] with [entity] about [thought]".to_string(),
            added,
        };
        let thought = raw.as_thought()?;
        assert_eq!(
            Thought {
                raw: "This is a [thought] with [entity] about [thought]".to_string(),
                fragments: vec![
                    Plain { text: String::from("This is a ") },
                    Fragment::EntityRef { entity: String::from("thought"), raw: String::from("thought") },
                    Plain { text: String::from(" with ") },
                    Fragment::EntityRef { entity: String::from("entity"), raw: String::from("entity") },
                    Plain { text: String::from(" about ") },
                    Fragment::EntityRef { entity: String::from("thought"), raw: String::from("thought") },
                ],
                added,
            },
            thought);
        Ok(())
    }
}
