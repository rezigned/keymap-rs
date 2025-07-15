use std::io;

use crossterm::{
    event::{read, Event, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};

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
