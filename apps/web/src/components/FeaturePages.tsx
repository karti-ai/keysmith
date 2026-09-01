import { useMemo, useState } from "react";
import {
  Archive,
  Bluetooth,
  Cable,
  Check,
  CheckCircle2,
  ChevronDown,
  CircleOff,
  Clock3,
  Cpu,
  FileJson,
  GitFork,
  HardDriveDownload,
  Info,
  Lightbulb,
  LockKeyhole,
  Plus,
  Radio,
  RotateCcw,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
  Trash2,
  Usb,
  Waypoints,
  Wifi,
  Zap,
} from "lucide-react";
import type { CSSProperties, ReactNode } from "react";
import type { DraftChange, FirmwareProbeState, Inspection, Page } from "../types";
import { FirmwareProbeCard } from "./FirmwareProbeCard";
import { useDraft } from "../draft/DraftProvider";

interface Props {
  page: Page;
  inspection: Inspection;
  onPreview: (message: string) => void;
  firmwareProbe: FirmwareProbeState;
}

interface MetricProps {
  label: string;
  value: ReactNode;
  detail?: string;
}

const FACTORY_HASH = "210a4f41…de15a9a";
const SNAPSHOT_HASH = "5a6eb3fe…c9614d";
const V02_HASH = "19c88b034495ed9281ef06158bb6395f5165bc304922de4eb3884827136eb196";
const V03_HASH = "836c74cbaa72119d228368f8b3b37dbe94438d79d121db1ff0789da48f408277";

function Metric({ label, value, detail }: MetricProps) {
  return (
    <div className="metric-row">
      <span>{label}</span>
      <strong>{value}</strong>
      {detail ? <small>{detail}</small> : null}
    </div>
  );
}

function PageHeading({ title, description }: { title: string; description: string }) {
  return (
    <header className="feature-heading">
      <div>
        <h1>{title}</h1>
        <p>{description}</p>
      </div>
      <div className="read-only-mark"><LockKeyhole size={13} />Preview mode</div>
    </header>
  );
}

function Panel({ title, icon, className = "", children }: { title: string; icon?: ReactNode; className?: string; children: ReactNode }) {
  return (
    <section className={`feature-panel ${className}`}>
      <div className="panel-title">{icon}<h2>{title}</h2></div>
      {children}
    </section>
  );
}

function PreviewButton({ children, onPreview, secondary = false }: { children: ReactNode; onPreview: () => void; secondary?: boolean }) {
  return <button className={secondary ? "secondary-button" : "primary-button"} onClick={onPreview}>{secondary ? null : <LockKeyhole size={15} />}{children}</button>;
}

function SafetyActions({ onPreview, onStage, previewLabel = "Preview local draft", stageDisabled = false }: { onPreview: () => void; onStage: () => void; previewLabel?: string; stageDisabled?: boolean }) {
  return (
    <div className="safety-actions">
      <PreviewButton onPreview={onPreview}>{previewLabel}</PreviewButton>
      <button className="apply-disabled" onClick={onStage} disabled={stageDisabled} title="Save this change as a browser-local draft">Stage draft</button>
      <span><ShieldCheck size={13} />No Raw HID write will be sent</span>
    </div>
  );
}

const LIGHT_ROWS = [14, 14, 14, 13, 13, 10];

