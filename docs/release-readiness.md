# Release readiness

Citta is being prepared for its first public release. The release is **not yet qualified**. This record separates prepared source and demonstration evidence from the checks needed before publishing a production tag or launching on Product Hunt.

Last reviewed: November 11, 2026 (UTC). The recorded clean candidate is `e15fe0e3e687bda923a453546392519645c40a6c`, merged through [PR 13](https://github.com/priyankark/citta/pull/13) as `add9e3953a66dc269fdb3dd8d8014402edac8e66`. Those two commits have identical source trees. The artifact hashes below belong to the recorded candidate; later changes require their own qualification record.

## Launch gates

| Gate | Current evidence | What closes it |
| --- | --- | --- |
| Candidate CI | Candidate [run 42865293342](https://github.com/priyankark/citta/actions/runs/42865293342) and merged-main [run 42865550945](https://github.com/priyankark/citta/actions/runs/42865550945) failed before any build/test steps because GitHub reported an account billing or spending-limit block. The November 11 UTC refresh shows no recovery; hosted verification remains blocked until account capacity is restored. | Restore account capacity, then pass the configured suite on the final candidate. Preserve the original failures; do not repeatedly retry the unchanged condition. |
| Download and install | The clean candidate produced the verified Apple Silicon archive and source crate recorded below. The archive passed checksum, isolated fresh install, installed version/help, required-assets and existing-install refusal checks on both the build Mac and a physical ARM64 MacBook Air running macOS 26.2. The crate passed publish dry-run, packaging and installation from its extracted contents into an isolated destination. No registry upload or public binary release occurred. | Build and qualify every other advertised architecture. Verify the intended public download/install path on receiving machines; the isolated physical install is not browser-quarantine, notarization or physical-mesh acceptance. |
| macOS distribution | Neither qualification Mac has a usable Developer ID Application signing identity; only Apple Development identities were found. An existing Developer ID certificate entry was listed as "Not in Keychain", and the attempt to create a new identity was rejected because it requires the Account Holder. No certificate was created, and the candidate is not Developer ID signed or notarized. An earlier transferred binary was rejected by Gatekeeper. The recorded candidate later executed successfully with its existing ad-hoc signature in an isolated physical Air test, without an OS denial; that controlled transfer did not qualify public browser-download quarantine or notarization. | Obtain the intended Developer ID signing identity through the authorized account flow, then sign, notarize and qualify a downloaded artifact on a receiving Mac. Record signing identity continuity and OS permission behavior during upgrade. Do not disable Gatekeeper or edit TCC. |
| Same-build devices | The final archive passed isolated installation on the physical Air. Identical candidate executable bytes then ran as separate temporary daemons on the physical Mini and Air, automatically paired, and completed echo and hostname/input-digest tasks in both directions. These were isolated configurations; installed live services were not upgraded. | Complete the remaining [physical acceptance checklist](acceptance.md#physical-device-qualification--pending), including live upgrade/rollback and representative configured-agent/model behavior. Retain the distinction between temporary qualification daemons and installed services. |
| Recovery and soak | An isolated candidate Air-daemon restart interrupted a harmless running task; a distinct new task succeeded afterward, while the original remained interrupted without replay. Physical device sleep/wake or reboot, network interruption, live upgrade/rollback and representative soak remain outstanding. | Record duration, task counts, failures and outcomes, including uncertain external side effects, on the intended service configuration. |
| Desktop setup | Guarded-session sources are embedded by setup. The two missing native Air helpers were built on the Mini, transferred, checked against pinned source/binary hashes and platform requirements, and installed. A subsequent service-launched worker confirmed their hashes but reported Accessibility and Screen Recording unavailable. The latest November 11 UTC observation found a locked session at the login window, no active displays, and false permission flags. Native input/capture remains unverified. | Unlock the receiving desktop, verify the actual service process has the required OS permissions, restart the idle service if needed, then verify owned-window delivery and capture from a new worker. Do not advertise desktop readiness solely from pairing or helper installation. |
| Local model example | A real Air-origin task previously used the Mini's configured Ollama/Gemma capability. The candidate also completed a real Gemma 3:4b request through two isolated loopback daemons, using the documented adapter without changing live configuration. Ordinary setup does not provision the model or capability automatically. | Complete physical same-candidate acceptance and clean-device model provisioning. The loopback check is not a two-device test; retain the documented model/download requirements. |
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

Final source validation after the disk claim-protocol and symlink corrections passed 42 Rust tests, 25 real-process mesh scenarios, 30 isolated setup scenarios, Clippy, and the desktop/mobile browser regression, including low-space, stale observation and recovery messages. Nine installer tests and nine guarded-session tests passed, along with the native adapter contract suite. The advisory audit completed without reported vulnerabilities. These remain local checks; the full Air and hosted-platform gates above are open. A subsequent November 11 CI-tracker correction passed seven CI-history tests and classifies GitHub's specific pre-start billing annotation as infrastructure without guessing its account cause. That later tooling check is separate from the recorded candidate artifacts.

## Recorded candidate artifacts

Both artifacts were built from clean commit `e15fe0e3e687bda923a453546392519645c40a6c`. The crate VCS record and binary manifest identify that commit. These are unpublished qualification artifacts, not signed production downloads.

| Artifact | Size and contents | SHA-256 |
| --- | --- | --- |
| `citta-0.1.0-aarch64-apple-darwin.tar.gz` | 6,636,424 bytes; 442 manifest entries | `dc4a254727a4b8ad8cd221a9eb344596f4e57fe59bb504814a39434ed88b2348` |
| `citta-0.1.0.crate` | 191,030 bytes; 64 files | `84a7eab5410cf2ca6a017696128cf6ed5dcbbecf96766d47a89339f5ea532497` |

The native archive check used an isolated home on the build Mac and verified fresh installation, required assets, installed version/help and refusal to overwrite an existing installation. It started no service and changed no live configuration. `cargo publish --dry-run --locked`, `cargo package --locked`, and installation from the extracted crate succeeded. The dry run explicitly aborted the upload. Linux and Intel macOS candidate artifacts remain unqualified. Signing or any other artifact change requires new checksums and verification.

## November 11 physical Air installation

The recorded Apple Silicon archive passed a bounded isolated-install test on a physical ARM64 MacBook Air running macOS 26.2. The download matched the exact archive size and SHA-256 above. Inspection verified safe tar members, all 442 manifest file hashes, clean source commit, target architecture, macOS 14.0 minimum deployment target and the existing valid ad-hoc binary signature.

Installer verification created no installation. A subsequent fresh installation succeeded in an isolated destination, created no service or configuration, and refused a repeated install with the destination files unchanged. Installed version/help and init/check help succeeded. An isolated `init` followed by `check` passed and recorded the default disk reserve as 67,108,864 bytes (64 MiB). Each execution test completed within its 15-second deadline, with no OS denial or killed process.

The tested executable SHA-256 is `12460457cbe4ff9da8d3c05be77d012493b8b9edbd7038db027fa7959291c3af`. Live binary, configuration, service plist and sampled skill hashes remained unchanged. No service, network configuration, desktop interaction or compilation was involved. Evidence was saved outside the disposable task workspace. This establishes isolated installation and execution on that physical device; it does not establish public browser-download quarantine behavior, Developer ID signing/notarization, an upgraded live mesh, or sleep/wake and soak behavior. Separate temporary-daemon mesh checks are recorded below.

## November 11 physical candidate mesh

The physical Mini and Air ran the same candidate executable SHA-256 recorded above in separate temporary daemons, with isolated configuration/state and a separate private Tailscale Serve endpoint. Automatic pairing succeeded. Echo and shell fixtures reporting the executing hostname and input digest succeeded in both directions, establishing execution on the intended receiving device rather than a local loopback.

Both candidates reported fresh, accepting disk observations with the default 64 MiB reserve. A task requiring a remote ARM64 macOS worker with specified CPU and memory estimates completed on the Air. Reusing the same idempotency key returned the same task ID, and a request with an incorrect authentication token received HTTP 401.

Restarting only the isolated Air daemon interrupted a harmless running delay fixture. The task record retained `interrupted`; it was not automatically replayed. A deliberately separate task submitted after restart succeeded. This is a daemon-restart recovery check, not a physical device sleep/reboot, network-loss or external-action replay test.

Cleanup was verified on both devices: the isolated daemon processes exited, temporary Serve routes were removed, the exact prior Serve configuration was restored, and live executable/configuration/service-plist hashes remained unchanged. The temporary transfer server was also stopped. The remote supervising task completed successfully.

These checks used bounded shell fixtures, not a newly provisioned full agent or model. They do not establish an upgrade of the live services, desktop access, signed public distribution, physical sleep/wake or reboot, network-disconnect recovery, live rollback, or representative soak coverage.

## Air recovery and remaining desktop gate

The failed workspace-creation attempts remain in task history. Once a worker could start again, an initial Citta-build-cache audit found no safe deletion candidates. A later bounded recovery removed 1,914 verified unused npm download-cache blobs totaling 918,748,483 bytes (about 876 MiB). Free space rose to 1,159,290,880 bytes immediately after cleanup; a subsequent independent worker measured 1,044,856,832 bytes. A November 11 UTC worker later measured about 1.28 GiB free. These are point-in-time observations, not a storage reservation or a soak result.

The missing `desktop-context` and `owned-fixture` helpers were compiled for ARM64/macOS 14 or later on the build Mac and verified on the receiving macOS 26.2 device before installation. The later worker confirmed their installed hashes and obtained fresh permission flags: Accessibility false, Screen Recording false. OS attribution identified the installed Citta service as responsible. The guarded desktop lease remained waiting and was ended. The November 11 UTC recheck found the session locked at the login window, with no active displays and both permission flags false. The desktop must be unlocked before actual delivery/capture can be established. No fixture input/capture or external account action occurred, and no live service binary, configuration or permission grant was changed.

The per-file cleanup log was written under a disposable task workspace and was no longer available after automatic workspace cleanup. The returned byte/count totals and subsequent disk/helper checks are retained; do not claim that the removed-path ledger is still available. Future acceptance evidence must be returned with the task result or saved to a protected location outside the disposable workspace.
