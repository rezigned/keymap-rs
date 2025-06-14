# keymap-rs

**keymap-rs** is a lightweight and extensible key mapping library for Rust applications. It supports parsing key mappings from configuration files and mapping them to actions based on input events from backends like [`crossterm`](https://crates.io/crates/crossterm), [`termion`](https://docs.rs/termion/latest/termion/), `wasm` (via `web_sys`), and others.

---

## 🔧 Features (v1.0.0)

* ✅ Declarative key mappings via configuration (e.g., YAML, JSON, etc.)
* ⌨️ Supports single keys (e.g. `a`, `enter`, `ctrl-b`, etc.) and key **sequences** (e.g. `ctrl-b n`)
* 🧠 Supports **key groups**:
  * `@upper` – uppercase letters
  * `@lower` – lowercase letters
  * `@alpha` – all alphabetic characters
  * `@alnum` – alphanumeric
  * `@any` – match any key
* 🧬 **Derive-based config parser** via `keymap_derive`
* 🌐 Backend-agnostic (works with `crossterm`, `termion`, `web_sys`, etc.)
* 🪶 Lightweight and extensible

---

## WASM Backend (Experimental)

This crate includes an experimental WASM backend for handling keyboard events in a browser environment. It allows you to convert `web_sys::KeyboardEvent` objects directly into `KeyMap` instances.

**Enable the feature:**

Add the `wasm` feature in your `Cargo.toml`:

```toml
[dependencies.keymap] # Or your crate name
version = "..." # Specify your crate version
features = ["wasm"]
```

**Usage Example:**

This example shows how to capture `web_sys::KeyboardEvent` from the browser and convert it to `KeyMap` using the `ToKeyMap` trait.

```rust
use keymap::keymap::{ToKeyMap, KeyMap}; // Make sure ToKeyMap is in scope
use web_sys::KeyboardEvent; // Requires web-sys dependency with "KeyboardEvent" feature

fn handle_browser_event(event: &KeyboardEvent) {
    match event.to_keymap() { // Use the ToKeyMap trait directly
        Ok(keymap) => {
            // Now you have a KeyMap instance
            println!("KeyMap: {:?}", keymap);

            // If using with keymap::Config:
            // let config: keymap::Config<String> = keymap::Config::default();
            // ... populate config ...
            // if let Some(action) = config.get(event) { // Pass the original KeyboardEvent
            //     println!("Action found: {}", action);
            // }
        }
        Err(e) => {
            eprintln!("Failed to convert KeyboardEvent to KeyMap: {:?}", e);
        }
    }
}

// In your WASM setup (e.g., a function marked with #[wasm_bindgen]):
// You would get the KeyboardEvent from a JavaScript event listener.
// For a full example, see `examples/wasm/main.rs`.
```

The `parse` function available under the `wasm` feature (`keymap::backend::parse` if wasm is the active backend) can be used to convert string representations (e.g., "ctrl-a") directly into a `web_sys::KeyboardEvent`:
```rust
// use keymap::backend::parse; // Assuming wasm feature is active
// use web_sys::KeyboardEvent;  // For type annotation
//
// match parse("ctrl-shift-x") {
//     Ok(event_instance) => {
//         // event_instance is a web_sys::KeyboardEvent
//         // This can be used with config.get(&event_instance)
//         println!("Parsed into KeyboardEvent: key='{}'", event_instance.key());
//     },
//     Err(e) => { /* ... */ },
// }
```

---

**Important Note on Backend Features:**

The backend features (`crossterm`, `termion`, `wasm`) are **mutually exclusive**. You must enable only **one** of these features at a time in your `Cargo.toml`. Enabling multiple backend features simultaneously will result in a compilation error (E0119, due to conflicting trait implementations for `BackendConfig`).

*   **Default Backend:** This crate might enable one backend by default (e.g., `crossterm`). Check the `[features]` section in the crate's `Cargo.toml` (or on crates.io) to see which backend is included in the `default` features.
*   **Selecting a Specific Backend:** If you wish to use a different backend (e.g., `wasm` or `termion`), you **must** disable default features and explicitly enable only your desired backend feature.

**Example: Enabling only the `wasm` backend**
```toml
[dependencies.keymap] # Or your crate's actual name
version = "YOUR_CRATE_VERSION" # Replace with the actual crate version
default-features = false     # Crucial: disables the default backend feature
features = ["wasm"]          # Explicitly enable only the WASM backend
```

**Example: Enabling only the `termion` backend**
```toml
[dependencies.keymap] # Or your crate's actual name
version = "YOUR_CRATE_VERSION" # Replace with the actual crate version
default-features = false
features = ["termion"]
```

---

## 📦 Installation

Run the following command:

> \[!NOTE]
> By default, this installs with `crossterm` as the default backend. You can enable a different backend by specifying the feature flag:
>
> ```sh
> cargo add keymap --features termion  # or web_sys, etc.
> ```

```sh
cargo add keymap
```

---

## 🚀 Example

### Using `keymap_derive`

Define your actions and key mappings:

```rust
/// Game actions
#[derive(keymap::KeyMap, Debug)]
pub enum Action {
    /// Rage quit the game
    #[key("q", "esc")]
    Quit,

    /// Step left (dodge the trap!)
    #[key("left")]
    Left,

    /// Step right (grab the treasure!)
    #[key("right")]
    Right,

    /// Jump over obstacles (or just for fun)
    #[key("space")]
    Jump,
}
```

Use the config:

```rust
let config = Action::keymap_config();

if let Event::Key(key) = event::read()? {
    match config.get(&key) {
        Some(action) => match action {
            Action::Quit => break,
            Action::Jump => println!("Jump Jump!"),
            _ => println!("{:?} - {}", action, action.keymap_item().description),
        },
        None => println!("Unknown key {:?}", key),
    }
}
```

### Using external configuration (e.g. `toml`, `yaml`, etc.)

Define a config:

```toml
Jump = { keys = ["j", "up"], description = "Jump with 'j'!" }
Quit = { keys = ["@any"], description = "Quit!" }
```

#### Deserialize with `Config<T>`

> [!NOTE]
> The table below shows all keys that are deserialized only from the configuration file. Keys defined via `#[key("..")]` are **not** included.
>
> | Key           | Action |
> | ------------- | ------ |
> | `"j"`, `"up"` | Jump   |
> | `@any`        | Quit   |

```rust
let config: Config<Action> = toml::from_str("./config.toml").unwrap();
```

#### Deserialize with `DerivedConfig<T>`

> [!NOTE]
> The table below shows all keys when using both the configuration file **and** the keys defined via `#[key("..")]`. The sets are merged.
>
> | Key           | Action |
> | ------------- | ------ |
> | `"j"`, `"up"` | Jump   |
> | `"left"`      | Left   |
> | `"right"`     | Right  |
> | `@any`        | Quit   |

```rust
let config: DerivedConfig<Action> = toml::from_str("./config.toml").unwrap();
```

---
### 🛠️ Bonus: Compile-time Validation

One powerful advantage of using the `#[key(".."))]` attribute macro from `keymap_derive` is that invalid key definitions are caught at **compile time**, ensuring early feedback and safety.

#### Example: Invalid Key

```rust
#[derive(keymap::KeyMap)]
enum Action {
    #[key("enter2", "ctrl-b n")]
    Invalid,
}
```

#### Compile Error

```
error: Invalid key "enter2": Parse error at position 5: expect end of input, found: 2
 --> keymap_derive/tests/derive.rs:7:11
  |
7 |     #[key("enter2", "ctrl-b n")]
  |           ^^^^^^^^
```

This prevents runtime surprises and provides clear diagnostics during development.

## 📜 License

This project is licensed under the [MIT License](https://github.com/rezigned/keymap-rs/blob/main/LICENSE).

---

## 🙌 Contributions

Contributions, issues, and feature requests are welcome. Have an idea for a new backend, pattern rule, or integration? Open a PR!
