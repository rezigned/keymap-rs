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
    println!("{s}\r");
    false
}

#[allow(dead_code)]
pub(crate) fn quit(s: &str) -> bool {
    println!("{s}\r");
    true
}

#[allow(dead_code)]
pub(crate) fn print_config<T: std::fmt::Debug>(items: &[(T, Item)]) {
    println!("--- keymap ---");

    items.iter().for_each(|(action, v)| {
        println!(
            "{action:?} = keys: {:?}, description: {}",
            v.keys, v.description
        )
    });

    println!("--------------");
}
#[allow(unused)]
fn main() {}
