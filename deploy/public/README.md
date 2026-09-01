# keychron.karti.ai production runbook

`keychron.karti.ai` is a public **static project site**. It serves only the
client output from `apps/site/dist/client`; it never proxies the local Keysmith
server, USB inspection, plans, firmware, DFU, or any mutation path.

The live control application remains on its authenticated private-network
surface. Do not add a `reverse_proxy`, CORS, a GitHub Pages deployment, or an
API hostname to this deployment.

## Production contract

| Item | Value |
|---|---|
| Canonical public source | `https://github.com/karti-ai/keysmith` |
| Build output | `apps/site/dist/client` |
| Public origin | `https://keychron.karti.ai/` |
| Deployment host | SSH alias `cloud-2` |
| Release root | `/var/www/keychron.karti.ai/releases/<40-character-sha>` |
| Active release | Atomic symlink `/var/www/keychron.karti.ai/current` |
| Previous release | `/var/www/keychron.karti.ai/previous` |
| Versioned Caddy fragments | `/var/www/keychron.karti.ai/configs/<sha>.caddy` |
| Live Caddy fragment | `/etc/caddy/keychron.karti.ai.caddy` |
| DNS | Oracle DNS `A keychron.karti.ai` to cloud-2, TTL 300 |

The deploy script does not create, update, or delete DNS. DNS remains an
explicit operator action through the existing authenticated Oracle CLI path.

## Safety properties

- Caddy serves files only and contains no upstream.
- `/api` is always 404.
- Methods other than GET and HEAD are 405.
- Unknown paths are real 404 responses; there is no blanket SPA fallback.
- CSP has no inline-script/style exception, uses `connect-src 'none'`, and
  blocks objects, framing, forms, workers, and external base URLs.
- Permissions Policy disables USB, HID, serial, Bluetooth, camera, microphone,
  geolocation, payment, and display capture.
- The fleet `sec` snippet supplies HSTS, frame denial, MIME protection,
  Referrer Policy, and removal of the Server header. The site adds COOP/CORP
  and removes Server again defensively.
- Vite's content-hashed JavaScript and CSS files cache immutably for one year.
  HTML, stable-name images, manifests, checksum files, and all other documents
  revalidate.
- The first release is `noindex, nofollow` and must visibly retain the project's
  independent/non-affiliated positioning before indexing is reconsidered.

## Prerequisites

On the operator machine:

- clean checkout of the exact public GitHub commit to release;
- Node 24, npm, Git, rsync, SSH, and SHA-256 tooling;
- `cloud-2` SSH access in batch mode.

On cloud-2:

- Caddy 2.11 or newer, active as the existing system service;
- the shared `(sec)` snippet in `/etc/caddy/Caddyfile`;
- non-interactive sudo for Caddy validation/reload and bounded file installs;
- the public VNIC bind used by the checked-in fragment available to Caddy.

The guarded installer refuses an existing inline `keychron.karti.ai` block,
duplicate imports, symlinks in place of configuration files, any candidate
containing `reverse_proxy`, or a candidate missing its static-boundary rules.
On first use it adds exactly one import for the separately managed fragment.

## Build and local QA

From the repository root, while the checkout is clean and at the exact commit:

```bash
source_sha=$(git rev-parse HEAD)
npm --prefix apps/site ci
npm --prefix apps/site audit --audit-level=high
npm --prefix apps/site run test:sites
npm --prefix apps/site run build
npm --prefix apps/site run test:qa
node deploy/public/generate-release-manifest.mjs \
  --dir apps/site/dist/client \
  --source-sha "$source_sha"
node deploy/public/check-static.mjs \
  --dir apps/site/dist/client \
  --sha "$source_sha"
```

The manifest records the public repository, commit, tree, commit timestamp,
content hashes, byte counts, and the exact Caddy-fragment hash. `SHA256SUMS`
covers every deployed file except itself. Generation is deterministic for an
unchanged commit and build output. Remote verification binds the preserved
versioned Caddy fragment to that manifest before activation or rollback.

