# keymap_derive

This crate provides a derive macro `#[derive(KeyMap)]` for Rust enums to easily generate implementations for:

*   `TryFrom<KeyMap>` and `TryFrom<Vec<KeyMap>>`: Allowing conversion from parsed key events to your enum variants.
*   `KeyMapConfig<YourEnum>`: Providing the derived keybindings as default configurations.

This is part of the [keymap-rs](../../README.md) workspace.

## Usage

Add `keymap` with the `derive` feature to your `Cargo.toml`:

```toml
[dependencies]
keymap = { version = "0.5", features = ["derive"] } # Check for the latest version
```

Then, use the `KeyMap` derive macro on your enum:

```rust
use keymap::KeyMap; // For TryFrom<KeyMap> and parse_keymap
use keymap_derive::KeyMap; // For the derive macro itself

#[derive(Debug, PartialEq, KeyMap)]
enum Action {
    #[key("c", description = "Create an item")]
    Create,
    #[key("d", "del", description = "Delete an item")]
    Delete,
    #[key("q", "ctrl-c", description = "Quit the application")]
    Quit,
}

fn main() {
    // Example: Parsing a key string and converting to Action
    // In a real application, KeyMap would come from an event loop or config.
    let keymap_create = keymap::parse_keymap("c").unwrap();
    let action_create = Action::try_from(keymap_create).unwrap();
    assert_eq!(action_create, Action::Create);

    let keymap_delete_del = keymap::parse_keymap("del").unwrap();
    let action_delete_del = Action::try_from(keymap_delete_del).unwrap();
    assert_eq!(action_delete_del, Action::Delete);

    println!("Successfully derived and used KeyMap!");

    // The derive also generates KeyMapConfig<Action>
    // For more details on how this is used with DerivedConfig
    // to allow user overrides, please see the main [keymap-rs documentation](../../README.md#advanced-customizing-keymaps-with-derivedconfig).
}
```

## Attributes

*   `#[key("key1", "key2", ..., description = "Human-readable description")]`:
    *   Specifies one or more key sequences that map to this enum variant.
    *   The `description` is optional but recommended, especially when using `DerivedConfig` as it will be part of the default `Item` generated.

For more advanced usage, including how these derived defaults can be customized by users with `DerivedConfig`, please see the [main `keymap-rs` README](../../README.md).
