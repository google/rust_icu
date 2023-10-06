// Copyright 2021 Luis Cáceres
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

//! # ICU character conversion support for rust
//!
//! This crate provides [character encoding translation](https://en.wikipedia.org/wiki/Character_encoding#Character_encoding_translation),
//! based on the conversion functions implemented by the ICU library.
//! Specifically the functionality exposed through its C API, as available in the [header
//! `ucnv.h`](https://unicode-org.github.io/icu-docs/apidoc/released/icu4c/ucnv_8h.html).
//!
//! The main type is [UConverter], which can be created using `UConverter::open` with an encoding
//! name (as a `&str`). This type provides conversion functions between UTF-16 and the
//! provided encoding.
//!
//! This crate also provides [utf8::Converter] as a convenience type to work directly with UTF-8
//! strings, such as with Rust's `&str` and `String` types.
//!
//! For more information on ICU conversion, an interested reader can check out the
//! [conversion documentation on the ICU user guide](https://unicode-org.github.io/icu/userguide/conversion/).
//!
//! > Are you missing some features from this crate?  Consider [reporting an
//! issue](https://github.com/google/rust_icu/issues) or even [contributing the
//! functionality](https://github.com/google/rust_icu/pulls).

use std::{
    ffi::{CStr, CString},
    ops::Range,
    os::raw,
    ptr::{null_mut, NonNull},
};

use {
    rust_icu_common as common, rust_icu_sys as sys, rust_icu_sys::versioned_function,
};

pub mod utf8;

/// Get an iterator over all canonical converter names available to ICU.
///
/// The [AvailableConverters] iterator efficiently implements [Iterator::count] and [Iterator::nth]
/// to avoid calling `ucnv_getAvailableName` unnecessarily.
///
/// This interface wraps around `ucnv_countAvailable` and `ucnv_getAvailableName`.
pub fn available_converters() -> AvailableConverters {
    AvailableConverters {
        n: 0,
        count: unsafe { versioned_function!(ucnv_countAvailable)() } as u32,
    }
}

/// See [available_converters()]
pub struct AvailableConverters {
    n: u32,
    count: u32,
}

impl AvailableConverters {
    #[inline(always)]
    fn elements_left(&self) -> usize {
        self.count.saturating_sub(self.n) as usize
    }
}

impl Iterator for AvailableConverters {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let name_c_str = unsafe { versioned_function!(ucnv_getAvailableName)(self.n as i32) };
        if name_c_str.is_null() {
            return None;
        }

        self.n += 1;
        unsafe {
            Some(
                CStr::from_ptr(name_c_str)
                    .to_str()
                    .expect("converter name should be UTF-8 compatible")
                    .to_string(),
            )
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.elements_left();
        (count, Some(count))
    }

    fn count(self) -> usize {
        self.elements_left()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.elements_left() {
            self.n = self.count;
            None
        } else {
            self.n += n as u32;
            self.next()
        }
    }
}

impl ExactSizeIterator for AvailableConverters {}

/// Get an iterator over all aliases for the provided converter name (which may not necessarily be
/// the canonical converter name).
///
/// The [Aliases] iterator efficiently implements [Iterator::count] and [Iterator::nth] to avoid
/// calling `ucnv_getAlias` unnecessarily.
///
/// This interface wraps around `ucnv_countAliases` and `ucnv_getAlias`.
pub fn aliases(name: &str) -> Aliases {
    let name = CString::new(name).expect("converter name should not contain NUL");

    let mut status = sys::UErrorCode::U_ZERO_ERROR;
    let count = unsafe { versioned_function!(ucnv_countAliases)(name.as_ptr(), &mut status) };
    let is_ambiguous = status == sys::UErrorCode::U_AMBIGUOUS_ALIAS_WARNING;
    Aliases {
        n: 0,
        count,
        name,
        is_ambiguous,
    }
}

/// See [aliases()]
pub struct Aliases {
    n: u16,
    count: u16,
    name: CString,
    is_ambiguous: bool,
}

impl Aliases {
    #[inline(always)]
    fn elements_left(&self) -> usize {
        self.count.saturating_sub(self.n) as usize
    }

    /// Whether or not the converter name provided to [aliases()] is an ambiguous name.
    ///
    /// This value is the cached result of checking that the `status` of calling `ucnv_countAliases`
    /// was set to `U_AMBIGUOUS_ALIAS_WARNING`.
    pub fn is_ambiguous(&self) -> bool {
        self.is_ambiguous
    }
}

