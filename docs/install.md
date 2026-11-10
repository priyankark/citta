# Install Citta

Install Citta once on **each computer you want your agent to use**. Your computers should be signed in to Tailscale with the same owner account. You do not need SSH, copied tokens, or a new Citta account.

**Release status:** a public binary release has not been published yet. The archive instructions below describe the prepared release path; use the source instructions today if you have repository access. Candidate artifacts are for testing and are not signed or notarized macOS distributions.

## What you need

- A Mac with Apple Silicon or Intel, or Linux x86-64. macOS archives target macOS 14 or later; that deployment target is not yet a verified minimum OS support claim. The prepared Linux CI archive is built on Ubuntu 24.04 and requires its glibc 2.39 baseline or newer; other distributions need compatibility testing or a source build.
- [Tailscale](https://tailscale.com/download), connected on each device. Use the same owner account; tagged or other users' devices are excluded from automatic enrollment.
- Python 3.11 or later for the archive installer and example adapters.
- For delegated agent work: a supported agent harness installed and signed in on the receiving computer. Setup currently configures Codex automatically. The receiving agent uses the tools and app connections installed there.

An installed archive needs no Rust compiler. macOS desktop control additionally needs the developer tools used by the desktop helper; see the [access guide](setup.md#choose-what-this-device-should-do). You can use Citta for command and model tasks without desktop control.

## Install a published archive

When a release is available, open [Citta releases](https://github.com/priyankark/citta/releases). Choose the archive and the adjacent `.sha256` file for your device:

| Computer | Archive suffix |
| --- | --- |
| Mac with Apple Silicon | `aarch64-apple-darwin.tar.gz` |
| Mac with Intel processor | `x86_64-apple-darwin.tar.gz` |
| Linux x86-64 | `x86_64-unknown-linux-gnu.tar.gz` |

Download both files from that release into the same folder. On a Mac, verify the download before extracting it. For an Apple Silicon `0.1.0` archive:

```sh
cd "$HOME/Downloads"
shasum -a 256 -c citta-0.1.0-aarch64-apple-darwin.tar.gz.sha256
tar -xzf citta-0.1.0-aarch64-apple-darwin.tar.gz
python3 citta-0.1.0-aarch64-apple-darwin/install.py
```

Use the matching filename for Intel or Linux; on Linux use `sha256sum -c` for the first check. Continue only when the checksum reports `OK`. Checksums detect changes during download; download the checksum and archive from the authentic release page.

The installer verifies every bundled file, checks the device architecture, and installs to `~/.local/bin/citta`. Documentation, examples, skills and license notices go to `~/.local/share/citta/0.1.0`. It does not start Citta, change your shell settings, or replace existing installations. Run it as your normal user without `sudo`. `--verify-only` checks the archive and installation destination without copying files.

If `citta` is not found, add this directory to your shell's PATH:

```sh
export PATH="$HOME/.local/bin:$PATH"
citta --version
```

To keep that PATH setting in new terminals, add the export line once to your shell configuration (`~/.zshrc` on a typical Mac, or `~/.bashrc` for Bash).

If macOS blocks an unsigned candidate, record the warning and follow the OS's per-app approval flow only after verifying its origin. Candidate testing is not evidence of a signed, notarized release. Do not disable Gatekeeper globally. Stable signing identity and notarization remain release requirements; maintainers should follow [macOS signing and notarization](signing.md).

## Connect your computers

With Tailscale connected, run from the installed location on each computer:

```sh
citta setup
citta doctor
```

Setup creates a background service, configures private Tailscale HTTPS, installs Citta's skills, and connects the local Codex CLI when available. With a signed-in Codex CLI, it creates a full-tool profile: delegated work can run shell commands, edit files, and use the network as your user account. Read the setup permission summary. Use `citta setup --agent-access read-only` if you deliberately want a restricted worker instead.

Start a new agent session after setup so it loads the new tools. Ask:

> Use Citta to find my other computer and run a small test there. Show me which computer ran it and the result.

A successful task whose worker is the other device proves the connection. A device appearing in discovery alone does not prove execution. See [device setup](setup.md) for desktop permissions, connected apps, custom harnesses and troubleshooting.

## Build from source today

Once the first crate is published, a fresh installation can use `cargo install citta --version 0.1.0 --locked` instead of cloning the repository. This path requires Rust 1.99.1 or later and a C compiler/linker. The crate is not available yet; see [crate installation and publication](publishing.md) for its status and the checks required before release. Cargo installs the binary and its embedded UI/skills; download the matching source archive if you need the example adapters or maintenance scripts.

For development, install Rust 1.99.1, a C compiler, and Python 3.11+. On macOS, Xcode Command Line Tools provide the C compiler. On Linux, install your distribution's C compiler and linker. Check free disk space before compiling.

```sh
git clone https://github.com/priyankark/citta.git
cd citta
cargo +1.99.1 install --path . --locked
citta setup
citta doctor
```

The repository currently requires authorized access. A public launch must remove that prerequisite. Cargo normally installs to `~/.cargo/bin`; follow its PATH guidance if `citta` is not found. Keep the checkout for examples and operations scripts. For an existing installation, use the upgrade procedure below instead of replacing a running binary with `cargo install --force`.

## Upgrades and removal

The fresh installer refuses existing binaries or Citta configuration. Follow [upgrade and rollback](operations.md#upgrade-and-rollback) to drain work, stop the service, back up state, and replace the executable safely. `scripts/upgrade_device.py` is included in the archive's installed assets; pass the actual installed path, such as `~/.local/bin/citta`.

To stop using Citta, first stop its user service and disconnect only Citta's Tailscale Serve port as described in [roll back enrollment](setup.md#roll-back-enrollment). Preserve task history and backups until you no longer need them. Removing a binary alone does not remove pairing or stop a running service.

## Build a release candidate

Maintainers can run the manual **Release candidate** workflow for native Mac ARM, Mac Intel and Linux x86-64 archives. It only uploads private workflow artifacts; it does not create a GitHub release, change repository visibility or publish a tag.

From a clean, reviewed checkout:

```sh
python3 scripts/test_release.py
python3 scripts/package_release.py
python3 scripts/check_release_archive.py local/release-candidates/citta-0.1.0-aarch64-apple-darwin.tar.gz
```

Use the archive name for your native target in the last command. It tests installation in a temporary home, checks that the installed executable runs, and proves that a second install is refused. It does not start services or alter your existing Citta installation.

Each archive contains the binary, source commit and build provenance, per-file hashes, setup/delegation/desktop skills, documentation, example adapters, upgrade scripts, Citta's license and required attribution, and collected third-party license texts with a dependency inventory. The packaging script refuses missing third-party notices and existing output files. `--allow-dirty` is available only for an explicitly marked development candidate; do not publish such a build.

Archives are written under `local/release-candidates/`. A successful package build does not qualify a release: CI, stable macOS signing/notarization, public-source review, clean-machine installation and same-candidate physical-device acceptance must pass before publication. See [release acceptance](acceptance.md).
