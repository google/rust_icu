"""
Module extension for fetching upstream ICU dependencies.

This extension manages pulling specific release versions of the ICU library from the unicode-org github repository,
as well as tracking the top of tree (`icu_tot`) versions natively out of `main.zip`.
"""

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# ICU archives static libraries with
#
#     $(AR) $(ARFLAGS) $(AR_OUTOPT)$@ $^
#
# but never defines AR_OUTOPT, so no `-o` is emitted and the output name is
# passed positionally. That is the GNU `ar` convention. On Apple platforms
# rules_foreign_cc supplies `libtool` as AR (with ARFLAGS=-static), which needs
# an explicit `-o` and otherwise fails with "no output file (-o) specified".
#
# config/mh-darwin also does `ARFLAGS += -c`, appending to make's built-in
# default of `r`. Both `r` and `-c` are GNU ar spellings that libtool rejects.
#
# Replacing that one line with a `:=` assignment drops the inherited `r` and
# substitutes the flags libtool actually wants. mh-darwin is included after
# Makefile.inc has composed ARFLAGS, so this assignment is the last word and
# `-static` has to be restated here. This fragment is read only on Darwin, so
# Linux builds are unaffected.
_DARWIN_ARFLAGS_PATCH = (
    "sed -i.bak 's/^ARFLAGS += -c$/ARFLAGS := -static -o/' " +
    "icu4c/source/config/mh-darwin && " +
    "rm -f icu4c/source/config/mh-darwin.bak"
)

_PATCH_CMDS = [
    "find icu4c -name BUILD.bazel -delete",
    "find icu4c -name BUILD -delete",
    _DARWIN_ARFLAGS_PATCH,
]

def _icu_deps_impl(mctx):
    http_archive(
        name = "icu_74",
        build_file = Label("//third_party/icu_74:icu.BUILD.bazel"),
        strip_prefix = "icu-release-74-2",
        urls = ["https://github.com/unicode-org/icu/archive/refs/tags/release-74-2.tar.gz"],
        patch_cmds = _PATCH_CMDS,
    )

    http_archive(
        name = "icu_75",
        build_file = Label("//third_party/icu_75:icu.BUILD.bazel"),
        strip_prefix = "icu-release-75-1",
        urls = ["https://github.com/unicode-org/icu/archive/refs/tags/release-75-1.tar.gz"],
        patch_cmds = _PATCH_CMDS,
    )

    http_archive(
        name = "icu_76",
        build_file = Label("//third_party/icu_76:icu.BUILD.bazel"),
        strip_prefix = "icu-release-76-1",
        urls = ["https://github.com/unicode-org/icu/archive/refs/tags/release-76-1.tar.gz"],
        patch_cmds = _PATCH_CMDS,
    )

    http_archive(
        name = "icu_77",
        build_file = Label("//third_party/icu_77:icu.BUILD.bazel"),
        strip_prefix = "icu-release-77-1",
        urls = ["https://github.com/unicode-org/icu/archive/refs/tags/release-77-1.tar.gz"],
        patch_cmds = _PATCH_CMDS,
    )

    http_archive(
        name = "icu_tot",
        urls = ["https://github.com/unicode-org/icu/archive/refs/heads/main.zip"],
        strip_prefix = "icu-main",
        build_file = Label("//third_party/icu_tot:icu.BUILD.bazel"),
        patch_cmds = _PATCH_CMDS,
    )

icu_deps = module_extension(
    implementation = _icu_deps_impl,
    doc = """
Module extension exposing ICU archives to Bzlmod.

Populates multiple external repositories corresponding to unicode-org ICU versions
mapped natively across bazel build targets: `@icu_74`, `@icu_75`, `@icu_76`, `@icu_77`, and `@icu_tot`.
""",
)
