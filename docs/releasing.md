# Instructions for releasing new versions of `rust_icu` and the tools.

The current repository consists of two major parts: the binding library proper
(`rust_icu_*` crates) and the build environment (the [`build/`
directory](/build)).

See respective sections for the specific instructions.

# Releasing build environment

The build environment has self-contained code that is able to build and test
the library using Docker.  It lowers the barrier to entry for the new users
or developers of the library, since if you know how to install Docker, you know
how to build and run the library with success.

The build environment is also used for the [continuous integration
(CI)](https://travis-ci.org/google/rust_icu), and thanks to the guarantees
offered by Docker, we know that the CI builds are representative of what you
could do yourself on your machine.

You will need to make a new release of the build environment if you make a
change to the Dockerfiles in `build/`, and want to make those changes available
to the build environments.

## Prerequisites

* A docker container registry that you can read *and* push to.  The current
  default is specified as the variable `DOCKER_REPO` in the
  [`Makefile`](/Makefile), and can be overridden at `make` command line.
  Getting push access to a docker container registry is out of scope of this
  document, refer to the Docker documentation for those details.

  Note, you will need push access to the default container registry to be able
  to release a new buildenv for everyone.

## Process

1. Make the changes you need to make.  Feel free to commit the changes locally
   to git or not, based on your preferred dev workflow.

2. Push the changes to a temporary repository:

   ```
   make DOCKER_REPO=your_registry buildenv
   ```

   This will build and push the build environment containers to your
   repository, as well as print a temporary version identifier (call it
   `BUILT_VERSION` here). Take a note of that value.

3. Test your changes:

   ```
   make DOCKER_REPO=your_registry USED_BUILDENV_VERSION=BUILT_VERSION docker-test
   ```

   (replace `BUILT_VERSION` with the version output by the previous step)

4. If the tests pass, you can send for review and commit once approved.

5. Release a buildenv tag:

   ```
   git pull --rebase origin main  # Ensure that this commit is pushed.
   git tag -a buildenv-0.0.5  # Replace 0.0.5 with the version you intend to release.
   make buildenv # Build and push the container; requires push access to the repo.
   git push origin --tags  # Pushes the tag to the rust_icu repository
   ```

6. Start using the buildenv tag.  Bump up the semantic version value
   `USED_BUILDENV_VERSION` in the [`Makefile`](/Makefile).  Make sure to uphold
   the [semantic versioning ("semver")](https://www.semver.org) rules when
   changing version numbers.  Commit the resulting change.

# Releasing the library proper

## Prerequisites

* A clean git repository; that is, a repo on a main branch that has no unpushed commits, 
  and no other staged or unstaged changes.

* Write permission to publish the crates on `crates.io`.

## Process

The process below assumes that the previous library version was `0.0.4` and that
the new version will be `0.0.5`.

1.  Make sure that the repository is clean.

2. Tag the new release:

   ```
   git tag -a 0.0.5  # This will give you an option to add a description too
   git push origin --tags
   ```

3. Commit the new release:

   ```
   make UPREV_OLD_VERSION=0.0.4 UPREV_NEW_VERSION=0.0.5 uprev
   ```

   The above command will generate changes to the library that need to be committed
   to main.  Create a PR to do so and commit.

4. Publish the result to `crates.io`:

   ```
   make publish
   ```

   The above command will build locally first, and if it succeeds, it will also
   publish the crates to `crates.io`.  It is not possible to *test* easily whether a
   crate versoin is already published.  So you may need to edit the `publish` target
   in the `Makefile` temporarily to exclude the already published crates from the 
   publishing process temporarily. Rinse and repeat until success.

