# Keysmith public-site fidelity ledger

The three images in `concepts/` define the accepted **Attended Instrument**
direction. The browser captures in `screenshots/` are the production React
implementation at desktop, laptop, and mobile sizes.

| Comparison point | Concept evidence | Render evidence and decision |
|---|---|---|
| Hero hierarchy | Large two-line headline, real control surface, and the keyboard as one physical focal object. | The production hero preserves that composition, uses the checked-in control-center capture, and layers the Q3 Max-inspired firmware illustration below it. Copy and controls remain HTML. |
| Industrial palette | Near-black graphite, warm-white type, signal red, restrained verified green, and cool hairline borders. | CSS tokens lock those colors across the header, headings, state rails, screenshot frames, terminal, and footer. There are no rainbow, glass, or gamer-neon treatments. |
| Open editorial rhythm | Asymmetric bands and rails instead of a repeated bento grid. | Product model, agent contract, tour, safety sequence, source rows, roadmap, videos, gratitude, and footer deliberately change density while sharing one gutter and type system. |
| Product evidence | Large, recognizable Keysmith interface captures rather than invented metrics or a fake live demo. | All three checked-in UI captures are rendered with adjacent explanatory copy and explicit “reference capture” language. The public site performs no API or external runtime request. |
| Agent and safety boundary | The agent explains and prepares; the physical keyboard remains the authorization boundary. | The terminal is visibly preview-only, contains no apply command, and ends at offline preparation. The safety section distinguishes protocol-capable firmware from the non-executing public host. |
| Source setup | A numbered, evidence-first path with recovery and terminal context. | The production path is explicitly a source preview: host 0.1, firmware 0.3.0-candidate, no packaged installer, no firmware release, and official Keychron recovery links. |
| Closing story | Roadmap, planned field-note videos, a large Keychron thank-you, and an independence disclaimer. | The production close retains all four, marks videos “Coming soon,” promises captions/transcripts/source revisions, and avoids any endorsement-like Keychron lockup. |
| Mobile behavior | One-column continuation with touch-safe navigation and horizontally understandable technical sequences. | At 390×844, the menu and controls are at least 44 px, screenshots remain readable, long protocol rails scroll inside their own regions, focus is visible, and global horizontal overflow is clipped. |

## Intentional deviations

- Generated concepts contain approximate UI text. Production uses verified,
  code-native copy and real screenshots instead.
- One concept accidentally visualized an apply command. Production never
  includes that command and states that the public host has no executor.
- The release section says **Build with evidence** instead of presenting a
  download, because neither repository currently publishes a packaged release
  or firmware binary.
- The site launches `noindex` while the trademark-bearing hostname and
  independence presentation receive real-world review. It is still publicly
  reachable by URL.

## Verification

```bash
cd apps/site
npm ci
npm audit --audit-level=high
npm run build
npm run test:sites
SITE_SCREENSHOT_DIR=../../docs/site/screenshots npm run test:qa
```

The browser QA blocks external or `/api` requests, missing images, runtime
errors, an invented `keychronctl apply` command, absent independence/release
language, undersized mobile navigation, and page-level horizontal overflow.
