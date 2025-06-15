#[path = "../config_derive.rs"]
mod config_derive;

#[path = "./utils.rs"]
mod crossterm_utils;

use config_derive::Action;
use crossterm::{
    event::{self, read, Event, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use crossterm_utils::output;
use keymap::DerivedConfig;
use std::{
    io,
    time::{Duration, Instant},
};

fn main() -> io::Result<()> {
    let config = config_derive::derived_config();
    config_derive::print_config(&config.items);

    enable_raw_mode()?;

    handle_key_sequence(config)?;

    disable_raw_mode()
}

#[allow(dead_code)]
fn handle_key(config: DerivedConfig<Action>) -> io::Result<()> {
    let mut send = output();

    loop {
        let event = read()?;

        if let Event::Key(key) = event {
            // Or using config.get(key) if we don't need the item
            match config.get_item(&key) {
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

    Ok(())
}

#[allow(dead_code)]
fn handle_key_sequence(config: DerivedConfig<Action>) -> io::Result<()> {
    let mut send = output();

    let mut last_key: Option<KeyEvent> = None;
    let mut last_time = Instant::now();

    loop {
        if event::poll(Duration::from_millis(300))? {
            if let Event::Key(key) = event::read()? {
                match config.get(&key) {
                    Some(action) => match action {
                        Action::Quit => break,
                        Action::Up | Action::Down | Action::Left | Action::Right | Action::Jump => {
                            send(format!("{action:?}"))?
                        }
                    },
                    None => {
                        // Handle key sequence
                        if let Some(last) = last_key {
                            match config.get_seq(&[last, key]) {
                                Some(action) => send(format!(
                                    "Match key sequence: [{} {}] = {action:?}",
                                    last.code, key.code
                                ))?,
                                None => send(format!("Unknown key [{:?}]", key.code))?,
                            }
                        }

                        // Store last key
                        last_key = Some(key);
                        last_time = Instant::now();
                    }
                }

                // timeout for sequence (e.g., 1 sec)
                if last_time.elapsed() > Duration::from_secs(1) {
                    last_key = None;
                }
            }
        }
    }

    Ok(())
}
