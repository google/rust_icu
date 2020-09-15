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

//! # ICU text boundary analysis support for Rust
//!
//! This crate provides a Rust implementation of the ICU text boundary analysis APIs
//! in `ubrk.h`. Character (grapheme cluster), word, line-break, and sentence iterators
//! are available.
//!
//! ## Examples
//!
//! Sample code use is given below.
//! 
//! ```rust
//! use rust_icu_sys as sys;
//! use rust_icu_ubrk as ubrk;
//!
//! let text = "The lazy dog jumped over the fox.";
//! let mut iter = ubrk::UBreakIterator::try_new(
//!     sys::UBreakIteratorType::UBRK_WORD, "en", text).unwrap();
//!
//! assert_eq!(iter.last_boundary(), 33);
//! assert_eq!(iter.current(), 33);
//! assert!(iter.is_boundary(13));
//! assert!(iter.is_boundary(19));
//! assert!(!iter.is_boundary(15));
//! assert_eq!(iter.following(15), 19);
//! assert_eq!(iter.current(), 19);
//! assert_eq!(iter.preceding(15), 13);
//! assert_eq!(iter.current(), 13);
//! assert_eq!(iter.previous(), Some(12));
//! assert_eq!(iter.current(), 12);
//! 
//! // Reset to first boundary and consume `iter`.
//! assert_eq!(iter.first(), 0);
//! let breaks: Vec<i32> = iter.collect();
//! assert_eq!(breaks, vec![3, 4, 8, 9, 12, 13, 19, 20, 24, 25, 28, 29, 32, 33]);
//! ```
//!
//! See the [ICU user guide](https://unicode-org.github.io/icu/userguide/boundaryanalysis/)
//! and the C API documentation in the
//! [`ubrk.h` header](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/ubrk_8h.html)
//! for details.

use {
    rust_icu_common::{self as common, simple_drop_impl},
    rust_icu_sys::{self as sys, *},
    rust_icu_ustring as ustring,
    std::{convert::TryFrom, ffi, os::raw, ptr, rc::Rc},
};

/// Returned by break iterator to indicate that all text boundaries have been returned.
// UBRK_DONE is defined as a macro in ICU and macros are not currently supported
// by bindgen, so we define it ourselves here.
pub const UBRK_DONE: i32 = -1;

/// Rust wrapper for the ICU `UBreakIterator` type.
pub struct UBreakIterator {
    // Pointer to the underlying ICU representation.
    rep: ptr::NonNull<sys::UBreakIterator>,

    // The underlying C representation holds pointers to `text` and exactly one of
    // {`locale`, `rules`, `binary_rules`} throughout its lifetime. We are
    // responsible for ensuring that the pointers remain valid during that time,
    // and for dropping the referenced values once the underlying C representation
    // is released.
    //
    // A break iterator may be cloned, in which case the underlying C representation
    // of the cloned iterator will hold pointers to the values pointed to by the
    // original iterator, while maintaining its own internal iteration state.
    //
    // For these reasons, these fields are wrapped in `Rc`, ensuring that the
    // referenced values (`text`, `locale`, etc.) are not released prematurely
    // if the original break iterator is dropped before its clone. As break
    // iterators are inherently not thread-safe [1], `Rc` was chosen over `Arc`.
    //
    // [1] https://unicode-org.github.io/icu/userguide/boundaryanalysis/#thread-safety
    text: Rc<ustring::UChar>,
    locale: Option<Rc<ffi::CString>>,
    rules: Option<Rc<ustring::UChar>>,
    binary_rules: Option<Rc<Vec<u8>>>,
}

// Implements `ubrk_close`.
simple_drop_impl!(UBreakIterator, ubrk_close);

impl Iterator for UBreakIterator {
    type Item = i32;

