// Copyright 2023 Google LLC
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

//! # ICU character set detection support for rust
//!
//! This crate provides character set detection based on the detection
//! functions implemented by the ICU library, specifically in
//! the [header `ucsdet.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/ucsdet_8h.html).
//!
//! This crate provides two main type: [CharsetDetector] and [CharsetMatch],
//! [CharsetDetector] can detect text and return a [CharsetMatch].
//!
//! For more information on ICU character set detection, please see also:
//! [character set detection documentation on the ICU user guide](https://unicode-org.github.io/icu/userguide/conversion/detection.html).
//!
//! > Are you missing some features from this crate?  Consider [reporting an
//! issue](https://github.com/google/rust_icu/issues) or even [contributing the
//! functionality](https://github.com/google/rust_icu/pulls).

use {
    common::simple_drop_impl,
    rust_icu_common as common, rust_icu_sys as sys,
    rust_icu_sys::versioned_function,
    rust_icu_uenum::Enumeration,
    std::{
        ffi::CStr,
        marker::PhantomData,
        mem::transmute,
        ptr::{self, NonNull},
    },
};

#[allow(unused_imports)]
use sys::*;

/// This interface wraps around icu4c `UCharsetDetector`.
#[derive(Debug)]
pub struct CharsetDetector<'detector> {
    rep: ptr::NonNull<sys::UCharsetDetector>,
    _marker: PhantomData<&'detector ()>,
}

simple_drop_impl!(CharsetDetector<'_>, ucsdet_close);

impl<'detector> CharsetDetector<'detector> {
    /// Try to create a charset detector
    pub fn new() -> Result<CharsetDetector<'detector>, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let rep = unsafe { versioned_function!(ucsdet_open)(&mut status) };
        common::Error::ok_or_warning(status)?;
        let result = CharsetDetector {
            rep: ptr::NonNull::new(rep).unwrap(),
            _marker: PhantomData,
        };
        Ok(result)
    }

    /// Get an iterator over the set of all detectable charsets
    ///
    /// See also: [Enumeration]
    pub fn available_charsets(&self) -> Result<Enumeration, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let raw_enum = unsafe {
            versioned_function!(ucsdet_getAllDetectableCharsets)(self.rep.as_ptr(), &mut status)
        };
        common::Error::ok_or_warning(status)?;
        assert!(!raw_enum.is_null());
        Ok(unsafe { Enumeration::from_raw_parts(None, raw_enum) })
    }

    /// Set text for detection
    ///
    /// `text` should live longer than [self].
    pub fn set_text<'b: 'detector>(&mut self, text: &'b [u8]) -> Result<(), common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        unsafe {
            versioned_function!(ucsdet_setText)(
                self.rep.as_ptr(),
                text.as_ptr() as *const i8,
                text.len() as i32,
                &mut status,
            );
        }
        common::Error::ok_or_warning(status)
    }

    /// Set the declared encoding for the charset detection.
    pub fn set_declared_encoding(&self, encoding: &str) -> Result<(), common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        unsafe {
            versioned_function!(ucsdet_setDeclaredEncoding)(
                self.rep.as_ptr(),
                encoding.as_ptr() as *const i8,
                encoding.len() as i32,
                &mut status,
            );
        }
        common::Error::ok_or_warning(status)
    }

    /// Return the charset that best matches the supplied input data.
    pub fn detect(&self) -> Result<CharsetMatch, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let charset_match =
            unsafe { versioned_function!(ucsdet_detect)(self.rep.as_ptr(), &mut status) };
        common::Error::ok_or_warning(status)?;
        Ok(CharsetMatch {
            rep: NonNull::new(charset_match as *mut _).unwrap(),
            contrain: PhantomData,
        })
    }

    /// Find all charset matches that appear to be consistent
    /// with the input, returning an array of results.
    ///
    /// The results are ordered with the best quality match first.
    ///
    /// Returns `&[CharsetMatch]` if no error.
    pub fn detect_all(&self) -> Result<&[CharsetMatch], common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let mut len = 0;
        let charset_match = unsafe {
            versioned_function!(ucsdet_detectAll)(self.rep.as_ptr(), &mut len, &mut status)
        };
        common::Error::ok_or_warning(status)?;
        if len == 0 {
            return Ok(&[]);
        }
        let slice = unsafe { std::slice::from_raw_parts_mut(charset_match, len as usize) };
        for ele in slice.iter() {
            assert!(!ele.is_null())
        }
        // SAFETY: `CharsetMatch` is marked as `#[repr(transparent)]`, and
        // all pointers are checked non-null pointer, so it's safe to transmute
        let slice: &[CharsetMatch] = unsafe { transmute(slice) };
        Ok(slice)
    }

    /// Returns the previous setting
    pub fn set_input_filter(&self, enable: bool) -> Result<bool, common::Error> {
        let charset_match = unsafe {
            versioned_function!(ucsdet_enableInputFilter)(self.rep.as_ptr(), enable as i8)
        };
        Ok(charset_match == 1)
    }

    /// Enable/disable input filter
    pub fn input_filter_enabled(&self) -> bool {
        let charset_match =
            unsafe { versioned_function!(ucsdet_isInputFilterEnabled)(self.rep.as_ptr()) };
        charset_match == 1
    }
}

