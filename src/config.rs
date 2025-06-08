//! Key mapping configuration for customizable input bindings.
//!
//! This module defines a flexible and extensible system for mapping key
//! sequences to actions or items in applications like command-line tools,
//! games, or UI frameworks. It supports:
//!
//! - Default mappings via the [`KeyMapConfig`] trait.
//! - Deserializable user configurations via [`Config<T>`].
//! - Automatic fallback and merging behavior via [`DerivedConfig<T>`].
//! - Lookup utilities to resolve parsed key sequences (from [`keymap_parser`]).
//!
//! The [`Item`] struct represents a user-facing description and set of key
//! bindings. Mappings are indexed and reversed using [`Matcher`] to enable
//! fast lookups from parsed sequences to items.
//!
//! ## Example Usage
//!
//! ```toml
//! Create = { keys = ["c"], description = "Create a new item" }
//! Delete = { keys = ["d", "d e", "@digit"], description = "Delete an item" }
//! ```
//!
//! The parsed configuration allows reverse-lookup of actions based on input
//! like `"d"` or `"1"` (with `@digit`), and supports default values through
//! trait-based extension points.
//!
//! See [`Config`], [`DerivedConfig`], and [`Item`] for more details.
use keymap_parser::{parse_seq, Node};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{fmt, marker::PhantomData, ops::Deref};

use crate::{matcher::Matcher, KeyMap};

/// A trait for providing a default mapping between keys and items.
///
/// Implementors define the default associations between values of `T`
/// and their corresponding [`Item`]s. These defaults are used when
/// deserializing a [`DerivedConfig<T>`] or constructing a [`keymap::Config<T>`].
/// Users can override these defaults with their own entries.
///
/// # Examples
///
/// ```ignore
/// #[derive(Debug, PartialEq, Eq, Hash)]
/// enum Action {
///     Create,
///     Update,
/// }
///
/// // This is auto-implemented by `keymap_derive::KeyMap` proc macro.
/// impl KeyMapConfig<Action> for Action {
///     /// Returns the default key-to-item mappings.
///     fn keymap_config() -> Vec<(Action, Item)> {
///         vec![
///             (Action::Create, Item::new(vec!["c".into()], "Create an item".into())),
///             (Action::Update, Item::new(vec!["u".into()], "Update an item".into())),
///         ]
///     }
///
///     /// Returns the [`Item`] associated with this variant.
///     fn keymap_item(&self) -> Item {
///         match self {
///             Action::Create => Item::new(vec!["c".into()], "Create an item".into()),
///             Action::Update => Item::new(vec!["u".into()], "Update an item".into()),
///         }
///     }
/// }
/// ```
pub trait KeyMapConfig<T> {
    /// Returns the default key-to-item mappings.
    ///
    /// This method should return a vector of `(T, Item)` pairs representing
    /// the default associations between keys and their corresponding items.
    /// These defaults will be incorporated into a [`keymap::Config<T>`]
    /// and can be overridden by user-supplied configuration when deserializing.
    fn keymap_config() -> Config<T>;

    /// Returns the [`Item`] associated with this particular variant.
    ///
    /// This method allows looking up the default item corresponding to
    /// a specific value of `T`. It should produce the same data as
    /// found in the vector returned by [`keymap_config`].
    ///
    /// # Example
    ///
    /// ```ignore
    /// let item = Action::Create.keymap_item();
    ///
    /// assert_eq!(item.keys, vec!["c"]);
    /// assert_eq!(item.description, "Create an item");
    /// ```
    fn keymap_item(&self) -> Item;
}

/// A deserializable configuration structure that maps keys to items.
///
/// `Config<T>` maintains:
/// 1. `items`: a `Vec<(T, Item)>` of all associations from a key type `T`
///    to its corresponding `Item`.
/// 2. `keys`: an internal reverse-lookup vector of `(Vec<Node>, index)`
///    where each `Vec<Node>` is a parsed key sequence and `index` points
///    back into the `items` vector. This allows fast resolution from a
///    parsed sequence of key nodes to the stored `(T, Item)` pair.
///
/// The generic parameter `T` is the “key type” (for example, a `String` or
/// an enum). Deserialization of `Config<T>` expects a map where the keys
/// are of type `T` and the values are `Item` structs defined below.
///
/// # Example (T = Action)
///
/// ```
/// # use serde::Deserialize;
/// # use keymap::{Config, Item};
/// let toml = r#"
///     Create = { keys = ["c"], description = "Create a new item" }
///     Delete = { keys = ["d", "d e"], description = "Delete an item" }
/// "#;
///
/// #[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
/// enum Action {
///     Create,
///     Delete,
/// }
///
/// let config: Config<Action> = toml::from_str(toml).unwrap();
///
/// // Reverse lookup by the literal string "c"
/// let (key, item) = config.get_item_by_key_str("c").unwrap();
/// assert_eq!(key, &Action::Create);
/// assert_eq!(item.description, "Create a new item");
/// ```
#[derive(Debug)]
pub struct Config<T> {
    /// A list of `(T, Item)` pairs as provided by deserialization.
    pub items: Vec<(T, Item)>,

