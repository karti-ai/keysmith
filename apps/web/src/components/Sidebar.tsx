import {
  Activity,
  Bot,
  Cable,
  Command,
  Cpu,
  Lightbulb,
  Radio,
  Settings,
  SlidersHorizontal,
  Waypoints,
  LayoutDashboard,
} from "lucide-react";
import type { Page } from "../types";

const ITEMS = [
  ["Overview", LayoutDashboard],
  ["Keymap", Command],
  ["Lighting", Lightbulb],
  ["Macros", SlidersHorizontal],
  ["Wireless", Radio],
  ["Diagnostics", Waypoints],
  ["Agent", Bot],
  ["Firmware", Cpu],
  ["Activity", Activity],
  ["Settings", Settings],
] as const;

export function Sidebar({ activePage, onSelect }: { activePage: Page; onSelect: (page: Page) => void }) {
  return (
    <aside className="sidebar" aria-label="Primary navigation">
      <div className="brand">
        <span className="brand-mark" aria-hidden="true">K</span>
        <span>Keysmith</span>
      </div>
      <nav>
        {ITEMS.map(([label, Icon]) => (
          <button
            className={activePage === label ? "nav-item active" : "nav-item"}
            key={label}
            onClick={() => onSelect(label)}
            aria-current={activePage === label ? "page" : undefined}
          >
            <Icon size={18} strokeWidth={1.75} />
            <span>{label}</span>
          </button>
        ))}
      </nav>
      <div className="sidebar-foot">
        <Cable size={16} />
        <span>Tailnet only</span>
      </div>
    </aside>
  );
}
