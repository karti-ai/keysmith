export interface KeySpec {
  id: string;
  row: number;
  col: number;
  width?: number;
  gapBefore?: number;
  kind?: "key" | "knob";
}

export const Q3_ROWS: KeySpec[][] = [
  [
    { id: "esc", row: 0, col: 0 },
    { id: "f1", row: 0, col: 1, gapBefore: 0.55 },
    { id: "f2", row: 0, col: 2 },
    { id: "f3", row: 0, col: 3 },
    { id: "f4", row: 0, col: 4 },
    { id: "f5", row: 0, col: 5, gapBefore: 0.35 },
    { id: "f6", row: 0, col: 6 },
    { id: "f7", row: 0, col: 7 },
    { id: "f8", row: 0, col: 8 },
    { id: "f9", row: 0, col: 9, gapBefore: 0.35 },
    { id: "f10", row: 0, col: 10 },
    { id: "f11", row: 0, col: 11 },
    { id: "f12", row: 0, col: 12 },
    { id: "top13", row: 0, col: 13, gapBefore: 0.45 },
    { id: "top14", row: 0, col: 14, gapBefore: 0.25 },
    { id: "top15", row: 0, col: 15 },
    { id: "top16", row: 0, col: 16 },
    { id: "knob", row: 0, col: 13, gapBefore: 0.2, kind: "knob" },
  ],
  [
    { id: "grave", row: 1, col: 0 },
    ...Array.from({ length: 12 }, (_, index) => ({ id: `num${index}`, row: 1, col: index + 1 })),
    { id: "backspace", row: 1, col: 13, width: 2 },
    { id: "insert", row: 1, col: 14, gapBefore: 0.4 },
    { id: "home", row: 1, col: 15 },
    { id: "pgup", row: 1, col: 16 },
  ],
  [
    { id: "tab", row: 2, col: 0, width: 1.5 },
    ...Array.from({ length: 12 }, (_, index) => ({ id: `qrow${index}`, row: 2, col: index + 1 })),
    { id: "backslash", row: 2, col: 13, width: 1.5 },
    { id: "delete", row: 2, col: 14, gapBefore: 0.4 },
    { id: "end", row: 2, col: 15 },
    { id: "pgdn", row: 2, col: 16 },
  ],
  [
    { id: "caps", row: 3, col: 0, width: 1.75 },
    ...Array.from({ length: 11 }, (_, index) => ({ id: `arow${index}`, row: 3, col: index + 1 })),
    { id: "enter", row: 3, col: 13, width: 2.25 },
  ],
  [
    { id: "lshift", row: 4, col: 0, width: 2.25 },
    ...Array.from({ length: 10 }, (_, index) => ({ id: `zrow${index}`, row: 4, col: index + 2 })),
    { id: "rshift", row: 4, col: 13, width: 2.75 },
    { id: "up", row: 4, col: 15, gapBefore: 1.65 },
  ],
  [
    { id: "lctrl", row: 5, col: 0, width: 1.25 },
    { id: "lopt", row: 5, col: 1, width: 1.25 },
    { id: "lcmd", row: 5, col: 2, width: 1.25 },
    { id: "space", row: 5, col: 6, width: 6.25 },
    { id: "rcmd", row: 5, col: 10, width: 1.25 },
    { id: "ropt", row: 5, col: 11, width: 1.25 },
    { id: "fn", row: 5, col: 12, width: 1.25 },
    { id: "rctrl", row: 5, col: 13, width: 1.25 },
    { id: "left", row: 5, col: 14, gapBefore: 0.4 },
    { id: "down", row: 5, col: 15 },
    { id: "right", row: 5, col: 16 },
  ],
];

