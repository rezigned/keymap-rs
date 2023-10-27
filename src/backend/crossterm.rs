use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{de, Deserialize, Deserializer};

use crate::parser::{self, Key as Keys, Modifier, Node};
use super::{Key, NodeModifiers};

pub type KeyMap = Key<KeyEvent>;

pub fn parse(s: &str) -> Result<KeyMap, pom::Error> {
    parser::parse(s).map(KeyMap::from)
}

impl From<KeyEvent> for KeyMap {
    fn from(value: KeyEvent) -> Self {
        Self { event: value, node: Some(Node::from(value)) }
    }
}

impl From<Node> for KeyMap {
    fn from(node: Node) -> Self {
        Self {
            event: KeyEvent::from(&node),
            node: Some(node),
        }
    }
}

impl From<KeyEvent> for Node {
    fn from(value: KeyEvent) -> Self {
        match value {
            KeyEvent { code, modifiers, .. } => {
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

                Self { key, modifiers: NodeModifiers::from(modifiers).into() }
            }
        }
    }
}

impl<'a> From<&'a Node> for KeyEvent {
    fn from(node: &'a Node) -> Self {
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

        Self::new(key, NodeModifiers::from(node.modifiers).into())
    }
}

const MODIFIERS: [(KeyModifiers, parser::Modifier); 4] = [
    (KeyModifiers::ALT, Modifier::Alt),
    (KeyModifiers::CONTROL, Modifier::Ctrl),
    (KeyModifiers::META, Modifier::Cmd),
    (KeyModifiers::SHIFT, Modifier::Shift),
];

impl From<KeyModifiers> for NodeModifiers {
    fn from(value: KeyModifiers) -> Self {
        Self(MODIFIERS.into_iter().fold(0, |mut m, (m1, m2)| {
            if value.contains(m1) {
                m |= m2 as u8;
            }
            m
        }))
    }
}

impl From<NodeModifiers> for KeyModifiers {
    fn from(value: NodeModifiers) -> Self {
        MODIFIERS.into_iter().fold(KeyModifiers::NONE, |mut m, (m1, m2)| {
            if value.0 & m2 as u8 != 0 {
                m|= m1
            }
            m
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

    use crate::{backend::{
        crossterm::{parse, KeyMap},
        Key
    }, parser::{Node, self}};

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
            assert_eq!(Node::from(key), node);
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
