#[cfg(feature = "crossterm")]
use crossterm::{cursor, execute, style::Print};

#[allow(unused)]
#[cfg(feature = "crossterm")]
pub(crate) fn output<T: std::fmt::Display>() -> impl FnMut(T) -> std::io::Result<()> {
    use std::fmt::Display;

    use crossterm::terminal::{Clear, ClearType};

    let mut stdout = std::io::stdout();
    move |s| {
        execute!(
            stdout,
            cursor::MoveToNextLine(0),
            Clear(ClearType::CurrentLine),
            Print(s),
        )
    }
}

#[allow(unused)]
fn main() {}
