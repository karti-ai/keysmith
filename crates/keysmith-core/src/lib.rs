mod attended;
mod device;
mod planning;
mod protocol;

pub use device::{HidrawTransport, Transport, discover_q3_max_raw_hid};
pub use planning::{
    CONFIG_SNAPSHOT_SCHEMA_VERSION, ConfigurationChange, ConfigurationDiff, ConfigurationSnapshot,
    ConfirmationMetadata, KeyboardConfiguration, MutationPlan, PLAN_SCHEMA_VERSION, PlanError,
    PlanInspection, RiskAssessment, RiskLevel, RollbackEvidence, SnapshotDebounceInfo,
    SnapshotDevice, SnapshotEncoderBinding, SnapshotKeymapLayer, SnapshotMacroInfo,
    SnapshotRgbInfo, SnapshotSnapClickInfo, SnapshotWirelessPower, inspect_plan,
};
pub use protocol::{
    DebounceInfo, DeviceIdentity, EncoderBinding, Feature, Inspection, Inspector, KeymapLayer,
    KeysmithBuildInfo, KeysmithDeviceInfo, KeysmithMacroMetadata, KeysmithProbe, KeysmithProtocol,
    KeysmithRgbSummary, KeysmithRuntime, KeysmithWirelessSummary, KeysmithWriteStatus, MacroInfo,
    ProtocolError, RgbInfo, SnapClickInfo, WirelessPower, probe_keysmith,
};

pub const KEYCHRON_VENDOR_ID: u16 = 0x3434;
pub const Q3_MAX_ANSI_PRODUCT_ID: u16 = 0x0830;

pub fn inspect_connected() -> Result<Inspection, ProtocolError> {
    let path = discover_q3_max_raw_hid()?;
    let transport = HidrawTransport::open(&path)?;
    Inspector::new(transport, path).inspect()
}

pub fn probe_connected() -> Result<KeysmithProbe, ProtocolError> {
    let path = discover_q3_max_raw_hid()?;
    let mut transport = HidrawTransport::open(&path)?;
    probe_keysmith(&mut transport)
}
pub use attended::{AttendedBundle, AttendedOperation, PreparedOperation, compile_attended_bundle};
