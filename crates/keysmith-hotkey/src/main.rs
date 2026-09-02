// Copyright 2026 Kartios
// SPDX-License-Identifier: MIT

//! Turn a keyboard key into a command, with separate short and long presses.
//!
//! The Q3 Max can emit a key that nothing else uses, but a keyboard cannot make
//! an HTTP call. This daemon closes that gap: it watches for one virtual keycode
//! and runs a configured command, distinguishing a tap from a hold.
//!
//! A plain keycode is used rather than the Keysmith Raw HID protocol on purpose.
//! Raw HID is USB-only on this board, and the keyboard spends most of its life on
//! Bluetooth. A keycode arrives on every transport.
//!
//! Nothing here knows what the command does. House-specific actions live in the
//! configuration, not in this binary.

mod config;

#[cfg(target_os = "macos")]
mod mac;

fn main() -> anyhow::Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "run".to_owned());

    match mode.as_str() {
        "learn" => run_learn(),
        "run" => run_daemon(),
        "config-path" => {
            println!("{}", config::config_path().display());
            Ok(())
        }
        other => {
            eprintln!("unknown mode {other:?}");
            eprintln!("usage: keysmith-hotkey [run|learn|config-path]");
            std::process::exit(2);
        }
    }
}

#[cfg(target_os = "macos")]
fn run_learn() -> anyhow::Result<()> {
    mac::learn()
}

#[cfg(target_os = "macos")]
fn run_daemon() -> anyhow::Result<()> {
    // KEYSMITH_HOTKEY_LOG_ALL exists for first-time setup. macOS attributes a
    // TCC request to the launching process, so an event tap started over SSH is
    // attributed to sshd and the Accessibility prompt never reaches the user.
    // Discovery therefore has to happen under launchd, in the GUI session, where
    // there is no terminal to read `learn` output from -- so the daemon logs
    // every keycode it sees instead.
    let log_all = std::env::var_os("KEYSMITH_HOTKEY_LOG_ALL").is_some();
    let config = match config::load() {
        Ok(config) => config,
        Err(error) if log_all => {
            eprintln!("{error}");
            eprintln!("continuing anyway to log keycodes for setup");
            config::Config::default()
        }
        Err(error) => return Err(error),
    };
    mac::run(config, log_all)
}

#[cfg(not(target_os = "macos"))]
fn run_learn() -> anyhow::Result<()> {
    unsupported()
}

#[cfg(not(target_os = "macos"))]
fn run_daemon() -> anyhow::Result<()> {
    unsupported()
}

#[cfg(not(target_os = "macos"))]
fn unsupported() -> anyhow::Result<()> {
    anyhow::bail!(
        "keysmith-hotkey currently implements only the macOS event tap. \
         Run it on the machine the keyboard is paired to."
    )
}
