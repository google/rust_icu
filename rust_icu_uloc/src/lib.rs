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

use {
    rust_icu_common as common,
    rust_icu_sys::versioned_function,
    rust_icu_sys::*,
    rust_icu_uenum::Enumeration,
    std::{
        cmp::Ordering,
        convert::{From, TryFrom, TryInto},
        ffi,
        os::raw,
    },
};

/// Maximum length of locale supported by uloc.h.
/// See `ULOC_FULLNAME_CAPACITY`.
const LOCALE_CAPACITY: usize = 158;

/// A representation of a Unicode locale.
///
/// For the time being, only basic conversion and methods are in fact implemented.
///
/// To get basic validation when creating a locale, use
/// [`for_language_tag`](ULoc::for_language_tag) with a Unicode BCP-47 locale ID.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ULoc {
    // A locale's representation in C is really just a string.
    repr: String,
}

impl TryFrom<&str> for ULoc {
    type Error = common::Error;
    /// Creates a new ULoc from a string slice.
    ///
    /// The creation wil fail if the locale is nonexistent.
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = String::from(s);
        ULoc { repr: s }.canonicalize()
    }
}

impl TryFrom<&ffi::CStr> for ULoc {
    type Error = common::Error;

    /// Creates a new `ULoc` from a borrowed C string.
    fn try_from(s: &ffi::CStr) -> Result<Self, Self::Error> {
        let repr = s.to_str()?;
        ULoc {
            repr: String::from(repr),
        }
        .canonicalize()
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
/// ```
/// buffered_string_method_with_retry!(
///     my_method,
///     BUFFER_CAPACITY,
///     [before_arg_a: before_type_a, before_arg_b: before_type_b,],
///     [after_arg_a: after_type_a, after_arg_b: after_type_b,]
/// );   
/// ```
/// the generated method has a signature of the form
/// ```
/// fn my_method(
///     uloc_method: unsafe extern "C" fn(
///         before_type_a,
///         before_type_b,
///         *mut raw::c_char,
///         i32,
///         after_type_a,
///         after_type_b,
///         *mut UErrorCode,
///     ) -> i32,
///     before_arg_a: before_type_a,
///     before_arg_b: before_type_b,
///     after_arg_a: after_type_a,
///     after_arg_b: after_type_b
/// ) -> Result<String, common::Error> {}
/// ```
macro_rules! buffered_string_method_with_retry {

    ($method_name:ident, $buffer_capacity:expr,
     [$($before_arg:ident: $before_arg_type:ty,)*],
     [$($after_arg:ident: $after_arg_type:ty,)*]) => {
        fn $method_name(
            uloc_method: unsafe extern "C" fn(
                $($before_arg_type,)*
                *mut raw::c_char,
                i32,
                $($after_arg_type,)*
                *mut UErrorCode,
            ) -> i32,
            $($before_arg: $before_arg_type,)*
            $($after_arg: $after_arg_type,)*
        ) -> Result<String, common::Error> {
            let mut status = common::Error::OK_CODE;
            let mut buf: Vec<u8> = vec![0; $buffer_capacity];

            // Requires that any pointers that are passed in are valid.
            let full_len: i32 = unsafe {
                assert!(common::Error::is_ok(status));
                uloc_method(
                    $($before_arg,)*
                    buf.as_mut_ptr() as *mut raw::c_char,
                    $buffer_capacity as i32,
                    $($after_arg,)*
                    &mut status,
                )
            };

            // `uloc` methods are inconsistent in whether they silently truncate the output or treat
            // the overflow as an error, so we need to check both cases.
            if status == UErrorCode::U_BUFFER_OVERFLOW_ERROR ||
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
                    uloc_method(
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

impl ULoc {
    /// Implements `uloc_getLanguage`.
    pub fn language(&self) -> Option<String> {
        self.call_buffered_string_method_to_option(versioned_function!(uloc_getLanguage))
    }

    /// Implements `uloc_getScript`.
    pub fn script(&self) -> Option<String> {
        self.call_buffered_string_method_to_option(versioned_function!(uloc_getScript))
    }

    /// Implements `uloc_getCountry`.
    pub fn country(&self) -> Option<String> {
        self.call_buffered_string_method_to_option(versioned_function!(uloc_getCountry))
    }

    /// Implements `uloc_getVariant`.
    pub fn variant(&self) -> Option<String> {
        self.call_buffered_string_method_to_option(versioned_function!(uloc_getVariant))
    }

    /// Implements `uloc_canonicalize` from ICU4C.
    pub fn canonicalize(&self) -> Result<ULoc, common::Error> {
        self.call_buffered_string_method(versioned_function!(uloc_canonicalize))
            .map(|repr| ULoc { repr })
    }

    /// Implements `uloc_addLikelySubtags` from ICU4C.
    pub fn add_likely_subtags(&self) -> Result<ULoc, common::Error> {
        self.call_buffered_string_method(versioned_function!(uloc_addLikelySubtags))
            .map(|repr| ULoc { repr })
    }

    /// Implements `uloc_minimizeSubtags` from ICU4C.
    pub fn minimize_subtags(&self) -> Result<ULoc, common::Error> {
        self.call_buffered_string_method(versioned_function!(uloc_minimizeSubtags))
            .map(|repr| ULoc { repr })
    }

    /// Implements `uloc_toLanguageTag` from ICU4C.
    pub fn to_language_tag(&self, strict: bool) -> Result<String, common::Error> {
        buffered_string_method_with_retry!(
            buffered_string_to_language_tag,
            LOCALE_CAPACITY,
            [locale_id: *const raw::c_char,],
            [strict: rust_icu_sys::UBool,]
        );

        let locale_id = self.as_c_str();
        // No `UBool` constants available in rust_icu_sys, unfortunately.
        let strict = if strict { 1 } else { 0 };
        buffered_string_to_language_tag(
            versioned_function!(uloc_toLanguageTag),
            locale_id.as_ptr(),
            strict,
        )
    }

    /// Implements `uloc_openKeywords()` from ICU4C.
    pub fn keywords(&self) -> impl Iterator<Item = String> {
        rust_icu_uenum::uloc_open_keywords(&self.repr)
            .unwrap()
            .map(|result| result.unwrap())
    }

    /// Implements `icu::Locale::getUnicodeKeywords()` from the C++ API.
    pub fn unicode_keywords(&self) -> impl Iterator<Item = String> {
        self.keywords().filter_map(|s| to_unicode_locale_key(&s))
    }

    /// Implements `uloc_getKeywordValue()` from ICU4C.
    pub fn keyword_value(&self, keyword: &str) -> Result<Option<String>, common::Error> {
        buffered_string_method_with_retry!(
            buffered_string_keyword_value,
            LOCALE_CAPACITY,
            [
                locale_id: *const raw::c_char,
                keyword_name: *const raw::c_char,
            ],
            []
        );
        let locale_id = self.as_c_str();
        let keyword_name = str_to_cstring(keyword);
        buffered_string_keyword_value(
            versioned_function!(uloc_getKeywordValue),
            locale_id.as_ptr(),
            keyword_name.as_ptr(),
        )
        .map(|value| if value.is_empty() { None } else { Some(value) })
    }

    /// Implements `icu::Locale::getUnicodeKeywordValue()` from ICU4C.
    pub fn unicode_keyword_value(
        &self,
        unicode_keyword: &str,
    ) -> Result<Option<String>, common::Error> {
        let legacy_keyword = to_legacy_key(unicode_keyword);
        match legacy_keyword {
            Some(legacy_keyword) => match self.keyword_value(&legacy_keyword) {
                Ok(Some(legacy_value)) => {
                    Ok(to_unicode_locale_type(&legacy_keyword, &legacy_value))
                }
                Ok(None) => Ok(None),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }

    /// Returns the current label of this locale.
    pub fn label(&self) -> &str {
        &self.repr
    }

    /// Returns the current locale name as a C string.
    pub fn as_c_str(&self) -> ffi::CString {
        ffi::CString::new(self.repr.clone()).expect("ULoc contained interior NUL bytes")
    }

    /// Implements `uloc_forLanguageTag` from ICU4C.
    pub fn for_language_tag(tag: &str) -> Result<ULoc, common::Error> {
        buffered_string_method_with_retry!(
            buffered_string_for_language_tag,
            LOCALE_CAPACITY,
            [tag: *const raw::c_char,],
            [parsed_length: *mut i32,]
        );

        let tag = str_to_cstring(tag);
        let locale_id = buffered_string_for_language_tag(
            versioned_function!(uloc_forLanguageTag),
            tag.as_ptr(),
            std::ptr::null_mut(),
        )?;
        ULoc::try_from(&locale_id[..])
    }

    /// Call a `uloc` method that takes this locale's ID and returns a string.
    fn call_buffered_string_method(
        &self,
        uloc_method: unsafe extern "C" fn(
            *const raw::c_char,
            *mut raw::c_char,
            i32,
            *mut UErrorCode,
        ) -> i32,
    ) -> Result<String, common::Error> {
        buffered_string_method_with_retry!(
            buffered_string_char_star,
            LOCALE_CAPACITY,
            [char_star: *const raw::c_char,],
            []
        );
        let asciiz = self.as_c_str();
        buffered_string_char_star(uloc_method, asciiz.as_ptr())
    }

    /// Call a `uloc` method that takes this locale's ID, panics on any errors, and returns
    /// `Some(result)` if the resulting string is non-empty, or `None` otherwise.
    fn call_buffered_string_method_to_option(
        &self,
        uloc_method: unsafe extern "C" fn(
            *const raw::c_char,
            *mut raw::c_char,
            i32,
            *mut UErrorCode,
        ) -> i32,
    ) -> Option<String> {
        let value: String = self.call_buffered_string_method(uloc_method).unwrap();
        if value.is_empty() {
            None
        } else {
            Some(value)
        }
    }
}

/// This implementation is based on ULocale.compareTo from ICU4J.
/// See https://github.com/unicode-org/icu/blob/master/icu4j/main/classes/core/src/com/ibm/icu/util/ULocale.java
impl Ord for ULoc {
    fn cmp(&self, other: &Self) -> Ordering {
        /// Compare corresponding keywords from two `ULoc`s. If the keywords match, compare the
        /// keyword values.
        fn compare_keywords(
            this: &ULoc,
            self_keyword: &Option<String>,
            other: &ULoc,
            other_keyword: &Option<String>,
        ) -> Option<Ordering> {
            match (self_keyword, other_keyword) {
                (Some(self_keyword), Some(other_keyword)) => {
                    // Compare the two keywords
                    match self_keyword.cmp(&other_keyword) {
                        Ordering::Equal => {
                            // Compare the two keyword values
                            let self_val = this.keyword_value(&self_keyword[..]).unwrap();
                            let other_val = other.keyword_value(&other_keyword[..]).unwrap();
                            Some(self_val.cmp(&other_val))
                        }
                        unequal_ordering => Some(unequal_ordering),
                    }
                }
                // `other` has run out of keywords
                (Some(_), _) => Some(Ordering::Greater),
                // `this` has run out of keywords
                (_, Some(_)) => Some(Ordering::Less),
                // Both iterators have run out
                (_, _) => None,
            }
        }

        self.language()
            .cmp(&other.language())
            .then_with(|| self.script().cmp(&other.script()))
            .then_with(|| self.country().cmp(&other.country()))
            .then_with(|| self.variant().cmp(&other.variant()))
            .then_with(|| {
                let mut self_keywords = self.keywords();
                let mut other_keywords = other.keywords();

                while let Some(keyword_ordering) =
                    compare_keywords(self, &self_keywords.next(), other, &other_keywords.next())
                {
                    match keyword_ordering {
                        Ordering::Equal => {}
                        unequal_ordering => {
                            return unequal_ordering;
                        }
                    }
                }

                // All keywords and values were identical (or there were none)
                Ordering::Equal
            })
    }
}

impl PartialOrd for ULoc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Gets the current system default locale.
///
/// Implements `uloc_getDefault` from ICU4C.
pub fn get_default() -> ULoc {
    let loc = unsafe { versioned_function!(uloc_getDefault)() };
    let uloc_cstr = unsafe { ffi::CStr::from_ptr(loc) };
    crate::ULoc::try_from(uloc_cstr).expect("could not convert default locale to ULoc")
}

/// Sets the current default system locale.
///
/// Implements `uloc_setDefault` from ICU4C.
pub fn set_default(loc: &ULoc) -> Result<(), common::Error> {
    let mut status = common::Error::OK_CODE;
    let asciiz = str_to_cstring(&loc.repr);
    unsafe { versioned_function!(uloc_setDefault)(asciiz.as_ptr(), &mut status) };
    common::Error::ok_or_warning(status)
}

/// Implements `uloc_acceptLanguage` from ICU4C.
pub fn accept_language(
    accept_list: impl IntoIterator<Item = impl Into<ULoc>>,
    available_locales: impl IntoIterator<Item = impl Into<ULoc>>,
) -> Result<(Option<ULoc>, UAcceptResult), common::Error> {
    buffered_string_method_with_retry!(
        buffered_string_uloc_accept_language,
        LOCALE_CAPACITY,
        [],
        [
            out_result: *mut UAcceptResult,
            accept_list: *mut *const ::std::os::raw::c_char,
            accept_list_count: i32,
            available_locales: *mut UEnumeration,
        ]
    );

    let mut accept_result: UAcceptResult = UAcceptResult::ULOC_ACCEPT_FAILED;
    let mut accept_list_cstrings: Vec<ffi::CString> = vec![];
    // This is mutable only to satisfy the missing `const`s in the ICU4C API.
    let mut accept_list: Vec<*const raw::c_char> = accept_list
        .into_iter()
        .map(|item| {
            let uloc: ULoc = item.into();
            accept_list_cstrings.push(uloc.as_c_str());
            accept_list_cstrings
                .last()
                .expect("non-empty list")
                .as_ptr()
        })
        .collect();

    let available_locales: Vec<ULoc> = available_locales
        .into_iter()
        .map(|item| item.into())
        .collect();
    let available_locales: Vec<&str> = available_locales.iter().map(|uloc| uloc.label()).collect();
    let mut available_locales = Enumeration::try_from(&available_locales[..])?;

    let matched_locale = buffered_string_uloc_accept_language(
        versioned_function!(uloc_acceptLanguage),
        &mut accept_result,
        accept_list.as_mut_ptr(),
        accept_list.len() as i32,
        available_locales.repr(),
    );

    // Having no match is a valid if disappointing result.
    if accept_result == UAcceptResult::ULOC_ACCEPT_FAILED {
        return Ok((None, accept_result));
    }

    matched_locale
        .and_then(|s| ULoc::try_from(s.as_str()))
        .map(|uloc| (Some(uloc), accept_result))
}

/// Implements `uloc_toUnicodeLocaleKey` from ICU4C.
pub fn to_unicode_locale_key(legacy_keyword: &str) -> Option<String> {
    let legacy_keyword = str_to_cstring(legacy_keyword);
    let unicode_keyword: Option<ffi::CString> = unsafe {
        let ptr = versioned_function!(uloc_toUnicodeLocaleKey)(legacy_keyword.as_ptr());
        ptr.as_ref().map(|ptr| ffi::CStr::from_ptr(ptr).to_owned())
    };
    unicode_keyword.map(|cstring| cstring_to_string(&cstring))
}

/// Implements `uloc_toUnicodeLocaleType` from ICU4C.
pub fn to_unicode_locale_type(legacy_keyword: &str, legacy_value: &str) -> Option<String> {
    let legacy_keyword = str_to_cstring(legacy_keyword);
    let legacy_value = str_to_cstring(legacy_value);
    let unicode_value: Option<ffi::CString> = unsafe {
        let ptr = versioned_function!(uloc_toUnicodeLocaleType)(
            legacy_keyword.as_ptr(),
            legacy_value.as_ptr(),
        );
        ptr.as_ref().map(|ptr| ffi::CStr::from_ptr(ptr).to_owned())
    };
    unicode_value.map(|cstring| cstring_to_string(&cstring))
}

/// Implements `uloc_toLegacyKey` from ICU4C.
pub fn to_legacy_key(unicode_keyword: &str) -> Option<String> {
    let unicode_keyword = str_to_cstring(unicode_keyword);
    let legacy_keyword: Option<ffi::CString> = unsafe {
        let ptr = versioned_function!(uloc_toLegacyKey)(unicode_keyword.as_ptr());
        ptr.as_ref().map(|ptr| ffi::CStr::from_ptr(ptr).to_owned())
    };
    legacy_keyword.map(|cstring| cstring_to_string(&cstring))
}

/// Infallibly converts a Rust string to a `CString`. If there's an interior NUL, the string is
/// truncated up to that point.
fn str_to_cstring(input: &str) -> ffi::CString {
    ffi::CString::new(input)
        .unwrap_or_else(|e| ffi::CString::new(&input[0..e.nul_position()]).unwrap())
}

/// Infallibly converts a `CString` to a Rust `String`. We can safely assume that any strings
/// coming from ICU data are valid UTF-8.
fn cstring_to_string(input: &ffi::CString) -> String {
    input.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use {super::*, anyhow::Error};

    #[test]
    fn test_language() -> Result<(), Error> {
        let loc = ULoc::try_from("es-CO")?;
        assert_eq!(loc.language(), Some("es".to_string()));
        Ok(())
    }

    #[test]
    fn test_language_absent() -> Result<(), Error> {
        let loc = ULoc::for_language_tag("und-CO")?;
        assert_eq!(loc.language(), None);
        Ok(())
    }

    #[test]
    fn test_script() -> Result<(), Error> {
        let loc = ULoc::try_from("sr-Cyrl")?;
        assert_eq!(loc.script(), Some("Cyrl".to_string()));
        Ok(())
    }

    #[test]
    fn test_script_absent() -> Result<(), Error> {
        let loc = ULoc::try_from("sr")?;
        assert_eq!(loc.script(), None);
        Ok(())
    }

    #[test]
    fn test_country() -> Result<(), Error> {
        let loc = ULoc::try_from("es-CO")?;
        assert_eq!(loc.country(), Some("CO".to_string()));
        Ok(())
    }

    #[test]
    fn test_country_absent() -> Result<(), Error> {
        let loc = ULoc::try_from("es")?;
        assert_eq!(loc.country(), None);
        Ok(())
    }

    // This test yields a different result in ICU versions prior to 64:
    // "zh-Latn@collation=pinyin".
    #[cfg(features = "icu_version_64_plus")]
    #[test]
    fn test_variant() -> Result<(), Error> {
        let loc = ULoc::try_from("zh-Latn-pinyin")?;
        assert_eq!(
            loc.variant(),
            Some("PINYIN".to_string()),
            "locale was: {:?}",
            loc
        );
        Ok(())
    }

    #[test]
    fn test_variant_absent() -> Result<(), Error> {
        let loc = ULoc::try_from("zh-Latn")?;
        assert_eq!(loc.variant(), None);
        Ok(())
    }

    #[test]
    fn test_default_locale() {
        let loc = ULoc::try_from("fr-fr").expect("get fr_FR locale");
        set_default(&loc).expect("successful set of locale");
        assert_eq!(get_default().label(), loc.label());
        assert_eq!(loc.label(), "fr_FR", "The locale should get canonicalized");
        let loc = ULoc::try_from("en-us").expect("get en_US locale");
        set_default(&loc).expect("successful set of locale");
        assert_eq!(get_default().label(), loc.label());
    }

    #[test]
    fn test_add_likely_subtags() {
        let loc = ULoc::try_from("en-US").expect("get en_US locale");
        let with_likely_subtags = loc.add_likely_subtags().expect("should add likely subtags");
        let expected = ULoc::try_from("en_Latn_US").expect("get en_Latn_US locale");
        assert_eq!(with_likely_subtags.label(), expected.label());
    }

    #[test]
    fn test_minimize_subtags() {
        let loc = ULoc::try_from("sr_Cyrl_RS").expect("get sr_Cyrl_RS locale");
        let minimized_subtags = loc.minimize_subtags().expect("should minimize subtags");
        let expected = ULoc::try_from("sr").expect("get sr locale");
        assert_eq!(minimized_subtags.label(), expected.label());
    }

    #[test]
    fn test_to_language_tag() {
        let loc = ULoc::try_from("sr_Cyrl_RS").expect("get sr_Cyrl_RS locale");
        let language_tag = loc
            .to_language_tag(true)
            .expect("should convert to language tag");
        assert_eq!(language_tag, "sr-Cyrl-RS".to_string());
    }

    #[test]
    fn test_keywords() -> Result<(), Error> {
        let loc = ULoc::for_language_tag("az-Cyrl-AZ-u-ca-hebrew-fw-sunday-nu-deva-tz-usnyc")?;
        let keywords: Vec<String> = loc.keywords().collect();
        assert_eq!(
            keywords,
            vec![
                "calendar".to_string(),
                "fw".to_string(),
                "numbers".to_string(),
                "timezone".to_string()
            ]
        );
        Ok(())
    }

    #[test]
    fn test_keywords_empty() -> Result<(), Error> {
        let loc = ULoc::for_language_tag("az-Cyrl-AZ")?;
        let keywords: Vec<String> = loc.keywords().collect();
        assert!(keywords.is_empty());
        Ok(())
    }

    #[test]
    fn test_unicode_keywords() -> Result<(), Error> {
        let loc = ULoc::for_language_tag("az-Cyrl-AZ-u-ca-hebrew-fw-sunday-nu-deva-tz-usnyc")?;
        let keywords: Vec<String> = loc.unicode_keywords().collect();
        assert_eq!(
            keywords,
            vec![
                "ca".to_string(),
                "fw".to_string(),
                "nu".to_string(),
                "tz".to_string()
            ]
        );
        Ok(())
    }

    #[test]
    fn test_unicode_keywords_empty() -> Result<(), Error> {
        let loc = ULoc::for_language_tag("az-Cyrl-AZ")?;
        let keywords: Vec<String> = loc.unicode_keywords().collect();
        assert!(keywords.is_empty());
        Ok(())
    }

    #[test]
    fn test_keyword_value() -> Result<(), Error> {
        let loc = ULoc::for_language_tag("az-Cyrl-AZ-u-ca-hebrew-fw-sunday-nu-deva-tz-usnyc")?;
        assert_eq!(loc.keyword_value("calendar")?, Some("hebrew".to_string()));
        assert_eq!(loc.keyword_value("collation")?, None);
        Ok(())
    }

    #[test]
    fn test_unicode_keyword_value() -> Result<(), Error> {
        let loc = ULoc::for_language_tag("az-Cyrl-AZ-u-ca-hebrew-fw-sunday-nu-deva-tz-usnyc")?;
        assert_eq!(loc.unicode_keyword_value("ca")?, Some("hebrew".to_string()));
        assert_eq!(loc.unicode_keyword_value("fw")?, Some("sunday".to_string()));
        assert_eq!(loc.unicode_keyword_value("co")?, None);
        Ok(())
    }

    #[test]
    fn test_order() -> Result<(), Error> {
        assert!(ULoc::for_language_tag("az")? < ULoc::for_language_tag("az-Cyrl")?);
        assert!(ULoc::for_language_tag("az-Cyrl")? < ULoc::for_language_tag("az-Cyrl-AZ")?);
        assert!(
            ULoc::for_language_tag("az-Cyrl-AZ")? < ULoc::for_language_tag("az-Cyrl-AZ-variant")?
        );
        assert!(
            ULoc::for_language_tag("az-Cyrl-AZ-variant")?
                < ULoc::for_language_tag("az-Cyrl-AZ-variant-u-nu-arab")?
        );
        assert!(
            ULoc::for_language_tag("az-u-ca-gregory")? < ULoc::for_language_tag("az-u-fw-fri")?
        );
        assert!(
            ULoc::for_language_tag("az-u-ca-buddhist")?
                < ULoc::for_language_tag("az-u-ca-chinese")?
        );
        assert!(ULoc::for_language_tag("az-u-fw-mon")? < ULoc::for_language_tag("az-u-fw-tue")?);
        assert!(
            ULoc::for_language_tag("az-u-fw-mon")? < ULoc::for_language_tag("az-u-fw-mon-nu-arab")?
        );
        assert!(
            ULoc::for_language_tag("az-u-fw-mon-nu-arab")? > ULoc::for_language_tag("az-u-fw-mon")?
        );

        let loc = ULoc::for_language_tag("az-Cyrl-AZ-variant-u-nu-arab")?;
        assert_eq!(loc.cmp(&loc), Ordering::Equal,);
        Ok(())
    }

    #[test]
    fn test_accept_language_fallback() {
        let accept_list: Result<Vec<_>, _> = vec!["es_MX", "ar_EG", "fr_FR"]
            .into_iter()
            .map(ULoc::try_from)
            .collect();
        let accept_list = accept_list.expect("make accept_list");

        let available_locales: Result<Vec<_>, _> =
            vec!["de_DE", "en_US", "es", "nl_NL", "sr_RS_Cyrl"]
                .into_iter()
                .map(ULoc::try_from)
                .collect();
        let available_locales = available_locales.expect("make available_locales");

        let actual = accept_language(accept_list, available_locales).expect("call accept_language");
        assert_eq!(
            actual,
            (
                ULoc::try_from("es").ok(),
                UAcceptResult::ULOC_ACCEPT_FALLBACK
            )
        );
    }

    // This tests verifies buggy behavior which is fixed since ICU version 67.1
    #[cfg(not(features = "icu_version_67_plus"))]
    #[test]
    fn test_accept_language_exact_match() {
        let accept_list: Result<Vec<_>, _> = vec!["es_ES", "ar_EG", "fr_FR"]
            .into_iter()
            .map(ULoc::try_from)
            .collect();
        let accept_list = accept_list.expect("make accept_list");

        let available_locales: Result<Vec<_>, _> = vec!["de_DE", "en_US", "es_MX", "ar_EG"]
            .into_iter()
            .map(ULoc::try_from)
            .collect();
        let available_locales = available_locales.expect("make available_locales");

        let actual = accept_language(accept_list, available_locales).expect("call accept_language");
        assert_eq!(
            actual,
            (
                // "es_MX" should be preferred as a fallback over exact match "ar_EG".
                ULoc::try_from("ar_EG").ok(),
                UAcceptResult::ULOC_ACCEPT_VALID
            )
        );
    }

    #[cfg(features = "icu_version_67_plus")]
    #[test]
    fn test_accept_language_exact_match() {
        let accept_list: Result<Vec<_>, _> = vec!["es_ES", "ar_EG", "fr_FR"]
            .into_iter()
            .map(ULoc::try_from)
            .collect();
        let accept_list = accept_list.expect("make accept_list");

        let available_locales: Result<Vec<_>, _> = vec!["de_DE", "en_US", "es_MX", "ar_EG"]
            .into_iter()
            .map(ULoc::try_from)
            .collect();
        let available_locales = available_locales.expect("make available_locales");

        let actual = accept_language(accept_list, available_locales).expect("call accept_language");
        assert_eq!(
            actual,
            (
                ULoc::try_from("es_MX").ok(),
                UAcceptResult::ULOC_ACCEPT_FALLBACK,
            )
        );
    }

    #[test]
    fn test_accept_language_no_match() {
        let accept_list: Result<Vec<_>, _> = vec!["es_ES", "ar_EG", "fr_FR"]
            .into_iter()
            .map(ULoc::try_from)
            .collect();
        let accept_list = accept_list.expect("make accept_list");

        let available_locales: Result<Vec<_>, _> =
            vec!["el_GR"].into_iter().map(ULoc::try_from).collect();
        let available_locales = available_locales.expect("make available_locales");

        let actual = accept_language(accept_list, available_locales).expect("call accept_language");
        assert_eq!(actual, (None, UAcceptResult::ULOC_ACCEPT_FAILED))
    }

    #[test]
    fn test_to_unicode_locale_key() -> Result<(), Error> {
        let actual = to_unicode_locale_key("calendar");
        assert_eq!(actual, Some("ca".to_string()));
        Ok(())
    }

    #[test]
    fn test_to_unicode_locale_type() -> Result<(), Error> {
        let actual = to_unicode_locale_type("co", "phonebook");
        assert_eq!(actual, Some("phonebk".to_string()));
        Ok(())
    }

    #[test]
    fn test_to_legacy_key() -> Result<(), Error> {
        let actual = to_legacy_key("ca");
        assert_eq!(actual, Some("calendar".to_string()));
        Ok(())
    }

    #[test]
    fn test_str_to_cstring() -> Result<(), Error> {
        assert_eq!(str_to_cstring("abc"), ffi::CString::new("abc")?);
        assert_eq!(str_to_cstring("abc\0def"), ffi::CString::new("abc")?);

        Ok(())
    }
}
