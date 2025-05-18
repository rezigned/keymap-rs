use std::{fmt::{Display, Formatter}, ops::BitOr};

use serde::{Deserialize, Deserializer, de};
use strum_macros::{AsRefStr, Display, EnumString};

use crate::parse;

/// Key separator
pub(crate) const KEY_SEP: char = '-';

/// Represents a key with its modifiers.
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Node {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl Node {
    pub fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }
}

impl From<Key> for Node {
    fn from(key: Key) -> Self {
        Self {
            modifiers: Modifier::None as u8,
            key,
        }
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, Hash, PartialEq, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum Modifier {
    None = 0b0000,
    Alt = 0b0001,
    Cmd = 0b0010,
    Ctrl = 0b0100,
    Shift = 0b1000,
}

impl BitOr for Modifier {
    type Output = Modifiers;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

pub type Modifiers = u8;

pub(crate) const MODIFIERS: [Modifier; 4] = [
    Modifier::Alt,
    Modifier::Cmd,
    Modifier::Ctrl,
    Modifier::Shift,
];

#[derive(Debug, Display, Eq, Hash, PartialEq, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum Key {
    BackTab,
    Backspace,
    Char(char),
    #[strum(serialize = "del", serialize = "delete")]
    Delete,
    Down,
    End,
    Enter,
    Esc,
    Home,
    F(u8),
    Insert,
    Left,
    PageDown,
    PageUp,
    Right,
    Space,
    Tab,
    Up,
}

/// Deserializes into Node
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        MODIFIERS.iter().for_each(|m| {
            if self.modifiers & *m as u8 != 0 {
                write!(f, "{m}{KEY_SEP}").unwrap();
            }
        });

        match self.key {
            Key::Char(char) => write!(f, "{char}"),
            Key::F(n) => write!(f, "{}{n}", self.key),
            _ => write!(f, "{}", self.key),
        }
    }
}
