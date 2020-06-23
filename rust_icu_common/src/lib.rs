// Copyright 2019 Google LLC
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

//! # Commonly used functionality adapters.
//!
//! At the moment, this crate contains the declaration of various errors

use {
    rust_icu_sys as sys,
    std::{ffi, os},
    thiserror::Error,
};

/// Represents a Unicode error, resulting from operations of low-level ICU libraries.
///
/// This is modeled after absl::Status in the Abseil library, which provides ways
/// for users to avoid dealing with all the numerous error codes directly.
#[derive(Error, Debug)]
pub enum Error {
    /// The error originating in the underlying sys library.
    ///
    /// At the moment it is possible to produce an Error which has a zero error code (i.e. no
    /// error), because it makes it unnecessary for users to deal with error codes directly.  It
    /// does make for a bit weird API, so we may turn it around a bit.  Ideally, it should not be
    /// possible to have an Error that isn't really an error.
    #[error("ICU error code: {}", _0)]
    Sys(sys::UErrorCode),

    /// Errors originating from the wrapper code.  For example when pre-converting input into
    /// UTF8 for input that happens to be malformed.
    #[error(transparent)]
    Wrapper(anyhow::Error),
}

impl Error {
    /// The error code denoting no error has happened.
    pub const OK_CODE: sys::UErrorCode = sys::UErrorCode::U_ZERO_ERROR;

    /// Returns true if this error code corresponds to no error.
    pub fn is_ok(code: sys::UErrorCode) -> bool {
        code == Self::OK_CODE
    }

    /// Creates a new error from the supplied status.  Ok is returned if the error code does not
    /// correspond to an error code (as opposed to OK or a warning code).
    pub fn ok_or_warning(status: sys::UErrorCode) -> Result<(), Self> {
        if Self::is_ok(status) || status < Self::OK_CODE {
            Ok(())
        } else {
            Err(Error::Sys(status))
        }
    }

    /// Creates a new error from the supplied status.  Ok is returned if the
    /// error code does not constitute an error in preflight mode.
    ///
    /// This error check explicitly ignores the buffer overflow error when reporting whether it
    /// contains an error condition.
    ///
    /// Preflight calls to ICU libraries do a dummy scan of the input to determine the buffer sizes
    /// required on the output in case of conversion calls such as `ucal_strFromUTF8`.  The way
    /// this call is made is to offer a zero-capacity buffer (which could be pointed to by a `NULL`
    /// pointer), and then call the respective function.  The function will compute the buffer
    /// size, but will also return a bogus buffer overflow error.
    pub fn ok_preflight(status: sys::UErrorCode) -> Result<(), Self> {
        if status > Self::OK_CODE && status != sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR {
            Err(Error::Sys(status))
        } else {
            Ok(())
        }
    }

    /// Returns true if this error has the supplied `code`.
    pub fn is_code(&self, code: sys::UErrorCode) -> bool {
        if let Error::Sys(c) = self {
            return *c == code;
        }
        false
    }

    /// Returns true if the error is an error, not a warning.
    ///
    /// The ICU4C library has error codes for errors and warnings.
    pub fn is_err(&self) -> bool {
        match self {
            Error::Sys(code) => *code > sys::UErrorCode::U_ZERO_ERROR,
            Error::Wrapper(_) => true,
        }
    }

    /// Return true if there was an error in a preflight call.
    ///
    /// This error check explicitly ignores the buffer overflow error when reporting whether it
    /// contains an error condition.
    ///
    /// Preflight calls to ICU libraries do a dummy scan of the input to determine the buffer sizes
    /// required on the output in case of conversion calls such as `ucal_strFromUTF8`.  The way
    /// this call is made is to offer a zero-capacity buffer (which could be pointed to by a `NULL`
    /// pointer), and then call the respective function.  The function will compute the buffer
    /// size, but will also return a bogus buffer overflow error.
    pub fn is_preflight_err(&self) -> bool {
        // We may expand the set of error codes that are exempt from error checks in preflight.
        self.is_err() && !self.is_code(sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR)
    }

    /// Returns true if the error is, in fact, a warning (nonfatal).
    pub fn is_warn(&self) -> bool {
        match self {
            Error::Sys(c) => *c < sys::UErrorCode::U_ZERO_ERROR,
            _ => false,
        }
    }

    pub fn wrapper(source: impl Into<anyhow::Error>) -> Self {
        Self::Wrapper(source.into())
    }
}

