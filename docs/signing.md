# macOS signing and notarization

## Final candidate `c767ce5` — November 12, 2026 UTC

The current artifact source is `c767ce5b5c1ad283c00afad9684a4f809a7c857e`. Its Apple Silicon and Intel executables were separately signed with the intended Developer ID Application identity, stable identifier `io.citta.daemon`, hardened runtime and secure timestamps. Strict signature verification, manifest verification and explicit online notarization-ticket checks passed for both. Apple accepted ARM submission `0de8fa63-6ee0-4b5a-831d-2b0cef78722c` and Intel submission `a5386212-b6e7-4a54-8f31-5eb6656ac92b`; receipts and logs are retained. Signing was a separate operation after native packaging, not an output of the packaging workflow itself. Exact final artifact provenance and hashes are in [release readiness](release-readiness.md).

[Native signed Intel verification run 43164792893](https://github.com/priyankark/citta/actions/runs/43164792893) passed on macOS 26.6.1, architecture `x86_64`, using the existing signed archive SHA-256 `689c14b36826ab9cf52bb7f90dd79e18052bc0ef8ac1851d0f0b847094dc7200` without rebuilding. The verifier pinned the repository identity, unpublished draft, source, asset IDs and hashes. It checked the intended signing certificate/team, identifier, hardened runtime, timestamp, designated requirement and online ticket, then ran native setup help and isolated installation/version/help. All 442 manifest files and required assets were verified; repeat installation was refused and no service/configuration state was created. The job's draft-reading permission is confined to authenticated GET downloads; executable checks use a sanitized environment. The repository being public does not make an unpublished draft anonymously downloadable.

The exact signed ARM archive then passed fresh isolated installation and execution on both physical Macs, real private pairing and bidirectional tasks, an actual receiving coding worker, Mini Gemma inference, bounded daemon restart/cancellation and a 20.208-second owned-endpoint outage followed by queued-task recovery. Two setup-failure/retry fixtures passed on each Mac. All 18 candidate tasks reached their expected terminal states, including the deliberate cancellation and interruption; the separate supervisor succeeded. Cleanup restored prior routes, stopped owned processes and preserved the live installations. See [physical acceptance](acceptance.md#final-candidate-c767ce5--2026-11-12) for the precise scope. The earlier `f6fd91f` deployment, browser-download check and soak do not transfer to these new bytes.

[Version 0.1.0](https://github.com/priyankark/citta/releases/tag/v0.1.0) was published on November 12 at 06:03:10 UTC with the verified final archives, adjacent manifests and checksums; the exact source-crate upload also completed. All nine anonymous downloads and a fresh registry-only installation passed. The public ARM browser download/install check preserved quarantine and passed execution; see the follow-up below. Clean OS-account provisioning, Intel browser quarantine, signed-worker desktop acceptance and offline ticket availability remain unverified. The final physical smoke did not exercise a live upgrade, device sleep or whole-device network loss. A later branding/documentation commit does not change the fixed artifact source or the README bundled with it.

## Historical `f6fd91f` signing and deployment

**Earlier review — November 12, 2026 UTC:** staged Apple Silicon and Intel macOS executables from source `f6fd91fa7978e16b5aca6dec227c1e303de0daea` are Developer ID Application signed with identifier `io.citta.daemon`, hardened runtime and a secure timestamp. Both passed strict signature verification; updated manifests and submission ZIPs were prepared. Both ZIPs were submitted once to Apple after notarization credentials were validated and stored in a local Keychain profile. Both submissions were `Accepted` at the November 11, 17:40 UTC check; Apple logs and final archive hashes are retained. Explicit online notarization-ticket verification passed for both bare executables. Signed ARM fresh installation and temporary physical mesh checks passed on both Macs. Native signed Intel execution and isolated installation now pass in hosted CI. A real Chrome download of the private ARM draft also passed native tar extraction and installation with its original quarantine attribute preserved. Clean OS-account/public-URL qualification and signed desktop permission acceptance remain open. The CI packaging workflow itself still produces development candidates; signing was a separate maintainer operation.

An earlier Xcode attempt reported that certificate creation required the Account Holder, and an existing certificate was unavailable in Keychain. That failed attempt remains part of the record. The authorized web portal subsequently issued a Developer ID Application certificate, which was imported with its matching private key and used for the staged signatures. The earlier account-context failure is no longer the certificate blocker. Notarization authentication, submission and Apple acceptance are now complete. Never put signing secrets or notarization passwords in the repository, chat, shell history, or workflow output.

The November 11 bounded live Mini old→signed→old rehearsal passed service startup and rollback while preserving current state. The fresh signed worker lost both desktop permission flags; the prior binary restored those flags after rollback. No grants were changed, and actual input/capture remained unverified because the desktop became unavailable. A stable signing identifier supports future identity continuity, but it does not automatically migrate an old ad-hoc installation's OS grants. Both Macs subsequently retained the historical `f6fd91f` signed candidate for [coding work on November 12](release-readiness.md#coordinated-signed-live-coding-deployment--november-12); desktop permission migration remains pending.

The procedure below records the completed signing/notarization steps and the remaining distribution qualification. Exact signed hashes are listed in [release readiness](release-readiness.md#developer-id-signed-staging). Signing changes the executable identity and bytes; it does not extend the unsigned artifact's physical execution or rollback results to the signed artifact.

Apple requires Developer ID distribution signing, hardened runtime and a secure timestamp for notarization. See [Apple's notarization requirements](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution) and [signing command-line executables](https://developer.apple.com/documentation/xcode/creating-distribution-signed-code-for-the-mac).

<a id="completed-distribution-checks"></a>

## Historical `f6fd91f` distribution checks

On November 12 UTC, [native Intel verification run 43153509046](https://github.com/priyankark/citta/actions/runs/43153509046) passed on macOS 26.6.1, architecture `x86_64`. It downloaded the existing private draft's exact signed archive (`cbac9030a69d19636bd6d4244a9ac36370477bcb42bc38f27cfbdb4f0ad4414b`) without rebuilding. The executable hash was `98ae3a933036566c5da424d0e3acd737f5ab712a21235e4c9591eef76cfd80a4`. Strict signature, Developer ID certificate/team, identifier, hardened runtime, secure timestamp, designated requirement and explicit online notarization checks passed, followed by native setup help and fresh isolated installation/version/help. The installer refused an existing destination and created no service/configuration state. This does not qualify Intel desktop access, physical mesh operation, browser quarantine or offline ticket lookup.

The manual verifier was added in [PR #17](https://github.com/priyankark/citta/pull/17). Its [initial run 43152970631](https://github.com/priyankark/citta/actions/runs/43152970631) failed with HTTP 403 while reading the private draft, before downloading or executing the candidate. [PR #18](https://github.com/priyankark/citta/pull/18) granted `contents: write` only to that manual, private-repository job because GitHub requires push access to read draft releases. The download code uses GET requests only, checkout does not persist credentials, and executable/installer checks use a sanitized environment. The corrected run passed; the initial failure remains recorded. Neither change rebuilt or altered the release artifacts.

On the physical Mini, signed-in Chrome downloaded the exact ARM archive (`739b7bf3cd4c9b89163ca8f7cb3aa30d1d3335fe78df3887a5a5203448cecbc5`) from the private GitHub draft. `/usr/bin/tar -xzf` and the normal Python installer preserved its original `com.apple.quarantine` value on the installed executable. The installed binary matched `5551c4eae2c43ebe958d6ad0d79919062f5b20a40e334355e392c6f4597fea76`; version and setup help exited successfully, and a repeated installation was refused. No quarantine attributes or OS policies were changed. This used an isolated HOME/prefix under the existing Mini account, **not a new OS account or a public release URL**, and did not run setup or qualify a live rollout. Archive Utility produced no extracted output during the earlier attempt; the successful path was the documented tar extraction.

Private evidence is retained in `local/release-prep/intel-verification-plan/native-signed-intel-summary.json`, its `run-43153509046-attempt-1/` reports, and `local/release-prep/next-ready/browser-download.json`, `browser-quarantine-probe.json` and `browser-installer-report.json`. These bounded checks do not change the manifests' `release_qualified: false` or establish desktop permission acceptance.

## Prepare the candidate and identity

Use the reviewed, clean source commit and the native packaging workflow. Keep its original archive and checksum unchanged. The signing procedure changes the binary, so it must produce a **new** archive and new checksums; the unsigned archive's hashes no longer apply.

Inspect available distribution identities locally:

```sh
security find-identity -v -p codesigning
```

The identity must be the product owner's **Developer ID Application** certificate, backed by its private key in Keychain. Do not substitute Apple Development, an ad-hoc signature, or another application's identity. For future updates, keep the same team and stable signing identifier and compare designated requirements before changing a live installation.

Prepare a notarization Keychain profile with Apple's supported interactive workflow or the organization's existing signing process. Reference the profile by name in commands. [Apple's custom notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow) describes Keychain profiles and receipt/log retrieval.

## Sign a staged copy

The example below uses the Apple Silicon archive. Change only the archive name for Intel. Start in the repository root, use a new staging directory, and verify the checksum against the candidate's recorded provenance first.

```sh
cd local/release-candidates
shasum -a 256 -c citta-0.1.0-aarch64-apple-darwin.tar.gz.sha256
mkdir signing-stage
tar -xzf citta-0.1.0-aarch64-apple-darwin.tar.gz -C signing-stage
cd signing-stage
export CITTA_BUNDLE="citta-0.1.0-aarch64-apple-darwin"
export CITTA_SIGNING_IDENTITY="Developer ID Application: YOUR NAME (TEAMID)"
codesign --force --sign "$CITTA_SIGNING_IDENTITY" --identifier io.citta.daemon \
  --options runtime --timestamp "$CITTA_BUNDLE/bin/citta"
codesign --verify --strict --verbose=2 "$CITTA_BUNDLE/bin/citta"
codesign -d -r- --verbose=4 "$CITTA_BUNDLE/bin/citta"
"$CITTA_BUNDLE/bin/citta" --version
```

Inspect that the authority, team, identifier and designated requirement are the intended ones. No additional entitlements are proposed by this procedure. If hardened runtime exposes a real compatibility problem, diagnose it before adding a targeted entitlement; do not disable hardened runtime or introduce blanket exceptions.

Update the manifest to match the signed bytes. This changes only the staged copy:

```sh
python3 - <<'PY'
import hashlib, json, os
from pathlib import Path
bundle = Path(os.environ['CITTA_BUNDLE'])
path = bundle / 'manifest.json'
manifest = json.loads(path.read_text())
assert manifest['source_dirty'] is False
manifest['files']['bin/citta'] = hashlib.sha256((bundle / 'bin/citta').read_bytes()).hexdigest()
manifest['signing'] = 'Developer ID Application signed; notarization acceptance recorded separately'
manifest['signing_identifier'] = 'io.citta.daemon'
# Device acceptance has not been implied by signing.
assert manifest['release_qualified'] is False
path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
PY
```

Run the bundled installer's archive verification in a clean test account or use its `verify()` function from the repository against the staged bundle. `--verify-only` also checks the installation destination, so it will deliberately refuse an account that already has Citta installed.

## Submit the final staged contents to Apple

The notary service takes ZIP, disk image or installer containers. Create a ZIP of the already-signed, manifest-verified bundle, then submit it using the Keychain profile. [Apple's packaging guide](https://developer.apple.com/documentation/xcode/packaging-mac-software-for-distribution) describes supported containers.

```sh
ditto -c -k --keepParent "$CITTA_BUNDLE" "$CITTA_BUNDLE-notarization.zip"
xcrun notarytool submit "$CITTA_BUNDLE-notarization.zip" \
  --keychain-profile citta-notary --wait --output-format json > notarization-result.json
```

Require `status: Accepted`, retain the submission ID, and retrieve the log for that ID:

```sh
xcrun notarytool log SUBMISSION_ID --keychain-profile citta-notary notarization-log.json
```

A timeout or `In Progress` is not acceptance. Inspect the existing submission instead of repeatedly uploading it. An invalid submission must be fixed before distribution. Keep notarization receipts with the release evidence. Do not modify the signed binary or staged bundle after acceptance.

A bare CLI executable cannot be treated like a stapled `.app` bundle. This archive route needs verification of Gatekeeper's online ticket lookup on a clean Mac; do not claim offline notarization availability. A future signed installer or disk image can provide a stapled distribution container, but that is a separate packaging path to implement and test.

## Produce the distribution archive and qualify it

After acceptance, the same staged bundle can be placed in the documented `.tar.gz` distribution. Keep it separate from the original unsigned candidate:

```sh
mkdir signed-output
tar -czf "signed-output/$CITTA_BUNDLE.tar.gz" "$CITTA_BUNDLE"
cd signed-output
shasum -a 256 "$CITTA_BUNDLE.tar.gz" > "$CITTA_BUNDLE.tar.gz.sha256"
```

The ZIP submission and tar archive must contain the same signed executable and manifest. Record both container hashes, the source commit, binary hash, signing identity, designated requirement, submission ID and acceptance status in the release record. Copy the staged manifest alongside the final archive as its public provenance record. Do not use the unsigned candidate's adjacent manifest.

Before publication, download the final archive through the intended browser path on a clean Mac, preserve its normal quarantine behavior, verify the checksum, install through the documented Python installer, run `citta setup`, and complete real delegated tasks in both directions. Verify the daemon's actual desktop permissions when desktop access is advertised. Recheck on an upgraded installation using the same signing identity. Repeat for each Mac architecture.

If Gatekeeper, installation, service startup, desktop grants, or device acceptance fails, the release remains unqualified. Do not remove quarantine recursively, disable system checks, or replace a running user's binary to make the test appear to pass. Follow the existing [offline upgrade procedure](operations.md#upgrade-and-rollback).

## Public distribution follow-up — November 12

All nine published assets passed anonymous download/hash/full-manifest verification. A fresh registry-only Cargo installation passed with no publishing credentials. A new independent Chrome profile downloaded the public ARM archive with its original quarantine; native extraction and isolated installation preserved it, and execution/setup-help/online-notarization checks passed. This used an existing Mini OS account, not a clean OS account or Intel browser, and created no service/configuration. See [public distribution verification](release-readiness.md#public-distribution-verification).
