use std::collections::HashMap;
use keymap::KeyMap;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) enum Action {
    Jump,
    Up,
    Down,
    Left,
    Right,
    Quit,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub(crate) struct Config(pub HashMap<KeyMap, Action>);

#[allow(unused)]
pub(crate) const CONFIG_DATA: &str = r#"
up = "Up"
down = "Down"
left = "Left"
right = "Right"
ctrl-g = "Jump"
space = "Jump"
q = "Quit"
esc = "Quit"
"#;

#[allow(unused)]
pub(crate) fn parse_config() -> Config {
    toml::from_str(CONFIG_DATA).unwrap()
}

#[allow(unused)]
fn main() {}
