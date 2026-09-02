// Copyright 2026 Kartios
// SPDX-License-Identifier: MIT

//! Show whether services are up on the keyboard's F-row.
//!
//! Probing is done by shelling out to `curl` rather than linking an HTTP and
//! TLS stack into a keyboard utility. curl is present on every machine this
//! runs on, handles redirects, proxies and modern TLS correctly, and keeps this
//! binary free of a dependency tree far larger than the rest of the project.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use keysmith_core::{Indicator, open_keyboard, status_leds};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    /// LED to colour. On the Q3 Max ANSI, 1 through 12 are the F-row.
    pub led: u8,
    pub name: String,
    pub url: String,
    /// Treated as up. Defaults to any 2xx or 3xx.
    #[serde(default)]
    pub expect_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusConfig {
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Above this, a reachable service is shown amber rather than green.
    #[serde(default = "default_slow_ms")]
    pub slow_ms: u128,
    pub targets: Vec<Target>,
}

fn default_interval() -> u64 { 30 }
fn default_timeout() -> u64 { 5 }
fn default_slow_ms() -> u128 { 1500 }

pub fn config_path() -> PathBuf {
    if let Some(path) = std::env::var_os("KEYSMITH_STATUS_CONFIG") {
        return PathBuf::from(path);
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join(".config/keysmith/status.json")
}

pub fn load() -> Result<StatusConfig> {
    let path = config_path();
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let config: StatusConfig = serde_json::from_str(&text)
        .with_context(|| format!("{} is not valid JSON", path.display()))?;
    anyhow::ensure!(!config.targets.is_empty(), "{} lists no targets", path.display());
    Ok(config)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Up,
    Slow,
    Wrong,
    Down,
}

impl Health {
    fn colour(self) -> (u8, u8, u8) {
        match self {
            Health::Up => (0, 255, 40),
            Health::Slow => (255, 170, 0),
            // Reachable but answering wrongly is amber-red: the box is alive,
            // the service is not doing its job.
            Health::Wrong => (255, 60, 0),
            Health::Down => (255, 0, 0),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Health::Up => "up",
            Health::Slow => "slow",
            Health::Wrong => "wrong status",
            Health::Down => "down",
        }
    }
}

/// Probe one target. Never returns an error: an unprobeable target is `Down`,
/// which is the information the display needs.
pub fn probe(target: &Target, timeout: u64, slow_ms: u128) -> (Health, String) {
    let started = Instant::now();
    let output = Command::new("curl")
        .args([
            "-s", "-o", "/dev/null",
            "-w", "%{http_code}",
            "--max-time", &timeout.to_string(),
            "-L",
            &target.url,
        ])
        .output();

    let elapsed = started.elapsed().as_millis();

    let Ok(output) = output else {
        return (Health::Down, "curl could not be run".to_owned());
    };
    let code: u16 = String::from_utf8_lossy(&output.stdout).trim().parse().unwrap_or(0);

    if code == 0 {
        return (Health::Down, format!("unreachable after {elapsed} ms"));
    }
    let acceptable = match target.expect_status {
        Some(expected) => code == expected,
        None => (200..400).contains(&code),
    };
    if !acceptable {
        return (Health::Wrong, format!("HTTP {code}"));
    }
    if elapsed > slow_ms {
        return (Health::Slow, format!("HTTP {code} in {elapsed} ms"));
    }
    (Health::Up, format!("HTTP {code} in {elapsed} ms"))
}

/// Probe everything once and paint the result.
pub fn run_once(config: &StatusConfig, quiet: bool) -> Result<()> {
    let mut indicators = Vec::with_capacity(config.targets.len());
    for target in &config.targets {
        let (health, detail) = probe(target, config.timeout_seconds, config.slow_ms);
        if !quiet {
            println!("  {:<28} {:<12} {detail}", target.name, health.label());
        }
        indicators.push(Indicator::new(target.led, health.colour()));
    }

    let (mut transport, _) = open_keyboard()?;
    status_leds::set(&mut transport, &indicators)?;
    Ok(())
}

/// Probe and paint forever.
///
/// A cycle that fails to reach the keyboard is reported and the loop continues:
/// the board is regularly unplugged or on Bluetooth, where Raw HID does not
/// exist, and that must not end the watch. The firmware expires the display on
/// its own, so a keyboard we cannot reach goes dark rather than showing a stale
/// all-clear.
pub fn watch(config: &StatusConfig) -> Result<()> {
    let interval = Duration::from_secs(config.interval_seconds);
    println!(
        "watching {} targets every {}s",
        config.targets.len(),
        config.interval_seconds
    );
    loop {
        if let Err(error) = run_once(config, false) {
            eprintln!("cycle failed: {error}");
        }
        std::thread::sleep(interval);
    }
}
