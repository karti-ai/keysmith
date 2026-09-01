# Keysmith public-site agent notes

This directory builds the public, static project site for
`https://keychron.karti.ai/`. It is deliberately separate from `apps/web`,
which is the private control surface for a USB-connected keyboard.

## Non-negotiable boundary

- Do not call `/api`, open a WebSocket, load third-party runtime code, or use a
  browser HID, USB, serial, or Bluetooth API.
- Do not add a live demo mode. Real device inspection stays on the loopback
  Keysmith service and authenticated private-network access.
- Never present an apply, DFU, bootloader, flash, pairing, or receiver-control
  action on this site.
- Keysmith is a source preview. Do not invent a download, packaged installer,
  firmware binary, stable release, signature, attestation, or universal model
  claim.
- The only validated target is the Keychron Q3 Max ANSI encoder. Management is
  USB-only; Bluetooth and 2.4 GHz remain normal typing transports.
- Preserve the visible independence disclaimer. Keysmith is not affiliated
  with or endorsed by Keychron, QMK, or VIA.

## Product and visual truth

- Use the checked-in screenshots as labeled reference-lab evidence, not a live
  public dashboard.
- Recovery images, configuration evidence, full device dumps, pairing data,
  and macro contents are operator-private and never site assets.
- The accepted **Attended Instrument** concepts and comparison ledger live in
  `../../docs/site/`. Keep the industrial editorial system: near-black,
  warm-white type, signal red, restrained verified green, cool hairlines,
  compact radii, and open section rhythm.
- Keep all meaningful copy and controls code-native. Respect reduced motion,
  44 px mobile targets, visible focus, semantic landmarks, and descriptive alt
  text.

## Verification

```bash
npm ci
npm audit --audit-level=high
npm run build
npm run test:sites
npm run test:qa
```

The production Caddy deploy uses only `dist/client`. The optional worker files
remain checked in so the static surface can also be packaged by compatible
preview tooling; they must never gain device or network authority. Deployment
provenance, static-boundary scanning, and rollback live in `../../deploy/public/`.
