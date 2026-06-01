#[path = "./backend/mod.rs"]
mod backend;

#[path = "./action.rs"]
mod action;

use std::cell::RefCell;
use std::io::Write;

use crate::backend::{print, print_config, quit, run};
use action::Action;
use keymap::{DerivedConfig, KeyMap, KeyMapConfig, ToKeyMap};

// Multiple inline configs to switch between at runtime.
// Press 'r' to reload the keymap — the active config rotates on each reload.
const CONFIG_A: &str = r#"
Jump = { keys = ["j"] }
Up   = { keys = ["k"] }
"#;

const CONFIG_B: &str = r#"
Jump = { keys = ["w"] }
Down = { keys = ["s"] }
Left = { keys = ["a"] }
Right = { keys = ["d"] }
"#;

fn main() -> std::io::Result<()> {
    println!("# Example: Runtime keymap reload");
    println!("\rPress 'r' to rotate between configs at runtime");

    let configs = [CONFIG_A, CONFIG_B];
    let config = RefCell::new(toml::from_str::<DerivedConfig<Action>>(configs[0]).unwrap());
    let active = RefCell::new(0usize);

    print_config(&config.borrow().items);

    let reload_key = KeyMap::from(keymap::node::Key::Char('r'));

    run(move |key| {
        // Press 'r' to reload keymap from the next inline config
        if let Ok(k) = key.to_keymap() {
            if k == reload_key {
                let next = (*active.borrow() + 1) % configs.len();
                *config.borrow_mut() = toml::from_str(configs[next]).unwrap();
                *active.borrow_mut() = next;

                // Replace the stale header shortcuts in-place
                print!("\r\x1b[1A\x1b[J");
                std::io::stdout().flush().ok();
                print_config(&config.borrow().items);
                print(&format!("** switched to config {next} **"));
                return false;
            }
        }

        let config = config.borrow();
        match config.get(&key) {
            Some(action) => match action {
                Action::Quit => quit("quit!"),
                Action::Shoot(_) => print("Shoot!"),
                Action::Up | Action::Down | Action::Left | Action::Right | Action::Jump => print(
                    &format!("{action:?} = {}", action.keymap_item().description),
                ),
            },
            None => print(&format!("{key:?}")),
        }
    })
}
