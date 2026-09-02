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

Keysmith draws its boundary between configuration and firmware, not between
reading and writing.

**Configuration** -- keycodes, RGB profile, encoder bindings, debounce, wireless
power timeouts -- is written directly by the CLI on firmware 0.5 and later. Each
write is bound to one staged operation by a plan tag and index, cannot be
replayed against a different operation, and the board relocks after every commit,
error, timeout, transport change or disconnect. Every change is planned against a
snapshot of the current state and carries rollback evidence. This is recoverable
in software.

**Firmware** is not. It is absent from the protocol entirely: there is no flash,
bootloader-jump or DFU command, and legacy Keychron setters that could reach one
are denied before dispatch. DFU is entered by physically holding Esc while
reconnecting the keyboard, so no host software -- including this project -- can
put a board into a flashable state.

The server binds to loopback, serves a same-origin UI, and has no apply, DFU or
flash route. Writes go through the CLI only.

Reports that let software reach the bootloader, that let a commit apply an
operation other than the one staged, or that let the write gate stay unlocked
are treated as security issues rather than ordinary feature bugs.

Firmware built with `KEYSMITH_REQUIRE_ARM_CHORD` additionally requires a physical
Esc + Space + Right Control chord for every configuration write. `keychronctl
write status` reports which model a board is running.
