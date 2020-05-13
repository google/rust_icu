# Uses version tag of the form buildenv-0.0.0.
#
RAW_VERSION := $(shell git describe --dirty --tags --match="buildenv-*")
VERSION ?= $(RAW_VERSION:buildenv-%=%)

# The docker repo should be a more official one.
DOCKER_REPO ?= filipfilmar

.DEFAULT_GOAL: all

build-%: Dockerfile.%
	docker build \
			--build-arg DOCKER_REPO=${DOCKER_REPO} \
			--build-arg VERSION=${VERSION} \
			-f $< -t rust_icu_$*:${VERSION} .

build-maint-%: Dockerfile.maint Dockerfile.buildenv
	docker build \
			--build-arg DOCKER_REPO=${DOCKER_REPO} \
			--build-arg VERSION=${VERSION} \
			--build-arg ICU_VERSION_TAG=maint/maint-$* \
			-f $< -t rust_icu_maint-$*:${VERSION} .

build-testenv-%: Dockerfile.testenv Dockerfile.buildenv
	docker build \
			--build-arg DOCKER_REPO=${DOCKER_REPO} \
			--build-arg VERSION=${VERSION} \
			--build-arg ICU_VERSION_TAG=maint-$* \
			-f $< -t rust_icu_testenv-$*:${VERSION} .

tag-%: build-%
	docker tag rust_icu_$*:${VERSION} ${DOCKER_REPO}/rust_icu_$*:${VERSION}

push-%: tag-%
	docker push ${DOCKER_REPO}/rust_icu_$*:${VERSION}

all: \
	push-buildenv \
	push-maint-63 \
	push-testenv-63 \
	push-maint-64 \
	push-testenv-64 \
	push-maint-65 \
	push-testenv-65 \
	push-maint-66 \
	push-testenv-66 \
	push-maint-67 \
	push-testenv-67 \
	push-testenv \
	push-hermetic
	echo "buildenv-version: ${VERSION}"
