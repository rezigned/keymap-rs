#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

// Re-exports
pub use config::{BackendConfig, Config, DerivedConfig, Item, KeyMapConfig};
pub use keymap::{Error, FromKeyMap, IntoKeyMap, KeyMap};
pub use keymap_parser::parser;
pub use matcher::Matcher;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use keymap_derive::KeyMap;

pub mod backend;
pub mod config;
pub mod keymap;
pub mod matcher;
