//! A trie-based pattern matcher for sequences of input keys (`Node`s).
//!
//! This module provides the [`Matcher`] type, which maps sequences of key inputs to associated values.
//! It internally uses a recursive [`Trie`] structure to efficiently match input patterns.
//!
//! Patterns can include:
//!
//! 1. **Exact keys** — matches a specific input key (e.g., `Key::Char('a')`, `Key::F(1)`).
//! 2. **Character groups** — matches keys falling into categories like `@digit`, `@upper`, or `@any`,
//!    optionally with modifiers (e.g., `ctrl-@any`, `shift-@upper`).
//!
//! The matching logic follows a prioritized order:
//!
//! 1. **Exact match** — if the next input node exactly matches a key in the current trie level.
//! 2. **Group match** — if the next input character matches a character group and modifiers align.
//! 3. **Wildcard group match** — if the group is `@any` with matching modifiers.
//!
//! This ensures more specific patterns take precedence over broader ones.
//!
//! ## Example Patterns
//!
//! | Pattern                  | Input          | Match Result |
//! | ------------------------ | -------------- | ------------ |
//! | ctrl-\@any shift-\@upper | ctrl-x shift-B | true         |
//! | ctrl-\@any shift-\@upper | ctrl-x shift-3 | false        |
//! | a enter                  | a enter        | true         |
//! | a enter                  | a esc          | false        |
//! | @digit                   | '3'            | true         |
//! | @digit                   | 'a'            | false        |
//!
//! Each complete match path in the trie may store an associated value (e.g., action, ID, etc.).
//!
//! See [`Matcher`] for the main interface and [`Trie`] for the underlying structure.
use std::collections::HashMap;

use keymap_parser::node::{CharGroup, Key, Node};

#[derive(Debug)]
struct Trie<T> {
    value: Option<T>,
    exact: HashMap<Node, Trie<T>>,
    groups: Vec<(Node, Trie<T>)>,
}

impl<T> Trie<T> {
    /// Creates a new empty Trie node.
    fn new() -> Self {
        Self {
            value: None,
            exact: HashMap::new(),
            groups: Vec::new(),
        }
    }
}

/// A pattern matcher that maps sequences of `Node`s to values.
///
/// Supports both exact matches and grouped matches (e.g. `CharGroup::Upper`).
#[derive(Debug)]
pub struct Matcher<T> {
    root: Trie<T>,
}

impl<T> Default for Matcher<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<(Vec<Node>, T)> for Matcher<T> {
    fn from_iter<I: IntoIterator<Item = (Vec<Node>, T)>>(iter: I) -> Self {
        let mut matcher = Matcher::new();
        for (pattern, value) in iter {
            matcher.add(pattern, value);
        }
        matcher
    }
}

impl<T> Matcher<T> {
    /// Creates a new, empty matcher.
    pub fn new() -> Self {
        Self { root: Trie::new() }
    }

    /// Adds a pattern and its associated value to the matcher.
    pub fn add(&mut self, pattern: Vec<Node>, value: T) {
        let mut node = &mut self.root;

        for input_node in pattern {
            node = match input_node.key {
                Key::Group(_) => {
                    // Look for an existing group node
                    if let Some(pos) = node.groups.iter().position(|(n, _)| n == &input_node) {
                        &mut node.groups[pos].1
                    } else {
                        node.groups.push((input_node, Trie::new()));
                        &mut node.groups.last_mut().unwrap().1
                    }
                }
                _ => node.exact.entry(input_node).or_insert_with(Trie::new),
            };
        }

        node.value = Some(value);
    }

    /// Attempts to retrieve a value for the given input node sequence.
    pub fn get(&self, nodes: &[Node]) -> Option<&T> {
        search(&self.root, nodes, 0)
    }
}

/// Recursively searches the Trie for a matching value.
///
/// Priority order:
/// 1. Exact match
/// 2. Group match with same modifiers
/// 3. Any-char group match with same modifiers
fn search<'a, T>(node: &'a Trie<T>, nodes: &[Node], pos: usize) -> Option<&'a T> {
    if pos == nodes.len() {
        return node.value.as_ref();
    }

    let input_node = &nodes[pos];

    // 1. Exact match
    if let Some(result) = node
        .exact
        .get(input_node)
        .and_then(|child| search(child, nodes, pos + 1))
    {
        return Some(result);
    }

    // 2. Group match
    if let Key::Char(ch) = input_node.key {
        if let Some(result) = node.groups.iter().find_map(|(n, child)| match n.key {
            Key::Group(group) if n.modifiers == input_node.modifiers && group.matches(ch) => {
                search(child, nodes, pos + 1)
            }
            _ => None,
        }) {
            return Some(result);
        }
    }

    // 3. Any-char group match
    node.groups.iter().find_map(|(n, child)| {
        if matches!(n.key, Key::Group(CharGroup::Any)) {
            search(child, nodes, pos + 1)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use keymap_parser::parse_seq;

    use super::*;

    fn matches(inputs: &[(&'static str, &'static str, bool)]) {
        let items = inputs
            .iter()
            .enumerate()
            .map(|(i, (keys, _, _))| (parse_seq(keys).unwrap(), i))
            .collect::<Vec<_>>();

        let matcher = Matcher::from_iter(items);
        inputs.iter().enumerate().for_each(|(i, (_, v, pass))| {
            let key = parse_seq(v).unwrap();
            let result = matcher.get(&key);

            if *pass {
                assert_eq!(result, Some(i).as_ref(), "{key:?}");
            } else {
                assert_eq!(result, None);
            }
        });
    }

    #[test]
    fn test_exact_nodes() {
        matches(&[
            ("a", "a", true),
            ("ctrl-c", "ctrl-c", true),
            ("f12", "f12", true),
            ("f10", "f11", false),
            ("enter", "enter", true),
        ]);
    }

    #[test]
    fn test_groups() {
        matches(&[
            ("@upper", "A", true),
            ("@digit", "1", true),
            ("ctrl-@any", "ctrl-x", true),
            ("@any", "b", true),
            ("a", "a", true), // Exact match has highest priority
        ]);
    }

    #[test]
    fn test_sequences() {
        matches(&[
            ("a enter", "a enter", true),
            ("ctrl-@any shift-@upper", "ctrl-x shift-B", true),
        ]);
    }
}
