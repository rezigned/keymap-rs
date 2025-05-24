//! # Backends
use std::{fmt::{self, Display}, hash::{Hash, Hasher}};

#[cfg(feature = "crossterm")]
mod crossterm;

#[cfg(feature = "crossterm")]
pub use self::crossterm::{KeyMap, parse};

#[cfg(feature = "termion")]
mod termion;

#[cfg(feature = "termion")]
pub use self::termion::{KeyMap, parse};

use keymap_parser::{Node, Modifiers};

pub struct KeyMap2(Node);

#[derive(Debug, Eq)]
pub struct Key<T> {
    event: T,
    node: Option<Node>
}

impl<T: PartialEq> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event
    }
}

impl<T: Hash> Hash for Key<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.event.hash(state);
    }
}

impl Display for KeyMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.node {
            Some(node) => write!(f, "{node}"),
            None => write!(f, ""),
        }
    }
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

/// Parses a whitespace separated sequence of keys.
///
/// This splits the given string on whitespace and parses each component with
/// `parse`. The results are collected into a `Vec<KeyMap>`.
///
/// # Errors
///
/// This function will return an error if any of the components fail to parse.
pub fn parse_seq(s: &str) -> Result<Vec<KeyMap>, pom::Error> {
    str::split_whitespace(s)
        .map(parse)
        .collect()
}

#[test]
fn test_parse_seq() {
    [
        ("ctrl-b", Ok(vec![
            parse("ctrl-b").unwrap(),
        ])),
        ("ctrl-b l", Ok(vec![
            parse("ctrl-b").unwrap(),
            parse("l").unwrap(),
        ])),
        ("ctrl-b -l", Err(parse("-l").unwrap_err())),
    ].map(|(s, v)| {
        assert_eq!(parse_seq(s), v)
    });
}
