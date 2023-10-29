use std::{collections::HashMap, fmt::Debug};

use serde::Deserialize;

use crate::KeyMap;

trait KeyMapConfig<V> {
    fn keymap_config() -> Config<V>;
}

#[derive(Debug)]
pub struct Config<V>(pub HashMap<KeyMap, V>);

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

struct ConfigSeq<T>(pub HashMap<Vec<KeyMap>, T>);

// impl<'de, V> Deserialize<'de> for Config<V>
// where
//     V: Deserialize<'de>,
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//     }
// }

impl<'de, V> Deserialize<'de> for Config<V>
where
    V: Deserialize<'de> + KeyMapConfig<V>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        HashMap::deserialize(deserializer)
            .map(Config)
            .map(|mut c: Config<V>| {
                // Extend with derived config
                c.extend(V::keymap_config());
                c
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    use super::*;

    #[derive(Debug, Deserialize)]
    struct Config {
        keys: super::Config<Action>,
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
