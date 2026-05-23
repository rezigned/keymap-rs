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
    fn setKey(key: String, desc: String, symbol: String, help: String);
    fn setSkin(digit: u8);
    fn renderKeybindings(info: String);
}

#[derive(Debug, Clone, keymap::KeyMap, Hash, PartialEq, Eq)]
pub enum Action {
    /// Jump over obstacles
    #[key("space", symbol = "↑", help = "jump")] // symbol gets overridden by toml config
    Jump,

    /// Move leftward
    #[key("left", help = "move left")]
    Left,

    /// Move rightward
    #[key("right", help = "move right")]
    Right,

    /// Pause
    #[key("p", help = "pause")]
    Pause,

    /// Restart
    #[key("q", "esc", help = "quit")]
    Quit,

    /// Select a skin
    #[key("@digit", symbol = "0-9", help = "select skin")]
    SelectSkin(u8),
}

/// Overrides the default keymap
#[allow(unused)]
pub const DERIVED_CONFIG: &str = r#"
Jump = { keys = ["space", "up"], symbol = "␣", help = "jump", description = "Jump Jump!" }
Quit = { keys = ["q", "esc"], symbol = "↩", help = "quit", description = "Quit!" }
Left = { keys = ["left", "alt-l"], symbol = "←", help = "move left", description = "Move Left" }
Right = { keys = ["right", "alt-r"], symbol = "→", help = "move right", description = "Move Right" }
SelectSkin = { keys = ["@digit"], symbol = "0-9", help = "select skin", description = "Select a skin" }
"#;

#[allow(unused)]
pub fn derived_config() -> DerivedConfig<Action> {
    toml::from_str(DERIVED_CONFIG).unwrap()
}

fn json_escape(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => buf.push_str("\\\""),
            '\\' => buf.push_str("\\\\"),
            '\n' => buf.push_str("\\n"),
            '\r' => buf.push_str("\\r"),
            '\t' => buf.push_str("\\t"),
            c if c.is_control() => buf.push_str(&format!("\\u{:04x}", c as u32)),
            c => buf.push(c),
        }
    }
    buf
}

#[wasm_bindgen]
pub fn get_keymap_as_json() -> String {
    let keymap = derived_config();
    let keymap_info: Vec<String> = keymap
        .items
        .iter()
        .map(|(action, entry)| {
            let keys: Vec<String> = entry.keys.iter().map(|k| format!("\"{}\"", json_escape(k))).collect();
            let description = json_escape(&entry.description);
            let symbol = json_escape(entry.symbol.as_deref().unwrap_or_default());
            let help = json_escape(entry.help.as_deref().unwrap_or_default());
            let action_str = json_escape(&format!("{:?}", action));
            format!(
                "{{ \"action\": \"{action_str}\", \"keys\": [{}], \"description\": \"{description}\", \"symbol\": \"{symbol}\", \"help\": \"{help}\" }}",
                keys.join(","),
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

    renderKeybindings(get_keymap_as_json());
    resetGame();
}

pub fn handle_key_event(event: &KeyboardEvent, is_keydown: bool) {
    let config = derived_config();
    let is_game_over = isGameOver();

    // Log the key that was pressed
    if is_keydown {
        let key = event.to_keymap().unwrap();
        let mut desc = String::new();
        let mut symbol = String::new();
        let mut help = String::new();
        if let Some((_, item)) = config.get_item(event) {
            desc = item.description.clone();
            if item.keys.iter().any(|k| k.starts_with('@')) {
                symbol = key.to_string();
            } else {
                symbol = item.symbol.clone().unwrap_or_default();
            }
            help = item.help.clone().unwrap_or_default();
        };

        setKey(key.to_string(), desc, symbol, help);
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
