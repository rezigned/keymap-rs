#[derive(Debug, PartialEq, Eq, keymap_derive::KeyMap)]
enum Action {
    #[key("enter")]
    Create,
    #[key("d", "delete")]
    Delete
}

#[cfg(test)]
mod tests {
    use super::Action;

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
    fn test_key() {
        [
            (Action::Create, vec!["enter"]),
            (Action::Delete, vec!["d", "delete"]),
        ]
        .map(|(action, keys)| {
            assert_eq!(action.keymap_keys(), keys);
        });
    }
}
