//! # `KeyMap` Library
//!
//! This library provides a unified abstraction over key input representations from multiple backends,
//! such as [`crossterm`](https://crates.io/crates/crossterm), WebAssembly environments, and more.
//! It enables parsing, transforming, and serializing key input events to and from a common `KeyMap`
//! format, which is represented by a node tree from the `keymap_parser` crate.
//!
//! The main goal is to decouple application logic from backend-specific input handling, enabling easier
//! testing, configuration, and cross-platform support.

use keymap_parser::{node::Key, parser::ParseError, Node};

/// A type alias for a parsed keymap node tree.
///
/// This represents a keymap in an abstract format, using the [`Node`] type
/// from the `keymap_parser` crate.
pub type KeyMap = Node;

/// A trait for converting a [`KeyMap`] into a backend-specific key event type.
///
/// This trait should be implemented by types such as `crossterm::event::KeyEvent` or other
/// platform-native key event types. It allows translating a backend-agnostic keymap (typically
/// parsed from user configuration) into a format usable by a specific input backend.
///
/// # Errors
///
/// Returns [`Error::Parse`] if the `KeyMap` is invalid, or [`Error::UnsupportedKey`] if it
/// contains keys or structures not supported by the target backend.
pub trait FromKeyMap: Sized {
    fn from_keymap(keymap: KeyMap) -> Result<Self, Error>;
}

/// A trait for converting a backend-specific key event into a [`KeyMap`].
///
/// This trait should be implemented by types that represent native input events
/// from a backend, such as `crossterm`, web (WASM), or others. It provides a way
/// to unify platform-specific key events into a common `KeyMap` representation.
///
/// # Errors
///
/// Returns an [`Error`] if the conversion fails due to an unsupported or invalid key.
pub trait IntoKeyMap {
    fn into_keymap(self) -> Result<KeyMap, Error>;
}

/// A trait for converting a backend-specific key type into a [`KeyMap`].
///
/// This is typically implemented by types like `crossterm::event::KeyEvent`,
/// allowing the transformation of native input representations into the
/// abstract `KeyMap` format. This is useful for tasks such as exporting key
/// configurations, logging, or interfacing with cross-platform logic.
///
/// # Errors
///
/// Returns an [`Error`] if the conversion fails due to unsupported or unrepresentable keys.
pub trait ToKeyMap {
    /// Converts the type into a [`KeyMap`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if conversion fails due to unsupported or invalid keys.
    fn to_keymap(&self) -> Result<KeyMap, Error>;
}

/// A trait for types that can be extracted from a matched key group node.
///
/// When a variant field is bound via a key group (e.g. `@digit`, `@any`), the
/// derive macro calls `KeyGroupValue::from_keymap_node` on the matched [`KeyMap`]
/// node to produce the field value. This replaces the old string-based type
/// inspection, so type aliases (e.g. `type Bar = u32`) work transparently as
/// long as the underlying type implements this trait.
///
/// # Built-in implementations
///
/// | Type    | Behaviour                                                  |
/// |---------|------------------------------------------------------------|
/// | `char`  | Returns the matched character, or `'\0'` as the default.   |
/// | `u8`    | Parses the digit character as a decimal number.            |
/// | `u16`   | Same as `u8`, widened to `u16`.                            |
/// | `u32`   | Same as `u8`, widened to `u32`.                            |
/// | `u64`   | Same as `u8`, widened to `u64`.                            |
/// | `usize` | Same as `u8`, widened to `usize`.                          |
///
/// # Example
///
/// ```ignore
/// use keymap::KeyGroupValue;
///
/// type MyDigit = u32;
///
/// #[derive(keymap::KeyMap)]
/// enum Action {
///     #[key("@digit")]
///     Count(MyDigit),   // works because u32 implements KeyGroupValue
/// }
/// ```
pub trait KeyGroupValue: Default {
    /// Extracts a value from the matched key node.
    ///
    /// Receives the [`KeyMap`] node that was matched by the key group pattern.
    /// Returns `Self::default()` when the node does not carry a suitable value.
    fn from_keymap_node(node: &KeyMap) -> Self;
}

impl KeyGroupValue for char {
    fn from_keymap_node(node: &KeyMap) -> Self {
        match node.key {
            Key::Char(c) => c,
            _ => '\0',
        }
    }
}

macro_rules! impl_key_group_value_uint {
    ($($t:ty),+) => {
        $(
            impl KeyGroupValue for $t {
                fn from_keymap_node(node: &KeyMap) -> Self {
                    match node.key {
                        Key::Char(c) => c.to_digit(10).unwrap_or(0) as $t,
                        _ => 0,
                    }
                }
            }
        )+
    };
}

impl_key_group_value_uint!(u8, u16, u32, u64, usize);

/// Represents errors that can occur during keymap parsing or conversion.
#[derive(Debug)]
pub enum Error {
    /// A parsing error occurred while processing a `KeyMap`.
    Parse(ParseError),

    /// The key or structure is not supported by the current backend.
    UnsupportedKey(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "{e}"),
            Error::UnsupportedKey(k) => write!(f, "{k}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(e) => Some(e),
            Error::UnsupportedKey(_) => None,
        }
    }
}
