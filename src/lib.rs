#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
pub mod backend;
pub use keymap_parser::parser;

pub use backend::{parse, parse_seq, Key, KeyMap};
mod config;

pub use config::{Config, KeyMapConfig};

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use keymap_derive::KeyMap;
