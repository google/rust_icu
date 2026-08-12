"""
Module extension for fetching upstream ICU dependencies.

This extension manages pulling specific release versions of the ICU library from the unicode-org github repository,
as well as tracking the top of tree (`icu_tot`) versions natively out of `main.zip`.
"""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

def _icu_deps_impl(mctx):
    http_archive(
        name = "icu_74",
        build_file = Label("//third_party/icu_74:icu.BUILD.bazel"),
        strip_prefix = "icu-release-74-2",
        urls = ["https://github.com/unicode-org/icu/archive/refs/tags/release-74-2.tar.gz"],
        patch_cmds = [
            "find icu4c -name BUILD.bazel -delete",
            "find icu4c -name BUILD -delete",
        ],
    )

    http_archive(
        name = "icu_75",
        build_file = Label("//third_party/icu_75:icu.BUILD.bazel"),
        strip_prefix = "icu-release-75-1",
        urls = ["https://github.com/unicode-org/icu/archive/refs/tags/release-75-1.tar.gz"],
        patch_cmds = [
            "find icu4c -name BUILD.bazel -delete",
            "find icu4c -name BUILD -delete",
        ],
    )

    http_archive(
        name = "icu_76",
        build_file = Label("//third_party/icu_76:icu.BUILD.bazel"),
        strip_prefix = "icu-release-76-1",
        urls = ["https://github.com/unicode-org/icu/archive/refs/tags/release-76-1.tar.gz"],
        patch_cmds = [
            "find icu4c -name BUILD.bazel -delete",
            "find icu4c -name BUILD -delete",
        ],
    )

    http_archive(
        name = "icu_77",
        build_file = Label("//third_party/icu_77:icu.BUILD.bazel"),
        strip_prefix = "icu-release-77-1",
        urls = ["https://github.com/unicode-org/icu/archive/refs/tags/release-77-1.tar.gz"],
        patch_cmds = [
            "find icu4c -name BUILD.bazel -delete",
            "find icu4c -name BUILD -delete",
        ],
    )

    http_archive(
        name = "icu_tot",
        urls = ["https://github.com/unicode-org/icu/archive/refs/heads/main.zip"],
        strip_prefix = "icu-main",
        build_file = Label("//third_party/icu_tot:icu.BUILD.bazel"),
        patch_cmds = [
            "find icu4c -name BUILD.bazel -delete",
            "find icu4c -name BUILD -delete",
        ],
    )

icu_deps = module_extension(
    implementation = _icu_deps_impl,
    doc = """
Module extension exposing ICU archives to Bzlmod.

Populates multiple external repositories corresponding to unicode-org ICU versions
mapped natively across bazel build targets: `@icu_74`, `@icu_75`, `@icu_76`, `@icu_77`, and `@icu_tot`.
""",
)