impl Iterator for Aliases {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let alias_c_str = unsafe {
            let mut status = sys::UErrorCode::U_ZERO_ERROR;
            versioned_function!(ucnv_getAlias)(self.name.as_ptr(), self.n, &mut status)
        };
        if alias_c_str.is_null() {
            return None;
        }

        self.n += 1;
        unsafe {
            Some(
                CStr::from_ptr(alias_c_str)
                    .to_str()
                    .expect("alias name should be UTF-8 compatible")
                    .to_string(),
            )
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.elements_left();
        (count, Some(count))
    }

    fn count(self) -> usize {
        self.elements_left()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.elements_left() {
            self.n = self.count;
            None
        } else {
            self.n += n as u16;
            self.next()
        }
    }
}

impl ExactSizeIterator for Aliases {}

/// The result of a feed/stream operation on a converter.
///
/// See [UConverter] for examples.
pub struct FeedResult {
    /// Indicates how many units have been written to the destination buffer.
    pub dst_consumed: usize,

    /// Indicates how many units of the source buffer have been read and processed by the converter.
    pub src_consumed: usize,

    /// The status error reported by the underlying ICU call.
    pub result: Result<(), common::Error>,
}

/// The converter type that provides conversion to/from UTF-16.
///
/// This object can perform conversion of single strings (using the
/// [UConverter::convert_to_uchars] and [UConverter::convert_from_uchars] functions) or of a stream
/// of text data (using the [UConverter::feed_to_uchars] and [UConverter::feed_from_uchars] functions
/// to feed and process more input data).
///
/// Each conversion direction has separate state, which means you can use the `feed_to_uchars`
/// function at the same time as the `feed_from_uchars` to process two streams simultaneously *in
/// the same thread* (note that this type isn't [Sync]).
///
///
/// ## Examples
///
/// ### Single-string conversion
///
/// The single-string conversion functions are straightforward to use.
///
/// ```
/// # use rust_icu_ucnv::UConverter;
/// #
/// let mut converter = UConverter::open("UTF-8").unwrap();
///
/// let utf8_string = "スーパー";
/// let utf16_string: Vec<u16> = converter.convert_to_uchars(utf8_string.as_bytes()).unwrap();
///
/// assert_eq!(
///     utf8_string,
///     std::str::from_utf8(&converter.convert_from_uchars(&utf16_string).unwrap()).unwrap()
/// );
/// ```
///
/// ### Streaming conversion
///
/// The feeding/streaming functions take a mutable slice as destination buffer and an immutable
/// slice as source buffer. These functions consume the source buffer and write the converted text
/// into the destination buffer until one or the other have been fully consumed, or some conversion
/// error happens. These functions return the error (if any) and how much of the destination/source
/// buffers has been consumed. The idea is that after one of the buffer has been fully consumed, you
/// grab another buffer chunk (whether source or destination) and call the function again. Hence,
/// a processing loop might look like this:
///
/// ```
/// # use rust_icu_ucnv::UConverter;
/// use rust_icu_common as common;
/// use rust_icu_sys as sys;
///
/// # const UTF8_STRING: &str = "Shift_JIS（シフトジス）は、コンピュータ上で日本語を含む文字列を表現するために\
/// # 用いられる文字コードの一つ。シフトJIS（シフトジス）と表記されることもある。";
/// #
/// let mut converter = UConverter::open("UTF-8").unwrap();
///
/// # let mut dst_buffer: Vec<u16> = Vec::new();
/// # dst_buffer.resize(1024, 0);
/// #
/// # let mut dst_chunks = dst_buffer.chunks_mut(8);
/// # let mut get_dst_chunk = move || dst_chunks.next();
/// #
/// # let mut src_chunks = UTF8_STRING.as_bytes().chunks(6);
/// # let mut get_src_chunk = move || src_chunks.next();
/// #
/// let mut dst: &mut [u16] = get_dst_chunk().unwrap();
/// let mut src: &[u8] = get_src_chunk().unwrap();
///
/// // reset any previous state
/// converter.reset_to_uchars();
/// loop {
///     let res = converter.feed_to_uchars(dst, src);
///     match res.result {
///         Ok(_) | Err(common::Error::Sys(sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR)) => {
///             dst = dst.split_at_mut(res.dst_consumed).1;
///             src = src.split_at(res.src_consumed).1;
///         }
///         _ => panic!("conversion error"),
///     }
///
///     if dst.is_empty() {
///         dst = get_dst_chunk().unwrap();
///     }
///     if src.is_empty() {
///         src = match get_src_chunk() {
///             None => break,
///             Some(src) => src,
///         };
///     }
/// }
/// ```
#[derive(Debug)]
pub struct UConverter(NonNull<sys::UConverter>);

unsafe impl Send for UConverter {}

