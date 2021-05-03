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
    use ecma402_traits::datetimeformat::options;
    use rust_icu_common as common;
    use rust_icu_ustring as ustring;
    use rust_icu_uloc as uloc;
    use std::convert::TryFrom;
    use anyhow::anyhow;

    fn check_auto_styling_set(opts: &DateTimeFormatOptions) -> Result<(), common::Error> {
        if opts.date_style.is_some() || opts.time_style.is_some() {
            return Err(common::Error::Wrapper(
                anyhow!("can't use custom styling because date_style or time_style is set")));
        }
        Ok(())
    }

    /// The skeleton pattern components come from:
    /// <https://unicode-org.github.io/icu/userguide/format_parse/datetime/#datetimepatterngenerator>
    pub fn skeleton_from_opts(opts: &DateTimeFormatOptions) -> Result<ustring::UChar, common::Error> {
        let mut skel = vec![];
        // Looks like date and time style are mutually exclusive with other
        // settings.
        if let Some(ref date_style) = &opts.date_style {
            match date_style {
                options::Style::Full => {}
                options::Style::Long => {}
                options::Style::Medium => {}
                options::Style::Short => {}
            }
        }
        if let Some(ref time_style) = &opts.time_style {
            match time_style {
                options::Style::Full => {}
                options::Style::Long => {}
                options::Style::Medium => {}
                options::Style::Short => {}
            }
        }
        if let Some(ref digits) = &opts.fractional_second_digits {
            check_auto_styling_set(opts)?;
            match digits.get() {
                1 => skel.push("S"),
                2 => skel.push("SS"),
                3 => skel.push("SSS"),
                _ => skel.push("SSSS"),
            }
        }
        if let Some(ref day_period) = &opts.day_period {
            check_auto_styling_set(opts)?;
            match day_period {
                options::DayPeriod::Short => skel.push("a"),
                // Not obvious for other match arms.
                options::DayPeriod::Long => {},
                options::DayPeriod::Narrow => {}
            }
        }
        if let Some(ref weekday) = &opts.weekday {
            check_auto_styling_set(opts)?;
            match weekday {
                options::Weekday::Long => skel.push("EEEE"),
                options::Weekday::Short => skel.push("EEE"),
                options::Weekday::Narrow => skel.push("EEEEEE"),
            }
        }
        if let Some(ref era) = &opts.era {
            check_auto_styling_set(opts)?;
            match era {
                options::Era::Long => skel.push("GGGG"),
                options::Era::Short => skel.push("GG"),
                options::Era::Narrow => skel.push("GGGGG"),
            }
        }
        if let Some(ref year) =&opts.year {
            check_auto_styling_set(opts)?;
            match year {
                options::DisplaySize::Numeric => skel.push("yyyy"),
                options::DisplaySize::TwoDigit => skel.push("yy"),
            }
        }
        if let Some(ref month) = &opts.month {
            check_auto_styling_set(opts)?;
            match month {
                options::Month::Numeric => skel.push("M"),
                options::Month::TwoDigit => skel.push("MM"),
                options::Month::Short => skel.push("MMM"),
                options::Month::Long => skel.push("MMMM"),
                options::Month::Narrow => skel.push("MMMMM"),
            }
        }
        if let Some(day) = &opts.day {
            check_auto_styling_set(opts)?;
            match day {
                options::DisplaySize::Numeric => skel.push("d"),
                options::DisplaySize::TwoDigit => skel.push("dd"),
            }
        }
        if let Some(hour) = &opts.hour {
            check_auto_styling_set(opts)?;
            if let Some(hour_cycle) = &opts.hour_cycle {
                match hour_cycle {
                    options::HourCycle::H11 => 
                        match hour {
                            options::DisplaySize::Numeric => skel.push("K"),
                            options::DisplaySize::TwoDigit => skel.push("KK"),
                    },
                    options::HourCycle::H12 => 
                        match hour {
                            options::DisplaySize::Numeric => skel.push("h"),
                            options::DisplaySize::TwoDigit => skel.push("hh"),
                        },
                    options::HourCycle::H23 =>
                        match hour {
                            options::DisplaySize::Numeric => skel.push("H"),
                            options::DisplaySize::TwoDigit => skel.push("HH"),
                        },
                    options::HourCycle::H24 => match hour {
                            options::DisplaySize::Numeric => skel.push("k"),
                            options::DisplaySize::TwoDigit => skel.push("kk"),
                        }
                }
            } else {
                // This may not be correct for 23/24 hours locale default.
                match hour {
                    options::DisplaySize::Numeric => skel.push("h"),
                    options::DisplaySize::TwoDigit => skel.push("hh"),
                }
            }
        }
        if let Some(minute) = &opts.minute {
            check_auto_styling_set(opts)?;
            match minute {
                options::DisplaySize::Numeric => skel.push("m"),
                options::DisplaySize::TwoDigit => skel.push("mm"),
            }
        }
        if let Some(second) = &opts.second {
            check_auto_styling_set(opts)?;
            match second {
                options::DisplaySize::Numeric => skel.push("s"),
                options::DisplaySize::TwoDigit => skel.push("ss"),
            }
        }
        if let Some(time_zone_style) = &opts.time_zone_style {
            check_auto_styling_set(opts)?;
            match time_zone_style {
                options::TimeZoneStyle::Long => skel.push("zzzz"),
                options::TimeZoneStyle::Short => skel.push("O"),
            }
        }
        let concat: String = skel.join(" ");
        ustring::UChar::try_from(&concat[..])
    }
    
    // Modifies the input locale based on the input pattern.  The settings here can only
    // be set on the locale used for formatting, not on the date time formatting pattern.
    //
    // Note: this only works with the BCP47 timezones ("uslax", not "America/Los_Angeles").
    pub fn locale_from_opts(locale: uloc::ULoc, opts: &DateTimeFormatOptions) -> uloc::ULoc {
        let mut locale = uloc::ULocMut::from(locale);
        
        if let Some(calendar) = &opts.calendar {
            locale.set_unicode_keyvalue("ca", &calendar.0);
        }
        if let Some(n) = &opts.numbering_system {
            locale.set_unicode_keyvalue("nu", &n.0);
        }
        if let Some(tz) = &opts.time_zone {
            locale.set_unicode_keyvalue("tz", &tz.0);
        }
        if let Some(h) = &opts.hour_cycle {
            locale.set_unicode_keyvalue("hc", &format!("{}", h));
        }
        // This will panic if any of the settings above are invalid.
        let loc = uloc::ULoc::from(locale);
        loc
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
        let locale = internal::locale_from_opts(
            uloc::ULoc::try_from(&format!("{}", l)[..])?, &opts);
        let gen = udat::UDatePatternGenerator::new(&locale)?;

        let pattern = gen.get_best_pattern_ustring(
            &internal::skeleton_from_opts(&opts)?)?;

        // The correct timezone ID comes from the resulting locale.
        let tz_id = locale.keyword_value("timezone")?.or(Some("".to_owned())).unwrap();
        let tz_id = ustring::UChar::try_from(&tz_id[..])?;

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
    use ecma402_traits::datetimeformat::{options, DateTimeFormat, DateTimeFormatOptions, };
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
        let tests = vec![
            TestCase {
                locale: "sr-RS",
                opts: DateTimeFormatOptions{
                    year:  Some(options::DisplaySize::Numeric),
                    ..Default::default()
                },

                dates: vec![10000_f64],
                expected: vec!["1970."],
            },
            // In ICU 63 this gets reported as "GMT-08:00", likely the ICU data
            // from then didn't contain the Serbian long spellout of the "uslax"
            // time zone.  Turning on from 67 onwards, since at the time this
            // test was written that was the oldest version that did have the
            // long spellout.
            #[cfg(feature="icu_version_67_plus")]
            TestCase {
                locale: "sr",
                opts: DateTimeFormatOptions{
                    time_zone: Some(options::TimeZone("uslax".to_owned())),
                    time_zone_style: Some(options::TimeZoneStyle::Long),
                    ..Default::default()
                },

                dates: vec![10000_f64],
                expected: vec!["Северноамеричко пацифичко стандардно време"],
            },
            TestCase {
                locale: "en-US",
                opts: DateTimeFormatOptions{
                    time_zone: Some(options::TimeZone("uslax".to_owned())),
                    weekday: Some(options::Weekday::Long),
                    year:  Some(options::DisplaySize::Numeric),
                    day: Some(options::DisplaySize::Numeric,),
                    month: Some(options::Month::Short),
                    hour: Some(options::DisplaySize::Numeric),
                    minute: Some(options::DisplaySize::Numeric),
                    second: Some(options::DisplaySize::Numeric),
                    ..Default::default()
                },

                dates: vec![10000_f64],
                expected: vec!["Wednesday, Dec 31, 1969, 4:00:10 PM"],
            },
            TestCase {
                locale: "en-US",
                opts: DateTimeFormatOptions{
                    time_zone: Some(options::TimeZone("uslax".to_owned())),
                    weekday: Some(options::Weekday::Long),
                    year:  Some(options::DisplaySize::Numeric),
                    day: Some(options::DisplaySize::Numeric,),
                    month: Some(options::Month::Short),
                    hour: Some(options::DisplaySize::Numeric),
                    minute: Some(options::DisplaySize::Numeric),
                    second: Some(options::DisplaySize::Numeric),
                    time_zone_style: Some(options::TimeZoneStyle::Short),
                    ..Default::default()
                },

                dates: vec![10000_f64],
                expected: vec!["Wednesday, Dec 31, 1969, 4:00:10 PM GMT-8"],
            },
            TestCase {
                locale: "en-US",
                opts: DateTimeFormatOptions{
                    time_zone: Some(options::TimeZone("nlams".to_owned())),
                    weekday: Some(options::Weekday::Long),
                    year:  Some(options::DisplaySize::Numeric),
                    day: Some(options::DisplaySize::Numeric,),
                    month: Some(options::Month::Short),
                    hour: Some(options::DisplaySize::Numeric),
                    minute: Some(options::DisplaySize::Numeric),
                    second: Some(options::DisplaySize::Numeric),
                    time_zone_style: Some(options::TimeZoneStyle::Short),
                    ..Default::default()
                },

                dates: vec![10000_f64],
                expected: vec!["Thursday, Jan 1, 1970, 1:00:10 AM GMT+1"],
            },
            TestCase {
                locale: "en-US",
                opts: DateTimeFormatOptions{
                    time_zone: Some(options::TimeZone("rumow".to_owned())),
                    weekday: Some(options::Weekday::Long),
                    year:  Some(options::DisplaySize::Numeric),
                    day: Some(options::DisplaySize::Numeric,),
                    month: Some(options::Month::Short),
                    hour: Some(options::DisplaySize::Numeric),
                    minute: Some(options::DisplaySize::Numeric),
                    second: Some(options::DisplaySize::Numeric),
                    time_zone_style: Some(options::TimeZoneStyle::Short),
                    ..Default::default()
                },

                dates: vec![10000_f64],
                expected: vec!["Thursday, Jan 1, 1970, 3:00:10 AM GMT+3"],
            },
        ];
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
