Here's an improved and more comprehensive version of your `README.md` that reflects the upcoming `v1.0.0` release of `keymap-rs`. It highlights new features, enhances clarity, includes better usage documentation, and adds a structured layout that’s standard for popular Rust crates:

---

# keymap-rs

**keymap-rs** is a lightweight and extensible key mapping library for Rust applications. It supports parsing key mappings from configuration files and mapping them to actions based on input events from backends like [`crossterm`](https://crates.io/crates/crossterm), [`termion`](https://docs.rs/termion/latest/termion/), `wasm` (via `web_sys`), and others.

---

## 🔧 Features (v1.0.0)

* ✅ Declarative key mappings via configuration (e.g., YAML, JSON, etc.)
* ⌨️ Supports single keys (e.g. `a`, `enter`, `ctrl-b`, etc.) and key **sequences** (e.g. `ctrl-b n`)
* 🧠 **Key groups** via pattern matching:

  * `@upper` – uppercase letters
  * `@lower` – lowercase letters
  * `@alpha` – all alphabetic characters
  * `@alnum` – alphanumeric
  * `@any` – match any key
* 🧬 **Derive-based config parser** via `keymap_derive`
* 🌐 Backend-agnostic (works with `crossterm`, `termion`, `web_sys`, etc.)
* 🪶 Lightweight and extensible

---

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
keymap = "1.0.0"
```

---

## 🚀 Example

### Using `keymap_derive`
Define an `Action` and key mapping

```rust
/// Game actions triggered by key inputs
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

```rust
let config = Action::keymap_config();
if let Event::Key(key) = event::read()? {
    match config.get(&key) {
        Some(action) => match action {
            Action::Quit => break,

            _ => send(format!("{action:?}"))?,
        },
        None => println!("Unknown key {key:?}),
    }
}
```

---

## 📜 License

This project is licensed under the [MIT License](https://github.com/rezigned/keymap-rs/blob/main/LICENSE).

---

## 🙌 Contributions

Contributions, issues, and feature requests are welcome. If you have ideas for more backends, pattern matching rules, or integrations—feel free to open a PR!
