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

/// Sets the directory from which to load ICU data resources.
///
/// Implements `u_setDataDirectory`.
pub fn set_data_directory(dir: &Path) {
    let dir_cstr = ffi::CString::new
        (dir.to_str().expect("this should never be a runtime error"))
        .expect("this should never be a runtim error");
    unsafe {
        versioned_function!(u_setDataDirectory)(dir_cstr.as_ptr())
    };
}

/// The type of the ICU resource requested.  Some standard resources have their
/// canned types. In case you run into one that is not captured here, use `Custom`,
/// and consider sending a pull request to add the new resource type.
pub enum Type {
    /// An empty resource type. This is ostensibly allowed, but unclear when
    /// it is applicable.
    Empty,
    /// The unpacked resource type, equivalent to "res" in ICU4C.
    Res,
    /// The cnv resource type, equivalent to "cnv" in ICU4C.
    Cnv,
    /// The "common" data type, equivalent to "dat" in ICU4C.
    Dat,
    /// A custom data type, in case none of the above fit your use case. It
    /// is not clear whether this would ever be useful, but the ICU4C API
    /// allows for it, so we must too.
    Custom(String),
}

impl AsRef<str> for Type {
    fn as_ref(&self) -> &str {
        match self {
            Type::Empty => &"",
            Type::Res => &"res",
            Type::Dat => &"dat",
            Type::Cnv => &"cnv",
            Type::Custom(ref s) => &s,
        }
    }
}

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
    /// - The `path` is the file path at which to find the resource file. Ostensibly
    /// specifying `None` here will load from the "default" ICU_DATA path.
    /// I have not been able to confirm this.
    ///
    /// - The `a_type` is the type of the resource file. It is not clear whether
    /// the resource file type is a closed or open set, so we provide for both
    /// possibilities.
    ///
    /// - The `name` is the name of the resource file. It is documented nullable
    ///   in the ICU documentation. Pass `None` here to pass nullptr to the
    ///   underlying C API.
    ///
    /// Presumably using `UDataMemory::open(Some("/dir/too"), Type::Res, Some("filename")` would
    /// attempt to load ICU data from `/dir/too/filename.res`, as well as some other
    /// canonical permutations of the above.  The full documentation is
    /// [here][1], although I could not confirm that the documentation is actually
    /// describing what the code does.  Also, using `None` at appropriate places
    /// seems to be intended to load data from [some "default" sites][2]. I have
    /// however observed that the actual behavior diverges from that documentation.
    ///
    /// Implements `udata_open`.
    ///
    /// [1]: https://unicode-org.github.io/icu/userguide/icu_data/#how-data-loading-works
    /// [2]: https://unicode-org.github.io/icu/userguide/icu_data/#icu-data-directory
    pub fn open(path: Option<&Path>, a_type: Type, name: Option<&str>) -> Result<Self, common::Error> {
        let mut status = sys::UErrorCode::U_ZERO_ERROR;

        let path_cstr = path.map(|s| { ffi::CString::new(s.to_str().expect("should never be a runtime error")).unwrap()});
        let name_cstr = name.map(|s| { ffi::CString::new(s).expect("should never be a runtime error") } );
        let type_cstr = ffi::CString::new(a_type.as_ref()).expect("should never be a runtime errror");

        let rep = Self::get_resource(
            path_cstr.as_ref().map(|s| s.as_c_str()),
            type_cstr.as_c_str(), 
            name_cstr.as_ref().map(|s| s.as_c_str()), 
            &mut status);
        common::Error::ok_or_warning(status)?;

        // Make sure that all CStrs outlive the call to Self::get_resource. It is
        // all too easy to omit `path_cstr.as_ref()` above, resulting in *_cstr
        // being destroyed before a call to Self::get_resource happens. Fun.
        let (_a, _b, _c) = (path_cstr, name_cstr, type_cstr);
        Ok(crate::UDataMemory{ rep })
    }

    fn get_resource(path: Option<&ffi::CStr>, a_type: &ffi::CStr, name: Option<&ffi::CStr>, status: &mut sys::UErrorCode) -> Rep {
        unsafe {
            // Safety: we do what we must to call the underlying unsafe C API, and only return an
            // opaque enum, to ensure that no rust client code may touch the raw pointer.
            assert!(common::Error::is_ok(*status));

            // Would be nicer if there were examples of udata_open usage to
            // verify this.
            let rep: *const sys::UDataMemory = versioned_function!(udata_open)(
                path.map(|s| s.as_ptr()).unwrap_or(std::ptr::null()),
                a_type.as_ptr(),
                name.map(|c| c.as_ptr()).unwrap_or(std::ptr::null()),
                status);
            // Sadly we can not use NonNull, as we can not make the resulting
            // type Sync or Send.
            assert!(!rep.is_null());
            Rep::Resource(rep)
        }
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
