#! /bin/bash
set -eo pipefail
set -x

echo "${0}"

ICU_LIBRARY_PATH="${ICU_LIBRARY_PATH:-/build/icu-install}"
NUMCPU="${NUMCPU:-4}"

# Needed to take icu-config from ICU_LIBRARY_PATH, not the default
# /usr/local/bin.
PATH="${ICU_LIBRARY_PATH}/bin:${PATH}"
export PATH

readonly _local_cargo_options="\
    --target-dir=/build/cargo \
    "

cd $RUST_ICU_SOURCE_DIR
ls -d .
readonly __all_dirs="$(ls -d rust_icu_*)"

env

function run_cargo_test() {
  env LD_LIBRARY_PATH="${ICU_LIBRARY_PATH}/lib" \
      PKG_CONFIG_LIBDIR="${ICU_LIBRARY_PATH}/lib/pkgconfig" \
      cargo test \
          ${_local_cargo_options} \
            ${CARGO_TEST_ARGS}
}

function run_cargo_doc() {
  env LD_LIBRARY_PATH="${ICU_LIBRARY_PATH}/lib" \
      PKG_CONFIG_LIBDIR="${ICU_LIBRARY_PATH}/lib/pkgconfig" \
      cargo doc ${_local_cargo_options} ${CARGO_TEST_ARGS}
}

(
    echo "Checking out, building and installing the latest ICU"
    mkdir -p /build
    cd /build
    git clone --depth=1 https://github.com/unicode-org/icu.git
    mkdir -p /build/icu4c-build
    mkdir -p ${ICU_LIBRARY_PATH}
    cd /build/icu4c-build
    env CXXFLAGS="-ggdb -DU_DEBUG=1" \
        /build/icu/icu4c/source/runConfigureICU Linux \
            --enable-static \
            --prefix="${ICU_LIBRARY_PATH}" \
            --enable-debug && \
        make -j${NUMCPU} && \
        make install && \
        icu-config --version
)

ls -lR $ICU_LIBRARY_PATH/lib

echo "Testing rust_icu crates"
for directory in ${__all_dirs}; do
  (
    cd "${directory}"
    run_cargo_test
    run_cargo_doc
  )
done
