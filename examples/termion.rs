mod config;

use config::{CONFIG_DATA, Config, Action};
use keymap::KeyMap;
use termion::event::Event;
use termion::input::{TermRead, MouseTerminal};
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};

fn main() {
    let stdin = stdin();
    let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

    stdout.flush().unwrap();

    // Parse key event
    let bindings: Config = toml::from_str(CONFIG_DATA).unwrap();

    for c in stdin.events() {
        let evt = c.unwrap();

        if let Event::Key(key) = evt {
            if let Some((k, action)) = bindings.0.get_key_value(&KeyMap::from(key)) {
                if *action == Action::Quit {
                    break;
                }

                write!(stdout, "{}{}key:{k} - {}", termion::clear::All, termion::cursor::Goto(1, 1), action).unwrap();
            }
        }

        stdout.flush().unwrap();
    }
}