function LightingPage({ inspection, onPreview }: Omit<Props, "page">) {
  const source = inspection.rgb;
  const { replaceScope } = useDraft();
  const [effect, setEffect] = useState(source.effect);
  const [brightness, setBrightness] = useState(source.brightness);
  const [speed, setSpeed] = useState(source.speed);
  const [hue, setHue] = useState(source.hue);
  const [saturation, setSaturation] = useState(source.saturation);
  const [perKey, setPerKey] = useState(false);
  const dirty = effect !== source.effect || brightness !== source.brightness || speed !== source.speed || hue !== source.hue || saturation !== source.saturation || perKey;
  const lightStyle = {
    "--preview-hue": `${Math.round((hue / 255) * 360)}deg`,
    "--preview-light": `${Math.round(22 + (brightness / 255) * 48)}%`,
    "--preview-sat": `${Math.round((saturation / 255) * 100)}%`,
    "--preview-speed": `${(2.8 - (speed / 255) * 2.1).toFixed(2)}s`,
  } as CSSProperties;

  function reset() {
    setEffect(source.effect);
    setBrightness(source.brightness);
    setSpeed(source.speed);
    setHue(source.hue);
    setSaturation(source.saturation);
    setPerKey(false);
  }

  function preview() {
    const fields = [
      effect !== source.effect ? `effect ${source.effect} → ${effect}` : null,
      brightness !== source.brightness ? `brightness ${source.brightness} → ${brightness}` : null,
      speed !== source.speed ? `speed ${source.speed} → ${speed}` : null,
      hue !== source.hue ? `hue ${source.hue} → ${hue}` : null,
      saturation !== source.saturation ? `saturation ${source.saturation} → ${saturation}` : null,
      perKey ? "per-key canvas enabled locally" : null,
    ].filter(Boolean);
    onPreview(fields.length ? `Local RGB draft: ${fields.join(" · ")}. No EEPROM write was sent.` : "RGB draft matches the live inspection. No EEPROM write was sent.");
  }

  function stage() {
    const after = { effect, brightness, speed, hue, saturation };
    const fields = (["effect", "brightness", "speed", "hue", "saturation"] as const).filter((field) => after[field] !== source[field]);
    const changes: DraftChange[] = fields.map((field) => ({ id: `rgb-${field}`, scope: "lighting", group: "RGB profile", target: `RGB ${field}`, before: source[field], after: after[field], risk: "low", storage: "RGB EEPROM", operation: { kind: "rgb_profile", ...after }, rollbackComplete: true, executionSupport: "v0.3-attended" }));
    if (perKey) changes.push({ id: "rgb-per-key-canvas", scope: "lighting", group: "Per-key canvas", target: "Per-key color canvas", before: "Not captured", after: "Local exploration enabled", risk: "high", storage: "Browser draft only", operation: { kind: "rgb_profile", ...after }, rollbackComplete: false, executionSupport: "draft-only" });
    replaceScope("lighting", "Stage the reviewed RGB profile.", changes);
    onPreview(`${changes.length} RGB draft ${changes.length === 1 ? "entry" : "entries"} saved locally. No keyboard write was sent.`);
  }

  return (
    <>
      <PageHeading title="Lighting" description="Shape a local RGB draft from the live Keychron values, then inspect the difference before any future write." />
      <div className="lighting-layout">
        <Panel title="Effect settings" icon={<SlidersHorizontal size={16} />} className="controls-panel">
          <div className="effect-identity"><span>Effect ID</span><input aria-label="Effect ID draft" type="number" min="1" max="255" value={effect} onChange={(event) => setEffect(Number(event.target.value))} /><small>Inspected: {source.effect}</small></div>
          <RangeControl label="Brightness" value={brightness} onChange={setBrightness} />
          <RangeControl label="Speed" value={speed} onChange={setSpeed} />
          <RangeControl label="Hue" value={hue} onChange={setHue} rainbow />
          <RangeControl label="Saturation" value={saturation} onChange={setSaturation} />
          <button className="text-button" onClick={reset} disabled={!dirty}><RotateCcw size={14} />Reset to inspected values</button>
        </Panel>
        <div className="lighting-main">
          <Panel title="Effect preview" icon={<Lightbulb size={16} />} className="preview-panel">
            <div className="lighting-preview" style={lightStyle} aria-label="Local lighting effect preview">
              {LIGHT_ROWS.map((count, row) => <div className="light-row" key={row}>{Array.from({ length: count }, (_, column) => <i key={column} style={{ "--light-index": row * 15 + column } as CSSProperties} />)}</div>)}
            </div>
            <div className="preview-legend"><span><i />Inspection source</span><span>Local rendering · effect ID {effect}</span></div>
          </Panel>
          <Panel title="Per-key lighting" icon={<Sparkles size={16} />} className="action-panel">
            <div><p>Explore individual key colors in this browser. The board remains unchanged.</p><small>Per-key editing is a draft surface, not a stock-firmware write.</small></div>
            <button className={perKey ? "toggle-button active" : "toggle-button"} aria-pressed={perKey} onClick={() => setPerKey((current) => !current)}><span />{perKey ? "Canvas enabled" : "Enable canvas"}</button>
          </Panel>
        </div>
      </div>
      <SafetyActions onPreview={preview} onStage={stage} stageDisabled={!dirty} previewLabel={dirty ? "Preview RGB diff" : "Review inspected state"} />
    </>
  );
}

function RangeControl({ label, value, onChange, rainbow = false }: { label: string; value: number; onChange: (value: number) => void; rainbow?: boolean }) {
  return <label className="range-control"><span>{label}<output>{value}</output></span><input aria-label={`${label} draft`} className={rainbow ? "rainbow-range" : ""} type="range" min="0" max="255" value={value} onChange={(event) => onChange(Number(event.target.value))} /></label>;
}

type MacroAction = { id: number; kind: "Keystroke" | "Delay"; value: string };