impl UConverter {
    /// Attempts to open a converter with the given encoding name.
    ///
    /// This function wraps around `ucnv_open`.
    pub fn open(name: &str) -> Result<Self, common::Error> {
        let name_c_str = CString::new(name).expect("converter name must not contain NUL");

        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let converter_ptr =
            unsafe { versioned_function!(ucnv_open)(name_c_str.as_ptr(), &mut status) };
        common::Error::ok_or_warning(status)?;

        Ok(Self(
            NonNull::new(converter_ptr).expect("converter pointer should not be null"),
        ))
    }

    /// Attempts to clone a given converter.
    ///
    /// This function wraps around `ucnv_safeClone`.
    pub fn try_clone(&self) -> Result<Self, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let converter_ptr = unsafe {
            versioned_function!(ucnv_safeClone)(
                self.0.as_ptr(),
                null_mut(),
                null_mut(),
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;

        Ok(Self(
            NonNull::new(converter_ptr).expect("converter pointer should not be null"),
        ))
    }

    /// Determines whether the converter contains ambiguous mappings of the same character.
    ///
    /// This function wraps around `ucnv_isAmbiguous`.
    pub fn has_ambiguous_mappings(&self) -> bool {
        unsafe { versioned_function!(ucnv_isAmbiguous)(self.0.as_ptr()) != 0 }
    }

    /// Attempts to get the canonical name of the converter.
    ///
    /// This function wraps around `ucnv_getName`.
    pub fn name(&self) -> Result<&str, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let name_c_str = unsafe { versioned_function!(ucnv_getName)(self.0.as_ptr(), &mut status) };
        common::Error::ok_or_warning(status)?;

        unsafe {
            Ok(CStr::from_ptr(name_c_str)
                .to_str()
                .expect("converter name is UTF-8 compatible"))
        }
    }

    /// Resets the converter to a default state.
    ///
    /// This is equivalent to calling both [UConverter::reset_to_uchars] and [UConverter::reset_from_uchars].
    ///
    /// This function wraps around `ucnv_reset`.
    pub fn reset(&mut self) {
        unsafe { versioned_function!(ucnv_reset)(self.0.as_ptr()) }
    }

    /// Resets the `*_to_uchars` part of the converter to a default state.
    ///
    /// It is necessary to call this function when you want to start processing a new data stream
    /// using [UConverter::feed_to_uchars].
    ///
    /// This function wraps around `ucnv_resetToUnicode`.
    pub fn reset_to_uchars(&mut self) {
        unsafe { versioned_function!(ucnv_resetToUnicode)(self.0.as_ptr()) }
    }

    /// Resets the `*_from_uchars` part of the converter to a default state.
    ///
    /// It is necessary to call this function when you want to start processing a new data stream
    /// using [UConverter::feed_from_uchars].
    ///
    /// This function wraps around `ucnv_resetFromUnicode`.
    pub fn reset_from_uchars(&mut self) {
        unsafe { versioned_function!(ucnv_resetFromUnicode)(self.0.as_ptr()) }
    }

    /// Feeds more encoded data to be decoded to UTF-16 and put in the provided destination buffer.
    ///
    /// Make sure to call [UConverter::reset_to_uchars] before processing a new data stream.
    ///
    /// This function wraps around `ucnv_toUnicode`.
    pub fn feed_to_uchars(&mut self, dst: &mut [sys::UChar], src: &[u8]) -> FeedResult {
        self.feed_to(dst, src)
    }

    /// Feeds more UTF-16 to be encoded and put in the provided destination buffer.
    ///
    /// Make sure to call [UConverter::reset_from_uchars] before processing a new data stream.
    ///
    /// This function wraps around `ucnv_fromUnicode`.
    pub fn feed_from_uchars(&mut self, dst: &mut [u8], src: &[sys::UChar]) -> FeedResult {
        self.feed_from(dst, src)
    }

    /// Performs single-string conversion from an encoded string to a UTF-16 string.
    ///
    /// Note that this function resets the `*_to_uchars` state before conversion.
    pub fn convert_to_uchars(&mut self, src: &[u8]) -> Result<Vec<sys::UChar>, common::Error> {
        self.reset_to_uchars();

        self.convert_to(src)
    }

    /// Performs single-string conversion from a UTF-16 string to an encoded string.
    ///
    /// Note that this function resets the `*_from_uchars` state before conversion.
    pub fn convert_from_uchars(&mut self, src: &[sys::UChar]) -> Result<Vec<u8>, common::Error> {
        self.reset_from_uchars();

        self.convert_from(src)
    }
}

