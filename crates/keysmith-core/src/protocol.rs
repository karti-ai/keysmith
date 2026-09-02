use std::{collections::BTreeSet, path::PathBuf};

use serde::Serialize;
use thiserror::Error;

use crate::{KEYCHRON_VENDOR_ID, Q3_MAX_ANSI_PRODUCT_ID, Transport};

const ROWS: usize = 6;
const COLS: usize = 17;
const EXPECTED_LAYERS: usize = 4;
const KEYMAP_BYTES: usize = EXPECTED_LAYERS * ROWS * COLS * 2;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Keychron Q3 Max Raw HID interface was not found")]
    DeviceNotFound,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("keyboard did not answer within 1500 ms")]
    Timeout,
    #[error("request is {0} bytes; Raw HID payloads are limited to 32")]
    RequestTooLarge(usize),
    #[error("unexpected response for command 0x{command:02x}: {response}")]
    UnexpectedResponse { command: u8, response: String },
    #[error("device reports {0} layers; this build expects four Q3 Max layers")]
    UnexpectedLayerCount(u8),
    #[error("HID error: {0}")]
    Hid(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    DefaultLayer,
    Bluetooth,
    TwoPointFourGhz,
    StateNotifications,
    DynamicDebounce,
    SnapClick,
    KeychronRgb,
}

#[derive(Debug, Serialize)]
pub struct DeviceIdentity {
    pub name: &'static str,
    pub layout: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
    pub path: PathBuf,
    pub firmware: String,
    pub via_protocol: u16,
    pub keychron_protocol: u8,
    pub qmk_command_set: u8,
}

#[derive(Debug, Serialize)]
pub struct KeymapLayer {
    pub index: u8,
    pub name: &'static str,
    pub matrix: Vec<Vec<u16>>,
}

#[derive(Debug, Serialize)]
pub struct MacroInfo {
    pub slots: u8,
    pub buffer_bytes: u16,
    pub used_bytes: usize,
}

#[derive(Debug, Serialize)]
pub struct SnapClickInfo {
    pub pair_capacity: u8,
    pub configured_pairs: u8,
}

#[derive(Debug, Serialize)]
pub struct WirelessPower {
    pub backlight_timeout_seconds: u16,
    pub sleep_timeout_seconds: u16,
}

#[derive(Debug, Serialize)]
pub struct DebounceInfo {
    pub algorithm_id: u8,
    pub algorithm: &'static str,
    pub time_ms: u8,
}

#[derive(Debug, Serialize)]
pub struct RgbInfo {
    pub brightness: u8,
    pub effect: u8,
    pub speed: u8,
    pub hue: u8,
    pub saturation: u8,
}

#[derive(Debug, Serialize)]
pub struct EncoderBinding {
    pub layer: u8,
    pub counter_clockwise: u16,
    pub clockwise: u16,
}

