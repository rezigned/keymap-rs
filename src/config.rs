use std::{collections::HashMap, fmt::Debug};

use serde::Deserialize;

use crate::{KeyMap, KeyValPair};

#[derive(Debug)]
pub struct Config<V>(pub HashMap<KeyMap, V>);

impl<T> Config<T> {
    pub fn get(&self, key: &KeyMap) -> Option<&T> {
        self.0.get(key)
    }

    pub fn extend(&self) {

    }
    // pub fn keymaps(self) -> Self {
        // self.0.into_values().next().unwrap().
    // }
}

// impl<V> KeyValPair<V> for Config<V> {
//     fn keymaps() -> Self {
//         todo!()
//     }
// }

struct ConfigSeq<T>(pub HashMap<Vec<KeyMap>, T>);

// impl<'de, T> Deserialize<'de> for Config<T>
// where
//     T: Deserialize<'de> + Debug,
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         // let keys = HashMap::<String, T>::deserialize(deserializer)
//         //     .into_iter()
//         //     .map(|v| parse_seq()); // returns Vec<KeyMap>

//         // TODO: Add additional entries from KeyMap derive implementations
//         HashMap::deserialize(deserializer)
//             .map(Config)
//     }
// }

impl<'de, T> Deserialize<'de> for Config<T>
where
    T: Deserialize<'de> + KeyValPair<T> + Debug,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        HashMap::deserialize(deserializer)
            .map(Config)
            .map(|c: Config<T>| {
                // let c2 = c.0.values().next().unwrap().keymaps_self();
                let mut c2 = c.0.values().next().unwrap().keymaps_self();
                // c.0.extend(c2.0);
                c2.0.extend(c.0);
                c2
            })
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
        // fn keymaps() -> super::Config<Self> {
        //     super::Config(HashMap::from([
        //         // (parse("c").unwrap(), Self::Create),
        //         (parse("c").unwrap(), Self::Create),
        //     ]))
        // }

        fn keymaps_self(&self) -> super::Config<Self> {
            super::Config(HashMap::from([
                // (parse("c").unwrap(), Self::Create),
                (parse("c").unwrap(), Self::Create),
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
        // dbg!(c.);
    }
}
