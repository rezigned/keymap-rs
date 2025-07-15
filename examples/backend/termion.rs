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

    for key in stdin.keys() {
        let quit = f(key.unwrap());
        if quit {
            break;
        }

        stdout.flush().unwrap();
    }

    write!(stdout, "{}", termion::cursor::Show)
}

#[allow(unused)]
fn main() {}
