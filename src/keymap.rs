//! # KeyMap Library
//!
//! This library provides functionality for parsing and working with keymaps.
use keymap_parser::{parser::ParseError, Node};

pub trait FromKeyMap: Sized {
    fn from_keymap(keymap: KeyMap) -> Result<Self, Error>;
}

/// Converts a backend-specific key event into a [`KeyMap`].
pub trait IntoKeyMap {
    fn into_keymap(self) -> Result<KeyMap, Error>;
}

pub trait ToKeyMap {
    fn to_keymap(&self) -> Result<KeyMap, Error>;
}

#[derive(Debug)]
pub enum Error {
    Parse(ParseError),
    UnsupportedKey(String),
}

pub type KeyMap = Node;