    /// Advances the break iterator's position to the next boundary after its
    /// current position.
    ///
    /// Note that `ubrk_next` will _never_ return the first boundary. For example,
    /// given a newly-initialized break iterator whose internal position is `0`,
    /// the first invocation of `next` will return the _next_ boundary, not `0`.
    /// If the caller requires the first boundary, it should utilize [`first`].
    ///
    /// Also note that interleaving calls to [`first`], [`last_boundary`], [`previous`],
    /// [`preceding`], or [`following`] may change the break iterator's internal
    /// position, thereby affecting the next value returned by `next`.
    ///
    /// Implements `ubrk_next`.
    ///
    /// [`first`]: #method.first
    /// [`following`]: #method.following
    /// [`last_boundary`]: #method.last_boundary
    /// [`preceding`]: #method.preceding
    /// [`previous`]: #method.previous
    fn next(&mut self) -> Option<Self::Item> {
        let index =
            unsafe { versioned_function!(ubrk_next)(self.rep.as_ptr()) };
        if index == UBRK_DONE {
            None
        } else {
            Some(index)
        }
    }
}

impl UBreakIterator {
    /// Reports the number of locales for which text breaking information is
    /// available.
    ///
    /// Implements `ubrk_countAvailable`.
    pub fn count_available_locales() -> i32 {
        unsafe { versioned_function!(ubrk_countAvailable)() }
    }

    /// Returns the locale for which line breaking information is available
    /// at the specified index.
    ///
    /// Implements `ubrk_getAvailable`.
    pub fn get_available_locale_at(
        index: i32,
    ) -> Result<Option<String>, common::Error> {
        let locale_ptr =
            unsafe { versioned_function!(ubrk_getAvailable)(index) };
        if locale_ptr == 0 as *const raw::c_char {
            Ok(None)
        } else {
            let c_str = unsafe { ffi::CStr::from_ptr(locale_ptr) };
            let s = c_str.to_str().map(|s| s.to_owned())?;
            Ok(Some(s))
        }
    }

    /// Creates a new break iterator with the specified type (character, word,
    /// line, or sentence) in the specified locale over `text`.
    ///
    /// Implements `ubrk_open`.
    pub fn try_new(
        type_: sys::UBreakIteratorType,
        locale: &str,
        text: &str,
    ) -> Result<Self, common::Error> {
        let text = ustring::UChar::try_from(text)?;
        Self::try_new_ustring(type_, &locale, &text)
    }

