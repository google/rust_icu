// Copyright 2024 Google LLC
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

//! # Rust implementation of the `ures.h` C API header for ICU.
//!
//! Provides safe, idiomatic Rust access to ICU resource bundles.  A resource
//! bundle associates a locale with a hierarchy of typed values — strings,
//! integers, binary data, and nested tables — stored in compiled `.res` files
//! produced by the `genrb` tool.
//!
//! ## Quick start
//!
//! ```no_run
//! use rust_icu_ures::{UResourceBundle, ResourceType};
//!
//! // Open ICU's built-in root locale (always available, no custom .res needed).
//! let bundle = UResourceBundle::try_new(None, "root").unwrap();
//! assert_eq!(bundle.resource_type(), ResourceType::Table);
//!
//! // Open a custom bundle compiled from source with genrb.
//! let bundle = UResourceBundle::try_new(Some("/usr/share/myapp/i18n/com/example/MyBundle"), "fr")
//!     .unwrap();
//! let greeting = bundle.get_string_by_key("greeting").unwrap();
//! println!("{}", greeting); // e.g. "Bonjour"
//! ```
//!
//! ## Bundle file format
//!
//! ```text
//! // root.txt
//! root {
//!     greeting { "Hello" }
//!     count:int { 42 }
//!     errors {
//!         not_found { "Not found" }
//!     }
//! }
//! ```
//!
//! Compile with: `genrb -d /path/to/output  root.txt  fr.txt`
//!
//! Then open with: `UResourceBundle::try_new(Some("/path/to/output"), "fr")`
//!
//! ## Thread safety
//!
//! `UResourceBundle` is [`Send`] but not `Sync`.  The ICU bundle stores
//! iterator state directly inside the struct, making concurrent access unsafe.
//! Use a `Mutex<UResourceBundle>` if shared access is needed.

use {
    anyhow::anyhow,
    rust_icu_common as common,
    rust_icu_sys as sys,
    rust_icu_sys::versioned_function,
    rust_icu_uenum::Enumeration,
    std::{ffi, os::raw},
};

/// The type of a value held in a [`UResourceBundle`].
///
/// Mirrors the `UResType` C enum from `ures.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// No resource (ICU sentinel value `URES_NONE`).
    None,
    /// A 16-bit Unicode string.
    String,
    /// Raw binary data.
    Binary,
    /// An ordered collection of key–value pairs.
    Table,
    /// An alias to another resource (resolved transparently by ICU).
    Alias,
    /// A single 28-bit signed integer stored in 32 bits.
    Int,
    /// An ordered sequence of resources without keys.
    Array,
    /// A packed array of 32-bit integers.
    IntVector,
    /// A variant value not recognised by this version of the bindings
    /// (e.g. `RES_RESERVED`, `URES_LIMIT`).
    Unknown(i32),
}

impl From<sys::UResType> for ResourceType {
    fn from(t: sys::UResType) -> Self {
        match t {
            sys::UResType::URES_NONE        => ResourceType::None,
            sys::UResType::URES_STRING      => ResourceType::String,
            sys::UResType::URES_BINARY      => ResourceType::Binary,
            sys::UResType::URES_TABLE       => ResourceType::Table,
            sys::UResType::URES_ALIAS       => ResourceType::Alias,
            sys::UResType::URES_INT         => ResourceType::Int,
            sys::UResType::URES_ARRAY       => ResourceType::Array,
            sys::UResType::URES_INT_VECTOR  => ResourceType::IntVector,
            // Deprecated sentinel values that are not meaningful resource types.
            sys::UResType::RES_RESERVED | sys::UResType::URES_LIMIT => {
                ResourceType::Unknown(t as i32)
            }
        }
    }
}

/// Safe wrapper around an ICU `UResourceBundle`.
///
/// A resource bundle provides locale-specific typed data loaded from a compiled
/// `.res` file.  Values are accessed by key (for TABLE bundles) or by index
/// (for ARRAY bundles), and may be strings, integers, binary blobs, nested
/// tables, or integer vectors.
///
/// ## Ownership
///
/// `UResourceBundle` is the sole owner of the underlying ICU object and closes
/// it via `ures_close` when dropped.  Sub-bundles returned by [`get_by_key`],
/// [`get_by_index`], and [`next_resource`] are independently owned: each has
/// its own pointer and is closed when it goes out of scope, regardless of the
/// parent bundle's lifetime.
///
/// [`get_by_key`]: UResourceBundle::get_by_key
/// [`get_by_index`]: UResourceBundle::get_by_index
/// [`next_resource`]: UResourceBundle::next_resource
///
/// ## Thread safety
///
/// `UResourceBundle` is [`Send`]: it may be moved to another thread.  It is
/// intentionally **not** `Sync`: the ICU bundle stores mutable iterator state
/// directly inside the struct, so concurrent access would data-race on that
/// state.  Wrap in `Mutex<UResourceBundle>` if shared access across threads is
/// needed.
///
/// ## Move semantics
///
/// `UResourceBundle` is move-only (not `Clone`).  Call [`try_new`] or
/// [`try_new_direct`] to create independent bundles.
///
/// [`try_new`]: UResourceBundle::try_new
/// [`try_new_direct`]: UResourceBundle::try_new_direct
#[derive(Debug)]
pub struct UResourceBundle {
    // Invariant: rep is always a valid non-null *mut UResourceBundle obtained
    // from ures_open / ures_openDirect / ures_getByKey / ures_getByIndex /
    // ures_getNextResource, and not yet passed to ures_close.
    rep: *mut sys::UResourceBundle,
}

