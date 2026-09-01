import {
  ArrowRight,
  Bot,
  Check,
  CheckCircle2,
  ClipboardCheck,
  ClipboardCopy,
  Code2,
  ExternalLink,
  Eye,
  FileCode2,
  Github,
  Heart,
  LockKeyhole,
  Menu,
  MonitorSmartphone,
  PackageOpen,
  Radio,
  RotateCcw,
  ShieldCheck,
  TerminalSquare,
  X,
} from "lucide-react";
import { useState, type ReactNode } from "react";

const APP_REPO = "https://github.com/karti-ai/keysmith";
const FIRMWARE_REPO = "https://github.com/karti-ai/keysmith-qmk";
const QUICK_START = `${APP_REPO}#quick-start`;

const modelSteps = [
  ["01", "Observe", "Read identity, firmware, layers, RGB, encoders, debounce, and wireless policy over USB."],
  ["02", "Draft", "Keep proposed changes in browser-local state with explicit before and after values."],
  ["03", "Plan", "Build deterministic snapshots and diffs with risk and rollback evidence."],
  ["04", "Authorize", "Stop the public host at offline preparation. Protocol execution belongs to separate attended USB tooling and a physical keyboard action."],
  ["05", "Recover", "Any future execution must verify readback and keep private recovery evidence outside the public app."],
] as const;

const tourItems = [
  {
    id: "overview",
    label: "Control Center",
    title: "The whole board, without the mystery.",
    body: "Inspect the validated Q3 Max, move through all four layers, and stage local drafts while the visible write policy stays blocked.",
    image: "/assets/keysmith-control-center-overview.png",
    alt: "Keysmith Control Center showing the Keychron Q3 Max keymap, live USB inspection, local draft state, rollback evidence, and blocked browser write policy.",
  },
  {
    id: "firmware",
    label: "Firmware evidence",
    title: "Source identity and recovery stay visible.",
    body: "This reference-lab capture shows protocol and source evidence from one validated board. Each user must supply and privately preserve their own recovery image, configuration evidence, and attended recovery path.",
    image: "/assets/keysmith-firmware-workspace.png",
    alt: "Reference-lab Keysmith firmware workspace showing protocol 0.3, source identity, an operator-supplied recovery checklist, and the attended terminal boundary.",
  },
  {
    id: "mobile",
    label: "Responsive view",
    title: "Private visibility can travel. Authority does not.",
    body: "The responsive Control Center remains useful behind a private authenticated proxy, but management still requires USB and the public showcase never contacts a keyboard.",
    image: "/assets/keysmith-control-center-mobile.png",
    alt: "Responsive mobile Keysmith Control Center with device facts, keyboard canvas, draft summary, snapshots, and firmware track.",
  },
] as const;

const installSteps = [
  ["01", "Check your board", "Connect the Q3 Max ANSI encoder over USB and inspect its exact identity.", "keychronctl inspect --json"],
  ["02", "Capture config evidence", "Save a privacy-aware configuration snapshot, then obtain exact recovery firmware separately.", "keychronctl snapshot --json"],
  ["03", "Build from source", "Clone the public application and verify the Rust and React checks locally.", "git clone https://github.com/karti-ai/keysmith.git"],
  ["04", "Review at the terminal", "Compile and inspect the exact plan locally. No browser or agent command crosses into hardware.", "keychronctl plan prepare --file plan.json --json"],
  ["05", "Verify and relock", "Read the installed protocol and confirm the physical firmware gate is locked.", "keychronctl firmware-probe --json"],
] as const;

const roadmapColumns = [
  {
    label: "Now",
    tone: "red",
    items: ["Q3 Max USB inspection", "Browser-local drafts", "Deterministic Rust plans", "Source installation", "Validated protocol 0.3"],
  },
  {
    label: "Next",
    tone: "blue",
    items: ["Reproducible Linux packaging", "Clearer local-service setup", "Repeatable release evidence"],
  },
  {
    label: "Later",
    tone: "green",
    items: ["More layouts only after validation", "Explicit hardware profiles", "One carefully tested board at a time"],
  },
] as const;

