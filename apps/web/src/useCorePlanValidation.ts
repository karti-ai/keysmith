import { useEffect, useState } from "react";
import type { DraftPlan } from "./types";

interface ConfigurationSnapshot {
  configuration: {
    rgb: { brightness: number; effect: number; speed: number; hue: number; saturation: number };
    layers: Array<{ matrix: number[][] }>;
    wireless_power: { backlight_timeout_seconds: number; sleep_timeout_seconds: number };
    debounce: { algorithm_id: number; algorithm: string; time_ms: number };
    encoders: Array<{ layer: number; counter_clockwise: number; clockwise: number }>;
  };
  [key: string]: unknown;
}

interface CoreMutationPlan {
  plan_id: string;
  diff: { changes: unknown[] };
  risk: { level: string; reasons: string[] };
  confirmation: { required: boolean; attended: boolean };
  executable: boolean;
  [key: string]: unknown;
}

interface CorePlanInspection {
  valid: boolean;
  declared_plan_id: string;
  computed_plan_id: string | null;
  issues: string[];
  executable: boolean;
  mutation_endpoint_available: boolean;
}

export interface CorePlanValidation {
  loading: boolean;
  error: string | null;
  plan: CoreMutationPlan | null;
  inspection: CorePlanInspection | null;
  blockedReasons: string[];
}

async function jsonResponse<T>(response: Response) {
  const body = await response.json() as T & { error?: string };
  if (!response.ok) throw new Error(body.error ?? `Planner request failed (${response.status})`);
  return body;
}

export function useCorePlanValidation(draft: DraftPlan | null): CorePlanValidation {
  const [state, setState] = useState<CorePlanValidation>({ loading: Boolean(draft), error: null, plan: null, inspection: null, blockedReasons: [] });

  useEffect(() => {
    if (!draft) {
      setState({ loading: false, error: null, plan: null, inspection: null, blockedReasons: [] });
      return;
    }
    const controller = new AbortController();
    setState({ loading: true, error: null, plan: null, inspection: null, blockedReasons: [] });
    void (async () => {
      try {
        const baseline = await jsonResponse<ConfigurationSnapshot>(await fetch("/api/config/snapshot", { signal: controller.signal }));
        const target = structuredClone(baseline);
        const blockedReasons: string[] = [];
        for (const change of draft.changes) {
          const operation = change.operation;
          if (!change.rollbackComplete || change.executionSupport === "draft-only") blockedReasons.push(`${change.target}: complete rollback evidence is unavailable`);
          if (operation.kind === "rgb_profile") target.configuration.rgb = { effect: operation.effect, brightness: operation.brightness, speed: operation.speed, hue: operation.hue, saturation: operation.saturation };
          if (operation.kind === "keycode") {
            const targetLayer = target.configuration.layers[operation.layer];
            if (targetLayer?.matrix[operation.row]) targetLayer.matrix[operation.row][operation.column] = operation.keycode;
          }
          if (operation.kind === "wireless_power") target.configuration.wireless_power = { backlight_timeout_seconds: operation.backlight_timeout_seconds, sleep_timeout_seconds: operation.sleep_timeout_seconds };
          if (operation.kind === "debounce") { target.configuration.debounce.algorithm_id = operation.algorithm_id; target.configuration.debounce.time_ms = operation.time_ms; target.configuration.debounce.algorithm = `algorithm ${operation.algorithm_id}`; }
          if (operation.kind === "encoder") { const binding = target.configuration.encoders[operation.layer]; if (binding) binding[operation.clockwise ? "clockwise" : "counter_clockwise"] = operation.keycode; }
        }
        const plannerPlan = await jsonResponse<CoreMutationPlan>(await fetch("/api/plans/preview", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ baseline, target }),
          signal: controller.signal,
        }));
        const inspection = await jsonResponse<CorePlanInspection>(await fetch("/api/plans/inspect", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(plannerPlan),
          signal: controller.signal,
        }));
        setState({ loading: false, error: null, plan: plannerPlan, inspection, blockedReasons: Array.from(new Set(blockedReasons)) });
      } catch (reason) {
        if (controller.signal.aborted) return;
        setState({ loading: false, error: reason instanceof Error ? reason.message : String(reason), plan: null, inspection: null, blockedReasons: [] });
      }
    })();
    return () => controller.abort();
  }, [draft]);

  return state;
}
