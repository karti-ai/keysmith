import type { Inspection } from "../types";

export const PROJECT_EVIDENCE = {
  activity: [
    { time: "Reference", title: "Protocol 0.3 verified", detail: "USB discovery, exact source identity, and the default locked gate were validated on reference hardware.", type: "protocol" },
    { time: "Reference", title: "Configuration preserved", detail: "Canonical before/after configuration comparison found no drift during the attended firmware validation.", type: "snapshot" },
    { time: "Reference", title: "Write bypasses denied", detail: "Legacy setters and bootloader paths were rejected before dispatch by the firmware policy gate.", type: "security" },
    { time: "Reference", title: "Prepare and cancel tested", detail: "A no-op operation returned from prepared to locked without physical arming or device mutation.", type: "protocol" },
  ],
  snapshots: [
    { name: "Pre-operation snapshot", detail: "Required private rollback evidence", time: "operator supplied" },
    { name: "Post-operation snapshot", detail: "Required readback and comparison", time: "operator supplied" },
    { name: "Factory recovery image", detail: "Kept outside the source repository", time: "operator supplied" },
  ],
} as const;

export function fingerprintFor(inspection: Inspection) {
  const { identity } = inspection;
  return [
    identity.vendor_id.toString(16).padStart(4, "0"),
    identity.product_id.toString(16).padStart(4, "0"),
    identity.layout.toLowerCase().replace(/[^a-z0-9]+/g, "-"),
  ].join(":");
}