    /// A reverse-lookup structure: each element is `(Vec<Node>, usize)`, where
    /// `Vec<Node>` is the parsed key sequence and `usize` is an index into
    /// the `items` vector. This allows efficient lookup of `(T, Item)` by
    /// matching against a parsed `Node` sequence.
    matcher: Matcher<usize>,
}

/// A configuration that merges user-provided entries with defaults.
///
/// `DerivedConfig<T>` wraps a [`Config<T>`], but also uses the
/// [`KeyMapConfig<T>`] trait to provide fallback entries. Any entries
/// supplied by the user during deserialization override the corresponding
/// default; if no user entry exists for a given key in `T::keymap_config()`,
/// the default is preserved.
///
/// This is useful for applications where you want to ship a built-in keymap
/// (for example, commands in a CLI) but allow users to override or add entries
/// in their own configuration file.
///
/// # Example (T = enum implementing `KeyMapConfig`)
///
/// ```
/// # use serde::Deserialize;
/// # use keymap::{Config, DerivedConfig, Item, KeyMapConfig};
/// #[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
/// enum Action {
///     Create,
///     Update,
///     Delete,
/// }
///
/// impl KeyMapConfig<Action> for Action {
///     fn keymap_config() -> Vec<(Action, Item)> {
///         vec![
///             (Action::Create, Item::new(vec!["c".into()], "Create".into())),
///             (Action::Update, Item::new(vec!["u".into()], "Update".into())),
///             (Action::Delete, Item::new(vec!["d".into()], "Delete".into())),
///         ]
///     }
///
///     fn keymap_item(&self) -> Item {
///         match self {
///             Action::Create => Item::new(vec!["c".into()], "Create".into()),
///             Action::Update => Item::new(vec!["u".into()], "Update".into()),
///             Action::Delete => Item::new(vec!["d".into()], "Delete".into()),
///         }
///     }
/// }
///
/// // TOML snippet that overrides Create and Delete, but leaves Update as default:
/// let toml = r#"
///     Create = { keys = ["x"], description = "Custom create" }
///     Delete = { keys = ["z"], description = "Custom delete" }
/// "#;
///
/// let derived: DerivedConfig<Action> = toml::from_str(toml).unwrap();
///
/// // "Create" was overridden:
/// let (action, item) = derived.get_item_by_key_str("x").unwrap();
/// assert_eq!(*action, Action::Create);
/// assert_eq!(item.description, "Custom create");
///
/// // "Update" falls back to default because it was not in TOML:
/// let (action, item) = derived.get_item_by_key_str("u").unwrap();
/// assert_eq!(*action, Action::Update);
/// assert_eq!(item.description, "Update");
/// ```
#[derive(Debug)]
pub struct DerivedConfig<T>(Config<T>);

impl<T> Deref for DerivedConfig<T> {
    type Target = Config<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents a single mapping entry: a list of key strings and a human-
/// readable description. During deserialization, each string in `keys`
/// will be parsed into a `Vec<Node>` internally to build the reverse lookup.
///
/// # Fields
///
/// - `keys`: a `Vec<String>` where each element is a key expression
///   (e.g., `"a b"`, `"@digit"`, etc.) that can be parsed by `parse_seq`.
/// - `description`: a text description of what the key(s) do.
///
/// # Example
///
/// ```ignore
/// let item = Item {
///     keys: vec!["a".into(), "b c".into()],
///     description: "Some command".into(),
/// };
/// ```
#[derive(Debug, Deserialize, PartialEq)]
pub struct Item {
    /// A collection of key expressions. Each expression will be run through
    /// `keymap_parser::parse_seq`, so special notations like `@digit` or
    /// multi-key sequences (e.g., `"d e"`) are supported.
    pub keys: Vec<String>,

    /// A short description for display or documentation purposes.
    pub description: String,
}

impl<T> Config<T> {
    pub fn new(items: Vec<(T, Item)>) -> Self {
        let mut matcher = Matcher::new();

        items.iter().enumerate().for_each(|(index, (_, item))| {
            item.keys
                .iter()
                .map(|keys| parse_seq(keys).expect("a valid key"))
                .for_each(|keys| {
                    matcher.add(keys, index);
                });
        });

        Self { items, matcher }
    }