function MacrosPage({ inspection, onPreview }: Omit<Props, "page">) {
  const { replaceScope } = useDraft();
  const slots = useMemo(() => Array.from({ length: inspection.macros.slots }, (_, index) => index + 1), [inspection.macros.slots]);
  const [selectedSlot, setSelectedSlot] = useState(1);
  const [name, setName] = useState("");
  const [actions, setActions] = useState<MacroAction[]>([]);

  function addAction(kind: MacroAction["kind"]) {
    setActions((current) => [...current, { id: Date.now() + current.length, kind, value: kind === "Delay" ? "100 ms" : "KC_A" }]);
  }

  function selectSlot(slot: number) {
    setSelectedSlot(slot);
    setName("");
    setActions([]);
  }

  function stage() {
    const change: DraftChange = { id: `macro-${selectedSlot}`, scope: "macro", group: "Macro sequence", target: `Macro slot ${selectedSlot}`, before: "Contents intentionally private", after: `${name || "Untitled"} · ${actions.length} actions`, risk: "critical", storage: "Macro EEPROM", operation: { kind: "macro", slot: selectedSlot, name, actions: actions.map(({ kind, value }) => ({ kind, value })) }, rollbackComplete: false, executionSupport: "draft-only" };
    replaceScope("macro", `Draft macro slot ${selectedSlot}; do not execute without a complete private rollback capture.`, [change]);
    onPreview(`Macro ${selectedSlot} saved as a blocked local draft. Its contents were not read from or written to the keyboard.`);
  }

  return (
    <>
      <PageHeading title="Macros" description={`${inspection.macros.slots} firmware slots · ${inspection.macros.used_bytes} of ${inspection.macros.buffer_bytes} bytes used in the inspection.`} />
      <div className="macro-layout">
        <Panel title="Macro slots" icon={<Archive size={16} />} className="macro-slots">
          <div className="slot-list" role="listbox" aria-label="Macro slots">
            {slots.map((slot) => <button role="option" aria-selected={selectedSlot === slot} className={selectedSlot === slot ? "selected" : ""} key={slot} onClick={() => selectSlot(slot)}><span>{slot}</span><strong>{inspection.macros.used_bytes === 0 ? "Empty" : "Inspect"}</strong><small>{inspection.macros.used_bytes === 0 ? "0 B" : "—"}</small></button>)}
          </div>
        </Panel>
        <Panel title="Selected macro" icon={<Zap size={16} />} className="macro-editor">
          <div className="macro-fields"><label>Slot<input value={selectedSlot} readOnly /></label><label className="name-field">Draft name<input placeholder="Untitled local draft" value={name} onChange={(event) => setName(event.target.value)} /></label></div>
          <div className="sequence-title"><span>Draft sequence</span><div><button onClick={() => addAction("Keystroke")}><Plus size={13} />Keystroke</button><button onClick={() => addAction("Delay")}><Clock3 size={13} />Delay</button></div></div>
          {actions.length === 0 ? <div className="sequence-empty"><CircleOff size={32} /><strong>No draft actions</strong><span>Add an action to build a local, reversible preview.</span></div> : (
            <div className="sequence-list">{actions.map((action, index) => <div key={action.id}><span>{index + 1}</span><strong>{action.kind}</strong><input value={action.value} onChange={(event) => setActions((current) => current.map((item) => item.id === action.id ? { ...item, value: event.target.value } : item))} aria-label={`${action.kind} value`} /><button onClick={() => setActions((current) => current.filter((item) => item.id !== action.id))} aria-label={`Remove ${action.kind}`}><Trash2 size={14} /></button></div>)}</div>
          )}
          <div className="editor-footer"><span>{actions.length} draft actions · browser only</span><button className="text-button" disabled={actions.length === 0 && !name} onClick={() => { setActions([]); setName(""); }}><RotateCcw size={13} />Clear</button></div>
        </Panel>
      </div>
      <SafetyActions onPreview={() => onPreview(actions.length ? `Macro ${selectedSlot} draft contains ${actions.length} local actions. No macro buffer write was sent.` : `Macro ${selectedSlot} has no local draft actions. No macro buffer write was sent.`)} onStage={stage} stageDisabled={actions.length === 0 && !name} previewLabel="Preview macro diff" />
    </>
  );
}

const CONNECTIONS = [
  { id: "bt1", Icon: Bluetooth, name: "Bluetooth host 1", state: "Pairing state unavailable", kind: "Bluetooth" },
  { id: "bt2", Icon: Bluetooth, name: "Bluetooth host 2", state: "Pairing state unavailable", kind: "Bluetooth" },
  { id: "bt3", Icon: Bluetooth, name: "Bluetooth host 3", state: "Pairing state unavailable", kind: "Bluetooth" },
  { id: "rf", Icon: Radio, name: "2.4 GHz receiver", state: "Supported · receiver unavailable", kind: "2.4 GHz" },
  { id: "usb", Icon: Usb, name: "USB cable", state: "Connected", kind: "USB" },
] as const;

