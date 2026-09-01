import { Q3_ROWS, type KeySpec } from "../keyLayout";
import { shortKeycodeName } from "../keycodes";
import type { KeymapLayer, SelectedKey } from "../types";

interface Props {
  layer: KeymapLayer;
  selected: SelectedKey;
  onSelect: (key: SelectedKey) => void;
}

function KeyboardKey({ spec, layer, selected, onSelect }: { spec: KeySpec } & Props) {
  if (spec.kind === "knob") {
    return <div className="knob" style={{ marginLeft: `calc(var(--unit) * ${spec.gapBefore ?? 0})` }} aria-label="Rotary knob" />;
  }
  const code = layer.matrix[spec.row]?.[spec.col] ?? 0;
  const label = shortKeycodeName(code);
  const isSelected = selected.row === spec.row && selected.col === spec.col;
  return (
    <button
      className={isSelected ? "keyboard-key selected" : "keyboard-key"}
      style={{
        width: `calc(var(--unit) * ${spec.width ?? 1} + var(--key-gap) * ${(spec.width ?? 1) - 1})`,
        marginLeft: `calc(var(--unit) * ${spec.gapBefore ?? 0})`,
      }}
      onClick={() => onSelect({ row: spec.row, col: spec.col, code, label })}
      title={`${label} · matrix ${spec.row},${spec.col} · 0x${code.toString(16).padStart(4, "0")}`}
    >
      <span>{label}</span>
    </button>
  );
}

export function KeyboardCanvas(props: Props) {
  return (
    <div className="keyboard-shell">
      <div className="keyboard-board" aria-label={`${props.layer.name} keyboard layer`}>
        {Q3_ROWS.map((row, rowIndex) => (
          <div className="keyboard-row" key={rowIndex}>
            {row.map((spec) => <KeyboardKey {...props} spec={spec} key={spec.id} />)}
          </div>
        ))}
      </div>
    </div>
  );
}

