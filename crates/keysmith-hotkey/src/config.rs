// Copyright 2026 Kartios
// SPDX-License-Identifier: MIT

//! What the daemon watches for and what it runs.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// How long a key must be held to count as a long press.
const DEFAULT_LONG_PRESS_MS: u64 = 450;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    /// Platform virtual keycode, as reported by `keysmith-hotkey learn`.
    ///
    /// Not a QMK keycode and not a HID usage. Which virtual keycode a given key
    /// produces depends on how the OS maps it, so it is discovered rather than
    /// assumed.
    pub keycode: i64,

    /// Optional label used in log lines.
    #[serde(default)]
    pub name: Option<String>,

    /// Command and arguments run on a short press.
    #[serde(default)]
    pub on_press: Vec<String>,

    /// Command and arguments run on a long press. Falls back to `on_press`
    /// when empty, so a binding that does not care still behaves sensibly.
    #[serde(default)]
    pub on_long_press: Vec<String>,

    #[serde(default = "default_long_press_ms")]
    pub long_press_ms: u64,

    /// Swallow the key so the focused application never sees it.
    ///
    /// On by default: a key bound to an action should not also type something.
    #[serde(default = "default_true")]
    pub consume: bool,
}

fn default_long_press_ms() -> u64 {
    DEFAULT_LONG_PRESS_MS
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub bindings: Vec<Binding>,
}

impl Config {
    pub fn find(&self, keycode: i64) -> Option<&Binding> {
        self.bindings.iter().find(|b| b.keycode == keycode)
    }
}

/// `$KEYSMITH_HOTKEY_CONFIG`, else `~/.config/keysmith/hotkey.json`.
pub fn config_path() -> PathBuf {
    if let Some(path) = std::env::var_os("KEYSMITH_HOTKEY_CONFIG") {
        return PathBuf::from(path);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join(".config/keysmith/hotkey.json")
}

pub fn load() -> anyhow::Result<Config> {
    let path = config_path();
    let text = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("reading {}: {e}", path.display()))?;
    let config: Config = serde_json::from_str(&text)
        .map_err(|e| anyhow::anyhow!("{} is not valid JSON: {e}", path.display()))?;
    anyhow::ensure!(
        !config.bindings.is_empty(),
        "{} defines no bindings; run `keysmith-hotkey learn` to find a keycode",
        path.display()
    );
    Ok(config)
}
