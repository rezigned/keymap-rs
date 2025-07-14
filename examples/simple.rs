use keymap::{KeyMap, ToKeyMap};
use serde::Deserialize;
use std::collections::HashMap;

#[path = "./backend/mod.rs"]
mod backend;
use crate::backend::{print, quit, run};

#[derive(Debug, Deserialize, PartialEq)]
enum Action {
    Jump,
    Up,
    Down,
    Left,
    Right,
    Quit,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
struct Config(pub HashMap<KeyMap, Action>);

#[allow(unused)]
const CONFIG: &str = r#"
up     = "Up"
down   = "Down"
left   = "Left"
right  = "Right"
ctrl-g = "Jump"
space  = "Jump"
q      = "Quit"
esc    = "Quit"
"#;

fn main() -> std::io::Result<()> {
    let config: Config = toml::from_str(CONFIG).unwrap();
    println!("# Example: Basic key mapping without derive macros");

    run(|key| {
        let Some((_, action)) = config.0.get_key_value(&key.to_keymap().unwrap()) else {
            print(&format!("{key:?}"));
            return false;
        };

        match action {
            Action::Up => print("Up!"),
            Action::Down => print("Down!"),
            Action::Jump => print("Jump!"),
            Action::Left => print("Left!"),
            Action::Right => print("Right!"),
            Action::Quit => quit("quit"),
        }
    })
}
