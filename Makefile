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
USED_BUILDENV_VERSION ?= 1.73.0

CARGO_FEATURE_VERSION :=

ICU_VERSION ?= $(shell icu-config --version)
ICU_MAJOR_VERSION ?= $(basename ${ICU_VERSION})
ICU_LIBDIR := $(shell icu-config --libdir)
PKG_CONFIG_PATH := "${HOME}/local/lib/pkgconfig:${PKG_CONFIG_PATH}"
LD_LIBRARY_PATH := "${ICU_LIBDIR}"
test:
	       echo "ICU version detected:       ${ICU_VERSION}" \
	    && echo "ICU libdir:                 ${ICU_LIBDIR}" \
		&& echo "ICU major version detected: ${ICU_MAJOR_VERSION}" \
		&& echo "PKG_CONFIG_PATH:            ${PKG_CONFIG_PATH}" \
		&& PKG_CONFIG_PATH=${PKG_CONFIG_PATH} \
				LD_LIBRARY_PATH=${LD_LIBRARY_PATH} \
				cargo test \
		&& PKG_CONFIG_PATH=${PKG_CONFIG_PATH} \
				LD_LIBRARY_PATH=${LD_LIBRARY_PATH} \
				cargo doc
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
RUST_ICU_MAJOR_VERSION_NUMBER ?= 73
DOCKER_TEST_ENV ?= rust_icu_testenv-${RUST_ICU_MAJOR_VERSION_NUMBER}
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
			--env="RUST_ICU_MAJOR_VERSION_NUMBER=${RUST_ICU_MAJOR_VERSION_NUMBER}"\
			--env="RUST_BACKTRACE=full" \
			${DOCKER_REPO}/${DOCKER_TEST_ENV}:${USED_BUILDENV_VERSION}
.PHONY: docker-test

# Refreshes the static bindgen output (contents of ./rust_icu_sys/bindgen) based
# on the currently present ICU versions in the test environment.
#
# % is expected to be a number equal to a valid ICU major version number, such
# as "65" or such.
static-bindgen-%.stamp: rust_icu_sys/bindgen/run_bindgen.sh
	mkdir -p ${CARGO_TARGET_DIR}
	echo top_dir: ${TOP_DIR}
	echo pwd: $(shell pwd)
	docker run ${TTY} \
			--user=${UID}:${GID} \
			--volume=${TOP_DIR}:/src/rust_icu \
			--volume=${LOGNAME_HOME}/.cargo:/usr/local/cargo \
			--env="RUST_ICU_MAJOR_VERSION_NUMBER=$*" \
			--entrypoint="/bin/bash" \
			${DOCKER_REPO}/rust_icu_testenv-$*:${USED_BUILDENV_VERSION} \
			  "-c" "env OUTPUT_DIR=./rust_icu/rust_icu_sys/bindgen \
			  ./rust_icu/rust_icu_sys/bindgen/run_bindgen.sh"
	touch $@

# Keep only the latest version of the library in the static-bindgen target,
# and any versions that do not have a lib.rs in rust_icu_sys/bindgen.
static-bindgen: \
    static-bindgen-63.stamp \
    static-bindgen-71.stamp \
    static-bindgen-72.stamp \
    static-bindgen-73.stamp
.PHONY: static-bindgen


static-bindgen-special-%.stamp: rust_icu_sys/bindgen_special/run_bindgen.sh
	mkdir -p "${CARGO_TARGET_DIR}"
	echo top_dir: "${TOP_DIR}"
	echo pwd: "$(shell pwd)"
	if [ "${ICU_SOURCE_DIR}" == "" ]; then exit 1; fi
	docker run ${TTY} \
			--user=${UID}:${GID} \
			--volume=${TOP_DIR}:/src/rust_icu \
			--volume=${LOGNAME_HOME}/.cargo:/usr/local/cargo \
			--volume=${LOGNAME_HOME}/code/icu:/src/icu \
			--env="RUST_ICU_MAJOR_VERSION_NUMBER=$*" \
			--entrypoint="/bin/bash" \
			${DOCKER_REPO}/rust_icu_testenv-$*:${USED_BUILDENV_VERSION} \
			  "-c" "env OUTPUT_DIR=./rust_icu/rust_icu_sys/bindgen_special \
			  ./rust_icu/rust_icu_sys/bindgen_special/run_bindgen.sh"
	touch $@

# Keep only the latest version of the library in the static-bindgen target,
# and any versions that do not have a lib.rs in rust_icu_sys/bindgen.
static-bindgen-special: \
    static-bindgen-special-72.stamp \
    static-bindgen-special-73.stamp
.PHONY: static-bindgen-special

# Builds and pushes the build environment containers.  You would not normally
# need to do this.
buildenv:
	make -C build DOCKER_REPO=${DOCKER_REPO} all
