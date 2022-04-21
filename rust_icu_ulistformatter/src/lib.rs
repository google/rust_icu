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

//! # ICU list formatting support for rust
//!
//! This crate provides locale-sensitive list formatting, based on the list
//! formatting as implemente by the ICU library.  Specifically, the functionality
//! exposed through its C API, as available in the [header `ulisetformatter.h`][header].
//!
//!   [header]: https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/ulistformatter_8h.html
//!
//! > Are you missing some features from this crate?  Consider [reporting an
//! issue](https://github.com/google/rust_icu/issues) or even [contributing the
//! functionality](https://github.com/google/rust_icu/pulls).
//!
//! ## Examples
//!
//! > TBD

use {
    rust_icu_common as common, rust_icu_sys as sys,
    rust_icu_sys::versioned_function,
    rust_icu_sys::*,
    rust_icu_ustring as ustring,
    std::{convert::TryFrom, convert::TryInto, ffi, ptr},
};

#[derive(Debug)]
pub struct UListFormatter {
    rep: ptr::NonNull<sys::UListFormatter>,
}

impl Drop for UListFormatter {
    /// Implements ulistfmt_close`.
    fn drop(&mut self) {
        unsafe { versioned_function!(ulistfmt_close)(self.rep.as_ptr()) };
    }
}

