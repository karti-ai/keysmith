import { ChevronDown, Copy, LockKeyhole } from "lucide-react";
import { useEffect, useState } from "react";
import { qmkCode } from "../keycodes";
import type { SelectedKey } from "../types";
import { useDraft } from "../draft/DraftProvider";

interface Props {
  selected: SelectedKey;
  layer: number;
  layerName: string;
  onPreview: (message: string) => void;
}

const QUICK_CODES: Array<[string, number]> = [["⌘", 0x00e3], ["⌥", 0x00e2], ["⌃", 0x00e0], ["⇧", 0x00e1], ["Esc", 0x0029], ["Enter", 0x0028], ["Tab", 0x002b], ["Space", 0x002c]];

export function KeyInspector({ selected, layer, layerName, onPreview }: Props) {
  const { replaceScope } = useDraft();
  const [keycode, setKeycode] = useState(selected.code);
  useEffect(() => setKeycode(selected.code), [selected.code, selected.col, selected.row, layer]);
  function stage() {
    replaceScope("keymap", `Stage ${layerName} matrix ${selected.row},${selected.col} as keycode ${keycode}.`, keycode === selected.code ? [] : [{ id: `key-${layer}-${selected.row}-${selected.col}`, scope: "keymap", group: "Keymap key", target: `${layerName} · ${selected.label}`, before: `${qmkCode(selected.code)} (${selected.code})`, after: `${qmkCode(keycode)} (${keycode})`, risk: "high", storage: "Dynamic keymap", layer: layerName, row: selected.row, column: selected.col, operation: { kind: "keycode", layer, row: selected.row, column: selected.col, keycode }, rollbackComplete: true, executionSupport: "v0.3-attended" }]);
    onPreview("Key assignment saved as a browser-local transaction draft. No keyboard write was sent.");
  }
  return (
    <section className="key-inspector" aria-label="Selected key inspector">
      <div className="selected-key-summary">
        <span className="section-label">Selected key</span>
        <strong>{selected.label}</strong>
        <div className="keycap-preview">{selected.label.length > 7 ? selected.label.slice(0, 7) : selected.label}</div>
        <span className="matrix-address">Matrix {selected.row},{selected.col}</span>
      </div>
      <div className="assignment-editor">
        <label>Assigned action</label>
        <div className="select-control">
          <input aria-label="Assigned QMK keycode" type="number" min="0" max="65535" value={keycode} onChange={(event) => setKeycode(Number(event.target.value))} />
          <code>{qmkCode(keycode)}</code>
          <Copy size={15} />
          <ChevronDown size={16} />
        </div>
        <label>Tap / Hold behavior</label>
        <div className="behavior-row">
          <button className="select-control compact"><span>Tap · {selected.label}</span><ChevronDown size={15} /></button>
          <button className="select-control compact"><span>Hold · No action</span><ChevronDown size={15} /></button>
        </div>
        <div className="quick-codes">
          <span>Quick codes</span>
          {QUICK_CODES.map(([label, code]) => <button key={label} onClick={() => setKeycode(code)}>{label}</button>)}
        </div>
        <button className="primary-button inspector-apply" disabled={keycode === selected.code} onClick={stage}>
          <LockKeyhole size={15} />Stage key draft
        </button>
      </div>
    </section>
  );
}
