import { Command, RefreshCw } from "lucide-react";
import type { Inspection } from "../types";
import type { InspectionStatus } from "../useInspection";
import "../live-state.css";

interface Props {
  inspection: Inspection;
  loading: boolean;
  status?: InspectionStatus;
  observedAt?: number | null;
  error?: string | null;
  onRefresh: () => void;
  onPalette: () => void;
  protocolVersion?: string;
}

const statusLabels: Record<InspectionStatus, string> = {
  loading: "Inspecting",
  live: "Connected",
  refreshing: "Refreshing",
  stale: "Stale data",
  disconnected: "Disconnected",
};

function formatObservedAt(observedAt: number | null | undefined) {
  if (!observedAt) return null;
  return new Intl.DateTimeFormat(undefined, {
    hour: "numeric",
    minute: "2-digit",
    second: "2-digit",
  }).format(new Date(observedAt));
}

export function Topbar({ inspection, loading, status, observedAt, error, onRefresh, onPalette, protocolVersion }: Props) {
  const { identity } = inspection;
  const usbId = `${identity.vendor_id.toString(16).padStart(4, "0")}:${identity.product_id.toString(16).padStart(4, "0")}`;
  const firmware = identity.firmware.split(" ")[0];
  const effectiveStatus = status ?? (loading
    ? (inspection.connected ? "refreshing" : "loading")
    : (inspection.connected ? "live" : "disconnected"));
  const observedTime = formatObservedAt(observedAt);
  const isRefreshing = effectiveStatus === "loading" || effectiveStatus === "refreshing";
  const statusTitle = [
    observedTime ? `Last successful inspection: ${observedTime}` : null,
    error ? `Latest inspection failed: ${error}` : null,
  ].filter(Boolean).join("\n") || undefined;

  return (
    <header className="topbar">
      <div className="device-title">Q3 Max</div>
      <div
        className={`connected connection-status status-${effectiveStatus}`}
        role="status"
        aria-live="polite"
        aria-atomic="true"
        title={statusTitle}
      >
        <span className="status-dot" aria-hidden="true" />
        <span>{statusLabels[effectiveStatus]}</span>
        {observedTime && effectiveStatus === "stale" ? <span className="status-time">· {observedTime}</span> : null}
        {error ? <span className="visually-hidden">Latest inspection failed: {error}</span> : null}
      </div>
      <div className="top-divider" />
      <div className="device-meta usb-meta">USB · {usbId}</div>
      <div className="top-divider" />
      <div className="device-meta">{firmware}{protocolVersion ? ` · KS ${protocolVersion}` : ""}</div>
      <button
        className="refresh-button"
        onClick={onRefresh}
        aria-label={isRefreshing ? "Refreshing keyboard inspection" : "Refresh keyboard inspection"}
        aria-busy={isRefreshing}
        disabled={isRefreshing}
      >
        <RefreshCw className={isRefreshing ? "spin" : ""} size={16} aria-hidden="true" />
      </button>
      <button className="command-button" onClick={onPalette}>
        <Command size={16} />
        <span>Command palette</span>
        <kbd>⌘K</kbd>
      </button>
    </header>
  );
}