#[derive(Debug, Serialize)]
pub struct Inspection {
    pub connected: bool,
    pub identity: DeviceIdentity,
    pub features: BTreeSet<Feature>,
    pub active_default_layer: u8,
    pub layers: Vec<KeymapLayer>,
    pub macros: MacroInfo,
    pub snap_click: SnapClickInfo,
    pub wireless_power: WirelessPower,
    pub debounce: DebounceInfo,
    pub rgb: RgbInfo,
    pub encoders: Vec<EncoderBinding>,
    pub write_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct KeysmithProbe {
    pub installed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<KeysmithProtocol>,
}

#[derive(Debug, Serialize)]
pub struct KeysmithProtocol {
    pub major: u8,
    pub minor: u8,
    pub packet_bytes: u8,
    pub runtime_status: bool,
    pub usb_only: bool,
    pub mutation_capabilities: u32,
    pub read_capabilities: u8,
    pub build_page_count: u8,
    pub keymap_chunk_keycodes: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via_eeprom_magic: Option<String>,
    pub runtime: KeysmithRuntime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<KeysmithBuildInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<KeysmithDeviceInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rgb: Option<KeysmithRgbSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wireless: Option<KeysmithWirelessSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macro_metadata: Option<KeysmithMacroMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_status: Option<KeysmithWriteStatus>,
}

#[derive(Debug, Serialize)]
pub struct KeysmithWriteStatus {
    pub state_id: u8,
    pub state: &'static str,
    pub last_result: u8,
    pub operation: u8,
    pub plan_tag: String,
    pub operation_index: u8,
    pub operation_total: u8,
    pub usb_ready: bool,
}

#[derive(Debug, Serialize)]
pub struct KeysmithRuntime {
    pub transport_id: u8,
    pub transport: &'static str,
    pub wireless_state_id: u8,
    pub wireless_state: &'static str,
    pub usb_power: bool,
    pub mutations_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_layer: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_layer: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usb_state: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_leds: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wireless_host: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct KeysmithBuildInfo {
    pub keysmith: String,
    pub qmk_git_hash: String,
    pub qmk_version: String,
    pub qmk_build_date: String,
    pub keyboard: String,
    pub keymap: String,
}

#[derive(Debug, Serialize)]
pub struct KeysmithDeviceInfo {
    pub matrix_rows: u8,
    pub matrix_cols: u8,
    pub layer_count: u8,
    pub rgb_led_count: u8,
    pub encoder_count: u8,
    pub via_protocol: u16,
    pub protocol_version: u8,
    pub qmk_command_set: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version: u16,
    pub raw_packet_bytes: u8,
}

#[derive(Debug, Serialize)]
pub struct KeysmithRgbSummary {
    pub enabled: bool,
    pub suspended: bool,
    pub effect: u8,
    pub hue: u8,
    pub saturation: u8,
    pub brightness: u8,
    pub speed: u8,
    pub flags: u8,
    pub led_count: u8,
}

#[derive(Debug, Serialize)]
pub struct KeysmithWirelessSummary {
    pub state_id: u8,
    pub state: &'static str,
    pub host: u8,
    pub battery_percentage: u8,
    pub battery_valid: bool,
    pub battery_voltage_mv: u16,
    pub battery_empty: bool,
    pub battery_critical: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_sample_age_ms: Option<u32>,
    pub transport_id: u8,
    pub transport: &'static str,
}

#[derive(Debug, Serialize)]
pub struct KeysmithMacroMetadata {
    pub slots: u8,
    pub buffer_bytes: u16,
    pub contents_exposed: bool,
}

pub fn probe_keysmith<T: Transport>(transport: &mut T) -> Result<KeysmithProbe, ProtocolError> {
    let protocol = match transport.exchange(&[0xac, 0x00]) {
        Ok(response) => response,
        Err(ProtocolError::Timeout) => {
            return Ok(KeysmithProbe {
                installed: false,
                protocol: None,
            });
        }
        Err(error) => return Err(error),
    };

    /* Stock Keychron firmware answers unknown namespaces with 0xff. */
    if protocol[0] == 0xff {
        return Ok(KeysmithProbe {
            installed: false,
            protocol: None,
        });
    }

    if protocol[0] != 0xac || protocol[1] != 0x00 || protocol[2] != 0x00 || &protocol[3..5] != b"KS"
    {
        return Err(ProtocolError::UnexpectedResponse {
            command: 0xac,
            response: hex(&protocol),
        });
    }

    let runtime = keysmith_get(transport, 0x01, &[])?;
    let major = protocol[5];
    let minor = protocol[6];
    let is_v0_2_or_newer = major > 0 || minor >= 2;
    let is_v0_3_or_newer = major > 0 || minor >= 3;
    let read_capabilities = if is_v0_2_or_newer { protocol[10] } else { 0 };

    let build = if read_capabilities & 0x02 != 0 {
        Some(read_build_info(transport, protocol[14])?)
    } else {
        None
    };

    let device = if read_capabilities & 0x04 != 0 {
        let response = keysmith_get(transport, 0x03, &[])?;
        Some(KeysmithDeviceInfo {
            matrix_rows: response[3],
            matrix_cols: response[4],
            layer_count: response[5],
            rgb_led_count: response[6],
            encoder_count: response[7],
            via_protocol: u16::from_be_bytes([response[8], response[9]]),
            protocol_version: response[10],
            qmk_command_set: response[11],
            vendor_id: u16::from_be_bytes([response[12], response[13]]),
            product_id: u16::from_be_bytes([response[14], response[15]]),
            device_version: u16::from_be_bytes([response[16], response[17]]),
            raw_packet_bytes: response[18],
        })
    } else {
        None
    };

    let rgb = if read_capabilities & 0x08 != 0 {
        let response = keysmith_get(transport, 0x04, &[])?;
        Some(KeysmithRgbSummary {
            enabled: response[3] != 0,
            suspended: response[4] != 0,
            effect: response[5],
            hue: response[6],
            saturation: response[7],
            brightness: response[8],
            speed: response[9],
            flags: response[10],
            led_count: response[11],
        })
    } else {
        None
    };

    let wireless = if read_capabilities & 0x10 != 0 {
        let response = keysmith_get(transport, 0x05, &[])?;
        let sample_age =
            u32::from_be_bytes([response[11], response[12], response[13], response[14]]);
        Some(KeysmithWirelessSummary {
            state_id: response[3],
            state: wireless_state_name(response[3]),
            host: response[4],
            battery_percentage: response[5],
            battery_valid: response[6] != 0,
            battery_voltage_mv: u16::from_be_bytes([response[7], response[8]]),
            battery_empty: response[9] != 0,
            battery_critical: response[10] != 0,
            battery_sample_age_ms: (sample_age != u32::MAX).then_some(sample_age),
            transport_id: response[15],
            transport: transport_name(response[15]),
        })
    } else {
        None
    };

    let macro_metadata = if read_capabilities & 0x80 != 0 {
        let response = keysmith_get(transport, 0x08, &[])?;
        Some(KeysmithMacroMetadata {
            slots: response[3],
            buffer_bytes: u16::from_be_bytes([response[4], response[5]]),
            contents_exposed: response[6] != 0,
        })
    } else {
        None
    };

    let write_status = if is_v0_3_or_newer {
        Some(decode_write_status(keysmith_get(transport, 0x11, &[])?))
    } else {
        None
    };

    Ok(KeysmithProbe {
        installed: true,
        protocol: Some(KeysmithProtocol {
            major,
            minor,
            runtime_status: protocol[7] & 0x01 != 0,
            usb_only: protocol[7] & 0x02 != 0,
            mutation_capabilities: u32::from_le_bytes([
                protocol[8],
                protocol[11],
                protocol[12],
                protocol[13],
            ]),
            packet_bytes: protocol[9],
            read_capabilities,
            build_page_count: if is_v0_2_or_newer { protocol[14] } else { 0 },
            keymap_chunk_keycodes: if is_v0_2_or_newer { protocol[15] } else { 0 },
            via_eeprom_magic: (is_v0_2_or_newer && protocol[16] != 0).then(|| {
                format!(
                    "{:02x}{:02x}{:02x}",
                    protocol[17], protocol[18], protocol[19]
                )
            }),
            runtime: KeysmithRuntime {
                transport_id: runtime[3],
                transport: transport_name(runtime[3]),
                wireless_state_id: runtime[4],
                wireless_state: wireless_state_name(runtime[4]),
                usb_power: runtime[5] != 0,
                mutations_enabled: runtime[6] != 0,
                default_layer: is_v0_2_or_newer.then_some(runtime[7]),
                active_layer: is_v0_2_or_newer.then_some(runtime[8]),
                usb_state: is_v0_2_or_newer.then_some(runtime[9]),
                uptime_ms: is_v0_2_or_newer.then(|| {
                    u32::from_be_bytes([runtime[10], runtime[11], runtime[12], runtime[13]])
                }),
                host_leds: is_v0_2_or_newer.then_some(runtime[14]),
                wireless_host: is_v0_2_or_newer.then_some(runtime[15]),
            },
            build,
            device,
            rgb,
            wireless,
            macro_metadata,
            write_status,
        }),
    })
}

fn decode_write_status(response: [u8; 32]) -> KeysmithWriteStatus {
    KeysmithWriteStatus {
        state_id: response[3],
        state: match response[3] {
            0 => "locked",
            1 => "prepared",
            2 => "armed",
            _ => "unknown",
        },
        last_result: response[4],
        operation: response[5],
        plan_tag: hex(&response[6..14]),
        operation_index: response[14],
        operation_total: response[15],
        usb_ready: response[16] != 0,
    }
}

fn keysmith_get<T: Transport>(
    transport: &mut T,
    subcommand: u8,
    arguments: &[u8],
) -> Result<[u8; 32], ProtocolError> {
    let mut request = vec![0xac, subcommand];
    request.extend_from_slice(arguments);
    let response = transport.exchange(&request)?;
    if response[0] != 0xac || response[1] != subcommand || response[2] != 0 {
        return Err(ProtocolError::UnexpectedResponse {
            command: 0xac,
            response: hex(&response),
        });
    }
    Ok(response)
}

fn read_build_info<T: Transport>(
    transport: &mut T,
    page_count: u8,
) -> Result<KeysmithBuildInfo, ProtocolError> {
    let mut pages = Vec::with_capacity(page_count as usize);
    for page in 0..page_count {
        let response = keysmith_get(transport, 0x02, &[page])?;
        if response[3] != page || response[4] != page_count || response[5] > 25 {
            return Err(ProtocolError::UnexpectedResponse {
                command: 0xac,
                response: hex(&response),
            });
        }
        pages.push(null_terminated_ascii(
            &response[6..6 + response[5] as usize],
        ));
    }
    if pages.len() != 7 {
        return Err(ProtocolError::UnexpectedResponse {
            command: 0xac,
            response: format!("expected 7 build pages, received {}", pages.len()),
        });
    }
    Ok(KeysmithBuildInfo {
        keysmith: pages[0].clone(),
        qmk_git_hash: pages[1].clone(),
        qmk_version: pages[2].clone(),
        qmk_build_date: pages[3].clone(),
        keyboard: format!("{}{}", pages[4], pages[5]),
        keymap: pages[6].clone(),
    })
}

pub struct Inspector<T: Transport> {
    transport: T,
    path: PathBuf,
}

impl<T: Transport> Inspector<T> {
    pub fn new(transport: T, path: PathBuf) -> Self {
        Self { transport, path }
    }

