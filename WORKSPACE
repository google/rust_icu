load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "36ab8f9facae745c9c9c1b33d225623d976e78f2cc3f729b7973d8c20934ab95",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/download/0.31.0/rules_rust-v0.31.0.tar.gz"],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")

rules_rust_dependencies()

rust_register_toolchains(
    edition = "2021",
    versions = ["1.79.0"],
)

######################################################################

# Use to generate rust-project.json.
# Use bazel run @rules_rust//tools/rust_analyzer:gen_rust_project

load(
    "@rules_rust//tools/rust_analyzer:deps.bzl",
    "rust_analyzer_dependencies",
)

rust_analyzer_dependencies()

######################################################################

# This is how to generate new lock files.  At the project outset, you must
#   (1) create the empty files `//third_party/cargo:Cargo.lock`, and
#       `//third_party/cargo:Cargo.Bazel.lock`.
#   (2) Run `env CARGO_BAZEL_REPI=true bazel build //...` to initialize the
#   lockfiles.
load("@rules_rust//crate_universe:defs.bzl", "crate", "crates_repository", "render_config")

crates_repository(
    name = "crate_index",
    cargo_lockfile = "//:Cargo.lock",
    lockfile = "//:Cargo.Bazel.lock",
    packages = {
        # Add any other crates you need here.
        "bumpalo": crate.spec(
            version = "3.6.1",
        ),
        "libc": crate.spec(
            version = "0.2.34",
        ),
        "paste": crate.spec(
            version = "1.0",
        ),
        "anyhow": crate.spec(
            version = "1.0.72",
        ),
        "bindgen": crate.spec(
            version = "0.59.2",
        ),
        "lazy_static": crate.spec(
            version = "1.4.0",
        ),
    },
    render_config = render_config(
        default_package_name = "",
    ),
)

load("@crate_index//:defs.bzl", "crate_repositories")

crate_repositories()

######################################################################

http_archive(
    name = "icu_74",
    build_file = "//third_party/icu_74:icu.BUILD.bazel",
    integrity = "sha256-J7hlCpTfb5Rcs7aGvjvjIMKjLt8+zimBZyv5i7O6qeE=",
    patch_args = [
        "-p1",
        # Without this flag, the bazel build files just get emptied, but not
        # removed. This is not enough.
        "--remove-empty-files",
    ],
    patches = [
        # ICU has a nascent bazel build, which messes up our build based on
        # the configure_make rule from rules_foreign_cc. So we remove them.
        "//third_party/icu_74:0001-fix-removes-BUILD-files.patch",
    ],
    strip_prefix = "icu-release-74-2",
    urls = [
        "https://github.com/unicode-org/icu/archive/refs/tags/release-74-2.tar.gz",
        "https://github.com/unicode-org/icu/archive/refs/tags/release-74-2.zip",
    ],
)

http_archive(
    name = "icu_73",
    build_file = "//third_party/icu_73:icu.BUILD.bazel",
    integrity = "sha256-57a1QMiB6D0kC0oSty+8l+9frTDY0A8u9nS6GOO/NkA=",
    patch_args = [
        "-p1",
        "--remove-empty-files",
    ],
    patches = [
        "//third_party/icu_73:0001-fix-removes-BUILD-files.patch",
    ],
    strip_prefix = "icu-release-73-1",
    urls = [
        "https://github.com/unicode-org/icu/archive/refs/tags/release-73-1.tar.gz",
        "https://github.com/unicode-org/icu/archive/refs/tags/release-73-1.zip",
    ],
)

http_archive(
    name = "icu_72",
    build_file = "//third_party/icu_72:icu.BUILD.bazel",
    integrity = "sha256-Q8utYo2Y83o/lfbDRXn5FE70veYCSPpgBKTwBtdIfmk=",
    patch_args = [
        "-p1",
        "--remove-empty-files",
    ],
    patches = [
        "//third_party/icu_72:0001-fix-removes-BUILD-files.patch",
    ],
    strip_prefix = "icu-release-72-1",
    urls = [
        "https://github.com/unicode-org/icu/archive/refs/tags/release-72-1.tar.gz",
        "https://github.com/unicode-org/icu/archive/refs/tags/release-72-1.zip",
    ],
)
