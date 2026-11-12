# Publish Citta

Citta has two distribution paths: a Rust source crate for `cargo install`, and native binary archives for people who do not have Rust. Both must come from the same qualified source revision. This guide prepares those artifacts; it does not publish them automatically.

## Install from crates.io

For version 0.1.0 listed on [crates.io](https://crates.io/crates/citta), a fresh installation uses:

```sh
cargo install citta --version 0.1.0 --locked
citta setup
citta doctor
```

Use Rust 1.99.1 or later, a C compiler/linker, and the other prerequisites in the [installation guide](install.md). The manifest declares the qualified toolchain rather than an untested lower minimum. Cargo installs the executable to its bin directory, normally `~/.cargo/bin`. It does not copy the crate's documentation, examples or maintenance scripts into a shared data directory. Use the matching source archive for those files; Citta's runtime UI and setup skills are embedded in the binary.

For an existing installation, follow the [offline upgrade procedure](operations.md#upgrade-and-rollback). Do not run `cargo install --force` over a live worker.

## Review the source package

From the exact release checkout, run:

```sh
cargo package --list --locked
cargo publish --dry-run --locked
```

Cargo's dry run builds and verifies the package without uploading it. The resulting source archive is `target/package/citta-0.1.0.crate`. Review its contents and preserve its SHA-256 alongside the binary archive manifest. A development rehearsal can use `--allow-dirty`, but the final publish must use the committed, reviewed revision without that option. [Cargo publishing reference](https://doc.rust-lang.org/cargo/reference/publishing.html).

The manifest uses an explicit inclusion list. It contains the Rust sources, locked dependencies, embedded UI, all embedded setup/delegation/desktop skill files, integration tests and their public TLS fixtures, Python adapters, selected setup/operations documentation, and license notices. It excludes launch galleries, the demonstration video, local task records, credentials, build outputs and workflow files.

The private key in `tests/fixtures/enrollment-test-key.der` is deliberately public test material. It is not a device credential; retain the fixture README. Do not suppress other secret findings merely because this known fixture is present.

Check that every `include_str!` and `include_bytes!` path exists in the extracted package. Build and install from that extracted directory into a temporary prefix, then run the isolated binary's `--version` and `setup --help`. This verifies the published file set rather than relying on files left in the working tree.

## Before uploading

- [ ] CI and same-candidate physical-device acceptance are complete; review [release acceptance](acceptance.md).
- [ ] The package is from a clean committed revision and the version is still available on crates.io.
- [ ] All included files and git provenance have been reviewed for unintended private material.
- [ ] LICENSE and NOTICE identify the intended noncommercial terms; crate metadata agrees.
- [ ] The public repository and documentation links work without authentication. The crate README's relative documentation/media links must resolve to that public repository; media is intentionally absent from the `.crate` file.
- [ ] The crates.io account owns the package name or can create it, has a verified email address, and has suitable publishing credentials. A configured token alone does not prove these conditions.
- [ ] The dry run and isolated installation pass for the exact final file set.
- [ ] Binary archives have the same source revision and correct supported-platform provenance.

The `citta` name returned “does not exist” from the registry API during the November 10, 2026 preparation check. This is an observation, not a reservation; repeat the check immediately before publication.

## Publish the qualified revision

Only after release qualification and the publication decision, run:

```sh
cargo publish --locked
```

Let Cargo use its configured credential provider. Never paste tokens into documentation, command arguments or logs. Publication is generally permanent: versions cannot be overwritten, and yanking is not deletion. [Registry publication behavior](https://doc.rust-lang.org/cargo/reference/publishing.html).

After the registry confirms publication, test `cargo install citta --version 0.1.0 --locked --root /absolute/path/to/disposable-prefix` from a separate directory. This last test must use the registry, not the local checkout. Record the result, check the public README rendering and download links, and then update the release status in user-facing documentation.
