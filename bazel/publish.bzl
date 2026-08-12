"""
Bazel rule toolkit orchestrating topological automatic cargo publishing.

Exposes `PublishInfo` and `cargo_publish` which work together alongside `bazel run`
to traverse a workspace's internal dependencies and spawn sequences of execution blocks pushing to `crates.io`.
"""

PublishInfo = provider(
    doc = "Propagates nested structures tracing recursive Crate compilation boundaries topologically.",
    fields = {
        "crates": "A Depset containing crate directory names structured strictly in postorder formatting.",
    },
)

def _cargo_publish_impl(ctx):
    transitive_crates = []
    for dep in ctx.attr.deps:
        transitive_crates.append(dep[PublishInfo].crates)

    my_crates = depset(
        direct = [ctx.attr.crate_dir] if ctx.attr.crate_dir else [],
        transitive = transitive_crates,
        order = "postorder",
    )

    script = ctx.actions.declare_file(ctx.label.name + ".sh")
    lines = ["#!/bin/bash", "set -e"]
    lines.append("echo 'Starting topological publish...'")

    crate_list = my_crates.to_list()

    for crate in crate_list:
        lines.append("echo '==================================='")
        lines.append("echo 'Publishing {}'".format(crate))
        lines.append("cd $BUILD_WORKSPACE_DIRECTORY/{}".format(crate))
        lines.append("cargo publish || echo 'Publish failed, possibly already published'")
        lines.append("sleep 30")

    ctx.actions.write(
        output = script,
        content = "\n".join(lines),
        is_executable = True,
    )

    return [
        DefaultInfo(executable = script, runfiles = ctx.runfiles()),
        PublishInfo(crates = my_crates),
    ]

cargo_publish = rule(
    doc = """Executes consecutive `cargo publish` cascades targeting internally connected Rust crates seamlessly.

When executed natively via `bazel run`, the rule inherently computes proper semantic dependencies reading
`PublishInfo` configurations recursively, spawning shell commands traversing `crates.io` synchronously.
""",
    implementation = _cargo_publish_impl,
    executable = True,
    attrs = {
        "crate_dir": attr.string(
            default = "",
            doc = "The localized directory name of this active crate.",
        ),
        "deps": attr.label_list(
            providers = [PublishInfo],
            doc = "The target dependencies mapping child crates required upstream natively.",
        ),
    },
)
