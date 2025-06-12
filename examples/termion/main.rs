#[path = "../config.rs"]
mod config;
#[path = "./utils.rs"]
mod termion_utils;

use std::io::{stdin, Write};

use config::{parse_config, Action};
use keymap::ToKeyMap;
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

            match bindings.0.get_key_value(&key.to_keymap().unwrap()) {
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
