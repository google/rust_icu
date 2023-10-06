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

use {
    rust_icu_common as common, rust_icu_sys as sys,
    rust_icu_sys::versioned_function,
    rust_icu_ustring as ustring,
    std::{convert::TryFrom, ffi, os::raw, ptr},
};

// Implements the ICU type [`UFormattable`][ufmt].
//
// [UFormattable] is a thin wrapper for primitive types used for number formatting.
//
// Note from the ICU4C API:
//
// > Underlying is a C interface to the class `icu::Formatable`.  Static functions
// on this class convert to and from this interface (via `reinterpret_cast`). Many
// operations are not thread safe, and should not be shared between threads.
//
//   [ufmt]: https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/uformattable_8h.html
#[derive(Debug)]
pub struct UFormattable<'a> {
    // The underlying representation.
    rep: ptr::NonNull<sys::UFormattable>,
    owner: Option<&'a Self>,
}

impl<'a> Drop for crate::UFormattable<'a> {
    /// Implements `ufmt_close`.
    fn drop(&mut self) {
        if let None = self.owner {
            unsafe { versioned_function!(ufmt_close)(self.rep.as_ptr()) };
        }
    }
}

/// Generates a simple getter, which just shunts into an appropriately
/// named getter function of the sys crate and returns the appropriate type.
///
/// Example:
///
/// ```ignore
/// simple_getter!(get_array_length, ufmt_getArrayLength, i32);
/// ```
///
/// * `$method_name` is an identifier
macro_rules! simple_getter {
    ($method_name:ident, $impl_function_name:ident, $return_type:ty) => {
        #[doc = concat!("Implements `", stringify!($impl_function_name), "`.")]
        ///
        /// Use [UFormattable::get_type] to verify that the type matches.
        pub fn $method_name(&self) -> Result<$return_type, common::Error> {
            let mut status = common::Error::OK_CODE;
            let ret = unsafe {
                assert!(common::Error::is_ok(status));
                versioned_function!($impl_function_name)(self.rep.as_ptr(), &mut status)
            };
            common::Error::ok_or_warning(status)?;
            Ok(ret)
        }
    };
}

impl<'a> crate::UFormattable<'a> {
    /// Initialize a [crate::UFormattable] to type `UNUM_LONG`, value 0 may
    /// return error.
    ///
    /// Implements `ufmt_open`.
    pub fn try_new<'b>() -> Result<crate::UFormattable<'b>, common::Error> {
        let mut status = common::Error::OK_CODE;
        // We verify that status is OK on entry and on exit, and that the
        // returned representation is not null.
        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ufmt_open)(&mut status)
        };
        common::Error::ok_or_warning(status)?;
        Ok(UFormattable {
            rep: ptr::NonNull::new(rep).unwrap(),
            owner: None,
        })
    }

    /// Reveals the underlying representation as a mutable pointer.
    ///
    /// **DO NOT USE UNLESS YOU HAVE NO OTHER CHOICE**
    ///
    /// The intended use of this method is for other crates that need to obtain
    /// low-level representations of this type.
    #[doc(hidden)]
    pub fn as_mut_ptr(&mut self) -> *mut sys::UFormattable {
        self.rep.as_ptr()
    }

    pub fn as_ptr(&self) -> *const sys::UFormattable {
        self.rep.as_ptr()
    }

    /// Returns `true` if this formattable is numeric.
    ///
    /// Implements `ufmt_isNumeric`
    pub fn is_numeric(&self) -> bool {
        let ubool = unsafe { versioned_function!(ufmt_isNumeric)(self.rep.as_ptr()) };
        match ubool {
            0i8 => false,
            _ => true,
        }
    }

    // Returns the type of this formattable.  The comment here and below is
    // used in coverage analysis; the macro `simple_getter!` generates
    // user-visible documentation.
    //
    // Implements `ufmt_getType`
    simple_getter!(get_type, ufmt_getType, sys::UFormattableType);

    // Implements `ufmt_getDate`
    simple_getter!(get_date, ufmt_getDate, sys::UDate);

    // Implements `ufmt_getDouble`
    simple_getter!(get_double, ufmt_getDouble, f64);

    // Implements `ufmt_getLong`
    simple_getter!(get_i32, ufmt_getLong, i32);

    // Implements `ufmt_getInt64`
    simple_getter!(get_i64, ufmt_getInt64, i64);

    // Implements `ufmt_getArrayLength`
    simple_getter!(get_array_length, ufmt_getArrayLength, i32);

    // Implements `ufmt_getUChars`
    pub fn get_ustring(&self) -> Result<ustring::UChar, common::Error> {
        let mut status = common::Error::OK_CODE;
        let mut ustrlen = 0i32;
        let raw = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ufmt_getUChars)(self.rep.as_ptr(), &mut ustrlen, &mut status)
        } as *mut sys::UChar;
        common::Error::ok_or_warning(status)?;
        let ret = unsafe {
            assert_ne!(raw, 0 as *mut sys::UChar);
            assert!(ustrlen >= 0);
            ustring::UChar::clone_from_raw_parts(raw, ustrlen)
        };
        Ok(ret)
    }

    /// Implements `ufmt_getUChars`
    pub fn get_str(&self) -> Result<String, common::Error> {
        let ustr = self.get_ustring()?;
        String::try_from(&ustr)
    }

    /// Use [UFormattable::get_type] to ensure that this formattable is an array before using this
    /// method.  Otherwise you will get an error.  The lifetime of the resulting formattable is tied
    /// to this one.
    ///
    /// Implements `ufmt_getArrayItemByIndex`
    pub fn get_array_item_by_index(
        &'a self,
        index: i32,
    ) -> Result<crate::UFormattable<'a>, common::Error> {
        let mut status = common::Error::OK_CODE;
        let raw: *mut sys::UFormattable = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ufmt_getArrayItemByIndex)(self.rep.as_ptr(), index, &mut status)
        };
        common::Error::ok_or_warning(status)?;
        assert_ne!(raw, 0 as *mut sys::UFormattable);
        Ok(UFormattable {
            rep: ptr::NonNull::new(raw).unwrap(),
            owner: Some(&self),
        })
    }

    /// Implements `ufmt_getDecNumChars`
    pub fn get_dec_num_chars(&self) -> Result<String, common::Error> {
        let mut status = common::Error::OK_CODE;
        let mut cstrlen = 0i32;
        let raw: *const raw::c_char = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(ufmt_getDecNumChars)(self.rep.as_ptr(), &mut cstrlen, &mut status)
        };
        common::Error::ok_or_warning(status)?;
        let ret = unsafe {
            assert_ne!(raw, 0 as *const raw::c_char);
            assert!(cstrlen >= 0);
            ffi::CStr::from_ptr(raw)
        };
        Ok(ret
            .to_str()
            .map_err(|e: std::str::Utf8Error| common::Error::from(e))?
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    // There doesn't seem to be a way to initialize a nonzero Numeric for testing all of this code.
    // So it seems it would have to remain uncovered with tests, until some other code gets to
    // use it.

    #[test]
    fn basic() {
        let n = crate::UFormattable::try_new().expect("try_new");
        assert_eq!(
            sys::UFormattableType::UFMT_LONG,
            n.get_type().expect("get_type")
        );
        assert_eq!(0, n.get_i32().expect("get_i32"));
        assert_eq!("0", n.get_dec_num_chars().expect("get_dec_num_chars"));
    }
}
