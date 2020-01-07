TOP_DIR := ${PWD}
DOCKER_REPO ?= filipfilmar

UID := $(shell id -u)
GID := $(shell id -g)
INTERACTIVE:=$(shell [ -t 0 ] && echo 1)
ifeq (${INTERACTIVE},1)
  TTY := --tty --interactive
else
  TTY :=
endif

# The buildenv version that will be used to build and test.
USED_BUILDENV_VERSION := 0.0.1

.PHONY: test
test:
	env LD_LIBRARY_PATH="$(shell icu-config --libdir)" cargo test

# Run a test inside a Docker container.
.PHONY: docker-test
docker-test:
	docker run ${TTY} \
			--volume=${TOP_DIR}:/src/rust_icu \
			${DOCKER_REPO}/rust_icu_testenv:${USED_BUILDENV_VERSION}

# Builds and pushes the build environment containers.  You would not normally
# need to do this.
.PHONY: buildenv
buildenv:
	make -C build DOCKER_REPO=${DOCKER_REPO} all