The static checker refuses:

- missing or extra checksum entries;
- symlinks and non-regular files;
- private keys, credential-shaped values, CGNAT addresses, Tailnet hostnames,
  local home/device paths, or loopback service URLs;
- `fetch`, XHR, WebSocket, EventSource, beacon, browser hardware APIs, and
  same-origin `/api` calls in executable assets;
- external script, stylesheet, image, iframe, or CSS resource loads;
- unhashed JavaScript or CSS entry assets.

## First launch

1. Complete local QA and record the exact 40-character commit.
2. Confirm `keychron.karti.ai` is not already configured in Caddy or DNS.
3. Add the exact Oracle DNS A record with TTL 300. There is no wildcard; wait
   until every authoritative nameserver and at least two public resolvers
   return the intended cloud-2 address. Negative answers may remain cached for
   up to 1,800 seconds.
4. Run the guarded deployment:

   ```bash
   deploy/public/deploy.sh deploy "$(git rev-parse HEAD)"
   ```

5. The script rebuilds from the clean checkout, verifies the release locally
   and again on cloud-2, stages it in an immutable release directory, switches
   `current` atomically, validates the candidate Caddy fragment in the context
   of the full shared Caddyfile, installs it, validates production again, and
   **reloads** Caddy. It never restarts Caddy.
6. The final public smoke test requires HTTPS, exact provenance, the complete
   header policy, immutable asset caching, revalidated documents, `/api` denial,
   non-GET denial, an unknown-path 404, no CORS, and no Server header.

Always verify using the real public hostname from another machine. A local
`--resolve` test can miss cloud-2's public-bind routing class of failures.

## Routine releases

```bash
git status --short
git rev-parse HEAD
deploy/public/deploy.sh deploy <exact-40-character-sha>
deploy/public/deploy.sh status
```

The script refuses a dirty tree, a SHA other than checked-out `HEAD`, a changed
artifact for an already-existing immutable SHA, and a mismatched remote
manifest. A token-owned remote lock serializes deployments and rollbacks. If an
operator process is killed before cleanup, inspect
`/var/www/keychron.karti.ai/.deploy-lock/owner` and confirm no deployment is
running before removing that stale lock. Releases are not automatically pruned;
retaining them keeps rollback evidence and avoids deleting the only known-good
release. Release files and versioned Caddy fragments are write-protected after
upload; activation never edits an existing release directory.

## Explicit rollback

List the preserved release directories and select an exact known-good SHA.
Then run:

```bash
deploy/public/deploy.sh rollback <exact-existing-40-character-sha>
```

Rollback verifies that release's checksums and manifest, atomically moves the
`current` symlink, installs its preserved Caddy fragment, validates and reloads
Caddy, and runs the same public smoke test. The release being replaced becomes
`previous`.

If a normal deployment fails Caddy validation, reload, or public smoke, the
script restores the former `current` and `previous` targets automatically.
Before every
successful Caddy install, the guarded installer records the exact main
Caddyfile, whether the managed fragment existed, and its exact bytes in the
root-only `/var/lib/keysmith-public-deploy` state directory. A post-install
smoke failure restores that pair, validates it, and reloads it. On a failed
first launch this removes both the newly added import and live fragment, so the
configuration returns byte-for-byte to its pre-launch state. Caddy installation
also performs the same exact rollback itself if validation or reload fails.

For a failed first launch with no prior release, restore the previous validated
Caddy configuration before removing the exact DNS A record. Never delete the
DNS record while an unreviewed host block still exists.

## GitHub CI boundary

The `public-site` CI job installs from the lockfile, audits high-severity npm
findings, tests the optional Sites worker bundle, builds the static client,
runs site QA, generates exact provenance, and checks the static-only boundary.
It does not request deployment permissions, configure Pages, store SSH/Oracle
credentials, or publish production artifacts.
