// Copyright 2026 Kartios
// SPDX-License-Identifier: MIT

//! Named, declarative keyboard states.
//!
//! A scene is a sparse description of how the keyboard should be: set the
//! fields you care about and leave the rest alone. Applying one reads the
//! board, overlays the scene onto what it finds, and produces an ordinary
//! [`MutationPlan`]. Nothing here is a side channel around planning, so a scene
//! gets the same diff, risk assessment and rollback evidence as a hand-built
//! plan, and can be inspected before it touches hardware.
//!
//! Sparseness is the point. A scene that only sets brightness will not quietly
//! reset the hue, and a scene captured from one board will not carry another
//! board's keymap along with it.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::keycodes::Keycode;
use crate::planning::{ConfigurationSnapshot, MutationPlan, PlanError};

pub const SCENE_SCHEMA: &str = "keysmith.scene/v1";

#[derive(Debug, Error)]
pub enum SceneError {
    #[error("scene {0:?} was not found in any scene directory")]
    NotFound(String),
    #[error("scene name {0:?} must be lowercase letters, digits, dashes or underscores")]
    InvalidName(String),
    #[error("scene file {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("scene file {path} is not valid JSON: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("scene {name:?} declares schema {found:?}; this build understands {SCENE_SCHEMA:?}")]
    UnknownSchema { name: String, found: String },
    #[error(transparent)]
    Plan(#[from] PlanError),
}

/// RGB fields a scene may set. Every field is optional.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneRgb {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brightness: Option<u8>,
    /// Changing the effect index is only meaningful on firmware that compiles
    /// more than one effect. See [`Scene::changes_effect`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hue: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saturation: Option<u8>,
}

impl SceneRgb {
    fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneWireless {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backlight_timeout_seconds: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleep_timeout_seconds: Option<u16>,
}

impl SceneWireless {
    fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

/// One encoder direction rebound by a scene.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneEncoder {
    pub layer: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clockwise: Option<Keycode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter_clockwise: Option<Keycode>,
}

/// One key rebound by a scene.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneKey {
    pub layer: u8,
    pub row: u8,
    pub column: u8,
    pub keycode: Keycode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scene {
    #[serde(default = "default_schema")]
    pub schema: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "SceneRgb::is_empty")]
    pub rgb: SceneRgb,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debounce_ms: Option<u8>,
    #[serde(default, skip_serializing_if = "SceneWireless::is_empty")]
    pub wireless: SceneWireless,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub encoders: Vec<SceneEncoder>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<SceneKey>,
}

fn default_schema() -> String {
    SCENE_SCHEMA.to_owned()
}

impl Scene {
    /// True when the scene would move the RGB effect index off what the board
    /// currently runs. Worth surfacing: a firmware that compiles a single effect
    /// has no switch case for any other index and renders nothing at all.
    pub fn changes_effect(&self, baseline: &ConfigurationSnapshot) -> bool {
        self.rgb
            .effect
            .is_some_and(|effect| effect != baseline.configuration.rgb.effect)
    }

    /// Overlay this scene onto a baseline, producing the target state.
    ///
    /// Fields the scene leaves unset are copied from the baseline unchanged, so
    /// the resulting diff contains only what the scene actually asked for.
    pub fn overlay(&self, baseline: &ConfigurationSnapshot) -> ConfigurationSnapshot {
        let mut target = baseline.clone();
        let configuration = &mut target.configuration;

        if let Some(value) = self.rgb.brightness {
            configuration.rgb.brightness = value;
        }
        if let Some(value) = self.rgb.effect {
            configuration.rgb.effect = value;
        }
        if let Some(value) = self.rgb.speed {
            configuration.rgb.speed = value;
        }
        if let Some(value) = self.rgb.hue {
            configuration.rgb.hue = value;
        }
        if let Some(value) = self.rgb.saturation {
            configuration.rgb.saturation = value;
        }

        if let Some(value) = self.debounce_ms {
            configuration.debounce.time_ms = value;
        }
        if let Some(value) = self.wireless.backlight_timeout_seconds {
            configuration.wireless_power.backlight_timeout_seconds = value;
        }
        if let Some(value) = self.wireless.sleep_timeout_seconds {
            configuration.wireless_power.sleep_timeout_seconds = value;
        }

        for wanted in &self.encoders {
            if let Some(binding) = configuration
                .encoders
                .iter_mut()
                .find(|binding| binding.layer == wanted.layer)
            {
                if let Some(keycode) = wanted.clockwise {
                    binding.clockwise = keycode.0;
                }
                if let Some(keycode) = wanted.counter_clockwise {
                    binding.counter_clockwise = keycode.0;
                }
            }
        }

        for wanted in &self.keys {
            let Some(layer) = configuration
                .layers
                .iter_mut()
                .find(|layer| layer.index == wanted.layer)
            else {
                continue;
            };
            // Out-of-range positions are ignored rather than extending the
            // matrix: a scene must not be able to invent keys this board has not
            // got, and the resulting diff would be unappliable anyway.
            if let Some(slot) = layer
                .matrix
                .get_mut(usize::from(wanted.row))
                .and_then(|row| row.get_mut(usize::from(wanted.column)))
            {
                *slot = wanted.keycode.0;
            }
        }

        target
    }

    /// Build the plan this scene represents against a live baseline.
    pub fn plan(&self, baseline: &ConfigurationSnapshot) -> Result<MutationPlan, SceneError> {
        let target = self.overlay(baseline);
        Ok(MutationPlan::create(baseline.clone(), target)?)
    }

