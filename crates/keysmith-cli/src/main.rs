use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use keysmith_core::{
    ConfigurationSnapshot, MutationPlan, WriteMode, compile_attended_bundle,
    execute_bundle_connected, inspect_connected, inspect_plan, probe_connected,
};
use keysmith_core::{
    ConfigurationSnapshot as Snapshot, Scene, SceneRgb, cancel, open_keyboard, scenes, write_mode,
    write_state,
};

#[derive(Parser)]
#[command(
    name = "keychronctl",
    version,
    about = "Inspect and configure a Keychron Q3 Max. Configuration writes are direct on firmware 0.5+; firmware flashing is never reachable from here."
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
    /// Apply a verified plan to the connected keyboard.
    ///
    /// Compiles the plan, then runs prepare and commit for each operation in
    /// order. Firmware 0.5 or later commits on host request; a 0.3 board still
    /// demands the physical chord and this will say so rather than hanging.
    Apply {
        /// Path to a mutation plan produced by `plan create`.
        #[arg(long)]
        file: PathBuf,
        /// Compile and report what would be written, touching no hardware.
        #[arg(long)]
        dry_run: bool,
        /// Emit the machine-readable write receipt.
        #[arg(long)]
        json: bool,
    },
    /// Inspect or clear the firmware write gate.
    Write {
        #[command(subcommand)]
        command: WriteCommand,
    },
    /// Named, declarative keyboard states.
    ///
    /// A scene sets only the fields it names and leaves everything else alone.
    /// Applying one builds an ordinary plan against the live board, so it
    /// carries the same diff, risk and rollback evidence as a hand-built plan.
    Scene {
        #[command(subcommand)]
        command: SceneCommand,
    },
    /// Change one setting directly, without writing a scene file.
    ///
    /// Sugar over an unnamed scene: same planning, same rollback evidence.
    Set {
        #[command(subcommand)]
        command: SetCommand,
    },
    /// One-shot readiness report: what is connected and what is possible.
    ///
    /// Written for agents. Exits 0 when writes are possible, 1 when the board is
    /// reachable but read-only, and 2 when no board was found, so a caller can
    /// branch on the exit code without parsing anything.
    Doctor {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum SceneCommand {
    /// List every scene in the scene directory.
    List {
        #[arg(long)]
        json: bool,
    },
    /// Print one scene as stored.
    Show {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Save the board's current state as a scene.
    ///
    /// Captures RGB, debounce, wireless power and encoders. Deliberately not
    /// the keymap, so restoring a scene can never silently overwrite bindings.
    Capture {
        name: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show what applying a scene would change, touching no hardware.
    Diff {
        name: String,
        #[arg(long)]
        json: bool,
    },
    /// Apply a scene to the connected keyboard.
    Apply {
        name: String,
        /// Plan and report without writing.
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum SetCommand {
    /// Set any of the RGB fields. Unspecified fields are left alone.
    Rgb {
        #[arg(long)]
        brightness: Option<u8>,
        #[arg(long)]
        hue: Option<u8>,
        #[arg(long)]
        saturation: Option<u8>,
        #[arg(long)]
        speed: Option<u8>,
        /// Only meaningful on firmware compiling more than one effect.
        #[arg(long)]
        effect: Option<u8>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum WriteCommand {
    /// Report the gate state and whether this board commits directly.
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Discard any staged operation and return the board to locked.
    Cancel {
        #[arg(long)]
        json: bool,
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
        Command::Apply {
            file,
            dry_run,
            json,
        } => {
            let plan = read_json::<MutationPlan>(&file)?;
            let inspection = inspect_plan(&plan);
            anyhow::ensure!(inspection.valid, "refusing to apply an invalid plan");
            let bundle = compile_attended_bundle(&plan);
            anyhow::ensure!(
                bundle.eligible,
                "refusing to apply an ineligible bundle: {}",
                bundle.blockers.join("; ")
            );

            if dry_run {
                if json {
                    println!("{}", serde_json::to_string_pretty(&bundle)?);
                } else {
                    println!("Would apply {} ({} operations)", bundle.plan_id, bundle.operations.len());
                    for prepared in &bundle.operations {
                        println!("  [{}/{}] {:?}", prepared.index + 1, prepared.total, prepared.operation);
                    }
                    println!("  Device I/O   none (--dry-run)");
                }
                return Ok(());
            }

            let receipt = execute_bundle_connected(&bundle)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&receipt)?);
            } else {
                println!("Applied {}", receipt.plan_id);
                println!("  Plan tag     {}", receipt.plan_tag);
                println!("  Write mode   {:?}", receipt.mode);
                for entry in &receipt.operations {
                    println!("  [{}/{}] committed, board {:?}", entry.index + 1, entry.total, entry.final_state);
                }
            }
        }
        Command::Write { command } => match command {
            WriteCommand::Status { json } => {
                let (mut transport, _) = open_keyboard()?;
                let mode = write_mode(&mut transport)?;
                let state = write_state(&mut transport)?;
                if json {
                    println!(
                        "{}",
                        serde_json::json!({ "state": state, "mode": mode })
                    );
                } else {
                    println!("Write gate    {state:?}");
                    println!("Write mode    {mode:?}");
                    if mode == WriteMode::ChordRequired {
                        println!(
                            "  This board needs Esc + Space + Right Control held for three\n  \
                             seconds between prepare and commit. Firmware 0.5 or later\n  \
                             commits on host request."
                        );
                    }
                }
            }
            WriteCommand::Cancel { json } => {
                let (mut transport, _) = open_keyboard()?;
                let state = cancel(&mut transport)?;
                if json {
                    println!("{}", serde_json::json!({ "state": state }));
                } else {
                    println!("Write gate    {state:?}");
                }
            }
        },
        Command::Doctor { json } => return doctor(json),
        Command::Scene { command } => match command {
            SceneCommand::List { json } => {
                let entries = scenes::list();
                if json {
                    let ok: Vec<&Scene> = entries.iter().filter_map(|e| e.as_ref().ok()).collect();
                    println!("{}", serde_json::to_string_pretty(&ok)?);
                } else if entries.is_empty() {
                    println!("No scenes in {}", keysmith_core::scene_directory().display());
                    println!("Create one with: keychronctl scene capture <name>");
                } else {
                    for entry in &entries {
                        match entry {
                            Ok(scene) => println!(
                                "  {:<20} {}",
                                scene.name,
                                scene.description.as_deref().unwrap_or("")
                            ),
                            // Surfaced rather than skipped: a scene that fails to
                            // parse should not simply disappear from the list.
                            Err(error) => println!("  (unreadable) {error}"),
                        }
                    }
                }
            }
            SceneCommand::Show { name, json } => {
                let scene = scenes::load(&name)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&scene)?);
                } else {
                    println!("{}", serde_json::to_string_pretty(&scene)?);
                }
            }
            SceneCommand::Capture { name, description, json } => {
                scenes::validate_name(&name)?;
                let inspection = inspect_connected()?;
                let snapshot = Snapshot::from_inspection(&inspection);
                let scene = Scene::capture(&name, description, &snapshot);
                let path = scenes::save(&scene)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&scene)?);
                } else {
                    println!("Captured scene {name} to {}", path.display());
                }
            }
            SceneCommand::Diff { name, json } => {
                let scene = scenes::load(&name)?;
                let inspection = inspect_connected()?;
                let baseline = Snapshot::from_inspection(&inspection);
                let plan = scene.plan(&baseline)?;
                report_plan(&scene, &plan, &baseline, json, true)?;
            }
            SceneCommand::Apply { name, dry_run, json } => {
                let scene = scenes::load(&name)?;
                apply_scene(&scene, dry_run, json)?;
            }
        },
        Command::Set { command } => match command {
            SetCommand::Rgb {
                brightness,
                hue,
                saturation,
                speed,
                effect,
                dry_run,
                json,
            } => {
                let scene = Scene {
                    schema: keysmith_core::SCENE_SCHEMA.to_owned(),
                    name: "set-rgb".to_owned(),
                    description: Some("ad hoc RGB change".to_owned()),
                    rgb: SceneRgb { brightness, effect, speed, hue, saturation },
                    debounce_ms: None,
                    wireless: Default::default(),
                    encoders: Vec::new(),
                    keys: Vec::new(),
                };
                anyhow::ensure!(
                    scene.rgb != SceneRgb::default(),
                    "nothing to set; pass at least one of --brightness, --hue, --saturation, --speed or --effect"
                );
                apply_scene(&scene, dry_run, json)?;
            }
        },
    }
    Ok(())
}

