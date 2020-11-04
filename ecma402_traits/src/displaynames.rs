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
/// See [Options] for the contents of the options.  See the [DisplayNames::try_new] for the use of
/// the options.
pub mod options {

    /// The formatting style to use
    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum Style {
        /// Example: "US".
        Narrow,
        /// Example: "USA".
        Short,
        /// Default.  Example: "United States".
        Long,
    }

    /// The type of the spellout.  Spelled in the language of the
    /// requested locale.
    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum Type {
        /// The name of the language, e.g. "US English"
        Language,
        /// The name of the region, e.g. "United States".
        Region,
        /// The name of the script, e.g. "Latin",
        Script,
        /// The name of the currency, e.g. "US Dollar",
        Currency,
    }

    /// The fallback to use.
    #[derive(Eq, PartialEq, Debug, Clone)]
    pub enum Fallback {
        /// Default: the fallback is the region code, e.g. "us"
        Code,
        /// No fallback.
        None,
    }
}

/// The options set by the user at construction time.  Provides as a "bag of options" since we
/// don't expect any implementations to be attached to this struct.
///
/// The default values of all the options are prescribed by the TC39 report.
pub struct Options {
    /// The formatting style to use.
    pub style: options::Style,
    /// The type of information to format.
    pub in_type: options::Type,
    /// Sets what to do if the information is not availabe.
    pub fallback: options::Fallback,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            style: options::Style::Long,
            in_type: options::Type::Region,
            fallback: options::Fallback::Code,
        }
    }
}

/// Displays a region, language, script or currency using the language of
/// a specific locale.
pub trait DisplayNames {
    /// The type of error reported, if any.
    type Error: std::error::Error;

    /// Creates a new [DisplayNames].
    ///
    /// Creation may fail, for example, if the locale-specific data is not loaded, or if
    /// the supplied options are inconsistent.
    fn try_new<L>(l: L, opts: Options) -> Result<Self, Self::Error>
    where
        L: crate::Locale,
        Self: Sized;

    /// Formats the information about the given locale in the language used by
    /// this [DisplayNames].
    ///
    /// The function implements [`Intl.DisplayNames`][nfmt] from [ECMA 402][ecma].
    ///
    ///    [nfmt]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DisplayNames
    ///    [ecma]: https://www.ecma-international.org/publications/standards/Ecma-402.htm
    fn format<W, L>(&self, locale: L, writer: &mut W) -> fmt::Result
    where
        W: fmt::Write,
        L: crate::Locale;
}
