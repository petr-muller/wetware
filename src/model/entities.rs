use std::fmt::Formatter;

pub type Id = String;

#[derive(Debug, PartialEq)]
pub struct Entity {
    pub raw: String,
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}
