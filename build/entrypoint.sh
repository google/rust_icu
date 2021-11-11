#! /bin/bash
set -eo pipefail
set -x

ICU_LIBRARY_PATH="${ICU_LIBRARY_PATH:-/usr/local/lib}"

cd $RUST_ICU_SOURCE_DIR
ls -d .
readonly __all_dirs="$(ls -d rust_icu_*)"

env

function run_cargo_test() {
  env LD_LIBRARY_PATH="/usr/local/lib" cargo test ${CARGO_TEST_ARGS}
}

function run_cargo_doc() {
  env LD_LIBRARY_PATH="/usr/local/lib" cargo doc ${CARGO_TEST_ARGS}
}

# Running cargo test or doc in the top level directory actually does nothing.
# Because of that, descend in each directory individually and run tests and doc
# generation with features.
for directory in ${__all_dirs}; do
  (
    cd "${directory}"
    run_cargo_test
    run_cargo_doc
  )
done