    // fn merge(&mut self, items: Vec<(T, Item)>) {
    //     let mut map: HashMap<T, Item> = self.items.into_iter().collect();
    //
    //     self.items.extend(items);
    //
    // }

    /// Lookup an `(T, Item)` pair by a parsed `Node`, returning a
    /// reference to the key type `T` and the associated `Item` if found.
    ///
    /// This is a convenience alias for `get_item_by_key`.
    ///
    /// # Example
    ///
    /// ```
    /// # use keymap::{Config, Item};
    /// # use keymap_parser::parse;
    /// let config: Config<String> = toml::from_str(r#"
    ///     Create = { keys = ["c"], description = "Create a new item" }
    /// "#).unwrap();
    ///
    /// let node = parse("c").unwrap();
    /// if let Some((key, item)) = config.get_item_by_node(&node) {
    ///     println!("Found key: {:?}, desc: {}", key, item.description);
    /// }
    /// ```
    pub fn get_item_by_node(&self, node: &Node) -> Option<(&T, &Item)> {
        self.get_item_by_key(node)
    }

    /// Lookup by a full [`KeyMap`], which is a simple wrapper around `Node`.
    pub fn get_item_by_keymap(&self, keymap: &KeyMap) -> Option<(&T, &Item)> {
        self.get_item_by_node(&keymap.0)
    }

    /// Retrieve just the key type `T` (without the `Item`) by a parsed
    /// `Node`. Returns `None` if not found.
    pub fn get_by_node(&self, node: &Node) -> Option<&T> {
        self.get_item_by_node(node).map(|(t, _)| t)
    }

    /// Retrieve just the key type `T` by a full [`KeyMap`].
    pub fn get_by_keymap(&self, keymap: &KeyMap) -> Option<&T> {
        self.get_item_by_node(&keymap.0).map(|(t, _)| t)
    }

    /// Lookup an `(T, Item)` by a raw string. This will attempt to parse the
    /// string through `parse_seq` and then perform a lookup on the resulting
    /// slice of `Node`s.
    ///
    /// Returns `None` if parsing fails or if no matching entry exists.
    ///
    /// # Example
    ///
    /// ```
    /// # use keymap::{Config, Item};
    /// let config: Config<String> = toml::from_str(r#"
    ///     Create = { keys = ["c e"], description = "Create a new item" }
    /// "#).unwrap();
    ///
    /// if let Some((key, item)) = config.get_item_by_key_str("c e") {
    ///     println!("Found key: {:?}, desc: {}", key, item.description);
    /// }
    /// ```
    pub fn get_item_by_key_str(&self, key: &str) -> Option<(&T, &Item)> {
        self.get_item_by_keys(parse_seq(key).ok()?.as_slice())
    }

    /// Lookup an `(T, Item)` pair by a single parsed [`Node`]. Return the associated
    /// `(T, Item)` if the element of `Vec<Node>` matches `key`.
    ///
    /// This method does not clone the stored keys; it only compares the first
    /// element of each stored `Vec<Node>` to the provided `Node`.
    pub fn get_item_by_key(&self, key: &Node) -> Option<(&T, &Item)> {
        self.get_item_by_keys(std::slice::from_ref(key))
    }

    /// Lookup an `(T, Item)` pair by an entire slice of parsed [`Node`]s.
    /// This performs an exact match against one of the stored `Vec<Node>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use keymap::{Config, Item};
    /// # use keymap_parser::parse_seq;
    /// let config: Config<String> = toml::from_str(r#"
    ///     Create = { keys = ["x y"], description = "Create a new item" }
    /// "#).unwrap();
    ///
    /// let nodes = parse_seq("x y").unwrap();
    /// if let Some((key, item)) = config.get_item_by_keys(&nodes) {
    ///     println!("Exact match for {:?}: {:?}", nodes, item);
    /// }
    /// ```
    pub fn get_item_by_keys(&self, keys: &[Node]) -> Option<(&T, &Item)> {
        self.matcher
            .get(keys)
            .map(|i| (&self.items[*i].0, &self.items[*i].1))
    }
}

impl Item {
    /// Create a new `Item` with the given list of key expressions and a
    /// description.
    ///
    /// # Parameters
    ///
    /// - `keys`: `Vec<String>` where each string is a key expression that
    ///   can be passed to `parse_seq`.
    /// - `description`: a human-readable description of what this `Item` does.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let item = Item::new(vec!["c".into(), "x y".into()], "Some command".into());
    /// ```
    pub fn new(keys: Vec<String>, description: String) -> Self {
        Self { keys, description }
    }
}

/// Custom deserialization logic for [`Config<T>`], enabling a configuration format map of `T = Item` entries. During deserialization,
/// we build:
///
/// 1. `items`: a `Vec<(T, Item)>` in the insertion order.
/// 2. `keys`: for every string in `Item.keys`, parse it into a `Vec<Node>` and
///    store `(parsed_nodes, index)` so we can do reverse lookups quickly without
///    cloning `T`.
///
/// # Map Format
///
/// When deserializing, Serde expects a map whose keys are of type `T` and
/// whose values are `Item`. For example, with `T = String`:
///
/// ```toml
/// Create = { keys = ["c"], description = "Create a new item" }
/// Delete = { keys = ["d", "d e"], description = "Delete an item" }
/// ```
///
/// Or with `T = enum` (using `Deserialize` + `keymap_derive` to map variants
/// to string names):
///
/// ```toml
/// Create = { keys = ["n"], description = "Create an item" }
/// Delete = { keys = ["d"], description = "Delete an item" }
/// ```
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
                formatter.write_str("a map of items (key = T, value = Item)")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut items = Vec::new();

