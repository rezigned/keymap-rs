mod config;
mod termion_utils;

use std::io::{stdin, Write};

use config::{Action, parse_config};
use keymap::KeyMap;
use termion::event::Event;
use termion::input::TermRead;
use termion_utils::{output, print, Result};

fn main() -> Result {
    let stdin = stdin();
    let mut stdout = output();
    let bindings = parse_config();

    for event in stdin.events() {
        if let Event::Key(key) = event? {
            let mut send = |s: &str| print(&mut stdout, s);

            match bindings.0.get_key_value(&KeyMap::from(key)) {
                Some((key, action)) => {
                    if *action == Action::Quit {
                        break;
                    }
                    send(&format!("{key}"))
                }
                None => send(&format!("{key:?}")),
            }
        }

        stdout.flush().unwrap();
    }

    Ok(())
}
