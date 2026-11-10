// Optional browser regression check. Requires Playwright plus Chromium. Run from repository root.
const { chromium } = require("playwright");
const { execFileSync, spawn } = require("child_process");
const fs = require("fs"),
  os = require("os"),
  path = require("path"),
  net = require("net");
const root = fs.mkdtempSync(path.join(os.tmpdir(), "citta-coverage-ui-")),
  dir = path.join(root, "node"),
  binary = path.resolve("target/debug/citta");
let daemon, browser;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
(async () => {
  fs.mkdirSync("local", { recursive: true, mode: 0o700 });
  const s = net.createServer();
  await new Promise((r) => s.listen(0, "127.0.0.1", r));
  const port = s.address().port;
  await new Promise((r) => s.close(r));
  execFileSync(binary, [
    "init",
    "--dir",
    dir,
    "--node",
    "qa-device",
    "--port",
    String(port),
  ]);
  const config = path.join(dir, "config.toml");
  daemon = spawn(binary, ["--config", config, "daemon"], { stdio: "ignore" });
  for (let i = 0; i < 80; i++) {
    try {
      const r = await fetch("http://127.0.0.1:" + port + "/healthz");
      if (r.ok) break;
    } catch {}
    await sleep(100);
  }
  browser = await chromium.launch({
    headless: true,
    executablePath: process.env.CITTA_BROWSER || undefined,
  });
  const p = await browser.newPage({ viewport: { width: 1440, height: 1100 } }),
    errors = [];
  p.on("pageerror", (e) => errors.push(e.message));
  const link = JSON.parse(
    execFileSync(binary, ["--config", config, "panel"], { encoding: "utf8" }),
  );
  await p.goto(link.url);
  await p.locator("#dashboard").waitFor({ state: "visible" });
  const fixture = await p.evaluate(() =>
    fetch("/panel/overview").then((r) => r.json()),
  );
  const now = () => Math.floor(Date.now() / 1000);
  const session = {
    id: "same-id",
    harness: "codex",
    model: "fixture-model",
    label: "Mini session",
    state: "idle",
    source: "codex_app_server",
    observed_at: now(),
    managed: false,
  };
  const node = fixture.discovery.local;
  node.node_id = "mac-mini";
  node.inventory.agents = [session];
  node.inventory.runtimes = [];
  node.inventory.observed_at = now();
  node.inventory.codex_status = "connected";
  const air = JSON.parse(JSON.stringify(node));
  air.node_id = "macbook-air";
  air.disk = {
    observed_at: now(),
    data_available_bytes: 0,
    work_available_bytes: 0,
    reserve_bytes: 67108864,
    accepting: false,
    reason: "disk_low",
  };
  air.inventory.agents = [];
  air.inventory.runtimes = [1, 2, 3, 4].map((pid) => ({
    pid,
    name: "codex",
    memory_bytes: 1024,
    cpu_usage_percent: 0,
  }));
  const linux = JSON.parse(JSON.stringify(node));
  linux.node_id = "linux";
  linux.inventory.agents = [{ ...session, label: "Linux session" }];
  fixture.discovery.peers = [
    { node_id: "macbook-air", reachable: true, node: air },
    { node_id: "linux", reachable: true, node: linux },
    { node_id: "offline-device", reachable: false },
    {
      node_id: "older-device",
      reachable: true,
      node: {
        node_id: "older-device",
        capabilities: [],
        managed_agents: [],
        slots: 1,
        active: 0,
      },
    },
  ];
  await p.route("**/panel/overview", (route) =>
    route.fulfill({ json: fixture }),
  );
  const refresh = async () => {
    await p.locator("#refresh").click();
    await p.waitForFunction(() => !document.querySelector("#refresh").disabled);
  };
  await refresh();
  const diskCard = p.locator(".device").filter({ hasText: "macbook-air" });
  if (!(await diskCard.locator(".disk-note").innerText()).includes("Low disk space"))
    throw Error("full disk is not explained before submission");
  air.disk.observed_at = now() - 120;
  await refresh();
  if (!(await diskCard.locator(".disk-note").innerText()).includes("not verified"))
    throw Error("stale disk readiness presented as current");
  air.disk.observed_at = now();
  air.disk.accepting = true;
  air.disk.reason = null;
  air.disk.data_available_bytes = air.disk.work_available_bytes = 1073741824;
  await refresh();
  if (!(await diskCard.locator(".disk-note").innerText()).includes("1.0 GB available"))
    throw Error("disk recovery is not shown");
  await p.locator("[data-view=agents]").click();
  if ((await p.locator(".coverage-card").count()) !== 5)
    throw Error("not every device has coverage");
  if ((await p.locator("#agents tr").count()) !== 2)
    throw Error("remote session dropped or processes inflated into sessions");
  if ((await p.locator("#runtimes .runtime").count()) !== 4)
    throw Error("Air processes missing");
  if (!(await p.locator(".runtime-section").isVisible()))
    throw Error("processes hidden");
  const airCard = p
    .locator(".coverage-card")
    .filter({ hasText: "macbook-air" });
  if (!(await airCard.innerText()).includes("no reported sessions"))
    throw Error("Air zero-session coverage ambiguous");
  if (
    (await p
      .locator(".stat")
      .filter({ hasText: "Reported live sessions" })
      .locator(".stat-value")
      .innerText()) !== "2"
  )
    throw Error("processes inflated live count");
  await airCard.locator("button").click();
  if (await p.locator("#agents tr").count())
    throw Error("device filter leaked Mini");
  if ((await p.locator("#runtimes .runtime").count()) !== 4)
    throw Error("filter lost Air");
  await p.locator("#agent-state").selectOption("idle");
  if (await p.locator("#runtimes .runtime").count())
    throw Error("process inferred idle");
  await p.locator("#agent-state").selectOption("unknown");
  if ((await p.locator("#runtimes .runtime").count()) !== 4)
    throw Error("unknown processes missing");
  await p.locator("#agent-search").fill("3");
  if ((await p.locator("#runtimes .runtime").count()) !== 1)
    throw Error("process search failed");
  await p.locator("#agent-search").fill("");
  await p.locator("#agent-state").selectOption("all");
  await p.locator("#agent-device").selectOption("all");
  air.inventory.observed_at = now() - 120;
  await refresh();
  if (!(await airCard.innerText()).includes("Stale discovery"))
    throw Error("stale coverage claims fresh");
  if (!(await p.locator("#runtimes").innerText()).includes("Stale observation"))
    throw Error("stale runtime presented live");
  air.inventory.observed_at = now();
  air.inventory.runtimes[0].name = "<img src=x onerror=alert(1)>";
  await refresh();
  if (await p.locator("#runtimes img").count())
    throw Error("unsafe runtime rendering");
  air.inventory.runtimes[0].name = "codex";
  await refresh();
  await p.screenshot({
    path: path.resolve("local/agent-coverage-desktop.png"),
    fullPage: true,
  });
  await p.setViewportSize({ width: 390, height: 844 });
  await p.locator("#agent-device").selectOption("macbook-air");
  await p.locator(".runtime-section").scrollIntoViewIfNeeded();
  if (await p.evaluate(() => document.documentElement.scrollWidth > innerWidth))
    throw Error("mobile overflow");
  await p.screenshot({ path: path.resolve("local/agent-coverage-mobile.png") });
  await p.locator('[data-view="permissions"]').click();
  const guide = p.locator(".permission-guide");
  if (!(await guide.isVisible()) ||
      !(await guide.innerText()).includes("not verified by this panel"))
    throw Error("Task-specific permission guidance missing");
  if (await p.evaluate(() => document.documentElement.scrollWidth > innerWidth))
    throw Error("permission guide mobile overflow");
  await p.screenshot({ path: path.resolve("local/onboarding-permissions-mobile.png"), fullPage: true });
  await p.setViewportSize({ width: 1440, height: 1100 });
  await p.screenshot({ path: path.resolve("local/onboarding-permissions-desktop.png"), fullPage: true });
  // A hanging overview must release refresh and allow the next request to recover.
  const stall = async () => {};
  await p.route("**/panel/overview", stall);
  await p.locator("#refresh").click();
  await p.waitForFunction(() => document.getElementById("notice").textContent.includes("45 seconds"), null, { timeout: 50000 });
  if (!(await p.locator("#notice").innerText()).includes("citta doctor"))
    throw Error("Reconnect guidance lacks an actionable diagnosis");
  await p.unroute("**/panel/overview", stall);
  await p.waitForFunction(() => !document.getElementById("refresh").disabled);
  await p.locator("#refresh").click();
  await p.waitForFunction(() => document.getElementById("connection").textContent === "Connected");
  const firstVisit = await browser.newPage();
  firstVisit.on("pageerror", (e) => errors.push(e.message));
  await firstVisit.route("**/panel/login", async () => {});
  const freshLink = JSON.parse(execFileSync(binary, ["--config", config, "panel"], { encoding: "utf8" }));
  await firstVisit.goto(freshLink.url);
  await firstVisit.locator("#login").waitFor({ state: "visible", timeout: 20000 });
  if (!(await firstVisit.locator("#notice").innerText()).includes("fresh citta panel link"))
    throw Error("Stalled first login has no recovery guidance");
  await firstVisit.close();
  if (errors.length) throw Error(errors.join("\n"));
  console.log(
    JSON.stringify({
      devices: 5,
      remoteSessionsVisible: true,
      processesNotCountedAsSessions: true,
      airProcessesVisible: true,
      deviceStateSearchFilters: true,
      staleAndOfflineCoverage: true,
      olderDeviceUnknown: true,
      xssSafe: true,
      mobile: true,
      permissionGuidance: true,
      stalledRequestRecovery: true,
      stalledLoginRecovery: true,
      diskReadinessAndRecovery: true,
    }),
  );
})()
  .catch((e) => {
    console.error(e);
    process.exitCode = 1;
  })
  .finally(async () => {
    if (browser) await browser.close();
    if (daemon) {
      daemon.kill("SIGTERM");
      await new Promise((r) => {
        if (daemon.exitCode !== null) return r();
        daemon.once("exit", r);
      });
    }
    fs.rmSync(root, { recursive: true, force: true });
  });
