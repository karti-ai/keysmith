// Copyright 2026 Kartios
// SPDX-License-Identifier: MIT

//! Executing a compiled mutation plan against a connected keyboard.
//!
//! Everything up to this module is offline: a plan is built from two snapshots,
//! inspected, and compiled into prepare packets. This module is the only place
//! that turns those packets into writes on real hardware.
//!
//! Each operation is a two-step exchange. `PREPARE` stages exactly one
//! operation on the keyboard along with a plan tag and an index; `COMMIT`
//! replays that same tag and index to apply it. The firmware relocks after
//! every commit, error, timeout, transport change, or disconnect, so an
//! interrupted run can never leave a half-armed board.
//!
//! Firmware 0.5 and later accepts a host commit directly. Firmware 0.3 requires
//! a physical Esc + Space + Right Control chord between the two steps, and a
//! board built with `KEYSMITH_REQUIRE_ARM_CHORD` still does. [`write_mode`]
//! reports which of the two a board is running so callers can say so plainly
//! rather than discovering it from a lock error.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::attended::{AttendedBundle, AttendedOperation, KEYSMITH_COMMAND, KEYSMITH_PACKET_BYTES};
use crate::device::Transport;
use crate::protocol::ProtocolError;

const KEYSMITH_GET_PROTOCOL: u8 = 0x00;
const KEYSMITH_GET_WRITE_STATUS: u8 = 0x11;
const KEYSMITH_COMMIT_OPERATION: u8 = 0x12;
const KEYSMITH_CANCEL_OPERATION: u8 = 0x13;

/// Firmware write states, mirroring `enum keysmith_write_state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteState {
    Locked,
    Prepared,
    Armed,
    Unknown(u8),
}

impl WriteState {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Self::Locked,
            1 => Self::Prepared,
            2 => Self::Armed,
            other => Self::Unknown(other),
        }
    }
}

/// Whether a board commits on host request or demands the physical chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteMode {
    /// Firmware 0.5+ in the default build: the host may commit directly.
    Direct,
    /// Firmware 0.3, or 0.5 built with `KEYSMITH_REQUIRE_ARM_CHORD`.
    ChordRequired,
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error("this bundle cannot be applied: {}", .0.join("; "))]
    Ineligible(Vec<String>),
    #[error("prepare packet {index} is malformed: {reason}")]
    MalformedPacket { index: u8, reason: String },
    #[error("plan tag {0:?} is not sixteen hex characters")]
    MalformedPlanTag(String),
    #[error(
        "the keyboard refused the commit as locked. This board requires the physical \
         Esc + Space + Right Control chord held for three seconds between prepare and commit; \
         flash firmware 0.5 or later for direct host commits"
    )]
    ChordRequired,
    #[error("the keyboard rejected operation {index} at the {stage} step with status 0x{status:02x}")]
    Rejected { index: u8, stage: &'static str, status: u8 },
    #[error(
        "operation {index} committed but the keyboard reported state {state:?} instead of \
         relocking; the board may hold a stale prepared operation"
    )]
    DidNotRelock { index: u8, state: WriteState },
}

/// What one applied operation did, kept so a caller can archive the run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationReceipt {
    pub index: u8,
    pub total: u8,
    pub operation: AttendedOperation,
    pub committed: bool,
    pub final_state: WriteState,
}

/// The result of applying a whole bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteReceipt {
    pub schema: String,
    pub plan_id: String,
    pub plan_tag: String,
    pub mode: WriteMode,
    pub operations: Vec<OperationReceipt>,
}

/// Ask a board whether it commits directly or wants the chord.
///
/// Byte 11 of the protocol response carries the write mode from firmware 0.5.
/// Firmware 0.3 leaves it zero, which reads correctly as [`WriteMode::ChordRequired`].
pub fn write_mode<T: Transport>(transport: &mut T) -> Result<WriteMode, WriteError> {
    let response = transport.exchange(&[KEYSMITH_COMMAND, KEYSMITH_GET_PROTOCOL])?;
    expect_ok(&response, KEYSMITH_GET_PROTOCOL, 0, "protocol")?;
    Ok(if response[11] == 1 {
        WriteMode::Direct
    } else {
        WriteMode::ChordRequired
    })
}

/// Read the current write gate state without changing it.
pub fn write_state<T: Transport>(transport: &mut T) -> Result<WriteState, WriteError> {
    let response = transport.exchange(&[KEYSMITH_COMMAND, KEYSMITH_GET_WRITE_STATUS])?;
    expect_ok(&response, KEYSMITH_GET_WRITE_STATUS, 0, "status")?;
    Ok(WriteState::from_byte(response[3]))
}

/// Discard any staged operation and return the board to `locked`.
///
/// Safe to call at any time; a board with nothing staged reports success.
pub fn cancel<T: Transport>(transport: &mut T) -> Result<WriteState, WriteError> {
    let response = transport.exchange(&[KEYSMITH_COMMAND, KEYSMITH_CANCEL_OPERATION])?;
    expect_ok(&response, KEYSMITH_CANCEL_OPERATION, 0, "cancel")?;
    write_state(transport)
}

