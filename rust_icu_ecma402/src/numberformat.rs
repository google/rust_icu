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

use {
    ecma402_traits, rust_icu_common as common, rust_icu_unumberformatter as unumf,
    std::convert::TryInto, std::fmt,
};

#[derive(Debug)]
pub struct NumberFormat {
    // The internal representation of number formatting.
    rep: unumf::UNumberFormatter,
}

pub(crate) mod internal {
    use {
        ecma402_traits::numberformat, ecma402_traits::numberformat::options,
        rust_icu_common as common,
    };

    /// Produces a [skeleton][skel] that corresponds to the given option.
    ///
    /// The conversion may fail if the options are malformed, for example request currency
    /// formatting but do not have a currency defined.
    ///
    /// [skel]: https://github.com/unicode-org/icu/blob/master/docs/userguide/format_parse/numbers/skeletons.md
    pub fn skeleton_from(opts: &numberformat::Options) -> Result<String, common::Error> {
        let mut skel: Vec<String> = vec![];
        if let Some(ref c) = opts.compact_display {
            match c {
                options::CompactDisplay::Long => skel.push("compact-long".into()),
                options::CompactDisplay::Short => skel.push("compact-short".into()),
            }
        }
        match opts.style {
            options::Style::Currency => {
                match opts.currency {
                    None => {
                        return Err(common::Error::Wrapper(anyhow::anyhow!(
                            "currency not specified"
                        )));
                    }
                    Some(ref c) => {
                        skel.push(format!("currency/{}", &c.0));
                    }
                }
                match opts.currency_display {
                    options::CurrencyDisplay::Symbol => {
                        skel.push(format!("unit-width-short"));
                    }
                    options::CurrencyDisplay::NarrowSymbol => {
                        skel.push(format!("unit-width-narrow"));
                    }
                    options::CurrencyDisplay::Code => {
                        skel.push(format!("unit-width-iso-code"));
                    }
                    options::CurrencyDisplay::Name => {
                        skel.push(format!("unit-width-full-name"));
                    }
                }
                match opts.currency_sign {
                    options::CurrencySign::Accounting => {
                        skel.push(format!("sign-accounting"));
                    }
                    options::CurrencySign::Standard => {
                        // No special setup here.
                    }
                }
            }
            options::Style::Unit => match opts.unit {
                None => {
                    return Err(common::Error::Wrapper(anyhow::anyhow!(
                        "unit not specified"
                    )));
                }
                Some(ref u) => {
                    skel.push(format!("measure-unit/{}", &u.0));
                }
            },
            options::Style::Percent => {
                skel.push(format!("percent"));
            }
            options::Style::Decimal => {
                // Default, no special setup needed, apparently.
            }
        }
        match opts.notation {
            options::Notation::Standard => {
                // Nothing is needed here.
            }
            options::Notation::Engineering => match opts.sign_display {
                options::SignDisplay::Auto => {
                    skel.push(format!("scientific/*ee"));
                }
                options::SignDisplay::Always => {
                    skel.push(format!("scientific/*ee/sign-always"));
                }
                options::SignDisplay::Never => {
                    skel.push(format!("scientific/*ee/sign-never"));
                }
                options::SignDisplay::ExceptZero => {
                    skel.push(format!("scientific/*ee/sign-expect-zero"));
                }
            },
            options::Notation::Scientific => {
                skel.push(format!("scientific"));
            }
            options::Notation::Compact => {
                // ?? Is this true?
                skel.push(format!("compact-short"));
            }
        }
        if let Some(ref n) = opts.numbering_system {
            skel.push(format!("numbering-system/{}", &n.0));
        }

        if opts.notation != options::Notation::Engineering {
            match opts.sign_display {
                options::SignDisplay::Auto => {
                    skel.push("sign-auto".into());
                }
                options::SignDisplay::Never => {
                    skel.push("sign-never".into());
                }
                options::SignDisplay::Always => {
                    skel.push("sign-always".into());
                }
                options::SignDisplay::ExceptZero => {
                    skel.push("sign-always".into());
                }
            }
        }

        let minimum_integer_digits = opts.minimum_integer_digits.unwrap_or(1);
        // TODO: this should match the list at:
        // https://www.currency-iso.org/en/home/tables/table-a1.html
        let minimum_fraction_digits = opts.minimum_fraction_digits.unwrap_or(match opts.style {
            options::Style::Currency => 2,
            _ => 0,
        });
        let maximum_fraction_digits = opts.maximum_fraction_digits.unwrap_or(match opts.style {
            options::Style::Currency => std::cmp::max(2, minimum_fraction_digits),
            _ => 3,
        });
        let minimum_significant_digits = opts.minimum_significant_digits.unwrap_or(1);
        let maximum_significant_digits = opts.maximum_significant_digits.unwrap_or(21);

        // TODO: add skeleton items for min and max integer, fraction and significant digits.
        skel.push(integer_digits(minimum_integer_digits as usize));
        skel.push(fraction_digits(
            minimum_fraction_digits as usize,
            maximum_fraction_digits as usize,
            minimum_significant_digits as usize,
            maximum_significant_digits as usize,
        ));

        Ok(skel.iter().map(|s| format!("{} ", s)).collect())
    }