.PHONY: buildenv

clean:
	rm -f *.stamp
	cargo clean
.PHONY: clean
# Publishes all crates to crates.io.
#
# The sleep call is needed because we've observed that crates are sometimes
# not found by cargo immediately after a publish.  Sleeping on this is bad,
# but there doesn't seem to be a much better option available.
define publishfn
	( cd $(1) && cargo publish && sleep 30)
endef

######################################################################
## Targets for publishing crates to crates.io

# Everyone's dependency.
publish-rust_icu_sys.stamp:
	$(call publishfn,rust_icu_sys)
	touch $@


publish-rust_icu_common.stamp: \
	publish-rust_icu_sys.stamp
	$(call publishfn,rust_icu_common)
	touch $@

publish-rust_icu_uenum.stamp: \
	publish-rust_icu_common.stamp
	$(call publishfn,rust_icu_uenum)
	touch $@

publish-rust_icu_ustring.stamp: \
	publish-rust_icu_uenum.stamp
	$(call publishfn,rust_icu_ustring)
	touch $@

publish-rust_icu_utext.stamp: \
	publish-rust_icu_ustring.stamp
	$(call publishfn,rust_icu_utext)
	touch $@

publish-rust_icu_uloc.stamp: \
	publish-rust_icu_utext.stamp
	$(call publishfn,rust_icu_uloc)
	touch $@

publish-rust_icu.stamp: \
	publish-rust_icu_sys.stamp \
	publish-rust_icu_common.stamp \
	publish-rust_icu_uenum.stamp \
	publish-rust_icu_utext.stamp \
	publish-rust_icu_uloc.stamp
	$(call publishfn,rust_icu_ucal)
	$(call publishfn,rust_icu_udat)
	$(call publishfn,rust_icu_udata)
	$(call publishfn,rust_icu_ucol)
	$(call publishfn,rust_icu_umsg)
	$(call publishfn,rust_icu_ulistformatter)
	$(call publishfn,rust_icu_upluralrules)
	$(call publishfn,rust_icu_uformattable)
	$(call publishfn,rust_icu_unum)
	$(call publishfn,rust_icu_ubrk)
	$(call publishfn,rust_icu_utrans)
	$(call publishfn,rust_icu_unumberformatter)
	$(call publishfn,rust_icu_unorm2)
	$(call publishfn,rust_icu_uchar)
	$(call publishfn,rust_icu_ucnv)
	touch $@

publish-ecma402_traits.stamp:
	$(call publishfn,ecma402_traits)
	touch $@

publish-rust_icu_ecma402.stamp: publish-rust_icu.stamp publish-ecma402_traits.stamp
	$(call publishfn,rust_icu_ecma402)
	touch $@

publish.stamp: publish-rust_icu.stamp publish-rust_icu_ecma402.stamp
	$(call publishfn,rust_icu)
	touch $@

publish: publish.stamp
.PHONY: publish

# A helper to up-rev the cargo crate versions.
# NOTE: The cargo crate version number is completely independent of the Docker
# build environment version number.
UPREV_OLD_VERSION ?= 4.0.0
UPREV_NEW_VERSION ?= 4.0.1
define uprevfn
	( \
		cd $(1) && \
		sed -i -e s/${UPREV_OLD_VERSION}/$(UPREV_NEW_VERSION)/g Cargo.toml \
    )
endef

uprev:
	$(call uprevfn,rust_icu)
	$(call uprevfn,rust_icu_common)
	$(call uprevfn,rust_icu_intl)
	$(call uprevfn,rust_icu_sys)
	$(call uprevfn,rust_icu_ucal)
	$(call uprevfn,rust_icu_ucol)
	$(call uprevfn,rust_icu_udat)
	$(call uprevfn,rust_icu_udata)
	$(call uprevfn,rust_icu_uenum)
	$(call uprevfn,rust_icu_ulistformatter)
	$(call uprevfn,rust_icu_uloc)
	$(call uprevfn,rust_icu_umsg)
	$(call uprevfn,rust_icu_upluralrules)
	$(call uprevfn,rust_icu_ustring)
	$(call uprevfn,rust_icu_utext)
	$(call uprevfn,rust_icu_uformattable)
	$(call uprevfn,rust_icu_unum)
	$(call uprevfn,rust_icu_unumberformatter)
	$(call uprevfn,rust_icu_ubrk)
	$(call uprevfn,rust_icu_utrans)
	$(call uprevfn,rust_icu_ecma402)
	$(call uprevfn,ecma402_traits)
	$(call uprevfn,rust_icu_unorm2)
	$(call uprevfn,rust_icu_uchar)
	$(call uprevfn,rust_icu_ucnv)
.PHONY: uprev

cov:
	./build/showprogress.sh
