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

//! # ICU number formatting support for rust
//!
//! Since 0.3.1

use {
    paste, rust_icu_common as common, rust_icu_sys as sys,
    rust_icu_sys::versioned_function,
    rust_icu_sys::*,
    rust_icu_uloc as uloc, rust_icu_ustring as ustring,
    rust_icu_ustring::buffered_uchar_method_with_retry,
    std::{convert::TryFrom, convert::TryInto, ptr},
};

#[derive(Debug)]
pub struct UNumberFormat {
    rep: ptr::NonNull<sys::UNumberFormat>,
}

impl Drop for UNumberFormat {
    /// Implements `unum_close`
    fn drop(&mut self) {
        unsafe { versioned_function!(unum_close)(self.rep.as_ptr()) };
    }
}

/// There is a slew of near-identical method calls which differ in the type of
/// the input argument and the name of the function to invoke.
macro_rules! format_ustring_for_type{
    ($method_name:ident, $function_name:ident, $type_decl:ty) => (
        /// Implements `$function_name`.
        pub fn $method_name(&self, number: $type_decl) -> Result<String, common::Error> {
            let result = paste::item! {
                self. [< $method_name _ustring>] (number)?
            };
            String::try_from(&result)
        }

        // Should be able to use https://github.com/google/rust_icu/pull/144 to
        // make this even shorter.
        paste::item! {
            /// Implements `$function_name`.
            pub fn [<$method_name _ustring>] (&self, param: $type_decl) -> Result<ustring::UChar, common::Error> {
                const CAPACITY: usize = 200;
                buffered_uchar_method_with_retry!(
                    [< $method_name _ustring_impl >],
                    CAPACITY,
                    [ rep: *const sys::UNumberFormat, param: $type_decl, ],
                    [ field: *mut sys::UFieldPosition, ]
                    );

                [<$method_name _ustring_impl>](
                    versioned_function!($function_name),
                    self.rep.as_ptr(),
                    param,
                    // The field position is unused for now.
                    0 as *mut sys::UFieldPosition,
                    )
            }
        }
    )
}

impl UNumberFormat {
    /// Implements `unum_open`, with a pattern.
    pub fn try_new_decimal_pattern_ustring(
        pattern: &ustring::UChar,
        locale: &uloc::ULoc,
    ) -> Result<UNumberFormat, common::Error> {
        UNumberFormat::try_new_style_pattern_ustring(
            sys::UNumberFormatStyle::UNUM_PATTERN_DECIMAL,
            pattern,
            locale,
        )
    }

    /// Implements `unum_open`, with rule-based formatting,
    pub fn try_new_decimal_rule_based_ustring(
        rule: &ustring::UChar,
        locale: &uloc::ULoc,
    ) -> Result<UNumberFormat, common::Error> {
        UNumberFormat::try_new_style_pattern_ustring(
            sys::UNumberFormatStyle::UNUM_PATTERN_RULEBASED,
            rule,
            locale,
        )
    }

    /// Implements `unum_open`, with style-based formatting.
    pub fn try_new_with_style(
        style: sys::UNumberFormatStyle,
        locale: &uloc::ULoc,
    ) -> Result<UNumberFormat, common::Error> {
        let rule = ustring::UChar::try_from("")?;
        assert_ne!(style, sys::UNumberFormatStyle::UNUM_PATTERN_RULEBASED);
        assert_ne!(style, sys::UNumberFormatStyle::UNUM_PATTERN_DECIMAL);
        UNumberFormat::try_new_style_pattern_ustring(style, &rule, locale)
    }

