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

use std::path::Path;
use std::ffi;

use {
    rust_icu_common as common, rust_icu_sys as sys, rust_icu_sys::versioned_function,
    std::convert::TryFrom, std::os::raw,
};

/// Variants of [UDataMemory].
#[derive(Debug)]
enum Rep {
    /// The data memory is backed by a user-supplied buffer.
    Buffer(Vec<u8>),
    /// The data memory is backed by a resource file.
    Resource(
        // This would have been std::ptr::NonNull if we didn't have to
        // implement Send and Sync.
        // We only ever touch this pointer in Rust when we initialize
        // Rep::Resource, and when we dealocate Rep::Resource.
        *const sys::UDataMemory,
    ),
}

// Safety: The *const sys::UDataMemory above is only used by the underlying C++
// library.
unsafe impl Send for Rep {}
unsafe impl Sync for Rep {}


/// Implements `UDataMemory`.
///
/// Represents data memory backed by a borrowed memory buffer used for loading ICU data.
/// [UDataMemory] is very much not thread safe, as it affects the global state of the ICU library.
/// This suggests that the best way to use this data is to load it up in a main thread, or access
/// it through a synchronized wrapper.
#[derive(Debug)]
pub struct UDataMemory {
    // The internal representation of [UDataMemory].
    // May vary, depending on the way the struct is created.
    //
    // See: [UDataMemory::try_from] and [UDataMemory::open].
    rep: Rep,
}

impl Drop for UDataMemory {
    // Implements `u_cleanup`.
    fn drop(&mut self) {
        if let Rep::Resource(r) = self.rep {
            unsafe {
                // Safety: there is no other way to close the memory that the
                // underlying C++ library uses but to pass it into this function.
                versioned_function!(udata_close)(r as *mut sys::UDataMemory)
            };
        }
        // Without this, resource references will remain, but memory will be gone.
        unsafe {
            // Safety: no other way to call this function.
            versioned_function!(u_cleanup)()
        };
    }
}

impl TryFrom<Vec<u8>> for crate::UDataMemory {
    type Error = common::Error;
    /// Makes a UDataMemory out of a buffer.
    ///
    /// Implements `udata_setCommonData`.
    fn try_from(buf: Vec<u8>) -> Result<Self, Self::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        // Expects that buf is a valid pointer and that it contains valid
        // ICU data.  If data is invalid, an error status will be set.
        // No guarantees for invalid pointers.
        unsafe {
            versioned_function!(udata_setCommonData)(
                buf.as_ptr() as *const raw::c_void,
                &mut status,
            );
        };
        common::Error::ok_or_warning(status)?;
        Ok(UDataMemory { rep: Rep::Buffer(buf) })
    }
}

impl crate::UDataMemory {

    /// Uses the resources from the supplied resource file.
    ///
    /// This may end up being more efficient compared to loading from a buffer,
    /// as ostensibly the resources would be memory mapped to only the needed
    /// parts.
    ///
    /// The `path` is the file path at which to find the resource file.
    /// The `a_type` is the type of the resource file (can be empty).
    /// The `name` is the name of the resource file (can also be empty).
    ///
    /// Implements `udata_open`.
    pub fn open(path: &Path, a_type: &str, name: &str) -> Result<Self, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;
        let path_cstr = ffi::CString::new(path.to_str().unwrap())?;
        let name_cstr = ffi::CString::new(name)?;
        let type_cstr = ffi::CString::new(a_type)?;
        let rep = unsafe {
            // Safety: we do what we must to call the underlying unsafe C API, and only return an
            // opaque enum, to ensure that no rust client code may touch the raw pointer.
            assert!(common::Error::is_ok(status));

            // Would be nicer if there were examples of udata_open usage to
            // verify this.
            let rep: *const sys::UDataMemory = versioned_function!(udata_open)(
                path_cstr.as_ptr(),
                if type_cstr.is_empty() {
                    std::ptr::null()
                } else {
                    type_cstr.as_ptr()
                },
                if name_cstr.is_empty() {
                    std::ptr::null()
                } else {
                    name_cstr.as_ptr()
                },
                &mut status);
            // Sadly we can not use NonNull, as we can not make the resulting
            // type Sync or Send.
            assert!(!rep.is_null());
            Rep::Resource(rep)
        };
        common::Error::ok_or_warning(status)?;
        Ok(crate::UDataMemory{ rep })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::sync::{Mutex, Weak, Arc};
    use std::thread;

    // We don't use UDataMemory in threaded contexts, but our users do. So let's
    // ensure we can do this.
    #[test]
    fn send_sync_impl() {
        let memory: Arc<Mutex<Weak<UDataMemory>>>= Arc::new(Mutex::new(Weak::new()));
        // Ensure Sync.
        let _clone = memory.clone();
        thread::spawn(move || {
            // Ensure Send.
            let _m = memory;
        });
    }

    #[test]
    fn send_impl() {
        let memory: Weak<UDataMemory> = Weak::new();
        let _clone = memory.clone();
        thread::spawn(move || {
            let _m = memory;
        });
    }
}
