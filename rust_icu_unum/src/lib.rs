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
    paste, rust_icu_common as common,
    rust_icu_common::format_ustring_for_type,
    rust_icu_common::generalized_fallible_getter,
    rust_icu_common::generalized_fallible_setter,
    rust_icu_common::simple_drop_impl,
    rust_icu_sys as sys,
    rust_icu_sys::versioned_function,
    rust_icu_uformattable as uformattable, rust_icu_uloc as uloc, rust_icu_ustring as ustring,
    rust_icu_ustring::buffered_uchar_method_with_retry,
    std::{convert::TryFrom, convert::TryInto, ptr},
};

/// Generates a getter and setter method for a simple attribute with a value
/// of the specified type.
///
/// ```rust ignore
/// impl _ {
///   attribute!(attribute, Attribute, i32, get_prefix, set_prefix)
/// }
/// ```
///
/// generates:
///
/// ```rust ignore
/// impl _ {
///   get_attribute(&self, key: Attribute) -> i32;
///   set_attribute(&self, key: Attribute, value: i32);
/// }
/// ```
///
/// out of functions:
///
/// ```c++ ignore
/// unum_getAttribute(const UNumberFormat* fmt, UNumberFormatAttribute attr);
/// unum_setAttribute(
///     const UNumberFormat* fmt,
///     UNumberFormatAttribute attr,
///     double newValue);
/// ```
macro_rules! attribute{
    ($method_name:ident, $original_method_name:ident, $type_name:ty) => (

        paste::item! {
            #[doc = concat!("Implements `", stringify!($original_method_name), "`. Since 0.3.1.")]
            pub fn [< get_ $method_name >](&self, attr: sys::UNumberFormatAttribute) -> $type_name {
                unsafe {
                    versioned_function!([< unum_get $original_method_name >])(self.rep.as_ptr(), attr)
                }
            }
            #[doc = concat!("Implements `", stringify!($original_method_name), "`. Since 0.3.1.")]
            pub fn [< set_ $method_name >](&mut self, attr: sys::UNumberFormatAttribute, value: $type_name) {
                unsafe {
                    versioned_function!([< unum_set $original_method_name >])(self.rep.as_ptr(), attr, value)
                }
            }
        }

    )
}

/// The struct for number formatting.
#[derive(Debug)]
pub struct UNumberFormat {
    rep: ptr::NonNull<sys::UNumberFormat>,
}

