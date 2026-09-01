import { CheckCircle2, Clock3, RefreshCw, TriangleAlert } from "lucide-react";
import type { FirmwareProbeState, Inspection } from "../types";
import type { InspectionStatus } from "../useInspection";

export function ResourceFreshness({ inspection, firmwareProbe, status, observedAt, onRefresh }: {
  inspection: Inspection;
  firmwareProbe: FirmwareProbeState;
  status: InspectionStatus;
  observedAt: number | null;
  onRefresh: () => void;
}) {
  const usb = `${inspection.identity.vendor_id.toString(16).padStart(4, "0")}:${inspection.identity.product_id.toString(16).padStart(4, "0")}`;
  const protocol = firmwareProbe.probe?.installed && firmwareProbe.probe.protocol
    ? `${firmwareProbe.probe.protocol.major}.${firmwareProbe.probe.protocol.minor}`
    : "Unavailable";
  const live = status === "live" && !firmwareProbe.error;
  const observed = observedAt ? new Intl.DateTimeFormat(undefined, { hour: "numeric", minute: "2-digit", second: "2-digit" }).format(observedAt) : "not observed";
  return (
    <section className={`resource-strip ${live ? "is-live" : "is-stale"}`} aria-label="Unified device resource freshness">
      <div className="resource-cell"><span>Device</span><strong>{inspection.identity.name}</strong></div>
      <div className="resource-cell"><span>Connection</span><strong className="safe-value"><i />USB {usb}</strong></div>
      <div className="resource-cell"><span>Firmware</span><strong>{inspection.identity.firmware.split(" ")[0]}</strong></div>
      <div className="resource-cell"><span>Keysmith protocol</span><strong>{protocol}</strong></div>
      <div className="resource-cell"><span>Raw HID</span><strong className={live ? "safe-value" : "warn-value"}>{live ? <CheckCircle2 /> : <TriangleAlert />}{live ? "Healthy" : "Check"}</strong></div>
      <div className="resource-cell"><span>Write policy</span><strong className="blocked-value">Blocked</strong></div>
      <div className="resource-freshness">
        <span>{live ? <CheckCircle2 /> : <Clock3 />}{live ? "Live read-only" : "Last known data"}</span>
        <small>Observed {observed}. All edits stay local.</small>
        <button onClick={onRefresh} disabled={status === "refreshing" || firmwareProbe.loading}><RefreshCw className={status === "refreshing" ? "spin" : ""} />Refresh both</button>
      </div>
    </section>
  );
}
