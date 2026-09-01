#!/usr/bin/env node

import { createHash } from "node:crypto";
import { lstat, readFile, readdir } from "node:fs/promises";
import path from "node:path";

const EXPECTED_SCHEMA = "keysmith.public-release/v1";
const EXPECTED_REPOSITORY = "https://github.com/karti-ai/keysmith";
const SHA_PATTERN = /^[0-9a-f]{40}$/;
const HASH_PATTERN = /^[0-9a-f]{64}$/;

function fail(message) {
  throw new Error(message);
}

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith("--") || value === undefined) {
      fail("usage: check-static.mjs [--dir <dist/client>] [--url <https://host>] [--sha <40-hex-sha>]");
    }
    args[key.slice(2)] = value;
  }
  if (!args.dir && !args.url) fail("at least one of --dir or --url is required");
  if (args.sha && !SHA_PATTERN.test(args.sha)) fail("--sha must be exactly 40 lowercase hexadecimal characters");
  return args;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function walk(root, relative = "") {
  const entries = await readdir(path.join(root, relative), { withFileTypes: true });
  const files = [];
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    const child = path.posix.join(relative.split(path.sep).join(path.posix.sep), entry.name);
    const absolute = path.join(root, child);
    const metadata = await lstat(absolute);
    if (metadata.isSymbolicLink()) fail(`release contains a symlink: ${child}`);
    if (metadata.isDirectory()) files.push(...(await walk(root, child)));
    else if (metadata.isFile()) files.push(child);
    else fail(`release contains a non-regular entry: ${child}`);
  }
  return files;
}

function assertSafeRelative(relative) {
  if (
    !relative ||
    relative !== relative.trim() ||
    relative.startsWith("/") ||
    relative.includes("\\") ||
    relative.split("/").includes("..") ||
    /[\x00-\x1f\x7f]/.test(relative)
  ) {
    fail(`unsafe release path: ${JSON.stringify(relative)}`);
  }
}

function parseSums(contents) {
  const sums = new Map();
  for (const line of contents.trimEnd().split("\n")) {
    const match = line.match(/^([0-9a-f]{64})  (.+)$/);
    if (!match) fail(`invalid SHA256SUMS line: ${line}`);
    const [, hash, relative] = match;
    assertSafeRelative(relative);
    if (sums.has(relative)) fail(`duplicate SHA256SUMS path: ${relative}`);
    sums.set(relative, hash);
  }
  return sums;
}

