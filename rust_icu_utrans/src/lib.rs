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

//! # ICU transliteration support for Rust
//!
//! This crate provides a Rust implementation of the ICU transliteration APIs in `utrans.h`.
//!
//! ## Examples
//!
//! Sample code use is given below.
//!
//! ICU includes a number of built-in transliterators, which can be listed with
//! [`get_ids`](#method.get_ids).
//! Transliterators may be combined to form a sequence of text transformations by provding a
//! compound identifier as shown below.
//!
//! ```rust
//! use rust_icu_sys as sys;
//! use rust_icu_utrans as utrans;
//!
//! let compound_id = "NFC; Latin; Latin-ASCII";
//! let ascii_trans = utrans::UTransliterator::new(
//!     compound_id, None, sys::UTransDirection::UTRANS_FORWARD).unwrap();
//! assert_eq!(ascii_trans.transliterate("литература").unwrap(), "literatura");
//! ```
//!
//! It is also possible to define your own transliterators by providing a set of rules to
//! [`new`](#method.new).
//!
//! ```rust
//! use rust_icu_sys as sys;
//! use rust_icu_utrans as utrans;
//!
//! let id = "Coffee";
//! let rules = r"a > o; f > ff; \u00e9 > ee;";
//! let custom_trans = utrans::UTransliterator::new(
//!     id, Some(rules), sys::UTransDirection::UTRANS_FORWARD).unwrap();
//! assert_eq!(custom_trans.transliterate("caf\u{00e9}").unwrap(), "coffee");
//! ```
//!
//! See the [ICU user guide](https://unicode-org.github.io/icu/userguide/transforms/general/)
//! and the C API documentation in the
//! [`utrans.h` header](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/utrans_8h.html)
//! for details.

use {
    rust_icu_common::{self as common, simple_drop_impl},
    rust_icu_sys::{self as sys, *},
    rust_icu_uenum as uenum, rust_icu_ustring as ustring,
    std::{convert::TryFrom, ptr, slice},
};

/// Rust wrapper for the ICU `UTransliterator` type.
#[derive(Debug)]
pub struct UTransliterator {
    rep: ptr::NonNull<sys::UTransliterator>,
}

/// Implements `utrans_close`.
simple_drop_impl!(UTransliterator, utrans_close);

impl Clone for UTransliterator {
    /// Implements `utrans_clone`.
    fn clone(&self) -> Self {
        UTransliterator {
            rep: self.rep.clone(),
        }
    }
}

