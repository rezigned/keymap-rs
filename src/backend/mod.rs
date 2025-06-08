//! # Backends
#[cfg(feature = "crossterm")]
mod crossterm;

#[cfg(feature = "crossterm")]
pub use self::crossterm::parse;

#[cfg(feature = "termion")]
mod termion;

#[cfg(feature = "termion")]
pub use self::termion::parse;
