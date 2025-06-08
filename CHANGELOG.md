# Changelog

## [1.0.0](https://github.com/rezigned/keymap-rs/compare/v0.4.1...v1.0.0) (2025-06-08)

### ⚠ BREAKING CHANGES

* v1.0.0 release 🎉 ([#17](https://github.com/rezigned/keymap-rs/issues/17))

### Features

* v1.0.0 release 🎉 ([#17](https://github.com/rezigned/keymap-rs/issues/17)) ([8da2b52](https://github.com/rezigned/keymap-rs/commit/8da2b525ac0a628583bc8945a3eb74cd8a6c90dd))
* **Declarative key mapping** via config files (`yaml`, `json`, `toml`, etc.)
* **Support for key sequences** like `ctrl-b n`, `g g`, etc.
* **Pattern-matching key groups** (`@upper`, `@any`, etc.)
* **Derive-based config parsing** via `keymap_derive`
* **Backend-agnostic** design with support for `crossterm`, `termion`, `web_sys`, etc.
* `Config<T>`: loads keys exclusively from config files
* `DerivedConfig<T>`: merges config and `#[key("..")]` annotations

## [0.4.1](https://github.com/rezigned/keymap-rs/compare/v0.4.0...v0.4.1) (2024-11-24)

### Bug Fixes

* update dependencies ([#12](https://github.com/rezigned/keymap-rs/issues/12)) ([2d0bfda](https://github.com/rezigned/keymap-rs/commit/2d0bfda90e3eff9c2b89079f1096f08de666b600))

## [0.4.0](https://github.com/rezigned/keymap-rs/compare/v0.3.0...v0.4.0) (2023-10-11)

### Features

* Convert from Backend::Key to Node ([#10](https://github.com/rezigned/keymap-rs/issues/10)) ([2b46c7f](https://github.com/rezigned/keymap-rs/commit/2b46c7fe0fa4ec0f23555642c3f1464544cf59d4))

## [0.3.0](https://github.com/rezigned/keymap-rs/compare/v0.2.0...v0.3.0) (2023-09-24)

### Features

* Expose 'parse' function ([#7](https://github.com/rezigned/keymap-rs/issues/7)) ([3180c28](https://github.com/rezigned/keymap-rs/commit/3180c28992f30de63a48b1d2647f99637e4d020d))

## [0.2.0](https://github.com/rezigned/keymap-rs/compare/v0.1.0...v0.2.0) (2023-08-01)

### Features

* use char instead of u8 for parser's input to improve error message ([e1c1eb1](https://github.com/rezigned/keymap-rs/commit/e1c1eb1227443e86dbb3a806aee868ea14e9fe45))
