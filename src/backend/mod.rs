//! # Backends
use std::{hash::{Hasher, Hash}, fmt::{Display, self}};

#[cfg(feature = "crossterm")]
mod crossterm;

#[cfg(feature = "crossterm")]
pub use self::crossterm::{KeyMap, parse};

#[cfg(feature = "termion")]
mod termion;

#[cfg(feature = "termion")]
pub use self::termion::{KeyMap, parse};

use crate::parser::{Node, Modifiers};

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

/// A wrapper that allows conversion between backend's modifier
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
