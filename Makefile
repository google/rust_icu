TOP_DIR := $(shell pwd)
DOCKER_REPO ?= filipfilmar

# The environment is slightly different from the "regular" environment when
# docker is started with "sudo".  The settings below recover the original user
# name, UID, GID and home directory.
LOGNAME := $(shell logname)
LOGNAME_HOME := $(shell echo ~${LOGNAME})
UID := $(shell id -u ${LOGNAME})
GID := $(shell id -g ${LOGNAME})

INTERACTIVE:=$(shell [ -t 0 ] && echo 1)
ifeq (${INTERACTIVE},1)
  TTY := --tty --interactive
else
  TTY :=
endif

# The buildenv version that will be used to build and test.  This allows us to
# update the buildenv code but not use it immediately.  You can modify the 
# buildenv version by passing its value through env variables like so:
# 
#   make USED_BUILDENV_VERSION=whatever-you-want docker-test
#
# NOTE: This version number is completely independent of the crate version.
USED_BUILDENV_VERSION ?= 0.0.4

test:
	env LD_LIBRARY_PATH="$(shell icu-config --libdir)" cargo test
.PHONY: test

# Run a test inside a Docker container.  The --volume mounts attach local dirs
# so that as much as possible of the host configuration is retained.
TMP ?= /tmp
CARGO_TARGET_DIR := ${TMP}/rust_icu-${LOGNAME}-target

# The docker testing target.  Used to run tests in a dockerized environment,
# based off of a fresh checkout of source in the current directory.
# Pass different values for DOCKER_TEST_ENV and DOCKER_TEST_CARGO_TEST_ARGS to
# test different configurations.  This is useful in Travis CI matrix tests, for
# example.
DOCKER_TEST_ENV ?= rust_icu_testenv-64
DOCKER_TEST_CARGO_TEST_ARGS ?= 
docker-test:
	mkdir -p ${CARGO_TARGET_DIR}
	echo top_dir: ${TOP_DIR}
	echo pwd: $(shell pwd)
	docker run ${TTY} \
			--user=${UID}:${GID} \
			--volume=${TOP_DIR}:/src/rust_icu \
			--volume=${CARGO_TARGET_DIR}:/build/cargo \
			--volume=${LOGNAME_HOME}/.cargo:/usr/local/cargo \
			--env="CARGO_TEST_ARGS=${DOCKER_TEST_CARGO_TEST_ARGS}" \
			${DOCKER_REPO}/${DOCKER_TEST_ENV}:${USED_BUILDENV_VERSION}
.PHONY: docker-test

docker-test-65-renaming:
	$(call run-docker-test,rust_icu_testenv-65,--no-default-features --features=renaming)
.PHONY: docker-test-65-renaming

# Builds and pushes the build environment containers.  You would not normally
# need to do this.
buildenv:
	make -C build DOCKER_REPO=${DOCKER_REPO} all
.PHONY: buildenv

clean:
	cargo clean
.PHONY: clean
# Publishes all crates to crates.io.
#
# The sleep call is needed because we've observed that crates are sometimes
# not found by cargo immediately after a publish.  Sleeping on this is bad,
# but there doesn't seem to be a much better option available.
define publish
	( cd $(1) && cargo publish && sleep 5)
endef

# This is not the best method, since it will error out if a crate has already
# been published.
.PHONY: publish
publish:
	$(call publish,rust_icu_sys)
	$(call publish,rust_icu_common)
	$(call publish,rust_icu_uenum)
	$(call publish,rust_icu_ustring)
	$(call publish,rust_icu_utext)
	$(call publish,rust_icu_uloc)
	$(call publish,rust_icu_ucal)
	$(call publish,rust_icu_udat)
	$(call publish,rust_icu_udata)

# A helper to up-rev the cargo crate versions.
# NOTE: The cargo crate version number is completely independent of the Docker
# build environment version number.
UPREV_OLD_VERSION ?= 0.1.1
UPREV_NEW_VERSION ?= 0.1.2
define uprev
	( \
		cd $(1) && \
		sed -i -e s/${UPREV_OLD_VERSION}/$(UPREV_NEW_VERSION)/g Cargo.toml \
    )
endef

.PHONY: uprev
uprev:
	$(call uprev,rust_icu_sys,)
	$(call uprev,rust_icu_common)
	$(call uprev,rust_icu_uenum)
	$(call uprev,rust_icu_ustring)
	$(call uprev,rust_icu_utext)
	$(call uprev,rust_icu_uloc)
	$(call uprev,rust_icu_ucal)
	$(call uprev,rust_icu_udat)
	$(call uprev,rust_icu_udata)

cov:
	./build/showprogress.sh
