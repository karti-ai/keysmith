# Keysmith

Keysmith is a local-first control surface for the Keychron Q3 Max. It combines a reusable Rust Raw HID protocol core, a CLI, a loopback-only API, and a React keyboard editor.

The installed v0.3 firmware can inspect the connected keyboard and supports a narrowly bounded, one-operation transaction protocol gated by a physical chord. Keysmith preserves live state, stages browser-local drafts, and generates deterministic plans, but the browser and localhost server expose no apply route. Device execution remains a separate attended-terminal workflow.

![Keysmith control center](docs/screenshots/keysmith-overview.png)

<details>
<summary>More screenshots</summary>

![Attended-only firmware workspace](docs/screenshots/keysmith-firmware.png)

![Keysmith mobile control center](docs/screenshots/keysmith-mobile.png)

</details>

## Validated hardware

- Keychron Q3 Max ANSI
- USB VID/PID `3434:0830`
- Raw HID usage page `0xFF60`, usage `0x61`
- Stock firmware `v1.1.1 2025-04-23-11:57:08`

## Run

On Linux, install a scoped udev rule for this exact Q3 Max VID/PID so members of
`plugdev` can access its hidraw interfaces without running Keysmith as root,
then reconnect the keyboard:

```udev
KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3434", ATTRS{idProduct}=="0830", MODE="0660", GROUP="plugdev", TAG+="uaccess"
```

```bash
cargo run -p keysmith-cli -- inspect
cargo run -p keysmith-cli -- inspect --json
cargo run -p keysmith-cli -- firmware-probe --json
cargo run -p keysmith-cli -- snapshot --json
cargo run -p keysmith-cli -- plan create --baseline before.json --target draft.json --json
cargo run -p keysmith-cli -- plan prepare --file plan.json --json
cargo run -p keysmith-server

cd apps/web
npm ci
npm run dev
```

The production server listens on `127.0.0.1:3762` and serves both the API and the built web app. Vite listens on `127.0.0.1:4173` during frontend development.

## Private-network deployment

Keysmith always binds its server to loopback. If remote access is needed, put it
behind an authenticated private-network proxy such as Tailscale Serve; never
publish port `3762` directly. For example:

```text
https://your-device.your-tailnet.ts.net:8463/
```

The checked-in `deploy/keysmith.service.example` user-unit template expects the checkout at
`~/.local/share/keysmith`. Build before restarting it:

```bash
cd ~/.local/share/keysmith/apps/web
npm ci && npm run build
cd ../..
cargo build --release -p keysmith-server -p keysmith-cli
systemctl --user restart keysmith.service
```

Inspect or remove the tailnet route with:

```bash
tailscale serve status
tailscale serve --https=8463 off
```

## Safety boundary

The installed firmware exposes only its plan-bound, physically confirmed v0.3 transaction protocol; the server and browser expose no write path. `plan prepare` is an offline compiler: it prints candidate packets and never opens the keyboard. Any v0.3 operation must follow snapshot → plan → exact one-operation packet → physical chord → commit → readback → archive, and relocks on every terminal condition. Firmware flashing is never exposed through the unattended server or agent API.

The v0.2 read-only wire format is in
[docs/V0_2_PROTOCOL.md](docs/V0_2_PROTOCOL.md), and the physically confirmed
transaction design is in
[docs/V0_3_ATTENDED_PROTOCOL.md](docs/V0_3_ATTENDED_PROTOCOL.md).

Hardware and wireless findings are in
[docs/DEVICE_NOTES.md](docs/DEVICE_NOTES.md). Device dumps, configuration
snapshots, factory images, and private deployment records are intentionally not
part of the public source distribution.

## Firmware source and licensing

The Rust/React Keysmith application is licensed under MIT. The keyboard
firmware is maintained as a separate fork of Keychron QMK and remains under its
upstream GPL licenses and notices. Do not redistribute a firmware binary
without making the complete corresponding source and build inputs available.

Keysmith is an independent community project and is not affiliated with or
endorsed by Keychron, QMK, or VIA.
