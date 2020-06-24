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
    rust_icu_common as common, rust_icu_sys as sys,
    rust_icu_sys::versioned_function,
    rust_icu_sys::*,
    rust_icu_uloc as uloc, rust_icu_ustring as ustring,
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
    pub fn try_new_decimal_rulebased_ustring(
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

    /// Implements `unum_format`
    pub fn format(&self, number: i32) -> Result<String, common::Error> {
        let result = self.format_ustring(number)?;
        String::try_from(&result)
    }

    /// Implements `unum_format`
    // TODO: this method call is repetitive, and should probably be pulled out into a macro.
    pub fn format_ustring(&self, number: i32) -> Result<ustring::UChar, common::Error> {
        const CAPACITY: usize = 200;
        let mut status = common::Error::OK_CODE;
        let mut buf: Vec<sys::UChar> = vec![0; CAPACITY];

        let full_len: i32 = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(unum_format)(
                self.rep.as_ptr(),
                number,
                buf.as_mut_ptr(),
                buf.len() as i32,
                // Unsure what this field should be for.
                0 as *mut sys::UFieldPosition,
                &mut status,
            )
        };
        if status == sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR
            || (common::Error::is_ok(status)
                && full_len > CAPACITY.try_into().map_err(|e| common::Error::wrapper(e))?)
        {
            assert!(full_len > 0);
            let full_len: usize = full_len.try_into().map_err(|e| common::Error::wrapper(e))?;
            buf.resize(full_len, 0);
            unsafe {
                assert!(common::Error::is_ok(status));
                versioned_function!(unum_format)(
                    self.rep.as_ptr(),
                    number,
                    buf.as_mut_ptr(),
                    buf.len() as i32,
                    0 as *mut sys::UFieldPosition,
                    &mut status,
                )
            };
        }
        common::Error::ok_or_warning(status)?;
        if full_len >= 0 {
            let full_len: usize = full_len.try_into().map_err(|e| common::Error::wrapper(e))?;
            buf.resize(full_len, 0);
        }
        Ok(ustring::UChar::from(buf))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn format_decimal_pattern_ustring() {
        struct TestCase {
            locale: &'static str,
            pattern: &'static str,
            expected: &'static str,
        };

        let tests = vec![TestCase {
            locale: "sr-RS",
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
                .format(42)
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
            rule: &'static str,
            expected: &'static str,
        };

        let tests = vec![TestCase {
            locale: "sr-RS",
            rule: "",
            expected: "42",
        }];
        for test in tests {
            let locale = uloc::ULoc::try_from(test.locale).expect("locale exists");
            let pattern = ustring::UChar::try_from(test.rule).expect("pattern is set");
            let fmt = crate::UNumberFormat::try_new_decimal_rulebased_ustring(&pattern, &locale)
                .expect("formatter");

            let result = fmt
                .try_clone()
                .expect("clone")
                .format(42)
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
