//! # Parser
//!
//! The `parser` module provides functionality for parsing input events from plain-text.
//!
use std::str;
use std::{fmt::Debug, str::FromStr};

use pom::Error;
use pom::parser::{Parser, end, is_a, one_of, sym};

use crate::node::{KEY_SEP, Key, Modifier, Node};

/// Parses an input string and returns a Node on success.
///
/// # Errors
///
/// This function will return an error if it can't parse the given input.
pub fn parse(s: &str) -> Result<Node, Error> {
    let input = s.chars().collect::<Vec<char>>();
    let result = node().parse(&input);

    result
}

/// Parses a node from the input.
///
/// A node represents a combination of modifiers (ctrl, alt, shift, cmd) and a key.
/// The parser first parses the combination of modifiers, then parses the key.
///
/// Returns a `Node` struct containing the modifiers and the key.
///
/// Grammar:
///
/// node      = modifiers* key
/// modifiers = modifier "-"
/// modifier  = "ctrl" | "cmd" | "alt" | "shift"
/// key       = fn-key | named-key | char
/// fn-key    = "f" digit
/// named-key = "up" | "esc" | "del" | ...
/// char      = "a..z" | "A..Z" | "0".."9" | ...
///
fn node<'a>() -> Parser<'a, char, Node> {
    combination() - end()
}

/// Parses a key e.g. `f1`, `up`, `esc`, `c`.
fn key<'a>() -> Parser<'a, char, Key> {
    fn_key() | named_key::<Key>() | char()
}

/// Parses a function key.
///
/// This function parses a function key, which starts with 'f' followed by a digit (0-12).
///
/// For example: `f1`, `f2`, `f3`.
fn fn_key<'a>() -> Parser<'a, char, Key> {
    sym('f')
        * ((sym('1') * one_of("012")).map(|n| 10 + n as u8) | is_a(digit).map(|n| n as u8))
            .map(|n| Key::F(n - 48))
}

/// Parses a named key.
///
/// This function parses a named key, such as "up", "esc", or "del".
fn named_key<'a, T>() -> Parser<'a, char, T>
where
    T: FromStr + 'static,
    <T as FromStr>::Err: Debug,
{
    is_a(alpha)
        .repeat(2..)
        .convert(|s| s.iter().collect::<String>().parse::<T>())
}

/// Parses a character.
///
/// This function parses a single ASCII character and returns it as a `Key::Char`.
fn char<'a>() -> Parser<'a, char, Key> {
    is_a(ascii).map(Key::Char)
}

/// Parses a modifier.
///
/// This function parses a modifier key, such as "ctrl", "alt", "shift", or "cmd".
fn modifier<'a>() -> Parser<'a, char, Modifier> {
    named_key::<Modifier>() - sym(KEY_SEP).opt()
}

/// Parses a combination of modifiers and a key.
///
/// This function parses a combination of modifiers (e.g., "ctrl-alt-") followed by a key (e.g., "a").
fn combination<'a>() -> Parser<'a, char, Node> {
    (modifier().repeat(..4) + key()).map(|(m, key)| {
        let mods = m.into_iter().fold(0, |l, r| l | r as u8);

        Node::new(mods, key)
    })
}

/// Checks if a character is an ASCII alphabetic character.
#[inline]
fn alpha(term: char) -> bool {
    term.is_ascii_alphabetic()
}

/// Checks if a character is an ASCII character.
#[inline]
fn ascii(term: char) -> bool {
    term.is_ascii()
}

/// Checks if a character is an ASCII digit.
#[inline]
fn digit(term: char) -> bool {
    term.is_ascii_digit()
}

/// Parses a whitespace separated sequence of keys.
///
/// This splits the given string on whitespace and parses each component with
/// `parse`. The results are collected into a `Vec<KeyMap>`.
///
/// # Examples
///
/// ```
/// use keymap_parser::{parse_seq, Node, Key};
///
/// let seq = parse_seq("a b").unwrap();
///
/// assert_eq!(seq, vec![
///     Node::from(Key::Char('a')),
///     Node::from(Key::Char('b')),
/// ]);
/// ```
///
/// # Errors
///
/// This function will return an error if any of the components fail to parse.
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
        ("ctrl-b -l", Err(parse("-l").unwrap_err())),
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
        // Valid number
        (0..=12).for_each(|n| {
            let input = format!("f{n}").chars().collect::<Vec<char>>();
            let result = (fn_key() - end()).parse(&input);

            assert_eq!(Key::F(n), result.unwrap());
        });

        // Invalid number
        [13, 15].map(|n| {
            let input: Vec<char> = format!("f{n}").chars().collect();
            let result = (fn_key() - end()).parse(&input);

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_parse_enum() {
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