// Safety: UResourceBundle does not use thread-local storage and has no
// affinity to the thread that created it; it is safe to move to another thread.
unsafe impl Send for UResourceBundle {}

// Intentionally NOT implementing Sync: the ICU UResourceBundle struct stores
// mutable iterator position (modified by ures_resetIterator / ures_getNextResource
// / ures_getNextString), so concurrent method calls without external
// synchronisation would data-race on that internal mutable state.

impl Drop for UResourceBundle {
    /// Closes the underlying ICU resource bundle.
    ///
    /// Implements `ures_close`.
    fn drop(&mut self) {
        // Safety: self.rep is a valid non-null pointer that has not been
        // passed to ures_close yet.  This is the only call to ures_close for
        // this pointer (guaranteed by Rust's ownership model).
        unsafe { versioned_function!(ures_close)(self.rep) };
    }
}

impl UResourceBundle {
    /// Opens the resource bundle for `locale` from the package at `package`.
    ///
    /// `package` controls where ICU looks for the compiled `.res` file:
    ///
    /// - `None` — use ICU's built-in data (always available).
    /// - `Some(path)` — the absolute path to the directory containing the
    ///   compiled `.res` files.  For example, if the bundle was compiled as
    ///   `/usr/share/myapp/i18n/com/example/ServiceBundle/fr.res`, pass
    ///   `Some("/usr/share/myapp/i18n/com/example/ServiceBundle")`.
    ///
    /// Locale fallback is applied automatically: if the exact locale is not
    /// available, ICU falls back through the locale hierarchy to `root`.
    ///
    /// Implements `ures_open`.
    pub fn try_new(package: Option<&str>, locale: &str) -> Result<Self, common::Error> {
        let mut status = common::Error::OK_CODE;
        let c_package = match package {
            Some(p) => Some(ffi::CString::new(p)?),
            None    => None,
        };
        let c_locale = ffi::CString::new(locale)?;
        // Safety: both pointers are valid for the duration of the call.
        // The returned non-null pointer becomes the sole owner of the ICU object.
        let rep = unsafe {
            versioned_function!(ures_open)(
                c_package.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
                c_locale.as_ptr(),
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        assert!(!rep.is_null());
        Ok(UResourceBundle { rep })
    }

    /// Opens the resource bundle for `locale` **without** locale fallback.
    ///
    /// Unlike [`try_new`], returns an error if the exact locale is not available,
    /// rather than falling back to a parent locale or `root`.
    ///
    /// Implements `ures_openDirect`.
    ///
    /// [`try_new`]: UResourceBundle::try_new
    pub fn try_new_direct(package: Option<&str>, locale: &str) -> Result<Self, common::Error> {
        let mut status = common::Error::OK_CODE;
        let c_package = match package {
            Some(p) => Some(ffi::CString::new(p)?),
            None    => None,
        };
        let c_locale = ffi::CString::new(locale)?;
        // Safety: same as try_new().
        let rep = unsafe {
            versioned_function!(ures_openDirect)(
                c_package.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
                c_locale.as_ptr(),
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        assert!(!rep.is_null());
        Ok(UResourceBundle { rep })
    }

    /// Returns the locale for this resource bundle, for the given locale type.
    ///
    /// Pass [`sys::ULocDataLocaleType::ULOC_ACTUAL_LOCALE`] to get the locale
    /// for which data actually exists (after ICU fallback); pass
    /// [`sys::ULocDataLocaleType::ULOC_VALID_LOCALE`] to get the most specific
    /// locale for which a bundle exists in the hierarchy.
    ///
    /// Implements `ures_getLocaleByType`.
    pub fn get_locale_by_type(
        &self,
        data_loc_type: sys::ULocDataLocaleType,
    ) -> Result<String, common::Error> {
        let mut status = common::Error::OK_CODE;
        let raw = unsafe {
            versioned_function!(ures_getLocaleByType)(
                self.rep,
                data_loc_type,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        assert!(!raw.is_null());
        // Safety: ICU guarantees locale IDs are null-terminated ASCII strings.
        let c_str = unsafe { ffi::CStr::from_ptr(raw) };
        Ok(c_str.to_str().expect("ICU locale IDs are always valid UTF-8").to_string())
    }

    /// Returns the type of this resource.
    ///
    /// Implements `ures_getType`.
    pub fn resource_type(&self) -> ResourceType {
        // Safety: self.rep is always a valid pointer.
        let t = unsafe { versioned_function!(ures_getType)(self.rep) };
        ResourceType::from(t)
    }

    /// Returns the key associated with this resource, if any.
    ///
    /// Items inside a TABLE resource always have a key (`Some`).
    /// Items inside an ARRAY resource never have a key (`None`).
    /// The root bundle returned by [`try_new`] also has no key, since
    /// it is not an entry inside any parent resource.
    ///
    /// Implements `ures_getKey`.
    pub fn key(&self) -> Option<String> {
        // Safety: ures_getKey returns NULL or a pointer into the bundle's
        // internal memory that is valid for the lifetime of &self.
        let raw = unsafe { versioned_function!(ures_getKey)(self.rep) };
        if raw.is_null() {
            return None;
        }
        // Safety: ICU guarantees resource keys are null-terminated ASCII strings.
        let c_str = unsafe { ffi::CStr::from_ptr(raw) };
        Some(c_str.to_str().expect("ICU resource keys are always valid UTF-8").to_string())
    }

    /// Returns the number of items in this resource.
    ///
    /// For scalar types (strings, integers) this is always 1.  For TABLE and
    /// ARRAY resources this is the number of direct children.
    ///
    /// Implements `ures_getSize`.
    pub fn len(&self) -> usize {
        // Safety: self.rep is always a valid pointer; ures_getSize is infallible.
        let n = unsafe { versioned_function!(ures_getSize)(self.rep) };
        // ICU documents the return as >= 0; cast is safe.
        n as usize
    }

    /// Returns the string value of this resource as a UTF-8 [`String`].
    ///
    /// ICU stores strings in UTF-16; this method converts to UTF-8.  Returns
    /// an error if the resource is not of type [`ResourceType::String`] or
    /// [`ResourceType::Alias`].
    ///
    /// Implements `ures_getString`.
    pub fn get_string(&self) -> Result<String, common::Error> {
        let mut status = common::Error::OK_CODE;
        let mut len: i32 = 0;
        // Safety: self.rep is valid; the returned pointer points into the
        // bundle's internal memory and is valid for &self's lifetime.
        // We convert immediately to an owned String, so the lifetime is irrelevant.
        let raw = unsafe {
            versioned_function!(ures_getString)(self.rep, &mut len, &mut status)
        };
        common::Error::ok_or_warning(status)?;
        assert!(!raw.is_null());
        // Safety: raw points to exactly `len` UChar (= u16) values.
        let slice = unsafe { std::slice::from_raw_parts(raw, len as usize) };
        String::from_utf16(slice).map_err(|e| common::Error::Wrapper(anyhow!(e)))
    }

    /// Returns the binary data of this resource as an owned `Vec<u8>`.
    ///
    /// Returns an error if the resource is not of type [`ResourceType::Binary`].
    ///
    /// Implements `ures_getBinary`.
    pub fn get_binary(&self) -> Result<Vec<u8>, common::Error> {
        let mut status = common::Error::OK_CODE;
        let mut len: i32 = 0;
        // Safety: self.rep is valid; the returned pointer is valid for &self.
        // We copy immediately into an owned Vec, so the bundle lifetime is irrelevant.
        let raw = unsafe {
            versioned_function!(ures_getBinary)(self.rep, &mut len, &mut status)
        };
        common::Error::ok_or_warning(status)?;
        assert!(!raw.is_null());
        // Safety: raw points to `len` bytes owned by the bundle, valid for &self.
        Ok(unsafe { std::slice::from_raw_parts(raw, len as usize) }.to_vec())
    }

    /// Returns the integer vector of this resource as an owned `Vec<i32>`.
    ///
    /// Returns an error if the resource is not of type
    /// [`ResourceType::IntVector`].
    ///
    /// Implements `ures_getIntVector`.
    pub fn get_int_vector(&self) -> Result<Vec<i32>, common::Error> {
        let mut status = common::Error::OK_CODE;
        let mut len: i32 = 0;
        // Safety: self.rep is valid; the returned pointer is valid for &self.
        // We copy immediately into an owned Vec, so the bundle lifetime is irrelevant.
        let raw = unsafe {
            versioned_function!(ures_getIntVector)(self.rep, &mut len, &mut status)
        };
        common::Error::ok_or_warning(status)?;
        assert!(!raw.is_null());
        // Safety: raw points to `len` i32 values owned by the bundle, valid for &self.
        // i32 has the same representation as int32_t on all supported platforms.
        Ok(unsafe { std::slice::from_raw_parts(raw, len as usize) }.to_vec())
    }

    /// Returns the signed integer value of this resource.
    ///
    /// ICU stores integer resources as 28-bit values; the result is
    /// sign-extended to 32 bits.
    ///
    /// Returns an error if the resource is not of type [`ResourceType::Int`].
    ///
    /// Implements `ures_getInt`.
    pub fn get_int(&self) -> Result<i32, common::Error> {
        let mut status = common::Error::OK_CODE;
        // Safety: self.rep is valid.
        let val = unsafe {
            versioned_function!(ures_getInt)(self.rep, &mut status)
        };
        common::Error::ok_or_warning(status)?;
        Ok(val)
    }

    /// Returns the unsigned integer value of this resource.
    ///
    /// This is a reinterpretation of the same 28-bit storage as [`get_int`].
    ///
    /// Returns an error if the resource is not of type [`ResourceType::Int`].
    ///
    /// Implements `ures_getUInt`.
    ///
    /// [`get_int`]: UResourceBundle::get_int
    pub fn get_uint(&self) -> Result<u32, common::Error> {
        let mut status = common::Error::OK_CODE;
        // Safety: self.rep is valid.
        let val = unsafe {
            versioned_function!(ures_getUInt)(self.rep, &mut status)
        };
        common::Error::ok_or_warning(status)?;
        Ok(val)
    }

    /// Returns the sub-resource identified by `key` in a TABLE resource.
    ///
    /// The returned bundle is **independently owned**: it has its own lifetime
    /// and is closed when dropped, regardless of `self`'s lifetime.
    ///
    /// Returns an error if `self` is not a TABLE or the key does not exist.
    ///
    /// Implements `ures_getByKey`.
    /// Uses `fillIn = NULL` so that ICU allocates a fresh bundle with no aliasing to the parent.
    pub fn get_by_key(&self, key: &str) -> Result<Self, common::Error> {
        let mut status = common::Error::OK_CODE;
        let c_key = ffi::CString::new(key)?;
        // Safety: self.rep and c_key are valid; fill-in=NULL so ICU allocates.
        let rep = unsafe {
            versioned_function!(ures_getByKey)(
                self.rep,
                c_key.as_ptr(),
                std::ptr::null_mut(), // fill-in: ICU allocates a new bundle
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        assert!(!rep.is_null());
        Ok(UResourceBundle { rep })
    }

    /// Returns the sub-resource at the zero-based `index` in an ARRAY or
    /// TABLE resource.
    ///
    /// The returned bundle is independently owned.
    ///
    /// Returns an error if `self` is not indexable or `index` is out of range.
    ///
    /// Implements `ures_getByIndex`.
    /// Uses `fillIn = NULL` so that ICU allocates a fresh bundle.
    pub fn get_by_index(&self, index: i32) -> Result<Self, common::Error> {
        let mut status = common::Error::OK_CODE;
        // Safety: self.rep is valid; fill-in=NULL.
        let rep = unsafe {
            versioned_function!(ures_getByIndex)(
                self.rep,
                index,
                std::ptr::null_mut(), // fill-in: ICU allocates a new bundle
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        assert!(!rep.is_null());
        Ok(UResourceBundle { rep })
    }

    /// Returns the string value of the resource identified by `key` in a
    /// TABLE resource.
    ///
    /// Equivalent to `self.get_by_key(key)?.get_string()` but more efficient:
    /// no intermediate `UResourceBundle` is allocated.
    ///
    /// Implements `ures_getStringByKey`.
    pub fn get_string_by_key(&self, key: &str) -> Result<String, common::Error> {
        let mut status = common::Error::OK_CODE;
        let mut len: i32 = 0;
        let c_key = ffi::CString::new(key)?;
        // Safety: self.rep and c_key are valid; returned pointer is valid for &self.
        let raw = unsafe {
            versioned_function!(ures_getStringByKey)(
                self.rep,
                c_key.as_ptr(),
                &mut len,
                &mut status,
            )
        };
        common::Error::ok_or_warning(status)?;
        assert!(!raw.is_null());
        // Safety: raw points to `len` UChar (u16) values valid for &self.
        // We convert immediately so the lifetime is irrelevant after this point.
        let slice = unsafe { std::slice::from_raw_parts(raw, len as usize) };
        String::from_utf16(slice).map_err(|e| common::Error::Wrapper(anyhow!(e)))
    }

    /// Returns the string value at the zero-based `index` in an ARRAY or
    /// TABLE resource.
    ///
    /// Implements `ures_getStringByIndex`.
    pub fn get_string_by_index(&self, index: i32) -> Result<String, common::Error> {
        let mut status = common::Error::OK_CODE;
        let mut len: i32 = 0;
        // Safety: self.rep is valid; returned pointer is valid for &self.
        let raw = unsafe {
            versioned_function!(ures_getStringByIndex)(self.rep, index, &mut len, &mut status)
        };
        common::Error::ok_or_warning(status)?;
        assert!(!raw.is_null());
        // Safety: raw points to `len` UChar (u16) values valid for &self.
        let slice = unsafe { std::slice::from_raw_parts(raw, len as usize) };
        String::from_utf16(slice).map_err(|e| common::Error::Wrapper(anyhow!(e)))
    }

    /// Resets the iteration position to the first element.
    ///
    /// Implements `ures_resetIterator`.
    pub fn reset_iterator(&mut self) {
        // Safety: self.rep is a valid *mut UResourceBundle.
        unsafe { versioned_function!(ures_resetIterator)(self.rep) };
    }

    /// Returns `true` if there are more elements to iterate over.
    ///
    /// Implements `ures_hasNext`.
    pub fn has_next(&self) -> bool {
        // Safety: self.rep is valid.  ures_hasNext takes *const, which is
        // safe to call concurrently with other const-taking functions;
        // however, UResourceBundle is !Sync so this is only callable from
        // the owning thread anyway.
        let b = unsafe { versioned_function!(ures_hasNext)(self.rep) };
        b != 0
    }

    /// Returns the next sub-resource, or `None` when iteration is exhausted.
    ///
    /// Each call advances the internal iterator by one step.  Use
    /// [`reset_iterator`] to restart.  The returned bundle is independently
    /// owned.
    ///
    /// Implements `ures_getNextResource`.
    /// Uses `fillIn = NULL` so that ICU allocates a fresh bundle.
    ///
    /// [`reset_iterator`]: UResourceBundle::reset_iterator
    pub fn next_resource(&mut self) -> Option<Result<Self, common::Error>> {
        let mut status = common::Error::OK_CODE;
        // Safety: self.rep is a valid *mut UResourceBundle; fill-in=NULL so
        // ICU allocates a fresh bundle (no aliasing with the parent).
        let rep = unsafe {
            versioned_function!(ures_getNextResource)(
                self.rep,
                std::ptr::null_mut(), // fill-in: ICU allocates a new bundle
                &mut status,
            )
        };
        if rep.is_null() {
            // ICU signals end-of-iteration by returning NULL and setting
            // U_INDEX_OUTOFBOUNDS_ERROR.  This is not a real error; treat it
            // as the natural end of the sequence.
            return if common::Error::is_ok(status)
                || status == sys::UErrorCode::U_INDEX_OUTOFBOUNDS_ERROR
            {
                None
            } else {
                Some(Err(common::Error::Sys(status)))
            };
        }
        if let Err(e) = common::Error::ok_or_warning(status) {
            // Non-null pointer with an error: ICU may have leaked; close it.
            unsafe { versioned_function!(ures_close)(rep) };
            return Some(Err(e));
        }
        Some(Ok(UResourceBundle { rep }))
    }

    /// Returns the next string value and its optional key, or `None` when
    /// iteration is exhausted.
    ///
    /// The key is `Some` for TABLE resources (named items) and `None` for
    /// ARRAY resources (anonymous items).
    ///
    /// Implements `ures_getNextString`.
    pub fn next_string(&mut self) -> Option<Result<(String, Option<String>), common::Error>> {
        let mut status = common::Error::OK_CODE;
        let mut len: i32 = 0;
        let mut key_ptr: *const raw::c_char = std::ptr::null();
        // Safety: self.rep is a valid *mut UResourceBundle; key_ptr and len
        // are output parameters initialised above.
        let raw = unsafe {
            versioned_function!(ures_getNextString)(
                self.rep,
                &mut len,
                &mut key_ptr,
                &mut status,
            )
        };
        if raw.is_null() {
            // ICU signals end-of-iteration with NULL + U_INDEX_OUTOFBOUNDS_ERROR.
            // When the next resource exists but is not a string, ures_getNextString
            // advances the iterator, returns NULL, and leaves the status unchanged
            // (U_ZERO_ERROR).  Treat that as a type mismatch rather than as
            // end-of-iteration.
            return if status == sys::UErrorCode::U_INDEX_OUTOFBOUNDS_ERROR {
                None
            } else if common::Error::is_ok(status) {
                Some(Err(common::Error::Sys(
                    sys::UErrorCode::U_RESOURCE_TYPE_MISMATCH,
                )))
            } else {
                Some(Err(common::Error::Sys(status)))
            };
        }
        if let Err(e) = common::Error::ok_or_warning(status) {
            return Some(Err(e));
        }
        // Safety: raw points to exactly `len` UChar (u16) code units.  We
        // convert immediately to an owned String before any further mutation
        // of the iterator.
        let slice = unsafe { std::slice::from_raw_parts(raw, len as usize) };
        let string = match String::from_utf16(slice) {
            Ok(s)  => s,
            Err(e) => return Some(Err(common::Error::Wrapper(anyhow!(e)))),
        };
        // Safety: if non-null, key_ptr is a null-terminated ASCII string
        // owned by the bundle, valid until the iterator advances again.  We
        // copy it to an owned String immediately.
        let key = if key_ptr.is_null() {
            None
        } else {
            let c_str = unsafe { ffi::CStr::from_ptr(key_ptr) };
            Some(
                c_str
                    .to_str()
                    .expect("ICU resource keys are always valid UTF-8")
                    .to_string(),
            )
        };
        Some(Ok((string, key)))
    }
}

/// Returns an enumeration of all locales available in `package`.
///
/// Pass `None` for `package` to enumerate locales in ICU's built-in data.
///
/// The returned [`Enumeration`] yields locale ID strings (e.g. `"en"`, `"fr"`,
/// `"zh_Hans"`).
///
/// # Package requirements
///
/// When `package` is `Some`, the package directory must contain a compiled
/// `res_index.res` file alongside the per-locale `.res` files.  ICU reads
/// the `InstalledLocales` table from that bundle to build the enumeration.
/// If `res_index.res` is absent, ICU returns `U_MISSING_RESOURCE_ERROR`.
///
/// To generate `res_index.res`, create a `res_index.txt` source file:
///
/// ```text
/// res_index:table(nofallback) {
///     InstalledLocales {
///         fr   { "" }
///         root { "" }
///     }
/// }
/// ```
///
/// and compile it with `genrb` into the same output directory as the other
/// `.res` files.
///
/// Implements `ures_openAvailableLocales`.
pub fn open_available_locales(package: Option<&str>) -> Result<Enumeration, common::Error> {
    let mut status = common::Error::OK_CODE;
    let c_package = match package {
        Some(p) => Some(ffi::CString::new(p)?),
        None    => None,
    };
    // Safety: pointer is valid for the duration of the call; the returned
    // *mut UEnumeration is transferred to the Enumeration wrapper.
    let rep = unsafe {
        versioned_function!(ures_openAvailableLocales)(
            c_package.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
            &mut status,
        )
    };
    common::Error::ok_or_warning(status)?;
    assert!(!rep.is_null());
    // Safety: rep is a valid *mut UEnumeration.  Enumeration takes ownership
    // and will call uenum_close on drop.
    Ok(unsafe { Enumeration::from_raw_parts(None, rep) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::{Arc, Mutex}, thread};

    /// Path to the directory containing the compiled test `.res` files.
    /// Set at compile time by build.rs via `cargo:rustc-env=TEST_DATA_DIR`.
    const TEST_PKG: &str = env!("TEST_DATA_DIR");

    #[test]
    fn open_builtin_root_is_table() {
        // ICU's built-in root locale is always available and is a TABLE.
        let bundle = UResourceBundle::try_new(None, "root").unwrap();
        assert_eq!(bundle.resource_type(), ResourceType::Table);
    }

    #[test]
    fn open_test_bundle_root_locale() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        assert_eq!(bundle.resource_type(), ResourceType::Table);
    }

    #[test]
    fn open_test_bundle_fr_locale() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "fr").unwrap();
        assert_eq!(bundle.resource_type(), ResourceType::Table);
    }

    #[test]
    fn open_direct_fr_succeeds() {
        // try_new_direct on an existing locale must succeed.
        let bundle = UResourceBundle::try_new_direct(Some(TEST_PKG), "fr").unwrap();
        assert_eq!(bundle.resource_type(), ResourceType::Table);
    }

    #[test]
    fn open_direct_missing_locale_returns_error() {
        // "de" does not exist in the test data; try_new_direct must return an error.
        let err = UResourceBundle::try_new_direct(Some(TEST_PKG), "de").unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_MISSING_RESOURCE_ERROR));
    }


    #[test]
    fn get_locale_by_type_valid_locale() {
        let root = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        assert_eq!(
            root.get_locale_by_type(sys::ULocDataLocaleType::ULOC_VALID_LOCALE).unwrap(),
            "root"
        );
        let fr = UResourceBundle::try_new(Some(TEST_PKG), "fr").unwrap();
        assert_eq!(
            fr.get_locale_by_type(sys::ULocDataLocaleType::ULOC_VALID_LOCALE).unwrap(),
            "fr"
        );
    }

    #[test]
    fn locale_root() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        assert_eq!(
            bundle.get_locale_by_type(sys::ULocDataLocaleType::ULOC_ACTUAL_LOCALE).unwrap(),
            "root"
        );
    }

    #[test]
    fn locale_fr() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "fr").unwrap();
        assert_eq!(
            bundle.get_locale_by_type(sys::ULocDataLocaleType::ULOC_ACTUAL_LOCALE).unwrap(),
            "fr"
        );
    }


    #[test]
    fn top_level_bundle_key_is_none() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        assert_eq!(bundle.key(), None);
    }

    #[test]
    fn table_item_key_and_type() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let greeting = bundle.get_by_key("greeting").unwrap();
        assert_eq!(greeting.key(), Some("greeting".to_string()));
        assert_eq!(greeting.resource_type(), ResourceType::String);
    }


    #[test]
    fn get_string_root_greeting() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        assert_eq!(bundle.get_by_key("greeting").unwrap().get_string().unwrap(), "Hello");
    }

