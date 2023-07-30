use std::collections::HashMap;

use keymap::KeyMap;
use serde::Deserialize;
use strum_macros::Display;

#[derive(Debug, Deserialize, PartialEq, Display)]
pub(crate) enum Action {
    Bottom,
    Down,
    Explode,
    Jump,
    Top,
    Up,
    Quit,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub(crate) struct Config(pub HashMap<KeyMap, Action>);

#[allow(unused)]
pub(crate) const CONFIG_DATA: &str = r#"
up = "Up"
down = "Down"
ctrl-z = "Explode"
shift-g = "Bottom"
g = "Top"
q = "Quit"
esc = "Quit"
space = "Jump"
"#;

#[allow(unused)]
fn main() {}
