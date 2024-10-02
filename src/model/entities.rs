use std::fmt::Formatter;

pub type EntityId = String;

#[derive(Debug, PartialEq)]
pub struct Entity {
    pub raw: String,
}

impl Entity {
    pub fn from_raw(raw: String) -> Self {
        Entity { raw }
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}
