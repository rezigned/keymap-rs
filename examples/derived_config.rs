#[path = "./backend/mod.rs"]
mod backend;

#[path = "./action.rs"]
mod action;

use crate::backend::{print, quit, run};
use action::Action;
use keymap::{DerivedConfig, KeyMapConfig};

// Override default key mapping defined via #[derive(KeyMap)] in Action.
pub(crate) const CONFIG: &str = r#"
Jump = { keys = ["j"], description = "Jump Jump!" }
Up = { keys = ["u"], description = "Fly!" }
Quit = { keys = ["@digit"], description = "Quit!" }
"#;

fn main() -> std::io::Result<()> {
    println!("# Example: Merging derive macros with external config using DerivedConfig<T>");

    let config: DerivedConfig<Action> = toml::from_str(CONFIG).unwrap();

    // Use .get() for high-performance reference lookup of the "default" variant.
    // To capture the actual key pressed (e.g. the 'a' in @any), use .get_bound()
    // or see the `capturing` example.
    run(|key| match config.get(&key) {
        Some(action) => match action {
            Action::Quit => quit("quit!"),
            Action::Shoot(_) => print("Shoot! (Use .get_bound() to capture the character)"),
            Action::Up | Action::Down | Action::Left | Action::Right | Action::Jump => print(
                &format!("{action:?} = {}", action.keymap_item().description),
            ),
        },
        None => print(&format!("Unknown key {key:?}")),
    })
}
