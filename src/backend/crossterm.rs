//! Key event parsing and conversion for the `crossterm` backend.
//!
//! This module bridges `crossterm::event::KeyEvent` with a backend-agnostic
//! representation (`KeyMap`) used for keybinding configuration and matching.
//! It enables parsing human-readable key definitions and converting between
//! representations suitable for UI and configuration layers.
//!
//! # Key Features
//! - `parse`: Parses a string key representation (e.g., "Ctrl+S") into a `KeyEvent`.
//! - Implements `IntoKeyMap`, `ToKeyMap`, and `FromKeyMap` for `KeyEvent`.
//! - Converts between `KeyEvent` (from crossterm) and internal `KeyMap` format.
//!
//! # Limitations
//! - Some `KeyCode` variants are not supported and will return an error.
//! - Key groups (e.g., `@any`) are not reversible to `KeyEvent` due to the loss of specificity.
//!
//! # Examples
//!
//! Parsing from a string:
//! ```
//! use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
//! use keymap::backend::crossterm::parse;
//!
//! let key = parse("ctrl-a").unwrap();
//! assert_eq!(key, KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));
//! ```
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use keymap_parser::{self as parser, Key, Modifier, Node};

use crate::{
    keymap::{FromKeyMap, IntoKeyMap, KeyMap, ToKeyMap},
    Error,
};

/// Parses a string keybinding (e.g., `"ctrl-c"`, `"f1"`, `"alt+backspace"`) into a `KeyEvent`.
///
/// This uses the `keymap_parser` crate to interpret the string and maps it into a `KeyEvent`.
/// Returns an error if parsing fails or the resulting representation cannot be converted.
///
/// # Errors
/// - Returns `Error::Parse` if the string cannot be parsed.
/// - Returns `Error::UnsupportedKey` if the key or modifiers are unsupported.
pub fn parse(s: &str) -> Result<KeyEvent, Error> {
    parser::parse(s)
        .map_err(Error::Parse)
        .and_then(KeyEvent::from_keymap)
}

impl IntoKeyMap for KeyEvent {
    /// Converts a `KeyEvent` into a `KeyMap`.
    ///
    /// Internally delegates to `ToKeyMap`.
    fn into_keymap(self) -> Result<KeyMap, Error> {
        self.to_keymap()
    }
}

impl ToKeyMap for KeyEvent {
    /// Converts a `KeyEvent` to the `KeyMap` format.
    ///
    /// # Errors
    /// - Returns `Error::UnsupportedKey` if the `KeyEvent` variant is not supported for conversion.
    fn to_keymap(&self) -> Result<KeyMap, Error> {
        let KeyEvent {
            code, modifiers, ..
        } = self;
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
                    "Unsupported KeyEvent {code:?}"
                )))
            }
        };

        Ok(Node::new(modifiers_from_backend(modifiers), key))
    }
}

impl FromKeyMap for KeyEvent {
    /// Converts a `KeyMap` back into a `KeyEvent`.
    ///
    /// # Errors
    /// - Returns `Error::UnsupportedKey` if the `KeyMap` contains a `Group`, which cannot be
    ///   reversed into a concrete `KeyEvent`.
    fn from_keymap(keymap: KeyMap) -> Result<Self, Error> {
        let key = match keymap.key {
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

        Ok(KeyEvent::new(key, modifiers_from_node(keymap.modifiers)))
    }
}

/// Static mapping between `crossterm` modifiers and internal `keymap_parser::Modifier`s.
const MODIFIERS: [(KeyModifiers, parser::Modifier); 4] = [
    (KeyModifiers::ALT, Modifier::Alt),
    (KeyModifiers::CONTROL, Modifier::Ctrl),
    (KeyModifiers::META, Modifier::Cmd),
    (KeyModifiers::SHIFT, Modifier::Shift),
];

/// Converts a `KeyModifiers` bitflag into a `parser::Modifiers` bitfield.
fn modifiers_from_backend(value: &KeyModifiers) -> parser::Modifiers {
    MODIFIERS.into_iter().fold(0, |acc, (m1, m2)| {
        acc | if value.contains(m1) { m2 as u8 } else { 0 }
    })
}

/// Converts a `parser::Modifiers` bitfield into a `KeyModifiers` bitflag.
fn modifiers_from_node(value: parser::Modifiers) -> KeyModifiers {
    let none = KeyModifiers::NONE;
    MODIFIERS.into_iter().fold(none, |acc, (m1, m2)| {
        acc | if value & (m2 as u8) != 0 { m1 } else { none }
    })
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
            assert_eq!(key.to_keymap().unwrap(), node);
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
            assert_eq!(KeyEvent::from_keymap(node).unwrap(), key);
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
            let (key, _) = result.key.get_key_value(&n.to_keymap().unwrap()).unwrap();

            assert_eq!(key, &n.to_keymap().unwrap());
        });
    }
}
