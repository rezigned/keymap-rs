use keymap::{DerivedConfig, ToKeyMap};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, KeyboardEvent};

#[wasm_bindgen(module = "/game.js")]
extern "C" {
    fn jump();
    fn moveLeft(is_moving: bool);
    fn moveRight(is_moving: bool);
    fn isGameOver() -> bool;
    fn resetGame();
    fn pauseGame();
    fn setKey(key: String, desc: String);
    fn setSkin(c: char);
}

#[derive(Debug, Clone, keymap::KeyMap, Hash, PartialEq, Eq)]
pub enum Action {
    /// Jump over obstacles
    #[key("space")]
    Jump,

    /// Move leftward
    #[key("left")]
    Left,

    /// Move rightward
    #[key("right")]
    Right,

    /// Pause
    #[key("p")]
    Pause,

    /// Restart
    #[key("q", "esc")]
    Quit,

    /// Select a skin
    #[key("@digit")]
    SelectSkin(char),
}

/// Overrides the default keymap
#[allow(unused)]
pub const DERIVED_CONFIG: &str = r#"
Jump = { keys = ["space", "up"], description = "Jump Jump!" }
Quit = { keys = ["q", "esc"], description = "Quit!" }
Left = { keys = ["left", "alt-l"], description = "Move Left" }
Right = { keys = ["right", "alt-r"], description = "Move Right" }
SelectSkin = { keys = ["@digit"], description = "Select a skin" }
"#;

#[allow(unused)]
pub fn derived_config() -> DerivedConfig<Action> {
    toml::from_str(DERIVED_CONFIG).unwrap()
}

#[wasm_bindgen]
pub fn get_keymap_as_json() -> String {
    let keymap = derived_config();
    let keymap_info: Vec<String> = keymap
        .items
        .iter()
        .map(|(action, entry)| {
            let keys: Vec<String> = entry.keys.iter().map(|k| format!("\"{}\"", k)).collect();
            let description = entry.description.clone();
            format!(
                "{{ \"action\": \"{:?}\", \"keys\": [{}], \"description\": \"{}\" }}",
                action,
                keys.join(","),
                description
            )
        })
        .collect();

    format!("[{}]", keymap_info.join(","))
}

pub fn main() {
    let window = window().unwrap();
    let document = window.document().unwrap();

    let on_keydown = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        event.prevent_default();
        handle_key_event(&event, true);
    }) as Box<dyn FnMut(_)>);

    let on_keyup = Closure::wrap(Box::new(move |event: KeyboardEvent| {
        event.prevent_default();
        handle_key_event(&event, false);
    }) as Box<dyn FnMut(_)>);

    document.set_onkeydown(Some(on_keydown.as_ref().unchecked_ref()));
    document.set_onkeyup(Some(on_keyup.as_ref().unchecked_ref()));

    on_keydown.forget();
    on_keyup.forget();

    resetGame();
}

pub fn handle_key_event(event: &KeyboardEvent, is_keydown: bool) {
    let config = derived_config();
    let is_game_over = isGameOver();

    // Log the key that was pressed
    if is_keydown {
        let key = event.to_keymap().unwrap();
        let mut desc = String::new();
        if let Some((_, item)) = config.get_item(event) {
            desc = item.description.clone();
        };

        setKey(key.to_string(), desc);
    }

    // Use .get_bound() to support Key Group Capturing for SelectSkin
    if let Some(action) = config.get_bound(event) {
        match action {
            Action::Quit => {
                if is_keydown {
                    resetGame();
                }
            }
            Action::SelectSkin(c) => {
                if is_keydown {
                    setSkin(c);
                    setKey(format!("Skin selected: {c}"), "Key Group Capturing!".to_string());
                }
            }
            _ if !is_game_over => match action {
                Action::Left => moveLeft(is_keydown),
                Action::Right => moveRight(is_keydown),
                Action::Jump if is_keydown => jump(),
                Action::Pause if is_keydown => pauseGame(),
                _ => {}
            },
            _ => {}
        }
    }
}
