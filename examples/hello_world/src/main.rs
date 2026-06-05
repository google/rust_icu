// Copyright 2026 Google LLC
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

//! A minimal "Hello, World!" example for `rust_icu`.
//!
//! It uses [`rust_icu_umsg`] (ICU `MessageFormat`) to format a locale-aware
//! message.  `MessageFormat` is a pattern-based template language built into
//! ICU; it is *not* XLIFF.  Run it with:
//!
//! ```text
//! cargo run
//! ```
//!
//! The formatting logic lives in [`greet`] so it can be exercised by a unit
//! test (see the bottom of this file).  That test is what lets the project's
//! presubmits confirm the example actually produces the expected output.

use rust_icu_common as common;
use rust_icu_uloc as uloc;
use rust_icu_umsg::{self as umsg, message_format};
use rust_icu_ustring as ustring;
use std::convert::TryFrom;

/// Formats a locale-aware "Hello, World!" using ICU `MessageFormat`.
fn greet() -> Result<String, common::Error> {
    // Choose a locale.  Formatting of numbers, dates and plurals is driven by
    // the locale's ICU data.
    let loc = uloc::ULoc::try_from("en-US")?;

    // A MessageFormat pattern.  `{0}` is the first positional argument.
    let pattern = ustring::UChar::try_from("Hello, {0}!")?;
    let fmt = umsg::UMessageFormat::try_from(&pattern, &loc)?;

    // Bind a value to each positional parameter and format the message.
    let name = ustring::UChar::try_from("World")?;
    message_format!(fmt, { name => String })
}

fn main() -> Result<(), common::Error> {
    println!("{}", greet()?); // Hello, World!
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::greet;

    #[test]
    fn formats_hello_world() {
        assert_eq!(greet().expect("formatting should succeed"), "Hello, World!");
    }
}
