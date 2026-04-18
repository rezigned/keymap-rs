extern crate keymap_dev as keymap;

#[derive(Debug, PartialEq, Eq, keymap_derive::KeyMap, Clone)]
enum Action {
    #[key("enter")]
    Create,
    #[key("@digit")]
    Digit(char),
}

#[derive(Debug, PartialEq, Eq, keymap_derive::KeyMap, Clone)]
enum DigitAction {
    #[key("@digit")]
    DigitU8(u8),
    #[key("@any")]
    DigitAny(char),
}

#[derive(Debug, PartialEq, Eq, keymap_derive::KeyMap, Clone)]
enum DigitAction2 {
    #[key("@digit")]
    DigitU16(u16),
    #[key("a")]
    Letter(char),
}

// Type alias — the old string-based approach would fail here.
type MyDigit = u32;

#[derive(Debug, PartialEq, Eq, keymap_derive::KeyMap, Clone)]
enum AliasAction {
    #[key("@digit")]
    Count(MyDigit),
    #[key("@any")]
    Any(char),
}

#[cfg(test)]
mod tests {
    use keymap_dev::{Error, KeyMap, KeyMapConfig, ToKeyMap};

    use super::*;

    struct Wrapper(keymap_parser::Node);
    impl ToKeyMap for Wrapper {
        fn to_keymap(&self) -> Result<KeyMap, Error> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn test_digit_char() {
        let config = Action::keymap_config();
        let keys = keymap_parser::parse_seq("1")
            .unwrap()
            .into_iter()
            .map(Wrapper)
            .collect::<Vec<_>>();
        let bound = config.get_bound_seq(&keys).unwrap();
        assert_eq!(bound, Action::Digit('1'));
    }

    #[test]
    fn test_digit_u8() {
        let config = DigitAction::keymap_config();
        let keys = keymap_parser::parse_seq("5")
            .unwrap()
            .into_iter()
            .map(Wrapper)
            .collect::<Vec<_>>();
        let bound = config.get_bound_seq(&keys).unwrap();
        assert_eq!(bound, DigitAction::DigitU8(5));
    }

    #[test]
    fn test_digit_u16() {
        let config = DigitAction2::keymap_config();
        let keys = keymap_parser::parse_seq("7")
            .unwrap()
            .into_iter()
            .map(Wrapper)
            .collect::<Vec<_>>();
        let bound = config.get_bound_seq(&keys).unwrap();
        assert_eq!(bound, DigitAction2::DigitU16(7));
    }

    #[test]
    fn test_digit_type_alias() {
        let config = AliasAction::keymap_config();
        let keys = keymap_parser::parse_seq("3")
            .unwrap()
            .into_iter()
            .map(Wrapper)
            .collect::<Vec<_>>();
        let bound = config.get_bound_seq(&keys).unwrap();
        assert_eq!(bound, AliasAction::Count(3));
    }
}