function WirelessPage({ inspection, onPreview }: Omit<Props, "page">) {
  const { replaceScope } = useDraft();
  const power = inspection.wireless_power;
  const [selectedMode, setSelectedMode] = useState("bt1");
  const [backlight, setBacklight] = useState(power.backlight_timeout_seconds);
  const [sleep, setSleep] = useState(power.sleep_timeout_seconds);
  const selected = CONNECTIONS.find((item) => item.id === selectedMode) ?? CONNECTIONS[0];
  const dirty = backlight !== power.backlight_timeout_seconds || sleep !== power.sleep_timeout_seconds;

  function stage() {
    const changes: DraftChange[] = [];
    const operation = { kind: "wireless_power" as const, backlight_timeout_seconds: backlight, sleep_timeout_seconds: sleep };
    if (backlight !== power.backlight_timeout_seconds) changes.push({ id: "wireless-backlight-timeout", scope: "wireless", group: "Wireless power", target: "Backlight timeout", before: power.backlight_timeout_seconds, after: backlight, risk: "high", storage: "Wireless EEPROM", operation, rollbackComplete: true, executionSupport: "v0.3-attended" });
    if (sleep !== power.sleep_timeout_seconds) changes.push({ id: "wireless-sleep-timeout", scope: "wireless", group: "Wireless power", target: "Sleep timeout", before: power.sleep_timeout_seconds, after: sleep, risk: "high", storage: "Wireless EEPROM", operation, rollbackComplete: true, executionSupport: "v0.3-attended" });
    replaceScope("wireless", "Stage reviewed wireless power timeouts; transport selection remains physical.", changes);
    onPreview(`${changes.length} wireless timeout ${changes.length === 1 ? "entry" : "entries"} saved locally. No keyboard write was sent.`);
  }

  return (
    <>
      <PageHeading title="Wireless" description="Inspect transport support, follow physical pairing steps, and stage timeout changes without sending them." />
      <div className="wireless-layout">
        <Panel title="Connection modes" icon={<Wifi size={16} />} className="wireless-list">
          {CONNECTIONS.map(({ id, Icon, name, state, kind }) => <button className={selectedMode === id ? "wireless-row selected" : "wireless-row"} key={id} onClick={() => setSelectedMode(id)}><span className="connection-icon"><Icon size={16} /></span><span><strong>{name}</strong><small>{kind}</small></span><span className={state === "Connected" ? "connection-state connected-state" : "connection-state"}>{state === "Connected" ? <Check size={13} /> : null}{state}</span><ChevronDown size={14} /></button>)}
        </Panel>
        <div className="wireless-side">
          <Panel title="Pairing guide" icon={<Bluetooth size={16} />} className="pairing-guide">
            <strong>{selected.name}</strong>
            {selected.kind === "Bluetooth" ? <ol><li>Move the hardware switch to Bluetooth.</li><li>Hold the matching host shortcut for more than 2 seconds.</li><li>Finish pairing from the host operating system.</li></ol> : selected.kind === "USB" ? <p>USB is the active inspection path. Keysmith configuration remains USB-only.</p> : <p>The board supports 2.4 GHz, but the receiver is currently unavailable. Replacement receiver compatibility must be verified first.</p>}
            <p className="inline-note"><Info size={14} />Current source does not expose arbitrary Keysmith Raw HID configuration over Bluetooth.</p>
          </Panel>
          <Panel title="Quick facts" icon={<Cable size={16} />} className="quick-facts"><Metric label="Bluetooth hosts" value="3" /><Metric label="Configuration path" value="USB only" /><Metric label="Host state" value="Not exposed" /></Panel>
        </div>
      </div>
      <Panel title="Wireless timeouts" icon={<Clock3 size={16} />} className="timeout-panel">
        <label>Backlight timeout <span>seconds</span><input type="number" min="0" value={backlight} onChange={(event) => setBacklight(Number(event.target.value))} /></label>
        <label>Sleep timeout <span>seconds</span><input type="number" min="0" value={sleep} onChange={(event) => setSleep(Number(event.target.value))} /></label>
        <button className="text-button" disabled={!dirty} onClick={() => { setBacklight(power.backlight_timeout_seconds); setSleep(power.sleep_timeout_seconds); }}><RotateCcw size={13} />Reset to inspected values</button>
      </Panel>
      <SafetyActions onPreview={() => onPreview(dirty ? `Wireless timeout draft: backlight ${power.backlight_timeout_seconds} → ${backlight}s · sleep ${power.sleep_timeout_seconds} → ${sleep}s. No setting was written.` : "Wireless timeout draft matches the inspection. No setting was written.")} onStage={stage} stageDisabled={!dirty} previewLabel="Preview timeout diff" />
    </>
  );
}

