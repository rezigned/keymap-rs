//! # WASM Backend for Keymap
//!
//! This module provides a backend for handling keyboard events in a WebAssembly (WASM)
//! environment, specifically targeting web browsers. It uses the `web-sys` crate
//! to interface with JavaScript `KeyboardEvent` objects.
//!
//! ## Features
//! - Implements `ToKeyMap` for `web_sys::KeyboardEvent`, allowing direct conversion
//!   of browser key events into a `KeyMap` representation. This `KeyMap` can be used
//!   for logging, debugging, or other logic if needed.
//! - Implements `FromKeyMap` for `web_sys::KeyboardEvent`, enabling the creation of
//!   `KeyboardEvent` instances from `KeyMap` values. This involves translating
//!   `KeyMap` properties to `web_sys::KeyboardEventInit` and then constructing
//!   the `KeyboardEvent`.
//! - Provides a `parse` function that converts string representations of keybindings
//!   (e.g., "ctrl-c") into `web_sys::KeyboardEvent` instances. This function uses
//!   `keymap_parser` to obtain a `KeyMap` and then leverages the `FromKeyMap`
//!   implementation to create the `KeyboardEvent`.
//! - Integrates with the `Config` struct via the `BackendConfig` trait. For this backend,
//!   `BackendConfig::Key` is `web_sys::KeyboardEvent`. When `config.get(event)` is called
//!   with a live browser `KeyboardEvent` or one created by `parse`, the event is
//!   internally converted to a `KeyMap` (using its `ToKeyMap` implementation) for lookup
//!   against the configured keybindings.
//!
//! ## Usage
//!
//! ### Handling Live Browser Events
//! To convert a live browser `KeyboardEvent` to `KeyMap` (e.g., for inspection),
//! ensure `ToKeyMap` is in scope:
//!
//! ```rust,ignore
//! use keymap::keymap::ToKeyMap;
//! use web_sys::KeyboardEvent; // Assuming you have this from an event listener
//!
//! // fn process_event(live_js_event: &KeyboardEvent) {
//! //     // Using with Config:
//! //     // let config: keymap::Config<String> = keymap::Config::default();
//! //     // if let Some(action) = config.get(live_js_event) {
//! //     //     println!("Action: {}", action);
//! //     // }
//!
//! //     // Optional: Convert to KeyMap for other purposes
//! //     match live_js_event.to_keymap() {
//! //         Ok(keymap_repr) => {
//! //             println!("KeyMap representation: {:?}", keymap_repr);
//! //         }
//! //         Err(e) => {
//! //             eprintln!("Error converting live event to KeyMap: {:?}", e);
//! //         }
//! //     }
//! // }
//! ```
//!
//! ### Parsing String Keybindings
//! To parse a string keybinding into a `web_sys::KeyboardEvent` that can be used with `Config`:
//! ```rust,ignore
//! // use keymap::backend::parse; // If wasm feature is active and correctly resolved
//! // use web_sys::KeyboardEvent;
//!
//! // match parse("ctrl-alt-delete") {
//! //     Ok(parsed_event) => {
//! //         // parsed_event is a web_sys::KeyboardEvent
//! //         // This can be used with config.get(&parsed_event)
//! //         println!("Parsed into KeyboardEvent: key='{}'", parsed_event.key());
//! //     }
//! //     Err(e) => {
//! //         eprintln!("Error parsing key string: {:?}", e);
//! //     }
//! // }
//! ```
//!
//! To enable this backend, ensure the `wasm` feature is activated for the `keymap` crate
//! in your `Cargo.toml`, and that no other backend features (`crossterm`, `termion`) are
//! simultaneously active. See the main `README.md` for more on feature selection.

// src/backend/wasm.rs

use keymap_parser::{self as parser, Key, Modifier, Node};
use web_sys::{KeyboardEvent, KeyboardEventInit}; // Ensure KeyboardEventInit is imported

use crate::{
    config::BackendConfig,
    keymap::{FromKeyMap, IntoKeyMap, KeyMap, ToKeyMap}, // Add FromKeyMap here
    Config,
    Error,
    Item,
};

