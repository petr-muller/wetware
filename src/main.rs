use chrono::{DateTime, Utc};
use clap::{Args, command, Parser, Subcommand};
use rusqlite::{params, params_from_iter};
use crate::thoughts::thought_lexer::{ThoughtLexer, TokenValue};

#[derive(Debug, Parser)]
#[clap(name = "wet", version)]
pub struct Wet {
    #[clap(flatten)]
    globals: GlobalFlags,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a new thought
    #[command(name = "add", arg_required_else_help = true)]
    Add {
        /// The thought to add
        thought: String,
        #[arg(short, long)]
        datetime: Option<DateTime<Utc>>,
    },
    /// List thoughts
    #[command(name = "thoughts")]
    Thoughts {
        #[arg(long = "on")]
        entity: Option<String>,
    },
}


#[derive(Debug, Args)]
struct GlobalFlags {
    /// The path to the database
    #[arg(long, env = "WETWARE_DB_PATH", required(false))]
    db: Option<String>,
}

pub mod thoughts {
    // TODO(muller): As an exercise, I will implement a lexer manually using Eli Bendersky's blog post:
    //               https://eli.thegreenplace.net/2022/rewriting-the-lexer-benchmark-in-rust/
    //               Eventually I may want to use [Logos](https://github.com/maciejhirsz/logos)
    pub mod thought_lexer {
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

        pub struct ThoughtLexer<'source> {
            input: &'source str,
            iter: CharIndices<'source>,
            // c is the last char taken from iter
            c: char,
            // ci is the offset of c in the input
            ci: usize,
            // error is true iff lexer encoutered an error
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
                    // TokenValue::Error once an now we should terminate the iteration.
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

        pub fn tokenize(input: &str) -> Vec<Token> {
            let lex = ThoughtLexer::new(input);
            lex.collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::thoughts::thought_lexer::{ThoughtLexer, Token, TokenValue};
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
            let lex = ThoughtLexer::new("this is [broken");
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Wet::parse();

    let db = args.globals.db.unwrap_or_else(|| {
        eprintln!("No database path provided");
        std::process::exit(1);
    });

    match args.command {
        Commands::Thoughts { entity } => {
            let conn = rusqlite::Connection::open(db).unwrap();

            let mut stmt_lines = vec!["SELECT thought FROM thoughts"];
            let mut params = vec![];
            if let Some(entity) = entity {
                stmt_lines.append(&mut vec![
                    "JOIN thoughts_entities ON thoughts.id = thoughts_entities.thought_id",
                    "JOIN entities ON thoughts_entities.entity_id = entities.id",
                    "WHERE entities.name = ?1"]);
                params.push(entity)
            }
            stmt_lines.push("ORDER BY datetime");
            let mut stmt = conn.prepare(stmt_lines.join("\n").as_str())?;

            let rows = stmt.query_map(params_from_iter(params), |row| row.get::<usize, String>(0))?;
            for thought in rows {
                println!("{}", thought.unwrap());
            }
        }
        Commands::Add { thought, datetime } => {
            let conn = rusqlite::Connection::open(db).unwrap();

            conn.execute(
                "CREATE TABLE IF NOT EXISTS thoughts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    thought TEXT NOT NULL,
                    datetime INTEGER NOT NULL
                    )",
                (),
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS entities (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEST NOT NULL UNIQUE
                    )",
                (),
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS thoughts_entities (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    thought_id INTEGER,
                    entity_id INTEGER,
                    FOREIGN KEY(thought_id) REFERENCES thoughts(id),
                    FOREIGN KEY(entity_id) REFERENCES entities(id),
                    UNIQUE(thought_id, entity_id)
                    )",
                (),
            )?;

            let now = datetime.unwrap_or_else(chrono::offset::Utc::now);

            let lex = ThoughtLexer::new(thought.as_str());
            let mut entities: Vec<&str> = vec![];
            for token in lex {
                match token.value {
                    TokenValue::Error => {
                        eprintln!("Error adding thought: {}", thought);
                        eprintln!("                      {}^ invalid token at position {}", " ".repeat(token.position), token.position);
                        std::process::exit(1);
                    }
                    TokenValue::EntityReference(entity) => {
                        entities.push(entity)
                    }
                    _ => {}
                }
            }

            conn.execute(
                "INSERT INTO thoughts (thought, datetime) VALUES (?1, ?2)",
                params![&thought, &now],
            )?;
            let thought_id = conn.last_insert_rowid();

            for entity in entities {
                let entity_name = &entity[1..entity.len() - 1];
                conn.execute(
                    "INSERT INTO entities (name) VALUES (?1)
                    ON CONFLICT(name) DO NOTHING",
                    params![entity_name],
                )?;
                let mut stmt = conn.prepare("SELECT id FROM entities WHERE name=?1")?;
                let mut rows = stmt.query_map(params![entity_name], |row| row.get::<usize, usize>(0))?;
                let entity_id = rows.next().unwrap().unwrap();
                conn.execute(
                    "INSERT INTO thoughts_entities (thought_id, entity_id) VALUES (?1, ?2)",
                    params![thought_id, entity_id],
                )?;
            }
        }
    }
    Ok(())
}
