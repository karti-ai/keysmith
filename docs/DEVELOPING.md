# Developing Keysmith

## Layout

| Crate | What it owns |
|---|---|
| `keysmith-core` | Everything real. Protocol, planning, scenes, writes, transports. |
| `keysmith-cli` | Argument parsing and printing. No protocol logic. |
| `keysmith-server` | Read-only HTTP for the web interface. Never writes. |

Keep protocol logic in `keysmith-core`. If the CLI needs to know a packet
layout, that knowledge is in the wrong crate.

## The layers, in order

```
inspect ──> ConfigurationSnapshot ──> MutationPlan ──> AttendedBundle ──> write
 (read)         (portable state)        (diff+risk)     (packets)       (device)
```

Each arrow is a separate, testable step, and each is offline except the ends.
A scene is not a fifth concept: it overlays onto a snapshot and rejoins at
`MutationPlan`, which is why scenes get rollback evidence for free.

Nothing may skip a layer. A command that writes without producing a plan has no
diff to show and no rollback to offer, and should not be merged.

## Transports

`Transport::exchange` sends up to 32 bytes and returns 32. Two implementations:

- `HidrawTransport` — Linux. Discovers the right interface among several sharing
  the Q3 Max's VID/PID by matching the report descriptor, which only Linux
  exposes.
- `HidApiTransport` — macOS and Windows, via `hidapi`. Matches on Raw HID usage
  page `0xff60` and usage `0x61` instead, because macOS does not expose report
  descriptors.

`open_keyboard()` tries hidraw first and falls through. `hidapi` needs `libudev`
on Linux and Linux already has a working backend, so the dependency is scoped to
`cfg(not(target_os = "linux"))`.

## Building

```sh
cargo test                       # 30 tests, no hardware needed
cargo build --release -p keysmith-cli
```

Everything except the transports is testable without a keyboard: planning,
diffing, scene overlay and packet compilation are all pure. Add tests there
rather than reaching for hardware.

On macOS, build natively. There is no cross-compilation path, and the `hidapi`
backend cannot be exercised from Linux.

## Firmware

Firmware lives in a separate repository, `keysmith-qmk`, forked from Keychron's
QMK. Flashing is attended by policy and by physics: DFU is entered by holding Esc
while reconnecting, and no host tool can put the board there.

Before any DFU step, take a full 262,144-byte readback with `dfu-util -U`, verify
its size and SHA-256, and keep two copies on separate media. Confirm exactly one
`0483:df11` device is present. Present the exact image path and checksum for
review, and treat approval as single-use and specific to that image.

Two traps that have each cost a session:

- The live firmware source is the `keysmith/q3-max-v3`/`-v5` **worktree**, not
  the `keysmith/q3-max-v0` branch in the main checkout. The v0 branch is the
  abandoned read-only line and builds a plausible image that downgrades the
  protocol.
- QMK only resets a stored RGB effect index when it is exactly `0`. A build that
  removes effects leaves a stale index with no switch case, and the board renders
  nothing. `keyboard_post_init_user()` reconciles it.

## Protocol changes

The firmware and this crate share a packet layout with no generated bindings.
When changing it:

1. Bump `KEYSMITH_PROTOCOL_MINOR` in the firmware.
2. Advertise new capability bits in `GET_PROTOCOL` rather than having the host
   infer them from the version. Byte 11 already carries the write mode this way.
3. Teach the host to degrade against an older board, not to assume.

A host that infers capability from a version number will be wrong the first time
someone runs a custom build.
