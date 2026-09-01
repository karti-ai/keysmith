#!/usr/bin/env node

import { spawn } from "node:child_process";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { chromium } from "playwright-core";

const host = "127.0.0.1";
const port = 4174;
const base = `http://${host}:${port}`;
const chrome = process.env.CHROME_PATH || "/usr/bin/google-chrome";
const screenshotDir = process.env.SITE_SCREENSHOT_DIR;
const preview = spawn(process.execPath, ["node_modules/vite/bin/vite.js", "preview", "--host", host, "--port", String(port)], {
  cwd: path.resolve(import.meta.dirname, ".."),
  stdio: ["ignore", "pipe", "pipe"],
});

let previewOutput = "";
preview.stdout.on("data", (chunk) => { previewOutput += chunk; });
preview.stderr.on("data", (chunk) => { previewOutput += chunk; });

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function waitForPreview() {
  for (let attempt = 0; attempt < 80; attempt += 1) {
    try {
      const response = await fetch(base);
      if (response.ok) return;
    } catch {
      // Vite has not bound the socket yet.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`preview did not start:\n${previewOutput}`);
}

async function checkViewport(browser, name, viewport) {
  const context = await browser.newContext({ viewport, reducedMotion: "reduce" });
  const page = await context.newPage();
  const failures = [];
  const requests = [];

  page.on("console", (message) => {
    if (message.type() === "error") failures.push(`console: ${message.text()}`);
  });
  page.on("pageerror", (error) => failures.push(`page: ${error.message}`));
  page.on("request", (request) => requests.push(request.url()));

  await page.goto(base, { waitUntil: "networkidle" });
  assert(await page.title() === "Keysmith — Agent-first control for the Keychron Q3 Max", `${name}: wrong title`);
  assert(await page.locator("h1").count() === 1, `${name}: expected one h1`);
  assert(await page.locator("main section").count() >= 9, `${name}: major sections are missing`);
  assert(await page.getByRole("heading", { name: "Thank you, Keychron." }).isVisible(), `${name}: gratitude section is missing`);
  assert(await page.getByText("No packaged installer", { exact: false }).first().isVisible(), `${name}: source-preview boundary is missing`);
  assert(await page.getByText("not affiliated with, sponsored by, reviewed by, or endorsed", { exact: false }).isVisible(), `${name}: independence disclaimer is missing`);
  assert(await page.locator("body").innerText().then((text) => !text.includes("keychronctl apply")), `${name}: invented apply command is present`);

  const imageFailures = await page.locator("img").evaluateAll((images) => images.filter((image) => image.naturalWidth === 0).map((image) => image.getAttribute("src")));
  assert(imageFailures.length === 0, `${name}: failed images: ${imageFailures.join(", ")}`);
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  assert(overflow <= 1, `${name}: horizontal overflow is ${overflow}px`);

  const localOrigin = new URL(base).origin;
  const foreignRuntimeRequests = requests.filter((url) => new URL(url).origin !== localOrigin);
  assert(foreignRuntimeRequests.length === 0, `${name}: external runtime requests: ${foreignRuntimeRequests.join(", ")}`);
  assert(!requests.some((url) => new URL(url).pathname.startsWith("/api")), `${name}: public page requested /api`);

  if (name === "mobile") {
    const menu = page.getByRole("button", { name: "Open navigation" });
    const box = await menu.boundingBox();
    assert(box && box.width >= 44 && box.height >= 44, "mobile: menu target is smaller than 44px");
    await menu.click();
    assert(await page.getByRole("link", { name: "Source setup" }).last().isVisible(), "mobile: navigation did not open");
  } else {
    await page.getByRole("tab", { name: "Firmware evidence" }).click();
    assert(await page.getByRole("heading", { name: "Source identity and recovery stay visible." }).isVisible(), `${name}: product tour tab did not change`);
  }

  if (screenshotDir) {
    await mkdir(screenshotDir, { recursive: true });
    await page.screenshot({ path: path.join(screenshotDir, `keysmith-site-${name}.png`), fullPage: true });
  }
  assert(failures.length === 0, `${name}: runtime errors:\n${failures.join("\n")}`);
  await context.close();
  return { name };
}

let browser;
try {
  await waitForPreview();
  browser = await chromium.launch({ executablePath: chrome, headless: true });
  const results = [];
  results.push(await checkViewport(browser, "desktop", { width: 1536, height: 960 }));
  results.push(await checkViewport(browser, "laptop", { width: 1280, height: 800 }));
  results.push(await checkViewport(browser, "mobile", { width: 390, height: 844 }));
  console.log(`site browser QA passed: ${results.map((result) => result.name).join(", ")}`);
} finally {
  if (browser) await browser.close();
  preview.kill("SIGTERM");
}
