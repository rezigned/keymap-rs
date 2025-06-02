use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use crate::parse_seq;

const GROUP_PREFIX: &str = "@";

pub trait KeyMapConfig<T> {
    fn keymap_config() -> Vec<(T, Item)>;
}

/// A deserializable configuration structure that maps keys to items.
///
/// `Config<T>` stores a list of `(T, Item)` pairs and allows reverse lookups
/// from any key string to the corresponding entry via `Item.keys`.
///
/// The internal `keys` map avoids cloning `T` by indexing into the `items` list.
#[derive(Debug)]
pub struct Config<T> {
    pub items: Vec<(T, Item)>,
    keys: HashMap<String, usize>,
    groups: HashMap<String, usize>,
}

/// A variant of [`Config`] that merges user-defined and default entries.
///
/// `DerivedConfig<T>` uses the `KeyMapConfig` trait to provide fallback items,
/// which are overridden by any deserialized entries from the config source.
///
/// Useful for combining default behaviors with user customization.
#[derive(Debug)]
pub struct DerivedConfig<T>(Config<T>);

#[derive(Debug, Deserialize, PartialEq)]
pub struct Item {
    pub keys: Vec<String>,
    pub description: String,
}

impl<T> Config<T> {
    /// Returns the item and its associated [`Item`] for the given key.
    pub fn get_by_key(&self, key: &str) -> Option<(&T, &Item)> {
        // Try to find an exact key match e.g. "c", etc.
        // If not found, try to find a group match e.g. "@lower", etc.
        self.keys
            .get(key)
            .and_then(|&idx| self.items.get(idx))
            .map(|(t, item)| (t, item))
            .or_else(|| {
                self.groups
                    .iter()
                    .find(|(group, _)| match group.as_str() {
                        "@lower" => key.chars().all(|c| c.is_lowercase()),
                        "@upper" => key.chars().all(|c| c.is_uppercase()),
                        "@digit" => key.chars().all(|c| c.is_ascii_digit()),
                        "@alpha" => key.chars().all(|c| c.is_alphabetic()),
                        "@alnum" => key.chars().all(|c| c.is_alphanumeric()),
                        _ => false,
                    })
                    .and_then(|(_, &idx)| self.items.get(idx))
                    .map(|(t, item)| (t, item))
            })
    }
}

impl<T: KeyMapConfig<T> + PartialEq + Eq + std::hash::Hash> DerivedConfig<T> {
    pub fn get_by_key(&self, key: &str) -> Option<(&T, &Item)> {
        self.0.get_by_key(key)
    }
}

impl Item {
    pub fn new(keys: Vec<String>, description: String) -> Self {
        Self { keys, description }
    }
}

/// Custom deserialization for [`Config`] from a map.
///
/// This builds a lookup map from key strings to the index of each item,
/// allowing fast reverse lookups without cloning the key type `T`.
impl<'de, T> Deserialize<'de> for Config<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ConfigVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for ConfigVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Config<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of items")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut items = Vec::new();
                let mut keys = HashMap::new();
                let mut groups = HashMap::new();

                while let Some((key, item)) = map.next_entry::<T, Item>()? {
                    let item_index = items.len();

                    // Build reverse lookup map using index
                    for item_key in &item.keys {
                        let _ = parse_seq(item_key).map_err(de::Error::custom)?;

                        if item_key.starts_with(GROUP_PREFIX) {
                            groups.insert(item_key.clone(), item_index);
                        } else {
                            keys.insert(item_key.clone(), item_index);
                        }
                    }

                    items.push((key, item));
                }

                Ok(Config {
                    items,
                    keys,
                    groups,
                })
            }
        }

        deserializer.deserialize_map(ConfigVisitor(PhantomData))
    }
}

/// Custom deserialization for [`DerivedConfig`] with default fallback behavior.
///
/// Combines user-specified items with defaults provided by [`KeyMapConfig<T>`].
/// User-supplied items override defaults when keys match.
impl<'de, T: KeyMapConfig<T> + PartialEq + Eq + std::hash::Hash> Deserialize<'de>
    for DerivedConfig<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ConfigVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for ConfigVisitor<T>
        where
            T: Deserialize<'de> + KeyMapConfig<T> + PartialEq + Eq + std::hash::Hash,
        {
            type Value = DerivedConfig<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of items")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut items: HashMap<T, Item> = T::keymap_config().into_iter().collect();
                let mut keys = HashMap::new();
                let mut groups = HashMap::new();

                // Merge derived items with config items
                while let Some((key, item)) = map.next_entry::<T, Item>()? {
                    items.insert(key, item);
                }

                // Build reverse lookup map using index
                for (i, (_, item)) in items.iter().enumerate() {
                    for item_key in &item.keys {
                        let _ = parse_seq(item_key).map_err(de::Error::custom)?;

                        if item_key.starts_with(GROUP_PREFIX) {
                            groups.insert(item_key.clone(), i);
                        } else {
                            keys.insert(item_key.clone(), i);
                        }
                    }
                }

                Ok(DerivedConfig(Config {
                    items: items.into_iter().collect(),
                    keys,
                    groups,
                }))
            }
        }

        deserializer.deserialize_map(ConfigVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"
        Create = { keys = ["c"], description = "Create a new item" }
        Delete = { keys = ["d", "d d", "@digit"], description = "Delete an item" }
    "#;

    #[derive(Debug, keymap_derive::KeyMap, Deserialize, PartialEq, Eq, Hash)]
    enum Action {
        #[key("n")]
        Create,
        #[key("u")]
        Update,
        Delete,
    }

    #[test]
    fn test_deserialize_string_keys() {
        let config: Config<String> = toml::from_str(CONFIG).unwrap();

        // Test reverse lookup
        let (action, item) = config.get_by_key("c").unwrap();
        assert_eq!(action, "Create");
        assert_eq!(item.description, "Create a new item");

        let (action, item) = config.get_by_key("d d").unwrap();
        assert_eq!(action, "Delete");
        assert_eq!(item.keys, vec!["d", "d d", "@digit"]);

        // Test @digit group
        let (action, _) = config.get_by_key("1").unwrap();
        assert_eq!(action, "Delete");
    }

    #[test]
    fn test_deserialize_enum_keys() {
        let config: Config<Action> = toml::from_str(CONFIG).unwrap();

        // Test reverse lookup
        let (action, _) = config.get_by_key("c").unwrap();
        assert_eq!(*action, Action::Create);

        // There's no update key in the config.
        assert!(config.get_by_key("u").is_none());

        let (action, _) = config.get_by_key("d").unwrap();
        assert_eq!(*action, Action::Delete);

        // Test @digit group
        let (action, _) = config.get_by_key("1").unwrap();
        assert_eq!(*action, Action::Delete);
    }

    #[test]
    fn test_deserialize_with_override() {
        let config: DerivedConfig<Action> = toml::from_str(CONFIG).unwrap();

        // Test reverse lookup
        let (action, _) = config.get_by_key("c").unwrap();
        assert_eq!(*action, Action::Create);

        // Fallback to derived config's key i.e. "u"
        let (action, _) = config.get_by_key("u").unwrap();
        assert_eq!(*action, Action::Update);

        let (action, _) = config.get_by_key("d").unwrap();
        assert_eq!(*action, Action::Delete);
    }
}
