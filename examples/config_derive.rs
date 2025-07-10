#[cfg(feature = "derive")]
use keymap::{DerivedConfig, Item};

#[cfg(feature = "derive")]
use serde::Deserialize;

#[cfg(feature = "derive")]
#[derive(Debug, keymap::KeyMap, Deserialize, Hash, PartialEq, Eq)]
pub(crate) enum Action {
    /// Jump over obstacles
    #[key("space", "@digit")]
    Jump,

    /// Climb or move up
    #[key("up")]
    Up,

    /// Drop or crouch down
    #[key("down")]
    Down,

    /// Move leftward
    #[key("left")]
    Left,

    /// Move rightward
    #[key("right")]
    Right,

    /// Exit or pause game
    #[key("q", "esc")]
    Quit,
}

/// Overrides the default keymap
#[allow(unused)]
pub(crate) const DERIVED_CONFIG: &str = r#"
Jump = { keys = ["j"], description = "Jump Jump!" }
Up = { keys = ["u", "g g"], description = "Fly!" }
Quit = { keys = ["@digit"], description = "Quit!" }
"#;

#[cfg(feature = "derive")]
#[allow(unused)]
pub(crate) fn derived_config() -> DerivedConfig<Action> {
    toml::from_str(DERIVED_CONFIG).unwrap()
}

#[cfg(feature = "derive")]
#[allow(unused)]
pub(crate) fn print_config(items: &[(Action, Item)]) {
    println!("--- keymap ---");

    items
        .iter()
        .map(|(action, v)| {
            println!(
                "{action:?} = keys: {:?}, description: {}",
                v.keys, v.description
            )
        })
        .collect::<Vec<_>>();

    println!("--------------");
}

#[allow(unused)]
fn main() {}
