
# ICU patches

All ICU versions have been edited to remove existing bazel's BUILD files.
This is because BUILD files will affect how `rules_foreign_cc` builds.