    /// Implements `ubrk_open`.
    pub fn try_new_ustring(
        type_: sys::UBreakIteratorType,
        locale: &str,
        text: &ustring::UChar,
    ) -> Result<Self, common::Error> {
        let mut status = common::Error::OK_CODE;
        let locale = ffi::CString::new(locale)?;
        // Clone text for break iterator to own.
        let text = text.clone();
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ubrk_open)(
                type_,
                locale.as_ptr(),
                text.as_c_ptr(),
                text.len() as i32,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        assert_ne!(rep, 0 as *mut sys::UBreakIterator);
        Ok(Self {
            locale: Some(Rc::new(locale)),
            rules: None,
            binary_rules: None,
            text: Rc::new(text),
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Creates a new break iterator using the specified breaking rules.
    ///
    /// See the [ICU user guide](https://unicode-org.github.io/icu/userguide/boundaryanalysis/break-rules.html)
    /// for rules syntax.
    ///
    /// Implements `ubrk_openRules`.
    pub fn try_new_rules(
        rules: &str,
        text: &str,
    ) -> Result<Self, common::Error> {
        let rules = ustring::UChar::try_from(rules)?;
        let text = ustring::UChar::try_from(text)?;
        Self::try_new_rules_ustring(&rules, &text)
    }

    /// Implements `ubrk_openRules`.
    pub fn try_new_rules_ustring(
        rules: &ustring::UChar,
        text: &ustring::UChar,
    ) -> Result<Self, common::Error> {
        let mut status = common::Error::OK_CODE;
        let mut parse_status = common::NO_PARSE_ERROR;
        // Clone text and rules for break iterator to own.
        let rules = rules.clone();
        let text = text.clone();
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ubrk_openRules)(
                rules.as_c_ptr(),
                rules.len() as i32,
                text.as_c_ptr(),
                text.len() as i32,
                &mut parse_status,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        common::parse_ok(parse_status)?;
        assert_ne!(rep, 0 as *mut sys::UBreakIterator);
        Ok(Self {
            locale: None,
            rules: Some(Rc::new(rules)),
            binary_rules: None,
            text: Rc::new(text),
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Creates a new break iterator using pre-compiled binary rules.
    ///
    /// Binary rules can be obtained with [`get_binary_rules`].
    ///
    /// [`get_binary_rules`]: #method.get_binary_rules
    ///
    /// Implements `ubrk_openBinaryRules`.
    pub fn try_new_binary_rules(
        rules: &Vec<u8>,
        text: &str,
    ) -> Result<Self, common::Error> {
        let text = ustring::UChar::try_from(text)?;
        Self::try_new_binary_rules_ustring(rules, &text)
    }

    /// Implements `ubrk_openBinaryRules`.
    pub fn try_new_binary_rules_ustring(
        rules: &Vec<u8>,
        text: &ustring::UChar,
    ) -> Result<Self, common::Error> {
        let mut status = common::Error::OK_CODE;
        // Clone text and binary rules for break iterator to own.
        let rules = rules.clone();
        let text = text.clone();
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ubrk_openBinaryRules)(
                rules.as_ptr() as *const raw::c_uchar,
                rules.len() as i32,
                text.as_c_ptr(),
                text.len() as i32,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        assert_ne!(rep, 0 as *mut sys::UBreakIterator);
        Ok(Self {
            locale: None,
            rules: None,
            binary_rules: Some(Rc::new(rules)),
            text: Rc::new(text),
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Returns a `Vec<u8>` containing the compiled binary version of the rules
    /// specifying the behavior of this break iterator.
    ///
    /// Implements `ubrk_getBinaryRules`.
    pub fn get_binary_rules(&self) -> Result<Vec<u8>, common::Error> {
        let mut status = common::Error::OK_CODE;
        // Preflight to determine length of buffer for binary rules.
        let rules_len = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ubrk_getBinaryRules)(
                self.rep.as_ptr(),
                0 as *mut raw::c_uchar,
                0,
                &mut status,
            )
        };
        common::Error::ok_preflight(status)?;
        // Use determined length to get the actual binary rules.
        let mut status = common::Error::OK_CODE;
        let mut rules = vec![0; rules_len as usize];
        unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ubrk_getBinaryRules)(
                self.rep.as_ptr(),
                rules.as_mut_ptr() as *mut raw::c_uchar,
                rules_len,
                &mut status,
            );
        }
        common::Error::ok_or_warning(status)?;
        Ok(rules)
    }

    /// Performs a clone of the underlying representation.
    ///
    /// The cloned break iterator will hold pointers to the same text, and rules,
    /// binary rules, or locale, as the original break iterator. The clone's
    /// underlying C representation will maintain its own independent iteration
    /// state, but it will be initialized to that of the original (so, for example,
    /// if `self.current() == 11`, then `self.safe_clone().current() == 11`).
    ///
    /// Note that the `Clone` trait was not implemented as the underlying operation
    /// may fail.
    ///
    /// Implements `ubrk_safeClone`.
    pub fn safe_clone(&self) -> Result<Self, common::Error> {
        let mut status = common::Error::OK_CODE;
        let rep = unsafe {
            versioned_function!(ubrk_safeClone)(
                self.rep.as_ptr(),
                // The following two parameters, stackBuffer and pBufferSize,
                // are deprecated, so we pass NULL pointers.
                0 as *mut raw::c_void,
                0 as *mut i32,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        assert_ne!(rep, 0 as *mut sys::UBreakIterator);
        Ok(Self {
            locale: self.locale.as_ref().map(|x| x.clone()),
            rules: self.rules.as_ref().map(|x| x.clone()),
            binary_rules: self.binary_rules.as_ref().map(|x| x.clone()),
            text: self.text.clone(),
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Instructs this break iterator to point to a new piece of text.
    ///
    /// Implements `ubrk_setText`.
    pub fn set_text(&mut self, text: &str) -> Result<(), common::Error> {
        let text = ustring::UChar::try_from(text)?;
        self.set_text_ustring(&text)
    }

    /// Implements `ubrk_setText`.
    pub fn set_text_ustring(
        &mut self,
        text: &ustring::UChar,
    ) -> Result<(), common::Error> {
        let mut status = common::Error::OK_CODE;
        // Clone text and take ownership.
        let text = text.clone();
        unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ubrk_setText)(
                self.rep.as_ptr(),
                text.as_c_ptr(),
                text.len() as i32,
                &mut status,
            );
        }
        common::Error::ok_or_warning(status)?;
        self.text = Rc::new(text);
        Ok(())
    }

    /// Reports the most recently-returned text boundary.
    ///
    /// Implements `ubrk_current`.
    pub fn current(&self) -> i32 {
        unsafe { versioned_function!(ubrk_current)(self.rep.as_ptr()) }
    }

    /// Sets the break iterator's position to the boundary preceeding its current
    /// position.
    ///
    /// Implements `ubrk_previous`.
    pub fn previous(&self) -> Option<i32> {
        let result =
            unsafe { versioned_function!(ubrk_previous)(self.rep.as_ptr()) };
        if result == UBRK_DONE {
            None
        } else {
            Some(result)
        }
    }

    /// Moves the iterator to the beginning of its text and returns the new
    /// position (zero).
    ///
    /// Implements `ubrk_first`.
    pub fn first(&self) -> i32 {
        unsafe { versioned_function!(ubrk_first)(self.rep.as_ptr()) }
    }

    /// Moves the iterator to the position immediately _beyond_ the last character
    /// in its text and returns the new position.
    ///
    /// Named as such so as to avoid conflict with the `last` method provided by
    /// `Iterator`.
    ///
    /// Implements `ubrk_last`.
    pub fn last_boundary(&self) -> i32 {
        unsafe { versioned_function!(ubrk_last)(self.rep.as_ptr()) }
    }

    /// Moves the iterator to the boundary immediately preceding the specified offset
    /// and returns the new position.
    ///
    /// Implements `ubrk_preceding`.
    pub fn preceding(&self, offset: i32) -> i32 {
        unsafe {
            versioned_function!(ubrk_preceding)(self.rep.as_ptr(), offset)
        }
    }

    /// Moves the iterator to the boundary immediately following the specified offset
    /// and returns the new position.
    ///
    /// Implements `ubrk_following`.
    pub fn following(&self, offset: i32) -> i32 {
        unsafe {
            versioned_function!(ubrk_following)(self.rep.as_ptr(), offset)
        }
    }

    /// Reports whether the specified offset is a boundary.
    ///
    /// Implements `ubrk_isBoundary`.
    pub fn is_boundary(&self, offset: i32) -> bool {
        let result: sys::UBool = unsafe {
            versioned_function!(ubrk_isBoundary)(self.rep.as_ptr(), offset)
        };
        result != 0
    }

    /// Returns the locale, valid or actual, of this break iterator.
    ///
    /// Implements `ubrk_getLocaleByType`.
    pub fn get_locale_by_type(
        &self,
        type_: sys::ULocDataLocaleType,
    ) -> Result<String, common::Error> {
        let mut status = common::Error::OK_CODE;
        let char_ptr = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ubrk_getLocaleByType)(
                self.rep.as_ptr(),
                type_,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        let c_str = unsafe { ffi::CStr::from_ptr(char_ptr) };
        let s = c_str.to_str().map(|s| s.to_owned())?;
        Ok(s)
    }

    /// Returns the status of the break rule that determined the most-recently
    /// returned boundary. The default status for rules that do not explicitly
    /// provide one is zero.
    ///
    /// See the [ICU user guide](https://unicode-org.github.io/icu/userguide/boundaryanalysis/break-rules.html)
    /// for details on rule syntax and rule status values.
    ///
    /// Implements `ubrk_getRuleStatus`.
    pub fn get_rule_status(&self) -> i32 {
        unsafe { versioned_function!(ubrk_getRuleStatus)(self.rep.as_ptr()) }
    }

    /// Returns the statuses of the break rules that determined the most-recently
    /// returned boundary. The default status for rules that do not explicitly
    /// provide one is zero.
    ///
    /// See the [ICU user guide](https://unicode-org.github.io/icu/userguide/boundaryanalysis/break-rules.html)
    /// for details on rule syntax and rule status values.
    ///
    /// Implements `ubrk_getRuleStatusVec`.
    pub fn get_rule_status_vec(&self) -> Result<Vec<i32>, common::Error> {
        let mut status = common::Error::OK_CODE;
        // Preflight to determine buffer size.
        let rules_len = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ubrk_getRuleStatusVec)(
                self.rep.as_ptr(),
                0 as *mut i32,
                0,
                &mut status,
            )
        };
        common::Error::ok_preflight(status)?;
        let mut status = common::Error::OK_CODE;
        let mut rules: Vec<i32> = vec![0; rules_len as usize];
        unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ubrk_getRuleStatusVec)(
                self.rep.as_ptr(),
                rules.as_mut_ptr(),
                rules_len,
                &mut status,
            );
        }
        common::Error::ok_or_warning(status)?;
        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::UBreakIterator;
    use log::trace;
    use rust_icu_sys::{UBreakIteratorType::*, ULocDataLocaleType::*};
    use std::{convert::TryFrom, rc::Rc};

    const TEXT: &str =
        r#""It wasn't the wine," murmured Mr. Snodgrass. "It was the salmon.""#;

    const WORD_BOUNDARIES: [i32; 30] = [
        0, 1, 3, 4, 10, 11, 14, 15, 19, 20, 21, 22, 30, 31, 33, 34, 35, 44, 45,
        46, 47, 49, 50, 53, 54, 57, 58, 64, 65, 66,
    ];

    #[test]
    fn test_iteration() {
        let mut iter = UBreakIterator::try_new(UBRK_WORD, "en", TEXT).unwrap();

        assert_eq!(iter.first(), 0);
        assert_eq!(iter.current(), 0);
        assert!(iter.is_boundary(0));
        assert_eq!(iter.previous(), None);
        assert_eq!(iter.current(), 0);

        assert!(iter.is_boundary(22));
        assert!(!iter.is_boundary(25));
        assert_eq!(iter.preceding(25), 22);
        assert_eq!(iter.current(), 22);
        assert_eq!(iter.previous(), Some(21));
        assert_eq!(iter.current(), 21);
        assert_eq!(iter.next(), Some(22));
        assert_eq!(iter.current(), 22);

        assert!(!iter.is_boundary(55));
        assert!(iter.is_boundary(57));
        assert_eq!(iter.following(55), 57);
        assert_eq!(iter.current(), 57);
        assert_eq!(iter.next(), Some(58));
        assert_eq!(iter.current(), 58);
        assert_eq!(iter.preceding(58), 57);
        assert_eq!(iter.current(), 57);

        assert_eq!(iter.last_boundary(), 66);
        assert_eq!(iter.current(), 66);
        assert!(iter.is_boundary(66));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_binary_rules() {
        let iter1 = UBreakIterator::try_new(UBRK_WORD, "en", TEXT).unwrap();
        let iter1_rules = iter1.get_binary_rules().unwrap();
        iter1.first();
        let iter1_boundaries: Vec<i32> = iter1.collect();

        let iter2 =
            UBreakIterator::try_new_binary_rules(&iter1_rules, TEXT).unwrap();
        iter2.first();
        let iter2_boundaries: Vec<i32> = iter2.collect();

        assert_eq!(WORD_BOUNDARIES[1..].to_vec(), iter1_boundaries);
        assert_eq!(iter1_boundaries, iter2_boundaries);
    }

    #[test]
    fn test_rules() {
        let rules = r#"
# Our custom break rules: break on `w`s.

!!chain;
!!quoted_literals_only;

$w     = [w];
$not_w = [^w];

$not_w+;  # No breaks between code points other than `w`.
$w+ {99}; # Break on `w`s with custom rule status of `99`.
"#;

        let _w_boundaries: [i32; 8] = [0, 4, 5, 15, 16, 50, 51, 66];

        let mut iter = UBreakIterator::try_new_rules(rules, TEXT).unwrap();

        assert_eq!(iter.first(), 0);

        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.get_rule_status(), 0);
        assert_eq!(iter.get_rule_status_vec().unwrap(), vec![0]);

        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.get_rule_status(), 99);
        assert_eq!(iter.get_rule_status_vec().unwrap(), vec![99]);

        assert_eq!(iter.next(), Some(15));
        assert_eq!(iter.get_rule_status(), 0);
        assert_eq!(iter.get_rule_status_vec().unwrap(), vec![0]);

        assert_eq!(iter.next(), Some(16));
        assert_eq!(iter.get_rule_status(), 99);
        assert_eq!(iter.get_rule_status_vec().unwrap(), vec![99]);

        assert_eq!(iter.next(), Some(50));
        assert_eq!(iter.get_rule_status(), 0);
        assert_eq!(iter.get_rule_status_vec().unwrap(), vec![0]);

        assert_eq!(iter.next(), Some(51));
        assert_eq!(iter.get_rule_status(), 99);
        assert_eq!(iter.get_rule_status_vec().unwrap(), vec![99]);

        assert_eq!(iter.next(), Some(66));
        assert_eq!(iter.get_rule_status(), 0);
        assert_eq!(iter.get_rule_status_vec().unwrap(), vec![0]);

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_clone() {
        let mut original =
            UBreakIterator::try_new(UBRK_WORD, "en", TEXT).unwrap();
        original.first();

        assert_eq!(Rc::strong_count(&original.text), 1);
        assert_eq!(Rc::strong_count(original.locale.as_ref().unwrap()), 1);

        assert_eq!(original.next(), Some(1));
        assert_eq!(original.next(), Some(3));
        assert_eq!(original.current(), 3);

        // Clone in a new scope.
        {
            let mut clone = original.safe_clone().unwrap();

            assert_eq!(Rc::strong_count(&original.text), 2);
            assert_eq!(Rc::strong_count(original.locale.as_ref().unwrap()), 2);

            assert_eq!(clone.current(), 3);
            assert_eq!(clone.first(), 0);
            assert_eq!(clone.next(), Some(1));

            assert_eq!(original.next(), Some(4));
        }

        assert_eq!(Rc::strong_count(&original.text), 1);
        assert_eq!(Rc::strong_count(original.locale.as_ref().unwrap()), 1);

        assert_eq!(original.current(), 4);
        assert_eq!(original.next(), Some(10));
    }

    #[test]
    fn test_set_text() {
        let mut iter = UBreakIterator::try_new(UBRK_WORD, "en", TEXT).unwrap();
        let original_iter_text = iter.text.clone();

        assert_eq!(Rc::strong_count(&original_iter_text), 2);
        assert_eq!(iter.preceding(59), 58);
        assert_eq!(iter.current(), 58);

        iter.set_text("The lazy dog.").unwrap();

        assert_eq!(Rc::strong_count(&original_iter_text), 1);
        assert_eq!(String::try_from(&*iter.text).unwrap(), "The lazy dog.");
        assert_eq!(iter.current(), 0);
        assert_eq!(iter.last_boundary(), 13);
    }

    #[test]
    fn test_get_locale_by_type() {
        let iter =
            UBreakIterator::try_new(UBRK_WORD, "en_US_CA@lb=strict", TEXT)
                .unwrap();

        // The "valid locale" is the most specific locale supported by ICU, given
        // what was requested.
        assert_eq!(
            iter.get_locale_by_type(ULOC_VALID_LOCALE).unwrap(),
            "en_US"
        );

        // The "actual locale" is the locale that breaking information actually comes from.
        // In most cases this will be "root".
        assert_eq!(
            iter.get_locale_by_type(ULOC_ACTUAL_LOCALE).unwrap(),
            "root"
        );
    }

    #[test]
    fn test_available_locales() {
        trace!("Available locales");
        let count = UBreakIterator::count_available_locales();
        for i in 0..count {
            let locale = UBreakIterator::get_available_locale_at(i).unwrap();
            match locale {
                Some(loc) => trace!("  {}", loc),
                None => (),
            }
        }
    }
}