    pub fn inspect(mut self) -> Result<Inspection, ProtocolError> {
        let via = self.get(&[0x01])?;
        self.expect_command(0x01, &via)?;
        let via_protocol = u16::from_be_bytes([via[1], via[2]]);

        let keychron = self.get(&[0xa0])?;
        self.expect_command(0xa0, &keychron)?;

        let firmware = self.get(&[0xa1])?;
        self.expect_command(0xa1, &firmware)?;
        let firmware = null_terminated_ascii(&firmware[1..]);

        let feature_response = self.get(&[0xa2])?;
        self.expect_command(0xa2, &feature_response)?;
        let features = decode_features(feature_response[2]);

        let default_layer = self.get(&[0xa3])?;
        self.expect_command(0xa3, &default_layer)?;

        let layer_response = self.get(&[0x11])?;
        self.expect_command(0x11, &layer_response)?;
        let layer_count = layer_response[1];
        if layer_count as usize != EXPECTED_LAYERS {
            return Err(ProtocolError::UnexpectedLayerCount(layer_count));
        }

        let macro_count = self.get(&[0x0c])?;
        let macro_size = self.get(&[0x0d])?;
        let macro_buffer_size = u16::from_be_bytes([macro_size[1], macro_size[2]]);
        let macro_buffer = self.get_buffer(0x0e, macro_buffer_size as usize)?;

        let keymap = self.get_buffer(0x12, KEYMAP_BYTES)?;

        let misc = self.get(&[0xa7, 0x01])?;
        self.expect_command(0xa7, &misc)?;

        let debounce = self.get(&[0xa7, 0x05])?;
        let wireless = self.get(&[0xa7, 0x0b])?;
        let snap_info = self.get(&[0xa7, 0x07])?;
        let pair_capacity = snap_info[3];
        let configured_pairs = self.count_snap_pairs(pair_capacity)?;

        let rgb_brightness = self.get(&[0x08, 0x03, 0x01])?;
        let rgb_effect = self.get(&[0x08, 0x03, 0x02])?;
        let rgb_speed = self.get(&[0x08, 0x03, 0x03])?;
        let rgb_color = self.get(&[0x08, 0x03, 0x04])?;

        let mut encoders = Vec::with_capacity(EXPECTED_LAYERS);
        for layer in 0..EXPECTED_LAYERS as u8 {
            encoders.push(EncoderBinding {
                layer,
                counter_clockwise: self.get_encoder(layer, false)?,
                clockwise: self.get_encoder(layer, true)?,
            });
        }

        Ok(Inspection {
            connected: true,
            identity: DeviceIdentity {
                name: "Keychron Q3 Max",
                layout: "ANSI encoder",
                vendor_id: KEYCHRON_VENDOR_ID,
                product_id: Q3_MAX_ANSI_PRODUCT_ID,
                path: self.path,
                firmware,
                via_protocol,
                keychron_protocol: keychron[1],
                qmk_command_set: keychron[3],
            },
            features,
            active_default_layer: default_layer[1],
            layers: decode_keymap(&keymap),
            macros: MacroInfo {
                slots: macro_count[1],
                buffer_bytes: macro_buffer_size,
                used_bytes: macro_buffer.iter().filter(|byte| **byte != 0).count(),
            },
            snap_click: SnapClickInfo {
                pair_capacity,
                configured_pairs,
            },
            wireless_power: WirelessPower {
                backlight_timeout_seconds: u16::from_le_bytes([wireless[3], wireless[4]]),
                sleep_timeout_seconds: u16::from_le_bytes([wireless[5], wireless[6]]),
            },
            debounce: DebounceInfo {
                algorithm_id: debounce[4],
                algorithm: debounce_name(debounce[4]),
                time_ms: debounce[5],
            },
            rgb: RgbInfo {
                brightness: rgb_brightness[3],
                effect: rgb_effect[3],
                speed: rgb_speed[3],
                hue: rgb_color[3],
                saturation: rgb_color[4],
            },
            encoders,
            write_enabled: false,
        })
    }

