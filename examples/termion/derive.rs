#[path = "../config_derive.rs"]
mod config_derive;
#[path = "./utils.rs"]
mod termion_utils;

use config_derive::Action;
use keymap::{KeyMap, KeyMapConfig};
use std::io::{stdin, Write};
use termion::event::Event;
use termion::input::TermRead;
use termion_utils::{output, print, Result};

fn main() -> Result {
    config_derive::print_config(&Action::keymap_config());

    let stdin = stdin();
    let mut stdout = output();

    for event in stdin.events() {
        let mut send = |s: &str| print(&mut stdout, s);

        if let Event::Key(key) = event? {
            match Action::try_from(KeyMap::try_from(key).unwrap()) {
                Ok(action) => match action {
                    Action::Up => send("Up!"),
                    Action::Down => send("Down!"),
                    Action::Jump => send("Jump!"),
                    Action::Left => send("Left!"),
                    Action::Right => send("Right!"),
                    Action::Quit => break,
                },
                Err(ref e) => send(e),
            };
        }

        stdout.flush().unwrap();
    }

    Ok(())
}
