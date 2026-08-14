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
//! #   let _tz = TzSave(ucal::get_default_time_zone()?);
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
//!     let mut hello = ustring::UChar::try_from("Hello! Добар дан!")?;
//!     hello.make_z();
//!     let result = umsg::message_format!(
//!       fmt,
//!       { 43.4 => Double },
//!       { 31337 => Integer },
//!       { hello => String },
//!       { 0.0 => Date },
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

mod pattern;

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

    /// How many arguments `umsg_format` will read for this formatter's
    /// pattern, or [None] if that could not be determined.  See
    /// [pattern::required_arg_count].
    required_args: Option<usize>,
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
            required_args: self.required_args,
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
        // `umsg_format` reads its arguments from the pattern, so the pattern
        // decides how many arguments a later format call has to supply.  Work
        // that out once, here, where the pattern is known good: `umsg_open`
        // has just accepted it.
        let required_args = String::try_from(pattern)
            .ok()
            .and_then(|pattern| pattern::required_arg_count(&pattern));
        Ok(UMessageFormat {
            rep: std::rc::Rc::new(Rep { rep }),
            required_args,
        })
    }

    /// Formats `args` into this formatter's message, returning the formatted string.
    ///
    /// This is the explicit-`unsafe` counterpart to the [message_format!] macro: it carries exactly
    /// the same contract, but as an `unsafe fn` it forces the caller to acknowledge that contract
    /// with an `unsafe { .. }` block. Prefer it over the macro when you want the obligation to be
    /// visible at the call site.
    ///
    /// `args` is a tuple of the values to format, one per positional parameter, in order. Each
    /// element's type selects how ICU reads it (see the type table on [message_format!]):
    ///
    /// | Tuple element type | MessageFormat role |
    /// | ------------------ | ------------------ |
    /// | `f64` | `Double` (and `Date`, since [rust_icu_sys::UDate] is an `f64`) |
    /// | `i32` | `Integer` |
    /// | `i64` | `Long` |
    /// | [rust_icu_ustring::UChar] | `String` |
    ///
    /// For example, `(43.4_f64, 31337_i32)` binds a double to parameter `{0}` and an integer to
    /// parameter `{1}`. A single argument uses a one-element tuple, e.g. `(43.4_f64,)`.
    ///
    /// # Safety
    ///
    /// ICU's variadic `umsg_format` derives the number and types of the arguments it reads from the
    /// *pattern* passed to [UMessageFormat::try_from], not from `args`. The two have to agree, or
    /// the call is undefined behavior (segfault or silent memory corruption).
    ///
    /// * **Argument types.** Each element's type must match what the pattern expects at that index,
    ///   e.g. `{0,number}` expects an `f64` while `{0}` and `{0,number,integer}` expect a
    ///   [rust_icu_ustring::UChar] and an `i32` respectively. Nothing checks this, so it is the
    ///   caller's obligation.
    ///
    /// * **Argument count.** `args` must contain at least as many elements as the highest argument
    ///   index referenced by the pattern, plus one. For example the pattern `"String : {1}"` reads
    ///   *two* arguments (indices `0` and `1`). This one *is* checked: too few arguments returns an
    ///   error instead of reading past the end of `args`. Supplying more is harmless. See
    ///   [google/rust_icu#371](https://github.com/google/rust_icu/issues/371).
    ///
    ///   The count check is skipped, leaving the count a caller obligation too, for the patterns
    ///   whose argument count cannot be established: those using named arguments (`{name}`), which
    ///   `umsg_format` rejects anyway, and any pattern the scanner does not recognize.
    ///
    /// # Example
    ///
    /// ```
    /// use rust_icu_common as common;
    /// use rust_icu_ustring as ustring;
    /// use rust_icu_uloc as uloc;
    /// use rust_icu_umsg as umsg;
    /// use std::convert::TryFrom;
    ///
    /// # fn testfn() -> Result<(), common::Error> {
    /// let loc = uloc::ULoc::try_from("en-US")?;
    /// let msg = ustring::UChar::try_from(r"Formatted double: {0,number,##.#}")?;
    /// let fmt = umsg::UMessageFormat::try_from(&msg, &loc)?;
    ///
    /// // SAFETY: the pattern references exactly one argument, a double, matching the tuple below.
    /// let result = unsafe { fmt.try_format((43.4_f64,)) }?;
    /// assert_eq!("Formatted double: 43.4", result);
    /// # Ok(())
    /// # }
    /// # testfn().unwrap();
    /// ```
    ///
    /// Implements `umsg_format`.
    pub unsafe fn try_format(&self, args: impl FormatArgs) -> Result<String, common::Error> {
        format_args(self, args)
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
/// //     formatter, [{ value => <type_assertion> }, ...]);
/// let result = umsg::message_format!(formatter, { 31337 => Double });
/// ```
///
/// Each fragment `{ value => <type_assertion> }` represents a single positional parameter binding
/// for the pattern in `formatter`.  The first fragment corresponds to the positional parameter `0`
/// (which, if an integer, would be referred to as `{0,number,integer}` in a MessageFormat
/// pattern).  Since the original C API that this rust library is generated for uses variadic
/// functions for parameter passing, it is very important that the programmer matches the actual
/// parameter types to the types that are expected in the pattern.
///
/// # Warning: memory-safety hazards
///
/// For backward compatibility this macro is callable from safe code, but it is **not** actually
/// sound: it expands to a call into the ICU C *variadic* function `umsg_format`, which derives the
/// number and types of the arguments it reads from the *pattern* passed to
/// [UMessageFormat::try_from], not from the actual arguments supplied here. The two have to agree.
/// Violating either of the following is undefined behavior and typically manifests as a segfault or
/// silent memory corruption:
///
/// * **Argument types.** The `=> <type_assertion>` you write for each argument must match the type
///   the pattern expects at that index. `{0}` and `{0,number,integer}` expect a string and an
///   integer respectively, while `{0,number}` expects a double; passing the wrong Rust type (which
///   the `=> ..` assertion pins only on the Rust side) makes ICU reinterpret the argument's bytes
///   as the wrong C type -- e.g. an `f64` read back as a `UChar*` and dereferenced. Nothing checks
///   this, so it is the caller's obligation.
///
/// * **Argument count.** You must supply at least as many arguments as the highest argument index
///   referenced by the pattern, plus one. For example the pattern `"String : {1}"` references index
///   `1`, so ICU reads *two* variadic arguments (indices `0` and `1`). Note that argument indices
///   need not be contiguous or start at `0`, but every index the pattern mentions must be backed by
///   an argument here. Supplying *more* arguments than the pattern references is harmless. See
///   [google/rust_icu#371](https://github.com/google/rust_icu/issues/371).
///
///   The count *is* checked before the variadic call happens: supplying too few arguments returns
///   an error rather than letting ICU read past the end of the arguments. The check is skipped, and
///   the count becomes a caller obligation like the types, for patterns whose argument count cannot
///   be established: those using named arguments (`{name}`), which `umsg_format` rejects anyway,
///   and any pattern the scanner does not recognize.
///
/// If you would rather make this obligation explicit at the call site, use the
/// [UMessageFormat::try_format] method instead, which is an `unsafe fn` carrying the same contract
/// and therefore requires an `unsafe { .. }` block from the caller.
///
/// In general this is very brittle, and an API in a more modern lanugage, or a contemporary C++
/// flavor would probably take a different route were the library to be written today.  The rust
/// binding tries to make the API use a bit more palatable by requiring that the programmer
/// explicitly specifies a type for each of the parameters to be passed into the formatter.
///
/// The supported types are not those of a full rust system, but rather a very restricted subset
/// of types that MessageFormat supports:
///
/// | Type | Rust Type | Notes |
/// | ---- | --------- | ----------- |
/// | Double | `f64` | Any numeric parameter not specifically designated as different type, is always a double. See section below on Doubles. |
/// | String | [rust_icu_ustring::UChar] | |
/// | Integer | `i32` | |
/// | Date | [rust_icu_sys::UDate] (alias for `f64`) | Is used to format dates.  Depending on the date format requested in the pattern used in [UMessageFormat], the end result of date formatting could be one of a wide variety of [date formats](http://userguide.icu-project.org/formatparse/datetime).|
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
/// # let _tz = TzSave(ucal::get_default_time_zone()?);
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
///   let mut hello = ustring::UChar::try_from("Hello! Добар дан!")?;
///   hello.make_z();
///   let result = umsg::message_format!(
///     fmt,
///     { 43.4 => Double },
///     { 31337 => Integer },
///     { hello => String },
///     { 0.0 => Date },
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
#[macro_export]
macro_rules! message_format {
    ($dest:expr $(,)?) => {
        $crate::__std::compile_error!("you should not format a message without parameters")
    };
    ($dest:expr, $( {$arg:expr => $t:ident} ),+ $(,)?) => {
        unsafe {
            $crate::format_args(&$dest, ($($crate::checkarg!($arg, $t),)*))
        }
    };
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
pub unsafe fn format_args<A: FormatArgs>(
    fmt: &UMessageFormat,
    args: A,
) -> Result<String, common::Error> {
    // `umsg_format` is variadic and reads as many arguments as the pattern
    // refers to, whatever the caller passed. Supplying too few makes it read
    // past the end of the argument list, which is undefined behavior: with a
    // string argument it dereferences whatever it finds there and usually
    // segfaults. Refuse the call instead, when the pattern says how many
    // arguments it needs. See
    // https://github.com/google/rust_icu/issues/371.
    if let Some(required) = fmt.required_args {
        if A::ARITY < required {
            return Err(common::Error::Wrapper(anyhow::anyhow!(
                "message pattern needs {} argument(s) because it refers to argument index {}, \
                 but {} argument(s) were supplied",
                required,
                required - 1,
                A::ARITY,
            )));
        }
    }

    const CAP: usize = 1024;
    let mut status = common::Error::OK_CODE;
    let mut result = ustring::UChar::new_with_capacity(CAP);

    let total_size =
        args.format(fmt.rep.rep, result.as_mut_c_ptr(), CAP as i32, &mut status) as usize;

    // ICU is inconsistent about an output that does not fit: some paths raise
    // U_BUFFER_OVERFLOW_ERROR, others truncate silently and return the length
    // they wanted. Check both, as rust_icu_ulistformatter and
    // common::buffered_string_method_with_retry do.
    if status == sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR
        || (common::Error::is_ok(status) && total_size > CAP)
    {
        // The first call only measured the output. Clear the status before
        // calling again: an ICU entry point does no work when the status it
        // receives already holds an error, so a retry that reuses a failed
        // status is a silent no-op.
        status = common::Error::OK_CODE;
        result.resize(total_size);
        args.format(
            fmt.rep.rep,
            result.as_mut_c_ptr(),
            total_size as i32,
            &mut status,
        );
        common::Error::ok_or_warning(status)?;
    } else {
        common::Error::ok_or_warning(status)?;
        result.resize(total_size);
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
    /// The number of arguments in this tuple.
    #[doc(hidden)]
    const ARITY: usize;

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
                const ARITY: usize = [$($crate::__std::stringify!($param)),*].len();

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
        let _tz = TzSave(ucal::get_default_time_zone()?);
        ucal::set_default_time_zone("Europe/Amsterdam")?;
        Ok(())
    }

    #[test]
    fn basic() -> Result<(), common::Error> {
        let _tz = TzSave(ucal::get_default_time_zone()?);
        ucal::set_default_time_zone("Europe/Amsterdam")?;

        let loc = uloc::ULoc::try_from("en-US")?;
        let msg = ustring::UChar::try_from(
            r"Formatted double: {0,number,##.#},
              Formatted integer: {1,number,integer},
              Formatted string: {2},
              Date: {3,date,full}",
        )?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        let mut hello = ustring::UChar::try_from("Hello! Добар дан!")?;
        hello.make_z();
        let value: i32 = 31337;
        let result = message_format!(
            fmt,
            { 43.4 => Double },
            { value => Integer },
            { hello => String },
            { 0.0 => Date }
        )?;

        assert_eq!(
            r"Formatted double: 43.4,
              Formatted integer: 31,337,
              Formatted string: Hello! Добар дан!,
              Date: Thursday, January 1, 1970",
            result
        );
        Ok(())
    }

    /// A formatted result longer than the initial buffer must still format.
    ///
    /// ICU raises U_BUFFER_OVERFLOW_ERROR when the output does not fit, and the
    /// formatter has to grow the buffer and call again. Regression test for
    /// https://github.com/google/rust_icu/issues/390.
    #[test]
    fn format_longer_than_initial_buffer() -> Result<(), common::Error> {
        let loc = uloc::ULoc::try_from("en-US")?;
        let msg = ustring::UChar::try_from("{0}")?;
        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;

        // The initial buffer inside `format_args` holds 1024 UTF-16 units.
        let long = "x".repeat(4096);
        // `umsg_format` is variadic, so a String argument carries no length and
        // ICU reads it until a NUL. `UChar::try_from` does not add one, so the
        // argument must be terminated explicitly or ICU reads past its end.
        let mut arg = ustring::UChar::try_from(long.as_str())?;
        arg.make_z();

        let result = message_format!(fmt, { arg => String })?;
        assert_eq!(long.len(), result.len());
        assert_eq!(long, result);
        Ok(())
    }

    #[test]
    fn clone() -> Result<(), common::Error> {
        let loc = uloc::ULoc::try_from("en-US-u-tz-uslax")?;
        let msg = ustring::UChar::try_from(r"Formatted double: {0,number,##.#}")?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        #[allow(clippy::redundant_clone)]
        let result = message_format!(fmt.clone(), { 43.43 => Double })?;
        assert_eq!(r"Formatted double: 43.4", result);
        Ok(())
    }

    #[test]
    fn try_format_method() -> Result<(), common::Error> {
        let _tz = TzSave(ucal::get_default_time_zone()?);
        ucal::set_default_time_zone("Europe/Amsterdam")?;

        let loc = uloc::ULoc::try_from("en-US")?;
        let msg = ustring::UChar::try_from(
            r"Formatted double: {0,number,##.#},
              Formatted integer: {1,number,integer},
              Formatted string: {2},
              Date: {3,date,full}",
        )?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        let mut hello = ustring::UChar::try_from("Hello! Добар дан!")?;
        hello.make_z();
        let value: i32 = 31337;
        // SAFETY: the four tuple elements match the count and types referenced by the pattern.
        let result = unsafe { fmt.try_format((43.4_f64, value, hello, 0.0_f64)) }?;

        assert_eq!(
            r"Formatted double: 43.4,
              Formatted integer: 31,337,
              Formatted string: Hello! Добар дан!,
              Date: Thursday, January 1, 1970",
            result
        );
        Ok(())
    }

    /// Demonstrates the type-mismatch memory-unsafety hazard of `message_format!`.
    ///
    /// The pattern `{0}` (an argument with no explicit format) is treated by ICU's variadic
    /// `umsg_format` as a *string* argument, i.e. ICU performs a `va_arg(ap, UChar*)` for slot 0.
    /// Here, however, the caller's `=> Double` type assertion pushes an `f64` into that slot. Under
    /// the C ABI a double and a pointer are passed in different register classes, so ICU reads a
    /// `UChar*` from a slot the caller never populated with a pointer, then dereferences that
    /// garbage value -- undefined behavior that manifests as a segfault (or memory corruption).
    ///
    /// The `=> Double` assertion checked by [`checkarg!`] only guarantees the *Rust* type of the
    /// argument; nothing reconciles it against what the *pattern* expects, so this compiles and
    /// runs from entirely safe-looking code. This is the type-mismatch counterpart to the
    /// argument-count mismatch reported in google/rust_icu#371, and unlike a count mismatch it
    /// cannot be caught by simply scanning the pattern for the highest argument index.
    ///
    /// This test is `#[ignore]`d because triggering undefined behavior would abort the whole test
    /// binary. Run it deliberately, in isolation, to observe the crash:
    ///
    /// ```text
    /// cargo test -p rust_icu_umsg -- --ignored --exact tests::type_mismatch_is_undefined_behavior
    /// ```
    #[test]
    #[ignore = "deliberately triggers undefined behavior (segfault); run manually to reproduce"]
    fn type_mismatch_is_undefined_behavior() -> Result<(), common::Error> {
        let loc = uloc::ULoc::try_from("en-US")?;
        // `{0}` with no explicit format => ICU expects a string (UChar*) argument in slot 0.
        let msg = ustring::UChar::try_from(r"Value: {0}")?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        // ... but we assert `Double`, pushing an f64 where ICU will read a pointer. The result of
        // the `?` is never reached: `umsg_format` dereferences a garbage pointer first. Note this
        // is reachable from entirely safe code -- `message_format!` requires no `unsafe` block.
        let result = message_format!(fmt, { 43.4 => Double })?;

        // Unreachable in practice. If this line is ever reached, the type mismatch stopped being
        // undefined behavior on this platform/ICU version, and this test should be revisited.
        panic!(
            "expected undefined behavior, but formatting returned: {:?}",
            result
        );
    }

    /// Too few arguments are rejected instead of read past the end of the varargs.
    ///
    /// This is the pattern reported in
    /// [google/rust_icu#371](https://github.com/google/rust_icu/issues/371). `{1}` references
    /// argument index 1, so ICU's variadic `umsg_format` reads *two* arguments (indices 0 and 1).
    /// Only one is supplied here. Before the check this read argument 1 from past the end of the
    /// arguments actually passed and -- because a bare `{1}` is a string argument -- dereferenced
    /// that indeterminate value as a `UChar*`, which segfaults.
    #[test]
    fn too_few_arguments_is_an_error() -> Result<(), common::Error> {
        let loc = uloc::ULoc::try_from("en-US-u-tz-uslax")?;
        let msg = ustring::UChar::try_from(r"String : {1}")?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        let mut string = ustring::UChar::try_from("Hello!")?;
        string.make_z();
        let result = message_format!(fmt, { string => String });

        let err = result.expect_err("one argument must not satisfy a pattern needing two");
        assert!(
            matches!(err, common::Error::Wrapper(_)),
            "expected a wrapper error, got: {:?}",
            err
        );
        Ok(())
    }

    /// The same check, for a pattern where the indices are contiguous.
    #[test]
    fn too_few_arguments_is_an_error_for_contiguous_indices() -> Result<(), common::Error> {
        let loc = uloc::ULoc::try_from("en-US")?;
        let msg = ustring::UChar::try_from(r"{0}{1}{2}{3}{4}{5}{6}{7}")?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        let mut string = ustring::UChar::try_from("x")?;
        string.make_z();
        let result = message_format!(
            fmt,
            { string.clone() => String },
            { string.clone() => String },
            { string => String }
        );

        assert!(result.is_err(), "expected an error, got: {:?}", result);
        Ok(())
    }

    /// Supplying more arguments than the pattern references stays harmless.
    #[test]
    fn extra_arguments_are_allowed() -> Result<(), common::Error> {
        let loc = uloc::ULoc::try_from("en-US")?;
        let msg = ustring::UChar::try_from(r"Only: {0}")?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        let mut first = ustring::UChar::try_from("first")?;
        first.make_z();
        let mut second = ustring::UChar::try_from("second")?;
        second.make_z();

        let result = message_format!(fmt, { first => String }, { second => String })?;
        assert_eq!("Only: first", result);
        Ok(())
    }

    /// An argument nested in a submessage counts towards the required number.
    #[test]
    fn nested_arguments_count() -> Result<(), common::Error> {
        let loc = uloc::ULoc::try_from("en-US")?;
        let msg = ustring::UChar::try_from(r"{0,plural,one{one file}other{# files in {1}}}")?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        let mut dir = ustring::UChar::try_from("/tmp")?;
        dir.make_z();

        // `{1}` sits inside a submessage, but ICU still reads two arguments.
        let too_few = message_format!(fmt, { 3.0 => Double });
        assert!(too_few.is_err(), "expected an error, got: {:?}", too_few);

        let result = message_format!(fmt, { 3.0 => Double }, { dir => String })?;
        assert_eq!("3 files in /tmp", result);
        Ok(())
    }

    /// A quoted argument is literal text, and does not require an argument.
    #[test]
    fn quoted_arguments_are_not_required() -> Result<(), common::Error> {
        let loc = uloc::ULoc::try_from("en-US")?;
        let msg = ustring::UChar::try_from(r"'{1}' is literal, {0} is not")?;

        let fmt = crate::UMessageFormat::try_from(&msg, &loc)?;
        let mut arg = ustring::UChar::try_from("this")?;
        arg.make_z();

        let result = message_format!(fmt, { arg => String })?;
        assert_eq!("{1} is literal, this is not", result);
        Ok(())
    }
}
