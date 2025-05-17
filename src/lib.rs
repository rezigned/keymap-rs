#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

//! # KeyMap Library
//!
//! This library provides functionality for parsing and working with keymaps.
use keymap_parser::{parser::ParseError, Node};
use std::fmt;

// Re-exports
pub use config::Item;
pub use keymap_parser::parser;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use keymap_derive::KeyMap;

// Modules
pub mod backend;
mod config;

// Main KeyMap struct
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct KeyMap(Node);

// Implementations
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
    keymap_parser::parse(s).map(KeyMap)
}

/// Parses a sequence of keymaps from a string
pub fn parse_seq(s: &str) -> Result<Vec<KeyMap>, ParseError> {
    keymap_parser::parse_seq(s).map(|v| v.into_iter().map(KeyMap).collect())
}

/// Allows calling proc macro in main crate (for testing)
#[cfg(test)]
extern crate self as keymap;
