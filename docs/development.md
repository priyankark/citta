# Build and contribute

Citta is a Rust runtime for task delegation across trusted devices. Start with the [README](../README.md) for the product and [protocol](protocol.md) for the implementation contract. Read [AGENTS.md](../AGENTS.md) before changing the code.

## Build

The CI toolchain is Rust 1.99.1. Install a C compiler for bundled SQLite/TLS and Python 3.11+ for the helper suites. Node 22 runs the desktop adapter tests. macOS helper compilation also needs Swift. The package requires Rust 1.99.1, matching the qualified CI toolchain.

```sh
git clone https://github.com/priyankark/citta.git
cd citta
cargo build --release --locked
```

Repository access is currently required while the public release is being prepared. For a new installation from this checkout:

```sh
cargo install --path . --locked
citta setup
citta doctor
```

Do not overwrite a running installation with `cargo install`. Use the [offline upgrade procedure](operations.md#upgrade-and-rollback).

## Verify changes

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 scripts/test_mesh.py
python3 scripts/test_helpers.py
python3 scripts/test_upgrade.py
python3 scripts/test_setup.py
python3 scripts/test_ci_health.py
node skills/citta-desktop/scripts/test-control.cjs
python3 skills/citta-desktop/scripts/test-session.py
cargo audit
```

The mesh suite launches independent daemon processes and exercises capacity, cancellation, restart, task expiry, disconnected peers, and worker death. It does not require account logins or models. The setup suite uses isolated fixtures; it does not reconfigure your normal installation.

Refresh CI with `python3 scripts/ci_health.py`; inspect `local/ci-history.json` and [build health](build-health.md). Preserve failed attempts and distinguish source failures from unavailable runners or account limits. A local pass does not establish another platform passed CI.

## Make a useful contribution

Describe the problem, the expected behavior, and how you checked it. Keep changes scoped. Include a regression check for a behavior bug when practical; document prerequisites for anything requiring an account or physical device. Keep secrets, private task results, and device credentials out of commits and issues.

Citta uses [PolyForm Noncommercial 1.0.0](../LICENSE.md). Changes must preserve required notices and third-party license obligations. Release candidates must complete [physical device acceptance](acceptance.md) before a production tag.
