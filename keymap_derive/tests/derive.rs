// TODO: Fix release-please bug. See https://github.com/googleapis/release-please/issues/1662#issuecomment-1419080151
extern crate keymap_dev as keymap;

#[derive(Debug, PartialEq, Eq, keymap_derive::KeyMap, Clone)]
enum Action {
    /// Create a new file.
    /// Multi-line support.
    #[key("enter", "ctrl-b n")]
    Create,
    /// Delete a file
    #[key("d", "delete", "d d", "@lower")]
    Delete,
    /// Quit
    #[key("esc", "q")]
    Quit,

    /// Digit with char argument
    #[key("@digit")]
    Digit(char),

    /// Jump with char argument
    #[key("@any")]
    Jump(char),
}

#[cfg(test)]
mod tests {
    use keymap_dev::{Error, Item, KeyMap, KeyMapConfig, ToKeyMap};

    use super::*;

    struct Wrapper(keymap_parser::Node);

    impl ToKeyMap for Wrapper {
        fn to_keymap(&self) -> Result<KeyMap, Error> {
            Ok(self.0.clone())
        }
    }

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
            assert_eq!(&action, config.get_item_by_keymaps(&key).unwrap().0);
        });
    }

    #[test]
    fn test_derive_char_group() {
        let config = Action::keymap_config();

        [
            (Action::Delete, "x"),      // @lower
            (Action::Digit('\0'), "1"), // @digit
        ]
        .map(|(action, input)| {
            let key = keymap_parser::parse_seq(input).unwrap();
            assert_eq!(&action, config.get_item_by_keymaps(&key).unwrap().0);
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
                        ["d", "delete", "d d", "@lower"]
                            .map(ToString::to_string)
                            .to_vec(),
                        "Delete a file".to_string()
                    )
                ),
                (
                    Action::Quit,
                    Item::new(
                        ["esc", "q"].map(ToString::to_string).to_vec(),
                        "Quit".to_string()
                    )
                ),
                (
                    Action::Digit('\0'),
                    Item::new(
                        ["@digit"].map(ToString::to_string).to_vec(),
                        "Digit with char argument".to_string()
                    )
                ),
                (
                    Action::Jump('\0'),
                    Item::new(
                        ["@any"].map(ToString::to_string).to_vec(),
                        "Jump with char argument".to_string()
                    )
                ),
            ]
        );
    }

    #[test]
    fn test_bound_payload_extraction() {
        let config = Action::keymap_config();

        // When we press '1', it matches @digit, and we should extract '1'
        let keys = keymap_parser::parse_seq("1")
            .unwrap()
            .into_iter()
            .map(Wrapper)
            .collect::<Vec<_>>();
        let bound_action = config.get_bound_seq(&keys).unwrap();
        assert_eq!(bound_action, Action::Digit('1'));

        // When we press 'A', it matches @any, and we should extract 'A'
        let keys = keymap_parser::parse_seq("A")
            .unwrap()
            .into_iter()
            .map(Wrapper)
            .collect::<Vec<_>>();
        let bound_action = config.get_bound_seq(&keys).unwrap();

        assert_eq!(bound_action, Action::Jump('A'));

        // When we press 'Q', it matches @any, and we should extract 'Q'
        let keys = keymap_parser::parse_seq("Q")
            .unwrap()
            .into_iter()
            .map(Wrapper)
            .collect::<Vec<_>>();
        let nodes = keys.iter().map(|k| k.0.clone()).collect::<Vec<_>>();
        let (bound_action, item) = config.get_bound_item_by_keymaps(&nodes).unwrap();

        assert_eq!(bound_action, Action::Jump('Q'));
        assert_eq!(item.description, "Jump with char argument");

        // Standard keys should extract as well using get_bound_seq
        let keys = keymap_parser::parse_seq("enter")
            .unwrap()
            .into_iter()
            .map(Wrapper)
            .collect::<Vec<_>>();
        let bound_action = config.get_bound_seq(&keys).unwrap();
        assert_eq!(bound_action, Action::Create);
    }
}