function DiagnosticsPage({ inspection, onPreview }: Omit<Props, "page">) {
  const { replaceScope } = useDraft();
  const [algorithm, setAlgorithm] = useState(inspection.debounce.algorithm_id);
  const [debounceMs, setDebounceMs] = useState(inspection.debounce.time_ms);
  const [encoderLayer, setEncoderLayer] = useState(0);
  const encoder = inspection.encoders[encoderLayer];
  const [ccw, setCcw] = useState(encoder?.counter_clockwise ?? 0);
  const [cw, setCw] = useState(encoder?.clockwise ?? 0);
  const [snapMode, setSnapMode] = useState(0);
  const [snapA, setSnapA] = useState(26);
  const [snapB, setSnapB] = useState(7);
  function selectEncoderLayer(layer: number) { setEncoderLayer(layer); setCcw(inspection.encoders[layer]?.counter_clockwise ?? 0); setCw(inspection.encoders[layer]?.clockwise ?? 0); }
  function stage() {
    const changes: DraftChange[] = [];
    if (algorithm !== inspection.debounce.algorithm_id || debounceMs !== inspection.debounce.time_ms) changes.push({ id: "diagnostics-debounce", scope: "diagnostics", group: "Switch processing", target: "Debounce configuration", before: `${inspection.debounce.algorithm_id} · ${inspection.debounce.time_ms} ms`, after: `${algorithm} · ${debounceMs} ms`, risk: "high", storage: "Debounce EEPROM", operation: { kind: "debounce", algorithm_id: algorithm, time_ms: debounceMs }, rollbackComplete: true, executionSupport: "v0.3-attended" });
    if (encoder && ccw !== encoder.counter_clockwise) changes.push({ id: `encoder-${encoderLayer}-ccw`, scope: "diagnostics", group: "Encoder bindings", target: `Layer ${encoderLayer} counter-clockwise`, before: encoder.counter_clockwise, after: ccw, risk: "medium", storage: "Dynamic keymap", operation: { kind: "encoder", layer: encoderLayer, clockwise: false, keycode: ccw }, rollbackComplete: true, executionSupport: "v0.3-attended" });
    if (encoder && cw !== encoder.clockwise) changes.push({ id: `encoder-${encoderLayer}-cw`, scope: "diagnostics", group: "Encoder bindings", target: `Layer ${encoderLayer} clockwise`, before: encoder.clockwise, after: cw, risk: "medium", storage: "Dynamic keymap", operation: { kind: "encoder", layer: encoderLayer, clockwise: true, keycode: cw }, rollbackComplete: true, executionSupport: "v0.3-attended" });
    if (snapMode !== 0) changes.push({ id: "snap-click-0", scope: "diagnostics", group: "Snap Click", target: "Snap Click pair 1", before: "Pair definition not captured", after: `mode ${snapMode}: ${snapA} / ${snapB}`, risk: "critical", storage: "Snap Click EEPROM", operation: { kind: "snap_click", pair: 0, mode: snapMode, keycode_a: snapA, keycode_b: snapB }, rollbackComplete: false, executionSupport: "draft-only" });
    replaceScope("diagnostics", "Stage reviewed switch-processing and encoder changes.", changes);
    onPreview(`${changes.length} advanced-control ${changes.length === 1 ? "entry" : "entries"} saved locally. No keyboard write was sent.`);
  }
  const dirty = algorithm !== inspection.debounce.algorithm_id || debounceMs !== inspection.debounce.time_ms || Boolean(encoder && (ccw !== encoder.counter_clockwise || cw !== encoder.clockwise)) || snapMode !== 0;
  return <><PageHeading title="Diagnostics" description="Protocol health plus transaction-ready switch and encoder controls." /><div className="diagnostic-grid"><Panel title="Protocols" icon={<Waypoints size={16} />}><Metric label="VIA" value={inspection.identity.via_protocol} /><Metric label="Keychron" value={inspection.identity.keychron_protocol} /><Metric label="QMK command set" value={inspection.identity.qmk_command_set} /></Panel><Panel title="Debounce" icon={<SlidersHorizontal size={16} />}><Metric label="Algorithm" value={inspection.debounce.algorithm} /><Metric label="Time" value={`${inspection.debounce.time_ms} ms`} detail="Live inspected value" /></Panel><Panel title="Snap Click" icon={<Sparkles size={16} />}><Metric label="Pair capacity" value={inspection.snap_click.pair_capacity} /><Metric label="Configured" value={inspection.snap_click.configured_pairs} /></Panel><Panel title="Raw HID" icon={<Cable size={16} />}><Metric label="Path" value={inspection.identity.path} /><Metric label="USB" value="3434:0830" /><Metric label="Server writes" value={inspection.write_enabled ? "Enabled" : "Blocked"} /></Panel></div><Panel title="Advanced controls" icon={<SlidersHorizontal size={16} />} className="timeout-panel"><label>Debounce algorithm <input aria-label="Debounce algorithm ID" type="number" min="0" max="6" value={algorithm} onChange={(event) => setAlgorithm(Number(event.target.value))} /></label><label>Debounce time <span>ms</span><input aria-label="Debounce time" type="number" min="0" max="255" value={debounceMs} onChange={(event) => setDebounceMs(Number(event.target.value))} /></label><label>Encoder layer <select value={encoderLayer} onChange={(event) => selectEncoderLayer(Number(event.target.value))}>{inspection.encoders.map((item) => <option key={item.layer} value={item.layer}>Layer {item.layer}</option>)}</select></label><label>Counter-clockwise keycode <input type="number" min="0" max="65535" value={ccw} onChange={(event) => setCcw(Number(event.target.value))} /></label><label>Clockwise keycode <input type="number" min="0" max="65535" value={cw} onChange={(event) => setCw(Number(event.target.value))} /></label><label>Snap Click mode <select value={snapMode} onChange={(event) => setSnapMode(Number(event.target.value))}><option value="0">No local draft</option><option value="1">Last input wins</option><option value="2">Neutral</option></select></label>{snapMode ? <><label>Pair key A <input type="number" min="1" max="255" value={snapA} onChange={(event) => setSnapA(Number(event.target.value))} /></label><label>Pair key B <input type="number" min="1" max="255" value={snapB} onChange={(event) => setSnapB(Number(event.target.value))} /></label></> : null}</Panel><SafetyActions onPreview={() => onPreview(dirty ? "Advanced-control diff is ready for transaction review." : "Advanced controls match the live inspection.")} onStage={stage} stageDisabled={!dirty} previewLabel="Preview advanced diff" /></>;
}

