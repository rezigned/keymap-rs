use std::io::{stdin, stdout, Result, Write};

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

#[allow(dead_code)]
pub(crate) fn run<F>(mut f: F) -> Result<()>
where
    F: FnMut(Key) -> bool,
{
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode()?;

    write!(stdout, "{}", termion::cursor::Save)?;
    stdout.flush()?;

    for key in stdin.keys() {
        write!(
            stdout,
            "{}{}",
            termion::cursor::Restore,
            termion::clear::AfterCursor,
        )?;

        let quit = f(key.unwrap());
        stdout.flush()?;

        if quit {
            break;
        }
    }

    write!(stdout, "{}", termion::cursor::Show)
}

#[allow(unused)]
fn main() {}
