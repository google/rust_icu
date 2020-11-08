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

// The message format C api is variadic.

//! # Locale-aware message formatting.
//!
//! Implementation of the text formatting code from the ICU4C
//! [`umsg.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/umsg_8h.html) header.
//! Skip to the section ["Example use"](#example-use) below if you want to see it in action.
//!
//! The library inherits all pattern and formatting specifics from the corresponding [ICU C++
//! API](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/classicu_1_1MessageFormat.html).
//!
//! This is the support for [MessageFormat](http://userguide.icu-project.org/formatparse/messages)
//! message formatting.  The `MessageFormat` uses ICU data to format text properly based on the
//! locale selected at formatter initialization.  This includes formatting dates, times,
//! currencies, and other text.
//!
//! > **Note:** The `MessageFormat` library does not handle loading the format patterns in the
//! > appropriate language.  This task is left to the application author.
//!
//! # Example use
//!
//! The example below shows how to format values into an English text.  For more detail about
//! formatting specifics see [message_format!].
//!
//! ```ignore
//! use rust_icu_sys as sys;
//! use rust_icu_common as common;
//! use rust_icu_ustring as ustring;
//! use rust_icu_uloc as uloc;
//! use rust_icu_umsg::{self as umsg, message_format};
//! # use rust_icu_ucal as ucal;
//! # use std::convert::TryFrom;
//! #
//! # struct TzSave(String);
//! # impl Drop for TzSave {
//! #    fn drop(&mut self) {
//! #        ucal::set_default_time_zone(&self.0);
//! #    }
//! # }
//!
//! fn testfn() -> Result<(), common::Error> {
//! #   let _ = TzSave(ucal::get_default_time_zone()?);
//! #   ucal::set_default_time_zone("Europe/Amsterdam")?;
//!     let loc = uloc::ULoc::try_from("en-US-u-tz-uslax")?;
//!     let msg = ustring::UChar::try_from(
//!       r"Formatted double: {0,number,##.#},
//!         Formatted integer: {1,number,integer},
//!         Formatted string: {2},
//!         Date: {3,date,full}",
//!     )?;
//!
//!     let fmt = umsg::UMessageFormat::try_from(&msg, &loc)?;
//!     let hello = ustring::UChar::try_from("Hello! Добар дан!")?;
//!     let result = umsg::message_format!(
//!       fmt,
//!       43.4,
//!       31337,
//!       hello,
//!       0.0,
//!     )?;
//!
//!     assert_eq!(
//!       r"Formatted double: 43.4,
//!         Formatted integer: 31,337,
//!         Formatted string: Hello! Добар дан!,
//!         Date: Thursday, January 1, 1970",
//!       result
//!     );
//!     Ok(())
//! }
//! # fn main() -> Result<(), common::Error> {
//! #   testfn()
//! # }
//! ```

use {
    rust_icu_common as common, rust_icu_sys as sys, rust_icu_sys::*, rust_icu_uloc as uloc,
    rust_icu_ustring as ustring, std::convert::TryFrom,
};

use sealed::Sealed;

#[doc(hidden)]
pub use {rust_icu_sys as __sys, rust_icu_ustring as __ustring, std as __std};

/// The implementation of the ICU `UMessageFormat*`.
///
/// Use the [UMessageFormat::try_from] to create a message formatter for a given message pattern in
/// the [Messageformat](http://userguide.icu-project.org/formatparse/messages) and a specified
/// locale.  Use the macro [message_format!] to actually format the arguments.
///
/// [UMessageFormat] supports very few methods when compared to the wealth of functions that one
/// can see in
/// [`umsg.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/umsg_8h.html).  It is
/// not clear that other functions available there offer significantly more functionality than is
/// given here.
///
/// If, however, you find that the set of methods implemented at the moment are not adequate, feel
/// free to provide a [pull request](https://github.com/google/rust_icu/pulls) implementing what
/// you need.
///
/// Implements `UMessageFormat`.
#[derive(Debug)]
pub struct UMessageFormat {
    rep: std::rc::Rc<Rep>,
}

