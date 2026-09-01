import { Bot, Command, Cpu, LockKeyhole, RefreshCw, Waypoints, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { fetchFirmwareProbe } from "../useFirmwareProbe";

interface Props {
  onClose: () => void;
  onRefresh: () => void;
  onNotify: (message: string) => void;
  onNavigate: (page: "Overview" | "Diagnostics" | "Firmware" | "Activity") => void;
  onReview: () => void;
}

const FOCUSABLE_SELECTOR = "button:not(:disabled), [href], input, select, textarea, [tabindex]:not([tabindex='-1'])";

export function CommandPalette({ onClose, onRefresh, onNotify, onNavigate, onReview }: Props) {
  const dialog = useRef<HTMLDivElement>(null);
  const firstAction = useRef<HTMLButtonElement>(null);
  const returnFocus = useRef<HTMLElement | null>(document.activeElement instanceof HTMLElement ? document.activeElement : null);
  const [probing, setProbing] = useState(false);

  useEffect(() => {
    firstAction.current?.focus();
    const previous = returnFocus.current;
    return () => previous?.focus();
  }, []);

  function trapFocus(event: React.KeyboardEvent<HTMLDivElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      onClose();
      return;
    }
    if (event.key !== "Tab" || !dialog.current) return;

    const focusable = Array.from(dialog.current.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  async function probeFirmware() {
    setProbing(true);
    try {
      const probe = await fetchFirmwareProbe();
      const message = probe.installed && probe.protocol
        ? `Keysmith protocol ${probe.protocol.major}.${probe.protocol.minor} detected; mutation bits 0x${probe.protocol.mutation_capabilities.toString(16).padStart(2, "0")}, physical gate ${probe.protocol.write_status?.state ?? "not reported"}.`
        : "Stock Keychron firmware detected; Keysmith extension is not installed.";
      onNotify(message);
    } catch (reason) {
      onNotify(`Firmware probe unavailable: ${reason instanceof Error ? reason.message : String(reason)}`);
    }
    onClose();
  }

  return (
    <div className="palette-backdrop" role="presentation" onMouseDown={onClose}>
      <div ref={dialog} className="palette agentic-palette" role="dialog" aria-modal="true" aria-label="Command palette" onKeyDown={trapFocus} onMouseDown={(event) => event.stopPropagation()}>
        <div className="palette-title"><Command size={18} /><span>Commands</span><small>read-only</small><button type="button" onClick={onClose} aria-label="Close command palette"><X size={17} /></button></div>
        <div className="palette-section-label">Device</div>
        <button ref={firstAction} type="button" onClick={() => { onNavigate("Overview"); onClose(); }}><Command size={17} /><span><b>Open Control Center</b><small>Return to the read-only overview</small></span></button>
        <button type="button" onClick={() => { onRefresh(); onClose(); }}><RefreshCw size={17} /><span><b>Inspect device</b><small>Read live Q3 Max state again</small></span><kbd>↵</kbd></button>
        <button type="button" disabled={probing} onClick={() => void probeFirmware()}><Cpu className={probing ? "spin" : ""} size={17} /><span><b>{probing ? "Probing firmware…" : "Probe firmware protocol"}</b><small>Check namespace 0xAC without changing state</small></span></button>
        <button type="button" onClick={() => { onNavigate("Diagnostics"); onClose(); }}><Waypoints size={17} /><span><b>Show diagnostics</b><small>Inspect protocol and switch processing</small></span></button>
        <div className="palette-section-label">Agent</div>
        <button type="button" onClick={() => { onReview(); onClose(); }}><Bot size={17} /><span><b>Review pending transaction</b><small>Deterministic local diff, export, or discard</small></span></button>
        <footer><LockKeyhole size={13} /><span>Commands may inspect or draft. Mutations and firmware flashing are unavailable here.</span></footer>
      </div>
    </div>
  );
}
