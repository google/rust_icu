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

//! # ECMA 402 inspired APIs, based on Unicode's [ICU library](http://site.icu-project.org/home)
//!
//! This crate contains implementations of APIs with functionality analogous to those provided by
//! [ECMA 402](https://www.ecma-international.org/publications/standards/Ecma-402.htm) for
//! ECMAScript.  The APIs are said to be *inspired* by ECMA 402, since ECMAScript constructs are
//! not idiomatic in rust.  The differences should be minimal enough that the analogous
//! functionality is readily identified.
//!
//! The plural rules are taken verbatim from the ICU library that is pulled as a
//! dependency.
//!
//! # When to use this library
//!
//! The defining feature of this particular implementation of plural rules is that
//! it is based on the use of the ICU library.  If you need feature parity with
//! C or C++ or Java programs on ICU behavior and do not want to bring in other
//! dependencies, you may want to use this crate.
//!
//! # Alternatives
//!
//! There are other implementations of this functionality for rust.  Here are some,
//! but not necessarily all, different implementations:
//!
//! * [`intl_pluralrules`](https://crates.io/crates/intl_pluralrules): a library that parses the
//! plural rules from the [Unicode CLDR](http://cldr.unicode.org/) repository.
//!
//! # Example use
//!
//! ```
//! use rust_icu_uloc as uloc;
//! use rust_icu_intl as intl;
//! use std::convert::TryFrom;
//! let rules = intl::PluralRules::new(&uloc::ULoc::try_from("ar_EG").unwrap());
//! assert_eq!("zero", rules.select(0.0));
//! assert_eq!("one", rules.select(1.0));
//! assert_eq!("two", rules.select(2.0));
//! assert_eq!("few", rules.select(6.0));
//! assert_eq!("many", rules.select(18.0));
//! ```

use rust_icu_uloc as uloc;
use rust_icu_umsg::{self as umsg, message_format};
use rust_icu_ustring as ustring;
use std::convert::TryFrom;

/// Implements ECMA-402 `Intl.PluralRules` based on the ICU locale data.
pub struct PluralRules {
    formatter: umsg::UMessageFormat,
}

// These responses convert a plural class into English text, which is exactly
// what needs to be reported by the "select" method.
static RESPONSES: &str = r#"{0,plural,
zero {zero}
one {one}
two {two}
few {few}
many {many}
other {other}}"#;

impl PluralRules {
    /// Creates a new plural rules formatter.
    pub fn new(locale: &uloc::ULoc) -> PluralRules {
        let pattern = ustring::UChar::try_from(RESPONSES).expect("pattern should never fail");
        let formatter = umsg::UMessageFormat::try_from(&pattern, &locale)
            .expect("this formatter should never fail");
        PluralRules { formatter }
    }

    /// For a given numeric selector returns the class of plural that is to be used with
    /// this number in the language that [PluralRules] has been configured for.
    ///
    /// Note that the argument `n` is *always* a floating point, because it allows you
    /// to format expressions like "2.54 people on average".
    ///
    /// Returns one of the following classes:
    ///
    /// * `=n`: used for exact overrides, where `n` is a number; for example `=0`.
    /// * `zero`: used for all "zero-like" numbers.  Some languages may have zero-like
    ///   numbers that are not zero themselves.
    ///
    pub fn select(&self, n: f64) -> String {
        message_format!(self.formatter, { n => Double}).expect("should be formattable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Checks the rule and prints useful diagnostic messages in case of failure.
    fn check_rule(expected: &str, n: f64, rules: &PluralRules) {
        assert_eq!(expected, rules.select(n), "select({})", n);
    }

    #[test]
    fn selection_for_ar_eg() -> Result<(), anyhow::Error> {
        let rules = PluralRules::new(&uloc::ULoc::try_from("ar_EG")?);
        check_rule("zero", 0.0, &rules);
        check_rule("one", 1.0, &rules);
        check_rule("two", 2.0, &rules);
        check_rule("few", 6.0, &rules);
        check_rule("many", 18.0, &rules);
        Ok(())
    }

    #[test]
    fn selection_for_sr_rs() -> Result<(), anyhow::Error> {
        let rules = PluralRules::new(&uloc::ULoc::try_from("sr_RS")?);
        check_rule("other", 0.0, &rules);
        check_rule("one", 1.0, &rules);
        check_rule("few", 2.0, &rules);
        check_rule("few", 4.0, &rules);
        check_rule("other", 5.0, &rules);
        check_rule("other", 6.0, &rules);
        check_rule("other", 18.0, &rules);

        check_rule("other", 11.0, &rules);

        check_rule("one", 21.0, &rules);
        check_rule("few", 22.0, &rules);
        check_rule("few", 24.0, &rules);
        check_rule("other", 25.0, &rules);
        check_rule("other", 26.0, &rules);
        Ok(())
    }
}
