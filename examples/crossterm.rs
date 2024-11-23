use std::io::{self, Stdout};
mod config;

use config::{Action, Config, CONFIG_DATA};
use crossterm::{
    cursor,
    event::{read, Event},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use keymap::KeyMap;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;

    read_event(&mut stdout)?;

    disable_raw_mode()
}

fn read_event(stdout: &mut Stdout) -> io::Result<()> {
    let config: Config = toml::from_str(CONFIG_DATA).unwrap();

    loop {
        let event = read()?;

        if let Event::Key(key) = event {
            if let Some((k, action)) = config.0.get_key_value(&KeyMap::from(key)) {
                if *action == Action::Quit {
                    break;
                }

                execute!(
                    stdout,
                    Print(format!("key:{} - {}\n", k, action)),
                    cursor::MoveToNextLine(1),
                )?;
            }
        }
    }

    Ok(())
}