// impl ToKeyMap for KeyboardEvent remains as previously defined
impl ToKeyMap for KeyboardEvent {
    fn to_keymap(&self) -> Result<KeyMap, Error> {
        let key_str = self.key();
        let key_code_str = self.code();

        let key = match key_str.as_str() {
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
            "Tab" => Key::Tab,
            "ArrowUp" => Key::Up,
            _ if key_code_str == "Space" => Key::Space,
            s if s.starts_with("F") && s.len() > 1 && s[1..].chars().all(char::is_numeric) => {
                if let Ok(n) = s[1..].parse::<u8>() {
                    if n >= 1 && n <= 12 {
                        Key::F(n)
                    } else {
                        return Err(Error::UnsupportedKey(format!("Unsupported F key: F{}", n)));
                    }
                } else {
                    return Err(Error::UnsupportedKey(format!("Invalid F key format: {}", s)));
                }
            }
            s if s.chars().count() == 1 => Key::Char(s.chars().next().unwrap()),
            _ => return Err(Error::UnsupportedKey(format!(
                "Unsupported key: '{}' (code: '{}')", key_str, key_code_str
            ))),
        };

        let mut modifiers_val = 0;
        if self.alt_key() {
            modifiers_val |= Modifier::Alt as u8;
        }
        if self.ctrl_key() {
            modifiers_val |= Modifier::Ctrl as u8;
        }
        if self.meta_key() {
            modifiers_val |= Modifier::Cmd as u8;
        }
        if self.shift_key() {
            modifiers_val |= Modifier::Shift as u8;
        }

        Ok(Node { key, modifiers: modifiers_val })
    }
}

impl FromKeyMap for KeyboardEvent {
    fn from_keymap(keymap: KeyMap) -> Result<Self, Error> {
        let event_init = KeyboardEventInit::new();

        let (key_val, code_val) = match keymap.key {
            Key::Backspace => ("Backspace".to_string(), "Backspace".to_string()),
            Key::Delete => ("Delete".to_string(), "Delete".to_string()),
            Key::Down => ("ArrowDown".to_string(), "ArrowDown".to_string()),
            Key::End => ("End".to_string(), "End".to_string()),
            Key::Enter => ("Enter".to_string(), "Enter".to_string()),
            Key::Esc => ("Escape".to_string(), "Escape".to_string()),
            Key::Home => ("Home".to_string(), "Home".to_string()),
            Key::Insert => ("Insert".to_string(), "Insert".to_string()),
            Key::Left => ("ArrowLeft".to_string(), "ArrowLeft".to_string()),
            Key::PageDown => ("PageDown".to_string(), "PageDown".to_string()),
            Key::PageUp => ("PageUp".to_string(), "PageUp".to_string()),
            Key::Right => ("ArrowRight".to_string(), "ArrowRight".to_string()),
            Key::Tab => ("Tab".to_string(), "Tab".to_string()),
            Key::BackTab => {
                event_init.set_shift_key(true);
                ("Tab".to_string(), "Tab".to_string())
            }
            Key::Up => ("ArrowUp".to_string(), "ArrowUp".to_string()),
            Key::Space => (" ".to_string(), "Space".to_string()),
            Key::F(n) => (format!("F{}", n), format!("F{}", n)),
            Key::Char(c) => {
                let char_str = c.to_string();
                let code_str = if c.is_ascii_alphabetic() {
                    format!("Key{}", c.to_ascii_uppercase())
                } else if c.is_ascii_digit() {
                    format!("Digit{}", c)
                } else {
                    char_str.clone() // Simplified `code` for other chars
                };
                (char_str, code_str)
            }
            Key::Group(_) => return Err(Error::UnsupportedKey("Key groups cannot be converted to a KeyboardEvent".to_string())),
        };

        event_init.set_key(&key_val);
        event_init.set_code(&code_val);

        if keymap.modifiers & (Modifier::Alt as u8) != 0 {
            event_init.set_alt_key(true);
        }
        if keymap.modifiers & (Modifier::Ctrl as u8) != 0 {
            event_init.set_ctrl_key(true);
        }
        if keymap.modifiers & (Modifier::Cmd as u8) != 0 {
            event_init.set_meta_key(true);
        }
        if keymap.modifiers & (Modifier::Shift as u8) != 0 {
            event_init.set_shift_key(true);
        }

        event_init.set_bubbles(true);
        event_init.set_cancelable(true);

        KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &event_init)
            .map_err(|js_val| Error::UnsupportedKey(format!("Failed to create KeyboardEvent from KeyMap: {:?}", js_val)))
    }
}

