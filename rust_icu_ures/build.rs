// Copyright 2024 Google LLC
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

//! Build script for `rust_icu_ures`.
//!
//! Compiles the `.txt` resource bundle source files in `testdata/` into binary
//! `.res` files using `genrb`, placing them in `$OUT_DIR/testdata/`.  The
//! resulting directory path is exposed as the `TEST_DATA_DIR` environment
//! variable for use by `#[cfg(test)]` code via `env!("TEST_DATA_DIR")`.

use std::{env, path::PathBuf, process::Command};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let testdata_src = manifest_dir.join("testdata");
    let testdata_out = out_dir.join("testdata");

    std::fs::create_dir_all(&testdata_out).expect("failed to create testdata output directory");

    let genrb = locate_genrb();

    for entry in std::fs::read_dir(&testdata_src).expect("testdata/ directory not found") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("txt") {
            let status = Command::new(&genrb)
                .args(["-d", testdata_out.to_str().unwrap()])
                .arg(&path)
                .status()
                .unwrap_or_else(|e| panic!("failed to spawn genrb at {:?}: {}", genrb, e));
            assert!(status.success(), "genrb failed for {:?}", path);
        }
    }

    println!(
        "cargo:rustc-env=TEST_DATA_DIR={}",
        testdata_out.display()
    );
    println!("cargo:rerun-if-changed=testdata/");
    println!("cargo:rerun-if-changed=build.rs");
}

/// Locates the `genrb` binary.
///
/// Checks, in order:
/// 1. The `ICU_GENRB_PATH` environment variable.
/// 2. The prefix reported by `pkg-config`.
/// 3. The bin directory reported by `icu-config`.
/// 4. Well-known cross-platform installation paths.
/// 5. `genrb` on `$PATH`.
///
/// Panics with a helpful message if none of the above work.
fn locate_genrb() -> PathBuf {
    // 1. Explicit override takes highest priority.
    if let Ok(path) = env::var("ICU_GENRB_PATH") {
        return PathBuf::from(path);
    }

    // 2. Ask pkg-config for the ICU exec-prefix and derive the bin path from it.
    if let Ok(output) = Command::new("pkg-config")
        .args(["--variable=exec_prefix", "icu-i18n"])
        .output()
    {
        if output.status.success() {
            let prefix = std::str::from_utf8(&output.stdout)
                .unwrap_or("")
                .trim()
                .to_string();
            let candidate = PathBuf::from(&prefix).join("bin").join("genrb");
            if candidate.exists() {
                return candidate;
            }
        }
    }

    // 3. Older ICU installations may not ship a pkg-config file but do ship icu-config.
    if let Ok(output) = Command::new("icu-config").arg("--bindir").output() {
        if output.status.success() {
            let bindir = std::str::from_utf8(&output.stdout)
                .unwrap_or("")
                .trim()
                .to_string();
            let candidate = PathBuf::from(&bindir).join("genrb");
            if candidate.exists() {
                return candidate;
            }
        }
    }

    // 4. Common well-known paths and PATH fallback.
    let candidates = [
        "/usr/local/bin/genrb",
        "/usr/bin/genrb",
        "genrb",
    ];

    for candidate in &candidates {
        if Command::new(candidate)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return PathBuf::from(candidate);
        }
    }

    panic!(
        "genrb not found. Install the ICU4C tools package \
         (e.g. `brew install icu4c` on macOS, `apt install icu-devtools` on Debian/Ubuntu) \
         or set the ICU_GENRB_PATH environment variable to the full path of the genrb binary."
    );
}