impl UTransliterator {
    /// Returns an enumeration containing the identifiers of all available
    /// transliterators.
    ///
    /// Implements `utrans_openIDs`.
    pub fn get_ids() -> Result<uenum::Enumeration, common::Error> {
        let mut status = common::Error::OK_CODE;
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(utrans_openIDs)(&mut status)
        };
        common::Error::ok_or_warning(status)?;
        assert_ne!(rep, 0 as *mut sys::UEnumeration);
        // utrans_openIDs returns a pointer to an ICU UEnumeration, which we
        // are now responsible for closing, so we wrap it in its corresponding
        // rust_icu type.
        let ids = unsafe { uenum::Enumeration::from_raw_parts(None, rep) };
        Ok(ids)
    }

    /// Consumes `trans` and registers it with the underlying ICU system.
    /// A transliterator that has been registered with the system can be
    /// retrieved by calling [`new`](#method.new) with its identifier.
    ///
    /// Implements `utrans_register`.
    pub fn register(trans: Self) -> Result<(), common::Error> {
        let mut status = common::Error::OK_CODE;
        unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(utrans_register)(
                trans.rep.as_ptr(),
                &mut status,
            )
        }
        common::Error::ok_or_warning(status)?;
        // ICU4C now owns the transliterator and is responsible for closing it,
        // so we avoid dropping a resource we don't own.
        std::mem::forget(trans);
        Ok(())
    }

    /// If rules are given, creates a new transliterator with rules and identifier.
    /// Otherwise, returns the ICU system transliterator with the given identifier.
    ///
    /// Implements `utrans_openU`.
    pub fn new(
        id: &str,
        rules: Option<&str>,
        dir: sys::UTransDirection,
    ) -> Result<Self, common::Error> {
        let id = ustring::UChar::try_from(id)?;
        let rules = match rules {
            Some(s) => Some(ustring::UChar::try_from(s)?),
            None => None,
        };
        Self::new_ustring(&id, rules.as_ref(), dir)
    }

    /// Implements `utrans_openU`.
    pub fn new_ustring(
        id: &ustring::UChar,
        rules: Option<&ustring::UChar>,
        dir: sys::UTransDirection,
    ) -> Result<Self, common::Error> {
        let mut status = common::Error::OK_CODE;
        let mut parse_status = common::NO_PARSE_ERROR.clone();
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(utrans_openU)(
                id.as_c_ptr(),
                id.len() as i32,
                dir,
                rules.map_or(0 as *const sys::UChar, |r| r.as_c_ptr()),
                rules.as_ref().map_or(0, |r| r.len()) as i32,
                &mut parse_status,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        common::parse_ok(parse_status)?;
        assert_ne!(rep, 0 as *mut sys::UTransliterator);
        Ok(Self {
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Returns the identifier for this transliterator.
    ///
    /// Implements `utrans_getUnicodeID`.
    pub fn get_id(&self) -> Result<String, common::Error> {
        let mut id_len: i32 = 0;
        let rep = unsafe {
            versioned_function!(utrans_getUnicodeID)(
                self.rep.as_ptr(),
                &mut id_len,
            )
        };
        assert_ne!(rep, 0 as *const sys::UChar);
        let id_buf =
            unsafe { slice::from_raw_parts(rep, id_len as usize) }.to_vec();
        let id = ustring::UChar::from(id_buf);
        String::try_from(&id)
    }

    /// Returns the inverse of this transliterator, provided that the inverse
    /// has been registered with the underlying ICU system, i.e., a built-in
    /// ICU transliterator or one registered with [`register`](#method.register).
    ///
    /// Implements `utrans_openInverse`.
    pub fn inverse(&self) -> Result<Self, common::Error> {
        let mut status = common::Error::OK_CODE;
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(utrans_openInverse)(
                self.rep.as_ptr(),
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        assert_ne!(rep, 0 as *mut sys::UTransliterator);
        Ok(Self {
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Returns a rules string for this transliterator in the same format
    /// expected by [`new`](#method.new).
    ///
    /// Implements `utrans_toRules`.
    pub fn to_rules(
        &self,
        escape_unprintable: bool,
    ) -> Result<String, common::Error> {
        let mut status = common::Error::OK_CODE;
        // Preflight to determine length of rules text.
        let rules_len = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(utrans_toRules)(
                self.rep.as_ptr(),
                escape_unprintable as sys::UBool,
                0 as *mut sys::UChar,
                0,
                &mut status,
            )
        };
        common::Error::ok_preflight(status)?;
        let mut status = common::Error::OK_CODE;
        let mut rules: Vec<sys::UChar> = vec![0; rules_len as usize];
        unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(utrans_toRules)(
                self.rep.as_ptr(),
                escape_unprintable as sys::UBool,
                rules.as_mut_ptr(),
                rules_len,
                &mut status,
            );
        }
        common::Error::ok_or_warning(status)?;
        let rules = ustring::UChar::from(rules);
        String::try_from(&rules)
    }

    /// Apply a filter to this transliterator, causing certain characters to
    /// pass through untouched. The filter is formatted as a
    /// [UnicodeSet](https://unicode-org.github.io/icu/userguide/strings/unicodeset.html)
    /// string. If the filter is `None`, then any previously-applied filter
    /// is cleared.
    ///
    /// Implements `utrans_setFilter`.
    pub fn set_filter(
        &mut self,
        pattern: Option<&str>,
    ) -> Result<(), common::Error> {
        let pattern = match pattern {
            Some(s) => Some(ustring::UChar::try_from(s)?),
            None => None,
        };
        self.set_filter_ustring(pattern.as_ref())
    }

    /// Implements `utrans_setFilter`.
    pub fn set_filter_ustring(
        &mut self,
        pattern: Option<&ustring::UChar>,
    ) -> Result<(), common::Error> {
        let mut status = common::Error::OK_CODE;
        unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(utrans_setFilter)(
                self.rep.as_ptr(),
                pattern.map_or(0 as *const sys::UChar, |p| p.as_c_ptr()),
                pattern.as_ref().map_or(0, |p| p.len()) as i32,
                &mut status,
            )
        }
        common::Error::ok_or_warning(status)
    }

    /// Returns a string containing the transliterated text.
    ///
    /// Implements `utrans_transUChars`.
    pub fn transliterate(&self, text: &str) -> Result<String, common::Error> {
        let text = ustring::UChar::try_from(text)?;
        let trans_text = self.transliterate_ustring(&text)?;
        String::try_from(&trans_text)
    }

    /// Implements `utrans_transUChars`.
    pub fn transliterate_ustring(
        &self,
        text: &ustring::UChar,
    ) -> Result<ustring::UChar, common::Error> {
        let start: i32 = 0;
        let text_len = text.len() as i32;
        let mut trans_text = text.clone();
        let mut trans_text_len = text_len;
        let mut limit = text_len;
        let mut status = common::Error::OK_CODE;
        // Text is transliterated in place, serving as both the source text and
        // the destination buffer for transliterated text. Because transliteration
        // may produce text that is longer than the source text, we preflight
        // in case of buffer overflow.
        unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(utrans_transUChars)(
                self.rep.as_ptr(),
                trans_text.as_mut_c_ptr(),
                &mut trans_text_len,
                text_len,
                start,
                &mut limit,
                &mut status,
            )
        }
        common::Error::ok_preflight(status)?;
        if trans_text_len > text_len {
            // Transliterated text is longer than source text, so resize buffer
            // and try again.
            trans_text = text.clone();
            trans_text.resize(trans_text_len as usize);
            limit = text_len;
            let mut status = common::Error::OK_CODE;
            let mut length = text_len;
            unsafe {
                assert!(common::Error::is_ok(status));
                versioned_function!(utrans_transUChars)(
                    self.rep.as_ptr(),
                    trans_text.as_mut_c_ptr(),
                    &mut length,
                    trans_text_len as i32,
                    start,
                    &mut limit,
                    &mut status,
                )
            }
            common::Error::ok_or_warning(status)?;
        }
        if trans_text.len() > limit as usize {
            // Transliterated text is shorter than source text, so truncate
            // buffer to new length.
            trans_text.resize(limit as usize);
        }
        Ok(trans_text)
    }

    /// Unregister a transliterator from the underlying ICU system.
    ///
    /// Implements `utrans_unregisterID`.
    pub fn unregister(id: &str) -> Result<(), common::Error> {
        let id = ustring::UChar::try_from(id)?;
        unsafe {
            versioned_function!(utrans_unregisterID)(
                id.as_c_ptr(),
                id.len() as i32,
            )
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::sys;
    use super::UTransliterator;
    use log::trace;

    const DIR_FWD: sys::UTransDirection = sys::UTransDirection::UTRANS_FORWARD;
    const DIR_REV: sys::UTransDirection = sys::UTransDirection::UTRANS_REVERSE;

    #[test]
    fn test_builtin() {
        trace!("Available IDs");
        let ids = UTransliterator::get_ids().unwrap().map(|r| r.unwrap());
        for id in ids {
            trace!("  {}", id);
        }

        let id = "NFC;Cyrillic-Latin;Latin-ASCII";
        let trans = UTransliterator::new(id, None, DIR_FWD).unwrap();
        assert_eq!(trans.get_id().unwrap(), id);

        // "цäfé" (6) -> "cafe" (4)
        let text = "\u{0446}a\u{0308}fe\u{0301}";
        assert_eq!(text.chars().count(), 6);
        assert_eq!(trans.transliterate(text).unwrap(), "cafe");
    }

    #[test]
    fn test_inverse() {
        let trans =
            UTransliterator::new("Latin-ASCII", None, DIR_FWD).unwrap();
        let inverse = trans.inverse().unwrap();
        assert_eq!(inverse.get_id().unwrap(), "ASCII-Latin");
    }

    #[test]
    fn test_rules_based() {
        let rules = "a <> xyz;";

        let fwd_trans = UTransliterator::new("MyA-MyXYZ", Some(rules), DIR_FWD)
            .unwrap();
        assert_eq!(fwd_trans.transliterate("abc").unwrap(), "xyzbc");

        let rev_trans = UTransliterator::new("MyXYZ-MyA", Some(rules), DIR_REV)
            .unwrap();
        assert_eq!(rev_trans.transliterate("xyzbc").unwrap(), "abc");
    }

    #[test]
    fn test_to_rules() {
        let id = "MyA-MyXYZ";
        let rules = "a > xyz;";
        let trans = UTransliterator::new(id, Some(rules), DIR_FWD).unwrap();
        assert_eq!(trans.to_rules(false).unwrap(), rules);
    }

    #[test]
    fn test_set_filter() {
        let id = "MyABC-MyXYZ";
        let rules = "{a}bc > x; x{b}c > y; xy{c} > z;";
        let mut trans = UTransliterator::new(id, Some(rules), DIR_FWD).unwrap();

        trans.set_filter(Some("[ac]")).unwrap();
        assert_eq!(trans.transliterate("abc").unwrap(), "xbc");

        trans.set_filter(None).unwrap();
        assert_eq!(trans.transliterate("abc").unwrap(), "xyz");
    }

    #[test]
    fn test_register_unregister() {
        let count_available = || UTransliterator::get_ids().unwrap().count();
        let initial_count = count_available();
        let id = "MyA-MyXYZ";
        let rules = "a > xyz;";
        let trans = UTransliterator::new(&id, Some(&rules), DIR_FWD).unwrap();

        UTransliterator::register(trans).unwrap();
        assert_eq!(count_available(), initial_count + 1);

        UTransliterator::unregister(id).unwrap();
        assert_eq!(count_available(), initial_count);
    }
}
