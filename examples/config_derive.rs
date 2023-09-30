#[cfg(feature = "derive")]
#[derive(Debug, keymap::KeyMap)]
pub(crate) enum Action {
    #[key("space", "ctrl-g")]
    Jump,
    #[key("up")]
    Up,
    #[key("down")]
    Down,
    #[key("left")]
    Left,
    #[key("right")]
    Right,
    #[key("q", "esc")]
    Quit,
}

#[allow(unused)]
fn main() {}
