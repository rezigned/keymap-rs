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
//! char      = ascii-char | group
//! group     = "@" ("digit" | "lower" | "upper" | "alnum" | "alpha" | "char")
//! ```
use crate::node::{CharGroup, KEY_SEP, Key, Modifier, Node};

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
    consumed: (usize, &'a str),
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            consumed: (0, ""),
        }
    }

    /// Get the current character without consuming it
    fn peek(&self) -> Option<char> {
        self.input.chars().next()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.input.chars().nth(n)
    }

    /// Get the current character and advance position
    fn next(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.position += ch.len_utf8();
            self.input = &self.input[ch.len_utf8()..];

            Some(ch)
        } else {
            None
        }
    }

    /// Check if we're at the end of input
    fn is_end(&self) -> bool {
        self.input.is_empty()
    }

    /// Consume a specific character, returning error if not found
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

    fn start(&mut self) {
        self.consumed = (self.position, self.input);
    }

    fn restore(&mut self) {
        if self.position != self.consumed.0 {
            self.position = self.consumed.0;
            self.input = self.consumed.1;
            self.consumed = (0, "");
        }
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

    if !parser.is_end() {
        return Err(parser.error(format!(
            "expect end of input, found: {}",
            parser.peek().unwrap()
        )));
    }

    Ok(node)
}

/// Parses a combination of modifiers followed by a key
///
/// node      = modifiers* key
/// modifiers = modifier "-"
/// modifier  = "ctrl" | "cmd" | "alt" | "shift"
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
    parser.start();

    // Try to parse a named modifier
    let name = parser.consume_while(|ch| ch.is_ascii_alphabetic());
    if name.is_empty() {
        return Ok(None);
    }

    let modifier = match name.parse::<Modifier>() {
        Ok(m) => m,
        Err(_) => {
            parser.restore();

            return Ok(None);
        }
    };

    // Parse separator
    parser.consume(KEY_SEP)?;

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
    parser.start();

    if !parser.try_consume('f') {
        return Ok(None);
    }

    // Parse the number
    let num = parser.consume_while(|ch| ch.is_ascii_digit());
    if num.is_empty() {
        parser.restore();

        return Ok(None);
    }

    match num.parse::<u8>() {
        Ok(n) if n <= 12 => Ok(Some(Key::F(n))),
        _ => {
            parser.restore();

            Err(parser.error("invalid function key number (must be 0-12)".to_string()))
        }
    }
}

/// Try to parse a named key
fn try_parse_named_key(parser: &mut Parser) -> Result<Option<Key>, ParseError> {
    parser.start();

    let name = parser.consume_while(|ch| ch.is_ascii_alphabetic());
    if name.len() < 2 {
        parser.restore();

        return Ok(None);
    }

    match name.parse::<Key>() {
        Ok(key) => Ok(Some(key)),
        Err(_) => {
            parser.restore();

            Ok(None)
        }
    }
}

/// Try to parse a single character key or character group
fn try_parse_char(parser: &mut Parser) -> Result<Option<Key>, ParseError> {
    // Check for character group syntax: @group
    if parser.peek() == Some('@') && parser.peek_at(1).is_some() {
        return try_parse_group(parser);
    }

    // Parse regular ASCII character
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

/// Try to parse a character group (@digit, @lower, etc.)
fn try_parse_group(parser: &mut Parser) -> Result<Option<Key>, ParseError> {
    parser.start();

    // Consume the '@' symbol
    parser.consume('@')?;

    // Parse the group name
    let group_name = parser.consume_while(|ch| ch.is_ascii_alphabetic());
    if group_name.is_empty() {
        parser.restore();

        return Err(parser.error("expected group name after '@'".to_string()));
    }

    let group = match group_name.as_str() {
        "digit" => Key::Group(CharGroup::Digit),
        "lower" => Key::Group(CharGroup::Lower),
        "upper" => Key::Group(CharGroup::Upper),
        "alnum" => Key::Group(CharGroup::Alnum),
        "alpha" => Key::Group(CharGroup::Alpha),
        "char" => Key::Group(CharGroup::Char),
        _ => {
            parser.restore();

            return Err(parser.error(format!("unknown character group: '{group_name}'", )));
        }
    };

    Ok(Some(group))
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

    use crate::parser::{CharGroup, Key, Modifier, Node};

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
    fn test_parse_char_groups() {
        [
            ("@digit", Key::Group(CharGroup::Digit)),
            ("@lower", Key::Group(CharGroup::Lower)),
            ("@upper", Key::Group(CharGroup::Upper)),
            ("@alnum", Key::Group(CharGroup::Alnum)),
            ("@alpha", Key::Group(CharGroup::Alpha)),
            ("@char", Key::Group(CharGroup::Char)),
        ]
        .map(|(input, expected_key)| {
            let result = parse(input);
            assert_eq!(result.unwrap().key, expected_key);
        });

        // Test invalid group names
        let result = parse("@invalid");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("unknown character group")
        );

        // Test incomplete group syntax
        let result = parse("@x");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .message
                .contains("unknown character group: 'x'")
        );
    }

    #[test]
    fn test_format() {
        [
            (Node::new(0, Key::F(3)), "f3"),
            (Node::new(0, Key::Delete), "delete"),
            (Node::new(0, Key::Space), "space"),
            (Node::new(0, Key::Char('g')), "g"),
            (Node::new(0, Key::Char('#')), "#"),
            (Node::new(0, Key::Group(CharGroup::Digit)), "@digit"),
            (Node::new(0, Key::Group(CharGroup::Lower)), "@lower"),
            (Node::new(Modifier::Alt as u8, Key::Char('f')), "alt-f"),
            (
                Node::new(Modifier::Alt as u8, Key::Group(CharGroup::Alpha)),
                "alt-@alpha",
            ),
            (
                Node::new(Modifier::Shift as u8 | Modifier::Cmd as u8, Key::Char('f')),
                "cmd-shift-f",
            ),
        ]
        .map(|(node, expected)| {
            assert_eq!(expected, format!("{node}"));
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
"@digit" = "number"
"alt-@lower" = "alt-lowercase"
    "#,
        )
        .unwrap();

        [
            Node::new(Modifier::Alt as u8, Key::Char('d')),
            Node::new(Modifier::Cmd as u8 | Modifier::Shift as u8, Key::Delete),
            Node::new(0, Key::Delete),
            Node::new(0, Key::Group(CharGroup::Digit)),
            Node::new(Modifier::Alt as u8, Key::Group(CharGroup::Lower)),
        ]
        .map(|n| {
            let (key, _) = result.keys.get_key_value(&n).unwrap();
            assert_eq!(key, &n);
        });
    }
}