// impl IntoKeyMap for KeyboardEvent remains as previously defined
impl IntoKeyMap for KeyboardEvent {
    fn into_keymap(self) -> Result<KeyMap, Error> {
        self.to_keymap()
    }
}

/// Parses a string keybinding (e.g., `"ctrl-c"`, `"f1"`) into a `web_sys::KeyboardEvent`.
pub fn parse(s: &str) -> Result<KeyboardEvent, Error> {
    let keymap = parser::parse(s).map_err(Error::Parse)?;
    // Now uses the FromKeyMap trait method.
    // KeyboardEvent::from_keymap(keymap) // This is how you call it if FromKeyMap is in scope.
    // Or, if FromKeyMap is brought into scope for KeyboardEvent,
    // you could potentially do keymap.try_into() if a TryFrom<KeyMap> for KeyboardEvent wrapper exists.
    // Given the current setup, explicitly calling from_keymap is clear.
    KeyboardEvent::from_keymap(keymap)
}

// BackendConfig impl remains here, will be verified in the next step
impl<T> BackendConfig<T> for Config<T> {
    type Key = KeyboardEvent;

    fn get(&self, key: &Self::Key) -> Option<&T> {
        match key.to_keymap() {
            Ok(keymap) => self.get_by_keymap(&keymap),
            Err(_) => None, // Or handle error appropriately
        }
    }

    fn get_seq(&self, keys: &[Self::Key]) -> Option<&T> {
        let nodes = keys
            .iter()
            .map(|key| key.to_keymap().ok())
            .collect::<Option<Vec<_>>>()?;
        self.get_item_by_keymaps(&nodes).map(|(t, _)| t)
    }

