use crate::thoughts::entities::EntityId;
use ratatui::style::Color;
use std::collections::HashMap;
use ratatui::style::palette::tailwind::{AMBER, BLUE, CYAN, EMERALD, FUCHSIA, GREEN, INDIGO, LIME, ORANGE, PINK, PURPLE, RED, SKY, TEAL, VIOLET, YELLOW};

/// Default color palette used for highlighting entities in thoughts
const DEFAULT_ENTITY_COLORS: &[Color] = &[
    RED.c500,
    AMBER.c500,
    LIME.c500,
    EMERALD.c500,
    CYAN.c500,
    BLUE.c500,
    VIOLET.c500,
    FUCHSIA.c500,
    ORANGE.c500,
    YELLOW.c500,
    GREEN.c500,
    SKY.c500,
    TEAL.c500,
    INDIGO.c500,
    PURPLE.c500,
    PINK.c500,
];

/// Assigns a persistent color to an Entity across thoughts. Once assigned, the color returned for
/// given Entity will never change
pub struct EntityColorizer {
    palette: Vec<Color>,
    assigned_colors: HashMap<EntityId, Color>,

    next_idx: usize,
}

impl Default for EntityColorizer {
    fn default() -> Self {
        EntityColorizer::new()
    }
}

impl EntityColorizer {
    /// Creates a new colorizer with a default palette
    pub fn new() -> Self {
        EntityColorizer::with_palette(Vec::from(DEFAULT_ENTITY_COLORS))
    }

    /// Creates a new colorizer with a provided palette
    pub fn with_palette(palette: Vec<Color>) -> Self {
        Self {
            palette,
            assigned_colors: HashMap::new(),
            next_idx: 0,
        }
    }

    /// Given an Entity identifier, returns a color that was previously assigned to it. If no color
    /// was assigned for this Entity yet, assigns a new one and returns it
    pub fn assign_color(&mut self, entity: EntityId) -> Color {
        match self.assigned_colors.get(&entity) {
            None => {
                let style = self.next_color();
                self.assigned_colors.insert(entity, style);
                style
            }
            Some(style) => { *style }
        }
    }

    /// Returns a next color from the palette to be assigned to an Entity
    fn next_color(&mut self) -> Color {
        let current = self.next_idx;
        self.next_idx += 1;
        if self.next_idx == self.palette.len() {
            self.next_idx = 0;
        };

        self.palette[current]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorizer_differs() {
        let mut colorizer = EntityColorizer::new();

        let entity1 = EntityId::from("first");
        let first = colorizer.assign_color(entity1);

        let entity2 = EntityId::from("second");
        let second = colorizer.assign_color(entity2);

        assert_ne!(first, second);

        let entity3 = EntityId::from("third");
        let third = colorizer.assign_color(entity3);
        assert_ne!(first, third);
        assert_ne!(second, third);
    }

    #[test]
    fn colorizer_stable() {
        let mut colorizer = EntityColorizer::new();

        let entity1 = EntityId::from("first");
        let first = colorizer.assign_color(entity1.clone());

        let entity2 = EntityId::from("second");
        let second = colorizer.assign_color(entity2.clone());

        assert_ne!(first, second);

        let first_again = colorizer.assign_color(entity1);
        assert_eq!(first, first_again);

        let second_again = colorizer.assign_color(entity2);
        assert_eq!(second, second_again);
    }

    #[test]
    fn colorizer_custom_palette() {
        let mut colorizer = EntityColorizer::with_palette(vec![
            BLUE.c200,
            GREEN.c800,
        ]);

        let entity1 = EntityId::from("first");
        let first = colorizer.assign_color(entity1);
        assert_eq!(BLUE.c200, first);

        let entity2 = EntityId::from("second");
        let second = colorizer.assign_color(entity2);
        assert_eq!(GREEN.c800, second);

        let entity3 = EntityId::from("third");
        let third = colorizer.assign_color(entity3);
        assert_eq!(BLUE.c200, third);
    }
}
