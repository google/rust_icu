// Macros for changing function names.
// This is created and edited manually, but should be kept roughly in sync with `build.rs`.

/// This library was build with version renaming, so rewrite every function name
/// with its name with version number appended.

/// The macro will rename a symbol `foo::bar` to `foo::bar_64` (where "64")
/// may be some other number depending on the ICU library in use.
#[cfg(all(feature = "renaming", not(feature = "icu_version_in_env")))]
#[macro_export]
macro_rules! versioned_function {
    ($i:ident) => {
      $crate::__private_do_not_use::paste::expr! {
        $crate::[< $i _ 64 >]
      }
    }
}

/// This library was build with version renaming, so rewrite every function name
/// with its name with version number appended.
///
/// The macro will rename a symbol `foo::bar` to `foo::bar_XX` (where "XX")
/// is a string coming from the environment variable RUST_ICU_MAJOR_VERSION_NUMBER,
/// which is expected to be defined at compile time.
#[cfg(all(feature = "renaming", feature = "icu_version_in_env"))]
#[macro_export]
macro_rules! versioned_function {
    ($i:ident) => {
      $crate::__private_do_not_use::paste::expr! {
        $crate::[< $i _ env!("RUST_ICU_MAJOR_VERSION_NUMBER") >]
      }
    }
}

/// This macro will be used when no function renaming is needed.
#[cfg(not(feature = "renaming"))]
#[macro_export]
macro_rules! versioned_function {
    ($func_name:ident) => {
        $crate::$func_name
    }
}
