#[path = "./backend/mod.rs"]
mod backend;

#[path = "./action.rs"]
mod action;

use crate::backend::{print, quit, run};
use action::Action;
use keymap::KeyMapConfig;

fn main() -> std::io::Result<()> {
    println!("# Example: Using the KeyMap derive macro");
    let config = Action::keymap_config();

    // Use .get() for high-performance reference lookup of the "default" variant.
    // To capture the actual key pressed (e.g. the 'a' in @any), use .get_bound()
    // or see the `capturing` example.
    run(|key| match config.get(&key) {
        Some(action) => match action {
            Action::Quit => quit("quit!"),
            Action::Shoot(_) => print("Shoot! (Use .get_bound() to capture the character)"),
            // Standard unit variants work as before
            Action::Jump | Action::Up | Action::Down | Action::Left | Action::Right => print(
                &format!("{action:?} = {}", action.keymap_item().description),
            ),
        },
        None => print(&format!("Unknown key {key:?}")),
    })
}
