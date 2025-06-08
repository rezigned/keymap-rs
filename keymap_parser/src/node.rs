//! Defines the core types for representing and parsing key combinations with modifiers.
//!
//! This module provides the `Node` struct—representing a combination of modifier keys and a key—as well as enums and constants for modifiers and keys.
//! It also implements deserialization and display formatting for these types.
use std::{
    fmt::{Display, Formatter},
    hash::Hash,
    ops::BitOr,
};

use serde::{de, Deserialize, Deserializer};
use strum_macros::{AsRefStr, Display, EnumString};

use crate::parse;

/// Separator character used between modifiers and keys in string representations.
pub(crate) const KEY_SEP: char = '-';

/// Represents a keyboard input node, consisting of modifier keys and a main key.
///
/// For example, "Ctrl-Shift-A" would be represented as a `Node` with the `Ctrl` and `Shift` modifiers and the `Char('A')` key.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Node {
    /// Bitflags representing active modifiers (see [`Modifier`]).
    pub modifiers: Modifiers,
    /// The main key (see [`Key`]).
    pub key: Key,
}

impl Node {
    /// Creates a new `Node` from the given modifiers and key.
    pub fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }
}

impl From<Key> for Node {
    /// Converts a [`Key`] value into a `Node` with no modifiers.
    fn from(key: Key) -> Self {
        Self {
            modifiers: Modifier::None as u8,
            key,
        }
    }
}

/// Modifier keys that can be combined with other keys.
///
/// Each variant is represented as a bitflag.
#[derive(Copy, Clone, Debug, Display, Hash, PartialEq, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum Modifier {
    /// No modifier.
    None = 0b0000,
    /// Alt key.
    Alt = 0b0001,
    /// Command (Meta/Windows) key.
    Cmd = 0b0010,
    /// Control key.
    Ctrl = 0b0100,
    /// Shift key.
    Shift = 0b1000,
}

impl BitOr for Modifier {
    type Output = Modifiers;

    /// Combines two modifiers with a bitwise OR, returning the combined flags as `Modifiers`.
    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

/// Type alias for storing a combination of modifier bitflags.
pub type Modifiers = u8;

/// Array of all possible modifier variants (excluding `None`).
pub(crate) const MODIFIERS: [Modifier; 4] = [
    Modifier::Alt,
    Modifier::Cmd,
    Modifier::Ctrl,
    Modifier::Shift,
];

/// Supported keyboard key types for input nodes.
///
/// This enum includes character keys, function keys, and special keys.
#[derive(Clone, Debug, Display, PartialEq, Eq, Hash, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum Key {
    /// Shift+Tab / Back tab.
    BackTab,
    /// Backspace key.
    Backspace,
    /// Delete key (also accepts "del" as a string).
    #[strum(serialize = "del", serialize = "delete")]
    Delete,
    /// Down arrow key.
    Down,
    /// End key.
    End,
    /// Enter/Return key.
    Enter,
    /// Escape key.
    Esc,
    /// Home key.
    Home,
    /// Insert key.
    Insert,
    /// Left arrow key.
    Left,
    /// Page Down key.
    PageDown,
    /// Page Up key.
    PageUp,
    /// Right arrow key.
    Right,
    /// Space bar.
    Space,
    /// Tab key.
    Tab,
    /// Up arrow key.
    Up,
    /// Function key (e.g., F1-F12).
    F(u8),
    /// A Unicode character key.
    Char(char),
    /// Group
    #[strum(disabled)]
    Group(CharGroup),
}

/// Character group types for pattern matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum CharGroup {
    /// Matches ASCII digits (0-9)
    Digit,
    /// Matches lowercase ASCII letters (a-z)
    Lower,
    /// Matches uppercase ASCII letters (A-Z)
    Upper,
    /// Matches ASCII letters (a-z, A-Z)
    Alpha,
    /// Matches alphanumeric ASCII characters (a-z, A-Z, 0-9)
    Alnum,
    /// Matches any character
    Any,
}

impl CharGroup {
    pub fn matches(&self, c: char) -> bool {
        match self {
            CharGroup::Digit => c.is_ascii_digit(),
            CharGroup::Lower => c.is_ascii_lowercase(),
            CharGroup::Upper => c.is_ascii_uppercase(),
            CharGroup::Alpha => c.is_ascii_alphabetic(),
            CharGroup::Alnum => c.is_ascii_alphanumeric(),
            CharGroup::Any => true,
        }
    }
}

impl std::fmt::Display for CharGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Digit => "digit",
            Self::Lower => "lower",
            Self::Upper => "upper",
            Self::Alpha => "alpha",
            Self::Alnum => "alnum",
            Self::Any => "any",
        };
        write!(f, "@{}", name)
    }
}

/// Custom deserialization for [`Node`] from a string.
///
/// Accepts a string representation (e.g., "Ctrl-Shift-A") and parses it into a `Node`.
impl<'s> Deserialize<'s> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'s>,
    {
        let key = String::deserialize(deserializer)?;
        parse(&key).map_err(de::Error::custom)
    }
}

impl Display for Node {
    /// Formats the node as a human-readable string (e.g., "ctrl-shift-a", "alt-f4").
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        MODIFIERS.iter().for_each(|m| {
            if self.modifiers & *m as u8 != 0 {
                write!(f, "{m}{KEY_SEP}").unwrap();
            }
        });

        match self.key {
            Key::Char(char) => write!(f, "{char}"),
            Key::F(n) => write!(f, "{}{n}", self.key),
            Key::Group(n) => write!(f, "{n}"),
            _ => write!(f, "{}", self.key),
        }
    }
}
