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

use std::ffi::{CStr, CString};

use {rust_icu_sys as sys, rust_icu_sys::versioned_function, rust_icu_sys::*};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
