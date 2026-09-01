# Contributing

Keysmith controls physical keyboard state, so changes should preserve a narrow,
inspectable trust boundary.

Before submitting a change:

```bash
cargo fmt -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
cd apps/web
npm ci
npm audit
npm run build
```

Keep protocol logic in `crates/keysmith-core`; the CLI, server, and web UI are
consumers. New operations must have a deterministic plan, complete rollback
evidence, bounded payload validation, explicit confirmation, and readback.

Do not commit device dumps, configuration snapshots, firmware binaries,
pairing data, credentials, internal URLs, or machine-specific deployment paths.
Do not add an unattended apply, bootloader, DFU, or flash endpoint.

Firmware contributions belong in the separate Keychron/QMK fork and retain its
upstream GPL licensing and notices.
