#[path = "../config_derive.rs"]
mod config_derive;
#[path = "./utils.rs"]
mod termion_utils;

use config_derive::Action;
use std::io::{stdin, Write};
use termion::event::Event;
use termion::input::TermRead;
use termion_utils::{output, print, Result};

fn main() -> Result {
    let config = config_derive::derived_config();
    config_derive::print_config(&config.items);

    let stdin = stdin();
    let mut stdout = output();

    for event in stdin.events() {
        let mut send = |s: &str| print(&mut stdout, s);

        if let Event::Key(key) = event? {
            match config.get(&key) {
                Some(action) => match action {
                    Action::Up => send("Up!"),
                    Action::Down => send("Down!"),
                    Action::Jump => send("Jump!"),
                    Action::Left => send("Left!"),
                    Action::Right => send("Right!"),
                    Action::Quit => break,
                },
                None => send("Unknown key"),
            };
        }

        stdout.flush().unwrap();
    }

    Ok(())
}
