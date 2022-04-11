# Uses version tag of the form buildenv-0.0.0.
RAW_VERSION := $(shell git describe --dirty --tags --match="buildenv-*")
VERSION ?= $(RAW_VERSION:buildenv-%=%)

# The docker repo should be a more official one.
DOCKER_REPO ?= filipfilmar

.DEFAULT: all

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

build-testenv-%: Dockerfile.testenv Dockerfile.buildenv entrypoint.sh
	docker build \
			--build-arg DOCKER_REPO=${DOCKER_REPO} \
			--build-arg VERSION=${VERSION} \
			--build-arg ICU_VERSION_TAG=maint-$* \
			-f $< -t rust_icu_testenv-$*:${VERSION} .

tag-%: build-%
	docker tag rust_icu_$*:${VERSION} ${DOCKER_REPO}/rust_icu_$*:${VERSION}

push-%: tag-%
	docker push ${DOCKER_REPO}/rust_icu_$*:${VERSION}

latest: \
	push-maint-71 \
	push-testenv-71
.PHONY: latest

all: \
	push-buildenv \
	push-maint-71 \
	push-testenv-71 \
	push-maint-70 \
	push-testenv-70 \
	push-maint-69 \
	push-testenv-69 \
	push-maint-68 \
	push-testenv-68 \
	push-maint-67 \
	push-testenv-67 \
	push-maint-63 \
	push-testenv-63 \
	push-testenv \
	push-hermetic
	echo "buildenv-version: ${VERSION}"
.PHONY: all
