use keymap_parser::{node::CharGroup, parse_seq, Key, Node};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{collections::HashMap, fmt, marker::PhantomData, ops::Deref};

use crate::KeyMap;

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
    /// A list of `(T, Item)` pairs.
    pub items: Vec<(T, Item)>,

    /// A map of keys to their index in the `items` list.
    keys: HashMap<String, usize>,

    /// An ordered list of group names.
    groups: Vec<(CharGroup, usize)>,
}

/// A variant of [`Config`] that merges user-defined and default entries.
///
/// `DerivedConfig<T>` uses the `KeyMapConfig` trait to provide fallback items,
/// which are overridden by any deserialized entries from the config source.
///
/// Useful for combining default behaviors with user customization.
#[derive(Debug)]
pub struct DerivedConfig<T>(Config<T>);

impl<T> Deref for DerivedConfig<T> {
    type Target = Config<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Item {
    pub keys: Vec<String>,
    pub description: String,
}

impl<T> Config<T> {
    pub fn get_item_by_node(&self, node: &Node) -> Option<(&T, &Item)> {
        self.get_item_by_key(&node.to_string())
    }

    pub fn get_item_by_keymap(&self, keymap: &KeyMap) -> Option<(&T, &Item)> {
        self.get_item_by_node(&keymap.0)
    }

    pub fn get_by_node(&self, node: &Node) -> Option<&T> {
        self.get_item_by_node(node).map(|(t, _)| t)
    }

    pub fn get_by_keymap(&self, keymap: &KeyMap) -> Option<&T> {
        self.get_item_by_node(&keymap.0).map(|(t, _)| t)
    }

    pub fn get_item_by_keys(&self, keys: &[&str]) -> Option<(&T, &Item)> {
        // Try to find an exact key match e.g. "c", etc.
        // If not found, try to find a group match e.g. "@lower", etc.
        keys.iter().find_map(|key| self.get_item_by_key(key))
    }

    /// Returns the item and its associated [`Item`] for the given key.
    pub fn get_item_by_key(&self, key: &str) -> Option<(&T, &Item)> {
        // Try to find an exact key match e.g. "c", etc.
        // If not found, try to find a group match e.g. "@lower", etc.
        self.keys
            .get(key)
            .and_then(|&idx| self.items.get(idx))
            .map(|(t, item)| (t, item))
            .or_else(|| {
                // Group's order is important, thus, the BTreeMap is used.
                self.groups
                    .iter()
                    .find(|(group, _)| key.chars().all(|c| group.matches(c)))
                    .and_then(|(_, idx)| self.items.get(*idx))
                    .map(|(t, item)| (t, item))
            })
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
                let mut groups = vec![];

                while let Some((key, item)) = map.next_entry::<T, Item>()? {
                    let i = items.len();

                    // Build reverse lookup map using index
                    for item_key in &item.keys {
                        let nodes = parse_seq(item_key).map_err(de::Error::custom)?;
                        nodes.iter().for_each(|node| match node.key {
                            Key::Group(group) => {
                                groups.push((group, i));
                            }
                            _ => {
                                keys.insert(node.to_string(), i);
                            }
                        });
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
                let mut groups = vec![];

                // Merge derived items with config items
                while let Some((key, item)) = map.next_entry::<T, Item>()? {
                    items.insert(key, item);
                }

                // Build reverse lookup map using index
                for (i, (_, item)) in items.iter().enumerate() {
                    for item_key in &item.keys {
                        let nodes = parse_seq(item_key).map_err(de::Error::custom)?;
                        nodes.iter().for_each(|node| match node.key {
                            Key::Group(group) => {
                                groups.push((group, i));
                            }
                            _ => {
                                keys.insert(node.to_string(), i);
                            }
                        });
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
        let (action, item) = config.get_item_by_key("c").unwrap();
        assert_eq!(action, "Create");
        assert_eq!(item.description, "Create a new item");

        let (action, item) = config.get_item_by_keys(&["d", "d"]).unwrap();
        assert_eq!(action, "Delete");
        assert_eq!(item.description, "Delete an item");

        // Test @digit group
        let (action, _) = config.get_item_by_key("1").unwrap();
        assert_eq!(action, "Delete");
    }

    #[test]
    fn test_deserialize_enum_keys() {
        let config: Config<Action> = toml::from_str(CONFIG).unwrap();

        // Test reverse lookup
        let (action, _) = config.get_item_by_key("c").unwrap();
        assert_eq!(*action, Action::Create);

        // There's no update key in the config.
        assert!(config.get_item_by_key("u").is_none());

        let (action, _) = config.get_item_by_key("d").unwrap();
        assert_eq!(*action, Action::Delete);

        // Test @digit group
        let (action, _) = config.get_item_by_key("1").unwrap();
        assert_eq!(*action, Action::Delete);
    }

    #[test]
    fn test_deserialize_with_override() {
        let config: DerivedConfig<Action> = toml::from_str(CONFIG).unwrap();

        // Test reverse lookup
        let (action, _) = config.get_item_by_key("c").unwrap();
        assert_eq!(*action, Action::Create);

        // Fallback to derived config's key i.e. "u"
        let (action, _) = config.get_item_by_key("u").unwrap();
        assert_eq!(*action, Action::Update);

        let (action, _) = config.get_item_by_key("d").unwrap();
        assert_eq!(*action, Action::Delete);
    }
}