    #[test]
    fn get_string_fr_greeting() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "fr").unwrap();
        assert_eq!(bundle.get_by_key("greeting").unwrap().get_string().unwrap(), "Bonjour");
    }

    #[test]
    fn get_string_by_key_root() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        assert_eq!(bundle.get_string_by_key("greeting").unwrap(), "Hello");
        assert_eq!(bundle.get_string_by_key("farewell").unwrap(), "Goodbye");
    }

    #[test]
    fn get_string_by_key_fr() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "fr").unwrap();
        assert_eq!(bundle.get_string_by_key("greeting").unwrap(), "Bonjour");
        assert_eq!(bundle.get_string_by_key("farewell").unwrap(), "Au revoir");
    }

    #[test]
    fn get_string_by_key_missing_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let err = bundle.get_string_by_key("no_such_key").unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_MISSING_RESOURCE_ERROR));
    }

    #[test]
    fn fr_bundle_falls_back_to_root_for_missing_keys() {
        // "fr" bundle does not define "count"; it must fall back to "root".
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "fr").unwrap();
        let count = bundle.get_by_key("count").unwrap();
        assert_eq!(count.get_int().unwrap(), 42);
    }


    #[test]
    fn get_int() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let count = bundle.get_by_key("count").unwrap();
        assert_eq!(count.resource_type(), ResourceType::Int);
        assert_eq!(count.get_int().unwrap(), 42);
    }

    #[test]
    fn get_uint() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let count = bundle.get_by_key("count").unwrap();
        assert_eq!(count.get_uint().unwrap(), 42u32);
    }


    #[test]
    fn get_int_vector() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let weights = bundle.get_by_key("weights").unwrap();
        assert_eq!(weights.resource_type(), ResourceType::IntVector);
        assert_eq!(weights.get_int_vector().unwrap(), &[10_i32, 20, 30]);
    }

    #[test]
    fn get_binary() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let data = bundle.get_by_key("data").unwrap();
        assert_eq!(data.resource_type(), ResourceType::Binary);
        assert_eq!(data.get_binary().unwrap(), &[0x01_u8, 0x02, 0x03]);
    }


    #[test]
    fn resource_type_alias() {
        // get_by_key follows aliases transparently: the returned resource has
        // the type of the aliased target, not ResourceType::Alias.
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let alias = bundle.get_by_key("greeting_alias").unwrap();
        assert_eq!(alias.resource_type(), ResourceType::String);
    }

    #[test]
    fn resource_type_array() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let phrases = bundle.get_by_key("phrases").unwrap();
        assert_eq!(phrases.resource_type(), ResourceType::Array);
    }

    #[test]
    fn array_item_key_is_none() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let mut phrases = bundle.get_by_key("phrases").unwrap();
        while let Some(item) = phrases.next_resource() {
            assert_eq!(item.unwrap().key(), None, "array items must not have keys");
        }
    }

    #[test]
    fn get_string_by_index_on_array() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let phrases = bundle.get_by_key("phrases").unwrap();
        assert_eq!(phrases.get_string_by_index(0).unwrap(), "one");
        assert_eq!(phrases.get_string_by_index(1).unwrap(), "two");
        assert_eq!(phrases.get_string_by_index(2).unwrap(), "three");
    }

    #[test]
    fn get_string_by_index_out_of_range_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let phrases = bundle.get_by_key("phrases").unwrap();
        let err = phrases.get_string_by_index(999).unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_MISSING_RESOURCE_ERROR));
    }

    #[test]
    fn get_by_key_on_non_table_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let greeting = bundle.get_by_key("greeting").unwrap();
        let err = greeting.get_by_key("anything").unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_RESOURCE_TYPE_MISMATCH));
    }

    #[test]
    fn get_string_by_key_on_non_table_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let greeting = bundle.get_by_key("greeting").unwrap();
        let err = greeting.get_string_by_key("anything").unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_RESOURCE_TYPE_MISMATCH));
    }

    #[test]
    fn next_string_returns_error_for_non_string_resource() {
        // "mixed" has a string ("label") followed by an int ("number").
        // ICU TABLE entries are stored in sorted key order: "label" < "number",
        // so the second call to next_string() hits the int and must return an error.
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let mut mixed = bundle.get_by_key("mixed").unwrap();
        let first = mixed.next_string().unwrap().unwrap();
        assert_eq!(first.0, "hello");
        let err = mixed.next_string().unwrap().unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_RESOURCE_TYPE_MISMATCH));
        assert!(mixed.next_string().is_none());
    }

    #[test]
    fn get_string_type_mismatch_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let count = bundle.get_by_key("count").unwrap();
        let err = count.get_string().unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_RESOURCE_TYPE_MISMATCH));
    }

    #[test]
    fn get_int_type_mismatch_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let greeting = bundle.get_by_key("greeting").unwrap();
        let err = greeting.get_int().unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_RESOURCE_TYPE_MISMATCH));
    }

    #[test]
    fn get_uint_type_mismatch_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let greeting = bundle.get_by_key("greeting").unwrap();
        let err = greeting.get_uint().unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_RESOURCE_TYPE_MISMATCH));
    }

    #[test]
    fn get_int_vector_type_mismatch_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let greeting = bundle.get_by_key("greeting").unwrap();
        let err = greeting.get_int_vector().unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_RESOURCE_TYPE_MISMATCH));
    }

    #[test]
    fn get_binary_type_mismatch_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let greeting = bundle.get_by_key("greeting").unwrap();
        let err = greeting.get_binary().unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_RESOURCE_TYPE_MISMATCH));
    }

    #[test]
    fn get_string_alias_to_string_succeeds() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let alias = bundle.get_by_key("greeting_alias").unwrap();
        assert_eq!(alias.get_string().unwrap(), "Hello");
    }

    #[test]
    fn get_string_alias_to_non_string_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let alias = bundle.get_by_key("count_alias").unwrap();
        let err = alias.get_string().unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_RESOURCE_TYPE_MISMATCH));
    }

    #[test]
    fn get_nested_table_root() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let errors = bundle.get_by_key("errors").unwrap();
        assert_eq!(errors.resource_type(), ResourceType::Table);
        assert_eq!(errors.get_string_by_key("not_found").unwrap(), "Not found");
        assert_eq!(errors.get_string_by_key("forbidden").unwrap(), "Forbidden");
    }

    #[test]
    fn get_nested_table_fr() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "fr").unwrap();
        let errors = bundle.get_by_key("errors").unwrap();
        assert_eq!(errors.get_string_by_key("not_found").unwrap(), "Introuvable");
        assert_eq!(errors.get_string_by_key("forbidden").unwrap(), "Interdit");
    }


    #[test]
    fn get_by_index_and_string_by_index() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let errors = bundle.get_by_key("errors").unwrap();
        // TABLE items are accessible by index.
        assert!(errors.get_by_index(0).is_ok());
        let s = errors.get_string_by_index(0).unwrap();
        assert!(!s.is_empty(), "expected non-empty string at index 0");
    }

    #[test]
    fn get_by_index_out_of_range_returns_error() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let errors = bundle.get_by_key("errors").unwrap();
        let err = errors.get_by_index(999).unwrap_err();
        assert!(err.is_code(sys::UErrorCode::U_MISSING_RESOURCE_ERROR));
    }


    #[test]
    fn len_on_root_bundle() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        // root has greeting, farewell, count, weights, errors → at least 5.
        assert!(bundle.len() >= 5, "expected ≥ 5 items, got {}", bundle.len());
    }

    #[test]
    fn len_on_scalar_is_one() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let greeting = bundle.get_by_key("greeting").unwrap();
        assert_eq!(greeting.len(), 1);
    }


    #[test]
    fn iterate_table_yields_all_items() {
        let mut bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let expected = bundle.len();
        let mut count = 0;
        while let Some(item) = bundle.next_resource() {
            item.unwrap();
            count += 1;
        }
        assert_eq!(count, expected);
    }

    #[test]
    fn reset_iterator_and_iterate_twice() {
        let mut bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let keys_pass1 = collect_keys(&mut bundle);
        bundle.reset_iterator();
        let keys_pass2 = collect_keys(&mut bundle);
        assert_eq!(keys_pass1, keys_pass2, "iteration order must be stable");
    }

    fn collect_keys(bundle: &mut UResourceBundle) -> Vec<String> {
        let mut keys = vec![];
        while let Some(item) = bundle.next_resource() {
            if let Some(k) = item.unwrap().key() {
                keys.push(k.to_string());
            }
        }
        keys
    }

    #[test]
    fn has_next_reflects_iteration_state() {
        let mut bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        assert!(bundle.has_next(), "fresh bundle should have items");
        // Exhaust the iterator.
        while bundle.next_resource().is_some() {}
        assert!(!bundle.has_next(), "exhausted iterator should report no next");
        bundle.reset_iterator();
        assert!(bundle.has_next(), "reset iterator should have items again");
    }

    #[test]
    fn next_string_yields_table_strings_with_keys() {
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let mut errors = bundle.get_by_key("errors").unwrap();
        let mut pairs: Vec<(String, Option<String>)> = vec![];
        while let Some(result) = errors.next_string() {
            pairs.push(result.unwrap());
        }
        assert_eq!(pairs.len(), 2, "expected 2 string items in errors table");
        // Every item in a TABLE should have a key.
        for (_, key) in &pairs {
            assert!(key.is_some(), "table items must have keys");
        }
        let keys: Vec<&str> = pairs.iter().filter_map(|(_, k)| k.as_deref()).collect();
        assert!(keys.contains(&"not_found"));
        assert!(keys.contains(&"forbidden"));
    }


    #[test]
    fn open_available_locales_from_builtin_is_nonempty() {
        let locales: Vec<String> = open_available_locales(None)
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(!locales.is_empty(), "expected at least one available ICU locale");
    }

    #[test]
    fn open_available_locales_from_test_package() {
        let mut locales: Vec<String> = open_available_locales(Some(TEST_PKG))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        locales.sort();
        assert_eq!(locales, &["fr", "root"]);
    }


    #[test]
    fn resource_bundle_is_send() {
        // A bundle may be moved to another thread and used there.
        let bundle = UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap();
        let handle = thread::spawn(move || {
            bundle.get_string_by_key("greeting").unwrap()
        });
        assert_eq!(handle.join().unwrap(), "Hello");
    }

    #[test]
    fn resource_bundle_in_mutex_is_usable_from_multiple_threads() {
        // Although UResourceBundle is !Sync, wrapping it in Mutex makes it
        // usable from multiple threads safely.
        let shared = Arc::new(Mutex::new(
            UResourceBundle::try_new(Some(TEST_PKG), "root").unwrap(),
        ));
        let shared2 = Arc::clone(&shared);
        let handle = thread::spawn(move || {
            shared2.lock().unwrap().get_string_by_key("greeting").unwrap()
        });
        let from_main = shared.lock().unwrap().get_string_by_key("farewell").unwrap();
        let from_thread = handle.join().unwrap();
        assert_eq!(from_main, "Goodbye");
        assert_eq!(from_thread, "Hello");
    }
}
