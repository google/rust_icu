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

//! Implements the traits found in [ecma402_traits::numberformat].

use ecma402_traits;
use rust_icu_common as common;
use rust_icu_sys as sys;
use rust_icu_uloc as uloc;
use rust_icu_unum as unum;
use std::convert::TryFrom;
use std::fmt;

pub struct NumberFormat {
    // The internal representation of number formatting.
    rep: unum::UNumberFormat,
}

pub(crate) mod internal {
    use ecma402_traits::numberformat::options;
    use rust_icu_sys as sys;
}

impl ecma402_traits::numberformat::NumberFormat for NumberFormat {
    type Error = common::Error;

    /// Creates a new [NumberFormat].
    ///
    /// Creation may fail, for example, if the locale-specific data is not loaded, or if
    /// the supplied options are inconsistent.
    fn try_new<L>(l: L, opts: ecma402_traits::numberformat::Options) -> Result<Self, Self::Error>
    where
        L: ecma402_traits::Locale,
        Self: Sized,
    {
        let locale = format!("{}", l);
        let locale = uloc::ULoc::try_from(&locale[..])?;
            if opts.style == ecma402_traits::numberformat::options::Style::Currency {
                if let None = opts.currency {
                    panic!("no currency")
                }
                return Ok(NumberFormat{ rep: unum::UNumberFormat::try_new_with_style(
                    sys::UNumberFormatStyle::UNUM_CURRENCY,
                    &locale,
                )?});
            }
        let rep = unum::UNumberFormat::try_new_with_style(sys::UNumberFormatStyle::UNUM_DECIMAL, &locale)?;
        Ok(NumberFormat { rep })
    }

    /// Formats the plural class of `number` into the supplied `writer`.
    ///
    /// The function implements [`Intl.NumberFormat`][plr] from [ECMA 402][ecma].
    ///
    ///    [plr]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat
    ///    [ecma]: https://www.ecma-international.org/publications/standards/Ecma-402.htm
    fn format<W>(&self, number: f64, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        let (uchars, _) = self
            .rep
            .format_double_for_fields_ustring(number)
            .map_err(|e| e.into())?;
        let result = String::try_from(&uchars).expect(&format!("unable to format: {:?}", uchars));
        write!(writer, "{}", result)
    }
}

#[cfg(test)]
mod testing {

    use super::*;
    use ecma402_traits::numberformat;
    use ecma402_traits::numberformat::NumberFormat;
    use rust_icu_uloc as uloc;
    use std::convert::TryFrom;

    #[test]
    fn formatting() {
        #[derive(Debug, Clone)]
        struct TestCase {
            locale: &'static str,
            opts: numberformat::Options,
            numbers: Vec<f64>,
            expected: Vec<&'static str>,
        }
        let tests = vec![
            TestCase {
                locale: "sr-RS",
                opts: Default::default(),
                numbers: vec![0.0, 1.0, -1.0, 1.5, -1.5, 100.0, 1000.0, 10000.0],
                expected: vec!["0", "1", "-1", "1,5", "-1,5", "100", "1.000", "10.000"],
            },
            TestCase {
                locale: "de-DE",
                opts: numberformat::Options {
                    style: numberformat::options::Style::Currency,
                    currency: Some("EUR".into()),
                    ..Default::default()
                },
                numbers: vec![123456.789],
                expected: vec!["123.456,79\u{a0}€"],
            },
            TestCase {
                locale: "ja-JP",
                opts: numberformat::Options {
                    style: numberformat::options::Style::Currency,
                    currency: Some("JPY".into()),
                    ..Default::default()
                },
                numbers: vec![123456.789],
                expected: vec!["￥123,457"],
            },
            TestCase {
                locale: "en-IN",
                opts: numberformat::Options {
                    maximum_significant_digits: 3,
                    ..Default::default()
                },
                numbers: vec![123456.789],
                expected: vec!["1,23,000"],
            },
        ];
        for test in tests {
            let locale =
                uloc::ULoc::try_from(test.locale).expect(&format!("locale exists: {:?}", &test));
            let format = crate::numberformat::NumberFormat::try_new(locale, test.clone().opts)
                .expect(&format!("try_from should succeed: {:?}", &test));
            let actual = test
                .numbers
                .iter()
                .map(|n| {
                    let mut result = String::new();
                    format
                        .format(*n, &mut result)
                        .expect(&format!("formatting succeeded for: {:?}", &test));
                    result
                })
                .collect::<Vec<String>>();
            assert_eq!(test.expected, actual, "for test case: {:?}", &test);
        }
    }
}
