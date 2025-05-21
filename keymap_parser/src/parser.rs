//! # Parser
//!
//! The `parser` module provides functionality for parsing input events from plain-text keymap definitions.
//! It supports sequences like "ctrl-alt-f1" or "a b", mapping them to structured key/modifier representations.
//!

use std::str;
use std::{fmt::Debug, str::FromStr};

use pom::Error;
use pom::parser::{Parser, end, is_a, one_of, sym};

use crate::node::{KEY_SEP, Key, Modifier, Node};

/// Parses an input string representing a key or key combination into a [`Node`] on success.
///
/// # Errors
///
/// Returns an [`Error`] if the input cannot be parsed as a valid key or key combination.
pub fn parse(s: &str) -> Result<Node, Error> {
    let input = s.chars().collect::<Vec<char>>();
    let result = node().parse(&input);

    result
}

/// Top-level parser for a key combination node.
///
/// A node represents a combination of zero or more modifiers (e.g., ctrl, alt)
/// and a key (e.g., 'a', 'esc', 'f1').
///
/**
Grammar:
    node      = modifiers* key
    modifiers = modifier "-"
    modifier  = "ctrl" | "cmd" | "alt" | "shift"
    key       = fn-key | named-key | char
    fn-key    = "f" digit
    named-key = "up" | "esc" | "del" | ...
    char      = "a..z" | "A..Z" | "0".."9" | ...
*/
fn node<'a>() -> Parser<'a, char, Node> {
    combination() - end()
}

/// Parses a key (function key, named key, or character).
fn key<'a>() -> Parser<'a, char, Key> {
    fn_key() | named_key::<Key>() | char()
}

/// Parses a function key (e.g., "f1", "f12").
///
/// Accepts "f0" through "f12".
fn fn_key<'a>() -> Parser<'a, char, Key> {
    sym('f')
        * ((sym('1') * one_of("012")).map(|n| 10 + n as u8) // "f10", "f11", or "f12"
          | is_a(digit).map(|n| n as u8))                   // "f0" through "f9"
            .map(|n| Key::F(n - 48)) // Adjust ASCII to digit value as needed
}

/// Parses a named key such as "up", "esc", or "del".
fn named_key<'a, T>() -> Parser<'a, char, T>
where
    T: FromStr + 'static,
    <T as FromStr>::Err: Debug,
{
    is_a(alpha)
        .repeat(2..) // At least 2 alphabetic characters
        .convert(|s| s.iter().collect::<String>().parse::<T>())
}

/// Parses a single ASCII character key (e.g., 'a', 'Z', '7').
fn char<'a>() -> Parser<'a, char, Key> {
    is_a(ascii).map(Key::Char)
}

/// Parses a modifier key (e.g., "ctrl", "alt", "shift", "cmd").
///
/// Optionally allows for a trailing separator (e.g., "-").
fn modifier<'a>() -> Parser<'a, char, Modifier> {
    named_key::<Modifier>() - sym(KEY_SEP).opt()
}

/// Parses a combination of modifiers (up to 4) followed by a key (e.g., "ctrl-alt-a").
///
/// Returns a [`Node`] encoding the modifier bitmask and the key.
fn combination<'a>() -> Parser<'a, char, Node> {
    (modifier().repeat(..4) + key()).map(|(m, key)| {
        // Combine modifiers using bitwise OR into a single u8
        let mods = m.into_iter().fold(0, |l, r| l | r as u8);

        Node::new(mods, key)
    })
}

/// Returns true if the character is an ASCII alphabetic letter.
#[inline]
fn alpha(term: char) -> bool {
    term.is_ascii_alphabetic()
}

/// Returns true if the character is any ASCII character.
#[inline]
fn ascii(term: char) -> bool {
    term.is_ascii()
}

/// Returns true if the character is an ASCII digit ('0' - '9').
#[inline]
fn digit(term: char) -> bool {
    term.is_ascii_digit()
}

/// Parses a whitespace-separated sequence of key expressions.
///
/// Splits the input string on whitespace and parses each part as a [`Node`].
///
/// # Examples
///
/// ```
/// use keymap_parser::{parse_seq, Node, Key};
///
/// let seq = parse_seq("a b").unwrap();
/// assert_eq!(
///     seq,
///     vec![Node::from(Key::Char('a')), Node::from(Key::Char('b'))]
/// );
/// ```
///
/// # Errors
///
/// Returns an error if any portion of the sequence fails to parse.
pub fn parse_seq(s: &str) -> Result<Vec<Node>, pom::Error> {
    str::split_whitespace(s).map(parse).collect()
}

