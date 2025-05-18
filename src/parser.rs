//! # Parser
//!
//! The `parser` module provides functionality for parsing terminal input events from plain-text.
//!
use std::fmt::{Display, Formatter};
use std::str;
use std::{fmt::Debug, str::FromStr};

use pom::Error;
use pom::parser::{end, is_a, one_of, sym, Parser};
use serde::{de, Deserialize, Deserializer};
use strum_macros::{AsRefStr, Display, EnumString};

/// Key separator
const KEY_SEP: char = '-';

#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) struct Node {
    pub modifiers: Modifiers,
    pub key: Key,
}

impl Node {
    fn new(modifiers: Modifiers, key: Key) -> Self {
        Self { modifiers, key }
    }
}

#[derive(Copy, Clone, Debug, Display, Eq, Hash, PartialEq, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum Modifier {
    None = 0b0000,
    Alt = 0b0001,
    Cmd = 0b0010,
    Ctrl = 0b0100,
    Shift = 0b1000,
}

pub(crate) type Modifiers = u8;

const MODIFIERS: [Modifier; 4] = [
    Modifier::Alt,
    Modifier::Cmd,
    Modifier::Ctrl,
    Modifier::Shift,
];

#[derive(Debug, Display, Eq, Hash, PartialEq, EnumString, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum Key {
    BackTab,
    Backspace,
    Char(char),
    #[strum(serialize = "del", serialize = "delete")]
    Delete,
    Down,
    End,
    Enter,
    Esc,
    Home,
    F(u8),
    Insert,
    Left,
    PageDown,
    PageUp,
    Right,
    Space,
    Tab,
    Up,
}

/// Parses an input string and returns a Node on success.
///
/// # Errors
///
/// This function will return an error if it can't parse the given input.
pub(crate) fn parse(s: &str) -> Result<Node, Error> {
    let input = s.chars().collect::<Vec<char>>();
    let result = node().parse(&input);

    result
}

// node      = modifiers* key
// modifiers = modifier "-"
// modifier  = "ctrl" | "cmd" | "alt" | "shift"
// key       = char | "esc" | "del" | "enter" | ...
// char      = "a..z" | "A..Z" | "0".."9" | ...
fn node<'a>() -> Parser<'a, char, Node> {
    combination() - end()
}

fn key<'a>() -> Parser<'a, char, Key> {
    fn_key() | parse_enum::<Key>() | char()
}

fn char<'a>() -> Parser<'a, char, Key> {
    is_a(ascii).map(Key::Char)
}

/// Parses F0..F12
fn fn_key<'a>() -> Parser<'a, char, Key> {
    sym('f')
        * ((sym('1') * one_of("012")).map(|n| 10 + n as u8) | is_a(digit).map(|n| n as u8))
            .map(|n| Key::F(n - 48))
}

fn modifier<'a>() -> Parser<'a, char, Modifier> {
    parse_enum::<Modifier>() - sym(KEY_SEP).opt()
}

fn combination<'a>() -> Parser<'a, char, Node> {
    (modifier().repeat(..4) + key()).map(|(m, key)| {
        let mods = m.into_iter().map(|v| v as u8).sum();

        Node::new(mods, key)
    })
}

/// Parses a string into Enum<T>
fn parse_enum<'a, T>() -> Parser<'a, char, T>
where
    T: FromStr + 'static,
    <T as FromStr>::Err: Debug,
{
    let p = is_a(alpha).repeat(2..);
    p.convert(|s| s.iter().collect::<String>().parse::<T>())
}

#[inline]
fn alpha(term: char) -> bool {
    term.is_ascii_alphabetic()
}

#[inline]
fn ascii(term: char) -> bool {
    term.is_ascii()
}

#[inline]
fn digit(term: char) -> bool {
    term.is_ascii_digit()
}

/// Deserializes into Node
impl<'s> Deserialize<'s> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'s>,
    {
        let s = String::deserialize(deserializer)?;
        parse(&s).map_err(de::Error::custom)
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        MODIFIERS.iter().for_each(|m| {
            if self.modifiers & *m as u8 != 0 {
                write!(f, "{m}{KEY_SEP}").unwrap();
            }
        });

        match self.key {
            Key::Char(char) => write!(f, "{char}"),
            Key::F(n) => write!(f, "{}{n}", self.key),
            _ => write!(f, "{}", self.key),
        }
    }
}

#[cfg(test)]
mod tests {
    use pom::{parser::end, Error};
    use serde::Deserialize;

    use crate::parser::{parse_enum, Key, Modifier, Node};

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
            let r = parse_enum::<Key>().parse(&input);
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

    #[test]
    fn test_node() {
        let n = Node::new(0, Key::Up);
        match n {
            Node { modifiers: 0, key: Key::Up } => println!("1"),
            Node { modifiers: 1, key: Key::Up } => println!("1"),
            _ => (),
        }
    }
}
