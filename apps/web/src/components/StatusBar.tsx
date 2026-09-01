import { CheckCircle2, Layers3, Radio, ShieldAlert, ShieldCheck, SquareM } from "lucide-react";
import type { Inspection } from "../types";
import "../live-state.css";

export function StatusBar({ inspection, activeLayer }: { inspection: Inspection; activeLayer: number }) {
  const viewedLayerName = inspection.layers[activeLayer]?.name ?? `Layer ${activeLayer}`;
  const defaultLayerName = inspection.layers[inspection.active_default_layer]?.name ?? `Layer ${inspection.active_default_layer}`;
  const mutationPolicy = inspection.write_enabled
    ? "Server writes enabled · preview and confirmation required"
    : "Server read-only · previews only";
  const macroUsage = inspection.macros.used_bytes === 0
    ? `${inspection.macros.slots} macro slots · none used`
    : `${inspection.macros.used_bytes} of ${inspection.macros.buffer_bytes} macro bytes used`;

  return (
    <footer className="statusbar" aria-label="Keyboard inspection status">
      <div><CheckCircle2 size={17} className="ok" />Factory snapshot saved</div>
      <div role="status" aria-live="polite" aria-atomic="true">
        <Layers3 size={17} aria-hidden="true" />
        <span>Viewing {viewedLayerName} · device default {defaultLayerName}</span>
      </div>
      <div><SquareM size={17} aria-hidden="true" />{macroUsage}</div>
      <div className="status-spacer" />
      <div className={inspection.write_enabled ? "policy-warning" : "policy-safe"}>
        {inspection.write_enabled ? <ShieldAlert size={16} aria-hidden="true" /> : <ShieldCheck size={16} aria-hidden="true" />}
        <span>{mutationPolicy}</span>
      </div>
      <div><Radio size={16} className="ok" />Raw HID · local</div>
    </footer>
  );
}
