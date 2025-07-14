use std::collections::HashMap;

use keymap::DerivedConfig;
use serde::Deserialize;

use crate::crossterm_utils::{print, quit, run};

#[path = "./crossterm/utils.rs"]
mod crossterm_utils;

#[derive(keymap::KeyMap, Deserialize, Debug, Hash, Eq, PartialEq)]
enum HomeAction {
    #[key("esc")]
    Quit,
    #[key("e")]
    Edit,
}

#[derive(keymap::KeyMap, Deserialize, Debug, Hash, Eq, PartialEq)]
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
            None => print(&format!("{}", key.code)),
        },
        Actions::Edit(config) => match config.get(&key) {
            Some(action) => match action {
                EditAction::Exit => {
                    mode = "home";
                    print("exit edit mode!")
                }
            },
            None => print(&format!("{}", key.code)),
        },
    })
}
