# Release readiness

Citta is being prepared for its first public release. The release is **not yet qualified**. This record separates prepared source and demonstration evidence from the checks needed before publishing a production tag or launching on Product Hunt.

Last reviewed: November 12, 2026 (UTC). The current tested source is `f6fd91fa7978e16b5aca6dec227c1e303de0daea`, the [PR 14](https://github.com/priyankark/citta/pull/14) merge. Its exact artifacts and physical checks are recorded below. Earlier `e15fe0e` evidence and failed attempts remain as history. A later documentation-only evidence update does not change the source provenance of these binaries; runtime or packaging changes require a new qualification record.

## Setup recovery correction awaiting qualification

A subsequent review reproduced a first-setup retry defect: if Tailscale Serve failed before enrollment completed, retry could retain the legacy resource-admission defaults. The correction writes setup defaults in the initial configuration, preserves intentional settings on existing installations, and adds actionable HTTPS/operator recovery guidance without exposing subprocess output. Local validation passed 43 Rust tests and 33 onboarding scenarios, including failure/retry, initial publication, existing-policy preservation and sanitized timeout errors. Independent review passed. These runtime changes require a new candidate; the signed `f6fd91f` artifacts and physical evidence below remain historical and do not qualify the corrected executable.

## Launch gates

| Gate | Current evidence | What closes it |
| --- | --- | --- |
| Candidate CI | Main [run 42974236178, attempt 2](https://github.com/priyankark/citta/actions/runs/42974236178/attempts/2) passed on `f6fd91f`: Linux, Intel macOS (including ARM compilation) and audit all completed successfully. Native [candidate packaging run 42974812756](https://github.com/priyankark/citta/actions/runs/42974812756) also passed all three platform jobs. The original billing failure is retained and linked to the successful retry. | This CI gate is satisfied for the pinned source. Requalify runtime or packaging changes beyond it; a later documentation-only commit is not the source of these tested binaries. |
| Download and install | Verified native archives and source-crate checks passed. The signed ARM archive also passed a real Chrome download from the private draft, native tar extraction and isolated installation with the original quarantine attribute preserved. Native signed Intel installation passed in hosted CI. The private draft contains nine verified assets; no crate upload or public binary release occurred. | Verify a clean OS account and the intended public download/setup path. The existing-account private-draft browser check does not establish either. |
| macOS distribution | Both exact signed macOS binaries passed strict Developer ID verification, Apple notarization acceptance and online ticket checks. Signed ARM physical checks and native signed Intel [run 43153509046](https://github.com/priyankark/citta/actions/runs/43153509046) passed. Earlier certificate-creation, Gatekeeper and draft-download failures remain recorded. | Qualify the public download and clean-user setup, Intel browser-quarantine behavior and advertised desktop permissions. Offline ticket availability is unverified. |
| Same-build devices | Both live Macs now retain the exact signed ARM candidate. Fresh service-launched agents used Bash, wrote/tested Python and read installed skills; loaded Mini MCP discovery, bidirectional echo and an actual Air-origin Mini Gemma task passed. Current history, identity, configuration, credentials, policy and routes were preserved. | Coding deployment is complete for these existing accounts. Complete the remaining [physical acceptance](acceptance.md#physical-device-qualification--pending), especially clean-user provisioning and desktop permission migration. |
| Recovery and soak | The original ad-hoc candidate completed a 1,267.61-second lightweight physical soak with 110 forward and 41 reverse jobs, bounded endpoint/restart recovery and cleanup. Isolated rollback and a real Mini old→signed→old rehearsal preserved current state. Coordinated signed coding deployment is complete; a separate 30-minute signed agent/model soak passed all 136 workloads with cleanup and both services idle. | Physical sleep/wake or reboot, whole-device network-loss recovery and representative multi-hour stress coverage remain unverified. Keep each result tied to its actual workload and binary. |
| Desktop setup | Native Air helpers are installed and verified. During signed rollout, a fresh Air worker still found a locked session, no displays and false permission flags. The Mini was unlocked with one display, but its signed worker reported Accessibility and Screen Recording false; the prior binary had both true. No input/capture was performed. | Complete normal OS grants for the actual signed service, then verify owned-window input and capture from a fresh worker on each advertised receiving device. Coding deployment does not establish desktop readiness. |
| Local model example | The original CI candidate returned categorization output from the Mini's Gemma 3:4b through an Air-origin task. The retained signed live deployment repeated that path successfully with a fresh resource-qualified task and 26 generated tokens. | Verify clean-user model download/provisioning and representative workload budgets. Ordinary setup does not provision the model or capability automatically. |
| Source and notices | PolyForm Noncommercial 1.0.0, attribution notices and dependency-license collection are prepared. The latest targeted scan covered 300 unique blobs reachable through local refs at `4c782ac`, following the earlier 250-blob scan. No high-confidence credential signatures were found; reviewed matches were synthetic fixtures or compressed PNG byte patterns. | A separate sampled media review found no unintended private data in 104 frames, 45 captions and 2,210 OCR lines. Review final changes and retain third-party terms; signature scans, sampling and OCR can miss content. |
| Product Hunt entry | The Product Hunt draft is created with the new tagline, description, maker role/comment, free pricing, three categories, logo and four gallery images. Saved gallery order is visually verified. Product Hunt confirms the draft is not scheduled. The README demo is local; hosted video is pending. User-selected launch target is November 14. | A working public product URL, usable install, qualified advertised functionality, hosted video with the adjacent license caption and completed external listing; schedule only after the gates above pass. |

## Current candidate artifacts and native checks

All files in this table identify clean source `f6fd91fa7978e16b5aca6dec227c1e303de0daea`. Native [packaging run 42974812756](https://github.com/priyankark/citta/actions/runs/42974812756) completed successfully on Apple Silicon macOS, Intel macOS and Linux x86-64. Each job built its native archive and passed the native installer check; downloaded archives were then checked against every manifest hash. These are unpublished artifacts, not signed production downloads.

| Artifact | Size and contents | SHA-256 |
| --- | --- | --- |
| `citta-0.1.0-aarch64-apple-darwin.tar.gz` | 6,607,300 bytes; 442 manifest entries | `a004a35c43cdc38b0fb97a92cb48f4ba14ca7cdf9a66d1062bc791cbce40aeb5` |
| `citta-0.1.0-x86_64-apple-darwin.tar.gz` | 6,997,680 bytes; 442 manifest entries | `81361b680eb39a88ad52869dbaa573d9f0858de98a1567839d4f7e047f218e5a` |
| `citta-0.1.0-x86_64-unknown-linux-gnu.tar.gz` | 7,238,848 bytes; 434 manifest entries | `9a59db1f9eb79be062edada4a6ec41d69874b54aaf51027257e0a26924b5b732` |
| `citta-0.1.0.crate` | 191,028 bytes; 64 files | `1bbedb213c9d96883fe4a2931ae17a941f0b54a9b8295823295aa8081343629d` |

The crate's VCS record identifies the same clean source. Packaging, `cargo publish --dry-run --locked`, installation from extracted contents and installed version/setup-help checks passed locally on Apple Silicon. Required embedded assets and notices were present; private, media and launch-material paths were excluded. The dry run did not upload to crates.io. Signing or any other change to these artifacts requires new hashes and verification.

## Developer ID signed staging

Staged copies of both macOS binaries from clean source `f6fd91fa7978e16b5aca6dec227c1e303de0daea` were signed with the intended Developer ID Application identity, stable identifier `io.citta.daemon`, hardened runtime and secure timestamps. No additional entitlements were added. Strict signature verification passed, and all 442 files in each updated manifest were verified. The original CI archives remain unchanged. These are unpublished staged artifacts with `release_qualified: false`; both ZIPs were submitted once after supported authentication was validated, and Apple accepted both.

| Target | Signed executable SHA-256 | Prepared notarization ZIP SHA-256 |
| --- | --- | --- |
| Apple Silicon macOS | `5551c4eae2c43ebe958d6ad0d79919062f5b20a40e334355e392c6f4597fea76` | `702547135f6b958a8e6d97a6b10e4fdbe98036dc8aef956c73bedae01c511654` |
| Intel macOS | `98ae3a933036566c5da424d0e3acd737f5ab712a21235e4c9591eef76cfd80a4` | `62178e7b5ee75c6d2e4da80271e7d9794bcd27f023c828df627210b22458ebe4` |

The earlier Xcode route rejected certificate creation with an Account Holder requirement and showed an existing certificate unavailable in Keychain. That attempt is retained, but the authorized web portal subsequently issued a usable Developer ID Application certificate, which was imported with its matching private key. The certificate blocker is resolved; notarization authentication, submission and Apple acceptance are complete, while clean-user/public-download and desktop permission qualification remain open. Native signed Intel execution subsequently passed as recorded below. An earlier transferred binary's Gatekeeper denial also remains in history. Later execution of an ad-hoc candidate did not qualify browser quarantine or public distribution. See the [signing procedure](signing.md).

The Apple Silicon submission is `1e0a3998-8722-4015-8a33-16d0dd1aed8b`; Intel is `cf98a983-6da8-462c-a902-fcefa8109305`. The Apple Silicon submission was `In Progress` at 17:39 UTC; at the 17:40 UTC check both were `Accepted`. Apple logs and receipts were retained. Final tar archives contain the accepted signed executable and unchanged signed manifest:

| Unpublished signed/notarized archive | Bytes | SHA-256 |
| --- | --- | --- |
| `citta-0.1.0-aarch64-apple-darwin.tar.gz` | 6,558,442 | `739b7bf3cd4c9b89163ca8f7cb3aa30d1d3335fe78df3887a5a5203448cecbc5` |
| `citta-0.1.0-x86_64-apple-darwin.tar.gz` | 7,163,974 | `cbac9030a69d19636bd6d4244a9ac36370477bcb42bc38f27cfbdb4f0ad4414b` |

Explicit `codesign --verify --strict -R=notarized --check-notarization` checks passed for both bare executables. The separate `spctl --assess --type execute` attempts returned exit 3 with "the code is valid but does not seem to be an app"; these rejected assessments remain recorded. No quarantine or system policy was changed. Online ticket verification is not public browser-download acceptance or a claim of a stapled container/offline Gatekeeper availability.

The signed Apple Silicon bundle separately passed installer verification, a fresh isolated installation, all 442 installed hashes, version/help/setup-help execution and repeat-install refusal. The final tar archive also passed checksum, fresh isolated install, version/help, required-asset and existing-install refusal checks. Live services and the existing home installation stayed unchanged. The signed Intel bundle correctly refused installation on the ARM host; Rosetta was unavailable, so the signed Intel executable was not run locally. Original native Intel CI success remains valid for the earlier CI artifact, not as execution evidence for these new signed bytes. These initial checks did not establish browser-quarantine behavior or signed desktop permissions. The subsequent native Intel and bounded ARM browser checks are recorded below.

The earlier physical soak and isolated rollback used the original ad-hoc candidate. Separate signed ARM physical results are recorded next; neither set of temporary-daemon checks establishes live desktop permission continuity.

## Native signed Intel and ARM browser installation — November 12

[Native Intel verification run 43153509046](https://github.com/priyankark/citta/actions/runs/43153509046) passed on macOS 26.6.1, architecture `x86_64`, using the exact existing signed archive and executable hashes above without rebuilding. Strict signature, intended identity/identifier, hardened runtime, secure timestamp, designated requirement and online notarization checks passed, followed by native setup help, fresh isolated installation/version/help, existing-install refusal and checks that no service/configuration state was created. This qualifies native signed Intel CLI execution and isolated installation, not Intel desktop, physical mesh, browser quarantine or offline ticket lookup.

The verifier's [initial run 43152970631](https://github.com/priyankark/citta/actions/runs/43152970631) failed with HTTP 403 when reading the private draft, before artifact download or execution. The manual private-repository job received the permission GitHub requires to read draft releases; its download uses GET only and execution checks use a sanitized environment. The corrected run passed. The failed attempt remains recorded, and neither workflow change rebuilt the candidate.

Signed-in Chrome downloaded the exact signed ARM archive from the private GitHub draft on the existing Mini account. Native `/usr/bin/tar -xzf` extraction and the normal Python installer preserved the original browser quarantine value exactly on the installed executable. Its hash matched the signed candidate; version/setup help passed and a repeat installation was refused. No quarantine attributes or OS policies were modified. An earlier Archive Utility attempt produced no extraction; native tar was the successful path. This isolated HOME/prefix test is not a new OS account, public URL, setup or live-rollout check. See [distribution scope](signing.md#completed-distribution-checks).

## Signed ARM physical checks

The exact signed/notarized ARM archive above passed fresh isolated installation on both physical Macs. Each verified the same signed executable hash and designated requirement, strict signature validity, hardened runtime and notarization before execution. Temporary daemons automatically paired across private HTTPS and passed bidirectional echo plus executing-host/input-digest proof, fresh resource/disk checks, same-key task identity, incorrect-token rejection and a three-job short batch.

A real receiving Codex worker used Bash, wrote and tested Python, and read the installed delegation skill. An intentional worker interruption and daemon restart left the original task interrupted without replay; a distinct new task succeeded. Independent cleanup verified owned daemons/processes/listeners exited, exact prior Serve configurations were restored and live binary/configuration/plist hashes stayed unchanged. The remote supervisor completed successfully.

This closes the bounded signed ARM installation/execution and temporary physical mesh checks. Separate November 12 evidence in this record adds native signed Intel execution, a private-draft ARM browser install and coordinated live coding deployment. Clean-user provisioning, public-download UX, desktop grants and whole-device sleep/reboot/network loss remain unverified. The longer 21-minute workload below belongs to the original ad-hoc binary; the signed short batch is not the same soak.

## Current candidate physical agent and model checks

The exact CI Apple Silicon archive above was installed into fresh isolated destinations on the physical Mini and Air. Both ran executable SHA-256 `52df8c5bcfee57061eb829ac8d01e54fdd2ad78a7fd6a67c0e060a543a7600a0` with separate temporary configuration/state. Automatic same-owner pairing, echo and executing-host/input-digest fixtures succeeded across private HTTPS in both directions. Fresh disk/resource observations, a resource-qualified remote task, same-key task identity and wrong-token HTTP 401 were verified.

An actual full-tool Codex process on the Air used Bash, wrote a Python file, ran it and asserted its result, and read the installed delegation skill. This verifies shell/code/skill availability through the candidate worker; it does not claim every configured tool, app login or provider is provisioned on a clean user account. A separate Air-origin task used the Mini's actual Gemma 3:4b through the candidate Ollama adapter and returned categorization output with 32 generated tokens.

A short batch completed 20/20 lightweight remote tasks over 197.38 seconds. A controlled restart of the isolated Air daemon interrupted a harmless running fixture; a deliberately distinct task succeeded afterward while the original remained interrupted without replay. This is bounded task/restart coverage, not a representative long-running soak or a device sleep/reboot/network-loss test.

Both temporary daemons and their task processes exited. Independent cleanup checks found no remaining test listeners or owned processes, exact prior Serve configurations were restored, and live executable/configuration/service-plist hashes remained unchanged. The transfer server stopped and the remote supervisor finished successfully. These checks did not upgrade live services, change desktop grants, provision a clean user's provider/model account, or qualify signed/notarized public distribution.

## Physical lightweight soak and recovery

The original exact CI Apple Silicon executable (`52df8c5bcfee57061eb829ac8d01e54fdd2ad78a7fd6a67c0e060a543a7600a0`) ran in separate temporary daemons on both Macs for 1,267.61 seconds, just over 21 minutes. All 110 forward and 41 reverse lightweight jobs completed without unexpected workload failures. The workload alternated echo with Bash/Python file-and-digest checks; it was not a sustained agent/model stress test.

For 35 seconds, only the test origin's owned private endpoint was unavailable. A task stayed queued during the outage and succeeded with the same identity after restoration. A controlled receiving-daemon restart interrupted a harmless running task; the original remained interrupted without replay. These checks establish the bounded endpoint/restart behavior, not a whole-device network disconnect or sleep/reboot recovery.

Both devices supplied 42 resource observations. Independent cleanup confirmed owned daemons/task processes and test listeners exited, exact prior Serve configurations were restored and live executable/configuration/service-plist hashes were unchanged. The remote supervisor completed successfully. The earlier unrelated live-service interruption remains unexplained. This result does not qualify live upgrades, signed-artifact behavior or multi-hour representative soak coverage.

## Publication staging

A private `v0.1.0` draft release targets `f6fd91fa7978e16b5aca6dec227c1e303de0daea` and contains nine verified assets: archives, adjacent manifests and checksum files for signed/notarized Apple Silicon and Intel macOS plus the Linux candidate. All uploaded asset SHA-256 values matched the local files. The release is unpublished and no `v0.1.0` tag has been created.

The launch logo and four gallery images were reviewed at full size, with their hashes and listing links verified. The unchanged 110.8-second README video received a separate privacy/story review: 104 sampled frames, all 45 captions and 2,210 OCR lines, including scene-boundary samples, were checked. No unintended private data or publication-blocking leak was observed in that sample. Visible task UUIDs and generic device labels are not credentials; the public repository attribution is intentional. This was not frame-by-frame inspection or independent audio transcription, and OCR/sampling can miss content. Preserve the adjacent noncommercial license caption when hosting the video.

The registry account is ready and a crate-scoped publishing credential is configured locally. Source-crate packaging, publish dry-run and isolated installation passed as recorded above; no crate upload occurred. Draft assets and configured credentials do not close the remaining functional/public-install acceptance gates.

## Isolated upgrade and rollback rehearsal

All seven offline-upgrade tests passed against the exact CI Apple Silicon executable. A separate real-process rehearsal copied the existing installed binary into an isolated installation, ran successful and intentionally failed tasks, upgraded to the pinned candidate, completed new work, then rolled back only the binary using the same offline helper. The current schema-4 database, configuration and credential remained intact; all four task records survived, including the failure and the task created under the candidate. New work succeeded after rollback. The helper created a fresh backup of current state before each replacement. All owned processes stopped, the fixture port closed, and live executable/configuration/service-plist hashes stayed unchanged.

Compatibility is conditional: the older executable accepts the preserved existing configuration but rejects a newly generated candidate configuration because it does not recognize `admission.disk_headroom_bytes`. Recheck the prior binary against the current configuration as well as the current schema before any rollback; do not restore stale history to recover compatibility. See [upgrade and rollback](operations.md#upgrade-and-rollback). This isolated rehearsal did not replace either live service, exercise launchd maintenance or establish signed-binary/desktop permission continuity.

## Bounded Mini live upgrade and rollback

After the signed physical tests completed cleanup, both live origin queues and workers were checked idle and submissions were quiesced. The actual Mini launchd service was stopped, current state was privately backed up through the offline helper, and the verified signed ARM executable replaced the installed binary atomically. The unchanged service plist then started the signed service successfully. Schema 4, node identity, configuration, credentials and all 82 original task records were preserved.

A Mini-local echo and a fresh service-launched Codex worker both succeeded. The worker wrote and tested Python and ran a Bash assertion; its process ancestry and executable hash confirmed the signed service. Its Accessibility and Screen Recording flags were both false, whereas the earlier old-service worker had reported both true. No input/capture was attempted. The desktop became unavailable before the normal OS approval flow could complete, and no OS grant was changed.

The service was quiesced again, a new backup of current state was taken, and only the exact prior binary was restored. All 84 records present before rollback, including the signed-phase tasks, remained intact. The original service restarted, targeted echo succeeded in both directions with the Air, and a new old-service worker again reported both permission flags true. That observation still had no active display, so actual desktop input/capture remained unverified. Final checks preserved all prior task payloads, identity, configuration, credentials, service plist, policy and audit records, with exact Serve restoration and no queued/running work on either device.

This November 11 rehearsal qualifies the bounded Mini live launchd upgrade and compatible-current-state rollback for these exact binaries. It left both devices on their original binaries at that time. The newer origin rejects older worker disk claims, so leaving only the Mini upgraded would impair delegation to the old Air. The subsequent coordinated coding deployment is recorded below; desktop permission continuity remains open. No stale database/configuration was restored and no task was replayed.

## Coordinated signed live coding deployment — November 12

Both live Macs now retain the exact signed/notarized ARM executable `5551c4eae2c43ebe958d6ad0d79919062f5b20a40e334355e392c6f4597fea76` from source `f6fd91fa7978e16b5aca6dec227c1e303de0daea`. Both origin queues and workers were checked idle before maintenance. An explicit, bounded launchd-owned controller allowed the Air service to be replaced without terminating its supervising recovery process. Each offline replacement made a private backup of current state; the previous binaries remain available. Existing compatible configuration, schema-4 identity, credentials, service plists, policy, audit and original task payloads were preserved, and Serve configuration was unchanged.

Fresh service-launched Codex workers on both devices ran Bash, wrote and tested Python and read installed skills. The Mini worker also discovered Citta through its loaded MCP connection. Echo succeeded in both directions; an actual Air-origin resource-qualified task invoked the Mini's Gemma 3:4b and returned correct synthetic expense categories with 26 generated tokens. The deployment was explicitly committed for coding use. The Air controller exited and its temporary maintenance job was unloaded. All maintenance tasks were collected, and both origins and managed workers were idle at the final handoff.

The Mini's fresh signed worker had one active display but both desktop permission flags false, compared with true/true under the prior binary. The Air remained locked, with no active displays and both flags false. No input or capture was attempted. Retaining the working coding deployment does not qualify desktop permission migration; normal OS approval and actual worker delivery/capture remain required.

Preparation retained two failures before any service change: a worker looked for an origin task in the wrong device's database, then standalone Python HTTPS lacked local issuer trust. The private controller was corrected to use authenticated system curl with normal TLS verification and credentials passed only through stdin; 11 isolated controller tests passed. A later cleanup-report task timed out after 180 seconds and was not replayed. A distinct read-only inspection confirmed the controller was committed/unloaded and the exact signed service healthy. The timeout's cause is unproven; no runtime fix or deadlock diagnosis is claimed. The controller checked all original Air task payloads before commit; the independent inspection confirmed installation/configuration hashes but returned no independent full-history comparison.

A separate signed coding/model soak ran for 1,800.31 seconds on the Mini origin and 1,800.37 seconds on the Air origin. All 136 workloads succeeded: six real Air Codex tasks, ten Mini Gemma tasks and 120 bidirectional echoes. There were no failures, interruptions, cancellations, retries or outstanding submissions. The separate Air supervisor also succeeded and is excluded from the workload count. Sampled daemon RSS stayed within 17,648–20,560 KiB on the Mini and 19,728–21,504 KiB on the Air; model-process RAM was not measured separately. Resource values used for placement were admission estimates, not OS-enforced limits. Owned scripts, transfer server and receiving workspaces were cleaned up; both live services were healthy and idle. This is 30-minute coverage of the signed `f6fd91f` executable, not a multi-hour test or evidence for a later runtime change. The completed deployment establishes existing-account coding operation, not clean-user setup, desktop continuity, whole-device sleep/network recovery or multi-hour reliability. Future rollback must quiesce both devices, check the prior binary against current configuration/schema and preserve current history; never restore stale state or leave an incompatible mixed-version cluster. See [rollback constraints](operations.md#upgrade-and-rollback).

## Live preflight interruption and follow-up

At 17:21 UTC on November 11, a bounded preflight on the older live Air installation was interrupted after about 18 seconds with `lease renewal failed or task cancelled`. The origin retained queued/running/interrupted events, and the worker successfully reported that interruption. Its 300-second task deadline had not elapsed. No automatic replay occurred.

A distinct read-only diagnostic completed successfully and found only the worker's start and interruption-report log entries, without the underlying renewal error. The current implementation collapses renewal errors into a generic outcome, so this evidence does not establish a transport failure, cancellation or code race as the cause. The unexplained incident remains open and is separate from candidate qualification.

The fresh service-worker observation at 17:29 UTC found the Air locked at the login window, no active displays, both Accessibility and Screen Recording flags false, and 1,214,992,384 bytes (about 1.13 GiB) free. It did not identify current OS permission attribution or perform input/capture. Ordinary shell work remained available. No live service, route, configuration or permission grant was changed by these checks.

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
