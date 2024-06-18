load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "icu_74",
    strip_prefix = "icu-release-74-2",
    integrity = "sha256-J7hlCpTfb5Rcs7aGvjvjIMKjLt8+zimBZyv5i7O6qeE=",
    build_file = "//third_party/icu_74:icu.BUILD.bazel",
    urls = [
        "https://github.com/unicode-org/icu/archive/refs/tags/release-74-2.tar.gz",
        "https://github.com/unicode-org/icu/archive/refs/tags/release-74-2.zip",
    ],
    patches = [
        # ICU has a nascent bazel build, which messes up our build based on
        # the configure_make rule from rules_foreign_cc. So we remove them.
        "//third_party/icu_74:0001-fix-removes-BUILD-files.patch",
    ],
    patch_args = [
        "-p1",
        # Without this flag, the bazel build files just get emptied, but not
        # removed. This is not enough.
        "--remove-empty-files",
    ]
)

http_archive(
    name = "icu_73",
    strip_prefix = "icu-release-73-1",
    integrity = "sha256-57a1QMiB6D0kC0oSty+8l+9frTDY0A8u9nS6GOO/NkA=",
    build_file = "//third_party/icu_73:icu.BUILD.bazel",
    urls = [
        "https://github.com/unicode-org/icu/archive/refs/tags/release-73-1.tar.gz",
        "https://github.com/unicode-org/icu/archive/refs/tags/release-73-1.zip",
    ],
    patches = [
        "//third_party/icu_73:0001-fix-removes-BUILD-files.patch",
    ],
    patch_args = [
        "-p1",
        "--remove-empty-files",
    ]
)

http_archive(
    name = "icu_72",
    strip_prefix = "icu-release-72-1",
    integrity = "sha256-Q8utYo2Y83o/lfbDRXn5FE70veYCSPpgBKTwBtdIfmk=",
    build_file = "//third_party/icu_72:icu.BUILD.bazel",
    urls = [
        "https://github.com/unicode-org/icu/archive/refs/tags/release-72-1.tar.gz",
        "https://github.com/unicode-org/icu/archive/refs/tags/release-72-1.zip",
    ],
    patches = [
        "//third_party/icu_72:0001-fix-removes-BUILD-files.patch",
    ],
    patch_args = [
        "-p1",
        "--remove-empty-files",
    ]
)
