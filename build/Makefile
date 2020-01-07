# Uses version tag of the form buildenv-0.0.0.
#
RAW_VERSION := $(shell git describe --dirty --tags --match="buildenv-*")
VERSION = $(RAW_VERSION:buildenv-%=%)

# The docker repo should be a more official one.
DOCKER_REPO ?= filipfilmar

build-%: Dockerfile.%
	docker build \
			--build-arg DOCKER_REPO=${DOCKER_REPO} \
			--build-arg VERSION=${VERSION} \
			-f $< -t $*:latest .

tag-%: build-%
	docker tag $*:latest ${DOCKER_REPO}/rust_icu_$*:${VERSION}

push-%: tag-%
	docker push ${DOCKER_REPO}/rust_icu_$*:${VERSION}

all: push-buildenv push-maint-64 push-hermetic push-testenv
