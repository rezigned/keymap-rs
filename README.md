# keymap-rs

[![crates.io](https://img.shields.io/crates/v/keymap.svg)](https://crates.io/crates/keymap)
[![Rust](https://github.com/rezigned/keymap-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/rezigned/keymap-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-brightgreen.svg)](https://opensource.org/licenses/MIT)

`keymap-rs` is a library for parsing terminal input event from configurations and mapping them to the terminal library's event. (e.g. [crossterm](https://github.com/crossterm-rs/crossterm) or [termion](https://gitlab.redox-os.org/redox-os/termion))

## Introduction

Using terminal library's input event directly is sometimes not flexible. Let consider the following example of matching `ctrl-z` event:

```rs
match read() {
    // `ctrl-z`
    Event::Key(KeyEvent {
        modifiers: KeyModifiers::CONTROL,
        KeyCode::Char('z'),
        ..
    }) => {}
}
```

This code works perfectly fine. But it will prevent us from remapping the action to different input event. For example, if we want the end users to customize the key mappings to a different one (e.g. `ctrl-x`, `shift-c`, etc.).

`keymap` provides flexibility by allowing developers to define input event in plain-text, which can be used in any configuration formats (e.g. `yaml`, `toml`, etc.) and convert them to the terminal's event.

```toml
[keys]
ctrl-z = "..."
```

The `ctrl-z` above will be converted to `KeyEvent { ... }` in the first example.

## Getting started

_Please check the [examples](examples/) directory for complete examples._

<details>
<summary>
Click to show <code>Cargo.toml</code>.
</summary>

```toml
[dependencies]
keymap = "0.1"
```
</details>

Let's started by defining a simple structure for mapping input key to `String`.

```rs
use keymap::KeyMap;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config(pub HashMap<KeyMap, String>)

let config = r#"
up   = "Up"
down = "Down"
g    = "Top"
G    = "Bottom" # This is the same as `shift-g`
esc  = "Quit"
"
```

Then in your terminal library of choice (we'll be using [crossterm](https://github.com/crossterm-rs/crossterm) here). You can use any deserializer (e.g. `toml`, `json`, etc.) to deserialize a key from the configuration above into the terminal library's event (e.g. `crossterm::event::KeyEvent`).

```rs
let mapping: Config = toml::from_str(config).unwrap();

// Read input event
match read()? {
    Event::Key(key) => {
        // `KeyMap::from` will convert `crossterm::event::KeyEvent` to `keymap::KeyMap`
        if let Some(action) = config.0.get(&Key::from(key)) {
            match action {
                "Up" => println!("Move up!"),
                "Down" => println!("Move down!"),
                // ...
                "Quit" => break,
            }
        }
    }
}
```
