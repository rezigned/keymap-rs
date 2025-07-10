//! Key event parsing and conversion for the `termion` backend.
//!
//! This module bridges `termion::event::Key` with a backend-agnostic
//! representation (`KeyMap`) used for keybinding configuration and matching.
//! It enables parsing human-readable key definitions and converting between
//! representations suitable for UI and configuration layers.
//!
//! # Key Features
//! - `parse`: Parses a string key representation (e.g., "ctrl-a") into a `KeyEvent`.
//! - Implements `IntoKeyMap`, `ToKeyMap`, and `FromKeyMap` for `KeyEvent`.
//! - Converts between `KeyEvent` (from termion) and internal `KeyMap` format.
//!
//! # Limitations
//! - Termion only supports `alt` and `ctrl` modifiers on character keys.
//! - `Shift` is inferred by converting characters to uppercase.
//! - `Meta` is not supported.
//! - Non-character keys with modifiers are not expressible.
//! - Char groups cannot be converted back to Termion keys.
//!
//! # Examples
//!
//! Parsing from a string:
//! ```
//! use keymap::backend::termion::parse;
//! use termion::event::Key as KeyEvent;
//!
//! let key: KeyEvent = parse("ctrl-a").unwrap();
//! assert_eq!(key, KeyEvent::Ctrl('a'));
//! ```
use keymap_parser::{self as parser, Key, Modifier, Node};
use termion::event::Key as KeyEvent;

use crate::{keymap::ToKeyMap, Error, FromKeyMap, IntoKeyMap, KeyMap};

/// Parses a string representation of a key into a `KeyEvent`.
///
/// Uses the generic keymap parser and attempts to convert the result into a `KeyEvent`.
///
/// # Errors
/// Returns [`Error::Parse`] if the input cannot be parsed,
/// or [`Error::UnsupportedKey`] if the parsed result cannot be converted.
pub fn parse(s: &str) -> Result<KeyEvent, Error> {
    parser::parse(s)
        .map_err(Error::Parse)
        .and_then(KeyEvent::from_keymap)
}

impl IntoKeyMap for KeyEvent {
    /// Converts `KeyEvent` into a `KeyMap`.
    fn into_keymap(self) -> Result<KeyMap, Error> {
        self.to_keymap()
    }
}

impl ToKeyMap for KeyEvent {
    /// Converts `KeyEvent` into a `KeyMap` node.
    ///
    /// Returns [`Error::UnsupportedKey`] if the key cannot be represented as a `KeyMap`.
    fn to_keymap(&self) -> Result<KeyMap, Error> {
        let (key, modifiers) = match self {
            KeyEvent::BackTab => (Key::BackTab, 0),
            KeyEvent::Backspace => (Key::Backspace, 0),
            KeyEvent::Delete => (Key::Delete, 0),
            KeyEvent::Down => (Key::Down, 0),
            KeyEvent::End => (Key::End, 0),
            KeyEvent::Char('\n') => (Key::Enter, 0),
            KeyEvent::Esc => (Key::Esc, 0),
            KeyEvent::Home => (Key::Home, 0),
            KeyEvent::F(n) => (Key::F(*n), 0),
            KeyEvent::Insert => (Key::Insert, 0),
            KeyEvent::Left => (Key::Left, 0),
            KeyEvent::PageDown => (Key::PageDown, 0),
            KeyEvent::PageUp => (Key::PageUp, 0),
            KeyEvent::Right => (Key::Right, 0),
            KeyEvent::Char(' ') => (Key::Space, 0),
            KeyEvent::Char('\t') => (Key::Tab, 0),
            KeyEvent::Up => (Key::Up, 0),
            KeyEvent::Char(c) => (Key::Char(*c), 0),
            KeyEvent::Alt(c) => (Key::Char(*c), Modifier::Alt as u8),
            KeyEvent::Ctrl(c) => (Key::Char(*c), Modifier::Ctrl as u8),
            KeyEvent::Null => (Key::Tab, 0),
            key => {
                return Err(Error::UnsupportedKey(format!(
                    "Unsupported KeyEvent {key:?}"
                )))
            }
        };

        Ok(Node::new(modifiers, key))
    }
}

impl FromKeyMap for KeyEvent {
    /// Converts a `KeyMap` into a `KeyEvent`.
    ///
    /// Returns [`Error::UnsupportedKey`] if the conversion is not possible.
    fn from_keymap(keymap: KeyMap) -> Result<Self, Error> {
        let key = match keymap.key {
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
                    "Group {group:?} not supported. Cannot map char group back to KeyEvent"
                )))
            }
        };

        match key {
            KeyEvent::Char(c) => {
                if keymap.modifiers & Modifier::Alt as u8 != 0 {
                    Ok(KeyEvent::Alt(c))
                } else if keymap.modifiers & Modifier::Ctrl as u8 != 0 {
                    Ok(KeyEvent::Ctrl(c))
                } else if keymap.modifiers & Modifier::Shift as u8 != 0 {
                    Ok(KeyEvent::Char(c.to_ascii_uppercase()))
                } else {
                    Ok(key)
                }
            }
            _ => Ok(key),
        }
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
            assert_eq!(key.to_keymap().unwrap(), node);
        });
    }
}