const videoItems = [
  {
    title: "Why consent belongs on the keyboard",
    body: "The reasoning behind local control, physical authorization, and user agency.",
    image: "/assets/keysmith-control-center-mobile.png",
    alt: "Preview crop of the mobile Keysmith Control Center for the planned consent video.",
  },
  {
    title: "From agent request to deterministic plan",
    body: "A transparent walk-through of inspection, validation, and preview without writes.",
    image: "/assets/keysmith-control-center-overview.png",
    alt: "Preview of the Keysmith Control Center for the planned deterministic planning video.",
  },
  {
    title: "How we safely flashed the Q3 Max",
    body: "The backup, source, physical access, and post-flash verification process.",
    image: "/assets/keysmith-firmware-workspace.png",
    alt: "Preview of the Keysmith firmware evidence workspace for the planned safe firmware engineering video.",
  },
] as const;

function ExternalLinkIcon() {
  return <ExternalLink aria-hidden="true" size={15} strokeWidth={1.8} />;
}

function GithubLink({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <a className={className} href={APP_REPO} target="_blank" rel="noreferrer"><Github aria-hidden="true" size={18} />{children}<ExternalLinkIcon /></a>;
}

function CopyButton({ value, label, onStatus }: { value: string; label: string; onStatus: (message: string) => void }) {
  const [copied, setCopied] = useState(false);

  async function copy() {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      onStatus(`${label} copied to clipboard.`);
      window.setTimeout(() => setCopied(false), 1800);
    } catch {
      onStatus(`Could not copy ${label}. Select the command manually.`);
    }
  }

  return <button className="copy-button" type="button" onClick={() => void copy()} aria-label={`Copy ${label}`}>{copied ? <ClipboardCheck aria-hidden="true" /> : <ClipboardCopy aria-hidden="true" />}<span>{copied ? "Copied" : "Copy"}</span></button>;
}

