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
use std::io;

fn main() -> io::Result<()> {
    let config = config_derive::derived_config();
    config_derive::print_config(&config.items);

    enable_raw_mode()?;

    let mut send = output();

    loop {
        let event = read()?;

        if let Event::Key(key) = event {
            // Or using config.get(key) if we don't need the item
            match config.get_item(key) {
                Some((action, item)) => match action {
                    Action::Quit => break,
                    Action::Up | Action::Down | Action::Left | Action::Right | Action::Jump => {
                        send(format!("{action:?} -> {}", item.description))?
                    }
                },
                None => send(format!("Unknown key [{:?}]", key.code))?,
            }
        }
    }

    disable_raw_mode()
}
