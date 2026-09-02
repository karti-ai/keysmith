// Copyright 2026 Kartios
// SPDX-License-Identifier: MIT

//! The macOS event tap.
//!
//! macOS will not hand global key events to an ordinary process without
//! permission, so the first run prompts for Accessibility and the tap is
//! disabled until it is granted. That is a one-time grant per binary path.

use std::cell::Cell;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType, EventField,
};

use crate::config::{Binding, Config};

/// Print the keycode of every key pressed, so a binding can be written against
/// a real value instead of a guess.
///
/// Function keys above F20 have no documented macOS virtual keycode, and whether
/// one arrives at all depends on the OS mapping it, so this is the only reliable
/// way to bind such a key.
pub fn learn() -> anyhow::Result<()> {
    eprintln!("Press the key you want to bind. Ctrl-C to stop.");
    eprintln!("If nothing appears, macOS has not granted Accessibility to this binary,");
    eprintln!("or the key produces no event at all and needs a different keycode in firmware.");

    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        // Listen-only: learn mode must never swallow keys, or a mistake here
        // would make the keyboard unusable while it runs.
        CGEventTapOptions::ListenOnly,
        vec![CGEventType::KeyDown],
        |_, _, event| {
            let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            println!("keycode {keycode}");
            None
        },
    )
    .map_err(|_| accessibility_error())?;

    attach(&tap);
    CFRunLoop::run_current();
    Ok(())
}

pub fn run(config: Config, log_all: bool) -> anyhow::Result<()> {
    if log_all {
        eprintln!("logging every keycode; press the key you want to bind");
    }
    for binding in &config.bindings {
        eprintln!(
            "watching keycode {} ({}), long press at {} ms",
            binding.keycode,
            binding.name.as_deref().unwrap_or("unnamed"),
            binding.long_press_ms
        );
    }

    // Press time per keycode. macOS repeats KeyDown while a key is held, so the
    // first KeyDown starts the clock and repeats are ignored; the decision is
    // made on KeyUp, which is the only point the duration is known.
    //
    // A Cell rather than a plain local: the tap callback is Fn, not FnMut,
    // because Core Graphics may invoke it from more than one place.
    let pressed_at: Cell<Option<(i64, Instant)>> = Cell::new(None);

    let tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown, CGEventType::KeyUp],
        move |_, event_type, event| {
            let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);

            if log_all && matches!(event_type, CGEventType::KeyDown) {
                eprintln!("keycode {keycode}");
            }

            let Some(binding) = config.find(keycode) else {
                return None;
            };

            match event_type {
                CGEventType::KeyDown => {
                    if !matches!(pressed_at.get(), Some((code, _)) if code == keycode) {
                        pressed_at.set(Some((keycode, Instant::now())));
                    }
                }
                CGEventType::KeyUp => {
                    let held = match pressed_at.replace(None) {
                        Some((code, at)) if code == keycode => at.elapsed(),
                        // A KeyUp with no matching KeyDown means the tap started
                        // mid-press. Treat it as a tap rather than dropping it.
                        _ => Duration::ZERO,
                    };
                    dispatch(binding, held);
                }
                _ => {}
            }

            if binding.consume {
                // There is no way to return NULL through this wrapper, so the
                // event is neutered instead: a Null-typed event is delivered but
                // means nothing, and the focused application never sees the key.
                event.set_type(CGEventType::Null);
            }
            None
        },
    )
    .map_err(|_| accessibility_error())?;

    attach(&tap);
    CFRunLoop::run_current();
    Ok(())
}

/// Run the command for however long the key was held.
///
/// Spawned detached and never waited on: a slow action, such as an unreachable
/// Home Assistant, must not stall the event tap. macOS disables a tap that
/// blocks for too long, which would silently stop every binding.
fn dispatch(binding: &Binding, held: Duration) {
    let long = held >= Duration::from_millis(binding.long_press_ms);
    let argv = if long && !binding.on_long_press.is_empty() {
        &binding.on_long_press
    } else {
        &binding.on_press
    };

    let Some((program, arguments)) = argv.split_first() else {
        return;
    };

    eprintln!(
        "{} {} press ({} ms)",
        binding.name.as_deref().unwrap_or("key"),
        if long { "long" } else { "short" },
        held.as_millis()
    );

    if let Err(error) = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
    {
        eprintln!("failed to run {program}: {error}");
    }
}

fn attach(tap: &CGEventTap) {
    let loop_source = tap
        .mach_port
        .create_runloop_source(0)
        .expect("event tap run loop source");
    let run_loop = CFRunLoop::get_current();
    unsafe { run_loop.add_source(&loop_source, kCFRunLoopCommonModes) };
    tap.enable();
}

fn accessibility_error() -> anyhow::Error {
    anyhow::anyhow!(
        "macOS refused to create the event tap.\n\
         Grant Accessibility to this binary: System Settings > Privacy & Security >\n\
         Accessibility, then add the keysmith-hotkey binary and enable it.\n\
         The grant follows the binary path, so re-grant after moving or rebuilding it."
    )
}
