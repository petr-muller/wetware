use crate::model::entities;
use crate::model::fragments::lexer::{ThoughtLexer, TokenValue};
use crate::model::fragments::Fragment::{EntityRef, Plain};
use crate::model::thoughts::Error;

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

// TODO(muller): As an exercise, I will implement a lexer manually using Eli Bendersky's blog post:
//               https://eli.thegreenplace.net/2022/rewriting-the-lexer-benchmark-in-rust/
//               Eventually I may want to use [Logos](https://github.com/maciejhirsz/logos)
pub(crate) mod lexer {
    #[derive(Debug, PartialEq)]
    pub struct EntityReference<'source> {
        /// entity refers to the part of the token identifying the actual referred entity
        pub entity: &'source str,
        /// under refers to the part of the token with the displayed string
        pub under: &'source str,
        /// raw refers to the full token
        pub raw: &'source str,
    }

    #[derive(Debug, PartialEq)]
    pub enum TokenValue<'source> {
        EOF,
        Error,

        Text(&'source str),
        EntityReference(EntityReference<'source>),
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

        /// assuming current character is a '(' immediately after a bare entity reference, consume
        /// all characters until a matching ')' character, or a character that should result in
        /// failure to parse an aliased entity reference: `[alias](entity)`
        fn consume_entity_reference_for_alias(&mut self) {
            self.scan_char();
            while !(self.is_at_end() || self.c == ')' || self.c == '(') {
                self.scan_char();
            }
        }

        fn scan_entity_reference(&mut self) -> Token<'source> {
            let start = self.ci;
            self.scan_char();
            while !(self.is_at_end() || self.c == ']' || self.c == '[') {
                self.scan_char();
            }
            match self.c {
                ']' => {
                    self.scan_char();
                    if self.c == '(' {
                        let alias_end = self.ci - 1;
                        self.scan_char();
                        let entity_start = self.ci;

                        self.consume_entity_reference_for_alias();
                        match self.c {
                            ')' => {
                                self.scan_char();
                                Token {
                                    value: TokenValue::EntityReference(EntityReference {
                                        entity: &self.input[entity_start..self.ci - 1],
                                        under: &self.input[start + 1..alias_end],
                                        raw: &self.input[start..self.ci],
                                    }),
                                    position: start,
                                }
                            }
                            '(' => self.error_token(self.ci),
                            _ => self.error_token(entity_start),
                        }
                    } else {
                        Token {
                            value: TokenValue::EntityReference(EntityReference {
                                entity: &self.input[start + 1..self.ci - 1],
                                under: &self.input[start + 1..self.ci - 1],
                                raw: &self.input[start..self.ci],
                            }),
                            position: start,
                        }
                    }
                }
                '[' => self.error_token(self.ci),
                _ => self.error_token(start),
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

#[derive(Debug, PartialEq, Clone)]
pub enum Fragment {
    Plain {
        text: std::string::String,
    },
    EntityRef {
        entity: entities::Id,
        under: std::string::String,
        raw: std::string::String,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct String {
    pub raw: std::string::String,
    pub fragments: Vec<Fragment>,
}

type Result<T> = std::result::Result<T, Error>;

impl String {
    pub fn parse(input: &str) -> Result<Self> {
        let lex = ThoughtLexer::new(input);
        let mut fragments = vec![];
        for token in lex {
            match token.value {
                TokenValue::Error => {
                    return Err(Error {
                        message: format!("invalid token at position {}", token.position),
                    });
                }
                TokenValue::EntityReference(entity_ref) => {
                    fragments.push(EntityRef {
                        entity: entity_ref.entity.into(),
                        under: entity_ref.under.into(),
                        raw: entity_ref.raw.into(),
                    });
                }
                TokenValue::Text(text) => fragments.push(Plain { text: text.into() }),
                _ => {}
            }
        }

        Ok(Self {
            raw: input.to_owned(),
            fragments,
        })
    }
}
