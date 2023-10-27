use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, keymap_derive::KeyMap, Deserialize)]
enum Action {
    #[key("enter")]
    Create,
    #[key("d", "delete")]
    Delete,
}

#[derive(Debug, Deserialize)]
struct Config {
    keys: keymap::Config<Action>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use keymap::KeyMap;

    use super::*;

    const CONFIG: &str = include_str!("./config.toml");

    #[test]
    fn test_derive_keys() {
        [
            (Action::Create, "enter"),
            (Action::Delete, "d"),
            (Action::Delete, "delete"),
        ]
        .map(|(action, input)| {
            let key = keymap::parse(input).unwrap();
            assert_eq!(action, Action::try_from(key).unwrap());
        });
    }

    #[test]
    fn test_keymap_keys() {
        [
            (Action::Create, vec!["enter"]),
            (Action::Delete, vec!["d", "delete"]),
        ]
        .map(|(action, keys)| {
            assert_eq!(action.keymap_keys(), keys);
        });
    }

    #[test]
    fn test_deserialize() {
        let config: Config = toml::from_str(CONFIG).unwrap();

        dbg!(config);
    }
}
