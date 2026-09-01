import { CheckCircle2, ChevronRight, FileJson, Info, Layers3, LockKeyhole, Radio, ShieldCheck, SquareM } from "lucide-react";
import { useMemo, useState } from "react";
import { PROJECT_EVIDENCE } from "../data/projectEvidence";
import { useDraft } from "../draft/DraftProvider";
import { keycodeName } from "../keycodes";
import type { FirmwareProbeState, Inspection, SelectedKey } from "../types";
import type { InspectionStatus } from "../useInspection";
import { KeyboardCanvas } from "./KeyboardCanvas";
import { ResourceFreshness } from "./ResourceFreshness";

export function OverviewPage({ inspection, firmwareProbe, status, observedAt, onRefresh, onReview, onNavigate, className = "" }: {
  inspection: Inspection;
  firmwareProbe: FirmwareProbeState;
  status: InspectionStatus;
  observedAt: number | null;
  onRefresh: () => void;
  onReview: () => void;
  onNavigate: (page: "Firmware" | "Activity") => void;
  className?: string;
}) {
  const { plan } = useDraft();
  const [activeLayer, setActiveLayer] = useState(inspection.active_default_layer);
  const [selected, setSelected] = useState<SelectedKey>(() => {
    const code = inspection.layers[inspection.active_default_layer]?.matrix[0]?.[0] ?? 0x29;
    return { row: 0, col: 0, code, label: keycodeName(code) };
  });
  const layer = inspection.layers[activeLayer];
  const groups = useMemo(() => new Set(plan?.changes.map((change) => change.group) ?? []).size, [plan]);
  const maxRisk = plan?.changes.some((change) => change.risk === "high") ? "High" : plan?.changes.some((change) => change.risk === "medium") ? "Medium" : "Low";
  const protocol = firmwareProbe.probe?.protocol;
  const protocolVersion = protocol ? `${protocol.major}.${protocol.minor}` : "unknown";
  const extensionDetected = firmwareProbe.probe?.installed === true;
  const liveExtensionStatus = firmwareProbe.loading && !firmwareProbe.probe
    ? "Checking"
    : firmwareProbe.error
      ? "Probe unavailable"
      : extensionDetected
        ? "Detected live"
        : "Not detected";
  const liveExtensionName = protocol?.build?.keysmith ?? "Keysmith extension";
  const liveExtensionDetail = extensionDetected && protocol
    ? `Protocol ${protocolVersion} reported over USB`
    : firmwareProbe.error
      ? "Live firmware evidence is unavailable"
      : firmwareProbe.loading
        ? "Checking the connected device"
        : "Not reported by the connected device";
  const liveExtensionSource = protocol?.build?.qmk_git_hash ? `Source ${protocol.build.qmk_git_hash}` : "Source not reported";

  function changeLayer(next: number) {
    setActiveLayer(next);
    const code = inspection.layers[next]?.matrix[selected.row]?.[selected.col] ?? 0;
    setSelected((current) => ({ ...current, code, label: keycodeName(code) }));
  }

  return (
    <main className={`overview-workspace ${className}`}>
      <header className="control-heading">
        <div><div className="title-line"><h1>Control Center</h1><span><CheckCircle2 />Live read-only</span></div><p>Inspect the connected board and stage local drafts. Nothing here can write to the keyboard.</p></div>
        <button className="device-details" onClick={() => onNavigate("Activity")}><Info />Device details</button>
      </header>
      <ResourceFreshness inspection={inspection} firmwareProbe={firmwareProbe} status={status} observedAt={observedAt} onRefresh={onRefresh} />

      <div className="overview-grid">
        <div className="overview-primary">
          <section className="overview-panel keyboard-overview">
            <div className="overview-panel-title"><div><h2>Active layer: {layer.name}{activeLayer === inspection.active_default_layer ? " (Default)" : ""}</h2><span>Read-only</span></div><label>Layer<select value={activeLayer} onChange={(event) => changeLayer(Number(event.target.value))}>{inspection.layers.map((item) => <option value={item.index} key={item.index}>{item.name}</option>)}</select></label></div>
            <KeyboardCanvas layer={layer} selected={selected} onSelect={setSelected} />
            <div className="key-legend"><span>□ Standard</span><span className="legend-purple">■ Layer/Tap</span><span className="legend-green">■ Modifier</span><span className="legend-yellow">■ Macro</span><span className="legend-blue">■ Lighting</span></div>
          </section>

          <div className="overview-mini-grid">
            <section className="overview-panel mini-panel"><h2>Selected key</h2><div className="selected-key-compact"><kbd>{selected.label}</kbd><span><small>Matrix</small><strong>{selected.row}, {selected.col}</strong></span><ChevronRight /></div></section>
            <section className="overview-panel mini-panel"><h2>Configuration draft <span>{plan?.changes.length ?? 0} entries</span></h2><button className="draft-summary" onClick={onReview} disabled={!plan}><span><small>Grouped scopes</small><strong>{groups}</strong></span><span><small>Highest risk</small><strong className="medium-text">{plan ? maxRisk : "None"}</strong></span><ChevronRight /></button></section>
          </div>

          <section className="overview-panel pending-plan">
            <div className="overview-panel-title"><div><h2>Pending change plan</h2><span>{groups} grouped scopes · {plan?.changes.length ?? 0} atomic entries</span></div>{plan ? <code>{plan.planId}</code> : null}</div>
            {plan ? <>
              <div className="plan-groups">{Array.from(new Set(plan.changes.map((change) => change.group))).map((group) => <div key={group}><strong>{group}</strong><span>{plan.changes.filter((change) => change.group === group).map((change) => `${change.target}: ${change.before} → ${change.after}`).join(" · ")}</span></div>)}</div>
              <div className="plan-footer"><span>Rollback evidence: {plan.source.label}</span><button onClick={onReview}>Review transaction</button></div>
            </> : <div className="empty-plan"><span>No local draft. Stage a change from Keymap, Lighting, Wireless, or Diagnostics.</span></div>}
            <div className="hardware-warning"><LockKeyhole /><span><strong>Browser and agent paths never apply hardware changes.</strong> Attended terminal confirmation is required for any future write.</span></div>
          </section>
        </div>

        <aside className="overview-secondary">
          <section className="overview-panel evidence-panel">
            <div className="overview-panel-title"><div><h2>Recent activity</h2><span>Checked-in static evidence</span></div><button onClick={() => onNavigate("Activity")}>View all</button></div>
            <div className="activity-list">{PROJECT_EVIDENCE.activity.map((item) => <div key={item.title}><i /><time>{item.time}</time><span><strong>{item.title}</strong><small>{item.detail}</small></span><em>{item.type}</em></div>)}</div>
          </section>
          <section className="overview-panel evidence-panel">
            <div className="overview-panel-title"><div><h2>Recovery evidence classes</h2><span>Reference requirements</span></div><button onClick={() => onNavigate("Firmware")}>View all</button></div>
            <div className="snapshot-list">{PROJECT_EVIDENCE.snapshots.map((item) => <div key={item.name}><FileJson /><span><strong>{item.name}</strong><small>{item.detail}</small></span><time>{item.time}</time></div>)}</div>
          </section>
          <section className="overview-panel firmware-track">
            <h2>Firmware track</h2>
            <div><strong>{liveExtensionName}</strong><span className={extensionDetected ? "installed-label" : "candidate-label"}>{liveExtensionStatus}</span><small>{liveExtensionDetail}</small><em>{liveExtensionSource}</em></div>
            <div><strong>Keysmith 0.3.0-candidate</strong><span className="candidate-label">Public candidate</span><small>Source bbbd4fac3c · CI build validated</small><em>No binary release</em></div>
            <p><Info />Firmware flashing remains attended-terminal-only. No web flash operations.</p>
          </section>
          <section className="evidence-source"><ShieldCheck /><span><strong>Rollback boundary</strong><small>Operator-supplied snapshots stay private</small></span></section>
        </aside>
      </div>
    </main>
  );
}
