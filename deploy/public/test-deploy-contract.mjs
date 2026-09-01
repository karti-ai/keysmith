#!/usr/bin/env node

import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const caddyfile = await readFile(path.join(here, "Caddyfile"), "utf8");
const deploy = await readFile(path.join(here, "deploy.sh"), "utf8");
const installerSource = await readFile(path.join(here, "install-caddy.sh"), "utf8");

assert.doesNotMatch(caddyfile, /^\s*reverse_proxy\b/m);
assert.match(caddyfile, /connect-src 'none'/);
assert.doesNotMatch(caddyfile, /unsafe-inline/);
assert.match(caddyfile, /respond @api 404/);
assert.match(caddyfile, /respond @non_read 405/);
assert.doesNotMatch(caddyfile, /try_files\s+\{path\}\s+\/index\.html/);
assert.match(deploy, /if ! public_smoke "\$source_sha"; then[\s\S]*restore_deployment "\$old_sha"/);
assert.match(
  deploy,
  /restore_deployment\(\)[\s\S]*remote_restore_pointer "\$old_sha" "\$old_previous"[\s\S]*restore_last_caddy/,
);

const caddyProbe = spawnSync("caddy", ["version"], { encoding: "utf8" });
if (caddyProbe.status === 0) {
  const validationRoot = await mkdtemp(path.join(os.tmpdir(), "keysmith-caddy-validate-"));
  try {
    const validationConfig = path.join(validationRoot, "Caddyfile");
    await writeFile(
      validationConfig,
      `(sec) {\n  header {\n    Strict-Transport-Security "max-age=31536000; includeSubDomains"\n    X-Frame-Options "DENY"\n    X-Content-Type-Options "nosniff"\n    Referrer-Policy "strict-origin-when-cross-origin"\n    -Server\n  }\n}\n\n${caddyfile}`,
    );
    run("caddy", ["validate", "--config", validationConfig]);
  } finally {
    await rm(validationRoot, { recursive: true, force: true });
  }
} else {
  console.log("caddy binary unavailable; production installer will perform full-context validation");
}

function shellQuote(value) {
  return `'${value.replaceAll("'", `'"'"'`)}'`;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { encoding: "utf8", ...options });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed (${result.status})\n${result.stdout}\n${result.stderr}`);
  }
  return result;
}

