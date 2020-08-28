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

/// Contains the API configuration as prescribed by ECMA 402.
///
/// The meaning of the options is the same as in the similarly named
/// options in the JS version.
///
/// See [DateTimeFormatOptions] for the contents of the options.  See the [DateTimeFormat::try_new]
/// for the use of the options.
pub mod options {
    /// The date and time formatting options.
    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum Style {
        /// Full length format style.
        ///
        /// * Date: "Wednesday, December 19, 2012"
        /// * Time: "7:00:00 PM Pacific Standard Time"
        Full,
        /// Long length format style.
        ///
        /// * Date: "December 19, 2012"
        /// * Time: "7:00:00 PM PST"
        Long,
        /// Medium length format style.
        ///
        /// * Date: "Dec 19, 2012"
        /// * Time: "7:00:00 PM"
        Medium,
        /// Short length format style.
        ///
        /// * Date: "12/29/12"
        /// * "7:00 PM"
        Short,
    }

    /// Controls the calendar to use.
    ///
    /// Possible values include: "buddhist", "chinese", " coptic", "ethiopia", "ethiopic",
    /// "gregory", " hebrew", "indian", "islamic", "iso8601", " japanese", "persian", "roc".
    ///
    /// The value entered as currency is not validated.  This responsibility is
    /// delegated to the implementor.
    #[derive(Eq, PartialEq, Debug, Clone)]
    pub struct Calendar(pub String);

    impl Default for Calendar {
        fn default() -> Self {
            Self("gregory".into())
        }
    }

    impl From<&str> for Calendar {
        fn from(s: &str) -> Self {
            Self(s.to_string())
        }
    }

    /// The way day periods should be expressed.
    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum DayPeriod {
        Narrow,
        Short,
        Long,
    }

    /// Controls the number formatting.
    ///
    /// Possible values include: "arab", "arabext", " bali", "beng", "deva", "fullwide", "gujr",
    /// "guru", "hanidec", "khmr", " knda", "laoo", "latn", "limb", "mlym", " mong", "mymr",
    /// "orya", "tamldec", " telu", "thai", "tibt".
    ///
    /// The value entered as currency is not validated.  This responsibility is
    /// delegated to the implementor.
    #[derive(Eq, PartialEq, Debug, Clone)]
    pub struct NumberingSystem(pub String);

    impl From<&str> for NumberingSystem {
        fn from(s: &str) -> Self {
            Self(s.to_string())
        }
    }

    impl Default for NumberingSystem {
        fn default() -> Self {
            Self("latn".to_string())
        }
    }

    /// Controls the time zone formatting.
    ///
    /// The value entered as currency is not validated.  This responsibility is
    /// delegated to the implementor.
    #[derive(Eq, PartialEq, Debug, Clone)]
    pub struct TimeZone(pub String);

    impl From<&str> for TimeZone {
        fn from(s: &str) -> Self {
            Self(s.to_string())
        }
    }

    impl Default for TimeZone {
        fn default() -> Self {
            Self("UTC".to_string())
        }
    }

    /// The hour cycle to use
    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum HourCycle {
        /// 12 hour cycle, 0..11.
        H11,
        /// 12 hour cycle, 1..12.
        H12,
        /// 4 hour cycle, 0..23.
        H23,
        /// 4 hour cycle, 1..24.
        H24
    }

    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum Weekday {
        /// "Thursday"
        Long,
        /// "Thu",
        Short,
        /// "T",
        Narrow,
    }

    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum Era {
        /// "Anno Domini"
        Long,
        /// "AD",
        Short,
        /// "A",
        Narrow,
    }

    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum DisplaySize {
        /// "2012"
        Numeric,
        /// "12"
        TwoDigit,
    }

    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum Month {
        /// "3"
        Numeric,
        /// "03",
        TwoDigit,
        /// "March",
        Long,
        /// "Mar"
        Short,
        /// "M"
        Narrow,
    }

    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum TimeZoneStyle {
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct DateTimeFormatOptions {
    /// The formatting style to use for formatting the date part.
    pub date_style: options::Style,
    /// The formatting style to use for formatting the time part.
    pub time_style: options::Style,
    /// The number of fractional seconds to apply when calling `format`.
    /// Valid values are 0 to 3.
    pub fractional_second_digits: u8,
    /// If left unspecified, the locale default is used.
    pub calendar: Option<options::Calendar>,
    /// If left unspecified, the locale default is used.
    pub day_period: Option<options::DayPeriod>,
    /// If left unspecified, the locale default is used.
    pub numbering_system: Option<options::NumberingSystem>,
    /// If left unspecified, the locale default is used.
    pub time_zone: Option<options::TimeZone>,
    /// If left unspecified, the locale default is used.
    pub hour_cycle: Option<options::HourCycle>,
    /// If left unspecified, the locale default is used.
    pub weekday: Option<options::Weekday>,
    /// If left unspecified, the locale default is used.
    pub era: Option<options::Era>,
    /// If left unspecified, the locale default is used.
    pub year: Option<options::DisplaySize>,
    /// If left unspecified, the locale default is used.
    pub month: Option<options::Month>,
    /// If left unspecified, the locale default is used.
    pub day: Option<options::DisplaySize>,
    /// If left unspecified, the locale default is used.
    pub hour: Option<options::DisplaySize>,
    /// If left unspecified, the locale default is used.
    pub minute: Option<options::DisplaySize>,
    /// If left unspecified, the locale default is used.
    pub second: Option<options::DisplaySize>,
    /// If left unspecified, the locale default is used.
    pub time_zone_style: Option<options::TimeZoneStyle>,
}

impl Default for DateTimeFormatOptions {
    fn default() -> Self {
        Self{
            date_style: options::Style::Long,
            time_style: options::Style::Long,
            fractional_second_digits: 2,
            day_period: None,
            numbering_system: None,
            calendar: None,
            time_zone: None,
            hour_cycle: None,
            weekday: None,
            era: None,
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: None,
            second: None,
            time_zone_style: None,
        }
    }
}

use std::fmt;

pub trait DateTimeFormat {
    /// The type of error reported, if any.
    type Error: std::error::Error;

    /// Creates a new [DateTimeFormat].
    ///
    /// Creation may fail, for example, if the locale-specific data is not loaded, or if
    /// the supplied options are inconsistent.
    fn try_new<L>(l: L, opts: DateTimeFormatOptions) -> Result<Self, Self::Error>
    where
        L: crate::Locale,
        Self: Sized;

    /// Formats `date` into the supplied standard `writer` [fmt::Write].
    ///
    /// The original [ECMA 402 function][ecma402fmt] returns a string.  This is likely the only
    /// reasonably generic option in JavaScript so it is adequate.  In Rust, however, it is
    /// possible to pass in a standard formatting strategy (through `writer`).
    ///
    ///   [ecma402fmt]:
    ///   https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat/format
    ///
    /// The `date` holds the number of seconds (with fractional part) since the beginning of the
    /// Unix epoch.  The date is a very generic type because there is no official date-time type
    /// in Rust.
    fn format<I, D, W>(&self, date: D, writer: &mut W) -> fmt::Result
    where
        I: fmt::Display,
        D: AsRef<f64>,
        W: fmt::Write;
}
