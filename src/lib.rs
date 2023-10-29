#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]
pub mod backend;
pub mod parser;

pub use backend::{KeyMap, Key, parse, parse_seq};
mod config;

pub use config::Config;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use keymap_derive::*;

pub trait KeyValPair<V> {
    fn keymaps(&self) -> Config<V>;
}
