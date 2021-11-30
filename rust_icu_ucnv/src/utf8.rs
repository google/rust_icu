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

use {
    rust_icu_common as common, rust_icu_sys as sys, rust_icu_sys::versioned_function,
    rust_icu_sys::*, std::ops::Range, std::os::raw,
};

use super::{FeedConverterRaw, FeedResult, UConverter};

/// This is a convenience type that provides conversion functions directly to/from UTF-8.
///
/// This type wraps around `ucnv_convertEx`. It keeps two converters for the specified encoding and for
/// UTF-8, as well as the UTF-16 pivot buffers used by `ucnv_convertEx`.
///
/// Its interface is analogous to the interface of [UConverter], so for examples and more detailed
/// information on its use, refer to the documentation page of [UConverter].
///
/// For convenience, the single-string conversion functions take a `&str` for UTF-8 input and
/// give a `String` for UTF-8 output.
#[derive(Debug)]
pub struct Converter {
    utf8: UConverter,
    converter: UConverter,
    pivot_buffer: Box<[sys::UChar]>,
    pivot_to: Range<*mut sys::UChar>,
    pivot_to_source: *mut sys::UChar,
    pivot_to_target: *mut sys::UChar,
    pivot_from: Range<*mut sys::UChar>,
    pivot_from_source: *mut sys::UChar,
    pivot_from_target: *mut sys::UChar,
}

unsafe impl Send for Converter {}

impl Converter {
    pub fn open(name: &str) -> Result<Self, common::Error> {
        let converter = UConverter::open(name)?;
        let utf8 = UConverter::open("UTF-8")?;
        let mut pivot_buffer = vec![0u16; 2 * 8192].into_boxed_slice();
        let (pivot_to, pivot_from) = pivot_buffer.split_at_mut(8192);
        let (pivot_to, pivot_from) = (pivot_to.as_mut_ptr_range(), pivot_from.as_mut_ptr_range());
        Ok(Self {
            utf8,
            converter,
            pivot_to_source: pivot_to.start,
            pivot_to_target: pivot_to.start,
            pivot_to,
            pivot_from_source: pivot_from.start,
            pivot_from_target: pivot_from.start,
            pivot_from,
            pivot_buffer,
        })
    }

    pub fn try_clone(&self) -> Result<Self, common::Error> {
        let utf8 = self.utf8.try_clone()?;
        let converter = self.converter.try_clone()?;
        let mut pivot_buffer = self.pivot_buffer.clone();
        let (pivot_to, pivot_from) = pivot_buffer.split_at_mut(8192);
        let (pivot_to, pivot_from) = (pivot_to.as_mut_ptr_range(), pivot_from.as_mut_ptr_range());

        // shift the pivot_{to,from}_{source,target} pointers to point to the newly-created buffer
        let pivot_to_source = unsafe {
            pivot_to
                .start
                .offset(self.pivot_to_source.offset_from(self.pivot_to.start))
        };
        let pivot_to_target = unsafe {
            pivot_to
                .start
                .offset(self.pivot_to_target.offset_from(self.pivot_to.start))
        };
        let pivot_from_source = unsafe {
            pivot_from
                .start
                .offset(self.pivot_from_source.offset_from(self.pivot_from.start))
        };
        let pivot_from_target = unsafe {
            pivot_from
                .start
                .offset(self.pivot_from_target.offset_from(self.pivot_from.start))
        };

        Ok(Self {
            utf8,
            converter,
            pivot_buffer,
            pivot_to,
            pivot_to_source,
            pivot_to_target,
            pivot_from,
            pivot_from_source,
            pivot_from_target,
        })
    }

    #[inline(always)]
    pub fn has_ambiguous_mappings(&self) -> bool {
        self.converter.has_ambiguous_mappings()
    }

    #[inline(always)]
    pub fn name(&self) -> Result<&str, common::Error> {
        self.converter.name()
    }

    pub fn reset(&mut self) {
        self.reset_to_utf8();
        self.reset_from_utf8();
    }

    pub fn reset_to_utf8(&mut self) {
        self.converter.reset_to_uchars();
        self.utf8.reset_from_uchars();
        self.pivot_to_source = self.pivot_to.start;
        self.pivot_to_target = self.pivot_to_target;
    }

    pub fn reset_from_utf8(&mut self) {
        self.utf8.reset_to_uchars();
        self.converter.reset_from_uchars();
        self.pivot_from_source = self.pivot_from.start;
        self.pivot_from_target = self.pivot_from_target;
    }

    pub fn feed_to_utf8(&mut self, dst: &mut [u8], src: &[u8]) -> FeedResult {
        self.feed_to(dst, src)
    }

    pub fn feed_from_utf8(&mut self, dst: &mut [u8], src: &[u8]) -> FeedResult {
        self.feed_from(dst, src)
    }

