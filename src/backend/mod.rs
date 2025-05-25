//! # Backends
#[cfg(feature = "crossterm")]
mod crossterm;

#[cfg(feature = "crossterm")]
pub use self::crossterm::parse as crossterm_parse;

#[cfg(feature = "termion")]
mod termion;

#[cfg(feature = "termion")]
pub use self::termion::parse as termion_parse;

use std::fmt;

use keymap_parser::{parser::ParseError, Modifiers, Node};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct KeyMap(Node);

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

pub fn parse(s: &str) -> Result<KeyMap, ParseError> {
    keymap_parser::parse(s).map(KeyMap)
}

pub fn parse_seq(s: &str) -> Result<Vec<KeyMap>, ParseError> {
    keymap_parser::parse_seq(s).map(|v| v.into_iter().map(KeyMap).collect())
}

/// A wrapper that allows conversion between terminal backend's modifier
/// and Node's modifier.
struct NodeModifiers(Modifiers);

impl From<NodeModifiers> for Modifiers {
    fn from(value: NodeModifiers) -> Self {
        value.0
    }
}

impl From<Modifiers> for NodeModifiers {
    fn from(value: Modifiers) -> Self {
        Self(value)
    }
}
