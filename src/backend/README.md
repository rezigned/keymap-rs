# Key Mapping Reference

This table maps the `Key` enum variants (in lowercase form) to their equivalents in various Rust libraries and web environments.

| `Key` Enum (lowercase) | `crossterm::event::KeyCode` | `termion::event::Key` | `termwiz::input::KeyCode` | `web_sys::KeyboardEvent.key` |
|------------------------|-----------------------------|------------------------|----------------------------|------------------------------|
| `backtab`              | `BackTab`                   | `BackTab`              | `BackTab`                  | `"Tab"` + `Shift`           |
| `backspace`            | `Backspace`                 | `Backspace`            | `Backspace`                | `"Backspace"`               |
| `delete`               | `Delete`                    | `Delete`               | `Delete`                   | `"Delete"`                  |
| `down`                 | `Down`                      | `Down`                 | `DownArrow`                | `"ArrowDown"`               |
| `end`                  | `End`                       | `End`                  | `End`                      | `"End"`                     |
| `enter`                | `Enter`                     | `Char('\n')` or `Char('\r')` | `Enter`             | `"Enter"`                   |
| `esc`                  | `Esc`                       | `Esc`                  | `Escape`                   | `"Escape"`                  |
| `home`                 | `Home`                      | `Home`                 | `Home`                     | `"Home"`                    |
| `insert`               | `Insert`                    | `Insert`               | `Insert`                   | `"Insert"`                  |
| `left`                 | `Left`                      | `Left`                 | `LeftArrow`                | `"ArrowLeft"`               |
| `pagedown`             | `PageDown`                  | `PageDown`             | `PageDown`                 | `"PageDown"`                |
| `pageup`               | `PageUp`                    | `PageUp`               | `PageUp`                   | `"PageUp"`                  |
| `right`                | `Right`                     | `Right`                | `RightArrow`               | `"ArrowRight"`              |
| `space`                | `Char(' ')`                 | `Char(' ')`            | `Char(' ')`                | `" "`                       |
| `tab`                  | `Tab`                       | `Char('\t')`           | `Tab`                      | `"Tab"`                     |
| `up`                   | `Up`                        | `Up`                   | `UpArrow`                  | `"ArrowUp"`                 |

### Additional Notes:

- **Dynamic variants**: `char(_)` and `f(_)` are dynamic and map as-is, e.g. `char('a')`, `f(1)` → `Char('a')`, `F(1)`.
- **Internal variants**: `group(_)` is internal to the parser and not applicable across these systems.
- **Termion specifics**: termion has additional modifier variants like `ShiftLeft`, `AltLeft`, `CtrlLeft`, etc. for modified arrow keys
- **Web API**: `web_sys::KeyboardEvent.key` returns a string; compare directly to literal key names.
- **Crossterm compatibility**: crossterm has expanded significantly and now includes variants like `CapsLock`, `ScrollLock`, `NumLock`, `PrintScreen`, `Pause`, `Menu`, `KeypadBegin`, `Media(MediaKeyCode)`, and `Modifier(ModifierKeyCode)`

### Termion-specific Behavior:
* **`termion` `tab` key**: termion uses `BackTab` for backward tab, but regular tab is handled as `Char('\t')`
* **`termion` `enter` key**: termion doesn't have a dedicated `Enter` variant - it uses `Char('\n')` or `Char('\r')`
- termion represents Tab as `Char('\t')` rather than having a dedicated Tab variant
- termion represents Enter/Return as `Char('\n')` or `Char('\r')` depending on the terminal
- termion has extensive modifier support with separate variants for Shift, Alt, and Ctrl combinations with arrow keys
