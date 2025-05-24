use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{de, Deserialize, Deserializer};

use super::{Key, KeyMap2, NodeModifiers};
use keymap_parser::{self as parser, Key as Keys, Modifier, Node};

pub type KeyMap = Key<KeyEvent>;

pub fn parse(s: &str) -> Result<KeyMap, pom::Error> {
    parser::parse(s).map(KeyMap::from)
}

impl From<KeyEvent> for KeyMap2 {
    fn from(value: KeyEvent) -> Self {
        Self(node_from_backend(value))
    }
}

impl From<KeyEvent> for KeyMap {
    fn from(value: KeyEvent) -> Self {
        Self {
            event: value,
            node: Some(node_from_backend(value)),
        }
    }
}

impl From<Node> for KeyMap {
    fn from(node: Node) -> Self {
        Self {
            event: backend_from_node(&node),
            node: Some(node),
        }
    }
}

fn node_from_backend(value: KeyEvent) -> Node {
    let KeyEvent {
        code, modifiers, ..
    } = value;
    {
        let key = match code {
            KeyCode::BackTab => Keys::BackTab,
            KeyCode::Backspace => Keys::Backspace,
            KeyCode::Char(' ') => Keys::Space,
            KeyCode::Char(c) => Keys::Char(c),
            KeyCode::Delete => Keys::Delete,
            KeyCode::Down => Keys::Down,
            KeyCode::End => Keys::End,
            KeyCode::Enter => Keys::Enter,
            KeyCode::Esc => Keys::Esc,
            KeyCode::F(n) => Keys::F(n),
            KeyCode::Home => Keys::Home,
            KeyCode::Insert => Keys::Insert,
            KeyCode::Left => Keys::Left,
            KeyCode::PageDown => Keys::PageDown,
            KeyCode::PageUp => Keys::PageUp,
            KeyCode::Right => Keys::Right,
            KeyCode::Tab => Keys::Tab,
            KeyCode::Up => Keys::Up,
            code => panic!("Unsupport KeyEvent {code:?}"),
        };

        Node {
            key,
            modifiers: NodeModifiers::from(modifiers).into(),
        }
    }
}

fn backend_from_node(node: &Node) -> KeyEvent {
    let key = match node.key {
        Keys::BackTab => KeyCode::BackTab,
        Keys::Backspace => KeyCode::Backspace,
        Keys::Char(c) => KeyCode::Char(c),
        Keys::Delete => KeyCode::Delete,
        Keys::Down => KeyCode::Down,
        Keys::End => KeyCode::End,
        Keys::Enter => KeyCode::Enter,
        Keys::Esc => KeyCode::Esc,
        Keys::F(n) => KeyCode::F(n),
        Keys::Home => KeyCode::Home,
        Keys::Insert => KeyCode::Insert,
        Keys::Left => KeyCode::Left,
        Keys::PageDown => KeyCode::PageDown,
        Keys::PageUp => KeyCode::PageUp,
        Keys::Right => KeyCode::Right,
        Keys::Tab => KeyCode::Tab,
        Keys::Space => KeyCode::Char(' '),
        Keys::Up => KeyCode::Up,
    };

    KeyEvent::new(key, NodeModifiers::from(node.modifiers).into())
}

const MODIFIERS: [(KeyModifiers, parser::Modifier); 4] = [
    (KeyModifiers::ALT, Modifier::Alt),
    (KeyModifiers::CONTROL, Modifier::Ctrl),
    (KeyModifiers::META, Modifier::Cmd),
    (KeyModifiers::SHIFT, Modifier::Shift),
];

impl From<KeyModifiers> for NodeModifiers {
    fn from(value: KeyModifiers) -> Self {
        Self(MODIFIERS.into_iter().fold(0, |acc, (m1, m2)| {
            acc | if value.contains(m1) { m2 as u8 } else { 0 }
        }))
    }
}

impl From<NodeModifiers> for KeyModifiers {
    fn from(value: NodeModifiers) -> Self {
        let none = KeyModifiers::NONE;
        MODIFIERS.into_iter().fold(none, |acc, (m1, m2)| {
            acc | if value.0 & (m2 as u8) != 0 { m1 } else { none }
        })
    }
}

/// Deserializes into Key
impl<'s> Deserialize<'s> for KeyMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'s>,
    {
        let s = String::deserialize(deserializer)?;
        parse(&s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use serde::Deserialize;

    use crate::backend::{
        crossterm::{node_from_backend, parse, KeyMap},
        Key,
    };
    use keymap_parser as parser;

    #[test]
    fn test_parse() {
        let alt_node = Key::from(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::ALT | KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ));

        [
            ("[", &Key::from(KeyEvent::from(KeyCode::Char('[')))),
            ("del", &Key::from(KeyEvent::from(KeyCode::Delete))),
            ("alt-ctrl-shift-a", &alt_node),
            ("alt-shift-ctrl-a", &alt_node),
            ("shift-alt-ctrl-a", &alt_node),
        ]
        .map(|(s, node)| {
            assert_eq!(*node, parse(s).unwrap());
        });
    }

    #[test]
    fn test_from_key_to_node() {
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
            assert_eq!(node_from_backend(key), node);
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
            let (key, _) = result.key.get_key_value(&Key::from(n)).unwrap();
            assert_eq!(key, &Key::from(n));
        });
    }
}
