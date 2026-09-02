mod attended;
mod device;
mod planning;
mod keycodes;
#[rustfmt::skip]
mod keycodes_generated;
mod protocol;
mod scene;
mod write;

pub use device::{
    HidrawTransport, KeyboardTransport, Transport, discover_q3_max_raw_hid, open_keyboard,
};
#[cfg(not(target_os = "linux"))]
pub use device::HidApiTransport;
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
    let (transport, label) = open_keyboard()?;
    Inspector::new(transport, std::path::PathBuf::from(label)).inspect()
}

pub fn probe_connected() -> Result<KeysmithProbe, ProtocolError> {
    let (mut transport, _) = open_keyboard()?;
    probe_keysmith(&mut transport)
}
pub use attended::{AttendedBundle, AttendedOperation, PreparedOperation, compile_attended_bundle};
pub use write::{
    OperationReceipt, WriteError, WriteMode, WriteReceipt, WriteState, cancel, execute_bundle,
    write_mode, write_state,
};

/// Apply a compiled bundle to the connected keyboard.
pub fn execute_bundle_connected(bundle: &AttendedBundle) -> Result<WriteReceipt, WriteError> {
    let (mut transport, _) = open_keyboard()?;
    execute_bundle(&mut transport, bundle)
}
pub use scene::{
    SCENE_SCHEMA, Scene, SceneEncoder, SceneError, SceneKey, SceneRgb, SceneWireless,
    scene_directory, scene_path,
};
pub mod scenes {
    //! Reading and writing scenes on disk.
    pub use crate::scene::{list, load, read_scene, save, validate_name};
}
pub use keycodes::{Keycode, KeycodeError};