impl From<ffi::NulError> for Error {
    fn from(e: ffi::NulError) -> Self {
        Self::wrapper(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::wrapper(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::wrapper(e)
    }
}

impl Into<std::fmt::Error> for Error {
    fn into(self) -> std::fmt::Error {
        // It is not possible to transfer any info into std::fmt::Error, so we log instead.
        eprintln!("error while formatting: {:?}", &self);
        std::fmt::Error{}
    }
}

/// Generates a method to wrap ICU4C `uloc` methods that require a resizable output string buffer.
///
/// The various `uloc` methods of this type have inconsistent signature patterns, with some putting
/// all their input arguments _before_ the `buffer` and its `capacity`, and some splitting the input
/// arguments.
///
/// Therefore, the macro supports input arguments in both positions.
///
/// For an invocation of the form
///
/// ```ignore
/// buffered_string_method_with_retry!(
///     my_method,
///     BUFFER_CAPACITY,
///     [before_arg_a: before_type_a, before_arg_b: before_type_b,],
///     [after_arg_a: after_type_a, after_arg_b: after_type_b,]
/// );
/// ```
///
/// the generated method has a signature of the form
///
/// ```ignore
/// fn my_method(
///     method_to_call: unsafe extern "C" fn(
///         before_type_a,
///         before_type_b,
///         *mut raw::c_char,
///         i32,
///         after_type_a,
///         after_type_b,
///         *mut sys::UErrorCode,
///     ) -> i32,
///     before_arg_a: before_type_a,
///     before_arg_b: before_type_b,
///     after_arg_a: after_type_a,
///     after_arg_b: after_type_b
/// ) -> Result<String, common::Error> {}
/// ```
#[macro_export]
macro_rules! buffered_string_method_with_retry {

    ($method_name:ident, $buffer_capacity:expr,
     [$($before_arg:ident: $before_arg_type:ty,)*],
     [$($after_arg:ident: $after_arg_type:ty,)*]) => {
        fn $method_name(
            method_to_call: unsafe extern "C" fn(
                $($before_arg_type,)*
                *mut raw::c_char,
                i32,
                $($after_arg_type,)*
                *mut sys::UErrorCode,
            ) -> i32,
            $($before_arg: $before_arg_type,)*
            $($after_arg: $after_arg_type,)*
        ) -> Result<String, common::Error> {
            let mut status = common::Error::OK_CODE;
            let mut buf: Vec<u8> = vec![0; $buffer_capacity];

            // Requires that any pointers that are passed in are valid.
            let full_len: i32 = unsafe {
                assert!(common::Error::is_ok(status));
                method_to_call(
                    $($before_arg,)*
                    buf.as_mut_ptr() as *mut raw::c_char,
                    $buffer_capacity as i32,
                    $($after_arg,)*
                    &mut status,
                )
            };

            // ICU methods are inconsistent in whether they silently truncate the output or treat
            // the overflow as an error, so we need to check both cases.
            if status == sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR ||
               (common::Error::is_ok(status) &&
                    full_len > $buffer_capacity
                        .try_into()
                        .map_err(|e| common::Error::wrapper(e))?) {

                assert!(full_len > 0);
                let full_len: usize = full_len
                    .try_into()
                    .map_err(|e| common::Error::wrapper(e))?;
                buf.resize(full_len, 0);

                // Same unsafe requirements as above, plus full_len must be exactly the output
                // buffer size.
                unsafe {
                    assert!(common::Error::is_ok(status));
                    method_to_call(
                        $($before_arg,)*
                        buf.as_mut_ptr() as *mut raw::c_char,
                        full_len as i32,
                        $($after_arg,)*
                        &mut status,
                    )
                };
            }

            common::Error::ok_or_warning(status)?;

            // Adjust the size of the buffer here.
            if (full_len >= 0) {
                let full_len: usize = full_len
                    .try_into()
                    .map_err(|e| common::Error::wrapper(e))?;
                buf.resize(full_len, 0);
            }
            String::from_utf8(buf).map_err(|e| e.utf8_error().into())
        }
    }
}

/// Used to simulate an array of C-style strings.
#[derive(Debug)]
pub struct CStringVec {
    // The internal representation of the vector of C strings.
    rep: Vec<ffi::CString>,
    // Same as rep, but converted into C pointers.
    c_rep: Vec<*const os::raw::c_char>,
}

impl CStringVec {
    /// Creates a new C string vector from the provided rust strings.
    ///
    /// C strings are continuous byte regions that end in `\0` and do not
    /// contain `\0` anywhere else.
    ///
    /// Use `as_c_array` to get an unowned raw pointer to the array, to pass
    /// into FFI C code.
    pub fn new(strings: &[&str]) -> Result<Self, Error> {
        let mut rep = Vec::with_capacity(strings.len());
        // Convert all to asciiz strings and insert into the vector.
        for elem in strings {
            let asciiz = ffi::CString::new(*elem)?;
            rep.push(asciiz);
        }
        let c_rep = rep.iter().map(|e| e.as_ptr()).collect();
        Ok(CStringVec { rep, c_rep })
    }

    /// Returns the underlying array of C strings as a C array pointer.  The
    /// array must not change after construction to ensure that this pointer
    /// remains valid.
    pub fn as_c_array(&self) -> *const *const os::raw::c_char {
        self.c_rep.as_ptr() as *const *const os::raw::c_char
    }

    /// Returns the number of elements in the vector.
    pub fn len(&self) -> usize {
        self.rep.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code() {
        let error = Error::ok_or_warning(sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR)
            .err()
            .unwrap();
        assert!(error.is_code(sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR));
        assert!(!error.is_preflight_err());
        assert!(!error.is_code(sys::UErrorCode::U_ZERO_ERROR));
    }

    #[test]
    fn test_into_char_array() {
        let values = vec!["eenie", "meenie", "minie", "moe"];
        let c_array = CStringVec::new(&values).expect("success");
        assert_eq!(c_array.len(), 4);
    }

    #[test]
    fn test_with_embedded_nul_byte() {
        let values = vec!["hell\0x00o"];
        let _c_array = CStringVec::new(&values).expect_err("should fail");
    }
}
