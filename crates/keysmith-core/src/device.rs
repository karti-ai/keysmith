use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    path::{Path, PathBuf},
};

use crate::protocol::ProtocolError;

const RAW_USAGE_DESCRIPTOR_PREFIX: [u8; 6] = [0x06, 0x60, 0xff, 0x09, 0x61, 0xa1];

pub trait Transport {
    fn exchange(&mut self, request: &[u8]) -> Result<[u8; 32], ProtocolError>;
}

pub fn discover_q3_max_raw_hid() -> Result<PathBuf, ProtocolError> {
    let entries = fs::read_dir("/sys/class/hidraw").map_err(ProtocolError::Io)?;
    let mut matches = Vec::new();

    for entry in entries {
        let entry = entry.map_err(ProtocolError::Io)?;
        let device = entry.path().join("device");
        let uevent = fs::read_to_string(device.join("uevent")).unwrap_or_default();
        if !uevent.contains("HID_ID=0003:00003434:00000830") {
            continue;
        }
        let descriptor = fs::read(device.join("report_descriptor")).unwrap_or_default();
        if descriptor.starts_with(&RAW_USAGE_DESCRIPTOR_PREFIX) {
            matches.push(PathBuf::from("/dev").join(entry.file_name()));
        }
    }

    matches.sort();
    matches
        .into_iter()
        .next()
        .ok_or(ProtocolError::DeviceNotFound)
}

pub struct HidrawTransport {
    file: File,
}

impl HidrawTransport {
    pub fn open(path: &Path) -> Result<Self, ProtocolError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
            .map_err(ProtocolError::Io)?;
        Ok(Self { file })
    }

    fn drain(&mut self) -> Result<(), ProtocolError> {
        let mut scratch = [0_u8; 64];
        loop {
            match self.file.read(&mut scratch) {
                Ok(0) => return Ok(()),
                Ok(_) => continue,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
                Err(error) => return Err(ProtocolError::Io(error)),
            }
        }
    }
}

impl Transport for HidrawTransport {
    fn exchange(&mut self, request: &[u8]) -> Result<[u8; 32], ProtocolError> {
        if request.len() > 32 {
            return Err(ProtocolError::RequestTooLarge(request.len()));
        }
        self.drain()?;

        // Linux hidraw write buffers begin with the report ID. QMK Raw HID has
        // no numbered reports, so byte zero is 0 and the 32-byte payload follows.
        let mut report = [0_u8; 33];
        report[1..1 + request.len()].copy_from_slice(request);
        self.file.write_all(&report).map_err(ProtocolError::Io)?;

        let mut poll_fd = libc::pollfd {
            fd: self.file.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut poll_fd, 1, 1_500) };
        if ready < 0 {
            return Err(ProtocolError::Io(std::io::Error::last_os_error()));
        }
        if ready == 0 {
            return Err(ProtocolError::Timeout);
        }

        let mut response = [0_u8; 32];
        self.file
            .read_exact(&mut response)
            .map_err(ProtocolError::Io)?;
        Ok(response)
    }
}

/// Raw HID usage page and usage of the Q3 Max's Keysmith interface.
///
/// Every Keychron Q3 Max exposes several HID interfaces on the same VID/PID;
/// only this one carries the vendor protocol. `hidraw` discovery above
/// distinguishes them by report descriptor, which Linux exposes and macOS does
/// not, so the portable path matches on usage page and usage instead.
#[cfg(not(target_os = "linux"))]
pub const RAW_USAGE_PAGE: u16 = 0xff60;
#[cfg(not(target_os = "linux"))]
pub const RAW_USAGE: u16 = 0x61;

#[cfg(not(target_os = "linux"))]
/// A `hidapi`-backed transport.
///
/// `hidraw` is Linux-only: it reads `/sys/class/hidraw`, which does not exist on
/// macOS or Windows. That made the whole tool Linux-only, which is a problem
/// when the keyboard normally lives on a laptop. This backend talks through
/// IOHIDManager on macOS and hid.dll on Windows, and still works on Linux.
pub struct HidApiTransport {
    device: hidapi::HidDevice,
}

#[cfg(not(target_os = "linux"))]
impl HidApiTransport {
    /// Open the Q3 Max's Keysmith interface, whichever platform we are on.
    pub fn open() -> Result<Self, ProtocolError> {
        let api = hidapi::HidApi::new().map_err(|error| ProtocolError::Hid(error.to_string()))?;
        let info = api
            .device_list()
            .find(|device| {
                device.vendor_id() == crate::KEYCHRON_VENDOR_ID
                    && device.product_id() == crate::Q3_MAX_ANSI_PRODUCT_ID
                    && device.usage_page() == RAW_USAGE_PAGE
                    && device.usage() == RAW_USAGE
            })
            .ok_or(ProtocolError::DeviceNotFound)?;
        let device = info
            .open_device(&api)
            .map_err(|error| ProtocolError::Hid(error.to_string()))?;
        Ok(Self { device })
    }
}

#[cfg(not(target_os = "linux"))]
impl Transport for HidApiTransport {
    fn exchange(&mut self, request: &[u8]) -> Result<[u8; 32], ProtocolError> {
        if request.len() > 32 {
            return Err(ProtocolError::RequestTooLarge(request.len()));
        }

        // Same convention as hidraw: byte zero is the report ID, and QMK Raw HID
        // uses unnumbered reports, so it stays zero.
        let mut report = [0_u8; 33];
        report[1..1 + request.len()].copy_from_slice(request);
        self.device
            .write(&report)
            .map_err(|error| ProtocolError::Hid(error.to_string()))?;

        let mut response = [0_u8; 32];
        match self.device.read_timeout(&mut response, 1_500) {
            Ok(0) => Err(ProtocolError::Timeout),
            Ok(_) => Ok(response),
            Err(error) => Err(ProtocolError::Hid(error.to_string())),
        }
    }
}

/// Either transport, so callers do not care which one opened.
pub enum KeyboardTransport {
    Hidraw(HidrawTransport),
    #[cfg(not(target_os = "linux"))]
    HidApi(HidApiTransport),
}

impl Transport for KeyboardTransport {
    fn exchange(&mut self, request: &[u8]) -> Result<[u8; 32], ProtocolError> {
        match self {
            Self::Hidraw(transport) => transport.exchange(request),
            #[cfg(not(target_os = "linux"))]
            Self::HidApi(transport) => transport.exchange(request),
        }
    }
}

/// Open the connected keyboard by whichever route this platform supports.
///
/// `hidraw` is tried first because it is the path validated on the reference
/// board, and it falls through to `hidapi` when the sysfs tree is absent or the
/// interface is not there.
pub fn open_keyboard() -> Result<(KeyboardTransport, String), ProtocolError> {
    match discover_q3_max_raw_hid() {
        Ok(path) => {
            let transport = HidrawTransport::open(&path)?;
            let label = path.display().to_string();
            Ok((KeyboardTransport::Hidraw(transport), label))
        }
        #[cfg(not(target_os = "linux"))]
        Err(_) => {
            let transport = HidApiTransport::open()?;
            Ok((KeyboardTransport::HidApi(transport), "hidapi".to_owned()))
        }
        #[cfg(target_os = "linux")]
        Err(error) => Err(error),
    }
}
