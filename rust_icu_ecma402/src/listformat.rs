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

//! Implements the traits found in [ecma402_traits::listformat].

use ecma402_traits;
use ecma402_traits::listformat;
use rust_icu_common as common;
use rust_icu_ulistformatter as ulfmt;
use std::fmt;

/// Implements [listformat::Format] using ICU as the underlying implementing library.
#[derive(Debug)]
pub struct Format {
    rep: ulfmt::UListFormatter,
}

// Full support for styled formatting is available since v67.
#[cfg(feature = "icu_version_67_plus")]
pub(crate) mod internal {
    use rust_icu_sys as usys;
    use ecma402_traits::listformat::options;

    // Converts the param types from ECMA-402 into ICU.
    pub fn to_icu_width(style: &options::Style) -> usys::UListFormatterWidth {
        use options::Style;
        use usys::UListFormatterWidth;
        match style {
            Style::Long => UListFormatterWidth::ULISTFMT_WIDTH_WIDE,
            Style::Short => UListFormatterWidth::ULISTFMT_WIDTH_SHORT,
            Style::Narrow => UListFormatterWidth::ULISTFMT_WIDTH_NARROW,
        }
    }

    // Converts the Type param from ECMA-402 into ICU.
    pub fn to_icu_type(in_type: &options::Type) -> usys::UListFormatterType {
        use options::Type;
        use usys::UListFormatterType;
        match in_type {
            Type::Conjunction => UListFormatterType::ULISTFMT_TYPE_AND,
            Type::Disjunction => UListFormatterType::ULISTFMT_TYPE_OR,
        }
    }
}

impl listformat::Format for Format {
    type Error = common::Error;

    /// Creates a new [Format], from a [ecma402_traits::Locale] and [listformat::Options].
    fn try_new<L: ecma402_traits::Locale>(
        l: L,
        _opts: listformat::Options,
    ) -> Result<Format, Self::Error> {
        let locale = format!("{}", l);

        #[cfg(feature = "icu_version_67_plus")]
        {
            let width = internal::to_icu_width(&_opts.style);
            let in_type = internal::to_icu_type(&_opts.in_type);
            let rep = ulfmt::UListFormatter::try_new_styled(&locale, in_type, width)?;
            Ok(Format { rep })
        }

        // The non-v67 implementation is less featureful.
        #[cfg(not(feature = "icu_version_67_plus"))]
        {
            let rep = ulfmt::UListFormatter::try_new(&locale)?;
            Ok(Format { rep })
        }
    }

    /// Formats the given string.
    fn format<I, L, W>(self, list: L, f: &mut W) -> fmt::Result
    where
        I: fmt::Display,
        L: IntoIterator<Item = I>,
        W: fmt::Write,
    {
        // This is an extremely naive implementation: collects the list into
        // a slice and formats that thing.
        let list_str: Vec<String> = list.into_iter().map(|e| format!("{}", e)).collect();
        let refs: Vec<&str> = list_str.iter().map(|e| e.as_str()).collect();
        let result = self.rep.format(&refs[..]).map_err(|e| e.into())?;
        write!(f, "{}", result)
    }
}

#[cfg(test)]
mod testing {

    use super::*;
    use ecma402_traits;
    use ecma402_traits::listformat;
    use ecma402_traits::listformat::Format;
    use rust_icu_uloc as uloc;
    use std::convert::TryFrom;

    #[test]
    fn test_formatting_table() {
        use listformat::options::{Style, Type};
        #[derive(Debug)]
        struct TestCase {
            locale: &'static str,
            array: Vec<&'static str>,
            opts: listformat::Options,
            expected: &'static str,
        };
        let tests = vec![
            TestCase {
                locale: "en-US",
                array: vec!["eenie", "meenie", "minie", "moe"],
                opts: listformat::Options::default(),
                expected: "eenie, meenie, minie, and moe",
            },
            #[cfg(feature = "icu_version_67_plus")]
            TestCase {
                locale: "en-US",
                array: vec!["eenie", "meenie", "minie", "moe"],
                opts: listformat::Options {
                    style: Style::Short,
                    in_type: Type::Conjunction,
                },
                expected: "eenie, meenie, minie, & moe",
            },
            #[cfg(feature = "icu_version_67_plus")]
            TestCase {
                locale: "en-US",
                array: vec!["eenie", "meenie", "minie", "moe"],
                opts: listformat::Options {
                    style: Style::Short,
                    in_type: Type::Disjunction,
                },
                expected: "eenie, meenie, minie, or moe",
            },
            #[cfg(feature = "icu_version_67_plus")]
            TestCase {
                locale: "en-US",
                array: vec!["eenie", "meenie", "minie", "moe"],
                opts: listformat::Options {
                    style: Style::Narrow,
                    in_type: Type::Conjunction,
                },
                expected: "eenie, meenie, minie, moe",
            },
            #[cfg(feature = "icu_version_67_plus")]
            TestCase {
                locale: "en-US",
                array: vec!["eenie", "meenie", "minie", "moe"],
                opts: listformat::Options {
                    style: Style::Narrow,
                    in_type: Type::Disjunction,
                },
                expected: "eenie, meenie, minie, or moe",
            },
            TestCase {
                locale: "en-US",
                array: vec!["eenie", "meenie", "minie", "moe"],
                opts: listformat::Options {
                    style: Style::Long,
                    in_type: Type::Conjunction,
                },
                expected: "eenie, meenie, minie, and moe",
            },
            #[cfg(feature = "icu_version_67_plus")]
            TestCase {
                locale: "en-US",
                array: vec!["eenie", "meenie", "minie", "moe"],
                opts: listformat::Options {
                    style: Style::Long,
                    in_type: Type::Disjunction,
                },
                expected: "eenie, meenie, minie, or moe",
            },
            // Try another sample locale.
            //
            TestCase {
                locale: "sr-RS",
                array: vec!["Раја", "Гаја", "Влаја"],
                opts: listformat::Options {
                    style: Style::Long,
                    in_type: Type::Conjunction,
                },
                expected: "Раја, Гаја и Влаја",
            },
            #[cfg(feature = "icu_version_67_plus")]
            TestCase {
                locale: "sr-RS",
                array: vec!["Раја", "Гаја", "Влаја"],
                opts: listformat::Options {
                    style: Style::Short,
                    in_type: Type::Disjunction,
                },
                expected: "Раја, Гаја или Влаја",
            },
            #[cfg(feature = "icu_version_67_plus")]
            TestCase {
                locale: "sr-RS",
                array: vec!["Раја", "Гаја", "Влаја"],
                opts: listformat::Options {
                    style: Style::Long,
                    in_type: Type::Disjunction,
                },
                expected: "Раја, Гаја или Влаја",
            },
        ];

        for test in tests {
            let locale = uloc::ULoc::try_from(test.locale).expect("locale exists");
            let formatter =
                super::Format::try_new(locale.clone(), test.opts.clone()).expect("has list format");

            let mut result = String::new();
            formatter
                .format(&test.array, &mut result)
                .expect("formatting worked");
            assert_eq!(
                test.expected, result,
                "actual: {}, from: {:?}",
                &result, &test
            );
        }
    }
}