impl UListFormatter {
    /// Implements ulistfmt_open`.
    pub fn try_new(locale: &str) -> Result<UListFormatter, common::Error> {
        let locale_cstr = ffi::CString::new(locale)?;
        let mut status = common::Error::OK_CODE;
        // Unsafety note: this is the way to create the formatter.  We expect all
        // the passed-in values to be well-formed.
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ulistfmt_open)(locale_cstr.as_ptr(), &mut status)
                as *mut sys::UListFormatter
        };
        common::Error::ok_or_warning(status)?;
        Ok(UListFormatter {
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Implements `ulistfmt_openForType`.  Since ICU 67.
    #[cfg(feature = "icu_version_67_plus")]
    pub fn try_new_styled(
        locale: &str,
        format_type: sys::UListFormatterType,
        format_width: sys::UListFormatterWidth,
    ) -> Result<UListFormatter, common::Error> {
        let locale_cstr = ffi::CString::new(locale)?;
        let mut status = common::Error::OK_CODE;
        // Unsafety note: this is the way to create the formatter.  We expect all
        // the passed-in values to be well-formed.
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ulistfmt_openForType)(
                locale_cstr.as_ptr(),
                format_type,
                format_width,
                &mut status,
            ) as *mut sys::UListFormatter
        };
        common::Error::ok_or_warning(status)?;
        Ok(UListFormatter {
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Implements `ulistfmt_format`.
    pub fn format(&self, list: &[&str]) -> Result<String, common::Error> {
        let result = self.format_uchar(list)?;
        String::try_from(&result)
    }

    /// Implements `ulistfmt_format`.
    // TODO: this method call is repetitive, and should probably be pulled out into a macro.
    // TODO: rename this function into format_uchar.
    pub fn format_uchar(&self, list: &[&str]) -> Result<ustring::UChar, common::Error> {
        let list_ustr = UCharArray::try_from(list)?;
        const CAPACITY: usize = 200;
        let (pointers, strlens, len) = unsafe { list_ustr.as_pascal_strings() };

        // This is similar to buffered_string_method_with_retry, except the buffer
        // consists of [sys::UChar]s.
        let mut status = common::Error::OK_CODE;
        let mut buf: Vec<sys::UChar> = vec![0; CAPACITY];

        let full_len: i32 = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ulistfmt_format)(
                self.rep.as_ptr(),
                pointers as *const *const sys::UChar,
                strlens as *const i32,
                len as i32,
                buf.as_mut_ptr(),
                CAPACITY as i32,
                &mut status,
            )
        };
        if status == sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR
            || (common::Error::is_ok(status)
                && full_len > CAPACITY.try_into().map_err(|e| common::Error::wrapper(e))?)
        {
            status = common::Error::OK_CODE;
            assert!(full_len > 0);
            let full_len: usize = full_len.try_into().map_err(|e| common::Error::wrapper(e))?;
            buf.resize(full_len, 0);
            unsafe {
                assert!(common::Error::is_ok(status), "status: {:?}", status);
                versioned_function!(ulistfmt_format)(
                    self.rep.as_ptr(),
                    pointers as *const *const sys::UChar,
                    strlens as *const i32,
                    len as i32,
                    buf.as_mut_ptr(),
                    buf.len() as i32,
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

/// A helper array that deconstructs [ustring::UChar] into constituent raw parts
/// that can be passed into formatting functions that use array APIs for parameter-passing.
///
/// Create with [UCharArray::try_from], and then use [UCharArray::as_pascal_strings] to get
/// the respective sizes.
#[derive(Debug)]
struct UCharArray {
    // The elements of the array.
    elements: Vec<ustring::UChar>,
    // Pointers to the respective elements.
    pointers: Vec<*const sys::UChar>,
    // The string lengths (in ustring::UChar units) of respective elements.
    // These strlens are what `listfmt_format` expects, so can't be `usize` but
    // *must* be `i32`.
    strlens: Vec<i32>,
}

impl<T> TryFrom<&[T]> for UCharArray
where
    T: AsRef<str>,
{
    type Error = common::Error;

    /// Creates a new [UCharArray] from an array of string-like types.
    fn try_from(list: &[T]) -> Result<Self, Self::Error> {
        let mut elements: Vec<ustring::UChar> = Vec::with_capacity(list.len());
        for element in list {
            let uchar = ustring::UChar::try_from(element.as_ref())?;
            elements.push(uchar);
        }
        let pointers = elements.iter().map(|e| e.as_c_ptr()).collect();
        let strlens = elements.iter().map(|e| e.len() as i32).collect();
        Ok(UCharArray {
            elements,
            pointers,
            strlens,
        })
    }
}

impl AsRef<Vec<ustring::UChar>> for UCharArray {
    fn as_ref(&self) -> &Vec<ustring::UChar> {
        &self.elements
    }
}

impl UCharArray {
    /// Returns the elements of the array decomposed as "pascal strings", i.e.
    /// separating out the pointers, the sizes of each individual string included,
    /// and the total size of the array.
    pub unsafe fn as_pascal_strings(self) -> (*mut *mut sys::UChar, *mut *mut i32, usize) {
        let pointers = self.pointers.as_ptr() as *mut *mut sys::UChar;
        let strlens = self.strlens.as_ptr() as *mut *mut i32;
        let len = self.elements.len();
        // Since 'self' was not moved anywhere in this method, we need to forget it before we
        // return pointers to its content, else all the pointers will be invalidated.
        std::mem::forget(self);
        (pointers, strlens, len)
    }

    /// Returns the number of elements in the array.
    #[allow(dead_code)] // Not used in production code yet.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Assembles an [UCharArray] from parts (ostensibly, obtained through
    /// [UCharArray::as_pascal_strings] above.  Unsafe, as there is no guarantee
    /// that the pointers are well-formed.
    ///
    /// Takes ownership away from the pointers and strlens.  Requires that
    /// `len` is equal to the capacities and lengths of the vectors described by
    /// `pointers` and `strlens`.
    #[allow(dead_code)]
    pub unsafe fn from_raw_parts(
        pointers: *mut *mut sys::UChar,
        strlens: *mut *mut i32,
        len: usize,
    ) -> UCharArray {
        let pointers_vec: Vec<*mut sys::UChar> = Vec::from_raw_parts(pointers, len, len);
        let strlens_vec: Vec<i32> = Vec::from_raw_parts(strlens as *mut i32, len, len);

        let elements = pointers_vec.into_iter().zip(strlens_vec);
        let elements = elements
            .map(|(ptr, len): (*mut sys::UChar, i32)| {
                assert!(len >= 0);
                let len_i32 = len as usize;
                let raw: Vec<sys::UChar> = Vec::from_raw_parts(ptr, len_i32, len_i32);
                ustring::UChar::from(raw)
            })
            .collect::<Vec<ustring::UChar>>();
        let pointers = elements.iter().map(|e| e.as_c_ptr()).collect();
        let strlens = elements.iter().map(|e| e.len() as i32).collect();

        UCharArray {
            elements,
            pointers,
            strlens,
        }
    }
}

#[cfg(test)]
mod testing {
    use crate::*;

    #[test]
    fn test_pascal_strings() {
        let array = UCharArray::try_from(&["eenie", "meenie", "minie", "moe"][..])
            .expect("created with success");
        let array_len = array.len();
        let (strings, strlens, len) = unsafe { array.as_pascal_strings() };
        assert_eq!(len, array_len);

        let reconstructed = unsafe { UCharArray::from_raw_parts(strings, strlens, len) };
        let result = reconstructed
            .as_ref()
            .iter()
            .map(|e| String::try_from(e).expect("conversion is a success"))
            .collect::<Vec<String>>();
        assert_eq!(vec!["eenie", "meenie", "minie", "moe"], result);
    }

    #[test]
    fn test_formatting() {
        let array = ["eenie", "meenie", "minie", "moe"];
        let formatter = crate::UListFormatter::try_new("en-US").expect("has list format");
        let result = formatter.format(&array).expect("formatting succeeds");
        assert_eq!("eenie, meenie, minie, and moe", result);
    }

    #[test]
    fn test_formatting_sr() {
        let array = ["Раја", "Гаја", "Влаја"]; // Huey, Dewey, and Louie.
        let formatter = crate::UListFormatter::try_new("sr-RS").expect("has list format");
        let result = formatter.format(&array).expect("formatting succeeds");
        assert_eq!("Раја, Гаја и Влаја", result);
    }

    #[test]
    #[cfg(feature = "icu_version_67_plus")]
    fn test_formatting_styled() {
        let array = ["Раја", "Гаја", "Влаја"];
        let formatter = crate::UListFormatter::try_new_styled(
            "sr-RS",
            sys::UListFormatterType::ULISTFMT_TYPE_OR,
            sys::UListFormatterWidth::ULISTFMT_WIDTH_WIDE,
        )
        .expect("has list format");
        let result = formatter.format(&array).expect("formatting succeeds");
        assert_eq!("Раја, Гаја или Влаја", result);
    }
}
