use std::io;

#[path = "../config.rs"]
mod config;

#[path = "./utils.rs"]
mod crossterm_utils;

use config::{parse_config, Action};
use crossterm::{
    event::{read, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use crossterm_utils::output;
use keymap::ToKeyMap;

fn main() -> io::Result<()> {
    enable_raw_mode()?;

    let mut send = output();
    let config = parse_config();

    loop {
        let event = read()?;

        if let Event::Key(key) = event {
            if let Some((_, action)) = config.0.get_key_value(&key.to_keymap().unwrap()) {
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
    }

    disable_raw_mode()
}
