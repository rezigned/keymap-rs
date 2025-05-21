# keymap-rs

[![crates.io](https://img.shields.io/crates/v/keymap.svg)](https://crates.io/crates/keymap)
[![Rust](https://github.com/rezigned/keymap-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/rezigned/keymap-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-brightgreen.svg)](https://opensource.org/licenses/MIT)

`keymap-rs` is a library for parsing terminal input events from configurations and mapping them to terminal library events (e.g., [crossterm](https://github.com/crossterm-rs/crossterm) or [termion](https://gitlab.redox-os.org/redox-os/termion)).

## Features

- **Configuration-driven key mapping**: Define key bindings in plain text formats like TOML, YAML, or JSON
- **Multiple terminal library support**: Works with popular Rust terminal libraries
- **Customizable key bindings**: Allow end users to customize key bindings without code changes
- **Derive macro** (`derive` feature): Easily map keys to enum variants with the `KeyMap` derive macro

## Installation

You can add `keymap-rs` to your project in several ways:

### Using `Cargo.toml`

```toml
[dependencies]
keymap = { version = "0.5", features = ["derive"] }
```

### Using `cargo add` command

```bash
# With derive macro support
cargo add keymap --features derive
```

> Note: The derive macro functionality requires enabling the `derive` feature as shown above.

## Quick Start

### Using the Derive Macro

The derive macro provides a simple and ergonomic way to define key mappings (requires the `derive` feature).

First, define an enum with the `KeyMap` derive macro:

```rust
#[derive(KeyMap)]
enum Action {
    #[key("left", "a")]
    Left,

    #[key("right", "d")]
    Right,

    #[key("q", "ctrl-c")]
    Quit,
}
```

Then, use the `try_from` method to convert the key event into an enum variant:

```rust
// In your event loop
loop {
    let key = read_key(); // Get key event from your terminal library

    match Action::try_from(KeyMap::from(key)) {
        Ok(action) => match action {
            Action::Left => println!("Left!"),
            Action::Right => println!("Right!"),
            Action::Quit => break,
        },
        Err(_) => {} // Key not mapped to any action
    }
}
```

### Using Configuration (toml, yaml, json)

For more dynamic configuration:

First, define a configuration (toml):

```toml
# config.toml
[keys]
left = "Left"
right = "Right"
ctrl-c = "Quit"
```

Then, use the `HashMap<KeyMap, Action>` type to map keys to actions:

```rust
use std::collections::HashMap;
use serde::Deserialize;
use keymap::KeyMap;

#[derive(Deserialize)]
struct Config {
    pub keys: HashMap<KeyMap, String>
}

// Load configuration from file or string
let config: Config = toml::from_str(include_str!("config.toml"))?;

// In your event loop
loop {
    let key = read_key(); // Get key event from your terminal library

    if let Some(action) = config.keys.get(&KeyMap::from(key)) {
        match action.as_str() {
            "Left" => println!("Left!"),
            "Right" => println!("Right!"),
            "Quit" => break,
            _ => {}
        }
    }
}
```

## Why Use keymap-rs?

Using terminal library's input events directly can be verbose and inflexible. Consider this example of matching a `ctrl-z` event with crossterm:

```rust
match read()? {
    // `ctrl-z`
    Event::Key(KeyEvent {
        modifiers: KeyModifiers::CONTROL,
        code: KeyCode::Char('z'),
        ..
    }) => {
        // Handle action
    }
}
```

With `keymap-rs`, you can define this binding in a configuration file:

```toml
[keys]
ctrl-z = "Undo"
```

This approach provides several benefits:
- **User customization**: Let users change key bindings without modifying code
- **Readability**: Key mappings are more readable in configuration format
- **Flexibility**: Easily switch between different key binding schemes

## Supported Key Formats

`keymap-rs` supports a human-readable format for key combinations:

- Single keys: `a`, `b`, `1`, `2`, etc.
- Special keys: `up`, `down`, `left`, `right`, `home`, `end`, etc.
- Modifier combinations: `ctrl-a`, `shift-home`, `alt-enter`, etc.

## Supported Terminal Libraries

- [crossterm](https://github.com/crossterm-rs/crossterm)
- [termion](https://gitlab.redox-os.org/redox-os/termion)

## Examples

Check out the [examples](examples/) directory for complete examples showing how to:
- Use the derive macro for key mapping
- Load key bindings from configuration files
- Integrate with crossterm and termion

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.