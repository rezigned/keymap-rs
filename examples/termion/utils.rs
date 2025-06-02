#[cfg(feature = "termion")]
use termion::input::MouseTerminal;
#[cfg(feature = "termion")]
use termion::raw::{IntoRawMode, RawTerminal};
#[cfg(feature = "termion")]
use std::io::{self, Write, stdout, Stdout};

#[cfg(feature = "termion")]
#[allow(unused)]
pub(crate) type Result = io::Result<()>;

#[cfg(feature = "termion")]
#[allow(unused)]
pub(crate) fn print<'a>(stdout: &'a mut MouseTerminal<RawTerminal<Stdout>>, s: &'a str) {
    write!(stdout, "{}{}KEY: {}", termion::clear::All, termion::cursor::Goto(1, 1), s).unwrap();
}

#[cfg(feature = "termion")]
#[allow(unused)]
pub(crate) fn output() -> MouseTerminal<RawTerminal<Stdout>> {
    MouseTerminal::from(stdout().into_raw_mode().unwrap())
}

#[allow(unused)]
fn main() {}
