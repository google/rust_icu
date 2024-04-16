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
        "//third_party/icu_74:remove-build-files.patch",
    ],
    patch_args = [
        "-p1",
        # Without this flag, the bazel build files just get emptied, but not
        # removed. This is not enough.
        "--remove-empty-files",
    ]
)
