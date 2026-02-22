#[path = "./backend/mod.rs"]
mod backend;

#[path = "./action.rs"]
mod action;

use std::time::{Duration, Instant};

use crate::backend::{print, quit, run, Key};
use action::Action;
use keymap::{DerivedConfig, KeyMapConfig};

// Override default key mapping defined via #[derive(KeyMap)] in Action.
pub(crate) const CONFIG: &str = r#"
Jump = { keys = ["j j"], description = "Jump Jump!" }
"#;

fn main() -> std::io::Result<()> {
    println!("# Example: Key Sequences (j j)");
    let config: DerivedConfig<Action> = toml::from_str(CONFIG).unwrap();

    let mut last_key: Option<Key> = None;
    let mut last_time = Instant::now();

    run(move |key| {
        let ret = match config.get(&key) {
            Some(action) => match action {
                Action::Quit => quit("quit!"),
                Action::Shoot(_) => print("Shoot!"),
                Action::Up | Action::Down | Action::Left | Action::Right | Action::Jump => print(
                    &format!("{action:?} = {}", action.keymap_item().description),
                ),
            },
            None => {
                // Handle key sequence
                let Some(last) = last_key else {
                    // Store last key
                    last_key = Some(key);
                    last_time = Instant::now();

                    return false;
                };

                match config.get_seq(&[last, key]) {
                    Some(action) => print(&format!("Key sequence: {action:?}")),
                    None => print(&format!("Unknown key [{key:?}]")),
                }
            }
        };

        // timeout for sequence (e.g., 1 sec)
        if last_time.elapsed() > Duration::from_secs(1) {
            last_key = None;
        }

        ret
    })
}