    fn get(&mut self, request: &[u8]) -> Result<[u8; 32], ProtocolError> {
        self.transport.exchange(request)
    }

    fn expect_command(&self, command: u8, response: &[u8; 32]) -> Result<(), ProtocolError> {
        if response[0] == command {
            Ok(())
        } else {
            Err(ProtocolError::UnexpectedResponse {
                command,
                response: hex(response),
            })
        }
    }

    fn get_buffer(&mut self, command: u8, total: usize) -> Result<Vec<u8>, ProtocolError> {
        let mut output = Vec::with_capacity(total);
        while output.len() < total {
            let offset = output.len();
            let size = (total - offset).min(28) as u8;
            let response = self.get(&[command, (offset >> 8) as u8, offset as u8, size])?;
            if response[0] != command
                || response[1] != (offset >> 8) as u8
                || response[2] != offset as u8
                || response[3] != size
            {
                return Err(ProtocolError::UnexpectedResponse {
                    command,
                    response: hex(&response),
                });
            }
            output.extend_from_slice(&response[4..4 + size as usize]);
        }
        Ok(output)
    }

    fn get_encoder(&mut self, layer: u8, clockwise: bool) -> Result<u16, ProtocolError> {
        let response = self.get(&[0x14, layer, 0, u8::from(clockwise)])?;
        self.expect_command(0x14, &response)?;
        Ok(u16::from_be_bytes([response[4], response[5]]))
    }