    // Returns the skeleton annotation for integer width
    // 1 -> "integer-width/*0"
    // 3 -> "integer-width/*000"
    fn integer_digits(digits: usize) -> String {
        let zeroes: String = std::iter::repeat("0").take(digits).collect();
        #[cfg(feature = "icu_version_67_plus")]
        return format!("integer-width/*{}", zeroes);
        #[cfg(not(feature = "icu_version_67_plus"))]
        return format!("integer-width/+{}", zeroes);
    }

    fn fraction_digits(min: usize, max: usize, min_sig: usize, max_sig: usize) -> String {
        eprintln!(
            "fraction_digits: min: {}, max: {} min_sig: {}, max_sig: {}",
            min, max, min_sig, max_sig
        );
        assert!(min <= max, "fraction_digits: min: {}, max: {}", min, max);
        let zeroes: String = std::iter::repeat("0").take(min).collect();
        let hashes: String = std::iter::repeat("#").take(max - min).collect();

        assert!(
            min_sig <= max_sig,
            "significant_digits: min: {}, max: {}",
            min_sig,
            max_sig
        );
        let ats: String = std::iter::repeat("@").take(min_sig).collect();
        let hashes_sig: String = std::iter::repeat("#").take(max_sig - min_sig).collect();

        return format!(".{}{}/{}{}", zeroes, hashes, ats, hashes_sig,);
    }

    #[cfg(test)]
    mod testing {
        use super::*;

        #[test]
        fn fraction_digits_skeleton_fragment() {
            assert_eq!(fraction_digits(0, 3, 1, 21), ".###/@####################");
            assert_eq!(fraction_digits(2, 2, 1, 21), ".00/@####################");
            assert_eq!(fraction_digits(0, 0, 0, 0), "./");
            assert_eq!(fraction_digits(0, 3, 3, 3), ".###/@@@");
        }
    }
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
        let skeleton: String = internal::skeleton_from(&opts)?;
        let rep = unumf::UNumberFormatter::try_new(&skeleton, &locale)?;
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
        let result = self.rep.format_double(number).map_err(|e| e.into())?;
        let result_str: String = result.try_into().map_err(|e: common::Error| e.into())?;
        write!(writer, "{}", result_str)
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
                numbers: vec![
                    0.0, 1.0, -1.0, 1.5, -1.5, 100.0, 1000.0, 10000.0, 123456.789,
                ],
                expected: vec![
                    "0",
                    "1",
                    "-1",
                    "1,5",
                    "-1,5",
                    "100",
                    "1.000",
                    "10.000",
                    "123.456,789",
                ],
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
                    // This is the default for JPY, but we don't consult the
                    // currency list.
                    minimum_fraction_digits: Some(0),
                    maximum_fraction_digits: Some(0),
                    ..Default::default()
                },
                numbers: vec![123456.789],
                expected: vec!["￥123,457"],
            },
            // TODO: This ends up being a syntax error, why?
            //TestCase {
            //locale: "en-IN",
            //opts: numberformat::Options {
            //maximum_significant_digits: Some(3),
            //..Default::default()
            //},
            //numbers: vec![123456.789],
            //expected: vec!["1,23,000"],
            //},
        ];
        for test in tests {
            let locale = crate::Locale::FromULoc(
                uloc::ULoc::try_from(test.locale).expect(&format!("locale exists: {:?}", &test)),
            );
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
            assert_eq!(
                test.expected, actual,
                "\n\tfor test case: {:?},\n\tformat: {:?}",
                &test, &format
            );
        }
    }
}
