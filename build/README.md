# Docker build and test environments for `rust_icu`

> See the [optional dependencies](/README.md#optional) section of the README.

This directory contains the build and test environments for `rust_icu`, built
using `docker`.  Since building and configuring ICU can be a somewhat finnicky
endeavor, we pre-built a few docker images that already contain ICU installed
from source.

This should make it fairly easy to get started building and testing `rust_icu`.

# Image registry

The provided [`Makefile`](Makefile) automates build, tag and push of the build
environment.

| Name | Prebuilt | Build/push command | Purpose |
| ---- | -------- | ------------- | ------- |
| testenv | filipfilmar/rust_icu_testenv | `make push-testenv` | Non-hermetic test environment for locally mounted source code. Depends on `maint-64`. |
| hermetic | filipfilmar/rust_icu_hermetic | `make push-hermetic` | A hermetic test ran completely inside the container. Depends on `maint-64`. |
| maint-64 | filipfilmar/rust_icu_maint-64 | `make push-maint-64` | A build environment using ICU 64 maintenance branch as wrapping basis. Depends on `buildenv`. |
| buildenv | filipfilmar/rust_icu_buildenv | `make buildenv` | The base build environment image, containing all the basic tools. All build environments, such as `maint-64` for example, use this image as basis. |

# Updating the images

To update all the build images, from the top level directory do the following:

```bash
cd build
make DOCKER_REPO=yourrepo all
```

Omitting `DOCKER_REPO` will cause the push to happen to the default repository, 
which you may not be allowed to write to.

# Building the images locally

It is possible to build the images locally with `docker`
