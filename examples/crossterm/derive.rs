#[path = "../config_derive.rs"]
mod config_derive;

#[path = "./utils.rs"]
mod crossterm_utils;

use config_derive::Action;
use crossterm::{
    event::{read, Event},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use crossterm_utils::output;
use keymap::{KeyMap, KeyMapConfig};
use std::io;

fn main() -> io::Result<()> {
    config_derive::print_config(&Action::keymap_config());

    enable_raw_mode()?;

    let mut send = output();

    loop {
        let event = read()?;

        if let Event::Key(key) = event {
            match Action::try_from(KeyMap::try_from(&key).unwrap()) {
                Ok(action) => match action {
                    Action::Quit => break,
                    Action::Up | Action::Down | Action::Left | Action::Right | Action::Jump => {
                        send(format!("{action:?}"))?
                    }
                },
                Err(e) => send(e.to_string())?,
            }
        }
    }

    disable_raw_mode()
}
