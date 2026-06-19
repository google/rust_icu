# `rust_icu` Hello, World!

A minimal, runnable example of locale-aware
[MessageFormat](http://userguide.icu-project.org/formatparse/messages) string
formatting with `rust_icu`.  `MessageFormat` is a pattern-based template
language built into ICU that lets you embed formatted numbers, dates, and
plurals directly inside a message string.  It is *not* XLIFF.

## Run it

You need a working ICU installation, the same as for the rest of `rust_icu`
(see the [top-level README](../../README.md)).  Then:

```sh
cd examples/hello_world
cargo run
```

Expected output:

```text
Hello, World!
```

## Verified by presubmits

This example is not just documentation. The project's CI builds and runs it on
every push and pull request, across the same ICU versions as the rest of the
test matrix. The `make docker-test-example` target builds the crate inside the
dockerized test environment, runs `cargo test` (which asserts that the formatted
output equals `Hello, World!`), and then runs the binary with `cargo run`. If
the example ever stops compiling or stops producing the expected output, CI
fails.

You can reproduce the presubmit locally with:

```sh
make DOCKER_TEST_ENV=rust_icu_testenv-77 RUST_ICU_MAJOR_VERSION_NUMBER=77 docker-test-example
```

## What it shows

- [`Cargo.toml`](Cargo.toml) — the dependencies required to format a message:
  `rust_icu_common`, `rust_icu_uloc`, `rust_icu_umsg`, and `rust_icu_ustring`.
- [`src/main.rs`](src/main.rs) — building a `UMessageFormat` from a pattern and
  a locale, then formatting a positional argument with the
  [`message_format!`](https://docs.rs/rust_icu_umsg) macro.

See the [`rust_icu_umsg` crate docs](https://docs.rs/rust_icu_umsg) for more
advanced patterns (numbers, dates, plural rules, etc.).
