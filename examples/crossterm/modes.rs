use std::collections::HashMap;

use keymap::{DerivedConfig, ToKeyMap};
use serde::Deserialize;

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
    ExitMode,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Actions {
    Home(DerivedConfig<HomeAction>),
    Edit(DerivedConfig<EditAction>),
}

#[allow(unused)]
pub(crate) const DERIVED_CONFIG: &str = r#"
[home]
Quit = { keys = ["esc", "q"], description = "Quit the app" }
Edit = { keys = ["e"], description = "Enter edit mode" }

[edit]
ExitMode = { keys = ["esc", "q"], description = "Exit edit mode" }
"#;

type Modes = HashMap<String, Actions>;

fn main() {
    let modes: Modes = toml::from_str(DERIVED_CONFIG).unwrap();
    let mode = "home";

    match mode {
        "home" => {
            if let Some(Actions::Home(config)) = modes.get(mode) {
                // config.get(key)
            }
        }
        "edit" => {
            if let Some(Actions::Edit(config)) = modes.get(mode) {
                // config.get(key)
            }
        }
        _ => {}
    }
}
