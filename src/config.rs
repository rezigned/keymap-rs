use std::{collections::HashMap, fmt::Debug};

use serde::Deserialize;

use crate::KeyMap;

#[derive(Debug)]
struct Config<T>(pub HashMap<KeyMap, T>);

impl<T> Config<T> {
    pub fn get(&self, key: &KeyMap) -> Option<&T> {
        self.0.get(key)
    }
}

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

        HashMap::deserialize(deserializer)
            .map(Config)
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
