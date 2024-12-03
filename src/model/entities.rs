use crate::model::fragments;
use crate::model::thoughts::Error;
use std::fmt::Formatter;

pub type Id = String;

#[derive(Clone, Debug, PartialEq)]
pub struct RawEntity {
    pub name: String,
    pub description: String,
}

impl std::fmt::Display for RawEntity {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

type Result<T> = std::result::Result<T, Error>;

impl RawEntity {
    pub fn as_entity(&self) -> Result<Entity> {
        let entity = Entity {
            name: Id::from(self.name.clone()),
            description: fragments::String::parse(&self.description)?,
        };
        Ok(entity)
    }
}

pub struct Entity {
    pub name: Id,
    pub description: fragments::String,
}