    /// Capture the parts of a snapshot a scene can express.
    ///
    /// Deliberately narrow: RGB, debounce, wireless power and encoders only.
    /// A captured scene does not carry a keymap, so restoring it onto a board
    /// with different bindings cannot silently overwrite them.
    pub fn capture(name: &str, description: Option<String>, snapshot: &ConfigurationSnapshot) -> Self {
        let configuration = &snapshot.configuration;
        Self {
            schema: SCENE_SCHEMA.to_owned(),
            name: name.to_owned(),
            description,
            rgb: SceneRgb {
                brightness: Some(configuration.rgb.brightness),
                effect: Some(configuration.rgb.effect),
                speed: Some(configuration.rgb.speed),
                hue: Some(configuration.rgb.hue),
                saturation: Some(configuration.rgb.saturation),
            },
            debounce_ms: Some(configuration.debounce.time_ms),
            wireless: SceneWireless {
                backlight_timeout_seconds: Some(configuration.wireless_power.backlight_timeout_seconds),
                sleep_timeout_seconds: Some(configuration.wireless_power.sleep_timeout_seconds),
            },
            encoders: configuration
                .encoders
                .iter()
                .map(|binding| SceneEncoder {
                    layer: binding.layer,
                    clockwise: Some(Keycode(binding.clockwise)),
                    counter_clockwise: Some(Keycode(binding.counter_clockwise)),
                })
                .collect(),
            keys: Vec::new(),
        }
    }
}

/// A scene name has to be safe to use as a filename.
pub fn validate_name(name: &str) -> Result<(), SceneError> {
    let ok = !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if ok {
        Ok(())
    } else {
        Err(SceneError::InvalidName(name.to_owned()))
    }
}

/// Where scenes live, most specific first.
///
/// `KEYSMITH_SCENE_DIR` wins so a test, an agent sandbox or a second board can
/// be pointed somewhere else without touching the user's own scenes.
pub fn scene_directory() -> PathBuf {
    if let Some(dir) = std::env::var_os("KEYSMITH_SCENE_DIR") {
        return PathBuf::from(dir);
    }
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("keysmith/scenes");
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join(".config/keysmith/scenes")
}

pub fn scene_path(name: &str) -> Result<PathBuf, SceneError> {
    validate_name(name)?;
    Ok(scene_directory().join(format!("{name}.json")))
}

pub fn load(name: &str) -> Result<Scene, SceneError> {
    let path = scene_path(name)?;
    if !path.exists() {
        return Err(SceneError::NotFound(name.to_owned()));
    }
    read_scene(&path)
}

pub fn read_scene(path: &Path) -> Result<Scene, SceneError> {
    let text = std::fs::read_to_string(path).map_err(|source| SceneError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let scene: Scene = serde_json::from_str(&text).map_err(|source| SceneError::Parse {
        path: path.to_path_buf(),
        source,
    })?;
    if scene.schema != SCENE_SCHEMA {
        return Err(SceneError::UnknownSchema {
            name: scene.name.clone(),
            found: scene.schema,
        });
    }
    Ok(scene)
}

pub fn save(scene: &Scene) -> Result<PathBuf, SceneError> {
    let path = scene_path(&scene.name)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| SceneError::Read {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let text = serde_json::to_string_pretty(scene).expect("scene serialises");
    std::fs::write(&path, format!("{text}\n")).map_err(|source| SceneError::Read {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

/// Every readable scene in the scene directory, sorted by name.
///
/// Unreadable files are reported rather than skipped silently, so a typo in one
/// scene does not make it quietly vanish from the list.
pub fn list() -> Vec<Result<Scene, SceneError>> {
    let directory = scene_directory();
    let Ok(entries) = std::fs::read_dir(&directory) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths.iter().map(|path| read_scene(path)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::tests::snapshot;

    #[test]
    fn an_empty_scene_changes_nothing() {
        let baseline = snapshot();
        let scene = Scene {
            schema: SCENE_SCHEMA.to_owned(),
            name: "empty".to_owned(),
            description: None,
            rgb: SceneRgb::default(),
            debounce_ms: None,
            wireless: SceneWireless::default(),
            encoders: Vec::new(),
            keys: Vec::new(),
        };
        assert_eq!(scene.overlay(&baseline), baseline);
    }

    #[test]
    fn a_sparse_scene_leaves_unset_fields_alone() {
        let baseline = snapshot();
        let mut scene = Scene::capture("captured", None, &baseline);
        scene.rgb = SceneRgb {
            brightness: Some(12),
            ..SceneRgb::default()
        };
        let target = scene.overlay(&baseline);
        assert_eq!(target.configuration.rgb.brightness, 12);
        assert_eq!(target.configuration.rgb.hue, baseline.configuration.rgb.hue);
        assert_eq!(
            target.configuration.rgb.saturation,
            baseline.configuration.rgb.saturation
        );
    }

    #[test]
    fn a_captured_scene_round_trips_to_the_same_state() {
        let baseline = snapshot();
        let scene = Scene::capture("restore", None, &baseline);
        assert_eq!(scene.overlay(&baseline), baseline);
    }

    #[test]
    fn capture_never_carries_a_keymap() {
        let baseline = snapshot();
        assert!(Scene::capture("k", None, &baseline).keys.is_empty());
    }

    #[test]
    fn names_must_be_filename_safe() {
        assert!(validate_name("focus-dim").is_ok());
        assert!(validate_name("../escape").is_err());
        assert!(validate_name("Focus").is_err());
        assert!(validate_name("").is_err());
    }
}
