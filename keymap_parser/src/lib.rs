//! # Keymap Parser
//!
//! This crate provides functionality for parsing keymaps from strings.
//! It defines the structures for representing keys, modifiers, and key combinations,
//! and provides a parser for converting strings into these structures.
//!
//! # Examples
//!
//! Parse a keymap string into a `Node`:
//! ```
//! use keymap_parser::{parse, Node, Key, Modifier};
//!
//! let input = "ctrl-alt-f";
//! let node = parse(input).unwrap();
//!
//! assert_eq!(node, Node { modifiers: Modifier::Ctrl | Modifier::Alt, key: Key::Char('f') });
//! ```
//! Parse a sequence of keymap strings into a `Vec<Node>`:
//! ```
//! use keymap_parser::{parse_seq, Node, Key};
//!
//! let input = "g g";
//! let nodes = parse_seq(&input).unwrap();
//!
//! assert_eq!(nodes, vec![
//!     Node::from(Key::Char('g')),
//!     Node::from(Key::Char('g')),
//! ]);
//! ```
pub mod node;
pub mod parser;

pub use node::{Key, Modifier, Modifiers, Node};
pub use parser::{parse, parse_seq};
