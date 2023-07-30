use std::fmt::{Display, Formatter};
use std::str;
use std::{fmt::Debug, str::FromStr};

use pom::char_class::{alpha, digit};
use pom::parser::{end, is_a, one_of, sym, Parser};
use serde::{de, Deserialize, Deserializer};
use strum_macros::{AsRefStr, Display, EnumString};

/// Key separator
const KEY_SEP: u8 = b'-';

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

type Modifiers = u8;
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
/// This function will return an error if .
pub(crate) fn parse(s: &str) -> Result<Node, pom::Error> {
    node().parse(s.as_bytes())
}

// node      = modifiers* key
// modifiers = modifier "-"
// modifier  = "ctrl" | "cmd" | "alt" | "shift"
// key       = char | "esc" | "del" | "enter" | ...
// char      = "a..z" | "A..Z" | "0".."9" | ...
fn node<'a>() -> Parser<'a, u8, Node> {
    combination() - end()
}

fn key<'a>() -> Parser<'a, u8, Key> {
    fn_key() | parse_enum::<Key>() | char()
}

fn char<'a>() -> Parser<'a, u8, Key> {
    is_a(|c: u8| c.is_ascii()).map(|c| Key::Char(c.into()))
}

/// Parses F0..F12
fn fn_key<'a>() -> Parser<'a, u8, Key> {
    sym(b'f') * ((sym(b'1') * one_of(b"012")).map(|n| 10 + n) | is_a(digit)).map(|n| Key::F(n - 48))
}

fn modifier<'a>() -> Parser<'a, u8, Modifier> {
    parse_enum::<Modifier>() - sym(KEY_SEP).opt()
}

fn combination<'a>() -> Parser<'a, u8, Node> {
    (modifier().repeat(..4) + key()).map(|(m, key)| {
        let mods = m.into_iter().map(|v| v as u8).sum();

        Node::new(mods, key)
    })
}

/// Parses a string into Enum<T>
fn parse_enum<'a, T>() -> Parser<'a, u8, T>
where
    T: FromStr + 'static,
    <T as FromStr>::Err: Debug,
{
    let p = is_a(alpha).repeat(2..);
    p.convert(|s| std::str::from_utf8(&s).unwrap().parse::<T>())
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
                write!(f, "{m}{}", KEY_SEP as char).unwrap();
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
    use pom::parser::end;
    use serde::Deserialize;

    use crate::parser::{parse_enum, Key, Modifier, Node};

    use super::fn_key;

    #[test]
    fn test_parse_fn_key() {
        // Valid number
        (0..=12).for_each(|n| {
            let input = format!("f{n}");
            let result = (fn_key() - end()).parse(input.as_bytes());

            assert_eq!(Key::F(n), result.unwrap());
        });

        // Invalid number
        assert!((fn_key() - end()).parse(b"f14").is_err());
    }

    #[test]
    fn test_parse_enum() {
        [("up", Key::Up), ("esc", Key::Esc), ("del", Key::Delete)].map(|(s, key)| {
            let r = parse_enum::<Key>().parse(s.as_bytes());
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
            (
                Node::new(Modifier::Alt as u8, Key::Char('f')),
                "alt-f",
            ),
            (
                Node::new(
                    Modifier::Shift as u8 | Modifier::Cmd as u8,
                    Key::Char('f'),
                ),
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
            Node::new(
                Modifier::Cmd as u8 | Modifier::Shift as u8,
                Key::Delete,
            ),
            Node::new(0, Key::Delete),
        ]
        .map(|n| {
            let (key, _) = result.keys.get_key_value(&n).unwrap();
            assert_eq!(key, &n);
        });
    }
}