impl FeedConverterRaw for UConverter {
    type ToUnit = sys::UChar;
    type FromUnit = u8;

    unsafe fn feed_to_raw(
        &mut self,
        dst: &mut Range<*mut Self::ToUnit>,
        src: &mut Range<*const Self::FromUnit>,
        should_flush: bool,
    ) -> sys::UErrorCode {
        // `ucnv_toUnicode` takes a c_char pointer
        let mut src_range = Range {
            start: src.start as *const raw::c_char,
            end: src.end as *const raw::c_char,
        };

        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        versioned_function!(ucnv_toUnicode)(
            self.0.as_ptr(),
            &mut dst.start,
            dst.end,
            &mut src_range.start,
            src_range.end,
            null_mut(),
            should_flush.into(),
            &mut status,
        );

        // update actual src range
        src.start = src_range.start as *const u8;

        return status;
    }

    unsafe fn feed_from_raw(
        &mut self,
        dst: &mut Range<*mut Self::FromUnit>,
        src: &mut Range<*const Self::ToUnit>,
        should_flush: bool,
    ) -> sys::UErrorCode {
        // `ucnv_fromUnicode` takes a c_char pointer
        let mut dst_range = Range {
            start: dst.start as *mut raw::c_char,
            end: dst.end as *mut raw::c_char,
        };

        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        versioned_function!(ucnv_fromUnicode)(
            self.0.as_ptr(),
            &mut dst_range.start,
            dst_range.end,
            &mut src.start,
            src.end,
            null_mut(),
            should_flush.into(),
            &mut status,
        );

        // update actual src range
        dst.start = dst_range.start as *mut u8;

        return status;
    }
}

// A private helper trait used to implement single-string conversion (convert_* functions) and
// feeding conversion (feed_* functions) on top of the raw feeding/streaming API exposed by ICU.
trait FeedConverterRaw {
    // `dst` type when calling *_to functions
    type ToUnit;

    // `dst` type when calling *_from functions
    type FromUnit;

    // These functions essentially wrap the ICU call.
    unsafe fn feed_to_raw(
        &mut self,
        dst: &mut Range<*mut Self::ToUnit>,
        src: &mut Range<*const Self::FromUnit>,
        should_flush: bool,
    ) -> sys::UErrorCode;
    unsafe fn feed_from_raw(
        &mut self,
        dst: &mut Range<*mut Self::FromUnit>,
        src: &mut Range<*const Self::ToUnit>,
        should_flush: bool,
    ) -> sys::UErrorCode;

    #[inline(always)]
    fn feed_to(&mut self, dst: &mut [Self::ToUnit], src: &[Self::FromUnit]) -> FeedResult {
        let mut dst_range = dst.as_mut_ptr_range();
        let mut src_range = src.as_ptr_range();

        let status = unsafe { self.feed_to_raw(&mut dst_range, &mut src_range, false) };

        FeedResult {
            dst_consumed: unsafe { dst_range.start.offset_from(dst.as_mut_ptr()) } as usize,
            src_consumed: unsafe { src_range.start.offset_from(src.as_ptr()) } as usize,
            result: common::Error::ok_or_warning(status),
        }
    }

    #[inline(always)]
    fn feed_from(&mut self, dst: &mut [Self::FromUnit], src: &[Self::ToUnit]) -> FeedResult {
        let mut dst_range = dst.as_mut_ptr_range();
        let mut src_range = src.as_ptr_range();

        let status = unsafe { self.feed_from_raw(&mut dst_range, &mut src_range, false) };

        FeedResult {
            dst_consumed: unsafe { dst_range.start.offset_from(dst.as_mut_ptr()) } as usize,
            src_consumed: unsafe { src_range.start.offset_from(src.as_ptr()) } as usize,
            result: common::Error::ok_or_warning(status),
        }
    }

