use std::fmt::Debug;

use keymap_parser::Node;
use serde::Deserialize;

#[derive(Debug)]
struct Config<T>(Vec<(T, Item)>);
// struct DerivedConfig<T>((T, Item));
impl<T> Config<T> {
    // Returns the T from the key
    fn from_keymap(node: Node) {}
}

trait KeyMapInfo {}
trait DerivedConfig<T> {
    fn keymap_config(&self, node: Node) -> Vec<(T, Item)>;
}

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

use serde::de::{Deserializer, MapAccess, Visitor};
use std::fmt;

impl<'de, T> Deserialize<'de> for Config<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ListVisitor<T> {
            marker: std::marker::PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for ListVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Config<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map from keys to Info values")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Config<T>, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = Vec::new();

                while let Some((key, value)) = map.next_entry()? {
                    entries.push((key, value));
                }

                Ok(Config(entries))
            }
        }

        deserializer.deserialize_map(ListVisitor {
            marker: std::marker::PhantomData,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(keymap_derive::KeyMap, Debug, Deserialize)]
    enum Action {
        #[key("n")]
        Create,
        Delete,
    }

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

    #[test]
    fn test_override_derived() {
        let config: Config<Action> = toml::from_str(ITEMS).unwrap();
        dbg!(config);
    }
}
