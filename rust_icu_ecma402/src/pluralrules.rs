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

//! Implements the traits found in [ecma402_traits::pluralrules].

use ecma402_traits;
use rust_icu_common as common;
use rust_icu_upluralrules as uplr;
use std::fmt;

/// Implements [ecma402_traits::pluralrules::PluralRules] using ICU as the underlying
/// implementing library.
pub struct PluralRules {
    // The internal representation of rules.
    rep: uplr::UPluralRules,
}

pub(crate) mod internal {
    use ecma402_traits::pluralrules::options;
    use rust_icu_sys as usys;

    // Converts the trait style option type to an equivalent ICU type.
    pub fn to_icu_type(style: &options::Type) -> usys::UPluralType {
        match style {
            options::Type::Ordinal => usys::UPluralType::UPLURAL_TYPE_ORDINAL,
            options::Type::Cardinal => usys::UPluralType::UPLURAL_TYPE_CARDINAL,
        }
    }
}

impl ecma402_traits::pluralrules::PluralRules for PluralRules {
    type Error = common::Error;

    /// Creates a new [PluralRules].
    ///
    /// Creation may fail, for example, if the locale-specific data is not loaded, or if
    /// the supplied options are inconsistent.
    ///
    /// > Note: not yet implemented: the formatting constraints (min integer digits and such).
    fn try_new<L>(l: L, opts: ecma402_traits::pluralrules::Options) -> Result<Self, Self::Error>
    where
        L: ecma402_traits::Locale,
        Self: Sized,
    {
        let locale = format!("{}", l);
        let style_type = internal::to_icu_type(&opts.in_type);
        let rep = uplr::UPluralRules::try_new_styled(&locale, style_type)?;
        Ok(PluralRules { rep })
    }

    /// Formats the plural class of `number` into the supplied `writer`.
    ///
    /// The function implements [`Intl.PluralRules`][plr] from [ECMA 402][ecma].
    ///
    ///    [plr]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/PluralRules
    ///    [ecma]: https://www.ecma-international.org/publications/standards/Ecma-402.htm
    fn select<W>(&self, number: f64, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        let result = self.rep.select(number).map_err(|e| e.into())?;
        write!(writer, "{}", result)
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use ecma402_traits::pluralrules;
    use ecma402_traits::pluralrules::PluralRules;
    use rust_icu_uloc as uloc;
    use std::convert::TryFrom;

    #[test]
    fn plurals_per_locale() -> Result<(), common::Error> {
        #[derive(Debug, Clone)]
        struct TestCase {
            locale: &'static str,
            opts: pluralrules::Options,
            numbers: Vec<f64>,
            expected: Vec<&'static str>,
        }
        let tests = vec![
            TestCase {
                locale: "ar_EG",
                opts: Default::default(),
                numbers: vec![0 as f64, 1 as f64, 2 as f64, 6 as f64, 18 as f64],
                expected: vec!["zero", "one", "two", "few", "many"],
            },
            TestCase {
                locale: "ar_EG",
                opts: pluralrules::Options {
                    in_type: pluralrules::options::Type::Ordinal,
                    ..Default::default()
                },
                numbers: vec![0 as f64, 1 as f64, 2 as f64, 6 as f64, 18 as f64],
                expected: vec!["other", "other", "other", "other", "other"],
            },
            TestCase {
                locale: "sr_RS",
                opts: Default::default(),
                numbers: vec![0 as f64, 1 as f64, 2 as f64, 4 as f64, 6 as f64, 18 as f64],
                expected: vec!["other", "one", "few", "few", "other", "other"],
            },
            TestCase {
                locale: "sr_RS",
                opts: pluralrules::Options {
                    in_type: pluralrules::options::Type::Ordinal,
                    ..Default::default()
                },
                numbers: vec![0 as f64, 1 as f64, 2 as f64, 4 as f64, 6 as f64, 18 as f64],
                expected: vec!["other", "other", "other", "other", "other", "other"],
            },
        ];
        for test in tests {
            let locale =
                crate::Locale::FromULoc(uloc::ULoc::try_from(test.locale).expect("locale exists"));
            let plr = super::PluralRules::try_new(locale, test.clone().opts)?;
            let actual = test
                .numbers
                .iter()
                .map(|n| {
                    let mut result = String::new();
                    plr.select(*n, &mut result).unwrap();
                    result
                })
                .collect::<Vec<String>>();
            assert_eq!(test.expected, actual, "for test case: {:?}", &test);
        }
        Ok(())
    }
}
