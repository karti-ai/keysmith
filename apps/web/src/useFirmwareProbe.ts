import { useCallback, useEffect, useState } from "react";
import type { FirmwareProbe } from "./types";

interface ApiError {
  error?: string;
}

function isFirmwareProbe(value: unknown): value is FirmwareProbe {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<FirmwareProbe>;
  if (candidate.installed === false) return true;
  if (candidate.installed !== true || !candidate.protocol) return false;

  const protocol = candidate.protocol;
  const runtime = protocol.runtime;
  return typeof protocol.major === "number"
    && typeof protocol.minor === "number"
    && typeof protocol.packet_bytes === "number"
    && typeof protocol.runtime_status === "boolean"
    && typeof protocol.usb_only === "boolean"
    && typeof protocol.mutation_capabilities === "number"
    && Boolean(runtime)
    && typeof runtime.transport === "string"
    && typeof runtime.wireless_state === "string"
    && typeof runtime.usb_power === "boolean"
    && typeof runtime.mutations_enabled === "boolean";
}

export async function fetchFirmwareProbe(signal?: AbortSignal): Promise<FirmwareProbe> {
  const response = await fetch("/api/firmware/probe", { signal });
  const body = await response.json() as unknown;

  if (!response.ok) {
    const apiError = body as ApiError;
    throw new Error(apiError.error ?? "Firmware probe failed");
  }
  if (!isFirmwareProbe(body)) throw new Error("Firmware probe returned an invalid response");
  return body;
}

export function useFirmwareProbe() {
  const [probe, setProbe] = useState<FirmwareProbe | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async (signal?: AbortSignal) => {
    setLoading(true);
    try {
      const nextProbe = await fetchFirmwareProbe(signal);
      setProbe(nextProbe);
      setError(null);
    } catch (reason) {
      if (reason instanceof DOMException && reason.name === "AbortError") return;
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      if (!signal?.aborted) setLoading(false);
    }
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    void refresh(controller.signal);
    return () => controller.abort();
  }, [refresh]);

  return { probe, error, loading, refresh };
}
