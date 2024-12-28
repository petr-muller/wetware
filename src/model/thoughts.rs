#[cfg(test)]
use crate::model::fragments::Fragment;
use crate::model::{entities, fragments};
use chrono::NaiveDate;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone)]
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

pub struct AddedThought {
    pub id: u32,
    pub thought: Thought,
    pub new_entities: Vec<entities::Id>,
}
