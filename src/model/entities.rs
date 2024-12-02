use std::fmt::Formatter;

pub type Id = String;

#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub name: String,
    pub description: String,
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
