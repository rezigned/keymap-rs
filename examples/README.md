# Examples

This directory contains examples demonstrating various features and use cases of the `keymap-rs` library. Each example showcases different aspects of key mapping, from basic usage to advanced configurations.

> [!NOTE]
>
> All examples below work with any backend by simply passing the backend feature (e.g., `crossterm`, `termion`, `wasm`) to the `--features` argument.
>
> ```
> cargo run --example {example} --features {backend}
> ```

### [`simple.rs`](./simple.rs)
**Basic key mapping without derive macros**

Illustrates how to use the library without the `#[derive(KeyMap)]` macro, including manual TOML configuration parsing and basic action handling.


### [`derive.rs`](./derive.rs)
**Using the KeyMap derive macro**

Presents the most common and recommended approach using the `#[derive(KeyMap)]` macro, showcasing automatic keymap generation from enum attributes and clean, declarative key mapping.


### [`config.rs`](./config.rs)
**External configuration with Config<T>**

Shows how to load key mappings exclusively from external configuration files, ignoring derive macro definitions, and highlights file-based key overrides and custom key descriptions.

### [`derived_config.rs`](./derived_config.rs)
**Merging derive macros with external config using DerivedConfig<T>**

Explores combining derive macro defaults with external configuration overrides, covering configuration precedence and key group patterns like `@digit`.


### [`modes.rs`](./modes.rs)
**Multi-mode application with different key mappings**

Illustrates building applications with multiple modes (like `vim`), where different key mappings are active depending on the current mode, including mode-based key mapping switching and dynamic mode transitions.

### [`sequences.rs`](./sequences.rs)
**Key sequences and timing**

Explains how to handle multi-key sequences (like `j j` for double-tap actions), including sequence detection, timing-based handling, and sequence timeout management.

---

## WebAssembly Example

### [`wasm/`](./wasm/)
**Complete WebAssembly game implementation**

A fully functional browser-based game demonstrating keymap-rs in WebAssembly.

**Try it live:** [https://rezigned.com/keymap-rs/](https://rezigned.com/keymap-rs/)


The WASM example requires additional setup:

```bash
cd examples/wasm

# Install trunk for WASM building
cargo install trunk

# Build and serve the WASM example
trunk serve

# Or build for production
trunk build --release
```

Then open your browser to `http://localhost:8080` to play the game.