#[test]
fn test_parse_seq() {
    [
        ("ctrl-b", Ok(vec![parse("ctrl-b").unwrap()])),
        (
            "ctrl-b l",
            Ok(vec![parse("ctrl-b").unwrap(), parse("l").unwrap()]),
        ),
        ("ctrl-b -l", Err(parse("-l").unwrap_err())), // Invalid: dangling separator
    ]
    .map(|(s, v)| assert_eq!(parse_seq(s), v));
}

#[cfg(test)]
mod tests {
    use pom::{Error, parser::end};
    use serde::Deserialize;

    use crate::parser::{Key, Modifier, Node, named_key};

    use super::{fn_key, parse};

    #[test]
    fn test_parse() {
        let err = |e| Err::<Node, Error>(e);

        [
            ("alt-f", Ok(Node::new(Modifier::Alt as u8, Key::Char('f')))),
            ("space", Ok(Node::new(0, Key::Space))),
            (
                "delta",
                err(Error::Mismatch {
                    message: "expect end of input, found: e".into(),
                    position: 1,
                }),
            ),
            (
                "shift-a",
                Ok(Node::new(Modifier::Shift as u8, Key::Char('a'))),
            ),
            (
                "shift-a-delete",
                err(Error::Mismatch {
                    message: "expect end of input, found: -".into(),
                    position: 7,
                }),
            ),
            (
                "al",
                err(Error::Mismatch {
                    message: "expect end of input, found: l".into(),
                    position: 1,
                }),
            ),
        ]
        .map(|(input, result)| {
            let output = parse(input);
            assert_eq!(result, output);
        });
    }

    #[test]
    fn test_parse_fn_key() {
        // Valid function key numbers: f0 - f12
        (0..=12).for_each(|n| {
            let input = format!("f{n}").chars().collect::<Vec<char>>();
            let result = (fn_key() - end()).parse(&input);

            assert_eq!(Key::F(n), result.unwrap());
        });

        // Invalid: above f12
        [13, 15].map(|n| {
            let input: Vec<char> = format!("f{n}").chars().collect();
            let result = (fn_key() - end()).parse(&input);

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_parse_enum() {
        // Check named keys
        [("up", Key::Up), ("esc", Key::Esc), ("del", Key::Delete)].map(|(s, key)| {
            let input: Vec<char> = s.chars().collect();
            let r = named_key::<Key>().parse(&input);
            assert_eq!(r.unwrap(), key);
        });
    }

    #[test]
    fn test_format() {
        [
            (Node::new(0, Key::F(3)), "f3"),
            (Node::new(0, Key::Delete), "delete"),
            (Node::new(0, Key::Space), "space"),
            (Node::new(0, Key::Char('g')), "g"),
            (Node::new(0, Key::Char('#')), "#"),
            (Node::new(Modifier::Alt as u8, Key::Char('f')), "alt-f"),
            (
                Node::new(Modifier::Shift as u8 | Modifier::Cmd as u8, Key::Char('f')),
                "cmd-shift-f",
            ),
        ]
        .map(|(node, expected)| {
            let result = format!("{}", node);
            assert_eq!(expected, result);
        });
    }

    #[test]
    fn test_deserialize() {
        use std::collections::HashMap;

        #[derive(Deserialize, Debug)]
        struct Test {
            keys: HashMap<Node, String>,
        }

        let result: Test = toml::from_str(
            r#"
[keys]
alt-d = "a"
cmd-shift-del = "b"
shift-cmd-del = "b" # this is the same as previous one
delete = "d"
    "#,
        )
        .unwrap();

        [
            Node::new(Modifier::Alt as u8, Key::Char('d')),
            Node::new(Modifier::Cmd as u8 | Modifier::Shift as u8, Key::Delete),
            Node::new(0, Key::Delete),
        ]
        .map(|n| {
            let (key, _) = result.keys.get_key_value(&n).unwrap();
            assert_eq!(key, &n);
        });
    }
}
