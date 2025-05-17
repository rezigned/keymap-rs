use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

pub trait KeyMapConfig<T> {
    // type Key;

    fn keymap_config() -> Vec<(T, Item)>;
}

#[derive(Debug)]
pub struct Config<T> {
    pub items: Vec<(T, Item)>,
    pub keys: HashMap<String, usize>, // Index into items Vec instead of cloning T
}

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
        self.keys
            .get(key)
            .and_then(|&idx| self.items.get(idx))
            .map(|(t, item)| (t, item))
    }
}

// impl<T: KeyMapConfig<T>> Config<T> {
//     pub fn get_by_key2(&self) -> Self {
//         let items = T::keymap_config();
//         let mut keys = HashMap::new();
//
//         for (index, (_, item)) in items.iter().enumerate() {
//             for key in &item.keys {
//                 keys.insert(key.clone(), index);
//             }
//         }
//
//         Config { items, keys }
//     }
// }

impl Item {
    pub fn new(keys: Vec<String>, description: String) -> Self {
        Self { keys, description }
    }
}

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

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut items = Vec::new();
                let mut keys = HashMap::new();

                while let Some((key, item)) = map.next_entry::<T, Item>()? {
                    let item_index = items.len();

                    // Build reverse lookup map using index
                    for item_key in &item.keys {
                        keys.insert(item_key.clone(), item_index);
                    }

                    items.push((key, item));
                }

                Ok(Config { items, keys })
            }
        }

        deserializer.deserialize_map(ConfigVisitor(PhantomData))
    }
}

impl<'de, T: KeyMapConfig<T>> Deserialize<'de> for DerivedConfig<T>
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
            T: Deserialize<'de> + KeyMapConfig<T>,
        {
            type Value = DerivedConfig<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of items")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut items = Vec::new();
                let mut keys = HashMap::new();

                let config: Vec<(T, Item)> = T::keymap_config();
                config.iter().for_each(|(t, item)| {
                    dbg!(item);
                    // let item_index = items.len();
                    //
                    // // Build reverse lookup map using index
                    // for item_key in &item.keys {
                    //     keys.insert(item_key.clone(), item_index);
                    // }
                    //
                    // items.push((t.clone(), item.clone()));
                });
                while let Some((key, item)) = map.next_entry::<T, Item>()? {
                    let item_index = items.len();

                    // Build reverse lookup map using index
                    for item_key in &item.keys {
                        keys.insert(item_key.clone(), item_index);
                    }

                    items.push((key, item));
                }

                Ok(DerivedConfig(Config { items, keys }))
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
        Delete = { keys = ["d", "d d"], description = "Delete an item" }
    "#;

    #[derive(Debug, keymap_derive::KeyMap, Deserialize, PartialEq)]
    enum Action {
        #[key("n")]
        Create,
        Delete,
    }

    #[test]
    fn test_deserialize_string_keys() {
        let config: Config<String> = toml::from_str(CONFIG).unwrap();

        println!("{:#?}", config);

        // Test reverse lookup
        let (action, item) = config.get_by_key("c").unwrap();
        assert_eq!(action, "Create");
        assert_eq!(item.description, "Create a new item");

        let (action, item) = config.get_by_key("d d").unwrap();
        assert_eq!(action, "Delete");
        assert_eq!(item.keys, vec!["d", "d d"]);

        // Test items
        assert_eq!(config.items.len(), 2);
        assert_eq!(config.items[0].0, "Create");
        assert_eq!(config.items[1].0, "Delete");
    }

    #[test]
    fn test_deserialize_enum_keys() {
        let config: Config<Action> = toml::from_str(CONFIG).unwrap();

        println!("{:#?}", config);

        // Test reverse lookup
        let (action, _) = config.get_by_key("c").unwrap();
        assert_eq!(*action, Action::Create);

        let (action, _) = config.get_by_key("d").unwrap();
        assert_eq!(*action, Action::Delete);
    }

    #[test]
    fn test_deserialize_with_override() {
        let config: DerivedConfig<Action> = toml::from_str(CONFIG).unwrap();

        println!("{:#?}", config);

        // // Test reverse lookup
        // let (action, _) = config.get_by_key("c").unwrap();
        // assert_eq!(*action, Action::Create);
        //
        // let (action, _) = config.get_by_key("d").unwrap();
        // assert_eq!(*action, Action::Delete);
    }
}