/// Representing a match that was identified from a charset detection operation
///
/// Owned by [CharsetDetector]
#[repr(transparent)]
pub struct CharsetMatch<'detector> {
    rep: ptr::NonNull<sys::UCharsetMatch>,
    contrain: PhantomData<&'detector ()>,
}

impl<'charset> ::core::fmt::Debug for CharsetMatch<'charset> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        f.debug_struct("CharsetMatch")
            .field("name", &self.name())
            .field("language", &self.language())
            .field("confidence", &self.confidence())
            .finish()
    }
}

impl<'charset> CharsetMatch<'charset> {
    /// Returns the name of the charset
    pub fn name(&self) -> Result<&str, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let raw_name = unsafe {
            let ptr = versioned_function!(ucsdet_getName)(self.rep.as_ptr(), &mut status);
            common::Error::ok_or_warning(status)?;
            assert!(!ptr.is_null());
            CStr::from_ptr(ptr)
        };
        Ok(raw_name.to_str()?)
    }

    /// Returns the RFC 3066 code for the language of the charset
    pub fn language(&self) -> Result<&str, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let raw_name = unsafe {
            let ptr = versioned_function!(ucsdet_getLanguage)(self.rep.as_ptr(), &mut status);
            common::Error::ok_or_warning(status)?;
            assert!(!ptr.is_null());
            CStr::from_ptr(ptr)
        };
        Ok(raw_name.to_str()?)
    }

    /// Get a confidence number for the quality of the match
    pub fn confidence(&self) -> Result<i32, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let confidence =
            unsafe { versioned_function!(ucsdet_getConfidence)(self.rep.as_ptr(), &mut status) };
        Ok(confidence)
    }

    /// Get the entire input text as a UChar string, placing it into
    /// a caller-supplied buffer
    pub fn get_uchars(&self, buf: &mut [sys::UChar]) -> Result<usize, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let len = unsafe {
            versioned_function!(ucsdet_getUChars)(
                self.rep.as_ptr(),
                buf.as_mut_ptr(),
                buf.len() as i32,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        Ok(len as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_charsets() {
        let ucsd = CharsetDetector::new().unwrap();
        for ele in ucsd.available_charsets().unwrap() {
            assert!(ele.unwrap().is_ascii())
        }
    }

    #[test]
    fn test_charset_detect_shift_jis() {
        let mut ucsd = CharsetDetector::new().unwrap();
        const SHIFT_JIS_STRING: &[u8] = &[
            0x82, 0xB1, 0x82, 0xF1, 0x82, 0xCE, 0x82, 0xF1, 0x82, 0xCD, 0x95, 0xBD, 0x89, 0xBC,
            0x96, 0xBC, 0x82, 0xB1, 0x82, 0xF1, 0x82, 0xCE, 0x82, 0xF1, 0x82, 0xCD, 0x95, 0xBD,
            0x89, 0xBC, 0x96, 0xBC, 0x82, 0xB1, 0x82, 0xF1, 0x82, 0xCE, 0x82, 0xF1, 0x82, 0xCD,
            0x95, 0xBD, 0x89, 0xBC, 0x96, 0xBC,
        ];
        ucsd.set_text(&SHIFT_JIS_STRING).unwrap();
        let detected = ucsd.detect().unwrap();
        assert_eq!(detected.name().unwrap(), "Shift_JIS");
        assert_eq!(detected.language().unwrap(), "ja");
        assert!(detected.confidence().unwrap() > 80);
    }

    #[test]
    fn test_charset_detect_all_utf8() {
        let mut ucsd = CharsetDetector::new().unwrap();
        ucsd.set_text(b"Hello World UTF-8").unwrap();
        let detected = ucsd.detect_all().unwrap();
        assert!(detected
            .into_iter()
            .any(|charset| charset.name().unwrap() == "UTF-8"))
    }
}
