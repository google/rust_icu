# How to Contribute

We'd love to accept your patches and contributions to this project. There are
just a few small guidelines you need to follow.

## Contributor License Agreement

Contributions to this project must be accompanied by a Contributor License
Agreement. You (or your employer) retain the copyright to your contribution;
this simply gives us permission to use and redistribute your contributions as
part of the project. Head over to <https://cla.developers.google.com/> to see
your current agreements on file or to sign a new one.

You generally only need to submit a CLA once, so if you've already submitted one
(even if it was for a different project), you probably don't need to do it
again.

## Code reviews

All submissions, including submissions by project members, require review. We
use GitHub pull requests for this purpose. Consult
[GitHub Help](https://help.github.com/articles/about-pull-requests/) for more
information on using pull requests.

### Contribution Guidelines

The advice here is intended to make contributions easier to review and merge.

It is maintained on an as-needed basis, with the intention to keep the long 
term health of the repository.

* Please keep PRs limited to a single topic of change. This makes the PR easier to
  review, and easier to roll back, if that becomes necessary.

## ICU version feature detection

Several crates in this workspace contain code that is conditionally compiled
based on the ICU library version, using attributes such as
`#[cfg(feature = "icu_version_64_plus")]`.

**There are two mechanisms by which these features are activated, and both
must be kept in sync:**

### 1. Automatic detection via `build.rs` (used when `icu_config` is active)

Crates that have a `build.rs` call `rust_icu_release::run()`, which queries
`pkg-config` for the installed ICU version and emits
`cargo:rustc-cfg=feature="icu_version_XX_plus"` instructions. This activates
the version features automatically when using the default feature set
(`use-bindgen`, `icu_config`, `renaming`), so `cargo test` works without
any explicit `--features` flags.

**Every crate that contains `#[cfg(feature = "icu_version_XX_plus")]`-gated
code MUST have a `build.rs`** that follows this pattern. Without it, the
version-gated code is silently excluded from compilation and testing when
the crate is built with default features, even if a modern ICU version is
installed.

Crates that currently have a `build.rs` for this purpose:
`rust_icu_sys`, `rust_icu_udat`, `rust_icu_uloc`, `rust_icu_ulistformatter`,
`rust_icu_unumberformatter`, `rust_icu_ecma402`.

### 2. Explicit Cargo features (used when `icu_config` is inactive)

When `icu_config` is disabled (e.g. when using pre-generated static bindgen
files with `icu_version_in_env`), the version features must be passed
explicitly via `--features=icu_version_64_plus,...`. The `build.rs` no-ops in
this case. The CI `test-with-features` matrix job covers this path.

### Adding a new crate with version-gated code

If you add `#[cfg(feature = "icu_version_XX_plus")]` to a new crate, you must
also add a `build.rs` (copy from any existing crate such as `rust_icu_uloc`)
and add the corresponding `[build-dependencies]` to the crate's `Cargo.toml`.

## Community Guidelines

This project follows [Google's Open Source Community
Guidelines](https://opensource.google/conduct/).