function AgentPage({ onPreview }: Pick<Props, "onPreview">) {
  return <><PageHeading title="Agent" description="Natural-language requests become transparent local drafts, never unattended device actions." /><Panel title="Illustrative keymap draft" icon={<Sparkles size={17} />} className="agent-workspace"><div className="agent-query">Make Caps Lock Escape when tapped and Control when held.</div><div className="agent-plan"><span>1</span><div><strong>Read current keymap</strong><p>Confirm the live layer, matrix coordinate, and original keycode.</p></div><span>2</span><div><strong>Draft and validate</strong><p>Illustrative QMK mapping: <code>MT(MOD_LCTL, KC_ESC)</code>. Validate against the target firmware before any future write.</p></div><span>3</span><div><strong>Stop at review</strong><p>This build creates no persisted rollback transaction and exposes no write command.</p></div></div><PreviewButton onPreview={() => onPreview("Agent draft reviewed locally. No keyboard command was sent.")}>Review local draft</PreviewButton></Panel></>;
}

function FirmwarePage({ inspection, onPreview, firmwareProbe }: Omit<Props, "page">) {
  const firmwareParts = inspection.identity.firmware.split(" ");
  const version = firmwareParts[0];
  const built = firmwareParts.slice(1).join(" ") || "2025-04-23 11:57";
  const installed = firmwareProbe.probe?.installed === true;
  const protocol = firmwareProbe.probe?.protocol;
  const v03Installed = Boolean(installed && protocol && (protocol.major > 0 || protocol.minor >= 3));
  const gateState = protocol?.write_status?.state ?? "unknown";
  const installedState = v03Installed ? `Installed · protocol 0.3 · gate ${gateState}` : "Archived · previous build";
  const evidencePreview = v03Installed
    ? `Keysmith v0.3 image 836c74cb…408277 is installed from source 2b422a5a12. Its live physical write gate is ${gateState}; the browser and server expose no apply route.`
    : "Keysmith v0.3 is not detected. Firmware flashing remains attended-terminal-only.";
  return (
    <>
      <PageHeading title="Firmware" description="Keep recovery evidence visible. A custom image cannot be flashed from the web application." />
      <div className="firmware-layout">
        <div className="firmware-grid">
          <Panel title="Current firmware" icon={<Cpu size={16} />} className="firmware-card"><Metric label="Version" value={version} /><Metric label="Build" value={built} /><Metric label="MCU" value="STM32F401" /><Metric label="Bootloader" value="stm32-dfu" /></Panel>
          <Panel title="Factory image" icon={<HardDriveDownload size={16} />} className="firmware-card"><p className="safe-line"><CheckCircle2 size={16} />Official ANSI image archived</p><code>{FACTORY_HASH}</code><small>Recovery source for the original factory v1.1.1 firmware.</small></Panel>
          <Panel title="Configuration snapshot" icon={<FileJson size={16} />} className="firmware-card"><p className="safe-line"><CheckCircle2 size={16} />Pre-flash inspection saved</p><code>{SNAPSHOT_HASH}</code><small>Layers, macros, RGB, wireless policy, debounce, and encoder bindings.</small></Panel>
          <Panel title="Installed custom image" icon={<ShieldCheck size={16} />} className="firmware-card candidate-card"><p className="candidate-state"><CheckCircle2 size={15} />{installedState}</p><Metric label="Source commit" value="2b422a5a12" /><code>{V03_HASH}</code><small>One plan-bound operation per physical chord, with visible Esc status. Legacy write bypasses are denied.</small><PreviewButton secondary onPreview={() => onPreview(evidencePreview)}>Review v0.3 boundary</PreviewButton></Panel>
          <Panel title="Previous v0.2 image" icon={<GitFork size={16} />} className="firmware-card"><p className="candidate-state"><Archive size={15} />Archived · rollback available</p><Metric label="Source commit" value="e9972e1a43" /><code>{V02_HASH}</code><small>The earlier read-only protocol 0.2 image remains preserved with its exact source and checksum.</small><PreviewButton secondary onPreview={() => onPreview("Keysmith v0.2 image 19c88b03…eb196 remains archived as the previous read-only build.")}>Review v0.2 evidence</PreviewButton></Panel>
        </div>
        <FirmwareProbeCard probe={firmwareProbe.probe} error={firmwareProbe.error} loading={firmwareProbe.loading} onRefresh={firmwareProbe.refresh} />
        <Panel title="Recovery checklist" icon={<ShieldCheck size={16} />} className="recovery-panel">
          <ol><li className="done"><Check size={13} /><span><strong>Factory image archived</strong><small>Checksum recorded twice</small></span></li><li className="done"><Check size={13} /><span><strong>Configuration saved</strong><small>Pre- and post-flash JSON snapshots</small></span></li><li className="done"><Check size={13} /><span><strong>v0.3 image verified</strong><small>Source and checksum recorded</small></span></li><li className={v03Installed ? "done" : ""}>{v03Installed ? <Check size={13} /> : <span>4</span>}<span><strong>Physical recovery access</strong><small>{v03Installed ? "Confirmed during attended flash" : "Confirm at flash time"}</small></span></li><li className={v03Installed ? "done" : ""}>{v03Installed ? <Check size={13} /> : <span>5</span>}<span><strong>Explicit human approval</strong><small>{v03Installed ? "Recorded for exact image 836c74cb" : "Required after terminal preview"}</small></span></li></ol>
        </Panel>
      </div>
      <section className="feature-panel flash-gate"><ShieldCheck size={22} /><div><h2>Attended terminal only</h2><p>Keysmith deliberately has no web flash control. A human must review the exact image, recovery steps, and device state at the local terminal.</p></div><strong>Web flashing unavailable</strong></section>
      <section className="feature-panel firmware-compatibility"><Info size={20} /><div><h2>Installed v0.3 · physical gate {gateState}</h2><p>Image <code>2b422a5a12</code> is verified live with stable VIA marker <code>260831</code>. The complete configuration matches the pre-v0.3 snapshot; web apply remains unavailable.</p></div><strong>v0.3 verified · config preserved</strong></section>
    </>
  );
}

