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

//! # ICU plural rules support for Rust
//!
//! This crate provides locale-sensitive plural rules, based on the list
//! formatting as implemente by the ICU library.  Specifically, the functionality
//! exposed through its C API, as available in the [header `upluralrules.h`][header].
//!
//!   [header]: https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/upluralrules_8h.html
//!
//! > Are you missing some features from this crate?  Consider [reporting an
//! issue](https://github.com/google/rust_icu/issues) or even [contributing the
//! functionality](https://github.com/google/rust_icu/pulls).

use {
    rust_icu_common as common,
    rust_icu_sys::{self as sys, versioned_function, *},
    rust_icu_uenum as uenum, rust_icu_ustring as ustring,
    rust_icu_ustring::buffered_uchar_method_with_retry,
    std::{convert::TryFrom, convert::TryInto, ffi, ptr},
};

/// The "plural rules" formatter struct.  Create a new instance with [UPluralRules::try_new], or
/// [UPluralRules::try_new_styled].
#[derive(Debug)]
pub struct UPluralRules {
    // Internal representation is the low-level ICU UPluralRules struct.
    rep: ptr::NonNull<sys::UPluralRules>,
}

impl Drop for UPluralRules {
    /// Implements uplrules_close`.
    fn drop(&mut self) {
        unsafe { versioned_function!(uplrules_close)(self.rep.as_ptr()) };
    }
}

impl UPluralRules {
    /// Implements uplrules_open`.
    pub fn try_new(locale: &str) -> Result<UPluralRules, common::Error> {
        let locale_cstr = ffi::CString::new(locale)?;
        let mut status = common::Error::OK_CODE;
        // Unsafety note: uplrules_open is the way to open a new formatter.
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(uplrules_open)(locale_cstr.as_ptr(), &mut status)
                as *mut sys::UPluralRules
        };
        common::Error::ok_or_warning(status)?;
        assert_ne!(rep, 0 as *mut sys::UPluralRules);
        Ok(UPluralRules {
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Implements `uplrules_openForType`.
    pub fn try_new_styled(
        locale: &str,
        format_type: sys::UPluralType,
    ) -> Result<UPluralRules, common::Error> {
        let locale_cstr = ffi::CString::new(locale)?;
        let mut status = common::Error::OK_CODE;
        // Unsafety note: all parameters are safe, so should be valid.
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(uplrules_openForType)(
                locale_cstr.as_ptr(),
                format_type,
                &mut status,
            ) as *mut sys::UPluralRules
        };
        common::Error::ok_or_warning(status)?;
        Ok(UPluralRules {
            rep: ptr::NonNull::new(rep).unwrap(),
        })
    }

    /// Implements `uplrules_select`.
    pub fn select_ustring(&self, number: f64) -> Result<ustring::UChar, common::Error> {
        const BUFFER_CAPACITY: usize = 20;
        buffered_uchar_method_with_retry!(
            select_impl,
            BUFFER_CAPACITY,
            [rep: *const sys::UPluralRules, number: f64,],
            []
        );

        select_impl(
            versioned_function!(uplrules_select),
            self.rep.as_ptr(),
            number,
        )
    }

    /// Implements `uplrules_select`.
    pub fn select(&self, number: f64) -> Result<String, common::Error> {
        let result = self.select_ustring(number);
        match result {
            Err(e) => Err(e),
            Ok(u) => String::try_from(&u).map_err(|e| e.into()),
        }
    }

    /// Implements `uplrules_getKeywords`
    pub fn get_keywords(&self) -> Result<uenum::Enumeration, common::Error> {
        let mut status = UErrorCode::U_ZERO_ERROR;
        let raw_enum = unsafe {
            assert_eq!(status, UErrorCode::U_ZERO_ERROR);
            versioned_function!(uplrules_getKeywords)(self.rep.as_ptr(), &mut status)
        };
        common::Error::ok_or_warning(status)?;
        Ok(unsafe {
            assert_ne!(raw_enum, 0 as *mut sys::UEnumeration);
            uenum::Enumeration::from_raw_parts(None, raw_enum)
        })
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn plurals_ar_eg() -> Result<(), common::Error> {
        let pl = crate::UPluralRules::try_new("ar_EG").expect("locale ar_EG exists");
        assert_eq!("zero", pl.select(0 as f64)?);
        assert_eq!("one", pl.select(1 as f64)?);
        assert_eq!("two", pl.select(2 as f64)?);
        assert_eq!("few", pl.select(6 as f64)?);
        assert_eq!("many", pl.select(18 as f64)?);
        Ok(())
    }

    #[test]
    fn plurals_ar_eg_styled() -> Result<(), common::Error> {
        let pl = crate::UPluralRules::try_new_styled("ar_EG", UPluralType::UPLURAL_TYPE_ORDINAL)
            .expect("locale ar_EG exists");
        assert_eq!("other", pl.select(0 as f64)?);
        assert_eq!("other", pl.select(1 as f64)?);
        assert_eq!("other", pl.select(2 as f64)?);
        assert_eq!("other", pl.select(6 as f64)?);
        assert_eq!("other", pl.select(18 as f64)?);
        Ok(())
    }

    #[test]
    fn plurals_sr_rs() -> Result<(), common::Error> {
        let pl = crate::UPluralRules::try_new("sr_RS").expect("locale sr_RS exists");
        assert_eq!("other", pl.select(0 as f64)?);
        assert_eq!("one", pl.select(1 as f64)?);
        assert_eq!("few", pl.select(2 as f64)?);
        assert_eq!("few", pl.select(4 as f64)?);
        assert_eq!("other", pl.select(5 as f64)?);
        assert_eq!("other", pl.select(6 as f64)?);
        assert_eq!("other", pl.select(18 as f64)?);
        assert_eq!("other", pl.select(11 as f64)?);

        assert_eq!("one", pl.select(21 as f64)?);
        Ok(())
    }

    #[test]
    fn plurals_sr_rs_styled() -> Result<(), common::Error> {
        let pl = crate::UPluralRules::try_new_styled("sr_RS", UPluralType::UPLURAL_TYPE_ORDINAL)
            .expect("locale sr_RS exists");
        assert_eq!("other", pl.select(0 as f64)?);
        assert_eq!("other", pl.select(1 as f64)?);
        assert_eq!("other", pl.select(2 as f64)?);
        assert_eq!("other", pl.select(6 as f64)?);
        assert_eq!("other", pl.select(18 as f64)?);
        Ok(())
    }

    #[test]
    fn all_keywords() -> Result<(), common::Error> {
        let pl = crate::UPluralRules::try_new("sr_RS").expect("locale sr_RS exists");
        let e = pl.get_keywords()?;
        let all: Vec<String> = e.into_iter().map(|r| r.unwrap()).collect();
        assert_eq!(vec!["few", "one", "other"], all);
        Ok(())
    }

    #[test]
    fn all_keywords_styled() -> Result<(), common::Error> {
        let pl = crate::UPluralRules::try_new_styled("sr_RS", UPluralType::UPLURAL_TYPE_ORDINAL)
            .expect("locale sr_RS exists");
        let e = pl.get_keywords()?;
        let all: Vec<String> = e.into_iter().map(|r| r.unwrap()).collect();
        assert_eq!(vec!["other"], all);
        Ok(())
    }
}
