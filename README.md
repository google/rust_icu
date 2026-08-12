# rust_icu: low-level rust language bindings for the ICU library

Item           | Description
-------------- | -----------
Testing        | [![Test status](https://github.com/google/rust_icu/workflows/Test/badge.svg)](https://github.com/google/rust_icu/workflows/Test/badge.svg)
Source         | https://github.com/google/rust_icu
README         | https://github.com/google/rust_icu/blob/main/README.md
Coverage       | [View report](/coverage/report.md)
Docs           | https://docs.rs/crate/rust_icu

![Quintus Junius Rusticus from Crabb's Historical Dictionary (lived c. 100-c. 170 AD), consul in 133 (consul suffectus) and 162 (consul ordinarius) - https://commons.wikimedia.org/wiki/File:Quintus_Junius_Rusticus_from_Crabb%27s_Historical_Dictionary.jpg#filelinks](docs/204px-Quintus_Junius_Rusticus_from_Crabbs_Historical_Dictionary.jpg)

*Project mascot: Quintus Junius Rusticus from Crabb's Historical Dictionary
(lived c. 100-c. 170 AD), consul in 133 (consul suffectus) and 162 (consul
ordinarius). Source: [Wikimedia Commons][wmc]*

[wmc]: https://commons.wikimedia.org/wiki/File:Quintus_Junius_Rusticus_from_Crabb%27s_Historical_Dictionary.jpg#filelinks

This is a library of low level native rust language bindings for the
International Components for Unicode (ICU) library for C (a.k.a. ICU4C).

If you just want quick instructions on how to download and install, see the
[quickstart guide][qsg]

[qsg]: #quickstart-guide

See the [ICU project home page][ipr] for details about the ICU library. The
library source can be [viewed on Github][uoi].

[ipr]: https://icu-project.org
[uoi]: https://github.com/unicode-org/icu

The latest version of this file is available at
https://github.com/google/rust_icu.

> This is not an officially supported Google product.

## Why wrap ICU (vs. doing anything else)?

*   The rust language
    [Internationalisation](https://www.arewewebyet.org/topics/i18n/) page
    confirms that ICU support in rust is spotty, so having a functional wrapper
    helps advance the state of the art.

*   Projects such as [Fuchsia](https://fuchsia.dev) already depend on ICU,
    and having rust bindings allows for an easy way to use Unicode algorithms
    without taking on more dependencies.

*   Cooperation on the interface with projects such as the
    [ICU4X](https://github.com/unicode-org/icu4x) could allow
    seamless transition to an all-rust implementation in the future.

# Structure of the repository

The repository is organized as a cargo workspace of rust crates. Each crate
corresponds to the respective header in the ICU4C library's C API. Please
consult the [coverage report](/coverage/report.md) for details about function
coverage in the headers.

Crate                                                                           | Description
------------------------------------------------------------------------------- | -----------
[rust_icu](https://crates.io/crates/rust_icu)                                   | Top-level crate. Include this if you just want to have all the functionality available for use.
[rust_icu_common](https://crates.io/crates/rust_icu_common)                     | Commonly used low-level wrappings of the bindings.
[rust_icu_intl](https://crates.io/crates/rust_icu_intl)                         | Implements ECMA 402 recommendation APIs.
[rust_icu_sys](https://crates.io/crates/rust_icu_sys)                           | Low-level bindings code
[rust_icu_ubrk](https://crates.io/crates/rust_icu_ubrk)                         | Support for text boundary analysis. Implements [`ubrk.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/ubrk_8h.html) C API header from the ICU library.
[rust_icu_ucal](https://crates.io/crates/rust_icu_ucal)                         | ICU Calendar. Implements [`ucal.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/ucal_8h.html) C API header from the ICU library.
[rust_icu_ucol](https://crates.io/crates/rust_icu_ucol)                         | Collation support. Implements [`ucol.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/ucol_8h.html) C API header from the ICU library.
[rust_icu_udat](https://crates.io/crates/rust_icu_udat)                         | ICU date and time. Implements [`udat.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/udat_8h.html) C API header from the ICU library.
[rust_icu_udata](https://crates.io/crates/rust_icu_udata)                       | ICU binary data. Implements [`udata.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/udata_8h.html) C API header from the ICU library.
[rust_icu_uenum](https://crates.io/crates/rust_icu_uenum)                       | ICU enumerations. Implements [`uenum.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/uenum_8h.html) C API header from the ICU library. Mainly `UEnumeration` and friends.
[rust_icu_uformattable](https://crates.io/crates/rust_icu_uformattable)         | Locale-sensitive list formatting support. Implements [`uformattable.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/uformattable_8h.html) C API header from the ICU library. Since 0.3.1.
[rust_icu_ulistformatter](https://crates.io/crates/rust_icu_ulistformatter)     | Locale-sensitive list formatting support. Implements [`ulistformatter.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/ulistformatter_8h.html) C API header from the ICU library.
[rust_icu_uloc](https://crates.io/crates/rust_icu_uloc)                         | Locale support. Implements [`uloc.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/uloc_8h.html) C API header from the ICU library.
[rust_icu_umsg](https://crates.io/crates/rust_icu_umsg)                         | MessageFormat support. Implements [`umsg.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/umsg_8h.html) C API header from the ICU library.
[rust_icu_unorm2](https://crates.io/crates/rust_icu_unorm2)                     | Unicode normalization support. Implements [`unorm2.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/unorm2_8h.html) C API header from the ICU library.
[rust_icu_unum](https://crates.io/crates/rust_icu_unum)                         | Number formatting support. Implements [`unum.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/unum_8h.html) C API header from the ICU library.
[rust_icu_unumberformatter](https://crates.io/crates/rust_icu_unumberformatter) | Number formatting support (modern). Implements [`unumberformatter.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/unumberformatter_8h.html) C API header from the ICU library.
[rust_icu_upluralrules](https://crates.io/crates/rust_icu_upluralrules)         | Locale-sensitive plural rules support. Implements [`upluralrules.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/upluralrules_8h.html) C API header from the ICU library.
[rust_icu_ures](https://crates.io/crates/rust_icu_ures)                         | Resource bundle support. Implements [`ures.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/ures_8h.html) C API header from the ICU library.
[rust_icu_ustring](https://crates.io/crates/rust_icu_ustring)                   | ICU strings. Implements [`ustring.h`]() C API header from the ICU library.
[rust_icu_utext](https://crates.io/crates/rust_icu_utext)                       | Text operations. Implements [`utext.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/utext_8h.html) C API header from the ICU library.
[rust_icu_utrans](https://crates.io/crates/rust_icu_utrans)                     | Transliteration support. Implements [`utrans.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/utrans_8h.html) C API header from the ICU library.

# Limitations

The generated rust language binding methods of today limit the availability of
language bindings to the available C API. The ICU library's C API (sometimes
referred to as ICU4C in the documentation) is distinct from the ICU C++ API.

The bindings offered by this library have somewhat limited applicability, which
means it may sometimes not work for you out of the box. If you come across such
a case, feel free to [file a bug](https://github.com/google/rust_icu/issues) for
us to fix. [Pull requests](https://github.com/google/rust_icu/pulls) are
welcome.

The limitations we know of today are as follows:

*   *There isn't a guaranted feature parity.* Some algorithms that are
    implemented in C++ don't have a C equivalent, and vice-versa. This is
    usually not a problem if you are using the library from C++, since you are
    free to choose whichever API surface works for you. But it is an issue for
    rust bindings, since we can only use the C API at the moment.

*   *A C++ implementation of a new algorithm is not necessarily always reflected
    in the C API*, leading to feature disparity between the C and C++ API
    surfaces. See for example
    [this bug](https://unicode-org.atlassian.net/browse/ICU-20931) as an
    illustration.

*   While using `icu_config` feature will likely allow you some freedom to
    auto-generate bindings for your own library version, we still need to keep a
    list of explicitly supported ICU versions to ensure that the wrappers are
    stable.

# Compatibility

Automated tests are executed for last three major ICU library versions in all
feature combinations of interest.

`rust_icu` version   | ICU 74.1 | ICU 75.0 | ICU 76.0 | ICU 77.0 |
-------------------- | -------- | -------- | -------- | -------- |
5.x                  |    ✅    |          |    ✅    |    ✅    |

# Features

The `rust_icu` library is intended to be compiled with `cargo`, with one of
several features enabled. Compilation with `cargo` allows us to do some library
detection in a custom `build.rs` file in the `rust_icu_sys` library and adapt
the build process to your build environment. However, since not every
development environment will use the same settings, we opted to offer certain
features (below) as configuration options.

While our intention is to keep the list of features below up to date with the
[actual list in `Cargo.toml`](https://github.com/google/rust_icu/blob/main/Cargo.toml),
the list may periodically go out of date.

To use any of the features, you will need to activate the feature in *all* the
`rust_icu_*` crates that you intend to use. Failing to do this will result in
confusing compilation end result.

Feature              | Default? | Description
-------------------- | -------- | -----------
`use-bindgen`        | Yes      | If set, cargo will run `bindgen` to generate bindings based on the installed ICU library. The program `icu-config` must be in $PATH for this to work. In the future there may be other approaches for auto-detecting libraries, such as via `pkg-config`.
`renaming`           | Yes      | If set, ICU bindings are generated with version numbers appended. This is called "renaming" in ICU, and is normally needed only when linking against specific ICU version is required, for example to work around having to link different ICU versions. See [the ICU documentation](https://unicode-org.github.io/icu/userguide/icu/design.html) for a discussion of renaming. **This feature MUST be used when `bindgen` is NOT used.**
`icu_config`         | Yes      | If set, the binary icu-config will be used to configure the library. Turn this feature off if you do not want `build.rs` to try to autodetect the build environment. You will want to skip this feature if your build environment configures ICU in a different way. **This feature is only meaningful when `bindgen` feature is used; otherwise it has no effect.**
`icu_version_in_env` | No       | If set, ICU bindings are made for the ICU version specified in the environment variable `RUST_ICU_MAJOR_VERSION_NUMBER`, which is made available to cargo at build time. See section below for details on how to use this feature. **This feature is only meaningful when `bindgen` feature is NOT used; otherwise it has no effect.**
`static`             | No       | If set, link ICU libraries statically (and the standard C++ dynamically). You can use `RUST_ICU_LINK_SEARCH_DIR` to add an extra path to the search path if you have a build of ICU in a non-standard directory.

# Prior art

There is plenty of prior art that has been considered:

*   https://github.com/servo/rust-icu
*   https://github.com/open-i18n/rust-unic
*   https://github.com/fullcontact/icu-sys
*   https://github.com/rust-locale
*   https://github.com/unicode-rs

The current state of things is that I'd like to do a few experiments on my own
first, then see if the work can be folded into any of the above efforts.

See also:

*   https://github.com/rust-lang/rfcs/issues/797
*   https://unicode-rs.github.io
*   https://github.com/i18n-concept/rust-discuss

# Assumptions

There are a few competing approaches for ICU bindings. However, it seems, at
least based on
[information available in rust's RFC repos](https://github.com/rust-lang/rfcs/issues/797),
that the work on ICU support in rust is still ongoing.

These are the assumptions made in the making of this library:

*   We need a complete, reusable and painless ICU low-level library for rust.

    This, for example, means that we must rely on an external ICU library, and
    not lug the library itself with the binding code. Such modularity allows the
    end user of the library to use an ICU library of their choice, and
    incorporate it in their respective systems.

*   No ICU algorithms will be reimplemented as part of the work on this library.

    An ICU reimplementation will likely take thousands of engineer years to
    complete. For an API that is as subtle and complex as ICU, I think that it
    is probably a better return on investment to maintain a single central
    implementation.

    Also, the existence of this library doesn't prevent reimplementation. If
    someone else wants to try their hand at reimplementing ICU, that's fine too.

*   This library should serve as a low-level basis for a rust implementation.

    A low level ICU API may not be an appropriate seam for the end users. A
    rust-ful API should be layered on top of these bindings. It will probably be
    a good idea to subdivide that functionality into crates, to match the
    expectations of rust developers.

    I'll gladly reuse the logical subdivision already made in some of the above
    mentioned projects.

*   I'd like to explore ways to combine with existing implementations to build a
    complete ICU support for rust.

    Hopefully it will be possible to combine the good parts of all the rust
    bindings available today into a unified rust library. I am always available
    to discuss options.

    The only reason I started a separate effort instead of contributing to any
    of the projects listed in the "Prior Art" section is that I wanted to try
    what a generated library would look like in rust.


# Building and Formatting with Bazel

`rust_icu` natively utilizes **Bazel** as its primary compilation engine, ensuring determinism, automated dependencies, and scalable C++ & Rust cross-compilation boundaries.

## Prerequisites

Before compiling the crate tree, you must have the Bazel wrapper, **Bazelisk**, natively installed on your system. Bazelisk automatically provisions and updates the Bazel version matching the workspace requirements.

* **Git**: Needed to clone the workspace.
* **Bazelisk**: Ensure `bazelisk` is downloaded and available on your system path.
    * *To install `bazelisk`:* You can find pre-compiled binaries [here via GitHub releases](https://github.com/bazelbuild/bazelisk#installation) or use standard package managers like Homebrew (`brew install bazelisk`) or NPM (`npm install -g @bazel/bazelisk`). 

## Quickstart Guide

The following sequence checks out the repository natively and verifies the compilation environments securely across all core ICU targets:

```bash
# 1. Clone the repository
git clone https://github.com/google/rust_icu.git
cd rust_icu

# 2. Compile every active Rust target
bazel build //...

# 3. Validatively test the cross-compiled C++ C-API links sequentially
# (Note: RUST_TEST_THREADS=1 natively avoids concurrent multithreaded ICU timezone mutation panics)
bazel test --test_env=RUST_TEST_THREADS=1 //...
```

## Matrix Testing (Advanced ICU Targets)

This Bazel workspace is automatically equipped with predefined labels allowing you to efficiently bind tests aggressively against specific ICU release matrices internally!

To execute natively against exact versions of the ICU backend rather than the default bindings, append the `--config` parameter targeting an active module extension:

```bash
# Cross-compiles test matrix exclusively utilizing ICU 75 backend bindings natively
bazel test --test_env=RUST_TEST_THREADS=1 //... --config=icu_75

# Validates Top of Tree (ToT) iCloud source bindings mapped directly to ICU upstream main!
bazel test --test_env=RUST_TEST_THREADS=1 //... --config=icu_tot
```

Currently active backend mappings are:
* `icu_74` (Default)
* `icu_75`
* `icu_76`
* `icu_77`
* `icu_tot` (Bleeding edge directly pulling `main.zip` from iCloud upstream `master` tags)

> **Note on legacy compilation:** `rust_icu` is currently migrating to native Bazel environments directly! Some legacy Rust compilation features and non-Bazel vestiges (e.g. `icu-config`, manually passing `bindgen` flags to `cargo`) securely exist purely as backward-compatible shims, but will be scheduled for total deprecation and removal once we prove out robust Bazel-powered automated releases natively to `crates.io`.
