use std::{
    collections::HashMap,
    fmt::Debug,
};

use crate::KeyMap;
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

// impl<'de> Deserialize<'de> for Item
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         HashMap::deserialize(deserializer).map(Item)
//     }
// }

pub trait KeyMapConfig<V> {
    fn keymap_config() -> Config<V>;
}

trait Configure<V> {
    fn get(&self, key: &KeyMap) -> Option<&V>;
    // fn get_seq(&self, key: &KeyMap) -> Option<&V>;
}

#[derive(Debug)]
pub struct Config<V>(pub HashMap<KeyMap, V>);

// impl<V, T: HashMap<String, V>> Configure<V> for T {
//     fn get(&self, key: &KeyMap) -> Option<&V> {
//         todo!()
//     }
// }

// TODO: Should we create a new trait in parser so that
// we can use it here (blanket implementation)
// impl<V, T: HashMap<String, V>> T {
//     fn get(&self, key: &KeyMap) -> Option<&V> {
//         todo!()
//     }
// }

impl<V> Config<V> {
    /// Extends the current config with the other config.
    pub fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
    }
}

impl<V> Configure<V> for Config<V> {
    /// Retrieves the value associated with the given key, if any.
    fn get(&self, key: &KeyMap) -> Option<&V> {
        self.0.get(key)
    }
}

// Deref doesn't work with trait bound.
// use std::ops::Deref;

// impl<V: KeyMapConfig<V>> Deref for DerivedConfig<V> {
//     type Target = Config<V>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

#[derive(Debug)]
pub struct DerivedConfig<V: KeyMapConfig<V>>(Config<V>);

impl<V: KeyMapConfig<V>> Configure<V> for DerivedConfig<V> {
    fn get(&self, key: &KeyMap) -> Option<&V> {
        self.0.get(key)
    }
}

impl<'de, V> Deserialize<'de> for Config<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        HashMap::deserialize(deserializer).map(Config)
    }
}

impl<'de, V> Deserialize<'de> for DerivedConfig<V>
where
    V: Deserialize<'de> + KeyMapConfig<V>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Config::deserialize(deserializer).map(|c| {
            // Extend with derived config
            let mut config = V::keymap_config();
            config.extend(c);

            DerivedConfig(config)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    use super::*;

    const ITEMS: &str = r#"
    Create = { keys = ["c"], description = "Create a new item" }
    Delete = { keys = ["d", "d d"], description = "Delete an item" }
    "#;

    #[derive(Debug, Deserialize)]
    struct Config {
        keys: super::Config<Action>,
    }

    #[derive(Debug, Deserialize)]
    struct DerivedConfig {
        keys: super::DerivedConfig<Action>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    enum Action {
        Create,
        Delete,
    }

    impl KeyMapConfig<Action> for Action {
        fn keymap_config() -> crate::Config<Action> {
            super::Config(HashMap::from([
                (parse("n").unwrap(), Action::Create),
                (parse("x").unwrap(), Action::Delete),
            ]))
        }
    }

    const CONFIG: &str = r#"
[keys]
c = "Create"
d = "Delete"
"#;

    fn run(keys: impl Configure<Action>, cases: &[(&str, Option<Action>)]) {
        cases.iter().for_each(|(k, v)| {
            let key = parse(k).unwrap();
            assert_eq!(v.as_ref(), keys.get(&key));
        })
    }

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
    fn test_deserialize_config() {
        let c: Config = toml::from_str(CONFIG).unwrap();

        run(
            c.keys,
            &[
                ("c", Some(Action::Create)),
                ("d", Some(Action::Delete)),
                ("n", None),
                ("x", None),
            ],
        );
    }

    #[test]
    fn test_deserialize_derived_config() {
        let c: DerivedConfig = toml::from_str(CONFIG).unwrap();

        run(
            c.keys,
            &[
                ("c", Some(Action::Create)),
                ("d", Some(Action::Delete)),
                ("n", Some(Action::Create)),
                ("x", Some(Action::Delete)),
            ],
        );
    }

    #[test]
    fn test_deserialize_and_override_derived_config() {
        const CONFIG: &str = r#"
        [keys]
        n = "Delete"
        x = "Create"
        "#;

        let c: DerivedConfig = toml::from_str(CONFIG).unwrap();

        run(
            c.keys,
            &[("n", Some(Action::Delete)), ("x", Some(Action::Create))],
        );
    }
}
