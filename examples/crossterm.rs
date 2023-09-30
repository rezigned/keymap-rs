use std::io;
mod config;
mod crossterm_utils;

use config::{parse_config, Action};
use crossterm::{
    event::{read, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use crossterm_utils::output;
use keymap::KeyMap;

fn main() -> io::Result<()> {
    enable_raw_mode()?;

    let mut send = output();
    let config = parse_config();

    loop {
        let event = read()?;

        match event {
            Event::Key(key) => {
                if let Some((_, action)) = config.0.get_key_value(&KeyMap::from(key)) {
                    match action {
                        Action::Up => send("Up!")?,
                        Action::Down => send("Down!")?,
                        Action::Jump => send("Jump!")?,
                        Action::Left => send("Left!")?,
                        Action::Right => send("Right!")?,
                        Action::Quit => break,
                    }
                }
            }
            _ => (),
        }
    }

    disable_raw_mode()
}
