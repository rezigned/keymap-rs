#[cfg(feature = "derive")]
#[allow(dead_code)]
#[derive(Debug, keymap::KeyMap, Hash, PartialEq, Eq, Clone)]
pub(crate) enum Action {
    /// Jump over obstacles
    #[key("space", "@digit", symbol = "␣", help = "jump")]
    Jump,

    /// Climb or move up
    #[key("up", symbol = "↑", help = "move up")]
    Up,

    /// Drop or crouch down
    #[key("down", symbol = "↓", help = "move down")]
    Down,

    /// Move leftward
    #[key("left", symbol = "←", help = "move left")]
    Left,

    /// Move rightward
    #[key("right", symbol = "→", help = "move right")]
    Right,

    /// Exit or pause game
    #[key("q", "esc", symbol = "esc", help = "quit")]
    Quit,

    /// Key Group Capturing action (e.g. tracking which character was pressed).
    #[key("@any", help = "shoot")]
    Shoot(char),
}

#[allow(dead_code)]
fn main() {}
