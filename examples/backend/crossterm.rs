use std::io;

use crossterm::{
    cursor,
    event::{read, Event, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{stdout, Write};

#[allow(dead_code)]
pub(crate) fn run<F>(mut f: F) -> io::Result<()>
where
    F: FnMut(KeyEvent) -> bool,
{
    enable_raw_mode()?;
    stdout().flush()?;

    let (_, row) = cursor::position()?;

    loop {
        if let Event::Key(key) = read()? {
            execute!(
                stdout(),
                cursor::MoveTo(0, row),
                Clear(ClearType::FromCursorDown),
            )?;

            let quit = f(key);
            stdout().flush()?;

            if quit {
                break;
            }
        }
    }

    disable_raw_mode()
}

#[allow(unused)]
fn main() {}
