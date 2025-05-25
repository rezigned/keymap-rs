//! # Parser
//!
//! The `parser` module provides functionality for parsing input events from plain-text keymap definitions.
//! It supports sequences like "ctrl-alt-f1" or "a b", mapping them to structured key/modifier representations.
//!
//! ## Grammar
//!
//! ```text
//! node      = modifiers* key
//! modifiers = modifier "-"
//! modifier  = "ctrl" | "cmd" | "alt" | "shift"
//! key       = fn-key | named-key | char
//! fn-key    = "f" digit+
//! named-key = "del" | "insert" | "end" | ...
//! char      = ascii-char
//! ```
use crate::node::{KEY_SEP, Key, Modifier, Node};
use std::str;

/// Custom error type for parsing failures
#[derive(Debug, PartialEq, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Parser state for recursive descent parsing
struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    /// Get the current character without consuming it
    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    /// Get the current character and advance position
    fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.position += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }

    /// Check if we're at the end of input
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    /// Consume a specific character, returning error if not found
    #[allow(dead_code)]
    fn consume(&mut self, expected: char) -> Result<(), ParseError> {
        match self.next() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(ParseError {
                message: format!("expected '{}', found '{}'", expected, ch),
                position: self.position - ch.len_utf8(),
            }),
            None => Err(ParseError {
                message: format!("expected '{}', found end of input", expected),
                position: self.position,
            }),
        }
    }

    /// Try to consume a specific character, returning true if successful
    fn try_consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.next();
            true
        } else {
            false
        }
    }

    /// Consume characters while predicate is true
    fn consume_while<F>(&mut self, predicate: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if predicate(ch) {
                result.push(ch);
                self.next();
            } else {
                break;
            }
        }
        result
    }

    /// Create an error at the current position
    fn error(&self, message: String) -> ParseError {
        ParseError {
            message,
            position: self.position,
        }
    }
}

/// Parses an input string representing a key or key combination into a [`Node`] on success.
///
/// # Errors
///
/// Returns a [`ParseError`] if the input cannot be parsed as a valid key or key combination.
pub fn parse(s: &str) -> Result<Node, ParseError> {
    let mut parser = Parser::new(s);
    let node = parse_combination(&mut parser)?;

    if !parser.is_at_end() {
        return Err(parser.error(format!(
            "expect end of input, found: {}",
            parser.peek().unwrap()
        )));
    }

    Ok(node)
}

/// Parse a combination of modifiers followed by a key
fn parse_combination(parser: &mut Parser) -> Result<Node, ParseError> {
    let mut modifiers = 0u8;

    // Parse up to 4 modifiers
    for _ in 0..4 {
        if let Some(modifier) = try_parse_modifier(parser)? {
            modifiers |= modifier as u8;
        } else {
            break;
        }
    }

    let key = parse_key(parser)?;
    Ok(Node::new(modifiers, key))
}

/// Try to parse a modifier, returning None if no modifier is found
fn try_parse_modifier(parser: &mut Parser) -> Result<Option<Modifier>, ParseError> {
    let start_pos = parser.position;

    // Try to parse a named modifier
    let name = parser.consume_while(|ch| ch.is_ascii_alphabetic());

    if name.is_empty() {
        return Ok(None);
    }

    let modifier = match name.parse::<Modifier>() {
        Ok(m) => m,
        Err(_) => {
            // Not a modifier, reset position
            parser.position = start_pos;
            return Ok(None);
        }
    };

    // Check for optional separator
    parser.try_consume(KEY_SEP);

    Ok(Some(modifier))
}

/// Parse a key (function key, named key, or character)
fn parse_key(parser: &mut Parser) -> Result<Key, ParseError> {
    if let Some(fn_key) = try_parse_fn_key(parser)? {
        Ok(fn_key)
    } else if let Some(named_key) = try_parse_named_key(parser)? {
        Ok(named_key)
    } else if let Some(char_key) = try_parse_char(parser)? {
        Ok(char_key)
    } else {
        Err(parser.error("expected a valid key".to_string()))
    }
}

/// Try to parse a function key (f0-f12)
fn try_parse_fn_key(parser: &mut Parser) -> Result<Option<Key>, ParseError> {
    let start_pos = parser.position;

    if !parser.try_consume('f') {
        return Ok(None);
    }

    // Parse the number
    let num_str = parser.consume_while(|ch| ch.is_ascii_digit());

    if num_str.is_empty() {
        parser.position = start_pos;
        return Ok(None);
    }

    match num_str.parse::<u8>() {
        Ok(n) if n <= 12 => Ok(Some(Key::F(n))),
        _ => {
            parser.position = start_pos;
            Err(parser.error("invalid function key number (must be 0-12)".to_string()))
        }
    }
}

/// Try to parse a named key
fn try_parse_named_key(parser: &mut Parser) -> Result<Option<Key>, ParseError> {
    let start_pos = parser.position;

    let name = parser.consume_while(|ch| ch.is_ascii_alphabetic());

    if name.len() < 2 {
        parser.position = start_pos;
        return Ok(None);
    }

    match name.parse::<Key>() {
        Ok(key) => Ok(Some(key)),
        Err(_) => {
            parser.position = start_pos;
            Ok(None)
        }
    }
}

/// Try to parse a single character key
fn try_parse_char(parser: &mut Parser) -> Result<Option<Key>, ParseError> {
    if let Some(ch) = parser.peek() {
        if ch.is_ascii() {
            parser.next();
            Ok(Some(Key::Char(ch)))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
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
pub fn parse_seq(s: &str) -> Result<Vec<Node>, ParseError> {
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
    use serde::Deserialize;

    use crate::parser::{Key, Modifier, Node};

    use super::{ParseError, parse};

    #[test]
    fn test_parse() {
        let err = |message: &str, position: usize| {
            Err::<Node, ParseError>(ParseError {
                message: message.to_string(),
                position,
            })
        };

        [
            ("alt-f", Ok(Node::new(Modifier::Alt as u8, Key::Char('f')))),
            ("space", Ok(Node::new(0, Key::Space))),
            ("delta", err("expect end of input, found: e", 1)),
            (
                "shift-a",
                Ok(Node::new(Modifier::Shift as u8, Key::Char('a'))),
            ),
            ("shift-a-delete", err("expect end of input, found: -", 7)),
            ("al", err("expect end of input, found: l", 1)),
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
            let input = format!("f{n}");
            let result = parse(&input);
            assert_eq!(Key::F(n), result.unwrap().key);
        });

        // Invalid: above f12
        [13, 15].map(|n| {
            let input = format!("f{n}");
            let result = parse(&input);
            assert!(result.is_err());
        });
    }

    #[test]
    fn test_parse_enum() {
        // Check named keys
        [("up", Key::Up), ("esc", Key::Esc), ("del", Key::Delete)].map(|(s, key)| {
            let result = parse(s);
            assert_eq!(result.unwrap().key, key);
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
