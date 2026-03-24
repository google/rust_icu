# Instructions for robotic code updates

As only formulaic changes to this repo are anticipated in the near to medium
future, it is reasonable to offload the generation of those changes to robotic
code assistants. This file explains the workflows that need generative input.


## General project information

* General information is available in the file `README.md`.

* Prefer using `git rebase` to `git merge` when taking in new changes. This
  keeps the commit history simple for analysis.

## Commit rules

* Use "Conventional Commits 1.0.0" to formulate the commit messages for each
  git commit.

* Every commit and PR created by Gemini must end with the following note:

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

* Run `make publish` to publish all crates to `crates.io`. Do not use `cargo`
  as we have a custom publication process.

* Run `make cov` to update the function coverage report.

* If the previous step was a success, copy the contents of the file
  `NEXT_VERSION` into the file `LAST_RELEASED_VERSION`. After that, increment
  the semver in `NEXT_VERSION` by one minor version number.

* If the repository is dirty, create a commit with all currently outstanding
  changes, upholding the "Commit rules" section above.


## Publishing a new buildenv for a new ICU version

A new buildenv must be published if the buildenv itself is updated, or upon
a new ICU release.

### Preliminaries

* Buildenv git tags use a format `buildenv-X.Y.Z`, where `X.Y.Z` is the semver
  version of a buildenv. Use this when looking for git tags.

* ICU versions are usually just an integer. For example ICU 77 is version 77.

* When publishing a new buildenv, ask the user which ICU version we are
  adding.

* In case of any errors, remove all commits you have created in this process.

## Procedure

* In `build/Makefile`:

  * add the following deps into `.PRECIOUS` section, for
   ICU version `N`:
     * `push-maint-N.stamp`, `push-testenv-N.stamp`, `build-testenv-N.stamp`,
       `tag-testenv-N.stamp` in that order, where `N` is replaced by the respective
       ICU version. Follow the style of the `.PRECIOUS` target.

  * Modify the target label in the `test:` target in the file `build/Makefile`
    to match the latest ICU version.

    * Tag  the current HEAD with `buildenv-X.Y.Z`, where `X.Y.Z` is the "next"
      version of buildenv, based on the previous version and an increment
      which follows the rules of "Semantic Versioning", based on conventional
      commits.

      * E.g. if only "fix:" and other tags are present in git commits made since
      the last buildenv tag, increment only the patch version number.

      * If at least one "feat:" is present, increment the minor version, reset
        minor version to zero.

      * If text "BREAKING CHANGE" is present in any commits since last,
        increment major version, reset minor and patch to zero.

* In file `Makefile` at the top level dir do as follows:

  * Add to target `static-bindgen` a dependency `static-bindgen-N.stamp`, where
    `N` is the ICU version.

  * Add to target `static-bindgen-special` a dependency
    `static-bindgen-special-N.stamp`, where `N` is the ICU version.

* Once all changes are ready, create a git commit detailing the change in
  the description.

* Make a new buildenv release as:

  `make buildenv`

* Then, send a PR for this change.

* Now, create a new commit:

  * In file `Makefile` at the top level dir do as follows:

    * Replace the value of `USED_BUILDENV_VERSION` with the just generated

      latest buildenv version.  For `buildenv-X.Y.Z` it will be just `X.Y.Z`.
  * Run `make static-bindgen`

  * Run `make static-bindgen-special`

  * Create a commit for changes so far.

  * Create a PR for this change.


