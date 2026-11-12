# Install Citta

Build your agent network by installing Citta on **each computer that will take part**. Your computers should be signed in to Tailscale with the same owner account. You do not need SSH, copied tokens, or a new Citta account.

Choose a native archive from [Citta 0.1.0](https://github.com/priyankark/citta/releases/tag/v0.1.0), or use the [source installation](#build-from-source) below. Each release records its tested platforms and limitations. Development workflow artifacts are separate from published releases.

## What you need

- A Mac with Apple Silicon or Intel, or Linux x86-64. macOS archives target macOS 14 or later; that deployment target is not yet a verified minimum OS support claim. The published Linux archive is built on Ubuntu 24.04 and requires its glibc 2.39 baseline or newer; other distributions need compatibility testing or a source build.
- [Tailscale](https://tailscale.com/download), connected on each device. Use the same owner account; tagged or other users' devices are excluded from automatic enrollment.
- [Python 3.11 or later](https://www.python.org/downloads/) for the archive installer and example adapters. Check with `python3 --version` in the terminal you will use for installation.
- For delegated agent work: a supported agent harness installed and signed in on the receiving computer. Setup currently configures the [Codex CLI](https://learn.chatgpt.com/docs/codex/cli) automatically. Check `codex --version`, run `codex` once and finish sign-in before running Citta setup. The receiving agent uses the tools and app connections installed there.

An installed archive needs no Rust compiler. macOS desktop control additionally needs the developer tools used by the desktop helper; see the [access guide](setup.md#choose-what-this-device-should-do). You can use Citta for command and model tasks without desktop control.

## Install a published archive

Open [Citta releases](https://github.com/priyankark/citta/releases). Choose the archive and the adjacent `.sha256` file for your device:

| Platform | 0.1.0 archive | Checksum |
| --- | --- | --- |
| Mac with Apple Silicon | [Download](https://github.com/priyankark/citta/releases/download/v0.1.0/citta-0.1.0-aarch64-apple-darwin.tar.gz) | [SHA-256](https://github.com/priyankark/citta/releases/download/v0.1.0/citta-0.1.0-aarch64-apple-darwin.tar.gz.sha256) |
| Mac with Intel processor | [Download](https://github.com/priyankark/citta/releases/download/v0.1.0/citta-0.1.0-x86_64-apple-darwin.tar.gz) | [SHA-256](https://github.com/priyankark/citta/releases/download/v0.1.0/citta-0.1.0-x86_64-apple-darwin.tar.gz.sha256) |
| Linux x86-64 | [Download](https://github.com/priyankark/citta/releases/download/v0.1.0/citta-0.1.0-x86_64-unknown-linux-gnu.tar.gz) | [SHA-256](https://github.com/priyankark/citta/releases/download/v0.1.0/citta-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256) |

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

The release notes identify the signing and notarization checks for each Mac archive. If macOS blocks a downloaded executable, keep the warning and verify that you downloaded the correct release; see [macOS signing and notarization](signing.md). Do not remove quarantine attributes or disable Gatekeeper to force it to run. A source build or development workflow artifact does not inherit a published binary's signing identity.

## Connect your computers

With Tailscale connected, run from the installed location on each computer:

```sh
citta setup
citta doctor
```

Setup creates a background service, configures private Tailscale HTTPS, installs Citta's skills, and connects the local Codex CLI when available. With a signed-in Codex CLI, it creates a full-tool profile: delegated work can run shell commands, edit files, and use the network as your user account. Read the setup permission summary. On a fresh installation, choose `citta setup --agent-access read-only` for a restricted worker. On an existing installation, this adds a read-only profile while preserving existing grants; use the panel's **Permissions** view to disable a previously allowed full-tool profile.

Start a new agent session after setup so it loads the new tools. Ask:

> Use Citta to find my other computer and run a small test there. Show me which computer ran it and the result.

A successful task whose worker is the other device proves the connection. A device appearing in discovery alone does not prove execution. See [device setup](setup.md) for desktop permissions, connected apps, custom harnesses and troubleshooting.

## Build from source

[Citta 0.1.0 is available on crates.io](https://crates.io/crates/citta/0.1.0). A fresh installation can use `cargo install citta --version 0.1.0 --locked` instead of cloning the repository. This path requires Rust 1.99.1 or later and a C compiler/linker. Cargo installs the binary and its embedded UI/skills; download the matching source archive if you need the example adapters or maintenance scripts. See [crate installation and publication](publishing.md).

For development, install Rust 1.99.1, a C compiler, and Python 3.11+. On macOS, Xcode Command Line Tools provide the C compiler. On Linux, install your distribution's C compiler and linker. Check free disk space before compiling.

```sh
git clone https://github.com/priyankark/citta.git
cd citta
cargo +1.99.1 install --path . --locked
citta setup
citta doctor
```

Cargo normally installs to `~/.cargo/bin`; follow its PATH guidance if `citta` is not found. Keep the checkout for examples and operations scripts. For an existing installation, use the upgrade procedure below instead of replacing a running binary with `cargo install --force`.

## Troubleshooting

Run `citta doctor` on both devices first.

| What you see | Next step |
| --- | --- |
| `citta` or `python3` not found | Check the Python prerequisite and PATH instructions above. |
| Tailscale HTTPS fails during setup | Follow the [Tailscale HTTPS prerequisite and recovery steps](setup.md). Enable the required tailnet settings, then rerun setup with the same config. |
| No agent profile or sign-in required | Confirm `codex --version` works and finish `codex` sign-in on the receiving device, then rerun `citta setup`. |
| Other device missing | Confirm both are awake, connected to Tailscale under the same owner, and running Citta. Check `citta doctor` on each. |
| Task queued | Check the receiving device's Permissions, available resources and worker slots in `citta panel`. |
| Desktop task denied | Follow the [receiving-device access checks](setup.md#choose-what-this-device-should-do). Coding can work while desktop permissions are pending. |
| Direct `localhost:7331` page does not open the panel | Run `citta panel` and use the private link it prints. |

Inspect failed or interrupted tasks before repeating them; they may already have changed a file or acted in an app. Include the error and task status when [reporting an issue](https://github.com/priyankark/citta/issues), but remove tokens, private paths and task contents you do not want to share.

## Upgrades and removal

The fresh installer refuses existing binaries or Citta configuration. Follow [upgrade and rollback](operations.md#upgrade-and-rollback) to drain work, stop the service, back up state, and replace the executable safely. `scripts/upgrade_device.py` is included in the archive's installed assets; pass the actual installed path, such as `~/.local/bin/citta`.

To stop using Citta, first stop its user service and disconnect only Citta's Tailscale Serve port as described in [roll back enrollment](setup.md#roll-back-enrollment). Preserve task history and backups until you no longer need them. Removing a binary alone does not remove pairing or stop a running service.

## Build a release candidate

Maintainers can run the manual **Release candidate** workflow for native Mac ARM, Mac Intel and Linux x86-64 archives. It uploads workflow artifacts; it does not create a GitHub release, change repository visibility or publish a tag.

From a clean, reviewed checkout:

```sh
python3 scripts/test_release.py
python3 scripts/package_release.py
python3 scripts/check_release_archive.py local/release-candidates/citta-0.1.0-aarch64-apple-darwin.tar.gz
```

Use the archive name for your native target in the last command. It tests installation in a temporary home, checks that the installed executable runs, and proves that a second install is refused. It does not start services or alter your existing Citta installation.

Each archive contains the binary, source commit and build provenance, per-file hashes, setup/delegation/desktop skills, documentation, example adapters, upgrade scripts, Citta's license and required attribution, and collected third-party license texts with a dependency inventory. The packaging script refuses missing third-party notices and existing output files. `--allow-dirty` is available only for an explicitly marked development candidate; do not publish such a build.

Archives are written under `local/release-candidates/`. A successful package build does not qualify a release: CI, stable macOS signing/notarization, public-source review, clean-machine installation and same-candidate physical-device acceptance must pass before publication. See [release acceptance](acceptance.md).
