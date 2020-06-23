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
    std::{convert::TryFrom, ffi, os::raw, ptr},
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
    /// Implements `unum_open`
    pub fn try_new_ustring(
        style: sys::UNumberFormatStyle,
        pattern: ustring::UChar,
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
}

#[cfg(test)]
mod tests {}
