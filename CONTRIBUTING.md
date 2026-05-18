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

## Adding ICU-version-dependent code

The workspace currently targets ICU versions for which all wrapped APIs are
uniformly available, so no version-gated code remains. If a future change
introduces functionality that exists only in newer (or only in older) ICU
versions, follow the pattern below — it must be wired up in **two** places
that are kept in sync.

### 1. The Cargo feature

Declare a feature in each affected crate's `Cargo.toml`, named after the
boundary version (e.g. `icu_version_80_plus`). Propagate it to dependent
crates the same way `icu_version_in_env` is propagated.

### 2. Automatic activation via `build.rs`

A crate's `build.rs` may emit `cargo:rustc-cfg=feature="icu_version_XX_plus"`
to activate the feature based on the ICU version detected at build time (via
`pkg-config`). This makes `#[cfg(feature = "icu_version_XX_plus")]`-gated
code light up automatically under the default feature set
(`use-bindgen`, `icu_config`, `renaming`) without requiring users to pass
`--features` manually.

**Any crate that contains `#[cfg(feature = "icu_version_XX_plus")]`-gated
code must have a `build.rs`** that emits the corresponding cfg. Without it,
the gated code is silently excluded when building with default features,
even if a matching ICU version is installed. Copy the build script from any
crate that has one (e.g. `rust_icu_uloc`) and add the matching
`[build-dependencies]` entry pointing at `rust_icu_release`.

### Explicit activation (when `icu_config` is inactive)

When `icu_config` is disabled (e.g. when using pre-generated static bindgen
files with `icu_version_in_env`), the `build.rs` does not emit cfgs, so
version features must be passed explicitly via
`--features=icu_version_XX_plus,...`. The CI `test-with-features` matrix
job is the right place to add coverage for that path.

## Community Guidelines

This project follows [Google's Open Source Community
Guidelines](https://opensource.google/conduct/).
