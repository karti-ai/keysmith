import { Cable, Check, Cpu, LoaderCircle, LockKeyhole, RefreshCw, TriangleAlert } from "lucide-react";
import type { FirmwareProbe } from "../types";

interface Props {
  probe: FirmwareProbe | null;
  error: string | null;
  loading: boolean;
  onRefresh: () => Promise<void>;
}

export function FirmwareProbeCard({ probe, error, loading, onRefresh }: Props) {
  const protocol = probe?.protocol;
  const writeStatus = protocol?.write_status;
  const mutationState = writeStatus
    ? writeStatus.state === "locked" ? "Physical gate locked" : `Physical gate ${writeStatus.state}`
    : protocol?.mutation_capabilities ? "Attended capability present" : "Locked (bits = 0)";

  return (
    <section className="firmware-probe" aria-live="polite" aria-busy={loading}>
      <header>
        <div className="firmware-probe-title"><Cpu size={16} /><span>Firmware protocol</span></div>
        <button type="button" onClick={() => void onRefresh()} disabled={loading} aria-label="Probe firmware protocol again">
          <RefreshCw className={loading ? "spin" : ""} size={14} />
        </button>
      </header>

      {loading && !probe ? (
        <div className="firmware-probe-state muted"><LoaderCircle className="spin" size={17} /><span>Sending one read-only probe…</span></div>
      ) : error ? (
        <div className="firmware-probe-state warning"><TriangleAlert size={17} /><span><strong>Probe unavailable</strong><small>{error}</small></span></div>
      ) : probe?.installed && protocol ? (
        <>
          <div className="firmware-probe-state safe"><Check size={17} /><span><strong>Keysmith protocol {protocol.major}.{protocol.minor}</strong><small>{protocol.packet_bytes}-byte Raw HID packets · mutation bits 0x{protocol.mutation_capabilities.toString(16).padStart(2, "0")}</small></span></div>
          <dl className="firmware-probe-facts">
            <div><dt>Transport</dt><dd><Cable size={13} />{protocol.runtime.transport}</dd></div>
            <div><dt>Wireless</dt><dd>{protocol.runtime.wireless_state}</dd></div>
            <div><dt>USB power</dt><dd>{protocol.runtime.usb_power ? "Present" : "Absent"}</dd></div>
            <div><dt>Mutations</dt><dd className="locked"><LockKeyhole size={12} />{mutationState}</dd></div>
          </dl>
          <p>{protocol.usb_only ? "Configuration channel: USB only." : "Configuration transport reported by firmware."} {writeStatus ? `Live gate state: ${writeStatus.state}; USB ${writeStatus.usb_ready ? "ready" : "not ready"}. ` : ""}This panel cannot write to the device.</p>
        </>
      ) : (
        <>
          <div className="firmware-probe-state stock"><Cable size={17} /><span><strong>Stock Keychron firmware</strong><small>Keysmith extension not installed</small></span></div>
          <p>The keyboard answered safely. Custom protocol commands and firmware mutations remain unavailable.</p>
        </>
      )}
    </section>
  );
}