/// Apply every operation in a compiled bundle, in order.
///
/// Stops at the first failure rather than continuing, and leaves the board
/// locked. Operations already committed stay committed: the caller should
/// consult the returned receipts, or the error's operation index, to know how
/// far the run got before rolling back from the plan's own rollback evidence.
pub fn execute_bundle<T: Transport>(
    transport: &mut T,
    bundle: &AttendedBundle,
) -> Result<WriteReceipt, WriteError> {
    if !bundle.eligible {
        return Err(WriteError::Ineligible(bundle.blockers.clone()));
    }

    let mode = write_mode(transport)?;
    let tag = decode_plan_tag(&bundle.plan_tag)?;

    // A board holding a stale prepared operation would fail the tag check on
    // our first commit. Clear it so the run starts from a known state.
    cancel(transport)?;

    let mut receipts = Vec::with_capacity(bundle.operations.len());

    for prepared in &bundle.operations {
        let packet = decode_prepare_packet(prepared.index, &prepared.prepare_packet_hex)?;

        let response = transport.exchange(&packet)?;
        expect_ok(&response, packet[1], prepared.index, "prepare")?;

        if mode == WriteMode::ChordRequired {
            cancel(transport)?;
            return Err(WriteError::ChordRequired);
        }

        let mut commit = vec![KEYSMITH_COMMAND, KEYSMITH_COMMIT_OPERATION];
        commit.extend_from_slice(&tag);
        commit.push(prepared.index);
        let response = transport.exchange(&commit)?;
        if response[2] == STATUS_LOCKED {
            return Err(WriteError::ChordRequired);
        }
        expect_ok(&response, KEYSMITH_COMMIT_OPERATION, prepared.index, "commit")?;

        let final_state = write_state(transport)?;
        if final_state != WriteState::Locked {
            return Err(WriteError::DidNotRelock {
                index: prepared.index,
                state: final_state,
            });
        }

        receipts.push(OperationReceipt {
            index: prepared.index,
            total: prepared.total,
            operation: prepared.operation.clone(),
            committed: true,
            final_state,
        });
    }

    Ok(WriteReceipt {
        schema: "keysmith.write-receipt/v1".to_owned(),
        plan_id: bundle.plan_id.clone(),
        plan_tag: bundle.plan_tag.clone(),
        mode,
        operations: receipts,
    })
}

/// `KEYSMITH_ERROR_LOCKED`, returned when a commit arrives without an arm.
const STATUS_LOCKED: u8 = 0x03;

fn expect_ok(
    response: &[u8; KEYSMITH_PACKET_BYTES],
    subcommand: u8,
    index: u8,
    stage: &'static str,
) -> Result<(), WriteError> {
    if response[0] != KEYSMITH_COMMAND || response[1] != subcommand {
        return Err(WriteError::Protocol(ProtocolError::UnexpectedResponse {
            command: KEYSMITH_COMMAND,
            response: response.iter().map(|b| format!("{b:02x}")).collect(),
        }));
    }
    if response[2] != 0 {
        return Err(WriteError::Rejected {
            index,
            stage,
            status: response[2],
        });
    }
    Ok(())
}

fn decode_plan_tag(tag: &str) -> Result<[u8; 8], WriteError> {
    if tag.len() != 16 {
        return Err(WriteError::MalformedPlanTag(tag.to_owned()));
    }
    let mut bytes = [0_u8; 8];
    for (slot, pair) in bytes.iter_mut().zip(tag.as_bytes().chunks(2)) {
        let text = std::str::from_utf8(pair).map_err(|_| WriteError::MalformedPlanTag(tag.to_owned()))?;
        *slot = u8::from_str_radix(text, 16)
            .map_err(|_| WriteError::MalformedPlanTag(tag.to_owned()))?;
    }
    Ok(bytes)
}

fn decode_prepare_packet(index: u8, hex: &str) -> Result<Vec<u8>, WriteError> {
    if hex.len() != KEYSMITH_PACKET_BYTES * 2 {
        return Err(WriteError::MalformedPacket {
            index,
            reason: format!("expected {} hex characters, found {}", KEYSMITH_PACKET_BYTES * 2, hex.len()),
        });
    }
    let mut packet = Vec::with_capacity(KEYSMITH_PACKET_BYTES);
    for pair in hex.as_bytes().chunks(2) {
        let text = std::str::from_utf8(pair).map_err(|_| WriteError::MalformedPacket {
            index,
            reason: "packet is not valid hex".to_owned(),
        })?;
        packet.push(
            u8::from_str_radix(text, 16).map_err(|_| WriteError::MalformedPacket {
                index,
                reason: format!("{text:?} is not a hex byte"),
            })?,
        );
    }
    if packet[0] != KEYSMITH_COMMAND {
        return Err(WriteError::MalformedPacket {
            index,
            reason: format!("packet does not start with the Keysmith command byte 0x{KEYSMITH_COMMAND:02x}"),
        });
    }
    Ok(packet)
}
