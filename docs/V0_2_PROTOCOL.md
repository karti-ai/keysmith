# Keysmith firmware protocol 0.2

Status: installed and host-validated on the Q3 Max on 2026-08-31. The protocol
remains read-only and USB-only; mutation capabilities are zero.

The protocol uses the Q3 Max 32-byte Raw HID report and Keysmith namespace
`0xAC`. It is accepted only when the report arrived over USB and USB power is
present. Every response echoes bytes 0 and 1, places a status code in byte 2,
and echoes the request correlation byte in byte 31.

## Safety contract

- Mutation capabilities are zero. Protocol 0.2 has no write, reset, transport,
  pairing, bootloader, or flashing command.
- Macro contents are never returned. Only slot count and buffer capacity are
  exposed.
- A battery value read while USB is selected is explicitly marked invalid and
  its sample age is unknown; clients must not present it as live telemetry.
- Bluetooth and 2.4 GHz remain input transports, not Keysmith configuration
  transports. Pairing continues to require the keyboard's physical workflow.
- The image pins the VIA EEPROM marker to `260831`, matching the installed
  Keysmith 0.1 image. Rebuilding on another day therefore does not by itself
  reset the dynamic keymap, encoders, or macro buffer.

Status values are `0x00` success, `0x01` bad command, `0x02` bad transport, and
`0x03` bad argument.

## Discovery (`0x00`)

| Byte | Meaning |
|---:|---|
| 3..4 | ASCII `KS` |
| 5..6 | protocol major/minor (`0`, `2`) |
| 7 | legacy capabilities: runtime status, USB-only |
| 8, 11..13 | 32-bit little-endian mutation bitmap; all zero |
| 9 | report size (`32`) |
| 10 | read bitmap: runtime, build, device, RGB, wireless, keymap chunk, encoder, macro metadata |
| 14 | build page count (`7`) |
| 15 | maximum keycodes in one keymap chunk (`12`) |
| 16 | stable VIA marker present |
| 17..19 | VIA marker BCD bytes (`26 08 31`) |

## Read commands

| Subcommand | Arguments | Response payload |
|---:|---|---|
| `0x01` | none | transport, wireless state, USB power, mutation lock, default/active layer, USB state, uptime, host LEDs, wireless host |
| `0x02` | page | page index/count/length and up to 25 ASCII bytes of build identity |
| `0x03` | none | matrix, layer, RGB and encoder counts; VIA/QMK versions; USB identity; report size |
| `0x04` | none | RGB enabled/suspended, effect, HSV, speed, flags and LED count |
| `0x05` | none | wireless state/host, battery value plus validity, voltage, empty/critical flags, sample age and active transport |
| `0x06` | layer, matrix offset | up to 12 big-endian keycodes |
| `0x07` | layer, encoder, clockwise | one big-endian encoder keycode |
| `0x08` | none | macro slot count, buffer capacity, and a false `contents exposed` flag |

Build pages are Keysmith version, QMK Git hash, QMK version, QMK build date,
two consecutive chunks of the keyboard identifier, and keymap identifier.

## Future mutation architecture

A future write protocol is intentionally separate from 0.2. The proposed flow
is: host creates an immutable plan with baseline and rollback evidence; user
reviews its exact hash; keyboard requires USB plus a three-second physical
matrix chord; a 30-second arm window permits one plan-bound operation; any
disconnect, error, timeout, or completion relocks the board. Firmware/radio
flashing will not be exposed through that channel and stays attended-terminal
only.
