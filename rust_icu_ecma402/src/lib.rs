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

use rust_icu_uloc::ULoc;

/// Implements ECMA-402 [`Intl.ListFormat`][link].
///
/// [link]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat/ListFormat
pub mod listformat;

/// Implements ECMA-402 [`Intl.PluralRules`][link].
///
/// [link]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/PluralRulres/PluralRules
pub mod pluralrules;

/// Implements ECMA-402 [`Intl.NumberFormat`][link].
///
/// [link]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/NumberFormat
pub mod numberformat;

/// Implements ECMA-402 [`Intl.DateTimeFormat`][link].
///
/// [link]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat/DateTimeFormat
pub mod datetimeformat;

pub enum Locale {
    FromULoc(ULoc),
}

impl ecma402_traits::Locale for crate::Locale {}

impl std::fmt::Display for Locale {
    /// Implementation that delegates printing the locale to the underlying
    /// `rust_icu` implementation.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Locale::FromULoc(ref l) => write!(f, "{}", l),
        }
    }
}
