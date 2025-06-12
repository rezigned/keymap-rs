//! Key event parsing and conversion for the `wasm` backend.
//!
//! This module bridges `web_sys::KeyboardEvent` with a backend-agnostic
//! representation (`KeyMap`) used for keybinding configuration and matching.
//! It enables parsing human-readable key definitions and converting between
//! representations suitable for UI and configuration layers.
//!
//! # Key Features
//! - `parse`: Parses a string key representation (e.g., "Ctrl+S") into a `KeyboardEvent`.
//! - Implements `IntoKeyMap`, `ToKeyMap`, and `FromKeyMap` for `KeyboardEvent`.
//! - Converts between `KeyboardEvent` (from web_sys) and internal `KeyMap` format.
//!
//! # Limitations
//! - Some `KeyboardEvent` variants are not supported and will return an error.
//! - Key groups (e.g., `@any`) are not reversible to `KeyboardEvent` due to the loss of specificity.
//!
//! # Examples
//!
//! Parsing from a string:
//! ```
//! use keymap::backend::wasm::parse;
//! use web_sys::KeyboardEvent;
//!
//! let event = parse("alt-b").unwrap();
//! assert_eq!(event.key(), "b");
//! assert_eq!(event.alt_key(), true);
//! ```
use keymap_parser::{self as parser, Key, Modifier, Node};
use web_sys::{KeyboardEvent, KeyboardEventInit};

use crate::{
    keymap::{FromKeyMap, IntoKeyMap, KeyMap, ToKeyMap},
    Error,
};

/// Parses a string keybinding (e.g., `"ctrl-c"`, `"f1"`, `"alt+backspace"`) into a `KeyboardEvent`.
///
/// This uses the `keymap_parser` crate to interpret the string and maps it into a `KeyboardEvent`.
/// Returns an error if parsing fails or the resulting representation cannot be converted.
///
/// # Errors
/// - Returns `Error::Parse` if the string cannot be parsed.
/// - Returns `Error::UnsupportedKey` if the key or modifiers are unsupported.
pub fn parse(s: &str) -> Result<KeyboardEvent, Error> {
    parser::parse(s)
        .map_err(Error::Parse)
        .and_then(KeyboardEvent::from_keymap)
}

impl IntoKeyMap for KeyboardEvent {
    /// Converts a `KeyboardEvent` into a `KeyMap`.
    ///
    /// Internally delegates to `ToKeyMap`.
    fn into_keymap(self) -> Result<KeyMap, Error> {
        self.to_keymap()
    }
}

impl ToKeyMap for KeyboardEvent {
    /// Converts a `KeyboardEvent` to the `KeyMap` format.
    ///
    /// # Errors
    /// - Returns `Error::UnsupportedKey` if the `KeyboardEvent` variant is not supported for conversion.
    fn to_keymap(&self) -> Result<KeyMap, Error> {
        ToKeyMap::to_keymap(&self)
    }
}

impl ToKeyMap for &KeyboardEvent {
    fn to_keymap(&self) -> Result<KeyMap, Error> {
        let modifiers = modifiers_from_backend(self);

        let key = match self.key().as_str() {
            // Backtab = Tab + Shift
            "Tab" if modifiers & Modifier::Shift as u8 != 0 => Key::BackTab,
            "Backspace" => Key::Backspace,
            "Delete" => Key::Delete,
            "ArrowDown" => Key::Down,
            "End" => Key::End,
            "Enter" => Key::Enter,
            "Escape" => Key::Esc,
            "Home" => Key::Home,
            "Insert" => Key::Insert,
            "ArrowLeft" => Key::Left,
            "PageDown" => Key::PageDown,
            "PageUp" => Key::PageUp,
            "ArrowRight" => Key::Right,
            " " => Key::Space,
            "Tab" => Key::Tab,
            "ArrowUp" => Key::Up,
            // Fx
            s if s.starts_with('F') && s.len() > 1 => {
                if let Ok(n) = s[1..].parse::<u8>() {
                    Key::F(n)
                } else {
                    return Err(Error::UnsupportedKey(format!(
                        "Unsupported KeyboardEvent key: {s:?}"
                    )));
                }
            }
            s => Key::Char(s.chars().next().unwrap_or_default()),
        };

        Ok(Node::new(modifiers_from_backend(self), key))
    }
}