// An internal representation of the message formatter, used to allow cloning.
#[derive(Debug)]
struct Rep {
    rep: *mut sys::UMessageFormat,
}

impl Drop for Rep {
    /// Drops the content of [sys::UMessageFormat] and releases its memory.
    ///
    /// Implements `umsg_close`.
    fn drop(&mut self) {
        unsafe {
            versioned_function!(umsg_close)(self.rep);
        }
    }
}

impl Clone for UMessageFormat {
    /// Implements `umsg_clone`.
    fn clone(&self) -> Self {
        // Note this is not OK if UMessageFormat ever grows mutable methods.
        UMessageFormat {
            rep: self.rep.clone(),
        }
    }
}

impl UMessageFormat {
    /// Creates a new message formatter.
    ///
    /// A single message formatter is created per each pattern-locale combination. Mutable methods
    /// from [`umsg`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/umsg_8h.html)
    /// are not implemented, and for now requires that all formatting be separate.
    ///
    /// Implements `umsg_open`.
    pub fn try_from(
        pattern: &ustring::UChar,
        locale: &uloc::ULoc,
    ) -> Result<UMessageFormat, common::Error> {
        let pstr = pattern.as_c_ptr();
        let loc = locale.as_c_str();
        let mut status = common::Error::OK_CODE;
        let mut parse_status = common::NO_PARSE_ERROR;

        let rep = unsafe {
            assert!(common::Error::is_ok(status));
            versioned_function!(umsg_open)(
                pstr,
                pattern.len() as i32,
                loc.as_ptr(),
                &mut parse_status,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        common::parse_ok(parse_status)?;
        Ok(UMessageFormat {
            rep: std::rc::Rc::new(Rep { rep }),
        })
    }
}

/// Given a formatter, formats the passed arguments into the formatter's message.
///
/// The general usage pattern for the formatter is as follows, assuming that `formatter`
/// is an appropriately initialized [UMessageFormat]:
///
/// ``` ignore
/// use rust_icu_umsg as umsg;
/// // let result = umsg::message_format!(
/// //     formatter, [value, ...]);
/// let result = umsg::message_format!(formatter, 31337);
/// ```
///
/// Each fragment represents a single positional parameter binding for the pattern in `formatter`.
/// The first fragment corresponds to the positional parameter `0` (which, if an integer, would be
/// referred to as `{0,number,integer}` in a MessageFormat pattern). Since the original C API that
/// this rust library is generated for uses variadic functions for parameter passing, it is very
/// important that the programmer matches the actual parameter types to the types that are expected
/// in the pattern.
///
/// > **Note:** If the types of parameter bindings do not match the expectations in the pattern,
/// > memory corruption may occur, so tread lightly here.
///
/// In general this is very brittle, and an API in a more modern lanugage, or a contemporary C++
/// flavor would probably take a different route were the library to be written today.  The rust
/// binding tries to make the API use a bit more palatable by requiring that the programmer
/// explicitly specifies a type for each of the parameters to be passed into the formatter.
///
/// This function supports passing in the types `f64`, [rust_icu_ustring::UChar], `i32` and `i64`.
/// Any numeric parameter not specifically designated as different type, is always a `f64`. See
/// section below on Doubles. Dates are passed in as `f64`s - depending on the date format requested
/// in the pattern used in [UMessageFormat], the end result of date formatting could be one of a
/// wide variety of [date formats](http://userguide.icu-project.org/formatparse/datetime).
///
/// # Explicit Types
///
/// You can also explicitly specify the types used by writing `{ value => <type> }` instead of a
/// plain expression.
///
/// | Type | Rust Type |
/// | ---- | ----------- |
/// | Double | `f64`.   |
/// | String | [rust_icu_ustring::UChar] |
/// | Integer | `i32` |
/// | Date | [rust_icu_sys::UDate]; alias for `f64` |
///
/// ## Double as numeric parameter
///
/// According to the [ICU documentation for
/// `umsg_format`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/umsg_8h.html#a90a4b5fe778754e5da52f7c2e5fd6048):
///
/// > for all numeric arguments double is assumed unless the type is explicitly
/// > integer (long).  All choice format arguments must be of type double.
///
/// ## Strings
///
/// We determined by code inspection that the string format must be `rust_icu_ustring::UChar`.
///
/// # Example use
///
/// ```
/// use rust_icu_sys as sys;
/// use rust_icu_common as common;
/// use rust_icu_ustring as ustring;
/// use rust_icu_uloc as uloc;
/// use rust_icu_umsg::{self as umsg, message_format};
/// # use rust_icu_ucal as ucal;
/// # use std::convert::TryFrom;
/// #
/// # struct TzSave(String);
/// # impl Drop for TzSave {
/// #    // Restore the system time zone upon exit.
/// #    fn drop(&mut self) {
/// #        ucal::set_default_time_zone(&self.0);
/// #    }
/// # }
///
/// fn testfn() -> Result<(), common::Error> {
/// # let _ = TzSave(ucal::get_default_time_zone()?);
/// # ucal::set_default_time_zone("Europe/Amsterdam")?;
///   let loc = uloc::ULoc::try_from("en-US")?;
///   let msg = ustring::UChar::try_from(
///     r"Formatted double: {0,number,##.#},
///       Formatted integer: {1,number,integer},
///       Formatted string: {2},
///       Date: {3,date,full}",
///   )?;
///
///   let fmt = umsg::UMessageFormat::try_from(&msg, &loc)?;
///   let hello = ustring::UChar::try_from("Hello! Добар дан!")?;
///   let result = umsg::message_format!(
///     fmt,
///     43.4,
///     31337,
///     hello,
///     0.0,
///   )?;
///
///   assert_eq!(
///     r"Formatted double: 43.4,
///       Formatted integer: 31,337,
///       Formatted string: Hello! Добар дан!,
///       Date: Thursday, January 1, 1970",
///     result
///   );
/// Ok(())
/// }
/// # fn main() -> Result<(), common::Error> {
/// #   testfn()
/// # }
/// ```
///
/// Implements `umsg_format`.
/// Implements `umsg_vformat`.
#[macro_export]
macro_rules! message_format {
    ($dest:expr $(,)?) => {
        $crate::__std::compile_error!("you should not format a message without parameters")
    };
    ($dest:expr, $( {$arg:expr => $t:ident} ),+ $(,)?) => {
        $crate::message_format!($dest, $($crate::checkarg!($arg, $t),)*)
    };
    ($dest:expr, $($arg:expr),+ $(,)?) => {
        unsafe {
            $crate::format_args(&$dest, ($($arg,)*),)
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! checkarg {
    ($e:expr, Double) => {{
        let x: $crate::__std::primitive::f64 = $e;
        x
    }};
    ($e:expr, String) => {{
        let x: $crate::__ustring::UChar = $e;
        x
    }};
    ($e:expr, Integer) => {{
        let x: $crate::__std::primitive::i32 = $e;
        x
    }};
    ($e:expr, Long) => {{
        let x: $crate::__std::primitive::i64 = $e;
        x
    }};
    ($e:expr, Date) => {{
        let x: $crate::__sys::UDate = $e;
        x
    }};
}

#[doc(hidden)]
pub unsafe fn format_args(
    fmt: &UMessageFormat,
    args: impl FormatArgs,
) -> Result<String, common::Error> {
    const CAP: usize = 1024;
    let mut status = common::Error::OK_CODE;
    let mut result = ustring::UChar::new_with_capacity(CAP);

    let total_size =
        args.format(fmt.rep.rep, result.as_mut_c_ptr(), CAP as i32, &mut status) as usize;
    common::Error::ok_or_warning(status)?;

    result.resize(total_size);

    if total_size > CAP {
        args.format(
            fmt.rep.rep,
            result.as_mut_c_ptr(),
            total_size as i32,
            &mut status,
        );
        common::Error::ok_or_warning(status)?;
    }
    String::try_from(&result)
}

mod sealed {
    pub trait Sealed {}
}

/// Traits for types that can be passed to the umsg_format variadic function.
#[doc(hidden)]
pub trait FormatArg: Sealed {
    type Raw;
    fn to_raw(&self) -> Self::Raw;
}

impl Sealed for f64 {}
impl FormatArg for f64 {
    type Raw = f64;
    fn to_raw(&self) -> Self::Raw {
        *self
    }
}

impl Sealed for ustring::UChar {}
impl FormatArg for ustring::UChar {
    type Raw = *const UChar;
    fn to_raw(&self) -> Self::Raw {
        self.as_c_ptr()
    }
}

impl Sealed for i32 {}
impl FormatArg for i32 {
    type Raw = i32;
    fn to_raw(&self) -> Self::Raw {
        *self
    }
}

impl Sealed for i64 {}
impl FormatArg for i64 {
    type Raw = i64;
    fn to_raw(&self) -> Self::Raw {
        *self
    }
}

/// Trait for tuples of elements implementing `FormatArg`.
#[doc(hidden)]
pub trait FormatArgs: Sealed {
    #[doc(hidden)]
    unsafe fn format(
        &self,
        fmt: *const sys::UMessageFormat,
        result: *mut UChar,
        result_length: i32,
        status: *mut UErrorCode,
    ) -> i32;
}

macro_rules! impl_format_args_for_tuples {
    ($(($($param:ident),*),)*) => {
        $(
            impl<$($param: FormatArg,)*> Sealed for ($($param,)*) {}
            impl<$($param: FormatArg,)*> FormatArgs for ($($param,)*) {
                unsafe fn format(
                    &self,
                    fmt: *const sys::UMessageFormat,
                    result: *mut UChar,
                    result_length: i32,
                    status: *mut UErrorCode,
                ) -> i32 {
                    #[allow(non_snake_case)]
                    let ($($param,)*) = self;
                    $(
                        #[allow(non_snake_case)]
                        let $param = $crate::FormatArg::to_raw($param);
                    )*

                    versioned_function!(umsg_format)(
                        fmt,
                        result,
                        result_length,
                        status,
                        $($param,)*
                    )
                }
            }
        )*
    }
}

impl_format_args_for_tuples! {
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
    (A, B, C, D, E, F, G, H, I, J, K, L, M),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_icu_ucal as ucal;

    struct TzSave(String);

    impl Drop for TzSave {
        // Restore the system time zone upon exit.
        fn drop(&mut self) {
            ucal::set_default_time_zone(&self.0).expect("timezone set success");
        }
    }

    #[test]
    fn tzsave() -> Result<(), common::Error> {
        let _ = TzSave(ucal::get_default_time_zone()?);
        ucal::set_default_time_zone("Europe/Amsterdam")?;
        Ok(())
    }

    #[test]
    fn basic() -> Result<(), common::Error> {
        let _ = TzSave(ucal::get_default_time_zone()?);
        ucal::set_default_time_zone("Europe/Amsterdam")?;

        let loc = uloc::ULoc::try_from("en-US")?;
        let msg = ustring::UChar::try_from(
            r"Formatted double: {0,number,##.#},
              Formatted integer: {1,number,integer},
              Formatted string: {2},
              Date: {3,date,full}",
        )?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        let hello = ustring::UChar::try_from("Hello! Добар дан!")?;
        let value: i32 = 31337;
        let result = message_format!(
            fmt,
            { 43.4 => Double },
            { value => Integer },
            { hello.clone() => String },
            { 0.0 => Date }
        )?;
        let result_2 = message_format!(fmt, 43.4, value, hello, 0.0,)?;

        assert_eq!(
            r"Formatted double: 43.4,
              Formatted integer: 31,337,
              Formatted string: Hello! Добар дан!,
              Date: Thursday, January 1, 1970",
            result
        );
        assert_eq!(result, result_2);
        Ok(())
    }

    #[test]
    fn clone() -> Result<(), common::Error> {
        let loc = uloc::ULoc::try_from("en-US-u-tz-uslax")?;
        let msg = ustring::UChar::try_from(r"Formatted double: {0,number,##.#}")?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        let result = message_format!(fmt, 43.43)?;
        assert_eq!(r"Formatted double: 43.4", result);
        Ok(())
    }
}
