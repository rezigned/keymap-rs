use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use keymap_parser::{self as parser, Key, Modifier, Node};
use serde::{de, Deserialize, Deserializer};

use crate::{Config, Error, Item, KeyMap};

pub fn parse(s: &str) -> Result<KeyEvent, Error> {
    parser::parse(s)
        .map_err(Error::Parse)
        .and_then(|node| backend_from_node(&node))
}

impl TryFrom<KeyEvent> for KeyMap {
    type Error = Error;

    fn try_from(value: KeyEvent) -> Result<Self, Self::Error> {
        node_from_backend(&value).map(Self)
    }
}

impl TryFrom<KeyMap> for KeyEvent {
    type Error = Error;

    fn try_from(value: KeyMap) -> Result<Self, Self::Error> {
        backend_from_node(&value.0)
    }
}

impl<T> Config<T> {
    pub fn get(&self, key: KeyEvent) -> Option<&T> {
        self.get_by_keymap(&key.try_into().ok()?)
    }

    pub fn get_item(&self, key: KeyEvent) -> Option<(&T, &Item)> {
        self.get_item_by_keymap(&key.try_into().ok()?)
    }
}

fn node_from_backend(value: &KeyEvent) -> Result<Node, Error> {
    let KeyEvent {
        code, modifiers, ..
    } = value;
    {
        let key = match code {
            KeyCode::BackTab => Key::BackTab,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Char(' ') => Key::Space,
            KeyCode::Char(c) => Key::Char(*c),
            KeyCode::Delete => Key::Delete,
            KeyCode::Down => Key::Down,
            KeyCode::End => Key::End,
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Esc,
            KeyCode::F(n) => Key::F(*n),
            KeyCode::Home => Key::Home,
            KeyCode::Insert => Key::Insert,
            KeyCode::Left => Key::Left,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::Right => Key::Right,
            KeyCode::Tab => Key::Tab,
            KeyCode::Up => Key::Up,
            code => {
                return Err(Error::UnsupportedKey(format!(
                    "Unsupport KeyEvent {code:?}"
                )))
            }
        };

        Ok(Node {
            key,
            modifiers: modifiers_from_backend(modifiers),
        })
    }
}

fn backend_from_node(node: &Node) -> Result<KeyEvent, Error> {
    let key = match node.key {
        Key::BackTab => KeyCode::BackTab,
        Key::Backspace => KeyCode::Backspace,
        Key::Char(c) => KeyCode::Char(c),
        Key::Delete => KeyCode::Delete,
        Key::Down => KeyCode::Down,
        Key::End => KeyCode::End,
        Key::Enter => KeyCode::Enter,
        Key::Esc => KeyCode::Esc,
        Key::F(n) => KeyCode::F(n),
        Key::Home => KeyCode::Home,
        Key::Insert => KeyCode::Insert,
        Key::Left => KeyCode::Left,
        Key::PageDown => KeyCode::PageDown,
        Key::PageUp => KeyCode::PageUp,
        Key::Right => KeyCode::Right,
        Key::Tab => KeyCode::Tab,
        Key::Space => KeyCode::Char(' '),
        Key::Up => KeyCode::Up,
        Key::Group(group) => {
            return Err(Error::UnsupportedKey(format!(
                "Group {group:?} not supported. There's no way to map char group back to KeyEvent"
            )))
        }
    };

    Ok(KeyEvent::new(key, modifiers_from_node(node.modifiers)))
}

const MODIFIERS: [(KeyModifiers, parser::Modifier); 4] = [
    (KeyModifiers::ALT, Modifier::Alt),
    (KeyModifiers::CONTROL, Modifier::Ctrl),
    (KeyModifiers::META, Modifier::Cmd),
    (KeyModifiers::SHIFT, Modifier::Shift),
];

fn modifiers_from_backend(value: &KeyModifiers) -> parser::Modifiers {
    MODIFIERS.into_iter().fold(0, |acc, (m1, m2)| {
        acc | if value.contains(m1) { m2 as u8 } else { 0 }
    })
}

fn modifiers_from_node(value: parser::Modifiers) -> KeyModifiers {
    let none = KeyModifiers::NONE;
    MODIFIERS.into_iter().fold(none, |acc, (m1, m2)| {
        acc | if value & (m2 as u8) != 0 { m1 } else { none }
    })
}

impl<'s> Deserialize<'s> for KeyMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'s>,
    {
        let s = String::deserialize(deserializer)?;
        keymap_parser::parse(&s)
            .map(KeyMap)
            .map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use keymap_parser as parser;
    use serde::Deserialize;

    use super::*;

    fn alt_node() -> KeyEvent {
        KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::ALT | KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        )
    }

    #[test]
    fn test_parse() {
        let alt_node = alt_node();

        [
            ("[", &KeyEvent::from(KeyCode::Char('['))),
            ("del", &KeyEvent::from(KeyCode::Delete)),
            ("alt-ctrl-shift-a", &alt_node),
            ("alt-shift-ctrl-a", &alt_node),
            ("shift-alt-ctrl-a", &alt_node),
        ]
        .map(|(s, node)| {
            assert_eq!(*node, parse(s).unwrap());
        });
    }

    #[test]
    fn test_from_backend_to_node() {
        let alt_a = KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::ALT | KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );

        [
            (KeyEvent::from(KeyCode::Char('[')), "["),
            (KeyEvent::from(KeyCode::Delete), "del"),
            (alt_a, "alt-ctrl-shift-a"),
        ]
        .map(|(key, code)| {
            let node = parser::parse(code).unwrap();
            assert_eq!(node_from_backend(&key).unwrap(), node);
        });
    }

    #[test]
    fn test_from_node_to_backend() {
        let alt_a = KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::ALT | KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );

        [
            (KeyEvent::from(KeyCode::Char('[')), "["),
            (KeyEvent::from(KeyCode::Delete), "del"),
            (alt_a, "alt-ctrl-shift-a"),
        ]
        .map(|(key, code)| {
            let node = parser::parse(code).unwrap();
            assert_eq!(backend_from_node(&node).unwrap(), key);
        });
    }

    #[test]
    fn test_deserialize() {
        use std::collections::HashMap;

        #[derive(Deserialize, Debug)]
        struct Test {
            key: HashMap<KeyMap, String>,
        }

        let result: Test = toml::from_str(
            r#"
[key]
alt-d = "a"
cmd-shift-del = "b"
shift-cmd-del = "b" # this is the same as previous one
delete = "d"
    "#,
        )
        .unwrap();

        [
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::ALT),
            KeyEvent::new(KeyCode::Delete, KeyModifiers::META | KeyModifiers::SHIFT),
            KeyEvent::from(KeyCode::Delete),
        ]
        .map(|n| {
            let (key, _) = result
                .key
                .get_key_value(&KeyMap::try_from(n).unwrap())
                .unwrap();
            assert_eq!(key, &KeyMap::try_from(n).unwrap());
        });
    }
}
