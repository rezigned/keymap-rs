#[path = "./backend/mod.rs"]
mod backend;

#[path = "./action.rs"]
mod action;

use crate::backend::{print, quit, run};
use action::Action;
use keymap::{Config, KeyMapConfig};

// Override default key mapping defined via #[derive(KeyMap)] in Action.
pub(crate) const CONFIG: &str = r#"
Jump = { keys = ["j"], description = "Jump Jump!" }
Quit = { keys = ["esc"], description = "Quit with ESC only!" }
"#;

fn main() -> std::io::Result<()> {
    println!("# Example: External configuration with Config<T>");

    let config: Config<Action> = toml::from_str(CONFIG).unwrap();

    // Use .get() for high-performance reference lookup of the "default" variant.
    // To capture the actual key pressed (e.g. the 'a' in @any), use .get_bound()
    // or see the `capturing` example.
    run(|key| match config.get(&key) {
        Some(action) => match action {
            Action::Quit => quit("quit!"),
            // Standard unit variants work as before
            Action::Up | Action::Down | Action::Left | Action::Right | Action::Jump => print(
                &format!("{action:?} = {}", action.keymap_item().description),
            ),
            Action::Shoot(_) => print("Shoot! (Use .get_bound() to capture the character)"),
        },
        None => print(&format!("Unknown key {key:?}")),
    })
}
