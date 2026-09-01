use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use keysmith_core::{
    ConfigurationSnapshot, MutationPlan, compile_attended_bundle, inspect_connected, inspect_plan,
    probe_connected,
};

#[derive(Parser)]
#[command(
    name = "keychronctl",
    version,
    about = "Inspect a Keychron Q3 Max safely"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read the keyboard's identity, capabilities, and configuration.
    Inspect {
        /// Emit the complete machine-readable inventory.
        #[arg(long)]
        json: bool,
    },
    /// Probe the versioned Keysmith firmware extension without writing.
    FirmwareProbe {
        /// Emit the complete machine-readable probe result.
        #[arg(long)]
        json: bool,
    },
    /// Capture a portable, read-only configuration snapshot.
    Snapshot {
        /// Emit the complete machine-readable snapshot instead of its ID.
        #[arg(long)]
        json: bool,
    },
    /// Create or verify immutable, non-executable mutation plans.
    Plan {
        #[command(subcommand)]
        command: PlanCommand,
    },
}

#[derive(Subcommand)]
enum PlanCommand {
    /// Compile an inspected plan into offline v0.3 prepare packets without opening the keyboard.
    Prepare {
        #[arg(long)]
        file: PathBuf,
        /// Emit the complete machine-readable attended bundle.
        #[arg(long)]
        json: bool,
    },
    /// Build a deterministic preview from baseline and target snapshot files.
    Create {
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        target: PathBuf,
        /// Emit the complete machine-readable plan.
        #[arg(long)]
        json: bool,
    },
    /// Verify a serialized plan's ID, diff, risk, and rollback evidence.
    Inspect {
        #[arg(long)]
        file: PathBuf,
        /// Emit the complete machine-readable verification result.
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Inspect { json } => {
            let inspection = inspect_connected()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&inspection)?);
            } else {
                print_summary(&inspection);
            }
        }
        Command::FirmwareProbe { json } => {
            let probe = probe_connected()?;
            if json {
                println!("{}", serde_json::to_string_pretty(&probe)?);
            } else if let Some(protocol) = probe.protocol {
                println!(
                    "Keysmith firmware protocol {}.{} ({} byte packets, {}, mutations: {})",
                    protocol.major,
                    protocol.minor,
                    protocol.packet_bytes,
                    if protocol.usb_only {
                        "USB-only"
                    } else {
                        "multi-transport"
                    },
                    if protocol.mutation_capabilities != 0 {
                        "attended capability present"
                    } else {
                        "disabled"
                    }
                );
                if let Some(build) = &protocol.build {
                    println!(
                        "  Build        {} · QMK {} ({})",
                        build.keysmith, build.qmk_version, build.qmk_git_hash
                    );
                }
                if let Some(device) = &protocol.device {
                    println!(
                        "  Device       {}×{} matrix · {} layers · {} RGB LEDs",
                        device.matrix_rows,
                        device.matrix_cols,
                        device.layer_count,
                        device.rgb_led_count
                    );
                }
                if let Some(magic) = &protocol.via_eeprom_magic {
                    println!("  VIA marker   {magic} (stable across Keysmith rebuilds)");
                }
                if let Some(wireless) = &protocol.wireless {
                    println!(
                        "  Wireless     {} · battery {}",
                        wireless.state,
                        if wireless.battery_valid {
                            format!("{}%", wireless.battery_percentage)
                        } else {
                            "unavailable on this transport".to_owned()
                        }
                    );
                }
                if let Some(macros) = &protocol.macro_metadata {
                    println!(
                        "  Macros       {} slots · {} bytes · contents {}",
                        macros.slots,
                        macros.buffer_bytes,
                        if macros.contents_exposed {
                            "exposed"
                        } else {
                            "private"
                        }
                    );
                }
                if let Some(status) = &protocol.write_status {
                    println!(
                        "  Write gate   {} · USB {} · operation {}/{}",
                        status.state,
                        if status.usb_ready {
                            "ready"
                        } else {
                            "not ready"
                        },
                        status.operation_index,
                        status.operation_total
                    );
                }
            } else {
                println!("Keysmith firmware extension is not installed (stock firmware detected)");
            }
        }
        Command::Snapshot { json } => {
            let snapshot = ConfigurationSnapshot::from_inspection(&inspect_connected()?);
            if json {
                println!("{}", serde_json::to_string_pretty(&snapshot)?);
            } else {
                println!("Configuration snapshot {}", snapshot.id()?);
                println!(
                    "  Device       {} ({})",
                    snapshot.device.name, snapshot.device.layout
                );
                println!("  Firmware     {}", snapshot.device.firmware);
                println!("  Captured     keymap, RGB, debounce, wireless power, and encoders");
                println!("  Keyboard writes were not performed");
            }
        }
        Command::Plan { command } => match command {
            PlanCommand::Create {
                baseline,
                target,
                json,
            } => {
                let baseline = read_json::<ConfigurationSnapshot>(&baseline)?;
                let target = read_json::<ConfigurationSnapshot>(&target)?;
                let plan = MutationPlan::create(baseline, target)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                } else {
                    println!("Mutation plan {}", plan.plan_id());
                    println!("  Changes      {}", plan.diff().changes.len());
                    println!("  Risk         {:?}", plan.risk().level);
                    println!(
                        "  Confirmation {}",
                        if plan.confirmation().required {
                            "required"
                        } else {
                            "not required (no changes)"
                        }
                    );
                    println!("  Executable   no (preview-only; no apply command exists)");
                }
            }
            PlanCommand::Inspect { file, json } => {
                let plan = read_json::<MutationPlan>(&file)?;
                let inspection = inspect_plan(&plan);
                if json {
                    println!("{}", serde_json::to_string_pretty(&inspection)?);
                } else {
                    println!("Plan {}", inspection.declared_plan_id);
                    println!("  Valid        {}", inspection.valid);
                    println!("  Risk         {:?}", inspection.risk.level);
                    println!("  Executable   no");
                    for issue in inspection.issues {
                        println!("  Issue        {issue}");
                    }
                }
            }
            PlanCommand::Prepare { file, json } => {
                let plan = read_json::<MutationPlan>(&file)?;
                let inspection = inspect_plan(&plan);
                anyhow::ensure!(inspection.valid, "refusing to prepare an invalid plan");
                let bundle = compile_attended_bundle(&plan);
                if json {
                    println!("{}", serde_json::to_string_pretty(&bundle)?);
                } else {
                    println!("Attended bundle for {}", bundle.plan_id);
                    println!("  Plan tag     {}", bundle.plan_tag);
                    println!("  Operations   {}", bundle.operations.len());
                    println!("  Eligible     {}", bundle.eligible);
                    for blocker in bundle.blockers {
                        println!("  Blocker      {blocker}");
                    }
                    println!("  Device I/O   none (offline compilation only)");
                }
            }
        },
    }
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Result<T> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("could not read {}", path.display()))?;
    serde_json::from_str(&contents)
        .with_context(|| format!("could not parse {} as JSON", path.display()))
}

