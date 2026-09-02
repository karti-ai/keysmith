---
name: keysmith
description: Inspect and configure a Keychron Q3 Max from the command line — read its identity, keymap, RGB and wireless state, capture and apply named scenes, and make configuration writes that carry rollback evidence. Use for any keyboard lighting, keymap, encoder, debounce or wireless-power change, and for diagnosing why a write did not land.
---

# Keysmith

`keychronctl` reads and configures a Keychron Q3 Max. Every write goes through a
plan: the tool snapshots the board, diffs it against the desired state, and
applies only the difference, keeping rollback evidence for what it changed.

Prefer this over Keychron Launcher or VIA when working non-interactively. It is
scriptable, every command takes `--json`, and it never touches firmware.

## Orient first

Run these before changing anything. They are read-only.

```sh
keychronctl firmware-probe     # is Keysmith firmware installed, which protocol
keychronctl write status       # is the gate locked, and does this board commit directly
keychronctl inspect --json     # full current state
```

`write status` decides what is possible:

| Write mode | Meaning |
|---|---|
| `Direct` | Firmware 0.5+. Configuration writes apply immediately. |
| `ChordRequired` | Firmware 0.3, or built with `KEYSMITH_REQUIRE_ARM_CHORD`. Every write needs a human holding Esc + Space + Right Control for three seconds. Do not attempt unattended writes; say so and stop. |

If `firmware-probe` reports Keysmith is not installed, the board is on stock
Keychron firmware. Reads still work; writes do not. Flashing is **not** available from
this tool and must not be attempted from an agent path. There is no flash,
bootloader-jump or DFU command in the protocol: DFU is entered by physically
holding Esc while reconnecting the keyboard, and no host software can put a board
there. If firmware needs changing, that is a separate attended procedure carried
out by a person at the keyboard.

## Change something

The safe order is always: capture, rehearse, apply.

```sh
keychronctl scene capture before-my-change     # rollback you can restore
keychronctl set rgb --hue 160 --dry-run        # rehearse: prints the plan, no I/O
keychronctl set rgb --hue 160                  # apply
```

To undo, apply the scene you captured:

```sh
keychronctl scene apply before-my-change
```

### Scenes

A scene sets only the fields it names. Everything unnamed is left as found, so a
scene that sets brightness will not reset the hue.

```sh
keychronctl scene list
keychronctl scene show focus
keychronctl scene diff focus          # what would change, no I/O
keychronctl scene apply focus
```

Scenes live in `$KEYSMITH_SCENE_DIR`, else `$XDG_CONFIG_HOME/keysmith/scenes`,
else `~/.config/keysmith/scenes`, as JSON named after the scene. Names must be
lowercase letters, digits, dashes or underscores because they resolve to paths.

`scene capture` records RGB, debounce, wireless power and encoders. It does
**not** record the keymap, so restoring a scene can never silently revert key
bindings someone changed in the meantime. Scenes can still *set* keys — write
them into the `keys` array by hand.

## Reading the output

- `scene diff` and `--dry-run` never touch hardware. Use them freely.
- `Nothing to do` means the board already matches; this is success, not failure.
- Applying reports each operation and the board state after it. The gate must
  read `Locked` afterwards.
- A warning about the RGB effect index means the scene moves the board to an
  effect the firmware may not compile. A firmware with a single effect has no
  switch case for any other value and renders **nothing** — the deck goes dark
  with no way to cycle out of it. Do not change `effect` unless you know how
  many effects the installed firmware has.

## Things that will catch you out

- **Writes are USB-only.** Raw HID does not exist over Bluetooth or 2.4 GHz on
  this board. If the keyboard is on a wireless slot, reads and writes both fail
  with `DeviceNotFound`; that is not a fault. Ask for the cable and the side
  switch set to Cable.
- **The device vanishing is not evidence of breakage.** The selector position
  and wireless use both remove it from USB intentionally.
- **`effect` numbering is firmware-specific.** QMK always compiles
  `RGB_MATRIX_SOLID_COLOR` at index 1, so a build with one custom effect puts it
  at 2, not 1. Read the current value; do not assume.
- **A scene captured from one board is not portable to another** with a
  different firmware build, because effect indices differ.

## Recovering

```sh
keychronctl write cancel      # discard a staged operation, return to locked
keychronctl write status      # confirm
```

`write cancel` is always safe: a board with nothing staged reports success. Run
it if a previous run was interrupted, because a stale staged operation makes the
next commit fail its plan-tag check.

If a write reports success but the board looks wrong, capture the state and diff
it against the scene you meant to apply, rather than writing again blindly.
