import { useEffect, useMemo, useRef, useState } from "react";
import { AgentPanel } from "./components/AgentPanel";
import { ActivityPage } from "./components/ActivityPage";
import { CommandPalette } from "./components/CommandPalette";
import { KeyboardCanvas } from "./components/KeyboardCanvas";
import { KeyInspector } from "./components/KeyInspector";
import { Sidebar } from "./components/Sidebar";
import { StatusBar } from "./components/StatusBar";
import { Topbar } from "./components/Topbar";
import { OverviewPage } from "./components/OverviewPage";
import { TransactionReview } from "./components/TransactionReview";
import { fingerprintFor } from "./data/projectEvidence";
import { DraftProvider } from "./draft/DraftProvider";
import { FeatureRail, FeatureWorkspace } from "./components/FeaturePages";
import { keycodeName } from "./keycodes";
import type { Page, SelectedKey } from "./types";
import { useFirmwareProbe } from "./useFirmwareProbe";
import { useInspection } from "./useInspection";

export function App() {
  const { inspection, error, loading, observedAt, status, refresh } = useInspection();
  const firmwareProbe = useFirmwareProbe();
  const [activeLayer, setActiveLayer] = useState(0);
  const [selected, setSelected] = useState<SelectedKey>({ row: 0, col: 0, code: 0x29, label: "Escape" });
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [activePage, setActivePage] = useState<Page>("Overview");
  const [reviewOpen, setReviewOpen] = useState(false);
  const initializedDefaultLayer = useRef(false);

  const layer = inspection?.layers[activeLayer];
  const layerNames = useMemo(() => inspection?.layers.map((item) => item.name) ?? [], [inspection]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPaletteOpen(true);
      }
      if (event.key === "Escape") setPaletteOpen(false);
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  useEffect(() => {
    if (!notice) return;
    const timer = window.setTimeout(() => setNotice(null), 3600);
    return () => window.clearTimeout(timer);
  }, [notice]);

  useEffect(() => {
    if (!inspection || initializedDefaultLayer.current) return;
    initializedDefaultLayer.current = true;
    const initialLayer = inspection.layers[inspection.active_default_layer] ? inspection.active_default_layer : 0;
    setActiveLayer(initialLayer);
    const code = inspection.layers[initialLayer]?.matrix[selected.row]?.[selected.col] ?? selected.code;
    setSelected((current) => ({ ...current, code, label: keycodeName(code) }));
  }, [inspection, selected.code, selected.col, selected.row]);

  function selectLayer(index: number) {
    setActiveLayer(index);
    if (!inspection) return;
    const code = inspection.layers[index].matrix[selected.row]?.[selected.col] ?? 0;
    setSelected((current) => ({ ...current, code, label: keycodeName(code) }));
  }

  if (!inspection) {
    return (
      <main className="loading-screen">
        <div className="brand-mark">K</div>
        <h1>{error ? "Q3 Max unavailable" : "Inspecting Q3 Max"}</h1>
        <p>{error ?? "Reading firmware, layers, macros, wireless settings, and RGB…"}</p>
        {error ? <button className="primary-button" onClick={refresh}>Try again</button> : <div className="loading-line" />}
      </main>
    );
  }

  const protocolVersion = firmwareProbe.probe?.installed && firmwareProbe.probe.protocol
    ? `${firmwareProbe.probe.protocol.major}.${firmwareProbe.probe.protocol.minor}`
    : undefined;
  const widePage = reviewOpen || activePage === "Overview" || activePage === "Activity";

  return (
    <DraftProvider inspection={inspection} fingerprint={fingerprintFor(inspection)}>
    <div className={`app-shell ${widePage ? "wide-page" : ""} ${reviewOpen ? "reviewing" : ""}`}>
      <Sidebar activePage={activePage} onSelect={setActivePage} />
      <Topbar
        inspection={inspection}
        loading={loading}
        status={status}
        observedAt={observedAt}
        error={error}
        onRefresh={refresh}
        onPalette={() => setPaletteOpen(true)}
        protocolVersion={protocolVersion}
      />
      {reviewOpen ? <>
        <TransactionReview onClose={() => setReviewOpen(false)} />
        <OverviewPage className="review-mobile-background" inspection={inspection} firmwareProbe={firmwareProbe} status={status} observedAt={observedAt} onRefresh={refresh} onReview={() => setReviewOpen(true)} onNavigate={setActivePage} />
        <TransactionReview compact onClose={() => setReviewOpen(false)} />
      </> : activePage === "Overview" ? (
        <OverviewPage inspection={inspection} firmwareProbe={firmwareProbe} status={status} observedAt={observedAt} onRefresh={refresh} onReview={() => setReviewOpen(true)} onNavigate={setActivePage} />
      ) : activePage === "Activity" ? <ActivityPage /> : activePage === "Keymap" ? (
        <main className="workspace">
          <div className="layer-tabs" role="tablist" aria-label="Viewed keyboard layer">
            {layerNames.map((name, index) => (
              <button role="tab" aria-selected={activeLayer === index} className={activeLayer === index ? "active" : ""} title={`View ${name} layer`} onClick={() => selectLayer(index)} key={name}>{name}</button>
            ))}
          </div>
          {layer ? <KeyboardCanvas layer={layer} selected={selected} onSelect={setSelected} /> : null}
          <KeyInspector selected={selected} layer={activeLayer} layerName={layer?.name ?? `Layer ${activeLayer}`} onPreview={setNotice} />
        </main>
      ) : <FeatureWorkspace page={activePage} inspection={inspection} onPreview={setNotice} firmwareProbe={firmwareProbe} />}
      {!reviewOpen && activePage === "Keymap"
        ? <AgentPanel onPreview={setNotice} firmwareProbe={firmwareProbe} />
        : !reviewOpen && activePage !== "Overview" && activePage !== "Activity" ? <FeatureRail page={activePage} inspection={inspection} onPreview={setNotice} firmwareProbe={firmwareProbe} /> : null}
      <StatusBar inspection={inspection} activeLayer={activeLayer} />
      {paletteOpen ? <CommandPalette onClose={() => setPaletteOpen(false)} onRefresh={refresh} onNotify={setNotice} onNavigate={(page) => { setReviewOpen(false); setActivePage(page); }} onReview={() => setReviewOpen(true)} /> : null}
      {notice ? <div className="toast" role="status">{notice}</div> : null}
    </div>
    </DraftProvider>
  );
}
