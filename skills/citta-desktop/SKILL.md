---
name: citta-desktop
description: Operate an authorized macOS desktop from a Citta worker using native screenshot and input helpers, or verify the receiving worker's real desktop access during onboarding. Use an already-connected app or browser tool when it directly supports the requested task.
---

# Desktop control from a Citta worker

A Citta worker can operate the desktop through its shell and image-viewing tools using the helpers in this skill. An app-hosted computer tool is not required for that path, but native controls still need the receiving session and OS grants. Keep the user's task, device and account constraints; this skill does not grant additional authorization or change the worker's normal tool access.

Use `scripts/desktop-control.cjs` with the installed `@priyankark/rusty-robot@0.1.2` library. Read [usage](references/usage.md) for installation, JSON request shapes, explicit capture backends and the owned-window test. The helpers target macOS; do not assume their packaged native binaries match another OS/architecture. The addon resolves by default from `~/.local/share/citta/desktop-control/node_modules/@priyankark/rusty-robot` on the receiving device. An explicit `CITTA_DESKTOP_ADDON` path overrides that location with another verified package. A failed native load is an installation error; do not silently switch libraries.

Before desktop actions, obtain a fresh `status`. It reports native permission flags, screen lock, active displays and foreground PID; it does not attest to delivery. Initial onboarding verifies actual input and capture in the owned fixture. Identify the process macOS attributes a denied action to before prescribing a grant, rather than assuming it is Terminal, Codex or Citta. Use the normal OS approval flow within the owner's authorization; do not edit TCC or borrow another application's signing identity. Restart a service only when idle, then test a newly launched worker.

For task input, inspect the current screenshot, identify the intended app/field, and set `expect_pid` to the observed foreground application. Use logical screen coordinates, converting resized screenshot positions with its reported image dimensions and logical display bounds. The adapter rechecks focus immediately before input; it cannot lock OS focus across a sequence. Observe again after navigation or focus changes.

`capture` returns an image path for the image-viewing tool. The Rust backend captures the main display; the explicit `system-window` backend captures a selected window. Select the backend deliberately and preserve errors. Capture has a bounded CLI watchdog; also set a caller-side subprocess deadline so synchronous native blocking cannot hang the task. A capture timeout is not proof of an image or permission failure. Verify the image visually; a permission flag or nonempty file alone is insufficient. Input results indicate issued actions, not delivered text or completed external work. `verify-fixture` checks delivery only in its own test field.

For email or other external actions, verify the intended account and inspect provider confirmation afterward. Follow existing task authorization without asking again solely because the worker is remote. Preserve unique task markers; inspect an uncertain action before repeating it. Stop the owned fixture and remove only your own temporary evidence when no longer needed, preserving any diagnostic evidence requested by the parent.
