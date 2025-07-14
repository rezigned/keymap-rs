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

#[allow(dead_code)]
fn main() {}