                // For each entry in the map, deserialize `T` (the key) and `Item`
                while let Some((t, item)) = map.next_entry::<T, Item>()? {
                    items.push((t, item));
                }

                Ok(Config::new(items))
            }
        }

        deserializer.deserialize_map(ConfigVisitor(PhantomData))
    }
}

/// Custom deserialization for [`DerivedConfig<T>`], which first loads
/// the default items from `T::keymap_config()` (via the `KeyMapConfig` trait)
/// and then overrides or extends them with any entries present in the
/// deserialized map. Finally, it rebuilds the internal reverse-lookup.
///
/// This ensures that:
/// 1. Any default entries provided by `KeyMapConfig<T>::keymap_config()` are
///    included unless overridden by the user.
/// 2. If the user’s config contains a key type `T` that matches a default,
///    the `Item` is replaced. Otherwise, a new `(T, Item)` is appended.
///
/// # Trait Bounds
///
/// - `T: KeyMapConfig<T>` to obtain defaults.
/// - `T: PartialEq + Eq + Hash` to find and replace default entries by key.
/// - `T: Deserialize<'de>` for parsing user-supplied keys.
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
                formatter.write_str("a map of items with defaults from KeyMapConfig")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                // Start with the default items from KeyMapConfig
                let mut config = T::keymap_config();

                // Merge user-specified entries: replace or append
                while let Some((t, item)) = map.next_entry::<T, Item>()? {
                    if let Some(pos) = config
                        .items
                        .iter()
                        .position(|(existing_key, _)| existing_key == &t)
                    {
                        // Override the default Item if the key matches
                        config.items[pos].1 = item;
                    } else {
                        // Append a new entry
                        config.items.push((t, item));
                    }
                }

                Ok(DerivedConfig(Config::new(config.items)))
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
        Delete = { keys = ["d", "d e", "@digit"], description = "Delete an item" }
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

        // Reverse lookup by key string "c"
        let (action, item) = config.get_item_by_key_str("c").unwrap();
        assert_eq!(action, "Create");
        assert_eq!(item.description, "Create a new item");

        // Reverse lookup by parsed sequence ["d", "e"]
        let (action, item) = config.get_item_by_keys(&parse_seq("d e").unwrap()).unwrap();
        assert_eq!(action, "Delete");
        assert_eq!(item.description, "Delete an item");

        // Test special @digit group: any digit character should map to Delete
        let (action, _) = config.get_item_by_key_str("1").unwrap();
        assert_eq!(action, "Delete");
    }

    #[test]
    fn test_deserialize_enum_keys() {
        let config: Config<Action> = toml::from_str(CONFIG).unwrap();

        // Reverse lookup by key "c"
        let (action, _) = config.get_item_by_key_str("c").unwrap();
        assert_eq!(*action, Action::Create);

        // No "u" in user config, so should return None
        assert!(config.get_item_by_key_str("u").is_none());

        // "d" maps to Delete
        let (action, _) = config.get_item_by_key_str("d").unwrap();
        assert_eq!(*action, Action::Delete);

        // Test @digit group on enums
        let (action, _) = config.get_item_by_key_str("1").unwrap();
        assert_eq!(*action, Action::Delete);
    }

    #[test]
    fn test_deserialize_with_override() {
        let config: DerivedConfig<Action> = toml::from_str(CONFIG).unwrap();

        // "c" was provided by user config
        let (action, _) = config.get_item_by_key_str("c").unwrap();
        assert_eq!(*action, Action::Create);

        // "u" falls back to default from KeyMapConfig
        let (action, _) = config.get_item_by_key_str("u").unwrap();
        assert_eq!(*action, Action::Update);

        // "d" was provided by user config
        let (action, _) = config.get_item_by_key_str("d").unwrap();
        assert_eq!(*action, Action::Delete);
    }
}
