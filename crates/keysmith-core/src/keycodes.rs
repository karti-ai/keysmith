// Copyright 2026 Kartios
// SPDX-License-Identifier: MIT

//! Keycodes as names rather than integers.
//!
//! A snapshot straight off the keyboard reports an encoder bound to `169`, which
//! tells a reader nothing. The same value written as `KC_VOLU` is self
//! explanatory, and a scene file authored by hand or by an agent can name a key
//! instead of looking up a number in QMK's headers.
//!
//! The table in [`keycodes_generated`] is produced from QMK's own
//! `quantum/keycodes.h` by `tools/generate-keycodes.py`, so it cannot drift from
//! the firmware, and it is committed so an ordinary build needs no QMK checkout.

use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::keycodes_generated::{BY_NAME, BY_VALUE};

/// A QMK keycode.
///
/// Serialises as its name when one is known and as a plain number otherwise, so
/// a value this build does not recognise still round-trips exactly rather than
/// being lost or rejected. Deserialises from either form for the same reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Keycode(pub u16);

impl Keycode {
    /// The preferred display name, if this build knows one.
    ///
    /// Where QMK defines several names for a value, the shortest `KC_` spelling
    /// wins, so this returns `KC_VOLU` rather than `KC_AUDIO_VOL_UP`.
    pub fn name(self) -> Option<&'static str> {
        BY_VALUE
            .binary_search_by(|(value, _)| value.cmp(&self.0))
            .ok()
            .map(|index| BY_VALUE[index].1)
    }

    /// Parse a name, or a `0x`-prefixed or plain decimal number.
    ///
    /// Names are matched case-insensitively and a bare `A` is accepted for
    /// `KC_A`, because writing `--keycode a` is what people try first.
    pub fn parse(text: &str) -> Result<Self, KeycodeError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(KeycodeError::Unknown(text.to_owned()));
        }

        if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
            return u16::from_str_radix(hex, 16)
                .map(Keycode)
                .map_err(|_| KeycodeError::Unknown(text.to_owned()));
        }
        if trimmed.chars().all(|c| c.is_ascii_digit()) {
            return trimmed
                .parse::<u16>()
                .map(Keycode)
                .map_err(|_| KeycodeError::Unknown(text.to_owned()));
        }

        let upper = trimmed.to_ascii_uppercase();
        for candidate in [upper.as_str(), &format!("KC_{upper}")] {
            if let Ok(index) = BY_NAME.binary_search_by(|(name, _)| (*name).cmp(candidate)) {
                return Ok(Keycode(BY_NAME[index].1));
            }
        }
        Err(KeycodeError::Unknown(text.to_owned()))
    }
}

impl fmt::Display for Keycode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name() {
            Some(name) => f.write_str(name),
            // Unknown values print as hex rather than decimal: they are almost
            // always a keycode range this build predates, and hex makes the
            // range obvious.
            None => write!(f, "0x{:04x}", self.0),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeycodeError {
    #[error("{0:?} is not a known keycode name or number")]
    Unknown(String),
}

impl Serialize for Keycode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.name() {
            Some(name) => serializer.serialize_str(name),
            None => serializer.serialize_u16(self.0),
        }
    }
}

impl<'de> Deserialize<'de> for Keycode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeycodeVisitor;

        impl<'de> Visitor<'de> for KeycodeVisitor {
            type Value = Keycode;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a keycode name such as \"KC_VOLU\", or a number")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Keycode, E> {
                Keycode::parse(value).map_err(E::custom)
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Keycode, E> {
                u16::try_from(value)
                    .map(Keycode)
                    .map_err(|_| E::custom(format!("{value} does not fit in a keycode")))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Keycode, E> {
                u16::try_from(value)
                    .map(Keycode)
                    .map_err(|_| E::custom(format!("{value} does not fit in a keycode")))
            }
        }

        deserializer.deserialize_any(KeycodeVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values_resolve_to_their_short_name() {
        assert_eq!(Keycode(0x00a9).name(), Some("KC_VOLU"));
        assert_eq!(Keycode(0x7827).name(), Some("UG_VALU"));
        assert_eq!(Keycode(0x006d).name(), Some("KC_F18"));
    }

    #[test]
    fn parsing_accepts_names_numbers_and_bare_letters() {
        assert_eq!(Keycode::parse("KC_VOLU").unwrap(), Keycode(0x00a9));
        assert_eq!(Keycode::parse("kc_volu").unwrap(), Keycode(0x00a9));
        assert_eq!(Keycode::parse("a").unwrap(), Keycode::parse("KC_A").unwrap());
        assert_eq!(Keycode::parse("0x00a9").unwrap(), Keycode(0x00a9));
        assert_eq!(Keycode::parse("169").unwrap(), Keycode(0x00a9));
        assert!(Keycode::parse("not_a_key").is_err());
        assert!(Keycode::parse("").is_err());
    }

    #[test]
    fn unknown_values_survive_a_round_trip() {
        // A value from a keycode range this build predates must not be lost.
        let unknown = Keycode(0xfffe);
        assert_eq!(unknown.name(), None);
        let json = serde_json::to_string(&unknown).unwrap();
        assert_eq!(json, "65534");
        assert_eq!(serde_json::from_str::<Keycode>(&json).unwrap(), unknown);
    }

    #[test]
    fn named_values_serialise_as_names_and_read_back() {
        let json = serde_json::to_string(&Keycode(0x00a9)).unwrap();
        assert_eq!(json, "\"KC_VOLU\"");
        assert_eq!(serde_json::from_str::<Keycode>(&json).unwrap(), Keycode(0x00a9));
    }

    #[test]
    fn old_scene_files_with_plain_numbers_still_load() {
        assert_eq!(serde_json::from_str::<Keycode>("169").unwrap(), Keycode(0x00a9));
    }

    #[test]
    fn the_generated_tables_are_sorted_for_binary_search() {
        assert!(BY_NAME.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(BY_VALUE.windows(2).all(|w| w[0].0 < w[1].0));
    }
}
