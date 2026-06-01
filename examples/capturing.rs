#[path = "./backend/mod.rs"]
mod backend;

#[path = "./action.rs"]
mod action;

use crate::backend::{print, print_config, quit, run};
use action::Action;
use keymap::{DerivedConfig, KeyMapConfig};

// In this example, we showcase Key Group Capturing using .get_bound()
// The Action::Shoot(char) variant is mapped to @any in action.rs.
pub(crate) const CONFIG: &str = r#"
Jump = { keys = ["j"], description = "Jump!" }
"#;

fn main() -> std::io::Result<()> {
    println!("# Example: Key Group Capturing using .get_bound()");

    let config: DerivedConfig<Action> = toml::from_str(CONFIG).unwrap();
    print_config(&config.items);

    run(|key| match config.get_bound(&key) {
        Some(action) => match action {
            Action::Quit => quit("quit!"),
            // This is matched via @any and the char is dynamically bound
            Action::Shoot(c) => print(&format!("Matched @any! Captured character: '{c}'")),
            // Standard unit variants work as before
            Action::Jump | Action::Up | Action::Down | Action::Left | Action::Right => {
                print(&format!(
                    "Action: {action:?} (Description: {})",
                    action.keymap_item().description
                ))
            }
        },
        None => print(&format!("Unknown key {key:?}")),
    })
}
