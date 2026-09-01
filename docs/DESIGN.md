# Keysmith design specification

Source: `keysmith-primary-concept.png`, generated from the live Q3 Max inventory on 2026-08-31.

## Direction

Premium industrial tool rather than gaming peripheral software. The keyboard is the focal canvas. Navigation and agent controls are slim rails around an open editor.

## Tokens

- Background: `#0f1113`
- Raised surface: `#181b1e`
- Key surface: `#222629`
- Primary text: `#f4f2ec`
- Muted text: `#92989f`
- Border: `#30353a`
- Selection/action: `#d84a43`
- Connected/safe: `#43b581`
- Radius: 4–10px, never pill-shaped for structural containers
- UI type: Inter/Geist-like sans; IDs and keycodes use a monospace face

## Primary screen anatomy

1. Slim navigation rail.
2. Restrained device top bar.
3. Four layer tabs.
4. Accurate ANSI TKL keyboard canvas with knob.
5. Selected-key inspector.
6. Agent proposal rail with a structured reversible diff.
7. Thin state/status strip.

## Interaction contract

- Selecting a layer changes the visible key labels.
- Selecting a key updates the inspector.
- Agent proposal review expands a concrete before/after change.
- Apply remains preview-only until the write milestone is explicitly enabled.
- All controls are keyboard accessible and focus-visible.

