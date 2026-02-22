#[cfg(feature = "derive")]
#[derive(Debug, keymap::KeyMap, Hash, PartialEq, Eq, Clone)]
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

    /// Key Group Capturing action (e.g. tracking which character was pressed).
    /// `char` will be captured from any matched key group macro (like `@any` or `@digit`) at runtime.
    #[key("@any")]
    Shoot(char),
}

#[allow(dead_code)]
fn main() {}
