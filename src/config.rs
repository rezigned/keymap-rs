use std::{collections::HashMap, fmt::Debug};

use serde::Deserialize;

use crate::KeyMap;

pub trait KeyMapConfig<V> {
    fn keymap_config() -> Config<V>;
}

#[derive(Debug)]
pub struct Config<V>(pub HashMap<KeyMap, V>);

#[derive(Debug)]
pub struct DerivedConfig<V: KeyMapConfig<V>>(Config<V>);

impl<V> Config<V> {
    /// Retrieves the value associated with the given key, if any.
    pub fn get(&self, key: &KeyMap) -> Option<&V> {
        self.0.get(key)
    }

    /// Extends the current config with the other config.
    pub fn extend(&mut self, other: Self) {
        self.0.extend(other.0);
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
        HashMap::deserialize(deserializer)
            .map(Config)
    }
}


impl<V: KeyMapConfig<V>> DerivedConfig<V> {
    pub fn get(&self, key: &KeyMap) -> Option<&V> {
        self.0.get(key)
    }

    /// Extends the current config with the other config.
    pub fn extend(&mut self, other: Config<V>) {
        self.0.extend(other);
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
        HashMap::deserialize(deserializer)
            .map(|v| DerivedConfig(Config(v)))
            .map(|mut c: DerivedConfig<V>| {
                // Extend with derived config
                c.extend(V::keymap_config());
                c
            })
    }
}

// trait DerivedConfig<'de> {
// }

// impl<'de, V> DerivedConfig<'de> for Config<V>
// where
//     V: Deserialize<'de> + KeyMapConfig<V>,
// {
// }

#[cfg(test)]
mod tests {
    use crate::parse;

    use super::*;

    #[derive(Debug, Deserialize)]
    struct Config {
        keys: super::DerivedConfig<Action>,
    }

    #[derive(Debug, Deserialize)]
    enum Action {
        Create,
        Delete,
    }

    impl KeyMapConfig<Action> for Action {
        fn keymap_config() -> crate::Config<Action> {
            super::Config(HashMap::from([
                (parse("x").unwrap(), Action::Create),
            ]))
        }
    }

    const CONFIG: &str = r#"
[keys]
c = "Create"
d = "Delete"
"#;

    #[test]
    fn test_deserialize() {
        let c: Config = toml::from_str(CONFIG).unwrap();
        let key = parse("c").unwrap();
        let v = c.keys.get(&key);

        dbg!(v);
        dbg!(c.keys);
    }
}
