#[cfg(feature = "crossterm")]
mod crossterm;

use std::{hash::{Hasher, Hash}, fmt::{Display, self}};

#[cfg(feature = "crossterm")]
pub use self::crossterm::KeyMap;

#[cfg(feature = "termion")]
mod termion;

#[cfg(feature = "termion")]
pub use self::termion::KeyMap;

use crate::parser::Node;

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
