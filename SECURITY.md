# Security policy

## Supported versions

Security fixes are applied to the current `main` branch. Keysmith is pre-1.0;
older snapshots and firmware experiments are not supported releases.

## Reporting a vulnerability

Please use GitHub private vulnerability reporting when available. Do not open a
public issue for a vulnerability that could enable keyboard mutation, expose
local device state, bypass physical confirmation, or invoke a bootloader or
firmware update.

Include the affected commit, operating system, keyboard model/layout, transport
(USB/Bluetooth/2.4 GHz), reproduction steps, and whether physical access was
required. Never attach full device-flash readbacks, pairing material, private
configuration snapshots, credentials, or sensitive logs.

## Trust boundary

The server binds to loopback and serves a same-origin UI. It intentionally has
no apply, DFU, or flash endpoint. Firmware mutation is a separate attended
workflow requiring an exact plan and physical confirmation. Reports that weaken
these boundaries are treated as security issues, not ordinary feature bugs.
