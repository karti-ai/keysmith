#!/usr/bin/env node

import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { lstat, readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SCHEMA = "keysmith.public-release/v1";
const PUBLIC_REPOSITORY = "https://github.com/karti-ai/keysmith";
const SHA_PATTERN = /^[0-9a-f]{40}$/;

function usage() {
  console.error("usage: generate-release-manifest.mjs --dir <apps/site/dist/client> --source-sha <40-hex-sha>");
  process.exit(2);
}

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith("--") || value === undefined) usage();
    args[key.slice(2)] = value;
  }
  if (!args.dir || !args["source-sha"]) usage();
  return args;
}

function git(...args) {
  return execFileSync("git", args, { encoding: "utf8" }).trim();
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function assertSafePath(relative) {
  if (
    !relative ||
    relative !== relative.trim() ||
    relative.startsWith("/") ||
    relative.includes("\\") ||
    relative.split("/").includes("..") ||
    /[\x00-\x1f\x7f]/.test(relative)
  ) {
    throw new Error(`refusing unsafe release path: ${JSON.stringify(relative)}`);
  }
}

async function collectFiles(root, relative = "") {
  const directory = path.join(root, relative);
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];

  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    const child = path.posix.join(relative.split(path.sep).join(path.posix.sep), entry.name);
    assertSafePath(child);
    const absolute = path.join(root, child);
    const metadata = await lstat(absolute);
    if (metadata.isSymbolicLink()) {
      throw new Error(`refusing release symlink: ${child}`);
    }
    if (metadata.isDirectory()) {
      files.push(...(await collectFiles(root, child)));
    } else if (metadata.isFile() && child !== "release.json" && child !== "SHA256SUMS") {
      const contents = await readFile(absolute);
      files.push({ path: child, bytes: contents.length, sha256: sha256(contents) });
    } else if (!metadata.isFile()) {
      throw new Error(`refusing non-regular release entry: ${child}`);
    }
  }
  return files;
}

const args = parseArgs(process.argv.slice(2));
const root = path.resolve(args.dir);
const sourceSha = args["source-sha"];
if (!SHA_PATTERN.test(sourceSha)) throw new Error("source SHA must be exactly 40 lowercase hexadecimal characters");

const head = git("rev-parse", "HEAD").toLowerCase();
if (head !== sourceSha) throw new Error(`source SHA ${sourceSha} does not match checked-out HEAD ${head}`);

const rootMetadata = await lstat(root);
if (!rootMetadata.isDirectory() || rootMetadata.isSymbolicLink()) {
  throw new Error(`release root is not a real directory: ${root}`);
}
const indexMetadata = await lstat(path.join(root, "index.html"));
if (!indexMetadata.isFile() || indexMetadata.isSymbolicLink()) {
  throw new Error("release root must contain a regular index.html");
}

const files = await collectFiles(root);
const sourceTree = git("rev-parse", `${sourceSha}^{tree}`).toLowerCase();
const sourceDate = git("show", "-s", "--format=%cI", sourceSha);
const caddyfile = await readFile(path.join(path.dirname(fileURLToPath(import.meta.url)), "Caddyfile"));
const manifest = {
  schema: SCHEMA,
  source: {
    repository: PUBLIC_REPOSITORY,
    commit: sourceSha,
    tree: sourceTree,
    committed_at: sourceDate,
  },
  artifact: {
    file_count: files.length,
    total_bytes: files.reduce((total, file) => total + file.bytes, 0),
    files,
  },
  deployment: {
    caddy_sha256: sha256(caddyfile),
  },
};

const manifestBytes = `${JSON.stringify(manifest, null, 2)}\n`;
await writeFile(path.join(root, "release.json"), manifestBytes, { encoding: "utf8", mode: 0o644 });

const sums = [
  ...files.map((file) => `${file.sha256}  ${file.path}`),
  `${sha256(Buffer.from(manifestBytes))}  release.json`,
].join("\n");
await writeFile(path.join(root, "SHA256SUMS"), `${sums}\n`, { encoding: "utf8", mode: 0o644 });

console.log(`generated ${path.join(root, "release.json")} for ${sourceSha} (${files.length} content files)`);
