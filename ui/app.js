"use strict";
const $ = (id) => document.getElementById(id);
const state = { data: null, view: "overview", loading: false, lastGood: 0 };
function h(tag, cls, ...children) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  for (const child of children.flat()) {
    if (child == null) continue;
    e.append(
      child instanceof Node ? child : document.createTextNode(String(child)),
    );
  }
  return e;
}
function replace(id, children) {
  $(id).replaceChildren(...children);
}
function badge(status, label = status) {
  return h("span", `badge ${status}`, label.replaceAll("_", " "));
}
function bytes(n) {
  if (!Number.isFinite(n)) return "—";
  return n >= 1073741824
    ? `${(n / 1073741824).toFixed(1)} GB`
    : `${Math.round(n / 1048576)} MB`;
}
function ago(at) {
  if (!at) return "—";
  const secs = Math.max(0, Math.floor(Date.now() / 1000 - at));
  return secs < 5
    ? "just now"
    : secs < 60
      ? `${secs}s ago`
      : secs < 3600
        ? `${Math.floor(secs / 60)}m ago`
        : `${Math.floor(secs / 3600)}h ago`;
}
function fresh(inventory) {
  return (
    Date.now() / 1000 - state.lastGood < 20 &&
    inventory &&
    Date.now() / 1000 - inventory.observed_at < 20
  );
}
function nodes() {
  if (!state.data) return [];
  const d = state.data.discovery;
  const current = Date.now() / 1000 - state.lastGood < 20;
  return [
    { ...d.local, reachable: current, isLocal: true },
    ...d.peers.map((p) =>
      p.reachable
        ? { ...p.node, reachable: current }
        : { node_id: p.node_id, reachable: false },
    ),
  ];
}
function agents() {
  return nodes().flatMap((node) => [
    ...(node.inventory?.agents || []).map((a) => ({
      ...a,
      node_id: node.node_id,
      stale: !fresh(node.inventory),
    })),
    ...(node.managed_agents || []).map((a) => ({
      ...a,
      node_id: node.node_id,
      stale: Date.now() / 1000 - state.lastGood >= 20,
    })),
  ]);
}
function notice(text) {
  $("notice").textContent = text;
  $("notice").hidden = !text;
}
function metric(label, value, detail, percent) {
  const bar = h("progress", "meter");
  bar.max = 100;
  bar.value = Math.max(0, Math.min(100, percent || 0));
  return h(
    "div",
    null,
    h("div", "metric-label", label),
    h("div", "metric-value", value),
    bar,
    h("div", "metric-detail", detail),
  );
}
function renderDevices(list) {
  replace(
    "devices",
    list.map((n) => {
      const inv = n.inventory,
        r = inv?.resources;
      const current = fresh(inv);
      const card = h("article", "device");
      card.append(
        h(
          "div",
          "device-head",
          h("div", "device-icon", n.isLocal ? "▣" : "▤"),
          h(
            "div",
            null,
            h("div", "device-name", n.node_id),
            h(
              "div",
              "device-system",
              n.reachable
                ? `${n.os} · ${n.arch}${n.isLocal ? " · this device" : ""}`
                : "Connection unavailable",
            ),
          ),
          badge(n.reachable ? "online" : "offline"),
        ),
      );
      if (r) {
        card.append(
          h(
            "div",
            "metrics",
            metric(
              "CPU",
              r.cpu_usage_percent == null
                ? "Sampling…"
                : `${Math.round(r.cpu_usage_percent)}%`,
              `${r.logical_cpus} logical · ${r.physical_cores || "—"} physical cores`,
              r.cpu_usage_percent,
            ),
            metric(
              "MEMORY",
              `${bytes(r.memory_used_bytes)} used`,
              `${bytes(r.memory_total_bytes)} installed · ${bytes(r.admission_memory_bytes ?? r.memory_available_bytes)} est. available`,
              (100 * r.memory_used_bytes) / r.memory_total_bytes,
            ),
          ),
        );
        card.append(
          h(
            "div",
            "device-foot",
            h("span", null, r.cpu_model),
            h(
              "span",
              null,
              current
                ? `${n.active} / ${n.slots} slots busy`
                : `Telemetry stale · ${ago(inv.observed_at)}`,
            ),
          ),
        );
      } else {
        card.append(
          h(
            "div",
            "unavailable",
            n.reachable
              ? "Resource telemetry is not reported by this device’s build."
              : "This device is unreachable. Its agents and capacity are unknown.",
          ),
        );
      }
      if (inv)
        card.append(
          h(
            "div",
            "collector-note",
            `Codex collector: ${inv.codex_status} · ${inv.agents?.length ?? "unknown"} reported sessions${inv.agents_truncated ? " · first 128 shown" : ""}`,
          ),
        );
      card.append(
        h(
          "div",
          "cap-tags",
          (n.capabilities || []).map((c) =>
            h(
              "div",
              "capability-meta",
              h("strong", null, c.name),
              h(
                "span",
                null,
                `Harness: ${c.harness || "not specified"} · Model: ${c.model || "not specified"}`,
              ),
            ),
          ),
        ),
      );
      const manage = h(
        "button",
        "text-button",
        n.permissions
          ? `Permissions · ${n.permissions.policy.mode.replaceAll("_", " ")} ↗`
          : "Configure permissions ↗",
      );
      manage.disabled = !n.reachable;
      manage.addEventListener("click", () => {
        state.view = "permissions";
        permission.node = n.node_id;
        $("permission-device").value = n.node_id;
        applyView();
        loadPermissions();
      });
      card.append(h("div", "device-permissions", manage));
      return card;
    }),
  );
}
function renderAgents() {
  const term = $("agent-search").value.toLowerCase(),
    filter = $("agent-state").value;
  let list = agents().map((a) => ({
    ...a,
    displayState: a.stale ? "unknown" : a.state,
  }));
  list = list.filter(
    (a) =>
      ($("agent-device").value === "all" ||
        a.node_id === $("agent-device").value) &&
      (filter === "all" || a.displayState === filter) &&
      `${a.label} ${a.harness} ${a.model || ""} ${a.node_id} ${a.id}`
        .toLowerCase()
        .includes(term),
  );
  const order = {
    busy: 0,
    blocked: 1,
    idle: 2,
    unknown: 3,
    failed: 4,
    interrupted: 5,
    completed: 6,
  };
  list.sort(
    (a, b) => (order[a.displayState] ?? 7) - (order[b.displayState] ?? 7),
  );
  replace(
    "agents",
    list.map((a) =>
      h(
        "tr",
        null,
        h(
          "td",
          null,
          h("div", "row-title", a.label),
          h(
            "div",
            "row-sub",
            `${a.harness} · ${a.model || "model not specified"}`,
            ` · ${a.id.slice(0, 19)}`,
          ),
        ),
        h("td", null, a.node_id),
        h(
          "td",
          null,
          badge(
            a.displayState,
            a.displayState === "blocked" ? "Needs attention" : a.displayState,
          ),
        ),
        h("td", null, a.managed ? "Citta managed" : "Harness observed"),
        h("td", "muted", ago(a.observed_at)),
      ),
    ),
  );
  $("agents-empty").hidden = list.length > 0;
}
function renderAgentCoverage(list) {
  const selection = $("agent-device").value;
  replace("agent-device", [
    new Option("All devices", "all"),
    ...list.map((n) => new Option(n.node_id, n.node_id)),
  ]);
  $("agent-device").value = list.some((n) => n.node_id === selection)
    ? selection
    : "all";
  replace(
    "agent-coverage",
    list.map((n) => {
      const inv = n.inventory;
      const current = n.reachable && fresh(inv);
      const sessions = inv?.agents?.length;
      const runtimes = inv?.runtimes || [];
      const counts = new Map();
      for (const p of runtimes)
        counts.set(p.name, (counts.get(p.name) || 0) + 1);
      const processText = [...counts]
        .map(([name, count]) => `${count} ${name}`)
        .join(" · ");
      let status, explanation;
      if (!n.reachable) {
        status = "Unreachable";
        explanation =
          "Agent availability is unknown until this device reconnects.";
      } else if (!current) {
        status = inv ? "Stale discovery" : "No session telemetry";
        explanation = "Current sessions and process availability are unknown.";
      } else if (inv.codex_status !== "connected") {
        status = "Session collector unavailable";
        explanation = `Codex collector: ${inv.codex_status || "unknown"}. Process presence does not report session state.`;
      } else if (sessions === 0 && runtimes.length) {
        status = "Processes detected · no reported sessions";
        explanation =
          "The connected Codex server returned no loaded sessions. Other Codex instances or harnesses can run outside that server.";
      } else {
        status = `${sessions ?? "Unknown"} sessions reported`;
        explanation =
          "Coverage is limited to the connected Codex server and Citta-managed runs. Other sessions may exist.";
      }
      const button = h("button", "coverage-select", "Show device");
      button.type = "button";
      button.addEventListener("click", () => {
        $("agent-device").value = n.node_id;
        $("agent-state").value = "all";
        $("agent-search").value = "";
        renderAgents();
        renderRuntimes(nodes());
        $("agents-section").scrollIntoView({ block: "start" });
      });
      return h(
        "article",
        "coverage-card",
        h(
          "div",
          "coverage-heading",
          h("strong", null, n.node_id),
          badge(current ? "online" : "unknown", current ? "Fresh" : "Unknown"),
        ),
        h("div", "coverage-status", status),
        h("p", "muted", explanation),
        h(
          "div",
          "row-sub",
          inv
            ? `${processText || "No detected runtimes"} · ${(n.managed_agents || []).filter((a) => a.state === "busy").length} Citta-managed running`
            : "Process telemetry unavailable",
        ),
        button,
      );
    }),
  );
}
function renderRuntimes(list) {
  const device = $("agent-device").value;
  const term = $("agent-search").value.toLowerCase();
  const filter = $("agent-state").value;
  const processes = list
    .flatMap((n) =>
      (n.inventory?.runtimes || []).map((p) => ({
        ...p,
        node: n.node_id,
        current: n.reachable && fresh(n.inventory),
        observed_at: n.inventory.observed_at,
      })),
    )
    .filter(
      (p) =>
        (device === "all" || p.node === device) &&
        (filter === "all" || filter === "unknown") &&
        `${p.node} ${p.name} ${p.pid}`.toLowerCase().includes(term),
    );
  $("runtime-count").textContent = `(${processes.length} processes)`;
  replace(
    "runtimes",
    processes.map((p) =>
      h(
        "article",
        "runtime",
        h(
          "div",
          "runtime-heading",
          h("strong", null, `${p.name} · PID ${p.pid}`),
          badge("unknown", "Session state unknown"),
        ),
        h("div", "runtime-device", p.node),
        h("div", null, "Model not reported · process observation"),
        h(
          "div",
          null,
          p.current
            ? `${bytes(p.memory_bytes)} memory · seen ${ago(p.observed_at)}`
            : `Stale observation · ${ago(p.observed_at)}`,
        ),
      ),
    ),
  );
  $("runtimes-empty").hidden = processes.length > 0;
}
function renderProfiles(list) {
  const profiles = list
    .filter((n) => n.reachable)
    .flatMap((n) =>
      (n.capabilities || [])
        .filter((c) => c.kind === "agent")
        .map((c) => ({ ...c, node: n.node_id, policy: n.permissions?.policy })),
    );
  replace(
    "profiles",
    profiles.length
      ? profiles.map((p) =>
          h(
            "article",
            "profile",
            h("div", "profile-name", p.name),
            h("div", "row-sub", p.node),
            h(
              "div",
              "profile-runtime",
              `Harness: ${p.harness || "not specified"} · Model: ${p.model || "not specified"}`,
            ),
            h("p", "profile-description", p.description),
            h(
              "div",
              "profile-limits",
              p.policy &&
                (p.policy.mode === "paused" ||
                  (p.policy.mode === "restricted" &&
                    !p.policy.allowed_capabilities.includes(p.name)) ||
                  p.policy.allowed_origins?.length === 0)
                ? "Blocked by device permissions"
                : `${Math.max(0, p.slots - p.active)} / ${p.slots} slots free · ${p.max_timeout_secs}s max`,
            ),
          ),
        )
      : [
          h(
            "div",
            "empty",
            "No agent spawn profiles are registered on reachable devices.",
          ),
        ],
  );
}
function runs() {
  const activity = state.data?.activity;
  if (!activity) return [];
  return [
    ...activity.local.tasks,
    ...activity.peers.filter((p) => p.reachable).flatMap((p) => p.tasks),
  ].sort((a, b) => b.updated_at - a.updated_at);
}
function renderRuns() {
  let list = runs();
  if ($("failures-only").checked)
    list = list.filter((t) => ["failed", "interrupted"].includes(t.status));
  replace(
    "runs",
    list.slice(0, 80).map((t) => {
      const button = h("button", "text-button", "Inspect ↗");
      button.addEventListener("click", () => showRun(t));
      return h(
        "tr",
        null,
        h(
          "td",
          null,
          h("div", "row-title", t.capability),
          h("div", "row-sub", t.id.slice(0, 18)),
        ),
        h(
          "td",
          null,
          `${t.origin_node} → ${t.worker_node || t.target_node || "awaiting worker"}`,
        ),
        h("td", null, badge(t.status)),
        h("td", "muted", ago(t.updated_at)),
        h("td", null, button),
      );
    }),
  );
  $("runs-empty").hidden = list.length > 0;
}
async function showRun(run) {
  const dialog = $("run-dialog");
  replace("run-detail", [h("p", null, "Loading run…")]);
  dialog.showModal();
  try {
    const response = await fetch(
      `/panel/task?node=${encodeURIComponent(run.origin_node)}&id=${encodeURIComponent(run.id)}`,
      { cache: "no-store" },
    );
    if (!response.ok) throw Error(`Run unavailable (${response.status})`);
    const t = await response.json();
    const children = [
      h("div", "detail-label", "Run"),
      h(
        "pre",
        null,
        `${t.id}\n${t.origin_node} → ${t.worker_node || "queued"}`,
      ),
      badge(t.status),
    ];
    for (const [label, value] of [
      ["Output", t.outcome?.stdout],
      ["Error output", t.outcome?.stderr],
      ["Failure", t.outcome?.error],
    ])
      if (value)
        children.push(h("div", "detail-label", label), h("pre", null, value));
    if (!t.outcome)
      children.push(h("p", null, "This run has no terminal result yet."));
    replace("run-detail", children);
  } catch (e) {
    replace("run-detail", [h("p", null, e.message)]);
  }
}
function render() {
  const list = nodes(),
    live = list.filter((n) => n.reachable),
    inventory = live.filter((n) => fresh(n.inventory));
  const all = agents(),
    liveAgents = all.filter(
      (a) => !a.stale && ["busy", "idle", "blocked"].includes(a.state),
    ),
    idle = liveAgents.filter((a) => a.state === "idle");
  const free = inventory.reduce(
    (sum, n) => sum + n.inventory.resources.memory_available_bytes,
    0,
  );
  replace(
    "stats",
    [
      [
        "Connected devices",
        `${live.length} / ${list.length}`,
        "Paired devices responding",
        "◈",
      ],
      [
        "Reported live sessions",
        liveAgents.length,
        `${liveAgents.filter((a) => a.state === "busy").length} busy · ${liveAgents.filter((a) => a.state === "blocked").length} need attention`,
        "◎",
      ],
      ["Reported idle sessions", idle.length, "Supported collectors only", "○"],
      [
        "Available memory",
        inventory.length ? bytes(free) : "—",
        `OS estimate · ${inventory.length} of ${live.length} devices`,
        "▥",
      ],
    ].map(([label, value, note, mark]) =>
      h(
        "article",
        "stat",
        h("div", "stat-top", label, h("span", "stat-mark", mark)),
        h("div", "stat-value", value),
        h("div", "stat-note", note),
      ),
    ),
  );
  $("resource-coverage").textContent =
    `${live.length} reachable · ${inventory.length} reporting resources`;
  $("nav-agent-count").textContent = liveAgents.length;
  $("nav-agent-count").title =
    "Live sessions reported by supported collectors; not a complete process count";
  renderDevices(list);
  renderAgentCoverage(list);
  renderAgents();
  renderProfiles(list);
  renderRuns();
  renderRuntimes(list);
  renderPermissionDevices();
  applyView();
}
function applyView() {
  const v = state.view;
  $("permissions-section").hidden = v !== "permissions";
  $("stats").hidden = v === "permissions";
  document.querySelector(".runtime-section").hidden = v === "permissions";
  for (const name of ["devices", "coverage", "agents", "profiles", "runs"])
    $(`${name}-section`).hidden =
      v !== "overview" &&
      !(v === "agents" && ["coverage", "agents", "profiles"].includes(name)) &&
      !(v === "runs" && name === "runs");
  $("page-title").textContent =
    v === "permissions"
      ? "Permissions across your devices"
      : v === "agents"
        ? "Agents across your mesh"
        : v === "runs"
          ? "Work across your network"
          : "Your agent network";
  $("page-subtitle").textContent =
    v === "permissions"
      ? "Choose what each device may execute, and which devices may send it work."
      : v === "agents"
        ? "See reported sessions, detected runtimes, and discovery coverage on every device."
        : v === "runs"
          ? "Follow execution, inspect results, and keep failed runs visible."
          : "A live view of the devices, agents, and work across your mesh.";
  document.querySelectorAll(".nav").forEach((b) => {
    b.classList.toggle("active", b.dataset.view === v);
    b.setAttribute("aria-current", b.dataset.view === v ? "page" : "false");
  });
}
async function refresh() {
  if (state.loading) return;
  state.loading = true;
  $("refresh").disabled = true;
  try {
    const r = await fetch("/panel/overview", { cache: "no-store" });
    if (r.status === 401) {
      $("login").hidden = false;
      $("dashboard").hidden = true;
      $("connection").textContent = "Session required";
      return;
    }
    if (!r.ok) throw Error(`Network status unavailable (${r.status})`);
    state.data = await r.json();
    state.lastGood = Date.now() / 1000;
    $("last-update").textContent = "Updated just now";
    $("login").hidden = true;
    $("dashboard").hidden = false;
    $("connection").textContent = "Connected";
    notice("");
    render();
  } catch (e) {
    $("connection").textContent = "Reconnecting";
    notice(`${e.message}. Showing the last successful snapshot, if available.`);
    if (state.data) render();
  } finally {
    state.loading = false;
    $("refresh").disabled = false;
  }
}
async function start() {
  const params = new URLSearchParams(location.hash.slice(1));
  const code = params.get("login");
  if (code) {
    history.replaceState(null, "", location.pathname);
    try {
      const r = await fetch("/panel/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ code }),
      });
      if (!r.ok)
        notice(
          "That login link expired or was already used. Open a fresh link from Citta.",
        );
    } catch {
      notice("Could not connect to the local Citta daemon.");
    }
  }
  await refresh();
  setInterval(refresh, 5000);
  setInterval(() => {
    $("last-update").textContent = state.lastGood
      ? `Updated ${ago(state.lastGood)}`
      : "Waiting for first update";
    if (state.data && Date.now() / 1000 - state.lastGood >= 20) render();
  }, 1000);
}
$("refresh").addEventListener("click", refresh);
function filterAgents() {
  renderAgents();
  renderRuntimes(nodes());
}
$("agent-search").addEventListener("input", filterAgents);
$("agent-state").addEventListener("change", filterAgents);
$("agent-device").addEventListener("change", filterAgents);
$("failures-only").addEventListener("change", renderRuns);
$("close-dialog").addEventListener("click", () => $("run-dialog").close());
document.querySelectorAll(".nav").forEach((button) =>
  button.addEventListener("click", () => {
    state.view = button.dataset.view;
    applyView();
    if (state.view === "permissions" && !permission.view && !permission.loading)
      loadPermissions();
  }),
);
$("logout").addEventListener("click", async () => {
  await fetch("/panel/logout", { method: "POST" });
  state.data = null;
  state.lastGood = 0;
  await refresh();
});
const permission = {
  node: null,
  view: null,
  loading: false,
  saving: false,
  dirty: false,
  sequence: 0,
};
function permissionMessage(message) {
  $("permission-message").textContent = message;
  $("permission-message").hidden = !message;
}
function renderPermissionDevices() {
  const select = $("permission-device");
  const list = nodes();
  const signature = JSON.stringify(list.map((n) => [n.node_id, n.reachable]));
  if (select.dataset.signature === signature) return;
  select.dataset.signature = signature;
  if (!permission.node) permission.node = state.data?.discovery.local.node_id;
  const options = list.map((n) => {
    const option = h(
      "option",
      null,
      n.node_id + (n.reachable ? "" : " · unavailable"),
    );
    option.value = n.node_id;
    return option;
  });
  if (permission.node && !list.some((n) => n.node_id === permission.node)) {
    const o = h("option", null, permission.node + " · unavailable");
    o.value = permission.node;
    options.push(o);
  }
  select.replaceChildren(...options);
  select.value = permission.node || "";
}
function editPermissions() {
  permission.dirty = true;
  $("permission-save").disabled = permission.saving;
  $("permission-dirty").textContent = "Unsaved changes";
  updatePermissionControls();
}
function updatePermissionControls() {
  const mode = document.querySelector(
    'input[name="permission-mode"]:checked',
  )?.value;
  $("permission-capabilities").disabled = mode !== "restricted";
  for (const input of document.querySelectorAll(
    "#permission-origin-list input",
  ))
    input.disabled = $("permission-all-origins").checked;
}
function permissionCheck(value, label, detail, checked) {
  const input = h("input");
  input.type = "checkbox";
  input.value = value;
  input.checked = checked;
  input.addEventListener("change", editPermissions);
  return h(
    "label",
    "permission-check",
    input,
    h("span", null, h("strong", null, label), h("small", null, detail)),
  );
}
function renderPermissionEditor(view) {
  const { snapshot } = view,
    policy = snapshot.policy;
  permission.view = view;
  permission.dirty = false;
  $("permission-title").textContent = view.node_id;
  $("permission-revision").textContent =
    `Revision ${snapshot.revision} · ${ago(snapshot.updated_at)}`;
  for (const radio of document.querySelectorAll(
    'input[name="permission-mode"]',
  ))
    radio.checked = radio.value === policy.mode;
  const caps = [...view.capabilities];
  for (const name of policy.allowed_capabilities)
    if (!caps.some((c) => c.name === name))
      caps.push({ name, description: "Currently not installed" });
  replace(
    "permission-cap-list",
    caps.length
      ? caps.map((c) =>
          permissionCheck(
            c.name,
            c.name,
            c.description,
            policy.allowed_capabilities.includes(c.name),
          ),
        )
      : [h("p", "muted", "No capabilities are installed on this device.")],
  );
  $("permission-all-origins").checked = policy.allowed_origins === null;
  const origins = [...view.origins];
  for (const id of policy.allowed_origins || [])
    if (!origins.some((o) => o.id === id))
      origins.push({ id, node_id: "Unavailable device", local: false });
  replace(
    "permission-origin-list",
    origins.map((o) =>
      permissionCheck(
        o.id,
        o.node_id + (o.local ? " · this device" : ""),
        o.id,
        policy.allowed_origins === null ||
          policy.allowed_origins.includes(o.id),
      ),
    ),
  );
  replace(
    "permission-history",
    view.history.map((entry) =>
      h(
        "div",
        "permission-event",
        h(
          "strong",
          null,
          `Revision ${entry.revision} · ${entry.policy.mode.replaceAll("_", " ")}`,
        ),
        h("span", "muted", `${entry.actor} · ${ago(entry.updated_at)}`),
      ),
    ),
  );
  $("permission-editor").hidden = false;
  $("permission-save").disabled = true;
  $("permission-dirty").textContent = "Saved on device";
  updatePermissionControls();
}
async function loadPermissions() {
  if (!permission.node || permission.saving) return;
  const sequence = ++permission.sequence;
  permission.loading = true;
  permission.view = null;
  $("permission-editor").hidden = true;
  permissionMessage("Loading device permissions…");
  try {
    const response = await fetch(
      `/panel/permissions?node=${encodeURIComponent(permission.node)}`,
      { cache: "no-store" },
    );
    const value = await response.json().catch(() => ({}));
    if (!response.ok)
      throw Error(
        value.error || `Device permissions unavailable (${response.status})`,
      );
    if (sequence !== permission.sequence) return;
    renderPermissionEditor(value);
    permissionMessage("");
  } catch (e) {
    if (sequence === permission.sequence) permissionMessage(e.message);
  } finally {
    if (sequence === permission.sequence) permission.loading = false;
  }
}
$("permission-device").addEventListener("change", () => {
  permission.node = $("permission-device").value;
  permission.dirty = false;
  loadPermissions();
});
for (const radio of document.querySelectorAll('input[name="permission-mode"]'))
  radio.addEventListener("change", editPermissions);