    fn count_snap_pairs(&mut self, capacity: u8) -> Result<u8, ProtocolError> {
        let mut configured = 0;
        let mut start = 0;
        while start < capacity {
            let count = (capacity - start).min(9);
            let response = self.get(&[0xa7, 0x08, start, count])?;
            self.expect_command(0xa7, &response)?;
            for pair in response[3..3 + count as usize * 3].as_chunks::<3>().0 {
                configured += u8::from(pair[0] != 0);
            }
            start += count;
        }
        Ok(configured)
    }
}

fn decode_features(mask: u8) -> BTreeSet<Feature> {
    let candidates = [
        (0x01, Feature::DefaultLayer),
        (0x02, Feature::Bluetooth),
        (0x04, Feature::TwoPointFourGhz),
        (0x10, Feature::StateNotifications),
        (0x20, Feature::DynamicDebounce),
        (0x40, Feature::SnapClick),
        (0x80, Feature::KeychronRgb),
    ];
    candidates
        .into_iter()
        .filter_map(|(flag, feature)| (mask & flag != 0).then_some(feature))
        .collect()
}

fn decode_keymap(buffer: &[u8]) -> Vec<KeymapLayer> {
    const NAMES: [&str; 4] = ["Mac", "Mac Fn", "Win", "Win Fn"];
    NAMES
        .into_iter()
        .enumerate()
        .map(|(layer, name)| {
            let layer_start = layer * ROWS * COLS * 2;
            let matrix = (0..ROWS)
                .map(|row| {
                    (0..COLS)
                        .map(|col| {
                            let index = layer_start + (row * COLS + col) * 2;
                            u16::from_be_bytes([buffer[index], buffer[index + 1]])
                        })
                        .collect()
                })
                .collect();
            KeymapLayer {
                index: layer as u8,
                name,
                matrix,
            }
        })
        .collect()
}

fn debounce_name(id: u8) -> &'static str {
    match id {
        0 => "symmetric defer, global",
        1 => "symmetric defer, per row",
        2 => "symmetric defer, per key",
        3 => "symmetric eager, per row",
        4 => "symmetric eager, per key",
        5 => "asymmetric eager/defer, per key",
        6 => "none",
        _ => "unknown",
    }
}

fn transport_name(id: u8) -> &'static str {
    match id {
        0 => "none",
        1 => "usb",
        2 => "bluetooth",
        4 => "2.4_ghz",
        _ => "unknown",
    }
}

