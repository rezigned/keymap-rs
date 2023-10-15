#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

pub mod backend;
pub mod parser;

pub use backend::{KeyMap, Key, parse};
mod config;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use keymap_derive::{self, *};
