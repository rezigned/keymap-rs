# Examples

This directory contains examples demonstrating various features and use cases of the `keymap-rs` library. Each example showcases different aspects of key mapping, from basic usage to advanced configurations.

> [!NOTE]
>
> All of the examples below work on any backend by simply passing `{backend}` to the `--features` argument.
>
> ```
> cargo run --example {example} --features {backend}
> ```

### [`simple.rs`](./simple.rs)
**Basic key mapping without derive macros**

Demonstrates how to use the library without the `#[derive(KeyMap)]` macro, including manual TOML configuration parsing and basic action handling.


### [`derive.rs`](./derive.rs)
**Using the KeyMap derive macro**

Demonstrates the most common and recommended approach using the `#[derive(KeyMap)]` macro, including automatic keymap generation from enum attributes and clean, declarative key mapping.


### [`config.rs`](./config.rs)
**External configuration with Config<T>**

Demonstrates loading key mappings exclusively from external configuration files, ignoring derive macro definitions, and shows file-based key overrides and custom key descriptions.

### [`derived_config.rs`](./derived_config.rs)
**Merging derive macros with external config using DerivedConfig<T>**

Demonstrates how to combine derive macro defaults with external configuration overrides, including configuration precedence and key group patterns like `@digit`.


### [`modes.rs`](./modes.rs)
**Multi-mode application with different key mappings**

Demonstrates building applications with multiple modes (like vim), where different key mappings are active depending on the current mode, including mode-based key mapping switching and dynamic mode transitions.

### [`sequences.rs`](./sequences.rs)
**Key sequences and timing**

Demonstrates how to handle multi-key sequences (like `j j` for double-tap actions), including sequence detection, timing-based handling, and sequence timeout management.

---

## WebAssembly Example

### [`wasm/`](./wasm/)
**Complete WebAssembly game implementation**

A fully functional browser-based game demonstrating keymap-rs in WebAssembly:

**Game Controls:**
- `Space`/`Up` - Jump
- `Left`/`Alt+L` - Move left
- `Right`/`Alt+R` - Move right
- `P` - Pause
- `Q`/`Esc` - Restart

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
