use serde::{de, Deserialize, Deserializer};
use termion::event::Key as KeyEvent;

use keymap_parser::{self as parser, parser::ParseError, Key as Keys, Modifier, Node};

use super::KeyMap;

pub fn parse(s: &str) -> Result<KeyEvent, ParseError> {
    parser::parse(s).map(backend_from_node)
}

impl From<KeyEvent> for KeyMap {
    fn from(value: KeyEvent) -> Self {
        Self(node_from_backend(value))
    }
}

fn node_from_backend(value: KeyEvent) -> Node {
    let (key, modifiers) = match value {
        KeyEvent::BackTab => (Keys::BackTab, 0),
        KeyEvent::Backspace => (Keys::Backspace, 0),
        KeyEvent::Delete => (Keys::Delete, 0),
        KeyEvent::Down => (Keys::Down, 0),
        KeyEvent::End => (Keys::End, 0),
        KeyEvent::Char('\n') => (Keys::Enter, 0),
        KeyEvent::Esc => (Keys::Esc, 0),
        KeyEvent::Home => (Keys::Home, 0),
        KeyEvent::F(n) => (Keys::F(n), 0),
        KeyEvent::Insert => (Keys::Insert, 0),
        KeyEvent::Left => (Keys::Left, 0),
        KeyEvent::PageDown => (Keys::PageDown, 0),
        KeyEvent::PageUp => (Keys::PageUp, 0),
        KeyEvent::Right => (Keys::Right, 0),
        KeyEvent::Char(' ') => (Keys::Space, 0),
        KeyEvent::Char('\t') => (Keys::Tab, 0),
        KeyEvent::Up => (Keys::Up, 0),
        KeyEvent::Char(c) => (Keys::Char(c), 0),
        KeyEvent::Alt(c) => (Keys::Char(c), Modifier::Alt as u8),
        KeyEvent::Ctrl(c) => (Keys::Char(c), Modifier::Ctrl as u8),
        KeyEvent::Null => (Keys::Tab, 0),
        key => panic!("Unsupport KeyEvent {key:?}"),
    };

    Node { key, modifiers }
}

fn backend_from_node(node: Node) -> KeyEvent {
    let key = match node.key {
        Keys::BackTab => KeyEvent::BackTab,
        Keys::Backspace => KeyEvent::Backspace,
        Keys::Delete => KeyEvent::Delete,
        Keys::Down => KeyEvent::Down,
        Keys::End => KeyEvent::End,
        Keys::Enter => KeyEvent::Char('\n'),
        Keys::Esc => KeyEvent::Esc,
        Keys::Home => KeyEvent::Home,
        Keys::F(n) => KeyEvent::F(n),
        Keys::Insert => KeyEvent::Insert,
        Keys::Left => KeyEvent::Left,
        Keys::PageDown => KeyEvent::PageDown,
        Keys::PageUp => KeyEvent::PageUp,
        Keys::Right => KeyEvent::Right,
        Keys::Space => KeyEvent::Char(' '),
        Keys::Tab => KeyEvent::Char('\t'),
        Keys::Up => KeyEvent::Up,
        Keys::Char(c) => KeyEvent::Char(c),
    };

    // Termion only allows modifier + char.
    // It also doesn't support Shift/Meta key.
    match key {
        KeyEvent::Char(c) => {
            if node.modifiers & Modifier::Alt as u8 != 0 {
                KeyEvent::Alt(c)
            } else if node.modifiers & Modifier::Ctrl as u8 != 0 {
                KeyEvent::Ctrl(c)
            } else if node.modifiers & Modifier::Shift as u8 != 0 {
                KeyEvent::Char(c.to_ascii_uppercase())
            } else {
                key
            }
        }
        _ => key,
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
            assert_eq!(node_from_backend(key), node);
        });
    }
}