/// Plan a scene against the live board and, unless this is a rehearsal, apply it.
fn apply_scene(scene: &Scene, dry_run: bool, json: bool) -> Result<()> {
    let inspection = inspect_connected()?;
    let baseline = Snapshot::from_inspection(&inspection);
    let plan = scene.plan(&baseline)?;

    if plan.diff().is_empty() {
        if json {
            println!("{}", serde_json::json!({ "changed": false, "reason": "board already matches" }));
        } else {
            println!("Nothing to do: the board already matches scene {}", scene.name);
        }
        return Ok(());
    }

    if dry_run {
        return report_plan(scene, &plan, &baseline, json, true);
    }

    report_plan(scene, &plan, &baseline, json, false)?;
    let bundle = compile_attended_bundle(&plan);
    anyhow::ensure!(
        bundle.eligible,
        "refusing to apply: {}",
        bundle.blockers.join("; ")
    );
    let receipt = execute_bundle_connected(&bundle)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&receipt)?);
    } else {
        println!("  Applied      {} operations, board {:?}", receipt.operations.len(), WriteMode::Direct);
    }
    Ok(())
}

fn report_plan(
    scene: &Scene,
    plan: &keysmith_core::MutationPlan,
    baseline: &Snapshot,
    json: bool,
    terminal: bool,
) -> Result<()> {
    if json {
        if terminal {
            println!("{}", serde_json::to_string_pretty(plan)?);
        }
        return Ok(());
    }
    println!("Scene {}", scene.name);
    println!("  Plan         {}", plan.plan_id());
    println!("  Risk         {:?}", plan.risk().level);
    for change in &plan.diff().changes {
        println!("  Change       {change:?}");
    }
    if scene.changes_effect(baseline) {
        println!(
            "  Warning      this scene moves the RGB effect index. A firmware that\n             \x20              compiles a single effect has no case for any other value\n             \x20              and will render nothing at all."
        );
    }
    if terminal {
        println!("  Device I/O   none (rehearsal only)");
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

/// Report everything a caller needs before deciding what it can do.
fn doctor(json: bool) -> Result<()> {
    let mut findings: Vec<(&str, String)> = Vec::new();
    let mut writable = false;

    let scene_dir = keysmith_core::scene_directory();
    let scene_count = scenes::list().iter().filter(|s| s.is_ok()).count();

    let transport = open_keyboard();
    let (mut transport, label) = match transport {
        Ok(pair) => pair,
        Err(error) => {
            // Not finding the board is the single most common state, and it is
            // usually not a fault: Raw HID is USB-only here, so a keyboard on
            // Bluetooth or 2.4 GHz is invisible by design.
            let report = serde_json::json!({
                "keyboard": "not found",
                "reason": error.to_string(),
                "hint": "Raw HID is USB-only on this board. Connect the cable and set the side switch to Cable.",
                "scene_directory": scene_dir,
                "scenes": scene_count,
                "writable": false,
            });
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("Keyboard      not found ({error})");
                println!("              Raw HID is USB-only on this board. Connect the cable");
                println!("              and set the side switch to Cable.");
                println!("Scenes        {scene_count} in {}", scene_dir.display());
            }
            std::process::exit(2);
        }
    };

    findings.push(("transport", label));

    let probe = keysmith_core::probe_keysmith(&mut transport)?;
    match &probe.protocol {
        Some(protocol) => {
            findings.push(("keysmith", format!("protocol {}.{}", protocol.major, protocol.minor)));
            if let Some(build) = &protocol.build {
                findings.push(("build", build.keysmith.clone()));
            }
        }
        None => findings.push((
            "keysmith",
            "not installed (stock Keychron firmware; reads work, writes do not)".to_owned(),
        )),
    }

    if probe.protocol.is_some() {
        let mode = write_mode(&mut transport)?;
        let state = write_state(&mut transport)?;
        writable = mode == WriteMode::Direct;
        findings.push(("write mode", format!("{mode:?}")));
        findings.push(("write gate", format!("{state:?}")));
        if !writable {
            findings.push((
                "note",
                "this board needs the physical Esc + Space + Right Control chord for every write"
                    .to_owned(),
            ));
        }
        if !matches!(state, keysmith_core::WriteState::Locked) {
            findings.push((
                "note",
                "a staged operation is pending; run `keychronctl write cancel`".to_owned(),
            ));
        }
    }

    findings.push(("scenes", format!("{scene_count} in {}", scene_dir.display())));

    if json {
        let map: serde_json::Map<String, serde_json::Value> = findings
            .iter()
            .map(|(k, v)| ((*k).to_owned(), serde_json::Value::String(v.clone())))
            .chain(std::iter::once((
                "writable".to_owned(),
                serde_json::Value::Bool(writable),
            )))
            .collect();
        println!("{}", serde_json::to_string_pretty(&map)?);
    } else {
        for (key, value) in &findings {
            println!("{key:<14}{value}");
        }
    }

    if writable { Ok(()) } else { std::process::exit(1) }
}