simple_drop_impl!(UNumberFormat, unum_close);

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
    pub fn format_double_currency(
        &self,
        number: f64,
        currency: &str,
    ) -> Result<String, common::Error> {
        let currency = ustring::UChar::try_from(currency)?;
        let result = self.format_double_currency_ustring(number, &currency)?;
        String::try_from(&result)
    }

    /// Implements `unum_formatDoubleCurrency`. Since 0.3.1
    pub fn format_double_currency_ustring(
        &self,
        number: f64,
        currency: &ustring::UChar,
    ) -> Result<ustring::UChar, common::Error> {
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

    /// Implements `unum_parseToUFormattable`. Since 0.3.1.
    ///
    /// > **WARNING** the `parse_position` parameter is with respect to the number index
    /// in the `UChar` string.  This won't work exactly for multibyte UTF8 values of
    /// `text`.  If you think you will have multibyte values, use instead
    /// [UNumberFormat::parse_to_formattable_ustring].
    pub fn parse_to_formattable<'a>(
        &'a self,
        text: &str,
        parse_position: Option<i32>,
    ) -> Result<uformattable::UFormattable<'a>, common::Error> {
        let ustr = ustring::UChar::try_from(text)?;
        self.parse_to_formattable_ustring(&ustr, parse_position)
    }

    /// Implements `unum_parseToUFormattable`. Since 0.3.1.
    pub fn parse_to_formattable_ustring<'a>(
        &'a self,
        text: &ustring::UChar,
        parse_position: Option<i32>,
    ) -> Result<uformattable::UFormattable<'a>, common::Error> {
        let mut fmt = uformattable::UFormattable::try_new()?;
        let mut status = common::Error::OK_CODE;
        let mut pos = parse_position.unwrap_or(0);
        unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(unum_parseToUFormattable)(
                self.rep.as_ptr(),
                fmt.as_mut_ptr(),
                text.as_c_ptr(),
                text.len() as i32,
                &mut pos,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        Ok(fmt)
    }

    /// Implements `unum_formatUFormattable`. Since 0.3.1.
    pub fn format_formattable<'a>(
        &self,
        fmt: &uformattable::UFormattable<'a>,
    ) -> Result<String, common::Error> {
        let result = self.format_formattable_ustring(fmt)?;
        String::try_from(&result)
    }

    /// Implements `unum_formatUFormattable`. Since 0.3.1.
    pub fn format_formattable_ustring<'a>(
        &self,
        fmt: &uformattable::UFormattable<'a>,
    ) -> Result<ustring::UChar, common::Error> {
        const CAPACITY: usize = 200;
        buffered_uchar_method_with_retry!(
            format_formattable_impl,
            CAPACITY,
            [
                format: *const sys::UNumberFormat,
                fmt: *const sys::UFormattable,
            ],
            [pos: *mut sys::UFieldPosition,]
        );

        format_formattable_impl(
            versioned_function!(unum_formatUFormattable),
            self.rep.as_ptr(),
            fmt.as_ptr(),
            0 as *mut sys::UFieldPosition,
        )
    }

    // Implements `unum_getAttribute`. Since 0.3.1.
    attribute!(attribute, Attribute, i32);

    // Implements `unum_getDoubleAttribute`. Since 0.3.1.
    attribute!(double_attribute, DoubleAttribute, f64);

    /// Implements `unum_getTextAttribute`. Since 0.3.1.
    pub fn get_text_attribute(
        &self,
        tag: sys::UNumberFormatTextAttribute,
    ) -> Result<String, common::Error> {
        let result = self.get_text_attribute_ustring(tag)?;
        String::try_from(&result)
    }

    /// Implements `unum_getTextAttribute`. Since 0.3.1.
    pub fn get_text_attribute_ustring(
        &self,
        tag: sys::UNumberFormatTextAttribute,
    ) -> Result<ustring::UChar, common::Error> {
        const CAPACITY: usize = 200;
        buffered_uchar_method_with_retry!(
            get_text_attribute_impl,
            CAPACITY,
            [
                rep: *const sys::UNumberFormat,
                tag: sys::UNumberFormatTextAttribute,
            ],
            []
        );
        get_text_attribute_impl(
            versioned_function!(unum_getTextAttribute),
            self.rep.as_ptr(),
            tag,
        )
    }

    /// Implements `unum_setTextAttribute`. Since 0.3.1.
    pub fn set_text_attribute(
        &mut self,
        tag: sys::UNumberFormatTextAttribute,
        new_value: &str,
    ) -> Result<(), common::Error> {
        let new_value = ustring::UChar::try_from(new_value)?;
        self.set_text_attribute_ustring(tag, &new_value)?;
        Ok(())
    }

    /// Implements `unum_setTextAttribute`. Since 0.3.1.
    pub fn set_text_attribute_ustring(
        &mut self,
        tag: sys::UNumberFormatTextAttribute,
        new_value: &ustring::UChar,
    ) -> Result<(), common::Error> {
        let mut status = common::Error::OK_CODE;
        unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(unum_setTextAttribute)(
                self.rep.as_ptr(),
                tag,
                new_value.as_c_ptr(),
                new_value.len() as i32,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        Ok(())
    }

    /// Implements `unum_toPattern`. Since 0.3.1.
    pub fn get_pattern(&self, is_localized: bool) -> Result<String, common::Error> {
        let result = self.get_pattern_ustring(is_localized)?;
        String::try_from(&result)
    }

    /// Implements `unum_toPattern`. Since 0.3.1.
    pub fn get_pattern_ustring(&self, is_localized: bool) -> Result<ustring::UChar, common::Error> {
        const CAPACITY: usize = 200;
        buffered_uchar_method_with_retry!(
            get_pattern_ustring_impl,
            CAPACITY,
            [rep: *const sys::UNumberFormat, is_localized: sys::UBool,],
            []
        );
        let result = get_pattern_ustring_impl(
            versioned_function!(unum_toPattern),
            self.rep.as_ptr(),
            is_localized as sys::UBool,
        );
        result
    }

    /// Implements `unum_getSymbol`. Since 0.3.1.
    pub fn get_symbol(&self, symbol: sys::UNumberFormatSymbol) -> Result<String, common::Error> {
        let result = self.get_symbol_ustring(symbol)?;
        String::try_from(&result)
    }

    /// Implements `unum_getSymbol`. Since 0.3.1.
    pub fn get_symbol_ustring(
        &self,
        symbol: sys::UNumberFormatSymbol,
    ) -> Result<ustring::UChar, common::Error> {
        const CAPACITY: usize = 200;
        buffered_uchar_method_with_retry!(
            get_symbol_impl,
            CAPACITY,
            [
                rep: *const sys::UNumberFormat,
                symbol: sys::UNumberFormatSymbol,
            ],
            []
        );
        get_symbol_impl(
            versioned_function!(unum_getSymbol),
            self.rep.as_ptr(),
            symbol,
        )
    }

    /// Implements `unum_setSymbol`. Since 0.3.1.
    pub fn set_symbol(
        &mut self,
        symbol: sys::UNumberFormatSymbol,
        value: &str,
    ) -> Result<(), common::Error> {
        let value = ustring::UChar::try_from(value)?;
        self.set_symbol_ustring(symbol, &value)
    }

    /// Implements `unum_setSymbol`. Since 0.3.1.
    pub fn set_symbol_ustring(
        &mut self,
        symbol: sys::UNumberFormatSymbol,
        value: &ustring::UChar,
    ) -> Result<(), common::Error> {
        let mut status = common::Error::OK_CODE;
        unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(unum_setSymbol)(
                self.rep.as_ptr(),
                symbol,
                value.as_c_ptr(),
                value.len() as i32,
                &mut status,
            );
        };
        common::Error::ok_or_warning(status)?;
        Ok(())
    }

    /// Implements `unum_getLocaleByType`. Since 0.3.1.
    pub fn get_locale_by_type<'a>(
        &'a self,
        data_loc_type: sys::ULocDataLocaleType,
    ) -> Result<&'a str, common::Error> {
        let mut status = common::Error::OK_CODE;
        let cptr = unsafe {
            assert!(common::Error::is_ok(status));
            let raw = versioned_function!(unum_getLocaleByType)(
                self.rep.as_ptr(),
                data_loc_type,
                &mut status,
            );
            std::ffi::CStr::from_ptr(raw)
        };
        common::Error::ok_or_warning(status)?;
        cptr.to_str().map_err(|e| e.into())
    }

    // Implements `unum_getContext`. Since 0.3.1.
    generalized_fallible_getter!(
        get_context,
        unum_getContext,
        [context_type: sys::UDisplayContextType,],
        sys::UDisplayContext
    );

    // Implements `unum_setContext`. Since 0.3.1.
    generalized_fallible_setter!(set_context, unum_setContext, [value: sys::UDisplayContext,]);
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
    /// Try creating a new iterator, based on data supplied by `owner`.
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
    ///
    /// **DO NOT USE UNLESS IMPLEMENTING LOW-LEVEL ICU4C INTERFACE**.
    #[doc(hidden)]
    pub fn as_mut_ptr(&mut self) -> *mut sys::UFieldPositionIterator {
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

/// Gets an iterator over all available formatting locales.
///
/// Implements `unum_getAvailable`. Since 0.3.1.
pub fn available_iter() -> UnumIter {
    let max = get_num_available();
    UnumIter { max, next: 0 }
}

fn get_num_available() -> usize {
    let result = unsafe { versioned_function!(unum_countAvailable)() } as usize;
    result
}

/// An iterator returned by `available_iter()`, containing the string representation of locales for
/// which formatting is available.
pub struct UnumIter {
    /// The total number of elements that this iterator can yield.
    max: usize,
    /// The next element index that can be yielded.
    next: usize,
}

impl Iterator for UnumIter {
    type Item = String;

    /// Yields the next available locale identifier per the currently loaded locale data.
    ///
    /// Example values: `en_US`, `rs`, `ru_RU`.
    fn next(&mut self) -> Option<String> {
        if self.max == 0 {
            return None;
        }
        if self.max != get_num_available() {
            // If the number of available locales changed while we were iterating this list, the
            // locale set may have been reloaded.  Return early to avoid indexing beyond limits.
            // This may return weird results, but won't crash the program.
            return None;
        }
        if self.next >= self.max {
            return None;
        }
        let cptr: *const std::os::raw::c_char =
            unsafe { versioned_function!(unum_getAvailable)(self.next as i32) };
        // This assertion could happen in theory if the locale data is invalidated as this iterator
        // is being executed.  I am unsure how that can be prevented.
        assert_ne!(
            cptr,
            std::ptr::null(),
            "unum_getAvailable unexpectedly returned nullptr"
        );
        let cstr = unsafe { std::ffi::CStr::from_ptr(cptr) };
        self.next = self.next + 1;
        Some(cstr.to_str().expect("can be converted to str").to_string())
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
        }

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
        }

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
        }

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
        }

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
        }

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

            let s = fmt.format_decimal(test.number).expect("format success");

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
        }

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

    #[test]
    fn format_and_parse_uformattable() {
        #[derive(Debug)]
        struct TestCase {
            source_locale: &'static str,
            number: &'static str,
            position: Option<i32>,
            style: sys::UNumberFormatStyle,

            target_locale: &'static str,
            expected: &'static str,
        }

        let tests = vec![
            TestCase {
                source_locale: "sr-RS",
                number: "123,44",
                position: None,
                style: sys::UNumberFormatStyle::UNUM_DECIMAL,

                target_locale: "en-US",
                expected: "123.44",
            },
            TestCase {
                source_locale: "sr-RS",
                number: "123,44",
                position: Some(2),
                style: sys::UNumberFormatStyle::UNUM_DECIMAL,

                target_locale: "en-US",
                expected: "3.44",
            },
        ];
        for test in tests {
            let locale = uloc::ULoc::try_from(test.source_locale).expect("locale exists");
            let fmt = crate::UNumberFormat::try_new_with_style(test.style, &locale)
                .expect("source_locale formatter");

            let formattable = fmt
                .parse_to_formattable(test.number, test.position)
                .expect(&format!("parse_to_formattable: {:?}", &test));

            let locale = uloc::ULoc::try_from(test.target_locale).expect("locale exists");
            let fmt = crate::UNumberFormat::try_new_with_style(test.style, &locale)
                .expect("target_locale formatter");

            let result = fmt
                .format_formattable(&formattable)
                .expect(&format!("format_formattable: {:?}", &test));

            assert_eq!(test.expected, result);
        }
    }

    #[test]
    fn test_available() {
        // Since the locale list is variable, we can not test for exact locales, but
        // we count them and make a sample to ensure sanity.
        let all = super::available_iter().collect::<Vec<String>>();
        let count = super::available_iter().count();
        assert_ne!(
            0, count,
            "there should be at least some available locales: {:?}",
            &all
        );
        let available = all
            .into_iter()
            .filter(|f| *f == "en_US")
            .collect::<Vec<String>>();
        assert_eq!(
            vec!["en_US"],
            available,
            "missing a locale that likely should be there"
        );
    }

    #[test]
    fn pattern() {
        #[derive(Debug)]
        struct TestCase {
            source_locale: &'static str,
            is_localized: bool,
            style: sys::UNumberFormatStyle,

            expected: &'static str,
        }

        let tests = vec![
            TestCase {
                source_locale: "en-US",
                is_localized: true,
                style: sys::UNumberFormatStyle::UNUM_DECIMAL,
                expected: "#,##0.###",
            },
            TestCase {
                source_locale: "sr-RS",
                is_localized: true,
                style: sys::UNumberFormatStyle::UNUM_DECIMAL,
                expected: "#.##0,###",
            },
            // TODO(https://github.com/google/rust_icu/issues/203): Figure out how to re-enable. I
            // don't like the prospect of introducing a new feature flag just to handle this.
            //TestCase {
                //source_locale: "sr-RS",
                //is_localized: false,
                //style: sys::UNumberFormatStyle::UNUM_DECIMAL,
                //expected: "#,##1.###",
            //},
        ];
        for test in tests {
            let locale = uloc::ULoc::try_from(test.source_locale).expect("locale exists");
            let fmt = crate::UNumberFormat::try_new_with_style(test.style, &locale)
                .expect("source_locale formatter");

            let pattern = fmt
                .get_pattern(test.is_localized)
                .expect(&format!("localized in test: {:?}", test));
            assert_eq!(test.expected, pattern, "in test: {:?}", test);
        }
    }
}
