#[cfg(feature = "crossterm")]
use crossterm::{
    cursor,
    execute,
    style::Print,
};
use std::io;

#[allow(unused)]
#[cfg(feature = "crossterm")]
pub(crate) fn output() -> impl FnMut(&str) -> io::Result<()> + 'static {
    let mut stdout = io::stdout();
    move |s| execute!(stdout, Print(s), cursor::MoveToNextLine(1))
}

#[allow(unused)]
fn main() {}
