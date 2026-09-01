export type Feature =
  | "default_layer"
  | "bluetooth"
  | "two_point_four_ghz"
  | "state_notifications"
  | "dynamic_debounce"
  | "snap_click"
  | "keychron_rgb";

export type Page =
  | "Overview"
  | "Keymap"
  | "Lighting"
  | "Macros"
  | "Wireless"
  | "Diagnostics"
  | "Agent"
  | "Firmware"
  | "Activity"
  | "Settings";

export type DraftScope = "lighting" | "keymap" | "macro" | "wireless" | "diagnostics";

export type DraftOperation =
  | { kind: "rgb_profile"; effect: number; brightness: number; speed: number; hue: number; saturation: number }
  | { kind: "keycode"; layer: number; row: number; column: number; keycode: number }
  | { kind: "encoder"; layer: number; clockwise: boolean; keycode: number }
  | { kind: "wireless_power"; backlight_timeout_seconds: number; sleep_timeout_seconds: number }
  | { kind: "debounce"; algorithm_id: number; time_ms: number }
  | { kind: "snap_click"; pair: number; mode: number; keycode_a: number; keycode_b: number }
  | { kind: "macro"; slot: number; name: string; actions: Array<{ kind: "Keystroke" | "Delay"; value: string }> };

export interface DraftChange {
  id: string;
  scope: DraftScope;
  group: string;
  target: string;
  before: number | string;
  after: number | string;
  risk: "low" | "medium" | "high" | "critical";
  storage: string;
  operation: DraftOperation;
  rollbackComplete: boolean;
  executionSupport: "v0.3-attended" | "draft-only";
  layer?: string;
  row?: number;
  column?: number;
}

export interface DraftPlan {
  schema: "keysmith.draft-plan/v2";
  planId: string;
  deviceFingerprint: string;
  source: {
    kind: "checked-in-static-evidence" | "live-usb-inspection";
    label: string;
    snapshotSha256: string;
    capturedAt: string;
  };
  request: string;
  changes: DraftChange[];
  createdAt: string;
}

export interface KeymapLayer {
  index: number;
  name: "Mac" | "Mac Fn" | "Win" | "Win Fn";
  matrix: number[][];
}

export interface Inspection {
  connected: boolean;
  identity: {
    name: string;
    layout: string;
    vendor_id: number;
    product_id: number;
    path: string;
    firmware: string;
    via_protocol: number;
    keychron_protocol: number;
    qmk_command_set: number;
  };
  features: Feature[];
  active_default_layer: number;
  layers: KeymapLayer[];
  macros: { slots: number; buffer_bytes: number; used_bytes: number };
  snap_click: { pair_capacity: number; configured_pairs: number };
  wireless_power: {
    backlight_timeout_seconds: number;
    sleep_timeout_seconds: number;
  };
  debounce: { algorithm_id: number; algorithm: string; time_ms: number };
  rgb: {
    brightness: number;
    effect: number;
    speed: number;
    hue: number;
    saturation: number;
  };
  encoders: Array<{
    layer: number;
    counter_clockwise: number;
    clockwise: number;
  }>;
  write_enabled: boolean;
}

export interface SelectedKey {
  row: number;
  col: number;
  code: number;
  label: string;
}

export interface KeysmithRuntime {
  transport_id: number;
  transport: string;
  wireless_state_id: number;
  wireless_state: string;
  usb_power: boolean;
  mutations_enabled: boolean;
  default_layer?: number;
  active_layer?: number;
  usb_state?: number;
  uptime_ms?: number;
  host_leds?: number;
  wireless_host?: number;
}

export interface KeysmithProtocol {
  major: number;
  minor: number;
  packet_bytes: number;
  runtime_status: boolean;
  usb_only: boolean;
  mutation_capabilities: number;
  read_capabilities?: number;
  build_page_count?: number;
  keymap_chunk_keycodes?: number;
  via_eeprom_magic?: string;
  runtime: KeysmithRuntime;
  build?: {
    keysmith: string;
    qmk_git_hash: string;
    qmk_version: string;
    qmk_build_date: string;
    keyboard: string;
    keymap: string;
  };
  device?: {
    matrix_rows: number;
    matrix_cols: number;
    layer_count: number;
    rgb_led_count: number;
    encoder_count: number;
    via_protocol: number;
    protocol_version: number;
    qmk_command_set: number;
    vendor_id: number;
    product_id: number;
    device_version: number;
    raw_packet_bytes: number;
  };
  rgb?: {
    enabled: boolean;
    suspended: boolean;
    effect: number;
    hue: number;
    saturation: number;
    brightness: number;
    speed: number;
    flags: number;
    led_count: number;
  };
  wireless?: {
    state_id: number;
    state: string;
    host: number;
    battery_percentage: number;
    battery_valid: boolean;
    battery_voltage_mv: number;
    battery_empty: boolean;
    battery_critical: boolean;
    battery_sample_age_ms?: number;
    transport_id: number;
    transport: string;
  };
  macro_metadata?: {
    slots: number;
    buffer_bytes: number;
    contents_exposed: boolean;
  };
  write_status?: {
    state_id: number;
    state: "locked" | "prepared" | "armed" | "unknown";
    last_result: number;
    operation: number;
    plan_tag: string;
    operation_index: number;
    operation_total: number;
    usb_ready: boolean;
  };
}

export interface FirmwareProbe {
  installed: boolean;
  protocol?: KeysmithProtocol;
}

export interface FirmwareProbeState {
  probe: FirmwareProbe | null;
  error: string | null;
  loading: boolean;
  refresh: () => Promise<void>;
}

export type AgentIntent = "keymap" | "lighting" | "macro" | "wireless" | "firmware";

export interface AgentPreviewPlan {
  intent: AgentIntent;
  title: string;
  summary: string;
  reads: string[];
  draft: string[];
  guardrails: string[];
}
