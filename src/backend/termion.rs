use serde::{de, Deserialize, Deserializer};
use termion::event::Key as KeyEvent;

use keymap_parser::{self as parser, Key, Modifier, Node};

use crate::{Error, KeyMap};

pub fn parse(s: &str) -> Result<KeyEvent, Error> {
    parser::parse(s)
        .map_err(Error::Parse)
        .and_then(backend_from_node)
}

impl TryFrom<KeyEvent> for KeyMap {
    type Error = Error;

    fn try_from(value: KeyEvent) -> Result<Self, Self::Error> {
        node_from_backend(value).map(Self)
    }
}

fn node_from_backend(value: KeyEvent) -> Result<Node, Error> {
    let (key, modifiers) = match value {
        KeyEvent::BackTab => (Key::BackTab, 0),
        KeyEvent::Backspace => (Key::Backspace, 0),
        KeyEvent::Delete => (Key::Delete, 0),
        KeyEvent::Down => (Key::Down, 0),
        KeyEvent::End => (Key::End, 0),
        KeyEvent::Char('\n') => (Key::Enter, 0),
        KeyEvent::Esc => (Key::Esc, 0),
        KeyEvent::Home => (Key::Home, 0),
        KeyEvent::F(n) => (Key::F(n), 0),
        KeyEvent::Insert => (Key::Insert, 0),
        KeyEvent::Left => (Key::Left, 0),
        KeyEvent::PageDown => (Key::PageDown, 0),
        KeyEvent::PageUp => (Key::PageUp, 0),
        KeyEvent::Right => (Key::Right, 0),
        KeyEvent::Char(' ') => (Key::Space, 0),
        KeyEvent::Char('\t') => (Key::Tab, 0),
        KeyEvent::Up => (Key::Up, 0),
        KeyEvent::Char(c) => (Key::Char(c), 0),
        KeyEvent::Alt(c) => (Key::Char(c), Modifier::Alt as u8),
        KeyEvent::Ctrl(c) => (Key::Char(c), Modifier::Ctrl as u8),
        KeyEvent::Null => (Key::Tab, 0),
        key => return Err(Error::UnsupportedKey(format!("Unsupport KeyEvent {key:?}"))),
    };

    Ok(Node { key, modifiers })
}

fn backend_from_node(node: Node) -> Result<KeyEvent, Error> {
    let key = match node.key {
        Key::BackTab => KeyEvent::BackTab,
        Key::Backspace => KeyEvent::Backspace,
        Key::Delete => KeyEvent::Delete,
        Key::Down => KeyEvent::Down,
        Key::End => KeyEvent::End,
        Key::Enter => KeyEvent::Char('\n'),
        Key::Esc => KeyEvent::Esc,
        Key::Home => KeyEvent::Home,
        Key::F(n) => KeyEvent::F(n),
        Key::Insert => KeyEvent::Insert,
        Key::Left => KeyEvent::Left,
        Key::PageDown => KeyEvent::PageDown,
        Key::PageUp => KeyEvent::PageUp,
        Key::Right => KeyEvent::Right,
        Key::Space => KeyEvent::Char(' '),
        Key::Tab => KeyEvent::Char('\t'),
        Key::Up => KeyEvent::Up,
        Key::Char(c) => KeyEvent::Char(c),
        Key::Group(group) => {
            return Err(Error::UnsupportedKey(format!(
                "Group {group:?} not supported. There's no way to map char group back to KeyEvent"
            )))
        }
    };

    // Termion only allows modifier + char.
    // It also doesn't support Shift/Meta key.
    match key {
        KeyEvent::Char(c) => {
            if node.modifiers & Modifier::Alt as u8 != 0 {
                Ok(KeyEvent::Alt(c))
            } else if node.modifiers & Modifier::Ctrl as u8 != 0 {
                Ok(KeyEvent::Ctrl(c))
            } else if node.modifiers & Modifier::Shift as u8 != 0 {
                Ok(KeyEvent::Char(c.to_ascii_uppercase()))
            } else {
                Ok(key)
            }
        }
        _ => Ok(key),
    }
}

/// Deserializes into Node
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
    use super::*;
    use keymap_parser as parser;
    use termion::event::Key as KeyEvent;

    #[test]
    fn test_parse() {
        [
            ("[", KeyEvent::Char('[')),
            ("del", KeyEvent::Delete),
            ("alt-a", KeyEvent::Alt('a')),
            ("shift-a", KeyEvent::Char('A')),
            ("A", KeyEvent::Char('A')),
            ("enter", KeyEvent::Char('\n')),
            ("ctrl-a", KeyEvent::Ctrl('a')),
        ]
        .map(|(s, node)| {
            assert_eq!(node, parse(s).unwrap());
        });
    }

    #[test]
    fn test_from_key_to_node() {
        let alt_a = KeyEvent::Alt('a');

        [
            (KeyEvent::Char('['), "["),
            (KeyEvent::Delete, "del"),
            (alt_a, "alt-a"),
        ]
        .map(|(key, code)| {
            let node = parser::parse(code).unwrap();
            assert_eq!(node_from_backend(key).unwrap(), node);
        });
    }
}
