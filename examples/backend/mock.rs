use std::io;

use keymap::ToKeyMap;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
pub(crate) enum Key {}

impl ToKeyMap for Key {
    fn to_keymap(&self) -> Result<keymap::KeyMap, keymap::Error> {
        todo!()
    }
}

#[allow(dead_code)]
pub(crate) fn run<F>(mut f: F) -> io::Result<()>
where
    F: FnMut(Key) -> bool,
{
    // no-op
    Ok(())
}

#[allow(unused)]
fn main() {}