    /// Implements `unum_open`
    fn try_new_style_pattern_ustring(
        style: sys::UNumberFormatStyle,
        pattern: &ustring::UChar,
        locale: &uloc::ULoc,
    ) -> Result<UNumberFormat, common::Error> {
        let mut status = common::Error::OK_CODE;
        let mut parse = common::NO_PARSE_ERROR.clone();
        let loc = locale.as_c_str();

        // Unsafety note: all variables should be valid.
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            assert!(common::parse_ok(parse).is_ok());
            versioned_function!(unum_open)(
                style,
                pattern.as_c_ptr(),
                // Mostly OK...
                pattern.len() as i32,
                loc.as_ptr(),
                &mut parse,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        common::parse_ok(parse)?;
        assert_ne!(rep, 0 as *mut sys::UNumberFormat);
        Ok(UNumberFormat {
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Implements `unum_clone`
    pub fn try_clone(&self) -> Result<UNumberFormat, common::Error> {
        let mut status = common::Error::OK_CODE;
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(unum_clone)(self.rep.as_ptr(), &mut status)
        };
        common::Error::ok_or_warning(status)?;
        Ok(UNumberFormat {
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    // Can we make this into a generic method somehow?

    // Implements `unum_format`
    format_ustring_for_type!(format, unum_format, i32);

    // Implements `unum_formatInt64`
    format_ustring_for_type!(format_i64, unum_formatInt64, i64);

    // Implements `unum_formatDouble`
    format_ustring_for_type!(format_f64, unum_formatDouble, f64);
}

/// Used to iterate over the field positions.
pub struct UFieldPositionIterator<'a, T> {
    rep: ptr::NonNull<sys::UFieldPositionIterator>,
    // Owner does not own the representation above, but does own the underlying
    // data.
    owner: Option<&'a T>,
}

impl<'a, T> Drop for UFieldPositionIterator<'a, T> {
    fn drop(&mut self) {
        unsafe { versioned_function!(ufieldpositer_close)(self.rep.as_ptr()) };
    }
}

impl<'a, T> UFieldPositionIterator<'a, T> {
    pub fn try_new_owned(owner: &'a T) -> Result<UFieldPositionIterator<'a, T>, common::Error> {
        let mut status = common::Error::OK_CODE;
        let raw = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ufieldpositer_open)(&mut status)
        };
        common::Error::ok_or_warning(status)?;
        Ok(UFieldPositionIterator {
            rep: ptr::NonNull::new(raw).expect("raw pointer is not null"),
            owner: None,
        })
    }

    pub fn try_new_unowned<'b>() -> Result<UFieldPositionIterator<'b, T>, common::Error> {
        let mut status = common::Error::OK_CODE;
        let raw = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ufieldpositer_open)(&mut status)
        };
        common::Error::ok_or_warning(status)?;
        assert_ne!(raw, 0 as *mut sys::UFieldPositionIterator);
        Ok(UFieldPositionIterator {
            rep: ptr::NonNull::new(raw).expect("raw pointer is not null"),
            owner: None,
        })
    }
}

impl<'a, T> Iterator for UFieldPositionIterator<'a, T> {
    // TODO: Consider turning this into a range once the range properties
    // are known.
    /// The begin of the range and the end of the range index, in that order.
    type Item = (i32, i32);

    /// Gets the next position iterator pair.
    fn next(&mut self) -> Option<Self::Item> {
        let mut begin = 0i32;
        let mut end = 0i32;

        unsafe { versioned_function!(fieldpositer_next)(self.rep.as_ptr(), &mut begin, &mut end) };
        if begin < 0 || end < 0 {
            return None;
        }
        Some((begin, end))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn format_decimal_pattern_ustring() {
        struct TestCase {
            locale: &'static str,
            number: i32,
            pattern: &'static str,
            expected: &'static str,
        };

        let tests = vec![TestCase {
            locale: "sr-RS",
            number: 42,
            pattern: "",
            expected: "42",
        }];
        for test in tests {
            let locale = uloc::ULoc::try_from(test.locale).expect("locale exists");
            let pattern = ustring::UChar::try_from(test.pattern).expect("pattern is set");
            let fmt = crate::UNumberFormat::try_new_decimal_pattern_ustring(&pattern, &locale)
                .expect("formatter");

            let result = fmt
                .try_clone()
                .expect("clone")
                .format(test.number)
                .expect("format success");
            assert_eq!(test.expected, result);
        }
    }

    // TODO: find example rules.
    #[test]
    #[should_panic(expected = "U_MEMORY_ALLOCATION_ERROR")]
    fn format_decimal_rulebased_ustring() {
        struct TestCase {
            locale: &'static str,
            number: i32,
            rule: &'static str,
            expected: &'static str,
        };

        let tests = vec![TestCase {
            locale: "sr-RS",
            number: 42,
            rule: "",
            expected: "42",
        }];
        for test in tests {
            let locale = uloc::ULoc::try_from(test.locale).expect("locale exists");
            let pattern = ustring::UChar::try_from(test.rule).expect("pattern is set");
            let fmt = crate::UNumberFormat::try_new_decimal_rule_based_ustring(&pattern, &locale)
                .expect("formatter");

            let result = fmt
                .try_clone()
                .expect("clone")
                .format(test.number)
                .expect("format success");
            assert_eq!(test.expected, result);
        }
    }

    // TODO: add more, and relevant test cases.
    #[test]
    fn format_style_ustring() {
        struct TestCase {
            locale: &'static str,
            number: i32,
            style: sys::UNumberFormatStyle,
            expected: &'static str,
        };

        let tests = vec![
            TestCase {
                locale: "sr-RS",
                number: 42,
                style: sys::UNumberFormatStyle::UNUM_CURRENCY,
                expected: "42\u{a0}RSD",
            },
            TestCase {
                locale: "sr-RS",
                number: 42,
                style: sys::UNumberFormatStyle::UNUM_SPELLOUT,
                expected: "четрдесет и два",
            },
        ];
        for test in tests {
            let locale = uloc::ULoc::try_from(test.locale).expect("locale exists");
            let fmt =
                crate::UNumberFormat::try_new_with_style(test.style, &locale).expect("formatter");

            let result = fmt
                .try_clone()
                .expect("clone")
                .format(test.number)
                .expect("format success");
            assert_eq!(test.expected, result);
        }
    }
}