export function App() {
  const [menuOpen, setMenuOpen] = useState(false);
  const [activeTour, setActiveTour] = useState(0);
  const [copyStatus, setCopyStatus] = useState("");
  const active = tourItems[activeTour];

  function closeMenu() {
    setMenuOpen(false);
  }

  return (
    <>
      <a className="skip-link" href="#main">Skip to main content</a>
      <header className="site-header">
        <a className="brand" href="#top" aria-label="Keysmith home" onClick={closeMenu}><img src="/assets/keysmith-mark.svg" alt="" /><span>Keysmith</span></a>
        <nav className={menuOpen ? "nav-links open" : "nav-links"} aria-label="Primary navigation">
          <a href="#product" onClick={closeMenu}>Product</a>
          <a href="#agent" onClick={closeMenu}>Agent guide</a>
          <a href="#install" onClick={closeMenu}>Source setup</a>
          <a href="#roadmap" onClick={closeMenu}>Roadmap</a>
          <a href="#source" onClick={closeMenu}>Source</a>
        </nav>
        <GithubLink className="header-github">View on GitHub</GithubLink>
        <button className="menu-button" type="button" aria-expanded={menuOpen} aria-controls="mobile-navigation" aria-label={menuOpen ? "Close navigation" : "Open navigation"} onClick={() => setMenuOpen((current) => !current)}>{menuOpen ? <X aria-hidden="true" /> : <Menu aria-hidden="true" />}</button>
        <div id="mobile-navigation" className={menuOpen ? "mobile-nav-panel open" : "mobile-nav-panel"} aria-hidden={!menuOpen}>
          <a href="#product" onClick={closeMenu}>Product</a><a href="#agent" onClick={closeMenu}>Agent guide</a><a href="#install" onClick={closeMenu}>Source setup</a><a href="#roadmap" onClick={closeMenu}>Roadmap</a><a href="#source" onClick={closeMenu}>Source</a><GithubLink>View on GitHub</GithubLink>
        </div>
      </header>

      <main id="main">
        <section className="hero" id="top" aria-labelledby="hero-title">
          <div className="hero-copy">
            <h1 id="hero-title">Your keyboard.<br /><span>Understood.</span></h1>
            <p className="hero-summary">Keysmith is an open-source, local-first control surface, Rust CLI, and safety-gated QMK firmware project for the Keychron Q3 Max.</p>
            <p className="hero-detail">It gives agents useful context, turns requests into inspectable local plans, and keeps hardware changes behind an attended USB terminal and physical keyboard authorization.</p>
            <div className="hero-actions"><GithubLink className="button primary">Explore Keysmith</GithubLink><a className="button secondary" href="#install">View source setup<ArrowRight aria-hidden="true" /></a></div>
            <p className="release-line"><span>Independent source preview</span><span>Linux host</span><span>Q3 Max ANSI encoder</span><span>No packaged installer</span></p>
          </div>
          <div className="hero-visual" aria-label="Keysmith product evidence">
            <figure className="hero-app-frame"><img src="/assets/keysmith-control-center-overview.png" alt="Keysmith Control Center reference capture showing a connected Q3 Max, protocol 0.3, blocked browser write policy, local keymap view, and rollback evidence." /><figcaption>Reference hardware capture · protocol 0.3 · gate locked</figcaption></figure>
            <figure className="hero-keyboard"><img src="/assets/keysmith-q3-max-firmware-hero.jpg" alt="Q3 Max-inspired keyboard illustration above a transparent circuit layer with a red physical authorization boundary at the center." /></figure>
          </div>
        </section>

        <section className="model-section section" id="product" aria-labelledby="model-title">
          <div className="section-heading"><p className="section-index">01 · Product model</p><h2 id="model-title">See everything. Change nothing by accident.</h2><p>Keysmith separates understanding a device from earning permission to change it.</p></div>
          <ol className="model-flow">{modelSteps.map(([index, title, body]) => <li key={title}><span className="step-number">{index}</span><div><h3>{title}</h3><p>{body}</p></div></li>)}</ol>
          <figure className="diagram architecture-diagram"><img src="/assets/keysmith-architecture.svg" alt="Architecture diagram showing the read-only UI and agent, loopback server, deterministic Rust planning core, USB Raw HID, physically gated QMK firmware, and a separate normal typing plane." /><figcaption>Two planes. One physical boundary. The public browser has no apply or flash route.</figcaption></figure>
        </section>

        <section className="agent-section section" id="agent" aria-labelledby="agent-title">
          <div className="agent-copy"><p className="section-index">02 · Agent-first</p><h2 id="agent-title">An agent that can explain before anything changes.</h2><p>Keysmith exposes snapshots, JSON plans, CLI commands, and explicit policy boundaries that an agent can inspect without inheriting device authority.</p><blockquote>“Make Caps Lock Escape when tapped and Control when held.”</blockquote><p className="truth-note"><LockKeyhole aria-hidden="true" />The current in-app agent flow is deterministic and local. Keysmith ships no hosted model, background agent, or hardware executor.</p></div>
          <div className="agent-contract" aria-label="Agent contract example">
            <div className="terminal-title"><TerminalSquare aria-hidden="true" /><span>Transparent plan</span><span className="terminal-state">preview only</span></div>
            <div className="terminal-body">
              <code><span className="terminal-prompt">agent@host:~$</span> keychronctl inspect --json</code>
              <pre>{`{
  "model": "Q3 Max",
  "usb": "3434:0830",
  "protocol": "0.3",
  "gate": "locked",
  "write_path": "attended-local-only"
}`}</pre>
              <ol><li><span>01</span><strong>Read first</strong><small>Inspect the live layer and current matrix position.</small></li><li><span>02</span><strong>Draft locally</strong><small>Propose MT(MOD_LCTL, KC_ESC) with an exact diff.</small></li><li><span>03</span><strong>Hard stops</strong><small>No web write, no unattended mutation, no firmware flash.</small></li></ol>
              <code><span className="terminal-prompt">agent:</span> Plan ends here. Physical authorization belongs to separate attended tooling.</code>
            </div>
          </div>
        </section>

        <section className="tour-section section" aria-labelledby="tour-title">
          <div className="section-heading"><p className="section-index">03 · Real product evidence</p><h2 id="tour-title">A control surface that shows its work.</h2><p>Every image below is a checked-in capture of the real Keysmith interface—not a simulated dashboard or live public demo.</p></div>
          <div className="tour-tabs" role="tablist" aria-label="Product tour views">{tourItems.map((item, index) => <button key={item.id} id={`tour-tab-${item.id}`} role="tab" aria-selected={activeTour === index} aria-controls="tour-panel" tabIndex={activeTour === index ? 0 : -1} onClick={() => setActiveTour(index)}>{item.label}</button>)}</div>
          <div className="tour-panel" id="tour-panel" role="tabpanel" aria-labelledby={`tour-tab-${active.id}`}>
            <div className="tour-copy"><span className="reference-label">Reference capture</span><h3>{active.title}</h3><p>{active.body}</p><ul><li><Check aria-hidden="true" />No public API calls</li><li><Check aria-hidden="true" />No live keyboard control</li><li><Check aria-hidden="true" />No web apply or flash route</li></ul></div>
            <figure className={`tour-image ${active.id}`}><img src={active.image} alt={active.alt} /></figure>
          </div>
        </section>

        <section className="safety-section section" id="safety" aria-labelledby="safety-title">
          <div className="section-heading"><p className="section-index">04 · Transaction safety</p><h2 id="safety-title">A transaction, not an unlock.</h2><p>Protocol-capable firmware can gate one reviewed operation and relock on every terminal path. The public host does not execute it.</p></div>
          <ol className="gate-states" aria-label="Firmware authorization states"><li><span>01</span><strong>Locked</strong><small>Default state</small></li><li><span>02</span><strong>Prepared</strong><small>Exact plan, 30 seconds</small></li><li><span>03</span><strong>Physically armed</strong><small>Esc + Space + Right Ctrl, 3 seconds</small></li><li><span>04</span><strong>Commit once</strong><small>Matching local operation only</small></li><li><span>05</span><strong>Relocked</strong><small>Verify and archive</small></li></ol>
          <figure className="diagram safety-diagram"><img src="/assets/keysmith-safety-gate.svg" alt="Protocol state diagram from locked to prepared, physically armed, apply once, and relocked, with cancel, timeout, error, disconnect, and reset returning to locked." /><figcaption>The physical chord authorizes only an already-prepared operation. It never creates a reusable write mode.</figcaption></figure>
          <div className="safety-boundary"><ShieldCheck aria-hidden="true" /><div><strong>The public host stops before execution.</strong><p>Browser and agent surfaces inspect, draft, plan, and compile offline packets. Firmware mutation and flashing remain attended local USB terminal work with explicit physical authorization.</p></div><span>USB management only</span></div>
          <p className="typing-note"><Radio aria-hidden="true" />Bluetooth and 2.4 GHz remain normal typing transports. They are not Keysmith configuration paths.</p>
        </section>

        <section className="install-section section" id="install" aria-labelledby="install-title">
          <div className="install-intro"><p className="section-index">05 · Source setup</p><h2 id="install-title">Build with evidence.</h2><p>A calm, reversible engineering path. Every step leaves context behind, and anything physical remains attended.</p><a className="button secondary accent-border" href={QUICK_START} target="_blank" rel="noreferrer">Read source setup<ExternalLinkIcon /></a><p className="install-disclaimer"><PackageOpen aria-hidden="true" />Host 0.1 source preview · firmware 0.3.0-candidate · no packaged installer or firmware release.</p><div className="install-resources"><a href="https://www.keychron.com/pages/firmware-and-json-files-of-the-keychron-qmk-keyboards" target="_blank" rel="noreferrer">Official Keychron recovery firmware<ExternalLinkIcon /></a><a href={FIRMWARE_REPO} target="_blank" rel="noreferrer">Build-validated firmware source<ExternalLinkIcon /></a></div></div>
          <ol className="install-timeline">{installSteps.map(([index, title, body, command]) => <li key={title}><span className="timeline-number">{index}</span><div><h3>{title}</h3><p>{body}</p><code>{command}</code></div></li>)}</ol>
          <div className="clone-command"><div><span>Start from the public application source</span><code>git clone https://github.com/karti-ai/keysmith.git</code></div><CopyButton value="git clone https://github.com/karti-ai/keysmith.git" label="clone command" onStatus={setCopyStatus} /></div>
        </section>

        <section className="source-section section" id="source" aria-labelledby="source-title">
          <div className="section-heading"><p className="section-index">06 · Open source</p><h2 id="source-title">One product. Two public repositories.</h2><p>The host and firmware stay separate so source history, build inputs, trust boundaries, and licensing remain inspectable.</p></div>
          <div className="repo-rows">
            <article><FileCode2 aria-hidden="true" /><div><p>Rust + React · MIT</p><h3>Keysmith application</h3><span>Inspection, local drafts, deterministic plans, loopback server, and CLI.</span></div><a href={APP_REPO} target="_blank" rel="noreferrer">Browse app source<ExternalLinkIcon /></a></article>
            <article><Code2 aria-hidden="true" /><div><p>GPL source · distribution obligations vary</p><h3>Keysmith firmware</h3><span>Keysmith additions are GPL-2.0-or-later. A combined ARM image links GPLv3 ChibiOS and requires GPLv3 distribution with complete corresponding source and build inputs.</span></div><a href={FIRMWARE_REPO} target="_blank" rel="noreferrer">Browse firmware source<ExternalLinkIcon /></a></article>
          </div>
          <p className="public-boundary"><Eye aria-hidden="true" />Firmware binaries, pairing material, full device dumps, and private recovery snapshots intentionally remain outside the public repository.</p>
        </section>

        <section className="roadmap-section section" id="roadmap" aria-labelledby="roadmap-title">
          <div className="roadmap-header"><div><p className="section-index">07 · Build in public</p><h2 id="roadmap-title">Built in public.<br />Explained in full.</h2><p>Direction, not a release promise. No dates until the work is validated.</p></div><figure><img src="/assets/keysmith-q3-max-firmware-hero.jpg" alt="Q3 Max-inspired keyboard illustration above a circuit layer with a central red physical boundary." /></figure></div>
          <div className="roadmap-grid">{roadmapColumns.map((column) => <article className={`roadmap-column ${column.tone}`} key={column.label}><h3>{column.label}</h3><ul>{column.items.map((item) => <li key={item}><CheckCircle2 aria-hidden="true" />{item}</li>)}</ul></article>)}</div>
        </section>

        <section className="videos-section section" aria-labelledby="videos-title">
          <div className="video-intro"><p className="section-index">08 · Field notes</p><h2 id="videos-title">Karti’s voice. HyperFrames-built.</h2><p>Planned educational videos for power users and builders. No fake playback—these are coming soon.</p><ul><li><MonitorSmartphone aria-hidden="true" />Captions, transcript, and chapters</li><li><Code2 aria-hidden="true" />Exact public source revision</li><li><Bot aria-hidden="true" />No hype. Just verifiable context.</li></ul></div>
          <div className="video-grid">{videoItems.map((video) => <article key={video.title}><figure><img src={video.image} alt={video.alt} /></figure><p className="coming-soon">Coming soon</p><h3>{video.title}</h3><p>{video.body}</p><span>Karti voice · HyperFrames</span></article>)}</div>
        </section>

        <section className="thanks-section section" aria-labelledby="thanks-title">
          <div className="thanks-title"><h2 id="thanks-title">Thank you, Keychron.</h2><Heart aria-hidden="true" /></div>
          <div className="thanks-copy"><p>Publishing QMK firmware source and device documentation made the Q3 Max inspectable, repairable, and worth learning from. That kind of openness moves the whole ecosystem forward. We’re grateful—and we hope Keysmith gives something useful back.</p><div><a href="https://github.com/Keychron/qmk_firmware/tree/2025q3" target="_blank" rel="noreferrer">Keychron QMK 2025q3 source<ExternalLinkIcon /></a><a href="https://www.keychron.com/pages/keychron-q3-max-user-guide" target="_blank" rel="noreferrer">Q3 Max user guide<ExternalLinkIcon /></a><a href="https://launcher.keychron.com/#/keymap" target="_blank" rel="noreferrer">Keychron Launcher<ExternalLinkIcon /></a></div></div>
          <p className="independence-disclaimer">Keysmith is an independent community project. It is not affiliated with, sponsored by, reviewed by, or endorsed by Keychron, QMK, or VIA.</p>
        </section>
      </main>

      <footer className="site-footer"><div className="footer-brand"><img src="/assets/keysmith-mark.svg" alt="" /><strong>Keysmith</strong><span>Inspect locally. Plan deterministically. Trust transparently.</span></div><nav aria-label="Footer navigation"><a href={APP_REPO} target="_blank" rel="noreferrer"><Github aria-hidden="true" />Source</a><a href={FIRMWARE_REPO} target="_blank" rel="noreferrer"><Code2 aria-hidden="true" />Firmware</a><a href={`${APP_REPO}/blob/main/docs/V0_3_ATTENDED_PROTOCOL.md`} target="_blank" rel="noreferrer"><RotateCcw aria-hidden="true" />Protocol</a><a href={`${APP_REPO}/blob/main/SECURITY.md`} target="_blank" rel="noreferrer"><ShieldCheck aria-hidden="true" />Security</a></nav><p>Independent project for the Keychron Q3 Max · Application MIT · Firmware repository contains GPL source; binary distribution carries GPLv3 obligations</p></footer>
      <p className="sr-only" role="status" aria-live="polite">{copyStatus}</p>
    </>
  );
}