function SettingsPage({ inspection }: Omit<Props, "page" | "onPreview">) {
  const endpoint = globalThis.location?.host || "127.0.0.1:3762";
  return <><PageHeading title="Settings" description="Local service, network access, and physical-device safety boundaries." /><div className="settings-list"><Panel title="Current endpoint" icon={<Wifi size={16} />}><p>Keysmith binds to loopback. Optional authenticated remote access should terminate in a trusted private network proxy.</p><code>{endpoint}</code><strong className="safe-text">Same origin</strong></Panel><Panel title="Server mutation policy" icon={<LockKeyhole size={16} />}><p>Keymap, macro, RGB EEPROM, wireless, and firmware writes require a reviewed plan and confirmation.</p><strong>{inspection.write_enabled ? "Enabled" : "Blocked"}</strong></Panel><Panel title="Recovery storage" icon={<Archive size={16} />}><p>Keep factory firmware, live snapshots, and device readbacks outside the source repository in private backup storage.</p><strong className="safe-text">Operator managed</strong></Panel></div></>;
}

export function FeatureWorkspace({ page, inspection, onPreview, firmwareProbe }: Props) {
  return <main className="workspace feature-workspace">{page === "Lighting" ? <LightingPage inspection={inspection} onPreview={onPreview} firmwareProbe={firmwareProbe} /> : null}{page === "Macros" ? <MacrosPage inspection={inspection} onPreview={onPreview} firmwareProbe={firmwareProbe} /> : null}{page === "Wireless" ? <WirelessPage inspection={inspection} onPreview={onPreview} firmwareProbe={firmwareProbe} /> : null}{page === "Diagnostics" ? <DiagnosticsPage inspection={inspection} onPreview={onPreview} firmwareProbe={firmwareProbe} /> : null}{page === "Agent" ? <AgentPage onPreview={onPreview} /> : null}{page === "Firmware" ? <FirmwarePage inspection={inspection} onPreview={onPreview} firmwareProbe={firmwareProbe} /> : null}{page === "Settings" ? <SettingsPage inspection={inspection} firmwareProbe={firmwareProbe} /> : null}</main>;
}

