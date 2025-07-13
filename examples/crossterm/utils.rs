use std::io;

#[cfg(feature = "crossterm")]
use crossterm::{cursor, execute, style::Print};
use crossterm::{
    event::{read, Event, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};

#[allow(dead_code)]
#[cfg(feature = "crossterm")]
pub(crate) fn output<T: std::fmt::Display>() -> impl FnMut(T) -> std::io::Result<()> {
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
pub(crate) fn run<F>(mut f: F) -> io::Result<()>
where
    F: FnMut(KeyEvent) -> bool,
{
    enable_raw_mode()?;

    loop {
        if let Event::Key(key) = read()? {
            let quit = f(key);
            if quit {
                break;
            }
        }
    }

    disable_raw_mode()
}

#[allow(unused)]
fn main() {}
