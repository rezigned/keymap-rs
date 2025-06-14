//! # `KeyMap` Library
//!
//! This library provides a unified abstraction over key input representations from various backends,
//! such as [`crossterm`](https://crates.io/crates/crossterm), WebAssembly environments, and more.
//! It enables parsing, transforming, and serializing key input events to and from a common `KeyMap`
//! format defined by a node tree from `keymap_parser`.
//!
//! The goal is to decouple application logic from backend-specific input handling, enabling easier
//! testing, configuration, and cross-platform compatibility.

use keymap_parser::{parser::ParseError, Node};

/// A type alias representing a parsed keymap structure.
///
/// Internally, this uses the [`Node`] type from the `keymap_parser` crate.
pub type KeyMap = Node;

/// A trait for converting a [`KeyMap`] into a backend-specific key type.
///
/// This trait is typically implemented by types like [`crossterm::event::KeyEvent`],
/// or other platform-specific representations of key events. It enables transforming
/// a backend-agnostic keymap (usually parsed from user configuration) into a format
/// suitable for use with a particular input backend.
///
/// # Errors
///
/// Returns [`Error::Parse`] if the `KeyMap` is malformed, or [`Error::UnsupportedKey`]
/// if it contains keys or structures not supported by the target backend.
pub trait FromKeyMap: Sized {
    fn from_keymap(keymap: KeyMap) -> Result<Self, Error>;
}

/// A trait for converting a backend-specific key event into a [`KeyMap`].
///
/// This trait should be implemented by input event types from backends like
/// [`crossterm`], browser environments (`wasm`), or others. It allows converting
/// native input events into the unified [`KeyMap`] format used across the application,
/// making it easier to handle key events consistently regardless of platform.
///
/// # Errors
///
/// Returns an [`Error`] if conversion fails due to unsupported or invalid keys.
pub trait IntoKeyMap {
    fn into_keymap(self) -> Result<KeyMap, Error>;
}

/// A trait for converting a backend-specific key type into a [`KeyMap`].
///
/// This trait should be implemented by types such as `crossterm::event::KeyEvent`,
/// or other platform-specific input event types, in order to convert them into the
/// abstract, backend-agnostic [`KeyMap`] format. This is useful for purposes like
/// configuration export, logging, or bridging platform-specific events with
/// application-agnostic keymap logic.
///
/// # Errors
///
/// Returns an [`Error`] if the conversion fails, such as when the backend event
/// contains an unsupported or unrepresentable key.
pub trait ToKeyMap {
    /// Converts the type into a [`KeyMap`].
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if conversion fails, such as unsupported or invalid keys.
    fn to_keymap(&self) -> Result<KeyMap, Error>;
}

/// Errors that can occur during keymap conversion or parsing.
#[derive(Debug)]
pub enum Error {
    /// A parsing error occurred while processing a `KeyMap`.
    Parse(ParseError),

    /// The key or format is not supported by the current backend or implementation.
    UnsupportedKey(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "{e}"),
            Error::UnsupportedKey(k) => write!(f, "{k}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(e) => Some(e),
            Error::UnsupportedKey(_) => None,
        }
    }
}
