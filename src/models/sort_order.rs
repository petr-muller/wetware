use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Sort order for the thought list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Oldest thoughts first
    Ascending,
    /// Newest thoughts first
    Descending,
}

impl SortOrder {
    /// Toggle between ascending and descending order.
    pub fn toggle(&mut self) {
        *self = match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
    }

    /// Human-readable label for the current sort order.
    pub fn label(&self) -> &str {
        match self {
            SortOrder::Ascending => "Oldest first",
            SortOrder::Descending => "Newest first",
        }
    }
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Ascending => write!(f, "ascending"),
            SortOrder::Descending => write!(f, "descending"),
        }
    }
}

impl FromStr for SortOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ascending" => Ok(SortOrder::Ascending),
            "descending" => Ok(SortOrder::Descending),
            _ => Err(format!(
                "Invalid sort order: '{s}'. Valid values: ascending, descending"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_order_toggle_ascending_to_descending() {
        let mut order = SortOrder::Ascending;
        order.toggle();
        assert_eq!(order, SortOrder::Descending);
    }

    #[test]
    fn test_sort_order_toggle_descending_to_ascending() {
        let mut order = SortOrder::Descending;
        order.toggle();
        assert_eq!(order, SortOrder::Ascending);
    }

    #[test]
    fn test_sort_order_label_ascending() {
        assert_eq!(SortOrder::Ascending.label(), "Oldest first");
    }

    #[test]
    fn test_sort_order_label_descending() {
        assert_eq!(SortOrder::Descending.label(), "Newest first");
    }

    #[test]
    fn test_sort_order_double_toggle_returns_to_original() {
        let mut order = SortOrder::Ascending;
        order.toggle();
        order.toggle();
        assert_eq!(order, SortOrder::Ascending);
    }

    #[test]
    fn test_display_ascending() {
        assert_eq!(SortOrder::Ascending.to_string(), "ascending");
    }

    #[test]
    fn test_display_descending() {
        assert_eq!(SortOrder::Descending.to_string(), "descending");
    }

    #[test]
    fn test_from_str_ascending() {
        assert_eq!("ascending".parse::<SortOrder>().unwrap(), SortOrder::Ascending);
    }

    #[test]
    fn test_from_str_descending() {
        assert_eq!("descending".parse::<SortOrder>().unwrap(), SortOrder::Descending);
    }

    #[test]
    fn test_from_str_invalid() {
        let result = "invalid".parse::<SortOrder>();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid sort order"));
    }

    #[test]
    fn test_serde_roundtrip() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Wrapper {
            order: SortOrder,
        }
        let w = Wrapper {
            order: SortOrder::Descending,
        };
        let serialized = toml::to_string(&w).unwrap();
        let deserialized: Wrapper = toml::from_str(&serialized).unwrap();
        assert_eq!(w, deserialized);
    }
}
