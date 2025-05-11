use std::fmt::Debug;

use serde::Deserialize;

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Item {
    pub keys: Vec<String>,
    pub description: String,
}

impl Item {
    pub fn new(keys: Vec<String>, description: String) -> Self {
        Self { keys, description }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const ITEMS: &str = r#"
    Create = { keys = ["c"], description = "Create a new item" }
    Delete = { keys = ["d", "d d"], description = "Delete an item" }
    "#;

    #[test]
    fn test_deserialize_items() {
        let items: HashMap<String, Item> = toml::from_str(ITEMS).unwrap();

        assert_eq!(
            items,
            HashMap::from([
                (
                    "Create".to_string(),
                    Item {
                        keys: vec!["c".to_string()],
                        description: "Create a new item".to_string()
                    }
                ),
                (
                    "Delete".to_string(),
                    Item {
                        keys: vec!["d".to_string(), "d d".to_string()],
                        description: "Delete an item".to_string()
                    }
                ),
            ])
        );
    }
}
