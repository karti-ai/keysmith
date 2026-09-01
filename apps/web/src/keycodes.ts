const NAMED = new Map<number, string>([
  [0x0000, "None"],
  [0x0001, "Transparent"],
  [0x0028, "Enter"],
  [0x0029, "Escape"],
  [0x002a, "Backspace"],
  [0x002b, "Tab"],
  [0x002c, "Space"],
  [0x002d, "−"],
  [0x002e, "="],
  [0x002f, "["],
  [0x0030, "]"],
  [0x0031, "\\"],
  [0x0033, ";"],
  [0x0034, "'"],
  [0x0035, "`"],
  [0x0036, ","],
  [0x0037, "."],
  [0x0038, "/"],
  [0x0039, "Caps Lock"],
  [0x0046, "Print"],
  [0x0049, "Insert"],
  [0x004a, "Home"],
  [0x004b, "PgUp"],
  [0x004c, "Delete"],
  [0x004d, "End"],
  [0x004e, "PgDn"],
  [0x004f, "→"],
  [0x0050, "←"],
  [0x0051, "↓"],
  [0x0052, "↑"],
  [0x0068, "F13"],
  [0x00a8, "Mute"],
  [0x00a9, "Volume +"],
  [0x00aa, "Volume −"],
  [0x00ab, "Next"],
  [0x00ac, "Previous"],
  [0x00ae, "Play / Pause"],
  [0x00bd, "Brightness +"],
  [0x00be, "Brightness −"],
  [0x00e0, "Control"],
  [0x00e1, "Shift"],
  [0x00e2, "Alt"],
  [0x00e3, "Command"],
  [0x00e4, "Right Ctrl"],
  [0x00e5, "Right Shift"],
  [0x00e6, "Right Alt"],
  [0x00e7, "Right GUI"],
  [0x5221, "Fn · Mac"],
  [0x5223, "Fn · Win"],
  [0x7820, "RGB toggle"],
  [0x7821, "RGB mode"],
  [0x7822, "RGB reverse"],
  [0x7823, "Hue +"],
  [0x7824, "Hue −"],
  [0x7825, "Saturation +"],
  [0x7826, "Saturation −"],
  [0x7827, "RGB brightness +"],
  [0x7828, "RGB brightness −"],
  [0x7829, "RGB speed +"],
  [0x782a, "RGB speed −"],
  [0x7e00, "Option"],
  [0x7e01, "Right Option"],
  [0x7e02, "Command"],
  [0x7e03, "Right Command"],
  [0x7e04, "Mission Control"],
  [0x7e05, "Launchpad"],
  [0x7e06, "Task View"],
  [0x7e07, "File Explorer"],
  [0x7e08, "Screenshot"],
  [0x7e0a, "Siri"],
  [0x7e0b, "Bluetooth 1"],
  [0x7e0c, "Bluetooth 2"],
  [0x7e0d, "Bluetooth 3"],
  [0x7e0e, "2.4 GHz"],
  [0x7e0f, "Battery level"],
]);

const SHORT = new Map<string, string>([
  ["Escape", "Esc"],
  ["Backspace", "Bksp"],
  ["Caps Lock", "Caps"],
  ["Right Ctrl", "Ctrl"],
  ["Right Shift", "Shift"],
  ["Right Alt", "Alt"],
  ["Right GUI", "GUI"],
  ["Right Option", "Opt"],
  ["Right Command", "⌘"],
  ["Brightness +", "Bri +"],
  ["Brightness −", "Bri −"],
  ["Volume +", "Vol +"],
  ["Volume −", "Vol −"],
  ["Play / Pause", "Play"],
  ["Mission Control", "Mission"],
  ["Bluetooth 1", "BT 1"],
  ["Bluetooth 2", "BT 2"],
  ["Bluetooth 3", "BT 3"],
  ["Battery level", "Battery"],
  ["RGB brightness +", "RGB +"],
  ["RGB brightness −", "RGB −"],
]);

export function keycodeName(code: number): string {
  if (code >= 0x0004 && code <= 0x001d) {
    return String.fromCharCode("A".charCodeAt(0) + code - 0x0004);
  }
  if (code >= 0x001e && code <= 0x0026) return String(code - 0x001d);
  if (code === 0x0027) return "0";
  if (code >= 0x003a && code <= 0x0045) return `F${code - 0x0039}`;
  return NAMED.get(code) ?? `0x${code.toString(16).padStart(4, "0").toUpperCase()}`;
}

export function shortKeycodeName(code: number): string {
  const name = keycodeName(code);
  return SHORT.get(name) ?? name;
}

export function qmkCode(code: number): string {
  const name = keycodeName(code);
  const normalized = name.toUpperCase().replaceAll(" ", "_").replaceAll("/", "_");
  return normalized.startsWith("0X") ? normalized : `KC_${normalized}`;
}

