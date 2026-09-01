import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import type { DraftChange, DraftPlan, DraftScope, Inspection } from "../types";

const STORAGE_VERSION = 2;

interface StoredDraft {
  version: typeof STORAGE_VERSION;
  plan: DraftPlan | null;
}

interface DraftContextValue {
  plan: DraftPlan | null;
  replaceScope: (scope: DraftScope, request: string, changes: DraftChange[]) => void;
  discardPlan: () => void;
  exportPlan: () => void;
}

const DraftContext = createContext<DraftContextValue | null>(null);

function stablePlanId(fingerprint: string, request: string, changes: DraftChange[]) {
  const value = JSON.stringify({ fingerprint, request, changes: [...changes].sort((a, b) => a.id.localeCompare(b.id)) });
  let hash = 0x811c9dc5;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193);
  }
  return `ksdraft_v2_${(hash >>> 0).toString(16).padStart(8, "0")}`;
}

function sourceFor(inspection: Inspection) {
  return {
    kind: "live-usb-inspection" as const,
    label: `Live USB inspection · ${inspection.identity.firmware}`,
    snapshotSha256: "validated-by-core-snapshot-endpoint",
    capturedAt: new Date().toISOString(),
  };
}

function readStored(key: string) {
  try {
    const raw = window.localStorage.getItem(key);
    if (!raw) return null;
    const stored = JSON.parse(raw) as StoredDraft;
    return stored.version === STORAGE_VERSION ? stored.plan : null;
  } catch {
    return null;
  }
}

export function DraftProvider({ inspection, fingerprint, children }: { inspection: Inspection; fingerprint: string; children: ReactNode }) {
  const storageKey = `keysmith:draft:v${STORAGE_VERSION}:${fingerprint}`;
  const [plan, setPlan] = useState<DraftPlan | null>(() => readStored(storageKey));

  const persist = useCallback((next: DraftPlan | null) => {
    setPlan(next);
    window.localStorage.setItem(storageKey, JSON.stringify({ version: STORAGE_VERSION, plan: next } satisfies StoredDraft));
  }, [storageKey]);

  const replaceScope = useCallback((scope: DraftScope, request: string, changes: DraftChange[]) => {
    const retained = plan?.changes.filter((change) => change.scope !== scope) ?? [];
    const nextChanges = [...retained, ...changes].sort((a, b) => a.id.localeCompare(b.id));
    if (nextChanges.length === 0) {
      persist(null);
      return;
    }
    const combinedRequest = nextChanges.length === changes.length
      ? request
      : `Stage ${nextChanges.length} reviewed entries: ${nextChanges.map((change) => change.target).join(", ")}.`;
    persist({
      schema: "keysmith.draft-plan/v2",
      planId: stablePlanId(fingerprint, combinedRequest, nextChanges),
      deviceFingerprint: fingerprint,
      source: sourceFor(inspection),
      request: combinedRequest,
      changes: nextChanges,
      createdAt: new Date().toISOString(),
    });
  }, [fingerprint, inspection, persist, plan]);
  const discardPlan = useCallback(() => persist(null), [persist]);
  const exportPlan = useCallback(() => {
    if (!plan) return;
    const blob = new Blob([`${JSON.stringify(plan, null, 2)}\n`], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${plan.planId}.json`;
    anchor.click();
    URL.revokeObjectURL(url);
  }, [plan]);

  const value = useMemo(() => ({ plan, replaceScope, discardPlan, exportPlan }), [discardPlan, exportPlan, plan, replaceScope]);
  return <DraftContext.Provider value={value}>{children}</DraftContext.Provider>;
}

export function useDraft() {
  const value = useContext(DraftContext);
  if (!value) throw new Error("useDraft must be used inside DraftProvider");
  return value;
}
