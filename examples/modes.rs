use std::collections::HashMap;

#[path = "./backend/mod.rs"]
mod backend;

use crate::backend::{print, quit, run};
use keymap::DerivedConfig;
use serde::Deserialize;

#[derive(keymap::KeyMap, Debug, Hash, Eq, PartialEq, Clone)]
enum HomeAction {
    #[key("esc")]
    Quit,
    #[key("e")]
    Edit,
}

#[derive(keymap::KeyMap, Debug, Hash, Eq, PartialEq, Clone)]
enum EditAction {
    #[key("esc")]
    Exit,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Actions {
    Home(DerivedConfig<HomeAction>),
    Edit(DerivedConfig<EditAction>),
}

type Modes = HashMap<String, Actions>;

#[allow(unused)]
pub(crate) const CONFIG: &str = r#"
[home]
Quit = { keys = ["esc", "q"], description = "Quit the app" }
Edit = { keys = ["e"], description = "Enter edit mode" }

[edit]
Exit = { keys = ["esc", "q"], description = "Exit edit mode" }
"#;

fn main() -> std::io::Result<()> {
    let modes: Modes = toml::from_str(CONFIG).unwrap();
    let mut mode = "home";

    println!("# Example: Multi-mode application with different key mappings");
    println!("mode: {mode}\r");

    run(move |key| match modes.get(mode).unwrap() {
        Actions::Home(config) => match config.get(&key) {
            Some(action) => match action {
                HomeAction::Quit => quit("quit!"),
                HomeAction::Edit => {
                    mode = "edit";
                    print("enter edit mode!")
                }
            },
            None => print(&format!("{key:?}")),
        },
        Actions::Edit(config) => match config.get(&key) {
            Some(action) => match action {
                EditAction::Exit => {
                    mode = "home";
                    print("exit edit mode!")
                }
            },
            None => print(&format!("{key:?}")),
        },
    })
}
