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
#![feature(proc_macro_hygiene)]

//! # ICU number formatting support for rust (modern)
//!
//! Since 0.3.1

use {
    paste, rust_icu_common as common,
    rust_icu_common::simple_drop_impl,
    rust_icu_sys as sys,
    rust_icu_sys::versioned_function,
    rust_icu_sys::*,
    rust_icu_uloc as uloc, rust_icu_unum as unum,
    rust_icu_ustring as ustring,
    rust_icu_ustring::buffered_uchar_method_with_retry,
    std::{convert::TryFrom, convert::TryInto, ptr},
};

macro_rules! format_type {
    ($method_name:ident, $impl_function_name:ident, $value_type:ty) => {
        /// Implements `$impl_function_name`. Since 0.3.1.
        pub fn $method_name(&self, value: $value_type) -> Result<UFormattedNumber, common::Error> {
            let mut result = UFormattedNumber::try_new()?;
            let mut status = sys::UErrorCode::U_ZERO_ERROR;
            unsafe {
                versioned_function!($impl_function_name)(
                    self.rep.as_ptr(),
                    value,
                    result.as_c_mut_ptr(),
                    &mut status,
                )
            };
            common::Error::ok_or_warning(status)?;
            Ok(result)
        }
    };
}

/// The struct for modern number formatting (akin to ECMA402).
///
/// Use [UNumberFormatter::try_new] to create a new instance of this type.
#[derive(Debug)]
pub struct UNumberFormatter {
    rep: ptr::NonNull<sys::UNumberFormatter>,
}

simple_drop_impl!(UNumberFormatter, unumf_close);

impl UNumberFormatter {
    /// Makes a new [UNumberFormatter], using ICU types.
    ///
    /// To make a new formatter if you have Rust types only, see [UNumberFormatter::try_new].  See
    /// that function also for the description of skeleton syntax.
    ///
    /// Returns the error description if an error is found.
    ///
    /// Implements `unumf_openForSkeletonAndLocaleWithError`. Since 0.3.1.
    /// Implements `unumf_openForSkeletonAndLocale`. Since 0.3.1.
    pub fn try_new_ustring(
        skeleton: &ustring::UChar,
        locale: &uloc::ULoc,
    ) -> Result<UNumberFormatter, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;

        // Uses the "with error" flavor of the constructor for ICU versions upwards of
        // 64.  This allows more elaborate error messages in case of an issue.
        #[cfg(feature = "icu_version_64_plus")]
        {
            let mut parse_status = common::NO_PARSE_ERROR.clone();
            let rep = unsafe {
                assert!(common::Error::is_ok(status));
                versioned_function!(unumf_openForSkeletonAndLocaleWithError)(
                    skeleton.as_c_ptr(),
                    skeleton.len() as i32,
                    locale.label().as_ptr() as *const std::os::raw::c_char,
                    &mut parse_status,
                    &mut status,
                )
            };
            assert_ne!(rep, 0 as *mut sys::UNumberFormatter);
            common::parse_ok(parse_status)?;
            common::Error::ok_or_warning(status)?;
            let result = UNumberFormatter {
                rep: std::ptr::NonNull::new(rep).unwrap(),
            };
            return Ok(result);
        }
        #[cfg(not(feature = "icu_version_64_plus"))]
        {
            let rep = unsafe {
                assert!(common::Error::is_ok(status));
                versioned_function!(unumf_openForSkeletonAndLocale)(
                    skeleton.as_c_ptr(),
                    skeleton.len() as i32,
                    locale.label().as_ptr() as *const std::os::raw::c_char,
                    &mut status,
                )
            };
            assert_ne!(rep, 0 as *mut sys::UNumberFormatter);
            common::Error::ok_or_warning(status)?;
            let result = UNumberFormatter {
                rep: std::ptr::NonNull::new(rep).unwrap(),
            };
            return Ok(result);
        }
    }

    /// Similar to [UNumberFormatter::try_new_ustring] but uses Rust types.
    ///
    /// The `skeleton` is a string that describes the formatting options.
    /// See [skeleton syntax][skel] for detailed documentation.
    ///
    /// Implements `unumf_openForSkeletonAndLocaleWithError`. Since 0.3.1.
    /// Implements `unumf_openForSkeletonAndLocale`. Since 0.3.1.
    ///
    /// [skel]: https://github.com/unicode-org/icu/blob/master/docs/userguide/format_parse/numbers/skeletons.md
    pub fn try_new(skeleton: &str, locale: &str) -> Result<UNumberFormatter, common::Error> {
        let locale = uloc::ULoc::try_from(locale)?;
        let skeleton = ustring::UChar::try_from(skeleton)?;
        UNumberFormatter::try_new_ustring(&skeleton, &locale)
    }

    // Implements `unumf_formatInt`. Since 0.3.1.
    format_type!(format_int, unumf_formatInt, i64);

    // Implements `unumf_formatDouble`. Since 0.3.1.
    format_type!(format_double, unumf_formatDouble, f64);

    /// Implements `unumf_formatDecimal`. Since 0.3.1.
    pub fn format_decimal(&self, value: &str) -> Result<UFormattedNumber, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let mut result = UFormattedNumber::try_new()?;
        unsafe {
            versioned_function!(unumf_formatDecimal)(
                self.rep.as_ptr(),
                value.as_ptr() as *const std::os::raw::c_char,
                value.len() as i32,
                result.as_c_mut_ptr(),
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        Ok(result)
    }
}

