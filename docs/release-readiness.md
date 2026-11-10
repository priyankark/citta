# Release readiness

Citta is being prepared for its first public release. The release is **not yet qualified**. This record separates prepared source and demonstration evidence from the checks needed before publishing a production tag or launching on Product Hunt.

Last reviewed: November 10, 2026. The candidate starts from `1c65372` and incorporates the release-preparation changes; record the final committed candidate and archive checksums before acceptance. A moving branch name or package version alone is insufficient.

## Launch gates

| Gate | Current evidence | What closes it |
| --- | --- | --- |
| Candidate CI | Main [run 42705201782](https://github.com/priyankark/citta/actions/runs/42705201782) failed before any build/test steps because GitHub reported failed account payments or a spending limit. This is an account/infrastructure blocker. | Restore account capacity, then pass the configured suite on the final candidate. Preserve the original failure; do not repeatedly retry the unchanged condition. |
| Download and install | A native Apple Silicon development archive passed checksum, isolated fresh install, executable version/help, and existing-install refusal checks. The source crate passed publish dry-run and isolated installation. These preliminary artifacts precede the disk-readiness fix; final clean-revision builds are still required. Existing CI artifacts alone are not a public distribution. | Build each advertised architecture from the recorded candidate; inspect archives, verify checksums and notices, and install without the author's checkout or private files. |
| macOS distribution | Only an Apple Development signing identity was available on the development Mac at review time. No Developer ID Application identity was available. An earlier transferred Air binary was rejected by Gatekeeper. | Establish the intended Developer ID signing/notarization path and verify a downloaded artifact on a receiving Mac. Record signing identity continuity and OS permission behavior during upgrade. Do not advise disabling Gatekeeper or editing TCC. |
| Same-build devices | Earlier physical Mini/Air tasks succeeded, but a fresh November 10 Air preflight failed before worker startup because its disk could not create a workspace. Installed binaries are not established as the final candidate on both devices. | Complete the [physical acceptance checklist](acceptance.md#physical-device-qualification--pending) against the same source candidate, with per-device artifact checksums and task evidence. |
| Recovery and soak | Local fault tests exist; complete physical sleep/wake, network interruption, upgrade/rollback and representative soak evidence is outstanding. | Record duration, task counts, failures and outcomes, including uncertain external side effects, on the intended service configuration. |
| Desktop setup | Guarded-session portable sources are now reconciled with the reviewed guard branch and embedded by setup. Existing Air native input/capture was not verified. | Compile current helpers on the receiving Mac, grant actual worker permissions, then verify owned-window delivery and capture. Do not advertise desktop readiness solely from successful pairing. |
| Local model example | A real Air-origin task used the Mini's configured Ollama/Gemma capability. Ordinary setup does not provision that model or capability automatically. | Exercise the documented model recipe on a clean configuration and state its model/download requirements. |
| Source and notices | PolyForm Noncommercial 1.0.0, attribution notices and dependency-license collection are prepared. Targeted historical scans found no matching credentials/private paths in 250 unique blobs. | Review the exact public tree/history and final binary contents; retain third-party terms. This limited scan is not comprehensive secret clearance. |
| Product Hunt entry | Launch copy and the README demo video are prepared; user-selected launch target is November 14. Gallery and external submission remain pending while functional readiness takes priority. | A working public product URL, usable install, qualified advertised functionality and completed listing; schedule only after the gates above pass. |

## Claims that current evidence supports

- Agents can delegate tasks across trusted devices with configured tools and capabilities. General work can use the receiving harness's shell, scripts, code and installed skills, subject to its normal permissions and installed dependencies.
- Tailscale discovery does not install software, authenticate an app account, grant OS permissions, download a model or establish that another device is awake.
- Real demonstration tasks ran Mini → Air against synthetic receipt files and Air → Mini against a local Gemma model. The expense report was returned to the Mini because the Air could not save a local report when its disk was full. The reverse model task returned categories and individual expense explanations; it did not follow every requested output field.
- The demonstration uses existing installations. It is not evidence of fresh installation, equal binary provenance, unattended desktop access, a fully local orchestration model, or a completed production soak.
- Citta places tasks on devices. It does not pool their RAM into one address space or migrate a live agent session.
- Citta is source-available for the uses permitted by PolyForm Noncommercial 1.0.0. Marketing must not call it OSI open source or suggest ordinary commercial use is licensed.

## Guarded desktop reconciliation

The release candidate incorporates portable source changes from `guard/mini-desktop-sessions-20261109` through `b324d5c` (including `d75fc50` and `8965717`). This adds the Python session broker and recovery tests, HID idle observations, native-helper gating, setup embedding, CI coverage, and operating guidance. Compiled helpers and live installed Citta binaries were not copied or replaced.

Local qualification on November 10 passed nine session tests, the Node adapter contract suite, Swift compilation of the context helper, five adapter/service helper tests, and six CI-history tests. The subsequent disk-admission change passed 40 Rust tests, warning-free Clippy, a debug build and its new real-process disk regression; complete mesh/setup/browser results must be tied to the final candidate record. These are isolated tests, not physical desktop acceptance.

The session broker is cooperative and per-user. It is not an OS security boundary around full-access shell tools. Its idle signal is advisory; foreground changes and human activity between observations can be missed. Waiting occupies a Citta task slot. A broker can outlive its originating task until bounded expiry; an in-flight native process can retain the kernel lease if its broker dies. Preserve these limits in [operations](operations.md#keeping-a-mac-available) and the [desktop usage guide](../skills/citta-desktop/references/usage.md#guarded-sessions).

## Release record

Before publication, attach a record containing:

- Final source commit, tag, CI URLs and all advertised artifact checksums.
- Signing/notarization evidence and clean-install OS/architecture results.
- Two-device versions, exact binary checksums, task IDs and useful returned results.
- Sleep/wake, disconnect, restart, cancellation, upgrade/rollback and soak outcomes.
- Dependency notice inventory and public-source review scope.
- Accepted limitations, unresolved issues, support instructions and launch URL.

Private operational evidence belongs under ignored `local/`, not in public archives. Configurations, mesh tokens, panel login URLs, personal account data and raw desktop recordings must not be bundled. The committed TLS fixture key is intentionally public test material, documented in [tests/fixtures/README.md](../tests/fixtures/README.md); it is not a deployment credential.

## November 10 functional verification

Before the disk-readiness correction, local verification passed 38 Rust tests, Clippy and formatting, 23 mesh scenarios, 30 isolated setup scenarios, 5 adapter/service helpers, 7 upgrade checks and 6 CI-history tests. Release installer tests covered tampering, unsafe paths, incompatible platform, existing configuration/binaries and partial-install failure. A real native archive installed in a temporary home; the extracted source crate passed a publish dry run and an isolated Cargo installation. These runs do not establish that the final amended candidate passed.

A fresh delegated preflight on the existing Air installation failed with `workspace creation failed: No space left on device (os error 28)` before any agent process started. Placement had reported the Air eligible. The task was preserved without replay. Disk-readiness checks and a visible panel diagnostic now address this failure before work is assigned. Free storage and receiving-device access are still required before new physical acceptance; a source fix cannot reclaim disk space on a worker that cannot launch.

Final source validation after the disk claim-protocol and symlink corrections passed 42 Rust tests, 25 real-process mesh scenarios, 30 isolated setup scenarios, Clippy, and the desktop/mobile browser regression, including low-space, stale observation and recovery messages. Nine installer tests and nine guarded-session tests passed, along with the native adapter contract suite. The advisory audit completed without reported vulnerabilities. These remain local checks; the full Air and hosted-platform gates above are open.
