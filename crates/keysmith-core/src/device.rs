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
