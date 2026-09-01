import { Bot, ChevronDown, Eye, LockKeyhole, Sparkles } from "lucide-react";
import { useState } from "react";
import type { AgentPreviewPlan, FirmwareProbeState } from "../types";
import "../agentic.css";
import { FirmwareProbeCard } from "./FirmwareProbeCard";

interface Props {
  onPreview: (message: string) => void;
  firmwareProbe: FirmwareProbeState;
}

const DEFAULT_REQUEST = "Make Caps Lock Escape when tapped and Control when held.";

function createPreviewPlan(request: string): AgentPreviewPlan {
  const normalized = request.toLowerCase();

  if (/firmware|flash|qmk/.test(normalized)) {
    return {
      intent: "firmware",
      title: "Firmware engineering review",
      summary: "Prepare evidence and recovery artifacts without producing an unattended flash action.",
      reads: ["Probe the installed Keysmith protocol", "Record device identity and current firmware", "Verify factory image and checksum"],
      draft: ["Compare the candidate build with the validated board target", "Generate a human-readable build and recovery manifest"],
      guardrails: ["No DFU transition", "No flash command", "Physical presence and a separate confirmation are mandatory"],
    };
  }
  if (/bluetooth|wireless|pair|2\.4/.test(normalized)) {
    return {
      intent: "wireless",
      title: "Wireless setup preview",
      summary: "Inspect transport state and produce a host-side pairing guide.",
      reads: ["Read USB and wireless runtime status", "Check the active host slot if firmware exposes it"],
      draft: ["Describe the required physical host-key step", "Prepare Mac or Linux host commands for review"],
      guardrails: ["No pairing command is sent", "No Bluetooth keys or host slots are changed"],
    };
  }
  if (/light|rgb|color|brightness/.test(normalized)) {
    return {
      intent: "lighting",
      title: "Lighting change preview",
      summary: "Model the requested lighting state as an explicit RGB diff.",
      reads: ["Read effect, brightness, speed, hue, and saturation", "Persist a rollback snapshot before any future write"],
      draft: ["Resolve the requested effect into explicit values", "Render a before/after preview"],
      guardrails: ["No RGB EEPROM write", "Preview values stay local to this browser"],
    };
  }
  if (/macro|sequence|shortcut/.test(normalized)) {
    return {
      intent: "macro",
      title: "Macro compilation preview",
      summary: "Compile the request into an inspectable sequence without touching macro storage.",
      reads: ["Read slot usage and buffer capacity", "Persist the selected slot as rollback data before a future write"],
      draft: ["Expand the request into press, release, and delay events", "Calculate the exact byte budget"],
      guardrails: ["No macro buffer write", "Existing slot data remains unchanged"],
    };
  }
  return {
    intent: "keymap",
    title: "Keymap change preview",
    summary: "Translate the request into a draft key assignment for live-state validation.",
    reads: ["Read the active layer and current matrix position", "Persist the original keycode before a future write"],
    draft: ["Illustrative mapping: KC_CAPS → MT(MOD_LCTL, KC_ESC)", "Validate the tap/hold behavior against the target QMK build"],
    guardrails: ["No keymap write", "No firmware compile or flash", "A later mutation requires an explicit reviewed confirmation"],
  };
}

export function AgentPanel({ onPreview, firmwareProbe }: Props) {
  const [expanded, setExpanded] = useState(true);
  const [request, setRequest] = useState(DEFAULT_REQUEST);
  const [plan, setPlan] = useState<AgentPreviewPlan>(() => createPreviewPlan(DEFAULT_REQUEST));

  function generatePlan() {
    const nextPlan = createPreviewPlan(request.trim() || DEFAULT_REQUEST);
    setPlan(nextPlan);
    setExpanded(true);
    onPreview(`${nextPlan.title} generated locally. No keyboard command was sent.`);
  }

  return (
    <aside className="agent-panel agentic-panel">
      <div className="agent-heading"><Bot size={19} /><span>Ask Keysmith</span><button type="button" onClick={() => setExpanded((value) => !value)} aria-expanded={expanded} aria-label="Toggle agent preview"><ChevronDown className={expanded ? "agent-chevron open" : "agent-chevron"} size={17} /></button></div>

      <label className="agent-request-label" htmlFor="agent-request">Describe the outcome</label>
      <textarea id="agent-request" className="agent-request" value={request} onChange={(event) => setRequest(event.target.value)} rows={3} />
      <button type="button" className="agent-generate" onClick={generatePlan}><Sparkles size={15} />Generate local preview</button>

      <div className="proposal-heading"><Eye size={17} /><span>Transparent plan</span><small>{plan.intent}</small></div>
      {expanded ? (
        <div className="agent-plan-preview">
          <div className="agent-plan-summary"><strong>{plan.title}</strong><p>{plan.summary}</p></div>
          <PlanGroup index="01" title="Read first" items={plan.reads} />
          <PlanGroup index="02" title="Draft locally" items={plan.draft} />
          <PlanGroup index="03" title="Hard stops" items={plan.guardrails} safe />
        </div>
      ) : null}

      <div className="agent-actions">
        <button className="secondary-button" type="button" onClick={() => setExpanded((value) => !value)}>{expanded ? "Hide details" : "Review plan"}</button>
        <button className="primary-button preview-only-button" type="button" onClick={() => onPreview(`${plan.title} reviewed. Hardware writes remain blocked.`)}><Eye size={15} />Preview only</button>
      </div>
      <p className="safety-note"><LockKeyhole size={14} />This agent can inspect and draft. It cannot silently mutate or flash the keyboard.</p>

      <FirmwareProbeCard probe={firmwareProbe.probe} error={firmwareProbe.error} loading={firmwareProbe.loading} onRefresh={firmwareProbe.refresh} />
    </aside>
  );
}

function PlanGroup({ index, title, items, safe = false }: { index: string; title: string; items: string[]; safe?: boolean }) {
  return (
    <section className={safe ? "agent-plan-group guarded" : "agent-plan-group"}>
      <span className="agent-plan-index">{index}</span>
      <div><h3>{title}</h3><ul>{items.map((item) => <li key={item}>{item}</li>)}</ul></div>
    </section>
  );
}
