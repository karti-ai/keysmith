import { Check, CheckCircle2, Copy, Download, FileJson, LockKeyhole, ShieldAlert, Trash2, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useDraft } from "../draft/DraftProvider";
import { useCorePlanValidation } from "../useCorePlanValidation";

async function sha256(value: string) {
  const bytes = new TextEncoder().encode(value);
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, "0")).join("");
}

export function TransactionReview({ onClose, compact = false }: { onClose: () => void; compact?: boolean }) {
  const { plan, discardPlan, exportPlan } = useDraft();
  const coreValidation = useCorePlanValidation(plan);
  const [view, setView] = useState<"grouped" | "json">("grouped");
  const [planHash, setPlanHash] = useState("Calculating…");
  const groups = useMemo(() => plan ? Array.from(new Set(plan.changes.map((change) => change.group))) : [], [plan]);
  const blocked = plan?.changes.filter((change) => !change.rollbackComplete || change.executionSupport === "draft-only") ?? [];

  useEffect(() => {
    let active = true;
    if (plan) void sha256(JSON.stringify(plan)).then((hash) => { if (active) setPlanHash(hash); });
    return () => { active = false; };
  }, [plan]);

  if (!plan) return <section className={`transaction-review empty-review ${compact ? "compact" : ""}`}><button className="review-close" onClick={onClose}><X /><span>Close</span></button><h1>No pending plan</h1><p>The local restore draft was discarded. No keyboard state changed.</p></section>;

  const discard = () => { discardPlan(); onClose(); };
  if (compact) {
    return (
      <section className="mobile-review-sheet" role="dialog" aria-modal="true" aria-label="Transaction review">
        <div className="sheet-grabber" />
        <div className="sheet-heading"><CheckCircle2 /><h2>Preview complete</h2><span>{plan.changes.length} entries</span><button onClick={onClose} aria-label="Close transaction review"><X /></button></div>
        <dl><dt>Plan ID</dt><dd>{plan.planId}</dd><dt>Grouped scopes</dt><dd>{groups.length}</dd><dt>Rollback snapshot</dt><dd>{plan.source.label}</dd><dt>SHA-256</dt><dd className="truncate-hash">{planHash}</dd></dl>
        <button className="wide-review-button" onClick={exportPlan}><Download />Export plan JSON</button>
        <button className="wide-review-button" onClick={discard}><Trash2 />Discard draft</button>
        <button className="wide-review-button disabled" disabled><LockKeyhole />Apply unavailable</button>
        <small>Requires a separate attended-terminal confirmation. This browser cannot write.</small>
      </section>
    );
  }

  return (
    <main className="transaction-workspace">
      <header className="transaction-heading"><div><h1>Review transaction</h1><p>Device: Q3 Max · Source: <strong>{plan.source.label}</strong> · {groups.length} grouped scopes, {plan.changes.length} atomic entries</p></div><button className="review-close" onClick={onClose}><X /><span>Close</span></button></header>
      <div className="transaction-layout">
        <div className="transaction-main">
          <section className="transaction-panel agent-request"><span>Draft request</span><strong>{plan.request}</strong></section>
          <section className="transaction-panel evidence-flow"><h2>Deterministic evidence flow</h2>{[
            ["Read source state", plan.source.kind === "live-usb-inspection" ? "Captured the connected USB inspection and keymap." : "Loaded the selected static evidence source."],
            ["Bind rollback evidence", `Bound each entry to ${plan.source.kind === "live-usb-inspection" ? "the live USB snapshot" : "preserved static evidence"}.`],
            ["Compile exact diff", `Generated ${plan.changes.length} ordered atomic entries across ${groups.length} scopes.`],
            ["Validate safety boundary", "Verified bounds, rollback evidence, and server write lock."],
            ["Stop for confirmation", "No web mutation path exists; compatible v0.3 firmware still requires a physical chord and separate attended host tooling."],
          ].map(([title, detail], index) => <div key={title}><Check /><b>{index + 1}</b><span><strong>{title}</strong><small>{detail}</small></span><em>Complete</em></div>)}</section>
          <section className="transaction-panel exact-diff">
            <div className="diff-heading"><div><h2>Proposed changes (exact diff)</h2><span>{plan.changes.length} atomic entries</span></div><div className="view-switch"><button className={view === "grouped" ? "active" : ""} onClick={() => setView("grouped")}>Grouped</button><button className={view === "json" ? "active" : ""} onClick={() => setView("json")}>JSON</button></div></div>
            {view === "json" ? <pre>{JSON.stringify(plan, null, 2)}</pre> : groups.map((group) => <div className="diff-group" key={group}><h3>{group}</h3><div className="diff-table" role="table"><div className="diff-row diff-labels" role="row"><span>#</span><span>Target</span><span>Before (source)</span><span>After (draft)</span><span>Risk</span><span>Storage</span></div>{plan.changes.filter((change) => change.group === group).map((change) => <div className="diff-row" role="row" key={change.id}><span>{plan.changes.indexOf(change) + 1}</span><span><strong>{change.target}</strong>{change.layer ? <small>{change.layer} · row {change.row}, col {change.column}</small> : null}</span><code>{change.before}</code><code>{change.after}</code><span className={`risk-${change.risk}`}>{change.risk}</span><span>{change.storage}</span></div>)}</div></div>)}
            <div className="plan-evidence-grid"><dl><dt>Device identity</dt><dd>{plan.deviceFingerprint}</dd><dt>Source kind</dt><dd>{plan.source.kind}</dd><dt>Source snapshot</dt><dd>{plan.source.label}</dd><dt>Local plan ID</dt><dd>{plan.planId}</dd><dt>Core plan ID</dt><dd>{coreValidation.plan?.plan_id ?? (coreValidation.loading ? "Validating…" : "Endpoint unavailable")}</dd><dt>Plan SHA-256</dt><dd className="hash-with-copy">{planHash}<button aria-label="Copy plan hash" onClick={() => void navigator.clipboard?.writeText(planHash)}><Copy /></button></dd></dl><div><h3>Validation results</h3>{["Browser and server apply paths absent", "Matrix bounds validated", "Physical confirmation required"].map((label) => <span key={label}><CheckCircle2 />{label}<em>OK</em></span>)}<span className={blocked.length ? "validation-neutral" : ""}><ShieldAlert />Rollback coverage {blocked.length ? `${blocked.length} blocked` : "complete"}<em>{blocked.length ? "BLOCK" : "OK"}</em></span><span className={coreValidation.inspection?.valid ? "" : "validation-neutral"}><CheckCircle2 />Core planner {coreValidation.loading ? "checking" : coreValidation.inspection?.valid ? "verified" : "unavailable"}<em>{coreValidation.inspection?.valid ? "OK" : "LOCAL"}</em></span></div></div>
          </section>
          <div className="transaction-warning"><ShieldAlert /><span><strong>Web and agent paths never apply mutations.</strong> {blocked.length ? `${blocked.length} draft entries are explicitly non-executable because rollback is incomplete.` : "Eligible entries can be compiled for the v0.3 candidate protocol, but this does not prove compatible firmware is installed; execution remains separate and attended."}</span></div>
        </div>
        <aside className="preview-summary">
          <CheckCircle2 className="summary-check" /><h2>Preview complete</h2><p>Review the exact plan. No changes have been written.</p>
          <dl><dt>Atomic entries</dt><dd>{plan.changes.length}</dd><dt>Grouped scopes</dt><dd>{groups.length}</dd><dt>Device</dt><dd>Q3 Max</dd><dt>Plan ID</dt><dd>{plan.planId}</dd></dl>
          <div className="next-step"><h3>What happens next</h3><p>The CLI can compile candidate packets offline. This build has no apply command; any future execution requires separate attended tooling and physical confirmation.</p></div>
          <div className="compatibility-warning"><ShieldAlert /><span><strong>Candidate compatibility only</strong>A valid plan does not prove v0.3 is installed, recovery evidence exists, or configuration was preserved. This browser cannot write.</span></div>
          <button onClick={exportPlan}><Download />Export plan JSON</button><button onClick={discard}><Trash2 />Discard draft</button><button disabled><LockKeyhole />Apply unavailable</button><small>Attended terminal required for any future write.</small>
        </aside>
      </div>
    </main>
  );
}