$("permission-all-origins").addEventListener("change", editPermissions);
$("permission-reload").addEventListener("click", loadPermissions);
$("permission-form").addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!permission.view || permission.saving || !permission.dirty) return;
  permission.saving = true;
  $("permission-save").disabled = true;
  $("permission-device").disabled = true;
  $("permission-reload").disabled = true;
  const policy = {
    mode: document.querySelector('input[name="permission-mode"]:checked').value,
    allowed_capabilities: [
      ...document.querySelectorAll("#permission-cap-list input:checked"),
    ].map((i) => i.value),
    allowed_origins: $("permission-all-origins").checked
      ? null
      : [
          ...document.querySelectorAll("#permission-origin-list input:checked"),
        ].map((i) => i.value),
  };
  const node = permission.node;
  for (const input of document.querySelectorAll("#permission-form input"))
    input.disabled = true;
  permissionMessage(`Saving permissions on ${node}…`);
  try {
    const response = await fetch("/panel/permissions", {
      method: "POST",
      headers: { "Content-Type": "application/json", "X-Citta-Panel": "1" },
      body: JSON.stringify({
        node,
        expected_revision: permission.view.snapshot.revision,
        policy,
      }),
    });
    const value = await response.json().catch(() => ({}));
    if (!response.ok)
      throw Error(
        value.error ||
          `Save failed (${response.status}). Reload before retrying.`,
      );
    permission.view.snapshot = value;
    permission.view.history = [value, ...permission.view.history].slice(0, 20);
    renderPermissionEditor(permission.view);
    permissionMessage(
      `Permissions saved on ${node}. Revision ${value.revision}.`,
    );
    await refresh();
  } catch (e) {
    permissionMessage(e.message);
    $("permission-dirty").textContent =
      "Not confirmed · reload saved settings before retrying";
    $("permission-save").disabled = true;
    permission.dirty = false;
  } finally {
    permission.saving = false;
    $("permission-device").disabled = false;
    $("permission-reload").disabled = false;
    for (const input of document.querySelectorAll("#permission-form input"))
      input.disabled = false;
    updatePermissionControls();
  }
});
start();