    #[inline(always)]
    fn convert_to(&mut self, src: &[Self::FromUnit]) -> Result<Vec<Self::ToUnit>, common::Error> {
        let mut buf: Vec<Self::ToUnit> = Vec::with_capacity(src.len());

        let mut dst_range = Range {
            start: buf.as_mut_ptr(),
            end: unsafe { buf.as_mut_ptr().add(buf.capacity()) },
        };
        let mut src_range = src.as_ptr_range();

        loop {
            unsafe {
                let status = self.feed_to_raw(&mut dst_range, &mut src_range, true);

                // calculate how many converted bytes have been written to the buffer to update
                // its length
                let written_bytes = dst_range.start.offset_from(buf.as_mut_ptr()) as usize;
                buf.set_len(written_bytes);

                if status != sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR {
                    // bail out and let the caller handle errors (if there are any)
                    common::Error::ok_or_warning(status)?;
                    return Ok(buf);
                } else {
                    // resize capacity
                    buf.reserve(buf.len());

                    // ensure dst_range points to new buffer
                    dst_range = Range {
                        start: buf.as_mut_ptr().add(buf.len()),
                        end: buf.as_mut_ptr().add(buf.capacity()),
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn convert_from(&mut self, src: &[Self::ToUnit]) -> Result<Vec<Self::FromUnit>, common::Error> {
        let mut buf: Vec<Self::FromUnit> = Vec::with_capacity(src.len());

        let mut dst_range = Range {
            start: buf.as_mut_ptr(),
            end: unsafe { buf.as_mut_ptr().add(buf.capacity()) },
        };
        let mut src_range = src.as_ptr_range();

        loop {
            unsafe {
                let status = self.feed_from_raw(&mut dst_range, &mut src_range, true);

                // calculate how many converted bytes have been written to the buffer to update
                // its length
                let written_bytes = dst_range.start.offset_from(buf.as_mut_ptr()) as usize;
                buf.set_len(written_bytes);

                if status != sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR {
                    // bail out and let the caller handle errors (if there are any)
                    common::Error::ok_or_warning(status)?;
                    return Ok(buf);
                } else {
                    // resize capacity
                    buf.reserve(buf.len());

                    // ensure dst_range points to new buffer
                    dst_range = Range {
                        start: buf.as_mut_ptr().add(buf.len()),
                        end: buf.as_mut_ptr().add(buf.capacity()),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::UConverter;

    #[test]
    fn test_available_converter_names() {
        let converters = available_converters();
        let converters_count = converters.len();

        let mut count = 0usize;
        for _ in converters {
            count += 1;
        }

        assert_eq!(converters_count, count);
    }

    #[test]
    fn test_converter_aliases() {
        let aliases = aliases("SHIFT_JIS");
        let aliases_count = aliases.len();

        let mut count = 0usize;
        for _ in aliases {
            count += 1;
        }

        assert_eq!(aliases_count, count);
    }

    #[test]
    fn test_utf8_conversion() {
        let mut converter = UConverter::open("UTF-8").unwrap();

        let utf8_string = "スーパー";
        let utf16_string: Vec<u16> = converter.convert_to_uchars(utf8_string.as_bytes()).unwrap();

        assert_eq!(
            utf8_string,
            std::str::from_utf8(&converter.convert_from_uchars(&utf16_string).unwrap()).unwrap()
        );
    }

    #[test]
    fn test_shiftjis_conversion() {
        const SHIFT_JIS_STRING: [u8; 8] = [0x83, 0x58, 0x81, 0x5B, 0x83, 0x70, 0x81, 0x5B];

        let mut converter = UConverter::open("SHIFT-JIS").unwrap();

        let utf16_string = converter.convert_to_uchars(&SHIFT_JIS_STRING).unwrap();

        assert_eq!(
            SHIFT_JIS_STRING.iter().copied().collect::<Vec<u8>>(),
            converter
                .convert_from_uchars(utf16_string.as_slice())
                .unwrap()
        );
    }

    #[test]
    fn test_shiftjis_feeding() {
        const UTF8_STRING: &str =
            "Shift_JIS（シフトジス）は、コンピュータ上で日本語を含む文字列を表現するために\
        用いられる文字コードの一つ。シフトJIS（シフトジス）と表記されることもある。";

        let mut converter = UConverter::open("UTF-8").unwrap();

        let mut dst_buffer: Vec<u16> = Vec::new();
        dst_buffer.resize(1024, 0);

        let mut dst_chunks = dst_buffer.chunks_mut(8);
        let mut get_dst_chunk = move || dst_chunks.next();

        let mut src_chunks = UTF8_STRING.as_bytes().chunks(6);
        let mut get_src_chunk = move || src_chunks.next();

        let mut dst: &mut [u16] = get_dst_chunk().unwrap();
        let mut src: &[u8] = get_src_chunk().unwrap();
        loop {
            let res = converter.feed_to_uchars(dst, src);
            match res.result {
                Ok(_) | Err(common::Error::Sys(sys::UErrorCode::U_BUFFER_OVERFLOW_ERROR)) => {
                    dst = dst.split_at_mut(res.dst_consumed).1;
                    src = src.split_at(res.src_consumed).1;
                }
                _ => panic!("conversion error"),
            }

            if dst.is_empty() {
                dst = get_dst_chunk().unwrap();
            }
            if src.is_empty() {
                src = match get_src_chunk() {
                    None => break,
                    Some(src) => src,
                };
            }
        }
    }
}
