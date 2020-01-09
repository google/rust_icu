#! /bin/bash
set -x
env
cd $RUST_ICU_SOURCE_DIR
cargo install bindgen rustfmt
(
  cd rust_icu_sys
  env LD_LIBRARY_PATH="/usr/local/lib" cargo test ${CARGO_TEST_ARGS}
)
(
  cd rust_icu_common
  env LD_LIBRARY_PATH="/usr/local/lib" cargo test ${CARGO_TEST_ARGS}
)
env LD_LIBRARY_PATH="/usr/local/lib" cargo test

