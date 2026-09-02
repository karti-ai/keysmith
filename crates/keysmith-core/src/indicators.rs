// Copyright 2026 Kartios
// SPDX-License-Identifier: MIT

//! Host-painted status LEDs.
//!
//! Firmware 0.6 lets the host colour arbitrary LEDs so the keyboard can display
//! something the keyboard cannot know: whether a service is up, a build passed,
//! a queue is draining.
//!
//! These are not configuration. They write no persistent state, so they do not
//! go through the plan machinery: there is nothing to roll back and nothing to
//! diff. They also expire. The firmware drops every override roughly 90 seconds
//! after the last refresh, because a status display still showing green after
//! its monitor died is worse than one that goes dark — it actively asserts that
//! everything is fine.
//!
//! Raw HID is USB-only on this board, so indicators only update while the
//! keyboard is on the cable. On Bluetooth they simply expire and the F-row
//! returns to the normal effect, which is the honest outcome.

use serde::{Deserialize, Serialize};

use crate::attended::KEYSMITH_COMMAND;
use crate::device::Transport;
use crate::write::WriteError;

const KEYSMITH_SET_INDICATORS: u8 = 0x20;
const FLAG_REPLACE: u8 = 0x01;

/// Entries that fit in one 32-byte packet after the four-byte header.
pub const ENTRIES_PER_PACKET: usize = (32 - 4) / 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Indicator {
    /// LED index in the board's own numbering. On the Q3 Max ANSI, 0 is Esc and
    /// 1 through 12 are the F-row.
    pub led: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Indicator {
    pub fn new(led: u8, (red, green, blue): (u8, u8, u8)) -> Self {
        Self { led, red, green, blue }
    }
}

/// Replace every override with `indicators`.
///
/// Sent as however many packets are needed. Only the first carries the replace
/// flag, so the set is applied additively after one clear rather than each
/// packet wiping the one before it.
///
/// An empty slice still sends one packet: that clears the display and refreshes
/// the expiry clock, which is how a monitor with nothing to report keeps the
/// keyboard from deciding the monitor is gone.
pub fn set<T: Transport>(transport: &mut T, indicators: &[Indicator]) -> Result<(), WriteError> {
    let chunks: Vec<&[Indicator]> = if indicators.is_empty() {
        vec![&[]]
    } else {
        indicators.chunks(ENTRIES_PER_PACKET).collect()
    };

    for (index, chunk) in chunks.iter().enumerate() {
        let mut packet = vec![KEYSMITH_COMMAND, KEYSMITH_SET_INDICATORS];
        packet.push(if index == 0 { FLAG_REPLACE } else { 0 });
        packet.push(chunk.len() as u8);
        for indicator in chunk.iter() {
            packet.extend_from_slice(&[indicator.led, indicator.red, indicator.green, indicator.blue]);
        }

        let response = transport.exchange(&packet)?;
        if response[0] != KEYSMITH_COMMAND || response[1] != KEYSMITH_SET_INDICATORS {
            return Err(WriteError::Protocol(
                crate::protocol::ProtocolError::UnexpectedResponse {
                    command: KEYSMITH_COMMAND,
                    response: response.iter().map(|b| format!("{b:02x}")).collect(),
                },
            ));
        }
        if response[2] != 0 {
            return Err(WriteError::Rejected {
                index: index as u8,
                stage: "indicators",
                status: response[2],
            });
        }
    }

    Ok(())
}

/// Clear the display and refresh the expiry clock.
pub fn clear<T: Transport>(transport: &mut T) -> Result<(), WriteError> {
    set(transport, &[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_row_needs_two_packets() {
        // Twelve F-keys do not fit in one 32-byte packet, and getting this
        // wrong silently truncates the display.
        assert_eq!(ENTRIES_PER_PACKET, 7);
        assert_eq!((12_usize).div_ceil(ENTRIES_PER_PACKET), 2);
    }
}
