import { useCallback, useEffect, useRef, useState } from "react";
import type { Inspection } from "./types";

export type InspectionStatus = "loading" | "live" | "refreshing" | "stale" | "disconnected";

export function useInspection() {
  const [inspection, setInspection] = useState<Inspection | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [observedAt, setObservedAt] = useState<number | null>(null);
  const [status, setStatus] = useState<InspectionStatus>("loading");
  const inspectionRef = useRef<Inspection | null>(null);
  const requestSequence = useRef(0);

  const refresh = useCallback(async () => {
    const request = ++requestSequence.current;
    setLoading(true);
    setStatus(inspectionRef.current ? "refreshing" : "loading");
    setError(null);

    try {
      const response = await fetch("/api/inspect");
      const body = await response.json() as Inspection & { error?: string };
      if (!response.ok) throw new Error(body.error ?? `Inspection failed (${response.status})`);
      if (request !== requestSequence.current) return;

      inspectionRef.current = body;
      setInspection(body);
      setObservedAt(Date.now());
      setError(null);
      setStatus(body.connected ? "live" : "disconnected");
    } catch (reason) {
      if (request !== requestSequence.current) return;

      const message = reason instanceof Error ? reason.message : String(reason);
      setError(message);
      setStatus(inspectionRef.current ? "stale" : "disconnected");
    } finally {
      if (request === requestSequence.current) setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { inspection, error, loading, observedAt, status, refresh };
}
