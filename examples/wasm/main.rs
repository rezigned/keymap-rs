use keymap::keymap::ToKeyMap; // For event.to_keymap()
use wasm_bindgen::prelude::*;
use web_sys::KeyboardEvent;

// Assuming keymap::Config and keymap::backend::parse are available
// if the keymap crate is correctly featured.
use keymap::Config;
// To get keymap::backend::parse, we need to ensure only one backend feature is active,
// or qualify it if necessary, though typically the top-level `keymap::backend::parse` resolves.
// For the example, we'll assume it resolves correctly due to Cargo.toml settings.
use keymap::backend::parse as parse_key_string;


#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"Keymap WASM Example Loaded. Click page and press keys.".into());

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");
    body.set_tab_index(0); // To allow body to receive focus and key events

    // --- Example Config Setup ---
    // This is a very basic config for demonstration.
    // A real application would load this from elsewhere or have a more complex setup.
    let mut config: Config<String> = Config::default();

    // Example of adding a binding using a parsed string:
    // `parse_key_string` (which is keymap::backend::parse) will return Result<KeyboardEvent, Error>
    // when the "wasm" feature is active.
    match parse_key_string("ctrl-s") {
        Ok(event_to_bind) => {
            // Config::add_binding is not a standard method in the provided snippets.
            // This assumes some way to populate the config.
            // For demonstration, let's imagine `config` has a method `insert(Key, Value, ItemInfo)`.
            // Since `BackendConfig::Key` is `KeyboardEvent` for wasm, we'd insert `event_to_bind`.
            // This part is illustrative of where `parse_key_string` would be used.
            // The actual mechanism to populate `config` depends on `keymap::Config`'s API.
            // For now, we'll just log that we parsed it.
            web_sys::console::log_1(&format!("Successfully parsed 'ctrl-s' into a KeyboardEvent (for binding): key='{}', code='{}'", event_to_bind.key(), event_to_bind.code()).into());
            // In a real scenario: config.add_key_binding(event_to_bind, "Save".to_string(), "Saves the document".to_string());
        }
        Err(e) => {
            web_sys::console::error_1(&format!("Error parsing 'ctrl-s' for binding: {:?}", e).into());
        }
    }
    // Manually adding a key for "Enter" for demonstration with a live event.
    // This is a placeholder for actual config population.
    // If config expects KeyboardEvent, we'd need to create one for "Enter".
    // For simplicity, this example won't have a working config.get() for now,
    // focusing on event capture and parsing.

    // --- Event Listener ---
    let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        web_sys::console::log_1(&format!(
            "JS Event: key='{}', code='{}', ctrl={}, shift={}, alt={}, meta={}",
            event.key(), event.code(), event.ctrl_key(), event.shift_key(), event.alt_key(), event.meta_key()
        ).into());

        // Convert to KeyMap for inspection or other uses if needed:
        match event.to_keymap() {
            Ok(keymap_representation) => {
                web_sys::console::log_1(&format!("Event as KeyMap: {:?}", keymap_representation).into());
            }
            Err(e) => {
                web_sys::console::error_1(&format!("Error converting live event to KeyMap: {:?}", e).into());
            }
        }

        // Using the live event directly with Config:
        // (This part is commented out as the config isn't fully populated for a demo)
        // match config.get(&event) {
        //     Some(action) => {
        //         web_sys::console::log_1(&format!("Action mapped for live event: {}", action).into());
        //     }
        //     None => {
        //         web_sys::console::log_1(&"No action mapped for this live event.".into());
        //     }
        // }

    }) as Box<dyn FnMut(_)>);

    body.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}