impl FromKeyMap for KeyboardEvent {
    /// Converts a `KeyMap` back into a `KeyboardEvent`.
    ///
    /// # Errors
    /// - Returns `Error::UnsupportedKey` if the `KeyMap` contains a `Group`, which cannot be
    ///   reversed into a concrete `KeyboardEvent`.
    fn from_keymap(keymap: KeyMap) -> Result<Self, Error> {
        let key_str = match keymap.key {
            Key::BackTab => "Tab".to_string(), // No direct equivalent, mapping to Tab
            Key::Backspace => "Backspace".to_string(),
            Key::Char(c) => c.to_string(),
            Key::Delete => "Delete".to_string(),
            Key::Down => "ArrowDown".to_string(),
            Key::End => "End".to_string(),
            Key::Enter => "Enter".to_string(),
            Key::Esc => "Escape".to_string(),
            Key::F(n) => format!("F{n}"),
            Key::Home => "Home".to_string(),
            Key::Insert => "Insert".to_string(),
            Key::Left => "ArrowLeft".to_string(),
            Key::PageDown => "PageDown".to_string(),
            Key::PageUp => "PageUp".to_string(),
            Key::Right => "ArrowRight".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::Space => " ".to_string(),
            Key::Up => "ArrowUp".to_string(),
            Key::Group(group) => {
                return Err(Error::UnsupportedKey(format!(
                "Group {group:?} not supported. There's no way to map char group back to KeyboardEvent"
            )))
            }
        };

        let event_init = KeyboardEventInit::new();
        event_init.set_key(&key_str);
        event_init.set_alt_key(keymap.modifiers & Modifier::Alt as u8 != 0);
        event_init.set_ctrl_key(keymap.modifiers & Modifier::Ctrl as u8 != 0);
        event_init.set_meta_key(keymap.modifiers & Modifier::Cmd as u8 != 0);
        event_init.set_shift_key(keymap.modifiers & Modifier::Shift as u8 != 0);

        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &event_init)
            .map_err(|e| Error::UnsupportedKey(format!("Failed to create KeyboardEvent: {:?}", e)))
    }
}

/// Converts a `KeyboardEvent` into a `parser::Modifiers` bitfield.
fn modifiers_from_backend(value: &KeyboardEvent) -> parser::Modifiers {
    let mut modifiers = 0;
    if value.alt_key() {
        modifiers |= Modifier::Alt as u8;
    }
    if value.ctrl_key() {
        modifiers |= Modifier::Ctrl as u8;
    }
    if value.meta_key() {
        modifiers |= Modifier::Cmd as u8;
    }
    if value.shift_key() {
        modifiers |= Modifier::Shift as u8;
    }
    modifiers
}

#[cfg(all(target_arch = "wasm32", test))]
mod tests {
    use super::*;
    use keymap_parser as parser;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn keyboard_event(key: &str, alt: bool, ctrl: bool, shift: bool, meta: bool) -> KeyboardEvent {
        let event_init = KeyboardEventInit::new();
        event_init.set_key(key);
        event_init.set_alt_key(alt);
        event_init.set_ctrl_key(ctrl);
        event_init.set_shift_key(shift);
        event_init.set_meta_key(meta);
        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &event_init).unwrap()
    }

    #[wasm_bindgen_test]
    fn test_to_keymap() {
        // Test cases as tuples: (key, modifiers string for parser)
        [
            ("a", true, true, true, false, "alt-ctrl-shift-a"),
            ("F1", false, false, false, false, "f1"),
            ("Delete", false, false, false, false, "del"),
        ]
        .into_iter()
        .for_each(|(key, alt, ctrl, shift, meta, expected)| {
            let event = keyboard_event(key, alt, ctrl, shift, meta);
            let expected_keymap = parser::parse(expected).unwrap();

            assert_eq!(event.to_keymap().unwrap(), expected_keymap);
        });
    }

    #[wasm_bindgen_test]
    fn test_from_keymap() {
        let keymap = parser::parse("alt-ctrl-shift-a").unwrap();
        let event = KeyboardEvent::from_keymap(keymap).unwrap();

        assert_eq!(event.key(), "a");
        assert!(event.alt_key());
        assert!(event.ctrl_key());
        assert!(event.shift_key());
        assert!(!event.meta_key());
    }
}