const RAIL_COPY: Record<Exclude<Page, "Keymap" | "Overview" | "Activity">, { title: string; body: string; items: Array<[string, string]> }> = {
  Lighting: { title: "Lighting status", body: "Values begin from the live stock-firmware inspection. Browser drafts never touch RGB EEPROM.", items: [["Effect ID", "Live"], ["Per-key canvas", "Local"], ["Server writes", "Blocked"]] },
  Macros: { title: "About macros", body: "Macro totals and storage usage come from the connected inspection. Draft actions are browser-local.", items: [["Slots", "Inspected"], ["Buffer", "Inspected"], ["Server writes", "Blocked"]] },
  Wireless: { title: "Transport boundary", body: "Pairing is a physical keyboard workflow. Current source exposes Keysmith configuration over USB only.", items: [["Bluetooth hosts", "3"], ["2.4 receiver", "Unavailable"], ["USB", "Connected"]] },
  Diagnostics: { title: "Inspection finding", body: "The reported 50 ms per-key symmetric eager debounce is intentional in factory firmware v1.1.1.", items: [["Raw HID", "Healthy"], ["Macros", "Inspected"], ["Snap Click", "Inspected"]] },
  Agent: { title: "Safety model", body: "The agent may inspect, explain, snapshot, and draft diffs. It cannot silently mutate the keyboard.", items: [["Inspect", "Allowed"], ["Plan", "Allowed"], ["Flash", "Human-only"]] },
  Firmware: { title: "Engineering gate", body: "Firmware state comes from a live read-only probe. Flashing remains an attended terminal operation.", items: [["Factory image", "Saved"], ["Snapshots", "Saved"], ["Custom image", "Checking"]] },
  Settings: { title: "Trust boundary", body: "Tailscale authenticates network access. Device mutation remains a separate privileged operation.", items: [["Network", "Tailnet"], ["Server", "Loopback"], ["Mutation", "Blocked"]] },
};

export function FeatureRail({ page, inspection, firmwareProbe }: Props) {
  if (page === "Keymap" || page === "Overview" || page === "Activity") return null;
  const base = RAIL_COPY[page];
  const firmwareState = firmwareProbe.loading && !firmwareProbe.probe ? "Checking" : firmwareProbe.error ? "Unavailable" : firmwareProbe.probe?.installed ? "Installed" : "Not installed";
  const items = page === "Macros"
    ? [["Total slots", String(inspection.macros.slots)], ["Used bytes", String(inspection.macros.used_bytes)], ["Server writes", inspection.write_enabled ? "Enabled" : "Blocked"]]
    : page === "Firmware"
      ? [["Factory image", "Saved"], ["Snapshots", "Saved"], ["Installed image", firmwareState]]
      : base.items;
  return <aside className="agent-panel feature-rail"><div className="agent-heading"><Lightbulb size={18} /><span>{base.title}</span></div><p>{base.body}</p><div className="rail-list">{items.map(([label, value]) => <div key={label}><span>{label}</span><strong>{value}</strong></div>)}</div><p className="safety-note"><LockKeyhole size={14} />No keyboard write on this page</p></aside>;
}