function assertNoPrivateOrActiveContent(relative, contents) {
  const genericPrivatePatterns = [
    [/-----BEGIN [A-Z ]*PRIVATE KEY-----/, "private key material"],
    [/\bgh[pousr]_[A-Za-z0-9_]{20,}\b/, "GitHub credential-shaped value"],
    [/\bAKIA[0-9A-Z]{16}\b/, "AWS credential-shaped value"],
    [/\b100\.(?:6[4-9]|[7-9][0-9]|1[01][0-9]|12[0-7])\.(?:[0-9]{1,3})\.(?:[0-9]{1,3})\b/, "private CGNAT address"],
    [/[A-Za-z0-9.-]+\.ts\.net\b/i, "private Tailscale hostname"],
    [/(?:^|["'\s])\/home\/[A-Za-z0-9._-]+\//, "absolute home-directory path"],
    [/\/dev\/hidraw[0-9]*/i, "local HID device path"],
    [/https?:\/\/(?:127\.0\.0\.1|localhost)(?::[0-9]+)?/i, "loopback service URL"],
  ];
  for (const [pattern, label] of genericPrivatePatterns) {
    if (pattern.test(contents)) fail(`${relative} contains ${label}`);
  }

  if (/\.(?:js|mjs|html)$/i.test(relative)) {
    let executableContents = contents;
    // Vite emits one leading compatibility IIFE that fetches only the href of
    // same-origin <link rel="modulepreload"> elements in older browsers. It
    // is not an application data channel, and the production CSP still has
    // connect-src 'none'. Ignore only that exact leading bootstrap shape.
    if (/\.js$/i.test(relative) && executableContents.startsWith("(function(){")) {
      const bootstrapEnd = executableContents.indexOf("})();");
      const bootstrap = bootstrapEnd === -1 ? "" : executableContents.slice(0, bootstrapEnd + 5);
      if (
        bootstrap.includes('supports("modulepreload")') &&
        bootstrap.includes("MutationObserver") &&
        /fetch\([A-Za-z_$][\w$]*\.href,[A-Za-z_$][\w$]*\)/.test(bootstrap)
      ) {
        executableContents = executableContents.slice(bootstrapEnd + 5);
      }
    }
    const activePatterns = [
      [/(?:^|[^A-Za-z])fetch\s*\(/, "fetch()"],
      [/new\s+XMLHttpRequest\s*\(/, "XMLHttpRequest"],
      [/new\s+WebSocket\s*\(/, "WebSocket"],
      [/new\s+EventSource\s*\(/, "EventSource"],
      [/navigator\.sendBeacon\s*\(/, "sendBeacon"],
      [/navigator\.(?:hid|usb|serial|bluetooth)\b/i, "browser hardware API"],
      [/["'`]\/api(?:\/|["'`])/, "same-origin API path"],
    ];
    for (const [pattern, label] of activePatterns) {
      if (pattern.test(executableContents)) fail(`${relative} contains forbidden active capability ${label}`);
    }
  }

  if (/\.css$/i.test(relative) && /url\(\s*["']?(?:https?:)?\/\//i.test(contents)) {
    fail(`${relative} loads an external CSS resource`);
  }
}

function referencedSubresources(html) {
  const resources = [];
  const tagPattern = /<(script|img|source|iframe|link)\b[^>]*>/gi;
  for (const match of html.matchAll(tagPattern)) {
    const tag = match[0];
    const kind = match[1].toLowerCase();
    if (kind === "link" && !/\brel=["'](?:stylesheet|preload|modulepreload|icon|manifest)["']/i.test(tag)) continue;
    const attribute = tag.match(/\b(?:src|href)=["']([^"']+)["']/i);
    if (attribute) resources.push(attribute[1]);
  }
  return resources;
}

async function checkDirectory(directory, expectedSha) {
  const root = path.resolve(directory);
  const files = await walk(root);
  for (const required of ["index.html", "release.json", "SHA256SUMS"]) {
    if (!files.includes(required)) fail(`release is missing ${required}`);
  }

  const sums = parseSums(await readFile(path.join(root, "SHA256SUMS"), "utf8"));
  const expectedSummedFiles = files.filter((file) => file !== "SHA256SUMS").sort();
  if (JSON.stringify([...sums.keys()].sort()) !== JSON.stringify(expectedSummedFiles)) {
    fail("SHA256SUMS does not cover every release file exactly once");
  }
  for (const [relative, expectedHash] of sums) {
    const actualHash = sha256(await readFile(path.join(root, relative)));
    if (actualHash !== expectedHash) fail(`checksum mismatch for ${relative}`);
  }

  const manifest = JSON.parse(await readFile(path.join(root, "release.json"), "utf8"));
  if (manifest.schema !== EXPECTED_SCHEMA) fail(`unexpected release schema: ${manifest.schema}`);
  if (manifest.source?.repository !== EXPECTED_REPOSITORY) fail("release manifest has the wrong source repository");
  if (!SHA_PATTERN.test(manifest.source?.commit ?? "")) fail("release manifest source commit is invalid");
  if (!SHA_PATTERN.test(manifest.source?.tree ?? "")) fail("release manifest source tree is invalid");
  if (expectedSha && manifest.source.commit !== expectedSha) fail("release manifest does not match --sha");
  if (!HASH_PATTERN.test(manifest.deployment?.caddy_sha256 ?? "")) fail("release manifest Caddy hash is invalid");
  const checkedInCaddy = await readFile(new URL("Caddyfile", import.meta.url));
  if (manifest.deployment.caddy_sha256 !== sha256(checkedInCaddy)) {
    fail("release manifest does not match the checked-in Caddy fragment");
  }

  const contentFiles = files.filter((file) => file !== "release.json" && file !== "SHA256SUMS").sort();
  const manifestFiles = manifest.artifact?.files ?? [];
  if (manifest.artifact?.file_count !== contentFiles.length || manifestFiles.length !== contentFiles.length) {
    fail("release manifest file count is wrong");
  }
  if (JSON.stringify(manifestFiles.map((file) => file.path).sort()) !== JSON.stringify(contentFiles)) {
    fail("release manifest does not cover every content file exactly once");
  }
  for (const file of manifestFiles) {
    assertSafeRelative(file.path);
    if (!HASH_PATTERN.test(file.sha256)) fail(`manifest hash is invalid for ${file.path}`);
    const bytes = await readFile(path.join(root, file.path));
    if (bytes.length !== file.bytes || sha256(bytes) !== file.sha256) fail(`manifest metadata mismatch for ${file.path}`);
  }
  if (manifest.artifact?.total_bytes !== manifestFiles.reduce((total, file) => total + file.bytes, 0)) {
    fail("release manifest total byte count is wrong");
  }

  const textExtensions = new Set([".css", ".html", ".js", ".json", ".mjs", ".svg", ".txt", ".webmanifest", ".xml"]);
  for (const relative of files) {
    if (!textExtensions.has(path.extname(relative).toLowerCase())) continue;
    assertNoPrivateOrActiveContent(relative, await readFile(path.join(root, relative), "utf8"));
  }

  const index = await readFile(path.join(root, "index.html"), "utf8");
  if (index.length < 300 || !/keysmith/i.test(index)) fail("index.html is not a meaningful Keysmith document");
  for (const resource of referencedSubresources(index)) {
    if (/^(?:https?:)?\/\//i.test(resource)) fail(`index.html loads an external resource: ${resource}`);
    if (/^(?:data:|#)/i.test(resource)) continue;
    const relative = resource.replace(/^\//, "").split(/[?#]/, 1)[0];
    assertSafeRelative(relative);
    if (!files.includes(relative)) fail(`index.html references missing resource: ${relative}`);
    if (/\.(?:css|js)$/i.test(relative) && !/^assets\/.+-[A-Za-z0-9_-]{6,}\.(?:css|js)$/i.test(relative)) {
      fail(`cacheable code asset is not content-hashed: ${relative}`);
    }
  }

  console.log(`local static QA passed: ${root} (${files.length} files, source ${manifest.source.commit})`);
}

function requireHeader(response, name, expected) {
  const value = response.headers.get(name);
  if (!value || (expected instanceof RegExp ? !expected.test(value) : !value.toLowerCase().includes(expected.toLowerCase()))) {
    fail(`${response.url} has invalid ${name}: ${value ?? "missing"}`);
  }
  return value;
}

async function checkPublic(rawUrl, expectedSha) {
  const base = new URL(rawUrl);
  if (base.protocol !== "https:" || base.pathname !== "/" || base.search || base.hash) {
    fail("--url must be an HTTPS origin ending in /");
  }

  const httpUrl = new URL(base);
  httpUrl.protocol = "http:";
  const redirect = await fetch(httpUrl, { redirect: "manual" });
  if (![301, 308].includes(redirect.status) || redirect.headers.get("location") !== base.href) {
    fail(`HTTP does not permanently redirect to ${base.href}`);
  }

  const root = await fetch(base, { redirect: "error" });
  if (root.status !== 200) fail(`${base.href} returned HTTP ${root.status}`);
  requireHeader(root, "strict-transport-security", /max-age=31536000/i);
  requireHeader(root, "x-frame-options", /^DENY$/i);
  requireHeader(root, "x-content-type-options", /^nosniff$/i);
  requireHeader(root, "referrer-policy", /strict-origin-when-cross-origin/i);
  requireHeader(root, "cross-origin-opener-policy", /^same-origin$/i);
  requireHeader(root, "cross-origin-resource-policy", /^same-origin$/i);
  requireHeader(root, "cache-control", /no-cache/i);
  requireHeader(root, "x-robots-tag", /noindex/i);
  if (root.headers.has("server")) fail("public response exposes a Server header");
  if (root.headers.has("access-control-allow-origin")) fail("static site unexpectedly enables CORS");

  const csp = requireHeader(root, "content-security-policy", /default-src 'self'/i);
  for (const directive of ["connect-src 'none'", "object-src 'none'", "frame-ancestors 'none'", "base-uri 'none'", "form-action 'none'"]) {
    if (!csp.includes(directive)) fail(`CSP is missing ${directive}`);
  }
  const permissions = requireHeader(root, "permissions-policy", /usb=\(\)/i);
  for (const feature of [
    "accelerometer",
    "autoplay",
    "bluetooth",
    "camera",
    "display-capture",
    "encrypted-media",
    "fullscreen",
    "gamepad",
    "geolocation",
    "gyroscope",
    "hid",
    "magnetometer",
    "microphone",
    "midi",
    "payment",
    "picture-in-picture",
    "publickey-credentials-get",
    "screen-wake-lock",
    "serial",
    "speaker-selection",
    "usb",
    "xr-spatial-tracking",
  ]) {
    if (!permissions.includes(`${feature}=()`)) fail(`Permissions-Policy does not disable ${feature}`);
  }

  const html = await root.text();
  if (html.length < 300 || !/keysmith/i.test(html)) fail("public root is not a meaningful Keysmith page");
  const asset = referencedSubresources(html).find((resource) => /^\/assets\/.+\.(?:css|js)(?:[?#]|$)/i.test(resource));
  if (!asset) fail("public index does not reference a hashed asset");
  const assetResponse = await fetch(new URL(asset, base), { redirect: "error" });
  if (assetResponse.status !== 200) fail(`public asset returned HTTP ${assetResponse.status}`);
  requireHeader(assetResponse, "cache-control", /max-age=31536000.*immutable/i);

  const stableAsset = referencedSubresources(html).find(
    (resource) => /^\/assets\//i.test(resource) && !/\.(?:css|js)(?:[?#]|$)/i.test(resource),
  );
  if (!stableAsset) fail("public index does not reference a stable-name asset");
  const stableAssetResponse = await fetch(new URL(stableAsset, base), { redirect: "error" });
  if (stableAssetResponse.status !== 200) fail(`stable public asset returned HTTP ${stableAssetResponse.status}`);
  requireHeader(stableAssetResponse, "cache-control", /no-cache/i);

  const manifestResponse = await fetch(new URL("release.json", base), { redirect: "error" });
  if (manifestResponse.status !== 200) fail(`release.json returned HTTP ${manifestResponse.status}`);
  requireHeader(manifestResponse, "cache-control", /no-cache/i);
  const manifest = await manifestResponse.json();
  if (
    manifest.schema !== EXPECTED_SCHEMA ||
    !SHA_PATTERN.test(manifest.source?.commit ?? "") ||
    !HASH_PATTERN.test(manifest.deployment?.caddy_sha256 ?? "")
  ) {
    fail("public release.json has invalid provenance");
  }
  if (expectedSha && manifest.source.commit !== expectedSha) fail("public release does not match --sha");

  const probes = [
    ["GET", "api/inspect", 404],
    ["POST", "api/plans/preview", 404],
    ["POST", "", 405],
    ["GET", `definitely-not-a-real-page-${Date.now()}`, 404],
  ];
  for (const [method, relative, status] of probes) {
    const response = await fetch(new URL(relative, base), { method, redirect: "manual" });
    if (response.status !== status) fail(`${method} /${relative} returned ${response.status}; expected ${status}`);
  }

  console.log(`public smoke passed: ${base.href} (source ${manifest.source.commit})`);
}

const args = parseArgs(process.argv.slice(2));
if (args.dir) await checkDirectory(args.dir, args.sha);
if (args.url) await checkPublic(args.url, args.sha);
