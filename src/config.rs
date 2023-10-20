use std::{collections::HashMap, fmt::Debug};

use serde::Deserialize;

use crate::KeyMap;

#[derive(Debug)]
struct Config<V>(pub HashMap<KeyMap, V>);

impl<T> Config<T> {
    pub fn get(&self, key: &KeyMap) -> Option<&T> {
        self.0.get(key)
    }
}

struct ConfigSeq<T>(pub HashMap<Vec<KeyMap>, T>);

impl<'de, T> Deserialize<'de> for Config<T>
where
    T: Deserialize<'de> + Debug,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // let keys = HashMap::<String, T>::deserialize(deserializer)
        //     .into_iter()
        //     .map(|v| parse_seq()); // returns Vec<KeyMap>

        // TODO: Add additional entries from KeyMap derive implementations
        HashMap::deserialize(deserializer)
            .map(Config)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse, KeyValPair};

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

    impl Into<HashMap<KeyMap, Self>> for Action {
        fn into(self) -> HashMap<KeyMap, Self> {
            HashMap::from([
                (parse("c").unwrap(), Self::Create)
            ])
        }
    }

    impl KeyValPair<Self> for Action {
        fn keymaps() -> HashMap<Vec<&'static str>, Self> {
            HashMap::from([
                // (parse("c").unwrap(), Self::Create),
                (vec!["c"], Self::Create),
            ])
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
        dbg!(Action::keymaps());
    }
}
