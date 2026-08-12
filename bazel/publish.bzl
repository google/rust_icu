PublishInfo = provider(fields = ["crates"])

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
    implementation = _cargo_publish_impl,
    executable = True,
    attrs = {
        "crate_dir": attr.string(default = ""),
        "deps": attr.label_list(providers = [PublishInfo]),
    },
)