    fn get_item(&self, key: &Self::Key) -> Option<(&T, &Item)> {
        match key.to_keymap() {
            Ok(keymap) => self.get_item_by_keymap(&keymap),
            Err(_) => None, // Or handle error appropriately
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Brings in parse, ToKeyMap for KeyboardEvent
    // For verifying intermediate KeyMap logic if needed, though not directly used in assertions here.
    // use keymap_parser::{Key, Modifier};
    // For type hints and clarity, though KeyboardEvent is usually inferred.
    // use web_sys::KeyboardEvent;

    // Helper to create KeyboardEventInit for comparison if needed, though we'll check event properties directly.
    // Note: Actually creating KeyboardEvents for `ToKeyMap` tests is hard without wasm-bindgen-test.
    // These tests focus on the `parse` function (String -> KeyMap -> KeyboardEventInit -> KeyboardEvent).

    #[test]
    fn test_parse_returns_keyboard_event() {
        // Test case 1: Simple character 'a'
        let result_event_a = parse("a");
        assert!(result_event_a.is_ok(), "Parsing 'a' failed: {:?}", result_event_a.err());
        if let Ok(event_a) = result_event_a {
            assert_eq!(event_a.key(), "a", "Key value for 'a'");
            assert_eq!(event_a.code(), "KeyA", "Code value for 'a'"); // Based on current parse logic
            assert!(!event_a.alt_key() && !event_a.ctrl_key() && !event_a.meta_key() && !event_a.shift_key(), "Modifiers for 'a'");
            assert_eq!(event_a.type_(), "keydown", "Event type for 'a'"); // Default type in parse
        }

        // Test case 2: "ctrl-shift-b"
        let result_event_cs_b = parse("ctrl-shift-b");
        assert!(result_event_cs_b.is_ok(), "Parsing 'ctrl-shift-b' failed: {:?}", result_event_cs_b.err());
        if let Ok(event_cs_b) = result_event_cs_b {
            assert_eq!(event_cs_b.key(), "b", "Key value for 'ctrl-shift-b'");
            assert_eq!(event_cs_b.code(), "KeyB", "Code value for 'ctrl-shift-b'");
            assert!(!event_cs_b.alt_key(), "Alt key for 'ctrl-shift-b'");
            assert!(event_cs_b.ctrl_key(), "Ctrl key for 'ctrl-shift-b'");
            assert!(!event_cs_b.meta_key(), "Meta key for 'ctrl-shift-b'");
            assert!(event_cs_b.shift_key(), "Shift key for 'ctrl-shift-b'");
        }

        // Test case 3: "alt-F5"
        let result_event_alt_f5 = parse("alt-F5");
        assert!(result_event_alt_f5.is_ok(), "Parsing 'alt-F5' failed: {:?}", result_event_alt_f5.err());
        if let Ok(event_alt_f5) = result_event_alt_f5 {
            assert_eq!(event_alt_f5.key(), "F5", "Key value for 'alt-F5'");
            assert_eq!(event_alt_f5.code(), "F5", "Code value for 'alt-F5'");
            assert!(event_alt_f5.alt_key(), "Alt key for 'alt-F5'");
            assert!(!event_alt_f5.ctrl_key() && !event_alt_f5.meta_key() && !event_alt_f5.shift_key(), "Other modifiers for 'alt-F5'");
        }

        // Test case 4: "enter"
        let result_event_enter = parse("enter");
        assert!(result_event_enter.is_ok(), "Parsing 'enter' failed: {:?}", result_event_enter.err());
        if let Ok(event_enter) = result_event_enter {
            assert_eq!(event_enter.key(), "Enter", "Key value for 'enter'");
            assert_eq!(event_enter.code(), "Enter", "Code value for 'enter'");
            assert!(!event_enter.alt_key() && !event_enter.ctrl_key() && !event_enter.meta_key() && !event_enter.shift_key(), "Modifiers for 'enter'");
        }

        // Test case 5: "shift-tab" (which should map to BackTab then to Tab with Shift)
        let result_event_sh_tab = parse("shift-tab");
        assert!(result_event_sh_tab.is_ok(), "Parsing 'shift-tab' failed: {:?}", result_event_sh_tab.err());
        if let Ok(event_sh_tab) = result_event_sh_tab {
            // Assuming keymap_parser("shift-tab") -> KeyMap { key: Key::Tab, modifiers: Shift }
            // OR keymap_parser("shift-tab") -> KeyMap { key: Key::BackTab, modifiers: 0 }
            // Current `parse` logic for Key::BackTab: sets shift_key(true), key("Tab"), code("Tab")
            assert_eq!(event_sh_tab.key(), "Tab", "Key value for 'shift-tab'");
            assert_eq!(event_sh_tab.code(), "Tab", "Code value for 'shift-tab'");
            assert!(event_sh_tab.shift_key(), "Shift key for 'shift-tab'");
            assert!(!event_sh_tab.alt_key() && !event_sh_tab.ctrl_key() && !event_sh_tab.meta_key(), "Other modifiers for 'shift-tab'");
        }

        // Test case 6: Space
        // Assuming keymap_parser("space") -> KeyMap { key: Key::Space, modifiers: 0 }
        let result_event_space = parse("space");
        assert!(result_event_space.is_ok(), "Parsing 'space' failed: {:?}", result_event_space.err());
        if let Ok(event_space) = result_event_space {
            assert_eq!(event_space.key(), " ", "Key value for 'space'");
            assert_eq!(event_space.code(), "Space", "Code value for 'space'");
            assert!(!event_space.alt_key() && !event_space.ctrl_key() && !event_space.meta_key() && !event_space.shift_key(), "Modifiers for 'space'");
        }

        // Test case 7: Invalid input string for keymap_parser
        let result_invalid = parse("invalid-gibberish-string");
        assert!(result_invalid.is_err(), "Parsing invalid string should fail");
        match result_invalid.err().unwrap() {
            Error::Parse(_) => {} // Expected error type
            e => panic!("Expected Error::Parse, got {:?}", e),
        }
    }

    // It's difficult to test `impl ToKeyMap for KeyboardEvent` here because creating
    // a `KeyboardEvent` with all desired properties (especially `key` and `code`)
    // without wasm-bindgen / JS environment is not straightforward for mock testing.
    // The `parse` tests above indirectly cover the logic that translates `KeyMap`
    // to `KeyboardEventInit` properties, which is the reverse of `ToKeyMap`'s core logic.
    // A full test of `ToKeyMap for KeyboardEvent` would typically be in a wasm_bindgen_test.
    #[test]
    fn note_on_to_keymap_testing() {
        // This test is just a placeholder for the comment.
        assert!(true, "See comment about testing ToKeyMap for KeyboardEvent.");
    }
}
