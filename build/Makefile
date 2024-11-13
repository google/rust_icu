# The make file for the build and test environments for the ICU library.
#
# The targets are named with a suffix `.stamp` so that we can avoid making
# unnecessary build steps, and so that the .stamp files can be easily removed
# together.
#
# Main targets:
#
#   - all (use as `make all`): makes all the docker images
#   - latest: makes only the most recent released docker image.
#   - clean: clean up

# Uses version tag of the form buildenv-0.0.0.
RAW_VERSION := $(shell git describe --dirty --tags --match="buildenv-*")
VERSION ?= $(RAW_VERSION:buildenv-%=%)

# The docker repo should be a more official one.
DOCKER_REPO ?= filipfilmar

.DEFAULT_GOAL: all

all: \
	push-buildenv.stamp \
	latest.stamp \
	v74.stamp \
	v73.stamp \
	v72.stamp \
	v71.stamp \
	v63.stamp \
	push-testenv.stamp \
	push-current.stamp \
	push-hermetic.stamp
	echo "buildenv-version: ${VERSION}"
.PHONY: all

latest: latest.stamp
.PHONY: latest

clean:
	rm -fr *.stamp
.PHONY: clean


build-%.stamp: Dockerfile.%
	docker build \
			--build-arg DOCKER_REPO=${DOCKER_REPO} \
			--build-arg VERSION=${VERSION} \
			-f $< -t rust_icu_$*:${VERSION} .
	touch $@

build-maint-%.stamp: Dockerfile.maint Dockerfile.buildenv
	docker build \
			--build-arg DOCKER_REPO=${DOCKER_REPO} \
			--build-arg VERSION=${VERSION} \
			--build-arg ICU_VERSION_TAG=maint/maint-$* \
			-f $< -t rust_icu_maint-$*:${VERSION} .
	touch $@

push-current.stamp: tag-current.stamp
	docker push ${DOCKER_REPO}/rust_icu_testenv-current:${VERSION}
	touch $@

tag-current.stamp: Makefile build-current.stamp
	docker tag rust_icu_testenv-current:${VERSION} ${DOCKER_REPO}/rust_icu_testenv-current:${VERSION}
	touch $@

build-current.stamp: Dockerfile.testenv.current \
	Dockerfile.buildenv Makefile \
	entrypoint.sh entrypoint-test-current.sh
	docker build \
			--build-arg DOCKER_REPO=${DOCKER_REPO} \
			--build-arg VERSION=${VERSION} \
			--build-arg ICU_VERSION_TAG=maint-74 \
			-f $< -t rust_icu_testenv-current:${VERSION} .
	touch $@

build-testenv-%.stamp: Dockerfile.testenv Dockerfile.buildenv \
	entrypoint.sh entrypoint-test-current.sh
	docker build \
			--build-arg DOCKER_REPO=${DOCKER_REPO} \
			--build-arg VERSION=${VERSION} \
			--build-arg ICU_VERSION_TAG=maint-$* \
			-f $< -t rust_icu_testenv-$*:${VERSION} .
	touch $@

tag-%.stamp: build-%.stamp
	docker tag rust_icu_$*:${VERSION} ${DOCKER_REPO}/rust_icu_$*:${VERSION}
	touch $@

push-%.stamp: tag-%.stamp
	docker push ${DOCKER_REPO}/rust_icu_$*:${VERSION}
	touch $@

latest.stamp: push-buildenv.stamp v74.stamp
	touch $@

# These intermediate files will not be removed. Ideally we'd want all *.stamp
# files to be preserved, but it seems that only explicitly named files can be
# preserved. I.e. %.stamp doesn't work.
.PRECIOUS: \
	push-maint-74.stamp push-testenv-74.stamp \
	build-testenv-74.stamp tag-testenv-74.stamp \
	push-maint-73.stamp push-testenv-73.stamp \
	build-testenv-73.stamp tag-testenv-73.stamp \
	push-maint-72.stamp push-testenv-72.stamp \
	build-testenv-72.stamp tag-testenv-72.stamp \
	push-maint-71.stamp push-testenv-71.stamp \
	build-testenv-71.stamp tag-testenv-71.stamp \
	push-maint-63.stamp push-testenv-63.stamp \
	build-testenv-63.stamp tag-testenv-63.stamp \
	build-current.stamp push-current.stamp


v%.stamp: push-maint-%.stamp push-testenv-%.stamp
	touch $@

# Use `make test` to test building various environments with local
# changes, and an unofficial tag.  This target should not push.
test: Makefile \
	build-buildenv.stamp tag-buildenv.stamp \
	build-maint-74.stamp tag-maint-74.stamp \
	build-current.stamp tag-current.stamp
.PHONY: test

