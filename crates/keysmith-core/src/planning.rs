use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{DeviceIdentity, Inspection, KEYCHRON_VENDOR_ID, Q3_MAX_ANSI_PRODUCT_ID};

pub const CONFIG_SNAPSHOT_SCHEMA_VERSION: u16 = 1;
pub const PLAN_SCHEMA_VERSION: u16 = 1;

const Q3_MAX_LAYERS: usize = 4;
const Q3_MAX_ROWS: usize = 6;
const Q3_MAX_COLUMNS: usize = 17;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotDevice {
    pub name: String,
    pub layout: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub firmware: String,
    pub via_protocol: u16,
    pub keychron_protocol: u8,
    pub qmk_command_set: u8,
}

impl From<&DeviceIdentity> for SnapshotDevice {
    fn from(identity: &DeviceIdentity) -> Self {
        Self {
            name: identity.name.to_owned(),
            layout: identity.layout.to_owned(),
            vendor_id: identity.vendor_id,
            product_id: identity.product_id,
            firmware: identity.firmware.clone(),
            via_protocol: identity.via_protocol,
            keychron_protocol: identity.keychron_protocol,
            qmk_command_set: identity.qmk_command_set,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotKeymapLayer {
    pub index: u8,
    pub name: String,
    pub matrix: Vec<Vec<u16>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotMacroInfo {
    pub slots: u8,
    pub buffer_bytes: u16,
    pub used_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotSnapClickInfo {
    pub pair_capacity: u8,
    pub configured_pairs: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotWirelessPower {
    pub backlight_timeout_seconds: u16,
    pub sleep_timeout_seconds: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotDebounceInfo {
    pub algorithm_id: u8,
    pub algorithm: String,
    pub time_ms: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotRgbInfo {
    pub brightness: u8,
    pub effect: u8,
    pub speed: u8,
    pub hue: u8,
    pub saturation: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotEncoderBinding {
    pub layer: u8,
    pub counter_clockwise: u16,
    pub clockwise: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardConfiguration {
    pub active_default_layer: u8,
    pub layers: Vec<SnapshotKeymapLayer>,
    pub macros: SnapshotMacroInfo,
    pub snap_click: SnapshotSnapClickInfo,
    pub wireless_power: SnapshotWirelessPower,
    pub debounce: SnapshotDebounceInfo,
    pub rgb: SnapshotRgbInfo,
    pub encoders: Vec<SnapshotEncoderBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationSnapshot {
    pub schema_version: u16,
    pub device: SnapshotDevice,
    pub configuration: KeyboardConfiguration,
}

impl ConfigurationSnapshot {
    pub fn from_inspection(inspection: &Inspection) -> Self {
        Self {
            schema_version: CONFIG_SNAPSHOT_SCHEMA_VERSION,
            device: SnapshotDevice::from(&inspection.identity),
            configuration: KeyboardConfiguration {
                active_default_layer: inspection.active_default_layer,
                layers: inspection
                    .layers
                    .iter()
                    .map(|layer| SnapshotKeymapLayer {
                        index: layer.index,
                        name: layer.name.to_owned(),
                        matrix: layer.matrix.clone(),
                    })
                    .collect(),
                macros: SnapshotMacroInfo {
                    slots: inspection.macros.slots,
                    buffer_bytes: inspection.macros.buffer_bytes,
                    used_bytes: inspection.macros.used_bytes,
                },
                snap_click: SnapshotSnapClickInfo {
                    pair_capacity: inspection.snap_click.pair_capacity,
                    configured_pairs: inspection.snap_click.configured_pairs,
                },
                wireless_power: SnapshotWirelessPower {
                    backlight_timeout_seconds: inspection.wireless_power.backlight_timeout_seconds,
                    sleep_timeout_seconds: inspection.wireless_power.sleep_timeout_seconds,
                },
                debounce: SnapshotDebounceInfo {
                    algorithm_id: inspection.debounce.algorithm_id,
                    algorithm: inspection.debounce.algorithm.to_owned(),
                    time_ms: inspection.debounce.time_ms,
                },
                rgb: SnapshotRgbInfo {
                    brightness: inspection.rgb.brightness,
                    effect: inspection.rgb.effect,
                    speed: inspection.rgb.speed,
                    hue: inspection.rgb.hue,
                    saturation: inspection.rgb.saturation,
                },
                encoders: inspection
                    .encoders
                    .iter()
                    .map(|encoder| SnapshotEncoderBinding {
                        layer: encoder.layer,
                        counter_clockwise: encoder.counter_clockwise,
                        clockwise: encoder.clockwise,
                    })
                    .collect(),
            },
        }
    }

    pub fn id(&self) -> Result<String, PlanError> {
        self.validate()?;
        Ok(format!("kssnap_v1_{}", canonical_hash(self)?))
    }

    pub fn validate(&self) -> Result<(), PlanError> {
        if self.schema_version != CONFIG_SNAPSHOT_SCHEMA_VERSION {
            return Err(PlanError::UnsupportedSnapshotSchema(self.schema_version));
        }
        if self.device.vendor_id != KEYCHRON_VENDOR_ID
            || self.device.product_id != Q3_MAX_ANSI_PRODUCT_ID
        {
            return Err(PlanError::UnsupportedDevice {
                vendor_id: self.device.vendor_id,
                product_id: self.device.product_id,
            });
        }
        if self.device.name.is_empty()
            || self.device.layout.is_empty()
            || self.device.firmware.is_empty()
        {
            return Err(PlanError::InvalidSnapshot(
                "device name, layout, and firmware must not be empty".to_owned(),
            ));
        }
        if self.configuration.layers.len() != Q3_MAX_LAYERS {
            return Err(PlanError::InvalidSnapshot(format!(
                "expected {Q3_MAX_LAYERS} layers, found {}",
                self.configuration.layers.len()
            )));
        }
        if self.configuration.active_default_layer as usize >= Q3_MAX_LAYERS {
            return Err(PlanError::InvalidSnapshot(format!(
                "default layer {} is outside the Q3 Max layer range",
                self.configuration.active_default_layer
            )));
        }
        for (expected_index, layer) in self.configuration.layers.iter().enumerate() {
            if layer.index as usize != expected_index {
                return Err(PlanError::InvalidSnapshot(format!(
                    "layers must be ordered by index; expected {expected_index}, found {}",
                    layer.index
                )));
            }
            if layer.matrix.len() != Q3_MAX_ROWS
                || layer.matrix.iter().any(|row| row.len() != Q3_MAX_COLUMNS)
            {
                return Err(PlanError::InvalidSnapshot(format!(
                    "layer {expected_index} must have a {Q3_MAX_ROWS}x{Q3_MAX_COLUMNS} matrix"
                )));
            }
        }
        if self.configuration.encoders.len() != Q3_MAX_LAYERS {
            return Err(PlanError::InvalidSnapshot(format!(
                "expected {Q3_MAX_LAYERS} encoder bindings, found {}",
                self.configuration.encoders.len()
            )));
        }
        for (expected_layer, encoder) in self.configuration.encoders.iter().enumerate() {
            if encoder.layer as usize != expected_layer {
                return Err(PlanError::InvalidSnapshot(format!(
                    "encoder bindings must be ordered by layer; expected {expected_layer}, found {}",
                    encoder.layer
                )));
            }
        }
        if self.configuration.macros.used_bytes > self.configuration.macros.buffer_bytes as usize {
            return Err(PlanError::InvalidSnapshot(
                "macro usage exceeds the reported buffer size".to_owned(),
            ));
        }
        if self.configuration.snap_click.configured_pairs
            > self.configuration.snap_click.pair_capacity
        {
            return Err(PlanError::InvalidSnapshot(
                "configured Snap Click pairs exceed capacity".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConfigurationChange {
    DefaultLayer {
        from: u8,
        to: u8,
    },
    Keycode {
        layer: u8,
        row: u8,
        column: u8,
        from: u16,
        to: u16,
    },
    MacroUsage {
        from: SnapshotMacroInfo,
        to: SnapshotMacroInfo,
    },
    SnapClickUsage {
        from: SnapshotSnapClickInfo,
        to: SnapshotSnapClickInfo,
    },
    WirelessPower {
        from: SnapshotWirelessPower,
        to: SnapshotWirelessPower,
    },
    Debounce {
        from: SnapshotDebounceInfo,
        to: SnapshotDebounceInfo,
    },
    Rgb {
        from: SnapshotRgbInfo,
        to: SnapshotRgbInfo,
    },
    Encoder {
        layer: u8,
        from_counter_clockwise: u16,
        to_counter_clockwise: u16,
        from_clockwise: u16,
        to_clockwise: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationDiff {
    pub baseline_snapshot_id: String,
    pub target_snapshot_id: String,
    pub changes: Vec<ConfigurationChange>,
}

impl ConfigurationDiff {
    pub fn between(
        baseline: &ConfigurationSnapshot,
        target: &ConfigurationSnapshot,
    ) -> Result<Self, PlanError> {
        validate_pair(baseline, target)?;
        let before = &baseline.configuration;
        let after = &target.configuration;
        let mut changes = Vec::new();

        if before.active_default_layer != after.active_default_layer {
            changes.push(ConfigurationChange::DefaultLayer {
                from: before.active_default_layer,
                to: after.active_default_layer,
            });
        }
        for (before_layer, after_layer) in before.layers.iter().zip(&after.layers) {
            for (row_index, (before_row, after_row)) in before_layer
                .matrix
                .iter()
                .zip(&after_layer.matrix)
                .enumerate()
            {
                for (column_index, (&from, &to)) in before_row.iter().zip(after_row).enumerate() {
                    if from != to {
                        changes.push(ConfigurationChange::Keycode {
                            layer: before_layer.index,
                            row: row_index as u8,
                            column: column_index as u8,
                            from,
                            to,
                        });
                    }
                }
            }
        }
        if before.macros != after.macros {
            changes.push(ConfigurationChange::MacroUsage {
                from: before.macros.clone(),
                to: after.macros.clone(),
            });
        }
        if before.snap_click != after.snap_click {
            changes.push(ConfigurationChange::SnapClickUsage {
                from: before.snap_click.clone(),
                to: after.snap_click.clone(),
            });
        }
        if before.wireless_power != after.wireless_power {
            changes.push(ConfigurationChange::WirelessPower {
                from: before.wireless_power.clone(),
                to: after.wireless_power.clone(),
            });
        }
        if before.debounce != after.debounce {
            changes.push(ConfigurationChange::Debounce {
                from: before.debounce.clone(),
                to: after.debounce.clone(),
            });
        }
        if before.rgb != after.rgb {
            changes.push(ConfigurationChange::Rgb {
                from: before.rgb.clone(),
                to: after.rgb.clone(),
            });
        }
        for (before_encoder, after_encoder) in before.encoders.iter().zip(&after.encoders) {
            if before_encoder != after_encoder {
                changes.push(ConfigurationChange::Encoder {
                    layer: before_encoder.layer,
                    from_counter_clockwise: before_encoder.counter_clockwise,
                    to_counter_clockwise: after_encoder.counter_clockwise,
                    from_clockwise: before_encoder.clockwise,
                    to_clockwise: after_encoder.clockwise,
                });
            }
        }

        Ok(Self {
            baseline_snapshot_id: baseline.id()?,
            target_snapshot_id: target.id()?,
            changes,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    None,
    Low,
    Moderate,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub level: RiskLevel,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmationMetadata {
    pub required: bool,
    pub attended: bool,
    pub scope: String,
    pub exact_phrase: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollbackEvidence {
    pub baseline_snapshot_id: String,
    pub embedded_baseline: ConfigurationSnapshot,
    pub complete_for_diff: bool,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutationPlan {
    schema_version: u16,
    plan_id: String,
    baseline: ConfigurationSnapshot,
    target: ConfigurationSnapshot,
    diff: ConfigurationDiff,
    risk: RiskAssessment,
    confirmation: ConfirmationMetadata,
    rollback: RollbackEvidence,
    executable: bool,
}

impl MutationPlan {
    pub fn create(
        baseline: ConfigurationSnapshot,
        target: ConfigurationSnapshot,
    ) -> Result<Self, PlanError> {
        let diff = ConfigurationDiff::between(&baseline, &target)?;
        let risk = assess_risk(&diff);
        let rollback = build_rollback(&baseline, &diff)?;
        let mut plan = Self {
            schema_version: PLAN_SCHEMA_VERSION,
            plan_id: String::new(),
            baseline,
            target,
            diff,
            risk,
            confirmation: ConfirmationMetadata {
                required: false,
                attended: false,
                scope: "this_plan_only".to_owned(),
                exact_phrase: None,
            },
            rollback,
            executable: false,
        };
        plan.plan_id = plan.computed_id()?;
        plan.confirmation = confirmation_for(&plan.plan_id, !plan.diff.is_empty());
        Ok(plan)
    }

    pub fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub fn plan_id(&self) -> &str {
        &self.plan_id
    }

    pub fn baseline(&self) -> &ConfigurationSnapshot {
        &self.baseline
    }

    pub fn target(&self) -> &ConfigurationSnapshot {
        &self.target
    }

    pub fn diff(&self) -> &ConfigurationDiff {
        &self.diff
    }

    pub fn risk(&self) -> &RiskAssessment {
        &self.risk
    }

    pub fn confirmation(&self) -> &ConfirmationMetadata {
        &self.confirmation
    }

    pub fn rollback(&self) -> &RollbackEvidence {
        &self.rollback
    }

    pub fn executable(&self) -> bool {
        self.executable
    }

    fn computed_id(&self) -> Result<String, PlanError> {
        #[derive(Serialize)]
        struct PlanDigest<'a> {
            schema_version: u16,
            baseline: &'a ConfigurationSnapshot,
            target: &'a ConfigurationSnapshot,
            diff: &'a ConfigurationDiff,
            risk: &'a RiskAssessment,
            rollback: &'a RollbackEvidence,
            executable: bool,
        }

        let digest = PlanDigest {
            schema_version: self.schema_version,
            baseline: &self.baseline,
            target: &self.target,
            diff: &self.diff,
            risk: &self.risk,
            rollback: &self.rollback,
            executable: self.executable,
        };
        Ok(format!("ksplan_v1_{}", canonical_hash(&digest)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanInspection {
    pub valid: bool,
    pub declared_plan_id: String,
    pub computed_plan_id: Option<String>,
    pub issues: Vec<String>,
    pub risk: RiskAssessment,
    pub confirmation: ConfirmationMetadata,
    pub executable: bool,
    pub mutation_endpoint_available: bool,
}

pub fn inspect_plan(plan: &MutationPlan) -> PlanInspection {
    let mut issues = Vec::new();
    if plan.schema_version != PLAN_SCHEMA_VERSION {
        issues.push(format!(
            "unsupported plan schema {}; expected {PLAN_SCHEMA_VERSION}",
            plan.schema_version
        ));
    }

    let expected_diff = ConfigurationDiff::between(&plan.baseline, &plan.target);
    match expected_diff {
        Ok(expected) if expected != plan.diff => {
            issues.push("declared diff does not match the embedded snapshots".to_owned())
        }
        Err(error) => issues.push(error.to_string()),
        Ok(_) => {}
    }

    let expected_risk = assess_risk(&plan.diff);
    if expected_risk != plan.risk {
        issues.push("risk metadata does not match the declared diff".to_owned());
    }
    match build_rollback(&plan.baseline, &plan.diff) {
        Ok(expected) if expected != plan.rollback => {
            issues.push("rollback evidence does not match the baseline snapshot".to_owned())
        }
        Err(error) => issues.push(error.to_string()),
        Ok(_) => {}
    }
    if plan.executable {
        issues.push("v0.2 plans must never be marked executable".to_owned());
    }

    let computed_plan_id = match plan.computed_id() {
        Ok(value) => {
            if value != plan.plan_id {
                issues.push("plan ID does not match the deterministic plan payload".to_owned());
            }
            Some(value)
        }
        Err(error) => {
            issues.push(error.to_string());
            None
        }
    };
    let expected_confirmation = confirmation_for(&plan.plan_id, !plan.diff.is_empty());
    if expected_confirmation != plan.confirmation {
        issues.push("confirmation metadata does not match the plan ID".to_owned());
    }

    PlanInspection {
        valid: issues.is_empty(),
        declared_plan_id: plan.plan_id.clone(),
        computed_plan_id,
        issues,
        risk: plan.risk.clone(),
        confirmation: plan.confirmation.clone(),
        executable: false,
        mutation_endpoint_available: false,
    }
}

#[derive(Debug, Error)]
pub enum PlanError {
    #[error("unsupported configuration snapshot schema {0}")]
    UnsupportedSnapshotSchema(u16),
    #[error("unsupported keyboard {vendor_id:04x}:{product_id:04x}")]
    UnsupportedDevice { vendor_id: u16, product_id: u16 },
    #[error("invalid configuration snapshot: {0}")]
    InvalidSnapshot(String),
    #[error("baseline and target describe different keyboard identities")]
    DeviceMismatch,
    #[error("could not serialize deterministic plan data: {0}")]
    Serialization(#[from] serde_json::Error),
}

fn validate_pair(
    baseline: &ConfigurationSnapshot,
    target: &ConfigurationSnapshot,
) -> Result<(), PlanError> {
    baseline.validate()?;
    target.validate()?;
    if baseline.device.name != target.device.name
        || baseline.device.layout != target.device.layout
        || baseline.device.vendor_id != target.device.vendor_id
        || baseline.device.product_id != target.device.product_id
    {
        return Err(PlanError::DeviceMismatch);
    }
    if baseline
        .configuration
        .layers
        .iter()
        .zip(&target.configuration.layers)
        .any(|(before, after)| before.name != after.name)
    {
        return Err(PlanError::InvalidSnapshot(
            "target snapshots may not rename fixed Q3 Max layers".to_owned(),
        ));
    }
    Ok(())
}

fn assess_risk(diff: &ConfigurationDiff) -> RiskAssessment {
    let mut level = RiskLevel::None;
    let mut reasons = BTreeSet::new();
    for change in &diff.changes {
        let (change_level, reason) = match change {
            ConfigurationChange::Rgb { .. } => {
                (RiskLevel::Low, "changes persistent lighting configuration")
            }
            ConfigurationChange::DefaultLayer { .. } => (
                RiskLevel::Moderate,
                "changes which operating-system layer is active",
            ),
            ConfigurationChange::Encoder { .. } => {
                (RiskLevel::Moderate, "changes a rotary encoder binding")
            }
            ConfigurationChange::Keycode { .. } => (
                RiskLevel::High,
                "changes one or more persistent key bindings",
            ),
            ConfigurationChange::WirelessPower { .. } => (
                RiskLevel::High,
                "changes wireless power-management behavior",
            ),
            ConfigurationChange::Debounce { .. } => (
                RiskLevel::High,
                "changes switch filtering and may affect typing reliability",
            ),
            ConfigurationChange::MacroUsage { .. } => (
                RiskLevel::Critical,
                "macro contents are not captured by the current inspection protocol",
            ),
            ConfigurationChange::SnapClickUsage { .. } => (
                RiskLevel::Critical,
                "Snap Click pair definitions are not captured by the current inspection protocol",
            ),
        };
        level = level.max(change_level);
        reasons.insert(reason.to_owned());
    }
    RiskAssessment {
        level,
        reasons: reasons.into_iter().collect(),
    }
}

fn build_rollback(
    baseline: &ConfigurationSnapshot,
    diff: &ConfigurationDiff,
) -> Result<RollbackEvidence, PlanError> {
    let mut limitations = BTreeSet::new();
    for change in &diff.changes {
        match change {
            ConfigurationChange::MacroUsage { .. } => {
                limitations.insert(
                    "macro byte contents are not present; only usage metadata was observed"
                        .to_owned(),
                );
            }
            ConfigurationChange::SnapClickUsage { .. } => {
                limitations.insert(
                    "Snap Click pair definitions are not present; only pair counts were observed"
                        .to_owned(),
                );
            }
            _ => {}
        }
    }
    Ok(RollbackEvidence {
        baseline_snapshot_id: baseline.id()?,
        embedded_baseline: baseline.clone(),
        complete_for_diff: limitations.is_empty(),
        limitations: limitations.into_iter().collect(),
    })
}

fn confirmation_for(plan_id: &str, required: bool) -> ConfirmationMetadata {
    ConfirmationMetadata {
        required,
        attended: required,
        scope: "this_plan_only".to_owned(),
        exact_phrase: required.then(|| format!("CONFIRM {plan_id}")),
    }
}

fn canonical_hash<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn snapshot() -> ConfigurationSnapshot {
        ConfigurationSnapshot {
            schema_version: CONFIG_SNAPSHOT_SCHEMA_VERSION,
            device: SnapshotDevice {
                name: "Keychron Q3 Max".to_owned(),
                layout: "ANSI encoder".to_owned(),
                vendor_id: KEYCHRON_VENDOR_ID,
                product_id: Q3_MAX_ANSI_PRODUCT_ID,
                firmware: "v1.1.1 test".to_owned(),
                via_protocol: 12,
                keychron_protocol: 2,
                qmk_command_set: 2,
            },
            configuration: KeyboardConfiguration {
                active_default_layer: 0,
                layers: (0..Q3_MAX_LAYERS)
                    .map(|index| SnapshotKeymapLayer {
                        index: index as u8,
                        name: format!("Layer {index}"),
                        matrix: vec![vec![0; Q3_MAX_COLUMNS]; Q3_MAX_ROWS],
                    })
                    .collect(),
                macros: SnapshotMacroInfo {
                    slots: 16,
                    buffer_bytes: 1698,
                    used_bytes: 0,
                },
                snap_click: SnapshotSnapClickInfo {
                    pair_capacity: 20,
                    configured_pairs: 0,
                },
                wireless_power: SnapshotWirelessPower {
                    backlight_timeout_seconds: 600,
                    sleep_timeout_seconds: 7200,
                },
                debounce: SnapshotDebounceInfo {
                    algorithm_id: 4,
                    algorithm: "symmetric eager, per key".to_owned(),
                    time_ms: 50,
                },
                rgb: SnapshotRgbInfo {
                    brightness: 255,
                    effect: 5,
                    speed: 127,
                    hue: 0,
                    saturation: 255,
                },
                encoders: (0..Q3_MAX_LAYERS)
                    .map(|layer| SnapshotEncoderBinding {
                        layer: layer as u8,
                        counter_clockwise: 0x80,
                        clockwise: 0x81,
                    })
                    .collect(),
            },
        }
    }

    #[test]
    fn snapshots_round_trip_and_have_stable_ids() {
        let snapshot = snapshot();
        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        let restored: ConfigurationSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, snapshot);
        assert_eq!(restored.id().unwrap(), snapshot.id().unwrap());
        assert!(snapshot.id().unwrap().starts_with("kssnap_v1_"));
    }

    #[test]
    fn diff_is_explicit_and_deterministically_ordered() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.configuration.active_default_layer = 2;
        target.configuration.layers[0].matrix[1][2] = 0x1234;
        target.configuration.rgb.brightness = 42;

        let diff = ConfigurationDiff::between(&baseline, &target).unwrap();
        assert_eq!(diff.changes.len(), 3);
        assert!(matches!(
            diff.changes[0],
            ConfigurationChange::DefaultLayer { from: 0, to: 2 }
        ));
        assert!(matches!(
            diff.changes[1],
            ConfigurationChange::Keycode {
                layer: 0,
                row: 1,
                column: 2,
                from: 0,
                to: 0x1234
            }
        ));
        assert!(matches!(diff.changes[2], ConfigurationChange::Rgb { .. }));
    }

    #[test]
    fn identical_inputs_create_identical_plan_ids() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.configuration.rgb.effect = 18;
        let first = MutationPlan::create(baseline.clone(), target.clone()).unwrap();
        let second = MutationPlan::create(baseline, target).unwrap();

        assert_eq!(first.plan_id(), second.plan_id());
        assert!(first.plan_id().starts_with("ksplan_v1_"));
        assert!(!first.executable());
        assert_eq!(first.risk().level, RiskLevel::Low);
        assert!(inspect_plan(&first).valid);
    }

    #[test]
    fn changed_intent_changes_plan_id() {
        let baseline = snapshot();
        let mut target_a = baseline.clone();
        target_a.configuration.rgb.brightness = 41;
        let mut target_b = baseline.clone();
        target_b.configuration.rgb.brightness = 42;

        let a = MutationPlan::create(baseline.clone(), target_a).unwrap();
        let b = MutationPlan::create(baseline, target_b).unwrap();
        assert_ne!(a.plan_id(), b.plan_id());
    }

    #[test]
    fn post_flash_drift_is_grouped_into_one_reviewable_plan() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.configuration.rgb.brightness = 239;
        target.configuration.rgb.effect = 18;
        let mut baseline = baseline;
        baseline.configuration.layers[2].matrix[0][15] = 32265;
        target.configuration.layers[2].matrix[0][15] = 104;

        let plan = MutationPlan::create(baseline, target).unwrap();
        assert_eq!(plan.diff().changes.len(), 2);
        assert!(matches!(
            plan.diff().changes[0],
            ConfigurationChange::Keycode {
                layer: 2,
                row: 0,
                column: 15,
                from: 32265,
                to: 104
            }
        ));
        assert!(matches!(
            plan.diff().changes[1],
            ConfigurationChange::Rgb { .. }
        ));
        assert_eq!(plan.risk().level, RiskLevel::High);
        assert!(plan.rollback().complete_for_diff);
        assert!(!plan.executable());
    }

    #[test]
    fn mutation_intent_requires_plan_bound_attended_confirmation() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.configuration.layers[0].matrix[0][0] = 41;
        let plan = MutationPlan::create(baseline, target).unwrap();

        assert_eq!(plan.risk().level, RiskLevel::High);
        assert!(plan.confirmation().required);
        assert!(plan.confirmation().attended);
        assert_eq!(
            plan.confirmation().exact_phrase.as_deref(),
            Some(format!("CONFIRM {}", plan.plan_id()).as_str())
        );
        assert!(!plan.executable());
    }

    #[test]
    fn no_op_plan_needs_no_confirmation_but_stays_non_executable() {
        let baseline = snapshot();
        let plan = MutationPlan::create(baseline.clone(), baseline).unwrap();
        assert!(plan.diff().is_empty());
        assert_eq!(plan.risk().level, RiskLevel::None);
        assert!(!plan.confirmation().required);
        assert!(!plan.executable());
    }

    #[test]
    fn incomplete_observation_is_visible_in_rollback_evidence() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.configuration.macros.used_bytes = 10;
        let plan = MutationPlan::create(baseline, target).unwrap();

        assert_eq!(plan.risk().level, RiskLevel::Critical);
        assert!(!plan.rollback().complete_for_diff);
        assert_eq!(plan.rollback().limitations.len(), 1);
    }

    #[test]
    fn inspection_rejects_tampered_serialized_plan() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.configuration.rgb.brightness = 42;
        let plan = MutationPlan::create(baseline, target).unwrap();
        let mut json = serde_json::to_value(&plan).unwrap();
        json["plan_id"] = serde_json::Value::String("ksplan_v1_forged".to_owned());
        let forged: MutationPlan = serde_json::from_value(json).unwrap();

        let inspection = inspect_plan(&forged);
        assert!(!inspection.valid);
        assert!(
            inspection
                .issues
                .iter()
                .any(|issue| issue.contains("plan ID"))
        );
    }

    #[test]
    fn rejects_cross_device_or_structurally_invalid_snapshots() {
        let baseline = snapshot();
        let mut wrong_device = baseline.clone();
        wrong_device.device.layout = "different".to_owned();
        assert!(matches!(
            MutationPlan::create(baseline.clone(), wrong_device),
            Err(PlanError::DeviceMismatch)
        ));

        let mut invalid = baseline.clone();
        invalid.configuration.layers[0].matrix.pop();
        assert!(matches!(
            MutationPlan::create(baseline, invalid),
            Err(PlanError::InvalidSnapshot(_))
        ));
    }

    #[test]
    fn compares_configuration_across_firmware_versions_on_the_same_keyboard() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.device.firmware = "v1.1.1 keysmith-0.2".to_owned();
        target.device.keychron_protocol += 1;

        let plan = MutationPlan::create(baseline, target).unwrap();
        assert!(plan.diff().is_empty());
        assert!(!plan.confirmation().required);
        assert!(!plan.executable());
    }

    #[test]
    fn rejects_unrepresented_layer_metadata_changes() {
        let baseline = snapshot();
        let mut target = baseline.clone();
        target.configuration.layers[0].name = "renamed".to_owned();
        assert!(matches!(
            MutationPlan::create(baseline, target),
            Err(PlanError::InvalidSnapshot(_))
        ));
    }
}