async function exerciseRollback({ initialMain, initialFragment, nextFragment }) {
  const root = await mkdtemp(path.join(os.tmpdir(), "keysmith-deploy-contract-"));
  try {
    const etc = path.join(root, "etc");
    const state = path.join(root, "state");
    const bin = path.join(root, "bin");
    const main = path.join(etc, "Caddyfile");
    const live = path.join(etc, "keychron.karti.ai.caddy");
    const candidate = path.join(root, "candidate.caddy");
    const installer = path.join(root, "install-caddy.sh");
    const renderedInitialMain = initialMain.replaceAll("TEST_LIVE", live);
    await mkdir(etc, { recursive: true });
    await mkdir(bin, { recursive: true });
    await writeFile(main, renderedInitialMain);
    if (initialFragment !== null) await writeFile(live, initialFragment);
    await writeFile(candidate, nextFragment);

    // The production script deliberately hard-codes its privileged paths.
    // This dry-run copy redirects only those constants into an isolated temp
    // tree, removes sudo, and accepts the one test candidate path.
    const transformed = installerSource
      .replace(/^MAIN=.*$/m, `MAIN=${shellQuote(main)}`)
      .replace(/^LIVE_FRAGMENT=.*$/m, `LIVE_FRAGMENT=${shellQuote(live)}`)
      .replace(/^STATE_DIR=.*$/m, `STATE_DIR=${shellQuote(state)}`)
      .replaceAll("sudo ", "")
      .replaceAll("-o root -g root ", "")
      .replace(
        'if [[ ! "$CANDIDATE_FRAGMENT" =~ ^/var/www/keychron\\.karti\\.ai/configs/[0-9a-f]{40}\\.caddy$ ]]; then',
        'if [[ "$CANDIDATE_FRAGMENT" != "$KEYSMITH_TEST_CANDIDATE" ]]; then',
      );
    assert.notEqual(transformed, installerSource);
    assert.match(transformed, /KEYSMITH_TEST_CANDIDATE/);
    await writeFile(installer, transformed, { mode: 0o700 });

    await writeFile(
      path.join(bin, "caddy"),
      "#!/usr/bin/env bash\nset -euo pipefail\n[[ $1 == validate && $2 == --config && -f $3 ]]\n",
      { mode: 0o700 },
    );
    await writeFile(
      path.join(bin, "systemctl"),
      "#!/usr/bin/env bash\nset -euo pipefail\n[[ $1 == reload || $1 == is-active ]]\n",
      { mode: 0o700 },
    );

    const env = {
      ...process.env,
      PATH: `${bin}:${process.env.PATH}`,
      KEYSMITH_TEST_CANDIDATE: candidate,
    };
    run("bash", [installer, candidate], { env });
    assert.equal(await readFile(live, "utf8"), nextFragment);
    assert.ok((await readFile(main, "utf8")).split("\n").includes(`import ${live}`));

    // Model the exact point requested by the production rollback contract:
    // Caddy installation/reload succeeded, then the public smoke check failed.
    run("bash", [installer, "--restore-last"], { env });
    assert.equal(await readFile(main, "utf8"), renderedInitialMain);
    if (initialFragment === null) {
      const probe = spawnSync("test", ["-e", live]);
      assert.notEqual(probe.status, 0, "failed first launch left a live Caddy fragment");
    } else {
      assert.equal(await readFile(live, "utf8"), initialFragment);
    }
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

const requiredStaticFragment = `keychron.karti.ai {
  header Content-Security-Policy "connect-src 'none'"
  @api path /api /api/*
  respond @api 404
  @non_read not method GET HEAD
  respond @non_read 405
}
`;

await exerciseRollback({
  initialMain: "(sec) {\n  header -Server\n}\n",
  initialFragment: null,
  nextFragment: requiredStaticFragment,
});

const priorFragment = `${requiredStaticFragment}# prior release\n`;
const priorMain = `(sec) {\n  header -Server\n}\nimport TEST_LIVE\n`;
// Substitute the temp live path inside the harness after it redirects MAIN.
// The installer preserves the bytes regardless of what another Caddy site is.
await exerciseRollback({
  initialMain: priorMain,
  initialFragment: priorFragment,
  nextFragment: `${requiredStaticFragment}# next release\n`,
});

const releaseFixture = await mkdtemp(path.join(os.tmpdir(), "keysmith-release-fixture-"));
try {
  await mkdir(path.join(releaseFixture, "assets"));
  await writeFile(
    path.join(releaseFixture, "index.html"),
    `<!doctype html><html><head><title>Keysmith release fixture</title></head><body><main>${"safe static content ".repeat(20)}</main><script type="module" src="/assets/app-abcdef12.js"></script></body></html>`,
  );
  await writeFile(
    path.join(releaseFixture, "assets/app-abcdef12.js"),
    'document.documentElement.dataset.keysmith = "static";\n',
  );
  const sourceSha = run("git", ["rev-parse", "HEAD"], { cwd: path.join(here, "../..") }).stdout.trim();
  const generator = path.join(here, "generate-release-manifest.mjs");
  const checker = path.join(here, "check-static.mjs");
  run("node", [generator, "--dir", releaseFixture, "--source-sha", sourceSha], {
    cwd: path.join(here, "../.."),
  });
  const firstManifest = await readFile(path.join(releaseFixture, "release.json"), "utf8");
  const firstSums = await readFile(path.join(releaseFixture, "SHA256SUMS"), "utf8");
  run("node", [checker, "--dir", releaseFixture, "--sha", sourceSha]);
  run("node", [generator, "--dir", releaseFixture, "--source-sha", sourceSha], {
    cwd: path.join(here, "../.."),
  });
  assert.equal(await readFile(path.join(releaseFixture, "release.json"), "utf8"), firstManifest);
  assert.equal(await readFile(path.join(releaseFixture, "SHA256SUMS"), "utf8"), firstSums);

  await writeFile(path.join(releaseFixture, "assets/app-abcdef12.js"), 'fetch("/api/inspect");\n');
  run("node", [generator, "--dir", releaseFixture, "--source-sha", sourceSha], {
    cwd: path.join(here, "../.."),
  });
  const unsafeCheck = spawnSync("node", [checker, "--dir", releaseFixture, "--sha", sourceSha], {
    encoding: "utf8",
  });
  assert.notEqual(unsafeCheck.status, 0, "static checker accepted an application fetch/API call");
  assert.match(unsafeCheck.stderr, /forbidden active capability fetch\(\)/);
} finally {
  await rm(releaseFixture, { recursive: true, force: true });
}

console.log("deployment contract QA passed (static boundary + exact Caddy rollback + deterministic provenance)");
