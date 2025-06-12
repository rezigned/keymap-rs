# keymap-rs

**keymap-rs** is a lightweight and extensible key mapping library for Rust applications. It supports parsing key mappings from configuration files and mapping them to actions based on input events from backends like [`crossterm`](https://crates.io/crates/crossterm), [`termion`](https://docs.rs/termion/latest/termion/), `wasm`, and others.

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
* 🌐 Backend-agnostic (works with `crossterm`, `termion`, `wasm`, etc.)
* 🪶 Lightweight and extensible

---

## 📦 Installation

Run the following command:

```sh
cargo add keymap --feature {crossterm | termion | wasm}
```

---

## 🚀 Example

### Parsing keys

Parse an input key string into a `KeyMap`
```rust
assert_eq("ctrl-l".parse::<KeyMap>(), KeyMap::new(Modifier::Ctrl, Key::Char('l'))

// Same as above
assert_eq(parser::parse("ctrl-l"), KeyMap::new(Modifier::Ctrl, Key::Char('l'))
```

Parse an input key string into the backend's key event.
```rust
assert_eq!(
    keymap::backend::crossterm::parse("ctrl-l"),
    crossterm::event::KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL)
)
```
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
