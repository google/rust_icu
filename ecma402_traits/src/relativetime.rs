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

use std::fmt;

/// Contains the API configuration as prescribed by [ECMA 402][ecma].
///
///    [ecma]: https://www.ecma-international.org/publications/standards/Ecma-402.htm
///
/// The meaning of the options is the same as in the similarly named
/// options in the JS version.
///
/// See [Options] for the contents of the options.  See the [RelativeTimeFormat::try_new]
/// for the use of the options.
pub mod options {
    /// Whether to use numeric formatting in the display, like "1 day ago".
    #[derive(Debug, Clone, PartialEq)]
    pub enum Numeric {
        /// Always use numeric formatting.
        Always,
        /// Allows not to use numeric formatting, and use "yesterday" instead of "1 day ago".
        Auto,
    }

    /// The length of the internationalized message.
    #[derive(Debug, Clone, PartialEq)]
    pub enum Style {
        /// Default, e.g. "in 1 month"
        Long,
        /// Short, e.g. "in 1 mo."
        Short,
        /// Short, e.g. "in 1 mo". The narrow style could be identical to [Style::Short] in some
        /// locales.
        Narrow,
    }
}

/// The options set by the user at construction time.
///
/// Provides as a "bag of options" since we don't expect any
/// implementations to be attached to this struct.
pub struct Options {
    pub numeric: options::Numeric,
    pub style: options::Style,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            numeric: options::Numeric::Auto,
            style: options::Style::Long,
        }
    }
}

pub trait RelativeTimeFormat {
    /// The type of the error reported, if any.
    type Error: std::error::Error;

    /// Creates a new [RelativeTimeFormat].
    ///
    /// Creation may fail if the locale-specific data is not loaded, or if the supplied options are
    /// inconsistent.
    fn try_new<L>(l: L, opts: Options) -> Result<Self, Self::Error>
    where
        L: crate::Locale,
        Self: Sized;

    /// Formats `days` into the supplied writer.
    ///
    /// A positive value means days in the future.  A negative value means days in the past.
    ///
    /// The function implements [`Intl.RelativeDateFormat`][ref] from [ECMA 402][ecma].
    ///
    /// [ref]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/RelativeTimeFormat
    /// [ecma]: https://www.ecma-international.org/publications/standards/Ecma-402.htm
    ///
    fn format<W>(&self, days: i32, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write;
}
