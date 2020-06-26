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

/// The struct for number formatting.
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
    /// Implements `unum_open`, with a pattern. Since 0.3.1.
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

    /// Implements `unum_open`, with rule-based formatting. Since 0.3.1
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

    /// Implements `unum_open`, with style-based formatting. Since 0.3.1.
    pub fn try_new_with_style(
        style: sys::UNumberFormatStyle,
        locale: &uloc::ULoc,
    ) -> Result<UNumberFormat, common::Error> {
        let rule = ustring::UChar::try_from("")?;
        assert_ne!(style, sys::UNumberFormatStyle::UNUM_PATTERN_RULEBASED);
        assert_ne!(style, sys::UNumberFormatStyle::UNUM_PATTERN_DECIMAL);
        UNumberFormat::try_new_style_pattern_ustring(style, &rule, locale)
    }

    /// Implements `unum_open`. Since 0.3.1
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

    /// Implements `unum_clone`. Since 0.3.1.
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

    // Implements `unum_format`. Since 0.3.1
    format_ustring_for_type!(format, unum_format, i32);

    // Implements `unum_formatInt64`. Since 0.3.1
    format_ustring_for_type!(format_i64, unum_formatInt64, i64);

    // Implements `unum_formatDouble`. Since 0.3.1
    format_ustring_for_type!(format_f64, unum_formatDouble, f64);

    /// Implements `unum_formatDoubleForFields`. Since 0.3.1.
    ///
    /// Returns a formatted Unicode string, with a field position iterator yielding the ranges of
    /// each individual formatted field as indexes into the returned string.  An UTF8 version of
    /// this is not provided because the field position iterator does not give UTF8 compatible
    /// character indices.
    pub fn format_double_for_fields_ustring<'a>(
        &'a self,
        number: f64,
    ) -> Result<
        (
            ustring::UChar,
            UFieldPositionIterator<'a, *const sys::UNumberFormat>,
        ),
        common::Error,
    > {
        let mut iterator = UFieldPositionIterator::try_new_unowned()?;
        const CAPACITY: usize = 200;

        buffered_uchar_method_with_retry!(
            format_for_fields_impl,
            CAPACITY,
            [format: *const sys::UNumberFormat, number: f64,],
            [iter: *mut sys::UFieldPositionIterator,]
        );

        let result = format_for_fields_impl(
            versioned_function!(unum_formatDoubleForFields),
            self.rep.as_ptr(),
            number,
            iterator.as_mut_ptr(),
        )?;
        Ok((result, iterator))
    }

    /// Implements `unum_formatDecimal`. Since 0.3.1.
    pub fn format_decimal(&self, decimal: &str) -> Result<String, common::Error> {
        let result = self.format_decimal_ustring(decimal)?;
        String::try_from(&result)
    }

    /// Implements `unum_formatDecimal`. Since 0.3.1.
    pub fn format_decimal_ustring(&self, decimal: &str) -> Result<ustring::UChar, common::Error> {
        use std::os::raw;
        const CAPACITY: usize = 200;

        buffered_uchar_method_with_retry!(
            format_decimal_impl,
            CAPACITY,
            [
                format: *const sys::UNumberFormat,
                ptr: *const raw::c_char,
                len: i32,
            ],
            [pos: *mut sys::UFieldPosition,]
        );

        format_decimal_impl(
            versioned_function!(unum_formatDecimal),
            self.rep.as_ptr(),
            decimal.as_ptr() as *const raw::c_char,
            decimal.len() as i32,
            0 as *mut sys::UFieldPosition,
        )
    }

    /// Implements `unum_formatDoubleCurrency`. Since 0.3.1.
    pub fn format_double_currency(&self, number: f64, currency: &str) -> Result<String, common::Error> {
        let currency = ustring::UChar::try_from(currency)?;
        let result = self.format_double_currency_ustring(number, &currency)?;
        String::try_from(&result)
    }

    /// Implements `unum_formatDoubleCurrency`. Since 0.3.1
    pub fn format_double_currency_ustring(&self, number: f64, currency: &ustring::UChar) -> Result<ustring::UChar, common::Error> {
        const CAPACITY: usize = 200;
        buffered_uchar_method_with_retry!(
            format_double_currency_impl,
            CAPACITY,
            [
                format: *const sys::UNumberFormat,
                number: f64, 
                // NUL terminated!
                currency: *mut sys::UChar,
            ],
            [pos: *mut sys::UFieldPosition,]
        );
        // This piece of gymnastics is required because the currency string is
        // expected to be a NUL-terminated UChar.  What?!
        let mut currencyz = currency.clone();
        currencyz.make_z();

        format_double_currency_impl(
            versioned_function!(unum_formatDoubleCurrency),
            self.rep.as_ptr(),
            number, 
            currencyz.as_mut_c_ptr(),
            0 as *mut sys::UFieldPosition,
        )
    }
   
}

/// Used to iterate over the field positions.
pub struct UFieldPositionIterator<'a, T> {
    rep: ptr::NonNull<sys::UFieldPositionIterator>,
    // Owner does not own the representation above, but does own the underlying
    // data.  That's why we let the owner squat here to ensure proper lifetime
    // containment.
    #[allow(dead_code)]
    owner: Option<&'a T>,
}

impl<'a, T> Drop for UFieldPositionIterator<'a, T> {
    fn drop(&mut self) {
        unsafe { versioned_function!(ufieldpositer_close)(self.rep.as_ptr()) };
    }
}

