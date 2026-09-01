# Keysmith v0.3 attended transaction protocol

Status: implemented in the `keysmith/q3-max-v3` firmware branch and validated
on a Keychron Q3 Max ANSI encoder.

The v0.3 firmware adds a narrow mutation protocol without turning the browser
or agent into an unattended keyboard writer. USB is the management plane; the
hardware transport switch remains authoritative.

## Transaction

1. Capture and archive the complete live configuration snapshot.
2. Build and inspect a deterministic core mutation plan.
3. Run `keychronctl plan prepare --file PLAN.json --json` offline. This command
   only compiles packets and never opens a HID device.
4. Prepare exactly one atomic operation containing its eight-byte plan tag,
   operation index, total operation count, and complete target payload.
5. Within 30 seconds, hold physical Esc + Space + Right Control for three
   seconds. Matrix positions are checked, so remapping cannot forge approval.
   Those three key events are suppressed while pending; Esc is amber while
   prepared and green only during the short-lived armed state.
6. Commit using the same plan tag and operation index.
7. Firmware applies once, verifies what it can read back, and immediately
   relocks.
8. The host reads and archives the complete after-state before another
   operation is prepared.

The physical action arms only the already prepared operation. It is not a
general unlock and does not survive success, error, timeout, transport change,
reset, or disconnect.

## Raw HID commands

All packets are 32 bytes, start with Keysmith command `0xAC`, and reserve byte
31 for correlation.

| Subcommand | Purpose |
|---:|---|
| `0x10` | Prepare one bounded operation |
| `0x11` | Read locked/prepared/armed status and last result |
| `0x12` | Commit the exact armed plan tag and operation index |
| `0x13` | Cancel and relock |

Supported operations are keycode (`1`), RGB profile (`2`), encoder
direction (`3`), wireless power timeouts (`4`), and debounce (`5`).

## Explicitly blocked

- Macro writes: current normal inspection intentionally hides content, so
  complete rollback bytes do not exist.
- Snap Click writes: only usage counts are captured, not pair definitions.
- Per-key RGB: complete prior per-key state is not represented in snapshots.
- Bluetooth pairing or host selection: credential replacement is a separate
  high-risk operation class and USB remains the management channel.
- 2.4 GHz receiver provisioning, radio DFU, and STM32 firmware flashing.
- Web/server apply: no apply route exists (`POST /api/apply` returns 405).

The firmware also denies legacy VIA/Keychron setters, VIA bootloader jump,
radio DFU, and factory-test commands before their normal dispatch. This closes
the alternate write paths that would otherwise bypass Keysmith's physical gate;
the corresponding read commands remain available.

## Reference verification

The reference build reports protocol `0.3`, mutation capabilities `0x1f`, and
USB-only management. A live status command returned `locked` with no prepared
operation. A no-op keycode operation was prepared and cancelled before physical
arming; the state returned to `locked`. A legacy VIA setter was rejected with
policy-locked status and the keycode remained unchanged. Canonical pre/post
configuration hashes were identical.

These checks validate the negative and cancellation paths without authorizing
or performing a configuration mutation. Every actual operation still requires
a new exact plan, complete rollback evidence, the physical chord, commit, and
readback.
