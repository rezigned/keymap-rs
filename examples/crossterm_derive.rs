mod config_derive;
mod crossterm_utils;

use config_derive::Action;
use crossterm::{
    event::{read, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use crossterm_utils::output;
use keymap::KeyMap;
use std::io;

fn main() -> io::Result<()> {
    enable_raw_mode()?;

    let mut send = output();

    loop {
        let event = read()?;

        if let Event::Key(key) = event {
            match Action::try_from(KeyMap::from(key)) {
                Ok(action) => match action {
                    Action::Up => send("Up!")?,
                    Action::Down => send("Down!")?,
                    Action::Jump => send("Jump!")?,
                    Action::Left => send("Left!")?,
                    Action::Right => send("Right!")?,
                    Action::Quit => break,
                },
                Err(e) => send(&e)?,
            }
        }
    }

    disable_raw_mode()
}
