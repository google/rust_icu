// Copyright 2021 Google LLC
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

//! Implements the traits found in [ecma402_traits::datetimeformat].

use ecma402_traits;
use rust_icu_common as common;
use rust_icu_udat as udat;
use rust_icu_uloc as uloc;
use rust_icu_ustring as ustring;
use std::{convert::TryFrom, fmt};

#[derive(Debug)]
pub struct DateTimeFormat {
    // The internal representation of date-time formatting.
    rep: udat::UDateFormat,
}

pub(crate) mod internal {
    use ecma402_traits::datetimeformat::DateTimeFormatOptions;
    use rust_icu_common as common;
    use rust_icu_ustring as ustring;
    use std::convert::TryFrom;

    // TODO: implement this conversion completely.
    pub fn opt_to_pattern(_opts: DateTimeFormatOptions) -> Result<ustring::UChar, common::Error> {
        ustring::UChar::try_from("YYYY")
    }
}

impl ecma402_traits::datetimeformat::DateTimeFormat for DateTimeFormat {
    type Error = common::Error;

    /// Creates a new [DateTimeFormat].
    ///
    /// Creation may fail, for example, if the locale-specific data is not loaded,
    /// or if the supplied options are inconsistent.
    fn try_new<L>(
        l: L,
        opts: ecma402_traits::datetimeformat::DateTimeFormatOptions,
    ) -> Result<Self, Self::Error>
    where
        L: ecma402_traits::Locale,
        Self: Sized,
    {
        let locale: &str = &format!("{}", l);
        let locale = uloc::ULoc::try_from(locale)?;
        let pattern = internal::opt_to_pattern(opts)?;
        // tz_id needs to be  empty per ECMA402 spec.  If you need an alternate
        // timezone, use a unicode "-tz-" extension on the locale.
        let tz_id = ustring::UChar::try_from("")?;
        let rep = udat::UDateFormat::new_with_pattern(&locale, &tz_id, &pattern)?;
        Ok(DateTimeFormat { rep })
    }

    /// Formats `date` into the supplied `writer`.
    ///
    /// The function implements [`Intl.DateTimeFormat`][link1] from [ECMA 402][ecma].  The `date`
    /// is expressed in possibly fractional seconds since the Unix Epoch.  The formatting time zone
    /// and calendar are taken from the locale that was passed into `DateTimeFormat::try_new`.
    ///
    ///    [link1]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat
    ///    [ecma]: https://www.ecma-international.org/publications/standards/Ecma-402.htm
    fn format<W>(&self, date: f64, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        let result = self.rep.format(date).map_err(|e| e.into())?;
        write!(writer, "{}", result)
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use ecma402_traits::{datetimeformat::DateTimeFormat, datetimeformat::DateTimeFormatOptions};
    use rust_icu_sys as usys;
    use rust_icu_uloc as uloc;
    use std::convert::TryFrom;

    #[test]
    fn date_time_format_examples() -> Result<(), common::Error> {
        #[derive(Debug, Clone)]
        struct TestCase {
            locale: &'static str,
            opts: DateTimeFormatOptions,
            dates: Vec<usys::UDate>,
            expected: Vec<&'static str>,
        }
        let tests = vec![TestCase {
            locale: "sr_RS-u-tz-uslax",
            opts: Default::default(),
            dates: vec![10000_f64],
            // TBD
            expected: vec!["1970"],
        }];
        for test in tests {
            let locale =
                crate::Locale::FromULoc(uloc::ULoc::try_from(test.locale).expect("locale exists"));
            let formatter = super::DateTimeFormat::try_new(locale, test.clone().opts)?;
            let actual = test
                .dates
                .iter()
                .map(|d| {
                    let mut result = String::new();
                    formatter
                        .format(*d, &mut result)
                        .expect(&format!("can format: {}", d));
                    result
                })
                .collect::<Vec<String>>();
            assert_eq!(test.expected, actual, "for test case: {:?}", &test);
        }
        Ok(())
    }
}
