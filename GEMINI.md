# Instructions for robotic code updates

## General project information

General information is available in the file README.md.

## Commit rules

Every commit created by Gemini must end with the following note:

```
This commit was created by an automated coding assistant, with human
supervision.
```

## Publishing a new release

* In case of any errors, remove all commits you have created in this process.

* Ensure you have a valid access token for submitting to crates.io.

* Ensure that the repository is in a clean state. Update all git tags from the
  remote repo `origin`.

* Read the last released semver version from the file `LAST_RELEASED_VERSION`.
  Ensure that there is a git tag in this repo that is equal to the last released
  semver version.

* Ensure that the contents of the file `NEXT_VERSION` is equal to the last
  released semver but with minor version number incremented by one.

* Run `make uprev` to ensure that all references to the last released version in
  all TOML files are updated to refer to the next release version.

* If the repository state is dirty, create a commit with all the currently
  outstanding changes. Put into the commit message a summary of all changes
  from the previous release to this release, while upholding the commit rules
  from the section "Commit rules" above.

* Add a git tag to the current commit equal to the content of the file
  `NEXT_VERSION`.

* Run `make publish` to publish all crates to `crates.io`.

* If the previous step was a success, copy the contents of the file
  `NEXT_VERSION` into the file `LAST_RELEASED_VERSION`. After that, increment
  the semver in `NEXT_VERSION` by one minor version number.

* If the repository is dirty, create a commit with all currently outstanding
  changes, upholding the "Commit rules" section above.


