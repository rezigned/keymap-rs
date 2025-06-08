use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, keymap_derive::KeyMap, Deserialize)]
enum Action {
    /// Create a new file.
    /// Multi-line support.
    #[key("enter", "ctrl-b n")]
    Create,
    /// Delete a file
    #[key("d", "delete", "d d", "@lower", "@digit")]
    Delete,
    /// Quit
    #[key("@any")]
    Quit,
}

#[cfg(test)]
mod tests {
    use keymap::{Item, KeyMapConfig};

    use super::*;

    #[test]
    fn test_derive_key() {
        let config = Action::keymap_config();

        [
            (Action::Create, "enter"),
            (Action::Delete, "d"),
            (Action::Delete, "d d"),
            (Action::Delete, "delete"),
        ]
        .map(|(action, input)| {
            let key = keymap_parser::parse_seq(input).unwrap();
            assert_eq!(&action, config.get_item_by_keys(&key).unwrap().0);
        });
    }

    #[test]
    fn test_derive_char_group() {
        let config = Action::keymap_config();

        [
            (Action::Delete, "x"), // @lower
            (Action::Delete, "1"), // @digit
        ]
        .map(|(action, input)| {
            let key = keymap_parser::parse_seq(input).unwrap();
            assert_eq!(&action, config.get_item_by_keys(&key).unwrap().0);
        });
    }

    #[test]
    fn test_keymap_config() {
        let config = Action::keymap_config();

        assert_eq!(
            config.items,
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
                        ["d", "delete", "d d", "@lower", "@digit"]
                            .map(ToString::to_string)
                            .to_vec(),
                        "Delete a file".to_string()
                    )
                ),
                (
                    Action::Quit,
                    Item::new(
                        ["@any"].map(ToString::to_string).to_vec(),
                        "Quit".to_string()
                    )
                ),
            ]
        );
    }
}
