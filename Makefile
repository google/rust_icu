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
USED_BUILDENV_VERSION := 0.0.2

test:
	env LD_LIBRARY_PATH="$(shell icu-config --libdir)" cargo test
.PHONY: test

# Run a test inside a Docker container.  The --volume mounts attach local dirs
# so that as much as possible of the host configuration is retained.
CARGO_TARGET_DIR := ${TMP}/rust_icu-${USER}-target
docker-test:
	mkdir -p ${CARGO_TARGET_DIR}
	docker run ${TTY} \
			--user=${UID}:${GID} \
			--volume=${TOP_DIR}:/src/rust_icu \
			--volume=${CARGO_TARGET_DIR}:/build/cargo \
			--volume=${HOME}/.cargo:/usr/local/cargo \
			${DOCKER_REPO}/rust_icu_testenv:${USED_BUILDENV_VERSION}
.PHONY: docker-test

# Builds and pushes the build environment containers.  You would not normally
# need to do this.
buildenv:
	make -C build DOCKER_REPO=${DOCKER_REPO} all
.PHONY: buildenv

clean:
	cargo clean
.PHONY: clean
