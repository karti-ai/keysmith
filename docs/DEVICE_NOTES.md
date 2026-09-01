# Keychron Q3 Max notes

The reference-lab observations below were validated against one Keychron Q3 Max
ANSI encoder over USB. They are not inferred for every Keysmith installation.

## Live inventory

- USB VID/PID: `3434:0830`
- Raw HID: dynamic `/dev/hidrawN`, usage page `0xFF60`, usage `0x61`
- Firmware: `v1.1.1 2025-04-23-11:57:08`
- VIA protocol: 12
- Keychron protocol: 2
- QMK command set: 2
- Four layers: Mac, Mac Fn, Win, Win Fn
- 16 macro slots, all empty
- 20 Snap Click pair slots, all empty
- Wireless backlight timeout: 600 seconds
- Wireless sleep timeout: 7200 seconds
- Per-key symmetric eager debounce: 50 ms
- RGB: brightness 239, effect 18, speed 127, hue 0, saturation 255
- Encoder: volume on layers 0/2; RGB brightness on layers 1/3

## Wireless boundary

The stock firmware pairs Bluetooth through the keyboard's `BT_HST1`, `BT_HST2`, and `BT_HST3` keycodes. Holding a host key for more than two seconds invokes the firmware pairing path. The stock Raw HID command set exposes wireless configuration and status, but not an arbitrary host-side command that starts Bluetooth pairing.

The board has two firmware domains: QMK runs on the STM32F401, while a separate LKBT51 module handles Bluetooth and 2.4 GHz radio traffic over SPI. Keychron's current source contains LKBT51 Raw HID packet commands, but the Q3 Max build does not enable the wireless receive dispatcher. Keychron's own Launcher instructions describe configuration over a cable or, on supported products, a 2.4 GHz receiver; they do not describe Bluetooth as a Launcher transport. Therefore Bluetooth management is not a Keysmith capability; both the reference-lab v0.3 image and the current public candidate keep Keysmith management USB-only.

That gives Keysmith two practical paths:

1. Safe stock-firmware orchestration: show the chosen slot, guide the physical long press, then open or drive the host OS Bluetooth UI.
2. Future firmware research: evaluate narrowly scoped select-slot or pairing assistance without weakening USB recovery access. The current candidate deliberately does not implement these commands.

Bluetooth itself is not a suitable Keysmith configuration transport for the stock board. Deep configuration stays on USB via the local Keysmith daemon. Keysmith does not detect or provision a 2.4 GHz receiver; any receiver must be confirmed as Q Max compatible and tested independently, and a random Keychron receiver should not be assumed interchangeable.

The Keysmith firmware source is based on Keychron's `2025q3` branch, whose Q3 Max USB device version is `1.1.1`. The current public `keysmith/q3-max-v3` candidate is `bbbd4fac3c`: it compiles in public CI, publishes no binary, and has not been validated as an exact flashed release image. The reference-lab v0.3 image was built from `2b422a5a12`; its successful hardware validation must not be attributed to the newer public candidate. Historical v0.2 source is `e9972e1a43`. The older `wireless_playground` definition reports `1.0.0` and should not be used for this target.

## Useful upstream work

- [Keychron QMK source](https://github.com/Keychron/qmk_firmware)
- [Keychron Launcher](https://launcher.keychron.com/#/keymap)
- [Keychron Q3 Max user guide](https://www.keychron.com/pages/keychron-q3-max-user-guide)
- [Linux hidraw documentation](https://docs.kernel.org/hid/hidraw.html)
- [blueutil for macOS Bluetooth automation](https://github.com/toy/blueutil)