/// Stores a formatted number result.
///
/// These objects are produced [UNumberFormatter::format_int], [UNumberFormatter::format_double],
/// [UNumberFormatter::format_decimal].
///
pub struct UFormattedNumber {
    rep: std::ptr::NonNull<sys::UFormattedNumber>,
}

impl UFormattedNumber {
    /// Implements `unumf_openResult`.  Since 0.3.1.
    fn try_new() -> Result<Self, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(unumf_openResult)(&mut status)
        };
        common::Error::ok_or_warning(status)?;
        assert_ne!(rep, 0 as *mut sys::UFormattedNumber);
        let result = std::ptr::NonNull::new(rep).unwrap();
        Ok(UFormattedNumber { rep: result })
    }

    /// Reveals the underlying C representation.
    fn as_c_mut_ptr(&mut self) -> *mut sys::UFormattedNumber {
        self.rep.as_ptr()
    }

    /// Reveals the underlying C representation.
    fn as_c_ptr(&self) -> *const sys::UFormattedNumber {
        self.rep.as_ptr()
    }

    /// Obtains the field iterator for the formatted result.
    ///
    /// Implements `unumf_resultGetAllFieldPositions`. Since 0.3.1.
    ///
    /// Implements `unumf_resultNextFieldPosition`. Since 0.3.1. All access is
    /// via iterators.
    pub fn try_field_iter<'a>(&'a self) -> Result<unum::UFieldPositionIterator<'a, Self>, common::Error> {
        let mut result = unum::UFieldPositionIterator::try_new_owned(self)?;
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        unsafe {
            versioned_function!(unumf_resultGetAllFieldPositions)(
                self.as_c_ptr(),
                result.as_mut_ptr(),
                &mut status,
                )
        };
        common::Error::ok_or_warning(status)?;
        Ok(result)
    }
}

simple_drop_impl!(UFormattedNumber, unumf_closeResult);

impl TryInto<ustring::UChar> for UFormattedNumber {
    type Error = common::Error;

    /// Converts this formatted number into a Unicode string.
    ///
    /// You want to use this method instead of `TryInto<String>` when you need
    /// to do additional processing on the result, such as extracting fields
    /// based on their indexes.
    ///
    /// Implements `unumf_resultToString`.  Since 0.3.1.
    fn try_into(self) -> Result<ustring::UChar, common::Error> {
        const CAPACITY: usize = 200;
        buffered_uchar_method_with_retry!(
            tryinto_impl,
            CAPACITY,
            [rep: *const sys::UFormattedNumber,],
            []
        );
        tryinto_impl(versioned_function!(unumf_resultToString), self.rep.as_ptr())
    }
}

impl TryInto<String> for UFormattedNumber {
    type Error = common::Error;

    /// Converts this formatted number into a Rust string.
    ///
    /// If you intend to use field position iterators on the result, you have to use
    /// `TryInto<ustring::UChar>` instead, because field position iterators use the fixed encoding
    /// of [ustring::UChar] for positioning.
    ///
    /// Implements `unumf_resultToString`.  Since 0.3.1.
    fn try_into(self) -> Result<String, common::Error> {
        let result: ustring::UChar = self.try_into()?;
        String::try_from(&result)
    }
}

#[cfg(test)]
mod testing {
    use std::convert::TryInto;

    #[test]
    fn basic() {
        let fmt = super::UNumberFormatter::try_new(
            "measure-unit/length-meter compact-long sign-always", "sr-RS").unwrap();
        let result = fmt.format_double(123456.7890).unwrap();
        let result_str: String = result.try_into().unwrap();
        assert_eq!("+123 хиљаде m", result_str);

        // Quickly check that the value is as expected.
        let result = fmt.format_double(123456.7890).unwrap();
        let num_fields = result.try_field_iter().unwrap().count();
        assert!(num_fields > 0);
    }

    #[test]
    fn thorough() {
        #[derive(Debug, Clone)]
        struct TestCase {
            locale: &'static str,
            number: f64,
            skeleton: &'static str,
            expected: &'static str,
        }
        let tests = vec![
           TestCase{
               locale: "sr-RS",
               number: 123456.7890,
               skeleton: "measure-unit/length-meter compact-long sign-always",
               expected: "+123 хиљаде m",
           },
           TestCase{
               locale: "sr-RS-u-nu-deva",
               number: 123456.7890,
               skeleton: "measure-unit/length-meter compact-long sign-always",
               expected: "+१२३ хиљаде m",
           },
           TestCase{
               locale: "sr-RS",
               number: 123456.7890,
               skeleton: "numbering-system/deva",
               expected: "१२३.४५६,७८९",
           },
        ];

        for test in tests {
            let fmt = super::UNumberFormatter::try_new(
                &test.skeleton, &test.locale).expect(&format!("for test {:?}", &test));
            let result = fmt.format_double(test.number).unwrap();
            let result_str: String = result.try_into().unwrap();
            assert_eq!(test.expected, result_str, "for test {:?}", &test);
        }
    }
}
