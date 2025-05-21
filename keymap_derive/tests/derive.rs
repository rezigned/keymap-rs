use serde::Deserialize;

#[derive(Debug, PartialEq)]
pub(crate) struct KeyMapInfo {
    pub keys: Vec<&'static str>,
    pub description: &'static str,
}

#[derive(Debug, PartialEq, Eq, keymap_derive::KeyMap, Deserialize)]
enum Action {
    /// Create a new file.
    /// Multi-line support.
    #[key("enter", "ctrl-b n")]
    Create,
    /// Delete a file
    #[key("d", "delete", "d d")]
    Delete,
}

#[derive(Debug, Deserialize)]
struct Config {
    keys: keymap::Config<Action>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = include_str!("./config.toml");

    #[test]
    fn test_derive_key() {
        [
            (Action::Create, "enter"),
            (Action::Delete, "d"),
            (Action::Delete, "delete"),
        ]
        .map(|(action, input)| {
            let key = keymap::parse(input).unwrap();
            dbg!(Action::keymap_config());
            assert_eq!(action, Action::try_from(key).unwrap());
        });
    }

    #[test]
    fn test_derive_key_seq() {
        [
            (Action::Create, "ctrl-b n"),
            (Action::Delete, "d d")
        ].map(|(action, input)| {
            let key = keymap::parse_seq(input).unwrap();
            assert_eq!(action, Action::try_from(key).unwrap());
        });
    }
    // #[test]
    // fn test_keymap_keys() {
    //     [
    //         (Action::Create, vec!["enter"]),
    //         (Action::Delete, vec!["d", "delete"]),
    //     ]
    //     .map(|(action, keys)| {
    //         assert_eq!(action.keymap_keys(), keys);
    //     });
    // }

    #[test]
    fn test_deserialize() {
        let config: Config = toml::from_str(CONFIG).unwrap();

        dbg!(config.keys);
    }
}
