use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{ConfigurationChange, MutationPlan};

pub const KEYSMITH_PACKET_BYTES: usize = 32;
pub const KEYSMITH_COMMAND: u8 = 0xac;
pub const KEYSMITH_PREPARE_OPERATION: u8 = 0x10;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AttendedOperation {
    Keycode {
        layer: u8,
        row: u8,
        column: u8,
        keycode: u16,
    },
    RgbProfile {
        effect: u8,
        brightness: u8,
        speed: u8,
        hue: u8,
        saturation: u8,
    },
    Encoder {
        layer: u8,
        clockwise: bool,
        keycode: u16,
    },
    WirelessPower {
        backlight_timeout_seconds: u16,
        sleep_timeout_seconds: u16,
    },
    Debounce {
        algorithm_id: u8,
        time_ms: u8,
    },
}

impl AttendedOperation {
    pub fn operation_type(&self) -> u8 {
        match self {
            Self::Keycode { .. } => 1,
            Self::RgbProfile { .. } => 2,
            Self::Encoder { .. } => 3,
            Self::WirelessPower { .. } => 4,
            Self::Debounce { .. } => 5,
        }
    }

    fn write_payload(&self, packet: &mut [u8; KEYSMITH_PACKET_BYTES]) {
        match self {
            Self::Keycode {
                layer,
                row,
                column,
                keycode,
            } => {
                packet[13] = *layer;
                packet[14] = *row;
                packet[15] = *column;
                packet[16..18].copy_from_slice(&keycode.to_be_bytes());
            }
            Self::RgbProfile {
                effect,
                brightness,
                speed,
                hue,
                saturation,
            } => {
                packet[13..18].copy_from_slice(&[*effect, *brightness, *speed, *hue, *saturation]);
            }
            Self::Encoder {
                layer,
                clockwise,
                keycode,
            } => {
                packet[13] = *layer;
                packet[14] = u8::from(*clockwise);
                packet[15..17].copy_from_slice(&keycode.to_be_bytes());
            }
            Self::WirelessPower {
                backlight_timeout_seconds,
                sleep_timeout_seconds,
            } => {
                packet[13..15].copy_from_slice(&backlight_timeout_seconds.to_be_bytes());
                packet[15..17].copy_from_slice(&sleep_timeout_seconds.to_be_bytes());
            }
            Self::Debounce {
                algorithm_id,
                time_ms,
            } => {
                packet[13] = *algorithm_id;
                packet[14] = *time_ms;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedOperation {
    pub index: u8,
    pub total: u8,
    pub operation: AttendedOperation,
    pub prepare_packet_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttendedBundle {
    pub schema: String,
    pub plan_id: String,
    pub plan_tag: String,
    pub firmware_minimum: String,
    pub eligible: bool,
    pub operations: Vec<PreparedOperation>,
    pub blockers: Vec<String>,
    pub requirements: Vec<String>,
}

pub fn compile_attended_bundle(plan: &MutationPlan) -> AttendedBundle {
    let mut operations = Vec::new();
    let mut blockers = Vec::new();
    for change in &plan.diff().changes {
        match change {
            ConfigurationChange::Keycode {
                layer,
                row,
                column,
                to,
                ..
            } => operations.push(AttendedOperation::Keycode {
                layer: *layer,
                row: *row,
                column: *column,
                keycode: *to,
            }),
            ConfigurationChange::Rgb { to, .. } => operations.push(AttendedOperation::RgbProfile {
                effect: to.effect,
                brightness: to.brightness,
                speed: to.speed,
                hue: to.hue,
                saturation: to.saturation,
            }),
            ConfigurationChange::Encoder {
                layer,
                from_counter_clockwise,
                to_counter_clockwise,
                from_clockwise,
                to_clockwise,
            } => {
                if from_counter_clockwise != to_counter_clockwise {
                    operations.push(AttendedOperation::Encoder {
                        layer: *layer,
                        clockwise: false,
                        keycode: *to_counter_clockwise,
                    });
                }
                if from_clockwise != to_clockwise {
                    operations.push(AttendedOperation::Encoder {
                        layer: *layer,
                        clockwise: true,
                        keycode: *to_clockwise,
                    });
                }
            }
            ConfigurationChange::WirelessPower { to, .. } => {
                operations.push(AttendedOperation::WirelessPower {
                    backlight_timeout_seconds: to.backlight_timeout_seconds,
                    sleep_timeout_seconds: to.sleep_timeout_seconds,
                })
            }
            ConfigurationChange::Debounce { to, .. } => {
                operations.push(AttendedOperation::Debounce {
                    algorithm_id: to.algorithm_id,
                    time_ms: to.time_ms,
                })
            }
            ConfigurationChange::DefaultLayer { .. } => blockers
                .push("default-layer mutation is not implemented by the v0.3 candidate".to_owned()),
            ConfigurationChange::MacroUsage { .. } => {
                blockers.push("macro contents and rollback bytes are not captured".to_owned())
            }
            ConfigurationChange::SnapClickUsage { .. } => blockers
                .push("Snap Click pair definitions and rollback bytes are not captured".to_owned()),
        }
    }
    if !plan.rollback().complete_for_diff {
        blockers.extend(plan.rollback().limitations.iter().cloned());
    }
    blockers.sort();
    blockers.dedup();
    let tag_bytes: [u8; 8] = Sha256::digest(plan.plan_id().as_bytes())[..8]
        .try_into()
        .expect("four-byte tag");
    let tag = tag_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let total = u8::try_from(operations.len()).unwrap_or(u8::MAX);
    let prepared = operations
        .into_iter()
        .enumerate()
        .map(|(index, operation)| {
            let mut packet = [0_u8; KEYSMITH_PACKET_BYTES];
            packet[0] = KEYSMITH_COMMAND;
            packet[1] = KEYSMITH_PREPARE_OPERATION;
            packet[2] = operation.operation_type();
            packet[3..11].copy_from_slice(&tag_bytes);
            packet[11] = index as u8;
            packet[12] = total;
            operation.write_payload(&mut packet);
            PreparedOperation {
                index: index as u8,
                total,
                operation,
                prepare_packet_hex: packet.iter().map(|byte| format!("{byte:02x}")).collect(),
            }
        })
        .collect();
    AttendedBundle {
        schema: "keysmith.attended-bundle/v1".to_owned(), plan_id: plan.plan_id().to_owned(), plan_tag: tag,
        firmware_minimum: "0.3.0".to_owned(), eligible: blockers.is_empty() && total > 0, operations: prepared,
        blockers,
        requirements: vec![
            "USB transport and USB power required".to_owned(),
            "one prepared operation per physical confirmation".to_owned(),
            "hold physical Esc + Space + Right Control for 3 seconds".to_owned(),
            "arm expires after 30 seconds and relocks after commit, error, timeout, reset, or disconnect".to_owned(),
            "read back and archive the complete after-state before the next operation".to_owned(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning::tests::snapshot;

    #[test]
    fn keycode_packet_is_deterministic_and_plan_bound() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.configuration.layers[2].matrix[0][15] = 104;
        let plan = MutationPlan::create(baseline, target).unwrap();
        let first = compile_attended_bundle(&plan);
        let second = compile_attended_bundle(&plan);
        assert_eq!(first, second);
        assert!(first.eligible);
        assert_eq!(first.operations.len(), 1);
        assert!(first.operations[0].prepare_packet_hex.starts_with("ac1001"));
        assert_eq!(
            first.operations[0].prepare_packet_hex.len(),
            KEYSMITH_PACKET_BYTES * 2
        );
    }

    #[test]
    fn incomplete_macro_rollback_blocks_bundle() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.configuration.macros.used_bytes = 10;
        let bundle = compile_attended_bundle(&MutationPlan::create(baseline, target).unwrap());
        assert!(!bundle.eligible);
        assert!(bundle.operations.is_empty());
        assert!(bundle.blockers.iter().any(|item| item.contains("macro")));
    }
}