impl<'a, T: 'a> UFieldPositionIterator<'a, T> {
    /// Try creatign a new iterator, based on data supplied by `owner`.
    pub fn try_new_owned(owner: &'a T) -> Result<UFieldPositionIterator<'a, T>, common::Error> {
        let raw = Self::new_raw()?;
        Ok(UFieldPositionIterator {
            rep: ptr::NonNull::new(raw).expect("raw pointer is not null"),
            owner: Some(owner),
        })
    }

    /// Try creating a new iterator, based on data with independent lifetime.
    pub fn try_new_unowned<'b>() -> Result<UFieldPositionIterator<'b, T>, common::Error> {
        let raw = Self::new_raw()?;
        Ok(UFieldPositionIterator {
            rep: ptr::NonNull::new(raw).expect("raw pointer is not null"),
            owner: None,
        })
    }

    fn new_raw() -> Result<*mut sys::UFieldPositionIterator, common::Error> {
        let mut status = common::Error::OK_CODE;
        let raw = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ufieldpositer_open)(&mut status)
        };
        common::Error::ok_or_warning(status)?;
        assert_ne!(raw, 0 as *mut sys::UFieldPositionIterator);
        Ok(raw)
    }

    /// Returns the interal representation pointer.
    fn as_mut_ptr(&mut self) -> *mut sys::UFieldPositionIterator {
        self.rep.as_ptr()
    }
}

/// Returned by [UFieldPositionIterator], represents the spans of each type of
/// the formatting string.
#[derive(Debug, PartialEq)]
pub struct UFieldPositionType {
    /// The field type for the formatting.
    pub field_type: i32,

    /// The index in the buffer at which the range of interest begins.  For
    /// example, in a string "42 RSD", the beginning of "42" would be at index 0.
    pub begin_index: i32,

    /// The index one past the end of the buffer at which the range of interest ends.
    /// For example, in a string "42 RSD", the end of "42" would be at index 2.
    pub past_end_index: i32,
}

impl<'a, T> Iterator for UFieldPositionIterator<'a, T> {
    // TODO: Consider turning this into a range once the range properties
    // are known.
    /// The begin of the range and the end of the range index, in that order.
    type Item = UFieldPositionType;

    /// Gets the next position iterator pair.
    fn next(&mut self) -> Option<Self::Item> {
        let mut begin = 0i32;
        let mut end = 0i32;

        let field_type = unsafe {
            versioned_function!(ufieldpositer_next)(self.rep.as_ptr(), &mut begin, &mut end)
        };
        if field_type < 0 {
            return None;
        }
        Some(UFieldPositionType {
            field_type,
            begin_index: begin,
            past_end_index: end,
        })
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

    #[test]
    fn format_double_with_fields() {
        struct TestCase {
            locale: &'static str,
            number: f64,
            style: sys::UNumberFormatStyle,
            expected: &'static str,
            expected_iter: Vec<UFieldPositionType>,
        };

        let tests = vec![TestCase {
            locale: "sr-RS",
            number: 42.1,
            style: sys::UNumberFormatStyle::UNUM_CURRENCY,
            expected: "42\u{a0}RSD",
            expected_iter: vec![
                // "42"
                UFieldPositionType {
                    field_type: 0,
                    begin_index: 0,
                    past_end_index: 2,
                },
                // "RSD"
                UFieldPositionType {
                    field_type: 7,
                    begin_index: 3,
                    past_end_index: 6,
                },
            ],
        }];
        for test in tests {
            let locale = uloc::ULoc::try_from(test.locale).expect("locale exists");
            let fmt =
                crate::UNumberFormat::try_new_with_style(test.style, &locale).expect("formatter");

            let (ustring, iter) = fmt
                .format_double_for_fields_ustring(test.number)
                .expect("format success");

            let s = String::try_from(&ustring)
                .expect(&format!("string is convertible to utf8: {:?}", &ustring));
            assert_eq!(test.expected, s);
            let iter_values = iter.collect::<Vec<UFieldPositionType>>();
            assert_eq!(test.expected_iter, iter_values);
        }
    }

    #[test]
    fn format_decimal() {
        struct TestCase {
            locale: &'static str,
            number: &'static str,
            style: sys::UNumberFormatStyle,
            expected: &'static str,
        };

        let tests = vec![TestCase {
            locale: "sr-RS",
            number: "1300.55",
            style: sys::UNumberFormatStyle::UNUM_CURRENCY,
            expected: "1.301\u{a0}RSD",
        }];
        for test in tests {
            let locale = uloc::ULoc::try_from(test.locale).expect("locale exists");
            let fmt =
                crate::UNumberFormat::try_new_with_style(test.style, &locale).expect("formatter");

            let s = fmt
                .format_decimal(test.number)
                .expect("format success");

            assert_eq!(test.expected, s);
        }
    }

    #[test]
    fn format_double_currency() {
        struct TestCase {
            locale: &'static str,
            number: f64,
            currency: &'static str,
            style: sys::UNumberFormatStyle,
            expected: &'static str,
        };

        let tests = vec![TestCase {
            locale: "sr-RS",
            number: 1300.55,
            currency: "usd",
            style: sys::UNumberFormatStyle::UNUM_CURRENCY,
            expected: "1.300,55\u{a0}US$",
        }];
        for test in tests {
            let locale = uloc::ULoc::try_from(test.locale).expect("locale exists");
            let fmt =
                crate::UNumberFormat::try_new_with_style(test.style, &locale).expect("formatter");

            let s = fmt
                .format_double_currency(test.number, test.currency)
                .expect("format success");

            assert_eq!(test.expected, s);
        }
    }
}
