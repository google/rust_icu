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

//! # Rust bindings for the ICU library
//!
//! The [ICU](http://site.icu-project.org/home) library is a C, C++ and Java library providing
//! Unicode functionality.  This crate makes the ICU functionality available to programs in rust.
//!
//! This top level crate serves two purposes:
//!
//! 1. Makes all ICU functionality providing crates available through a single
//! import.
//! 2. Provides a unified namespace for all of this functionality to live in.
//!
//! The crate makes the following reexports:
//!
//! | Original | Remapped |
//! | -------- | -------- |
//! | rust_icu_common | icu::common |
//! | rust_icu_sys | icu::sys |
//! | rust_icu_ubrk | brk |
//! | rust_icu_ucal | icu::cal |
//! | rust_icu_ucol | icu::col |
//! | rust_icu_udat | icu::dat |
//! | rust_icu_udata | icu::data |
//! | rust_icu_uenum | icu::enums |
//! | rust_icu_ulistformatter | icu::listformatter |
//! | rust_icu_uloc | icu::loc |
//! | rust_icu_umsg | icu::msg |
//! | rust_icu_unorm | unorm |
//! | rust_icu_ustring | icu::string |
//! | rust_icu_utext | text |
//! | rust_icu_utrans | trans |

pub use rust_icu_common as common;
pub use rust_icu_sys as sys;
pub use rust_icu_ubrk as brk;
pub use rust_icu_ucal as cal;
pub use rust_icu_ucol as col;
pub use rust_icu_udat as dat;
pub use rust_icu_udata as data;
pub use rust_icu_uenum as enums;
pub use rust_icu_ulistformatter as listformatter;
pub use rust_icu_uloc as loc;
pub use rust_icu_umsg as msg;
pub use rust_icu_ustring as string;
pub use rust_icu_utext as text;
pub use rust_icu_utrans as trans;
pub use rust_icu_unorm2 as norm;