    pub fn convert_to_utf8(&mut self, src: &[u8]) -> Result<String, common::Error> {
        self.reset_to_utf8();

        self.convert_to(src)
            .map(|v| String::from_utf8(v).expect("should be valid UTF-8"))
    }

    pub fn convert_from_utf8(&mut self, src: &str) -> Result<Vec<u8>, common::Error> {
        self.reset_from_utf8();

        self.convert_from(src.as_bytes())
    }
}

impl FeedConverterRaw for Converter {
    // for utf8
    type ToUnit = u8;

    // for other encoding
    type FromUnit = u8;

    unsafe fn feed_to_raw(
        &mut self,
        dst: &mut Range<*mut Self::ToUnit>,
        src: &mut Range<*const Self::FromUnit>,
        should_flush: bool,
    ) -> sys::UErrorCode {
        let mut dst_raw = Range {
            start: dst.start as *mut raw::c_char,
            end: dst.end as *mut raw::c_char,
        };
        let mut src_raw = Range {
            start: src.start as *const raw::c_char,
            end: src.end as *const raw::c_char,
        };

        // ucnv_convertEx documentation indicates it appends a 0-terminator at the end of the
        // converted output if possible. This does not advance the dst pointer so we don't need to
        // do anything about it.
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        versioned_function!(ucnv_convertEx)(
            self.utf8.0.as_ptr(),
            self.converter.0.as_ptr(),
            &mut dst_raw.start,
            dst_raw.end,
            &mut src_raw.start,
            src_raw.end,
            self.pivot_to.start,
            &mut self.pivot_to_source,
            &mut self.pivot_to_target,
            self.pivot_to.end,
            false.into(),
            should_flush.into(),
            &mut status,
        );
        dst.start = dst_raw.start as *mut u8;
        src.start = src_raw.start as *const u8;

        status
    }

    unsafe fn feed_from_raw(
        &mut self,
        dst: &mut Range<*mut Self::FromUnit>,
        src: &mut Range<*const Self::ToUnit>,
        should_flush: bool,
    ) -> sys::UErrorCode {
        let mut dst_raw = Range {
            start: dst.start as *mut raw::c_char,
            end: dst.end as *mut raw::c_char,
        };
        let mut src_raw = Range {
            start: src.start as *const raw::c_char,
            end: src.end as *const raw::c_char,
        };

        // ucnv_convertEx documentation indicates it appends a 0-terminator at the end of the
        // converted output if possible. This does not advance the dst pointer so we don't need to
        // do anything about it.
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        versioned_function!(ucnv_convertEx)(
            self.converter.0.as_ptr(),
            self.utf8.0.as_ptr(),
            &mut dst_raw.start,
            dst_raw.end,
            &mut src_raw.start,
            src_raw.end,
            self.pivot_from.start,
            &mut self.pivot_from_source,
            &mut self.pivot_from_target,
            self.pivot_from.end,
            false.into(),
            should_flush.into(),
            &mut status,
        );
        dst.start = dst_raw.start as *mut u8;
        src.start = src_raw.start as *const u8;

        status
    }
}

#[cfg(test)]
mod tests {
    use rust_icu_common as common;
    use rust_icu_sys as sys;

    use super::Converter;

    #[test]
    fn test_shiftjis_utf8_conversion() {
        const SHIFT_JIS_STRING: [u8; 8] = [0x83, 0x58, 0x81, 0x5B, 0x83, 0x70, 0x81, 0x5B];
        const UTF8_STRING: &str = "スーパー";

        let mut converter = Converter::open("SHIFT-JIS").unwrap();

        assert_eq!(
            UTF8_STRING,
            converter
                .convert_to_utf8(&SHIFT_JIS_STRING)
                .unwrap()
                .as_str()
        );

        assert_eq!(
            SHIFT_JIS_STRING.iter().copied().collect::<Vec<u8>>(),
            converter.convert_from_utf8(UTF8_STRING).unwrap()
        );
    }

    #[test]
    fn test_shiftjis_utf8_feeding() {
        const UTF8_STRING: &str =
            "Shift_JIS（シフトジス）は、コンピュータ上で日本語を含む文字列を表現するために\
        用いられる文字コードの一つ。シフトJIS（シフトジス）と表記されることもある。";

        let mut converter = Converter::open("SHIFT_JIS").unwrap();

        let mut dst_buffer: Vec<u8> = Vec::new();
        dst_buffer.resize(1024, 0);

        let mut dst_chunks = dst_buffer.chunks_mut(8);
        let mut get_dst_chunk = move || dst_chunks.next();

        let mut src_chunks = UTF8_STRING.as_bytes().chunks(6);
        let mut get_src_chunk = move || src_chunks.next();

        let mut dst: &mut [u8] = get_dst_chunk().unwrap();
        let mut src: &[u8] = get_src_chunk().unwrap();
        loop {
            let res = converter.feed_from_utf8(dst, src);
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
