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
USED_BUILDENV_VERSION := 0.0.3

test:
	env LD_LIBRARY_PATH="$(shell icu-config --libdir)" cargo test
.PHONY: test

# Run a test inside a Docker container.  The --volume mounts attach local dirs
# so that as much as possible of the host configuration is retained.
TMP ?= /tmp
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
UPREV_OLD_VERSION ?= 0.0.4
UPREV_NEW_VERSION ?= 0.0.5
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
