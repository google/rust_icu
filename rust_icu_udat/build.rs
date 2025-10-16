// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This build.rs script provides Cargo _features_ indicating the target ICU4C library version,
//! enabling some conditionally compiled Rust code in this crate that depends on the particular
//! ICU4C version.
//!
//! Please refer to README.md for instructions on how to build the library for your use.

#[cfg(feature = "icu_config")]
fn main() -> anyhow::Result<()> {
    use rust_icu_release::run;

    run()
}

/// No-op if icu_config is disabled.
#[cfg(not(feature = "icu_config"))]
fn main() {}
