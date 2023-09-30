mod config_derive;
mod termion_utils;

use keymap::KeyMap;
use std::io::{Write, stdin};
use termion::event::Event;
use termion::input::TermRead;
use config_derive::Action;
use termion_utils::{print, Result, output};

fn main() -> Result {
    let stdin = stdin();
    let mut stdout = output();

    for event in stdin.events() {
        let mut send = |s: &str| print(&mut stdout, s);

        if let Event::Key(key) = event? {
            match Action::try_from(KeyMap::from(key)) {
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
