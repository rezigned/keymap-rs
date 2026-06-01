use std::io::Write;

use keymap::Item;

#[cfg(feature = "crossterm")]
#[path = "./crossterm.rs"]
mod crossterm;

#[cfg(feature = "crossterm")]
pub(crate) use crossterm::run;

#[cfg(feature = "crossterm")]
#[allow(unused)]
pub(crate) use ::crossterm::event::KeyEvent as Key;

#[cfg(feature = "termion")]
#[path = "./termion.rs"]
mod termion;

#[cfg(feature = "termion")]
pub(crate) use termion::run;

#[cfg(feature = "termion")]
#[allow(unused)]
pub(crate) use ::termion::event::Key;

#[cfg(not(any(feature = "crossterm", feature = "termion", feature = "wasm")))]
#[path = "./mock.rs"]
mod mock;

#[allow(unused)]
#[cfg(not(any(feature = "crossterm", feature = "termion", feature = "wasm")))]
pub(crate) use mock::{run, Key};

#[allow(dead_code)]
pub(crate) fn print(s: &str) -> bool {
    print!("\r{s}");
    std::io::stdout().flush().ok();
    false
}

#[allow(dead_code)]
pub(crate) fn quit(s: &str) -> bool {
    println!("{s}\r");
    std::io::stdout().flush().ok();
    true
}

// ANSI colors
const RESET: &str = "\x1b[0m";
const COLOR_DIM: &str = "\x1b[38;2;68;72;85m";
const COLOR_KEY: &str = "\x1b[38;2;148;226;213m";
const COLOR_TEXT: &str = "\x1b[38;2;166;173;200m";

#[allow(dead_code)]
pub(crate) fn print_config<T: std::fmt::Debug>(items: &[(T, Item)]) {
    let keys = items
        .iter()
        .map(|(_, v)| {
            format!(
                "{COLOR_KEY}{}{RESET} {COLOR_TEXT}{}{RESET}",
                v.symbol.clone().unwrap_or_default(),
                v.help.clone().unwrap_or(v.description.clone())
            )
        })
        .collect::<Vec<_>>()
        .join(&format!(" {COLOR_DIM}|{RESET} "));

    println!("\r{keys}");
    std::io::stdout().flush().ok();
}

#[allow(unused)]
fn main() {}
