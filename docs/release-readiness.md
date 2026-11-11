# Release readiness

Citta is being prepared for its first public release. The release is **not yet qualified**. This record separates prepared source and demonstration evidence from the checks needed before publishing a production tag or launching on Product Hunt.

Last reviewed: November 11, 2026 (UTC). The current tested source is `f6fd91fa7978e16b5aca6dec227c1e303de0daea`, the [PR 14](https://github.com/priyankark/citta/pull/14) merge. Its exact artifacts and physical checks are recorded below. Earlier `e15fe0e` evidence and failed attempts remain as history. A later documentation-only evidence update does not change the source provenance of these binaries; runtime or packaging changes require a new qualification record.

## Launch gates

| Gate | Current evidence | What closes it |
| --- | --- | --- |
| Candidate CI | Main [run 42974236178, attempt 2](https://github.com/priyankark/citta/actions/runs/42974236178/attempts/2) passed on `f6fd91f`: Linux, Intel macOS (including ARM compilation) and audit all completed successfully. Native [candidate packaging run 42974812756](https://github.com/priyankark/citta/actions/runs/42974812756) also passed all three platform jobs. The original billing failure is retained and linked to the successful retry. | This CI gate is satisfied for the pinned source. Requalify runtime or packaging changes beyond it; a later documentation-only commit is not the source of these tested binaries. |
| Download and install | The current candidate produced verified Apple Silicon, Intel macOS and Linux x86-64 archives, each passing its native installer check and all manifest-file hashes. The source crate passed packaging, publish dry-run and isolated installation. The exact CI Apple Silicon archive also passed fresh installation and execution on both physical Macs. No registry upload or public binary release occurred. | Verify the intended public download/install path, including signed macOS distribution. Native runner and isolated physical installs do not establish browser-quarantine or notarization behavior. |
| macOS distribution | Neither qualification Mac has a usable Developer ID Application signing identity; only Apple Development identities were found. An existing Developer ID certificate entry was listed as "Not in Keychain", and the attempt to create a new identity was rejected because it requires the Account Holder. No certificate was created, and the candidate is not Developer ID signed or notarized. An earlier transferred binary was rejected by Gatekeeper. The earlier `e15fe0e` candidate later executed successfully with its existing ad-hoc signature in an isolated physical Air test, without an OS denial; that controlled transfer did not qualify public browser-download quarantine or notarization. | Obtain the intended Developer ID signing identity through the authorized account flow, then sign, notarize and qualify a downloaded artifact on a receiving Mac. Record signing identity continuity and OS permission behavior during upgrade. Do not disable Gatekeeper or edit TCC. |
| Same-build devices | Both physical Macs ran the exact CI Apple Silicon artifact in separate temporary daemons, automatically paired, and completed bidirectional shell proofs. An actual receiving Codex agent used Bash, wrote and ran Python, and read an installed skill. The Air also invoked the Mini's real local model through the candidate adapter. Live services were preserved. | Complete remaining [physical acceptance](acceptance.md#physical-device-qualification--pending), especially live upgrade/rollback and clean-user harness/provider/model provisioning. Retain the distinction between temporary qualification daemons and installed services. |
| Recovery and soak | Candidate daemon restart interrupted a harmless running task; a distinct new task succeeded afterward, with no replay of the original. A short batch completed 20/20 lightweight remote tasks over 197.38 seconds. | Complete physical sleep/wake or reboot, network-loss recovery, live upgrade/rollback and a representative long-running soak. The short batch is bounded smoke coverage, not soak qualification. |
| Desktop setup | Guarded-session sources are embedded by setup. The two missing native Air helpers were built on the Mini, transferred, checked against pinned source/binary hashes and platform requirements, and installed. A subsequent service-launched worker confirmed their hashes but reported Accessibility and Screen Recording unavailable. The latest November 11 UTC observation found a locked session at the login window, no active displays, and false permission flags. Native input/capture remains unverified. | Unlock the receiving desktop, verify the actual service process has the required OS permissions, restart the idle service if needed, then verify owned-window delivery and capture from a new worker. Do not advertise desktop readiness solely from pairing or helper installation. |
| Local model example | On the exact CI candidate, a physical Air-origin task invoked the Mini's actual Gemma 3:4b model through the Ollama adapter and returned categorization output (32 generated tokens). Earlier loopback and demonstration evidence remains below. | Verify clean-user model download/provisioning and representative workload budgets. Ordinary setup does not provision the model or capability automatically. |
| Source and notices | PolyForm Noncommercial 1.0.0, attribution notices and dependency-license collection are prepared. Targeted historical scans found no matching credentials/private paths in 250 unique blobs. | Review the exact public tree/history and final binary contents; retain third-party terms. This limited scan is not comprehensive secret clearance. |
| Product Hunt entry | Launch copy and the README demo video are prepared; user-selected launch target is November 14. Gallery and external submission remain pending while functional readiness takes priority. | A working public product URL, usable install, qualified advertised functionality and completed listing; schedule only after the gates above pass. |

## Current candidate artifacts and native checks

All files in this table identify clean source `f6fd91fa7978e16b5aca6dec227c1e303de0daea`. Native [packaging run 42974812756](https://github.com/priyankark/citta/actions/runs/42974812756) completed successfully on Apple Silicon macOS, Intel macOS and Linux x86-64. Each job built its native archive and passed the native installer check; downloaded archives were then checked against every manifest hash. These are unpublished artifacts, not signed production downloads.

| Artifact | Size and contents | SHA-256 |
| --- | --- | --- |
| `citta-0.1.0-aarch64-apple-darwin.tar.gz` | 6,607,300 bytes; 442 manifest entries | `a004a35c43cdc38b0fb97a92cb48f4ba14ca7cdf9a66d1062bc791cbce40aeb5` |
| `citta-0.1.0-x86_64-apple-darwin.tar.gz` | 6,997,680 bytes; 442 manifest entries | `81361b680eb39a88ad52869dbaa573d9f0858de98a1567839d4f7e047f218e5a` |
| `citta-0.1.0-x86_64-unknown-linux-gnu.tar.gz` | 7,238,848 bytes; 434 manifest entries | `9a59db1f9eb79be062edada4a6ec41d69874b54aaf51027257e0a26924b5b732` |
| `citta-0.1.0.crate` | 191,028 bytes; 64 files | `1bbedb213c9d96883fe4a2931ae17a941f0b54a9b8295823295aa8081343629d` |

The crate's VCS record identifies the same clean source. Packaging, `cargo publish --dry-run --locked`, installation from extracted contents and installed version/setup-help checks passed locally on Apple Silicon. Required embedded assets and notices were present; private, media and launch-material paths were excluded. The dry run did not upload to crates.io. Signing or any other change to these artifacts requires new hashes and verification.

## Current candidate physical agent and model checks

The exact CI Apple Silicon archive above was installed into fresh isolated destinations on the physical Mini and Air. Both ran executable SHA-256 `52df8c5bcfee57061eb829ac8d01e54fdd2ad78a7fd6a67c0e060a543a7600a0` with separate temporary configuration/state. Automatic same-owner pairing, echo and executing-host/input-digest fixtures succeeded across private HTTPS in both directions. Fresh disk/resource observations, a resource-qualified remote task, same-key task identity and wrong-token HTTP 401 were verified.

An actual full-tool Codex process on the Air used Bash, wrote a Python file, ran it and asserted its result, and read the installed delegation skill. This verifies shell/code/skill availability through the candidate worker; it does not claim every configured tool, app login or provider is provisioned on a clean user account. A separate Air-origin task used the Mini's actual Gemma 3:4b through the candidate Ollama adapter and returned categorization output with 32 generated tokens.

A short batch completed 20/20 lightweight remote tasks over 197.38 seconds. A controlled restart of the isolated Air daemon interrupted a harmless running fixture; a deliberately distinct task succeeded afterward while the original remained interrupted without replay. This is bounded task/restart coverage, not a representative long-running soak or a device sleep/reboot/network-loss test.

Both temporary daemons and their task processes exited. Independent cleanup checks found no remaining test listeners or owned processes, exact prior Serve configurations were restored, and live executable/configuration/service-plist hashes remained unchanged. The transfer server stopped and the remote supervisor finished successfully. These checks did not upgrade live services, change desktop grants, provision a clean user's provider/model account, or qualify signed/notarized public distribution.

## Claims that current evidence supports

- Agents can delegate tasks across trusted devices with configured tools and capabilities. General work can use the receiving harness's shell, scripts, code and installed skills, subject to its normal permissions and installed dependencies.
- Tailscale discovery does not install software, authenticate an app account, grant OS permissions, download a model or establish that another device is awake.
- Real demonstration tasks ran Mini → Air against synthetic receipt files and Air → Mini against a local Gemma model. The expense report was returned to the Mini because the Air could not save a local report when its disk was full. The reverse model task returned categories and individual expense explanations; it did not follow every requested output field.
- The demonstration uses existing installations. It is not evidence of fresh installation, equal binary provenance, unattended desktop access, a fully local orchestration model, or a completed production soak.
- Citta places tasks on devices. It does not pool their RAM into one address space or migrate a live agent session.
- Citta is source-available for the uses permitted by PolyForm Noncommercial 1.0.0. Marketing must not call it OSI open source or suggest ordinary commercial use is licensed.

## Guarded desktop reconciliation

The release candidate incorporates portable source changes from `guard/mini-desktop-sessions-20261109` through `b324d5c` (including `d75fc50` and `8965717`). This adds the Python session broker and recovery tests, HID idle observations, native-helper gating, setup embedding, CI coverage, and operating guidance. That source reconciliation did not replace live Citta service binaries. Later device repair installed the two previously missing native helpers on the Air, as recorded below; it did not replace the service executable.

Local qualification on November 10 passed nine session tests, the Node adapter contract suite, Swift compilation of the context helper, five adapter/service helper tests, and six CI-history tests. An intermediate disk-admission build passed 40 Rust tests; the final candidate subsequently passed the expanded 42-test Rust suite and the full local checks recorded below. These are isolated tests, not physical desktop acceptance.

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

A fresh delegated preflight on the existing Air installation failed with `workspace creation failed: No space left on device (os error 28)` before any agent process started. Placement had reported the Air eligible. The task was preserved without replay. The candidate adds disk-readiness checks and a visible panel diagnostic to reject work before assignment when storage is insufficient. Those source changes are not evidence that the older installed services have been upgraded. Free storage and receiving-device access are still required before physical acceptance; a source fix cannot reclaim disk space on a worker that cannot launch.

Final source validation after the disk claim-protocol and symlink corrections passed 42 Rust tests, 25 real-process mesh scenarios, 30 isolated setup scenarios, Clippy, and the desktop/mobile browser regression, including low-space, stale observation and recovery messages. Nine installer tests and nine guarded-session tests passed, along with the native adapter contract suite. The advisory audit completed without reported vulnerabilities. Those November 10 checks were local; later hosted and physical results are recorded separately below. A subsequent November 11 CI-tracker correction passed seven CI-history tests and classifies GitHub's specific pre-start billing annotation as infrastructure without guessing its account cause. That later tooling check is separate from the recorded candidate artifacts.

## Recorded candidate artifacts

This historical `e15fe0e` record is retained separately from the current `f6fd91f` artifacts. Both earlier artifacts were built from clean commit `e15fe0e3e687bda923a453546392519645c40a6c`. The crate VCS record and binary manifest identify that commit. These are unpublished qualification artifacts, not signed production downloads.

| Artifact | Size and contents | SHA-256 |
| --- | --- | --- |
| `citta-0.1.0-aarch64-apple-darwin.tar.gz` | 6,636,424 bytes; 442 manifest entries | `dc4a254727a4b8ad8cd221a9eb344596f4e57fe59bb504814a39434ed88b2348` |
| `citta-0.1.0.crate` | 191,030 bytes; 64 files | `84a7eab5410cf2ca6a017696128cf6ed5dcbbecf96766d47a89339f5ea532497` |

The native archive check used an isolated home on the build Mac and verified fresh installation, required assets, installed version/help and refusal to overwrite an existing installation. It started no service and changed no live configuration. `cargo publish --dry-run --locked`, `cargo package --locked`, and installation from the extracted crate succeeded. The dry run explicitly aborted the upload. Linux and Intel macOS artifacts were not qualified in that earlier record; the current three-platform artifacts are listed above. Signing or any other artifact change requires new checksums and verification.

## November 11 physical Air installation

The earlier `e15fe0e` Apple Silicon archive passed a bounded isolated-install test on a physical ARM64 MacBook Air running macOS 26.2. The download matched the exact archive size and SHA-256 above. Inspection verified safe tar members, all 442 manifest file hashes, clean source commit, target architecture, macOS 14.0 minimum deployment target and the existing valid ad-hoc binary signature.

Installer verification created no installation. A subsequent fresh installation succeeded in an isolated destination, created no service or configuration, and refused a repeated install with the destination files unchanged. Installed version/help and init/check help succeeded. An isolated `init` followed by `check` passed and recorded the default disk reserve as 67,108,864 bytes (64 MiB). Each execution test completed within its 15-second deadline, with no OS denial or killed process.

The tested executable SHA-256 is `12460457cbe4ff9da8d3c05be77d012493b8b9edbd7038db027fa7959291c3af`. Live binary, configuration, service plist and sampled skill hashes remained unchanged. No service, network configuration, desktop interaction or compilation was involved. Evidence was saved outside the disposable task workspace. This establishes isolated installation and execution on that physical device; it does not establish public browser-download quarantine behavior, Developer ID signing/notarization, an upgraded live mesh, or sleep/wake and soak behavior. Separate temporary-daemon mesh checks are recorded below.

## November 11 physical candidate mesh

For the earlier `e15fe0e` check, the physical Mini and Air ran the same executable SHA-256 recorded in the preceding installation section in separate temporary daemons, with isolated configuration/state and a separate private Tailscale Serve endpoint. Automatic pairing succeeded. Echo and shell fixtures reporting the executing hostname and input digest succeeded in both directions, establishing execution on the intended receiving device rather than a local loopback.

Both candidates reported fresh, accepting disk observations with the default 64 MiB reserve. A task requiring a remote ARM64 macOS worker with specified CPU and memory estimates completed on the Air. Reusing the same idempotency key returned the same task ID, and a request with an incorrect authentication token received HTTP 401.

Restarting only the isolated Air daemon interrupted a harmless running delay fixture. The task record retained `interrupted`; it was not automatically replayed. A deliberately separate task submitted after restart succeeded. This is a daemon-restart recovery check, not a physical device sleep/reboot, network-loss or external-action replay test.

Cleanup was verified on both devices: the isolated daemon processes exited, temporary Serve routes were removed, the exact prior Serve configuration was restored, and live executable/configuration/service-plist hashes remained unchanged. The temporary transfer server was also stopped. The remote supervising task completed successfully.

These checks used bounded shell fixtures, not a newly provisioned full agent or model. They do not establish an upgrade of the live services, desktop access, signed public distribution, physical sleep/wake or reboot, network-disconnect recovery, live rollback, or representative soak coverage.

## Air recovery and remaining desktop gate

The failed workspace-creation attempts remain in task history. Once a worker could start again, an initial Citta-build-cache audit found no safe deletion candidates. A later bounded recovery removed 1,914 verified unused npm download-cache blobs totaling 918,748,483 bytes (about 876 MiB). Free space rose to 1,159,290,880 bytes immediately after cleanup; a subsequent independent worker measured 1,044,856,832 bytes. A November 11 UTC worker later measured about 1.28 GiB free. These are point-in-time observations, not a storage reservation or a soak result.

The missing `desktop-context` and `owned-fixture` helpers were compiled for ARM64/macOS 14 or later on the build Mac and verified on the receiving macOS 26.2 device before installation. The later worker confirmed their installed hashes and obtained fresh permission flags: Accessibility false, Screen Recording false. OS attribution identified the installed Citta service as responsible. The guarded desktop lease remained waiting and was ended. The November 11 UTC recheck found the session locked at the login window, with no active displays and both permission flags false. The desktop must be unlocked before actual delivery/capture can be established. No fixture input/capture or external account action occurred, and no live service binary, configuration or permission grant was changed.

The per-file cleanup log was written under a disposable task workspace and was no longer available after automatic workspace cleanup. The returned byte/count totals and subsequent disk/helper checks are retained; do not claim that the removed-path ledger is still available. Future acceptance evidence must be returned with the task result or saved to a protected location outside the disposable workspace.
