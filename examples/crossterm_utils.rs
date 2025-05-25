#[cfg(feature = "crossterm")]
use crossterm::{
    cursor,
    execute,
    style::Print,
};

#[allow(unused)]
#[cfg(feature = "crossterm")]
pub(crate) fn output() -> impl FnMut(&str) -> std::io::Result<()> + 'static {
    let mut stdout = std::io::stdout();
    move |s| execute!(stdout, Print(s), cursor::MoveToNextLine(1))
}

#[allow(unused)]
fn main() {}
