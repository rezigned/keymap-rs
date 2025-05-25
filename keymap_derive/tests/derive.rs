use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, keymap_derive::KeyMap, Hash, Deserialize)]
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
    use keymap::Item;

    use super::*;

    const CONFIG: &str = include_str!("./config.toml");

    #[test]
    fn test_derive_key() {
        [
            (Action::Create, "enter"),
            (Action::Delete, "d"),
            (Action::Delete, "d d"),
            (Action::Delete, "delete"),
        ]
        .map(|(action, input)| {
            let key = keymap::parse_seq(input).unwrap();
            assert_eq!(action, Action::try_from(key).unwrap());
        });
    }

    #[test]
    fn test_keymap_config() {
        let config = Action::keymap_config();

        assert_eq!(
            config,
            vec![
                (
                    Action::Create,
                    Item::new(
                        ["enter", "ctrl-b n"].map(ToString::to_string).to_vec(),
                        "Create a new file.\nMulti-line support.".to_string()
                    )
                ),
                (
                    Action::Delete,
                    Item::new(
                        ["d", "delete", "d d"].map(ToString::to_string).to_vec(),
                        "Delete a file".to_string()
                    )
                ),
            ]
        );
    }

    #[test]
    fn test_deserialize() {
        let config: Config = toml::from_str(CONFIG).unwrap();

        dbg!(config.keys);
    }
}
