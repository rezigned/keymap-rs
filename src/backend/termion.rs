use serde::{de, Deserialize, Deserializer};
use termion::event::Key as KeyEvent;

use crate::{
    parser::{self, Key as Keys, Modifier, Node},
    Key,
};

pub type KeyMap = Key<KeyEvent>;

pub fn parse(s: &str) -> Result<KeyMap, pom::Error> {
    parser::parse(s).map(|n| n.into())
}

impl From<KeyEvent> for KeyMap {
    fn from(value: KeyEvent) -> Self {
        Self {
            event: value,
            node: None,
        }
    }
}

impl From<Node> for KeyMap {
    fn from(node: Node) -> Self {
        let key = match node.key {
            Keys::BackTab => KeyEvent::BackTab,
            Keys::Backspace => KeyEvent::Backspace,
            Keys::Char(c) => KeyEvent::Char(c),
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
        };

        // Termion only allows modifier + char.
        // It also doesn't support Shift/Meta key.
        let event = match key {
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
            },
            _ => key,
        };

        Self {
            event,
            node: Some(node),
        }
    }
}

/// Deserializes into Node
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
    use crate::{backend::termion::parse, Key};
    use termion::event::Key as KeyEvent;

    #[test]
    fn test_parse() {
        [
            ("[", KeyEvent::Char('[')),
            ("del", KeyEvent::Delete),
            ("alt-a", KeyEvent::Alt('a')),
            ("shift-a", KeyEvent::Char('A')),
            ("shift-=", KeyEvent::Char('+')),
            ("enter", KeyEvent::Char('\n')),
            ("ctrl-a", KeyEvent::Ctrl('a')),
        ]
        .map(|(s, node)| {
            assert_eq!(Key::from(node), parse(s).unwrap());
        });
    }
}
