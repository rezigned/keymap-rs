//! # KeyMap Library
//!
//! This library provides functionality for parsing and working with keymaps.
use std::fmt;
use keymap_parser::{parser::{self, ParseError}, Node};

pub trait IntoKeyMap {
    fn into_keymap(self) -> Result<KeyMap, Error>;
}

#[derive(Debug)]
pub enum Error {
    Parse(ParseError),
    UnsupportedKey(String),
}

// Main KeyMap struct
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct KeyMap(pub Node);

/// Converts a [`Node`] into a [`KeyMap`].
impl From<Node> for KeyMap {
    fn from(value: Node) -> Self {
        Self(value)
    }
}

impl fmt::Display for KeyMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Parses a single keymap from a string
pub fn parse(s: &str) -> Result<KeyMap, ParseError> {
    parser::parse(s).map(KeyMap)
}

/// Parses a sequence of keymaps from a string
pub fn parse_seq(s: &str) -> Result<Vec<KeyMap>, ParseError> {
    parser::parse_seq(s).map(|v| v.into_iter().map(KeyMap).collect())
}
