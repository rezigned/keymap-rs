#[path = "./backend/mod.rs"]
mod backend;

#[path = "./action.rs"]
mod action;

use crate::backend::{print, quit, run};
use keymap::KeyMapConfig;
use action::Action;

fn main() -> std::io::Result<()> {
    println!("# Example: Using the KeyMap derive macro");
    let config = Action::keymap_config();

    run(|key| match config.get(&key) {
        Some(action) => match action {
            Action::Quit => quit("quit!"),
            Action::Up | Action::Down | Action::Left | Action::Right | Action::Jump => {
                print(&format!("{action:?}"))
            }
        },
        None => print(&format!("Unknown key {key:?}")),
    })
}