fn print_summary(value: &keysmith_core::Inspection) {
    println!("{} ({})", value.identity.name, value.identity.layout);
    println!(
        "  USB          {:04x}:{:04x} via {}",
        value.identity.vendor_id,
        value.identity.product_id,
        value.identity.path.display()
    );
    println!("  Firmware     {}", value.identity.firmware);
    println!(
        "  Protocols    VIA 0x{:04x}, Keychron {}, QMK command set {}",
        value.identity.via_protocol,
        value.identity.keychron_protocol,
        value.identity.qmk_command_set
    );
    println!(
        "  Active layer {} ({})",
        value.active_default_layer, value.layers[value.active_default_layer as usize].name
    );
    println!("  Features     {:?}", value.features);
    println!(
        "  Macros       {}/{} bytes used across {} slots",
        value.macros.used_bytes, value.macros.buffer_bytes, value.macros.slots
    );
    println!(
        "  Snap Click   {}/{} pairs configured",
        value.snap_click.configured_pairs, value.snap_click.pair_capacity
    );
    println!(
        "  Wireless     backlight {}s, sleep {}s",
        value.wireless_power.backlight_timeout_seconds, value.wireless_power.sleep_timeout_seconds
    );
    println!(
        "  Debounce     {} ms ({})",
        value.debounce.time_ms, value.debounce.algorithm
    );
    println!(
        "  RGB          brightness {}, effect {}, speed {}, hue/saturation {}/{}",
        value.rgb.brightness,
        value.rgb.effect,
        value.rgb.speed,
        value.rgb.hue,
        value.rgb.saturation
    );
    println!("  Write access disabled (preview-only milestone)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_tree_is_internally_consistent() {
        <Cli as clap::CommandFactory>::command().debug_assert();
    }
}
