# keymap-rs

[![Crates.io](https://img.shields.io/crates/v/keymap.svg)](https://crates.io/crates/keymap)
[![Docs.rs](https://docs.rs/keymap/badge.svg)](https://docs.rs/keymap)
[![CI](https://github.com/rezigned/keymap-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/rezigned/keymap-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/keymap.svg)](https://github.com/rezigned/keymap-rs/blob/main/LICENSE)

**keymap-rs** is a lightweight and extensible key mapping library for Rust that simplifies input processing for terminal user interfaces (TUIs), WebAssembly (WASM) applications, and more. It parses keymaps from derive macros or configuration files and maps them to actions from various input backends, including [`crossterm`](https://crates.io/crates/crossterm), [`termion`](https://docs.rs/termion/latest/termion/), and [`wasm`](https://webassembly.org/).

---

## 🔧 Features

* ✅ **Declarative Key Mappings**: Define keymaps via simple configuration (e.g., TOML, YAML) or directly in your code using derive macros.
* ⌨️ **Key Patterns**: Supports single keys (`a`), combinations (`ctrl-b`), and multi-key sequences (`ctrl-b n`).
* 🧠 **Key Groups**: Use built-in pattern matching for common key groups:
  * `@upper` – Uppercase letters
  * `@lower` – Lowercase letters
  * `@alpha` – All alphabetic characters
  * `@alnum` – Alphanumeric characters
  * `@any` – Match any key
* 🧬 **Compile-Time Safety**: The `keymap_derive` macro validates key syntax at compile time, preventing runtime errors.
* 🌐 **Backend Agnostic**: Works with multiple backends, including `crossterm`, `termion`, and `wasm`.
* 🪶 **Lightweight & Extensible**: Designed to be minimal and easy to extend with new backends or features.

---

## 🕹️ Demo

See `keymap-rs` in action with the [WASM example](https://rezigned.com/keymap-rs/):

<p align="center">
  <img src="./examples/wasm/public/preview.png" alt="keymap-rs WASM Demo" width="700">
</p>

---

## 📦 Installation

Add `keymap` to your `Cargo.toml`, enabling the feature for your chosen backend:

```sh
cargo add keymap --feature {crossterm | termion | wasm}
```

---

## 🚀 Usage

### 1. Deriving Keymaps

The easiest way to get started is with the `keymap::KeyMap` derive macro.

**Define your actions:**

```rust
use keymap::KeyMap;

/// Application actions.
#[derive(KeyMap, Debug, PartialEq, Eq)]
pub enum Action {
    /// Quit the application.
    #[key("q", "esc")]
    Quit,

    /// Move left.
    #[key("left", "h")]
    Left,

    /// Move right.
    #[key("right", "l")]
    Right,

    /// Jump.
    #[key("space")]
    Jump,
}
```

**Use the generated keymap:**

The `KeyMap` derive macro generates an associated `keymap_config()` method that returns a `Config<Action>`.

```rust
// Retrieve the config
let config = Action::keymap_config();

// `key` is a key code from the input backend, e.g., `crossterm::event::KeyCode`
match config.get(&key) {
    Some(action) => match action {
        Action::Quit => break,
        Action::Jump => println!("Jump!"),
        _ => println!("Action: {action:?} - {}", action.keymap_item().description),
    }
    _ => {}
}
```

### 2. Using External Configuration

You can also load keymaps from external files (e.g., `config.toml`). This is useful for user-configurable keybindings.

**Example `config.toml`:**

```toml
# Override or add new keybindings
Jump = { keys = ["j", "up"], description = "Jump with 'j' or up arrow!" }
Quit = { keys = ["@any"], description = "Quit on any key press." }
```

You have two ways to load this configuration:

#### `Config<T>`: Load from File Only

This deserializes **only** the keybindings from the configuration file, ignoring any `#[key("...")]` attributes on your enum.

```rust
// This config will only contain 'Jump' and 'Quit' from the TOML file.
let config: Config<Action> = toml::from_str(config_str).unwrap();
```

| Key           | Action |
| ------------- | ------ |
| `"j"`, `"up"` | Jump   |
| `@any`        | Quit   |

#### `DerivedConfig<T>`: Merge Derived and File Configs

This **merges** the keybindings from the `#[key("...")]` attributes with the ones from the configuration file. Keys from the external file will override any conflicting keys defined in the enum.

```rust
// This config contains keys from both the derive macro and the TOML file.
let config: DerivedConfig<Action> = toml::from_str(config_str).unwrap();
```

| Key                      | Action |
| ------------------------ | ------ |
| `"j"`, `"up"`            | Jump   |
| `"h"`, `"left"`          | Left   |
| `"l"`, `"right"`         | Right  |
| `@any`                   | Quit   |
| *`"q"`, `"esc"`, `"space"` are ignored* |

### 3. Compile-Time Validation

The `keymap_derive` macro validates all key strings at **compile time**, so you get immediate feedback on invalid syntax.

**Invalid Key Example:**

```rust
#[derive(keymap::KeyMap)]
enum Action {
    // "enter2" is not a valid key.
    #[key("enter2", "ctrl-b n")]
    Invalid,
}
```

**Compiler Error:**

This code will fail to compile with a clear error message:

```
error: Invalid key "enter2": Parse error at position 5: expect end of input, found: 2
 --> keymap_derive/tests/derive.rs:7:11
  |
7 |     #[key("enter2", "ctrl-b n")]
  |           ^^^^^^^^
```

### 4. Direct Key Parsing

You can also parse key strings directly into a `KeyMap` or a backend-specific key event.

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use keymap::{backend::crossterm::parse, Key, KeyMap, Modifier};

// Parse into a generic KeyMap
assert_eq!(
    "ctrl-l".parse::<KeyMap>(),
    Ok(KeyMap::new(Some(Modifier::Ctrl), Key::Char('l')))
);

// Or use the backend-specific parser
assert_eq!(
    parse("ctrl-l").unwrap(),
    KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)
);
```

---

## 📖 Examples

For complete, runnable examples, check out the [`/examples`](https://github.com/rezigned/keymap-rs/tree/main/examples) directory, which includes demos for:
- `crossterm`
- `termion`
- `wasm`

---

## 📜 License

This project is licensed under the [MIT License](https://github.com/rezigned/keymap-rs/blob/main/LICENSE).

---

## 🙌 Contributions

Contributions, issues, and feature requests are welcome! Feel free to open an issue or submit a pull request.