fn wireless_state_name(id: u8) -> &'static str {
    match id {
        0 => "reset",
        1 => "initialized",
        2 => "disconnected",
        3 => "connected",
        4 => "pairing",
        5 => "reconnecting",
        6 => "suspended",
        _ => "unknown",
    }
}

fn null_terminated_ascii(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeTransport {
        responses: std::collections::VecDeque<Result<[u8; 32], ProtocolError>>,
    }

    impl Transport for FakeTransport {
        fn exchange(&mut self, _request: &[u8]) -> Result<[u8; 32], ProtocolError> {
            self.responses.pop_front().expect("unexpected request")
        }
    }

    #[test]
    fn decodes_live_feature_mask() {
        let features = decode_features(0xe7);
        assert!(features.contains(&Feature::Bluetooth));
        assert!(features.contains(&Feature::TwoPointFourGhz));
        assert!(features.contains(&Feature::DynamicDebounce));
        assert!(features.contains(&Feature::SnapClick));
        assert!(features.contains(&Feature::KeychronRgb));
        assert!(!features.contains(&Feature::StateNotifications));
    }

    #[test]
    fn decodes_four_layer_matrix() {
        let mut bytes = vec![0_u8; KEYMAP_BYTES];
        bytes[0] = 0x12;
        bytes[1] = 0x34;
        let layers = decode_keymap(&bytes);
        assert_eq!(layers.len(), 4);
        assert_eq!(layers[0].matrix[0][0], 0x1234);
        assert_eq!(layers[3].matrix.len(), ROWS);
    }

    #[test]
    fn decodes_v0_3_write_gate_status() {
        let mut response = [0_u8; 32];
        response[..17].copy_from_slice(&[
            0xac, 0x11, 0x00, 1, 0, 2, 0xde, 0xad, 0xbe, 0xef, 0x12, 0x34, 0x56, 0x78, 1, 3, 1,
        ]);

        let status = decode_write_status(response);
        assert_eq!(status.state_id, 1);
        assert_eq!(status.state, "prepared");
        assert_eq!(status.operation, 2);
        assert_eq!(status.plan_tag, "deadbeef12345678");
        assert_eq!(status.operation_index, 1);
        assert_eq!(status.operation_total, 3);
        assert!(status.usb_ready);
    }

    #[test]
    fn recognizes_stock_firmware_timeout_as_not_installed() {
        let mut transport = FakeTransport {
            responses: [Err(ProtocolError::Timeout)].into(),
        };
        let probe = probe_keysmith(&mut transport).unwrap();
        assert!(!probe.installed);
        assert!(probe.protocol.is_none());
    }

    #[test]
    fn recognizes_stock_firmware_unknown_command_as_not_installed() {
        let mut response = [0_u8; 32];
        response[0] = 0xff;
        let mut transport = FakeTransport {
            responses: [Ok(response)].into(),
        };
        let probe = probe_keysmith(&mut transport).unwrap();
        assert!(!probe.installed);
        assert!(probe.protocol.is_none());
    }

    #[test]
    fn decodes_read_only_keysmith_v0_1() {
        let mut protocol = [0_u8; 32];
        protocol[..10].copy_from_slice(&[0xac, 0x00, 0x00, b'K', b'S', 0, 1, 3, 0, 32]);
        let mut runtime = [0_u8; 32];
        runtime[..7].copy_from_slice(&[0xac, 0x01, 0x00, 1, 2, 1, 0]);
        let mut transport = FakeTransport {
            responses: [Ok(protocol), Ok(runtime)].into(),
        };

        let probe = probe_keysmith(&mut transport).unwrap();
        let protocol = probe.protocol.unwrap();
        assert!(probe.installed);
        assert_eq!((protocol.major, protocol.minor), (0, 1));
        assert!(protocol.usb_only);
        assert_eq!(protocol.mutation_capabilities, 0);
        assert_eq!(protocol.runtime.transport, "usb");
        assert!(!protocol.runtime.mutations_enabled);
        assert_eq!(protocol.read_capabilities, 0);
        assert!(protocol.build.is_none());
        assert!(protocol.via_eeprom_magic.is_none());
    }

    #[test]
    fn decodes_read_only_keysmith_v0_2_inventory_without_macro_contents() {
        let mut responses = std::collections::VecDeque::new();

        let mut protocol = [0_u8; 32];
        protocol[..10].copy_from_slice(&[0xac, 0x00, 0x00, b'K', b'S', 0, 2, 3, 0, 32]);
        protocol[10] = 0xff;
        protocol[14] = 7;
        protocol[15] = 12;
        protocol[16] = 1;
        protocol[17..20].copy_from_slice(&[0x26, 0x08, 0x31]);
        responses.push_back(Ok(protocol));

        let mut runtime = [0_u8; 32];
        runtime[..16].copy_from_slice(&[
            0xac, 0x01, 0x00, 1, 3, 1, 0, 2, 2, 4, 0, 0, 0x12, 0x34, 2, 1,
        ]);
        responses.push_back(Ok(runtime));

        for (page, value) in [
            "keysmith/0.2.0",
            "abcdef1234",
            "0.29.1",
            "2026-08-31",
            "keychron/q3_max/ansi_enco",
            "der",
            "keysmith",
        ]
        .into_iter()
        .enumerate()
        {
            let mut response = [0_u8; 32];
            response[..6].copy_from_slice(&[0xac, 0x02, 0x00, page as u8, 7, value.len() as u8]);
            response[6..6 + value.len()].copy_from_slice(value.as_bytes());
            responses.push_back(Ok(response));
        }

        let mut device = [0_u8; 32];
        device[..19].copy_from_slice(&[
            0xac, 0x03, 0x00, 6, 17, 4, 87, 1, 0, 12, 1, 2, 0x34, 0x34, 0x08, 0x30, 0, 1, 32,
        ]);
        responses.push_back(Ok(device));

        let mut rgb = [0_u8; 32];
        rgb[..12].copy_from_slice(&[0xac, 0x04, 0, 1, 0, 5, 120, 200, 239, 64, 0xff, 87]);
        responses.push_back(Ok(rgb));

        let mut wireless = [0_u8; 32];
        wireless[..16].copy_from_slice(&[
            0xac, 0x05, 0, 3, 1, 80, 0, 0x0f, 0xa0, 0, 0, 0xff, 0xff, 0xff, 0xff, 1,
        ]);
        responses.push_back(Ok(wireless));

        let mut macros = [0_u8; 32];
        macros[..7].copy_from_slice(&[0xac, 0x08, 0, 16, 0x10, 0, 0]);
        responses.push_back(Ok(macros));

        let probe = probe_keysmith(&mut FakeTransport { responses }).unwrap();
        let protocol = probe.protocol.unwrap();
        assert_eq!((protocol.major, protocol.minor), (0, 2));
        assert_eq!(protocol.via_eeprom_magic.as_deref(), Some("260831"));
        assert_eq!(protocol.build.unwrap().keysmith, "keysmith/0.2.0");
        assert_eq!(protocol.device.unwrap().product_id, 0x0830);
        assert_eq!(protocol.rgb.unwrap().brightness, 239);
        let wireless = protocol.wireless.unwrap();
        assert!(!wireless.battery_valid);
        assert_eq!(wireless.battery_sample_age_ms, None);
        assert!(!protocol.macro_metadata.unwrap().contents_exposed);
    }
}
