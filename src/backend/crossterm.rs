use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{de, Deserialize, Deserializer};

use crate::parser::{self, Key as Keys, Modifier, Node};
use super::Key;

pub type KeyMap = Key<KeyEvent>;

pub fn parse(s: &str) -> Result<KeyMap, pom::Error> {
    parser::parse(s).map(|n| n.into())
}

impl From<KeyEvent> for KeyMap {
    fn from(value: KeyEvent) -> Self {
        Self { event: value, node: None }
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

impl<'a> From<&'a Node> for KeyEvent {
    fn from(node: &'a Node) -> Self {
        let key = match node.key {
            Keys::BackTab => KeyCode::BackTab.into(),
            Keys::Backspace => KeyCode::Backspace.into(),
            Keys::Char(c) => KeyCode::Char(c).into(),
            Keys::Delete => KeyCode::Delete.into(),
            Keys::Down => KeyCode::Down.into(),
            Keys::End => KeyCode::End.into(),
            Keys::Enter => KeyCode::Enter.into(),
            Keys::Esc => KeyCode::Esc.into(),
            Keys::F(n) => KeyCode::F(n).into(),
            Keys::Home => KeyCode::Home.into(),
            Keys::Insert => KeyCode::Insert.into(),
            Keys::Left => KeyCode::Left.into(),
            Keys::PageDown => KeyCode::PageDown.into(),
            Keys::PageUp => KeyCode::PageUp.into(),
            Keys::Right => KeyCode::Right.into(),
            Keys::Tab => KeyCode::Tab.into(),
            Keys::Space => KeyCode::Char(' ').into(),
            Keys::Up => KeyCode::Up.into(),
        };

        Self::new(key, modifiers(node.modifiers))
    }
}

fn modifiers(m: u8) -> KeyModifiers {
    let mut mods = KeyModifiers::NONE;
    if m & Modifier::Alt as u8 != 0 {
        mods |= KeyModifiers::ALT
    }
    if m & Modifier::Cmd as u8 != 0 {
        mods |= KeyModifiers::META
    }
    if m & Modifier::Ctrl as u8 != 0 {
        mods |= KeyModifiers::CONTROL
    }
    if m & Modifier::Shift as u8 != 0 {
        mods |= KeyModifiers::SHIFT
    }

    mods
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
        crossterm::{parse, KeyMap},
        Key
    };

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
