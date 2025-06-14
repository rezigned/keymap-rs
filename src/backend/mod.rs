//! # Backends
//!
//! This module provides different backends for key event parsing.
//! Available backends:
//! - `crossterm`: For crossterm-based terminal applications.
//! - `termion`: For termion-based terminal applications.
//! - `wasm`: For WebAssembly applications (experimental).
//!
//! Enable them using feature flags in your `Cargo.toml`.
#[cfg(feature = "crossterm")]
mod crossterm;

#[cfg(feature = "termion")]
mod termion;

#[cfg(feature = "wasm")]
mod wasm;

// Priority for re-exporting `parse`
#[cfg(feature = "wasm")]
pub use self::wasm::parse;

#[cfg(all(feature = "termion", not(feature = "wasm")))]
pub use self::termion::parse;

#[cfg(all(feature = "crossterm", not(feature = "wasm"), not(feature = "termion")))]
pub use self::crossterm::parse;
