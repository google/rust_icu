#![doc(html_playground_url = "https://play.rust-lang.org")]
//! # The ECMA 402 abstract API surface proposal for `Intl.ListFormat`
//!
//! *fmil@google.com\
//! Created: 2020-05-28\
//! Last updated: 2020-06-09*
//!
//! This proposal is an elaboration of the article [Meta-proposal: Towards common ECMA 402 API
//! surface for Rust][meta1].  It contains traits declarations for a rust-flavored
//! [ECMA-402 API][ecma402api].
//!
//!   [ecma402api]: https://www.ecma-international.org/publications/standards/Ecma-402.htm
//!   [meta1]: https://github.com/unicode-org/icu4x/blob/%6d%61%73%74%65%72/proposals/pr001-ecma402.md
//!
//! ## A note about presentation
//!
//! This proposal is deliberately written in the form of compilable rust code, and is perhaps best
//! consumed by looking at the output of the command `cargo doc --open` ran at the top level
//! directory.  Such a presentation allows us to embed live code inline with the text, which can
//! be readily tested by clicking the "Run" button:
//!
//! ```rust
//! println!("Hello Unicode! ❤️!");
//! ```
//!
//! It's not quite [literate
//! programming](https://en.wikipedia.org/wiki/Literate_programming) but should be close enough for
//! getting a general feel for how the API will be used, and allows to follow the text together
//! with the presentation.
//!
//! ## Approach
//!
//! As outlined in the [meta-proposal][meta1approach], we will first test the feasibility of an
//! ECMA-402 API with a minimal example.  [`Intl.ListFormat`][listformat] was chosen. It is
//! a very small API surface with few configuration options compared to other members of the same
//! API family, while in general it is very similar to all other functions in the [`Intl`
//! library][intl].
//!
//!   [meta1approach]: https://github.com/unicode-org/icu4x/blob/%6d%61%73%74%65%72/proposals/pr001-ecma402.md#approach
//!   [listformat]:
//!   https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat
//!   [intl]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl
//!
//! # A closer look at `Intl.ListFormat`
//!
//! A [quick example of `Intl.ListFormat` use][lfquick] in JavaScript is shown below, for
//! completeness and as a baseline for comparison.
//!
//!   [lfquick]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat/ListFormat
//!
//! ```js
//! const vehicles = ['Motorcycle', 'Bus', 'Car'];
//!
//! const formatter = new Intl.ListFormat('en', { style: 'long', type: 'conjunction' });
//! console.log(formatter.format(vehicles));
//! // expected output: "Motorcycle, Bus, and Car"
//! ```
//!
//! Per [MDN][lfquick], disregarding the JavaScript declaration overhead, the following functions
//! are the focus:
//!
//! |  | Description |
//! |--|--|
//! | `Intl.supportedLocalesOf() => [String]` | Creates a ListFormat object, initialized for the given locale and options set |
//! | `Intl.ListFormat(locale..., options) => <ListFormat object>` | Creates a ListFormat object, initialized for the given locale and options set |
//! | `<ListFormat object>.format([element,...]) => <String>` | Formats the list of `element`s according to the formatting rules for `locale` that the `<ListFormat object>` was initialized with. |
//! | `<ListFormat object>.formatToParts([element,...]) => [String]` | Formats the list of `element`s according to the formatting rules for `locale` that the `<ListFormat object>` was initialized with. |
//!
//! The options are as follows (details omitted here, refer to [MDN][lfquick]):
//!
//! |  | Values |
//! |--|--|
//! | `locale` | [BCP 47][bcp47] language tag for the locale actually used. |
//! | `style` | One of `long`, `short`, `narrow` |
//! | `type` | One of `conjunction`, `disjunction` |
//!
//!   [bcp47]: https://tools.ietf.org/html/bcp47
//!
//! # `Intl.ListFormat` API proposal in Rust
//!
//! This section presents a condensed view of the implementation for `ListFormat` in Rust.  Please
//! see the section [Considerations](#considerations) for the design rationale.
//!
//! Implementation fragments, consult each link for specific on each of the items below:
//! - [Locale] trait (better name pending).
//! - [listformat] mod.
//! - [listformat::options] mod.
//! - [listformat::Format] trait.
//!
//! # Considerations
//!
//! ## Names, names
//!
//! Naming is always fun. This proposal is presented as a crate `ecma402_listformat`, but is
//! intended to be re-exported from a top-level crate with a name such as `ecma402::listformat`.
//!
//! This gives the following sample options uses, which can be lenthened to avoid ambiguity.
//!
//! ```rust ignore
//! use ecma402::listformat::options::In;
//! use ecma402::listformat::options::Style;
//! // ... etc
//! ```
//!
//! ## Options handling
//!
//! The configuration options may be the easy bits.  The style and type are probably simple enough
//! to define in place, instead of delegating that to the implementor.  We define them in a new
//! mod, to keep the `Style` and `Type` names free for reuse for other parts of the API surface.
//! Not all options can be packaged this way, some require an open ended set of possible elements
//! (of which some may be well-formed but invalid, for example).
//!
//! `Style` and `Type` are concrete types, since those are fixed by the API.
//!
//! ```rust
//! /// Deliberately placed in a separate mod, to make for ergonomic naming. Short
//! /// names are a personal preference: `options::Style` reads better to me than
//! /// `OptionsStyle` or such.  It does not seem in strife with ECMA-402, since
//! /// it says nothing about types at all.
//! mod options {
//!     pub enum Style {
//!         Long, Short, Narrow,
//!     }
//!     pub enum Type {
//!         Conjunction, Disjunction,
//!     }
//!     /// These are the options that go in.
//!     /// Use as `options::In`.  I happen to like short names on types that
//!     /// have the same name as the mod they are in, to reduce repetition.
//!     pub struct In {
//!         pub style: Style,
//!         /// `type` is a reserved word.  Better suggestions welcome.
//!         pub in_type: Type,
//!     }
//! }
//! ```
//!
//! You would refer to them by `listformat::options::Style::Long` and the like, which gives us
//! non-repetitive and reusable names.  `use` clauses can be used to shorten if needed:
//!
//! ```rust ignore
//! use listformat::options::Style;
//! // ...
//! ```
//!
//! ## Locale
//!
//! Passing in a locale specifier is the first nontrivial design decision.  This is since locales
//! may be implemented in a number of different ways:
//!
//! 1. Is the locale data owned or borrowed?
//! 1. Are locales fallible or infallible? I.e. are they *well-formed* or always *valid*.
//! 1. Do locales come from a system repository or are they somehow provided by the end user?
//!
//! In Rust, each of these concerns seems to ask for different interfaces, making general
//! interfaces hard to formulate.  I welcome a correction here if I'm wrong.  The objective is to
//! unify as many as possible of the above use cases in a trait that can express all of them.
//!
//! ### "Stringly typed" API
//!
//! One possibility is to require a string-like type (`AsRef<str>`).
//!
//! Pros:
//!
//! * This is the closest to the JavaScript API surface
//! * It is the easiest one to implement.
//!
//! Cons:
//!
//! * It has the loosest guarantees as to the functionality it provides.
//! * It implies convertibility to a string-like object, which may require allocation.
//!
//!   This approach is inferior compared to the Formatting API because it forces
//!   the user's hand in string use.  A better alternative is below.
//!
//! ### Formatting API
//!
//! The formatting API is just the following:
//!
//! ```rust
//! use std::fmt;
//! use std::hash;
//! pub trait Locale: fmt::Display + hash::Hash {
//!   // We may extend this with more functionality.
//! };
//! ```
//!
//! Pros:
//!
//! * Simplicity
//! * Defers the decision on how to format the `Locale` and delegates it to
//!   the user.
//!
//! Cons:
//!
//! * Assumes that on-screen display of a `Locale` is the same as string
//!   serialization of the locale.
//!
//!   We believe that this conflation is not an issue today, as the same effective
//!   approach is already being used in Gecko.
//!
//! ## Error handling
//!
//! It is conceivable that a method call could return an error.  As error reporting is fairly cheap
//! in Rust, returning `Result<_, E>` for `E` being some error type, should be expected and
//! natural.  The to make this useful, `E` should probably be constrained to a standard "error
//! reporting" type such as `std::error::Error`.
//!
//! This suggests a general error handling approach:
//!
//! ```rust
//! use std::error;
//! pub trait Trait  {
//!     type Error: error::Error;
//!     fn function() -> Result<(), Self::Error>;
//! }
//! ```
//!
//! Pros:
//!
//! - A standardized error reporting type.
//! - Allows the use of crates such as [`anyhow`](https://crates.io/crates/anyhow).
//!
//! Cons:
//!
//! - Requires `Trait::Error` to implement a potentially hefty trait `std::error::Error`.
//! - There can be only one implementation of `Trait` in a library.  In this case it seems
//!   that may be enough.
//!
//! ### Fallibility
//!
//! The issue of fallibility in the API comes up because an implementor can decide to implement
//! lazy checking of the constructed collaborator types.  A `LocaleIdentifier` type is one of
//! those types.  While [`Locale`][lid] validates its inputs, [`rust_icu_uloc::ULoc`][uloc] does
//! not: in fact, almost all of its operations are fallible, and there does not seem to be a way to
//! validate `ULoc` eagerly, the way the underlying ICU4C API is defined.
//!
//!   [lid]: https://github.com/unicode-org/icu4x/pull/47
//!   [uloc]: https://docs.rs/rust_icu_uloc/0.2.2/rust_icu_uloc/struct.ULoc.html
//!
//! Now since `Intl.supportedLocalesOf()` exists, could say that that any user will have
//! the chance to obtain a valid locale either by taking one from the list of supported locales,
//! or by language-matching desired locale with the list of supported locales ahead of time.
//!
//! This means that an infallible API could work for the case of `Intl.ListFormat`.  However,
//! we do not want to rely on a locale resolution protocol imposed by the end user.  Furthermore,
//! not all combination of input options will be valid across all constructors of `Intl.*` types.
//! With this in mind
//!
//! > Note: Previous versions of this proposal had an infallible constructor `new`.  This has been
//! determined to be infeasible, and a fallible constructor `try_new` has been put in place
//! instead.
//!
//! ```rust ignore
//! let loc = rust_icu_uloc::ULoc::try_from("nl-NL").expect("locale accepted");
//! let formatter = ecma402::listformat::Format::try_new(
//!     &loc, ecma402::listformat::options::In{ /* ... */ }).expect("formatter constructed");
//! ```
//!
//! ## Sequences as input parameters
//!
//! This section concerns input parameters that are sequences.  The general approach is to
//! defer the forming of the sequence and use `IntoIterator` to pass the values in like so:
//!
//! ```rust
//! // This is the trait that our objects will implement.
//! pub trait Trait {}
//!
//! pub trait Api {
//!     fn function<'a, T>(input: impl IntoIterator<Item=&'a T>)
//!         where T: Trait + 'a;
//! }
//! ```
//!
//! This approach does not work for output parameters returned in a trait, however.  An approach
//! for that is given below.
//!
//! ## Iteration over output parameters
//!
//! Next up, let's take a look at how output iteration (iterators as return types) may be
//! traitified.
//!
//! We are exploring this
//! because APIs that require iteration may naturally come out of data items that contain
//! sequences; such as the [variant subtags][vartags].  And since generic iteration may be
//! of more general interest, we explore it in a broader context.
//!
//!   [vartags]: http://unicode.org/reports/tr35/#unicode_variant_subtag
//!
//! Generic iteration seems somewhat complicated to express in a Rust trait.
//! [unic_langid::LanguageIdentifier`][ulangid], for example, has the following method:
//!
//!   [ulangid]: https://docs.rs/unic-langid/0.9.0/unic_langid/struct.LanguageIdentifier.html#method.variants
//!
//! ```rust ignore
//! pub fn variants(&self) -> impl ExactSizeIterator
//! ```
//!
//! This expresses quite a natural need to iterate over the specified variants of a locale.
//! We would very much like to traitify this function so that different implementors could
//! contribute their own.  The straightforward approach won't fly:
//!
//! ```rust
//! // This will not compile:
//! //     "`impl Trait` not alowed outside of function and inherent method return types"
//! use core::iter::ExactSizeIterator;
//! pub trait Trait {
//!     fn variants() -> impl ExactSizeIterator;
//! }
//! ```
//!
//! A second attempt fails too:
//!
//! ```rust
//! // This will not compile:
//! //   error[E0191]: the value of the associated type `Item`
//! //   (from trait `std::iter::Iterator`) must be specified
//! use core::iter::ExactSizeIterator;
//! pub trait Trait {
//!     fn variants() -> dyn ExactSizeIterator;
//! }
//! ```
//!
//! Of course this is all invalid Rust, but it was a naive attempt to express a seemingly natural
//! idea of "I'd like this trait to return me an iterator over some string-like objects".
//!
//! Here's more exhibits for the gallery of failed approaches:
//!
//! ```rust
//! // This will not compile:
//! //   error[E0191]: the value of the associated type `Item`
//! //   (from trait `std::iter::Iterator`) must be specified
//! use core::iter::ExactSizeIterator;
//! pub trait Trait {
//!     fn variants() -> Box<dyn ExactSizeIterator>;
//! }
//! ```
//!
//! This has a couple of problems:
//!
//! 1. E0191, requiring a concrete associated type as an iterator `Item`, which we don't have.
//! 2. If we were to bind `Item` to a concrete type, we would have made that type obligatory
//!     for all the implementors.
//! 3. [Box] requires an allocation, which is not useful for `#![no_std]` (without alloc).
//!
//! Even if we disregard (3), then (2) will get us.  The following snippet compiles, but
//! forever fixes the associated iteration type to `String`.  Any implementors that don't
//! implement variants as strings are out of luck.
//!
//! ```rust
//! use core::iter::ExactSizeIterator;
//! pub trait Trait {
//!     // Oops... associated type is fixed now.
//!     fn variants() -> Box<dyn ExactSizeIterator<Item=String>>;
//! }
//! ```
//!
//! Trying to be clever with genericizing the type also gets us nowhere.  This works, but
//! requires a `Box`, which in turn requires a change to the type signature of
//! `LanguageIdentifier::variants()`.
//!
//! ```rust
//! use core::iter::ExactSizeIterator;
//! pub trait Trait {
//!     type IterItem;
//!     fn variants() -> Box<dyn ExactSizeIterator<Item=Self::IterItem>>;
//! }
//! ```
//!
//! Getting to a generic iterator that doesn't require a box is a piece of gymnastics.  Getting
//! to an iterable of owned strings requires the iterating trait to be implemented over
//! *a lifetime-scoped reference* of the implementing type.
//!
//! ```rust
//! pub trait Variants {
//!     /// The type of the item yieled by the iterator returned by [Variants::variants].  Note
//!     /// that [Variants::Variant] may be a reference to the type stored in the iterator.
//!     type Variant;
//!     /// The type of the iterator returned by [Variants::variants].
//!     type Iter: ExactSizeIterator<Item = Self::Variant>;
//!     fn variants(self) -> Self::Iter;
//! }
//!
//! // Here's how to implement the trait when the underlying type is borrowed.
//!
//! pub struct BorrowedVariant {
//!     variants: Vec<&'static str>,
//! }
//!
//! impl<'a> Variants for &'a BorrowedVariant {
//!     type Variant = &'a &'a str;
//!     type Iter = std::slice::Iter<'a, &'a str>;
//!     fn variants(self) -> Self::Iter {
//!       self.variants.iter()
//!     }
//! }
//!
//! let borrowed = BorrowedVariant{ variants: vec!["a", "b"], };
//! assert_eq!(
//!     vec!["a", "b"],
//!     borrowed.variants()
//!         .map(|v| v.to_owned().to_owned())
//!         .collect::<Vec<String>>(),
//! );
//!
//! // Here is how to implement the trait when the underlying type is owned.
//!
//! pub struct OwnedVariant {
//!     variants: Vec<String>,
//! }
//!
//! impl<'a> Variants for &'a OwnedVariant {
//!     type Variant = &'a String;
//!     type Iter = std::slice::Iter<'a, String>;
//!     fn variants(self) -> Self::Iter {
//!         self.variants.iter()
//!     }
//! }
//!
//! let owned = OwnedVariant{ variants: vec!["a".to_string(), "b".to_string()], };
//! assert_eq!(
//!     vec!["a", "b"],
//!     owned.variants()
//!         .map(|v| v.to_owned())
//!         .collect::<Vec<String>>(),
//! );
//! ```

use std::fmt;

/// This trait contains the common features of the Locale object that must be shared among
/// all the implementations.  Every implementor of `listformat` should provide their
/// own version of [Locale], and should ensure that it implements [Locale]. as
/// specified here.
///
/// For the time being we agreed that a [Locale] *must* be convertible into its string
/// form, using `Display`.
pub trait Locale: fmt::Display {}

/// The [listformat] mod contains all the needed implementation bits for `Intl.ListFormat`.
///
/// > Note: This is not yet the entire API.  I'd like to get to a consensus on what has been
/// defined, then use the patterns adopted here for the rest.
pub mod listformat {
    /// Contains the API configuration as prescribed by ECMA 402.
    ///
    /// The meaning of the options is the same as in the similarly named
    /// options in the JS version.
    ///
    /// See [Options] for the contents of the options.  See the [Format::try_new]
    /// for the use of the options.
    pub mod options {
        /// Chooses the list formatting approach.
        #[derive(Eq, PartialEq, Debug, Clone)]
        pub enum Style {
            Long,
            Short,
            Narrow,
        }
        /// Chooses between "this, that and other", and "this, that or other".
        #[derive(Eq, PartialEq, Debug, Clone)]
        pub enum Type {
            /// "This, that and other".
            Conjunction,
            /// "This, that or other".
            Disjunction,
        }
    }

    /// The options set by the user at construction time.  See discussion at the top level
    /// about the name choice.  Provides as a "bag of options" since we don't expect any
    /// implementations to be attached to this struct.
    ///
    /// The default values of all the options are prescribed in by the [TC39 report][tc39lf].
    ///
    ///   [tc39lf]: https://tc39.es/proposal-intl-list-format/#sec-Intl.ListFormat
    #[derive(Eq, PartialEq, Debug, Clone)]
    pub struct Options {
        /// Selects a [options::Style] for the formatted list.  If unset, defaults
        /// to [options::Style::Long].
        pub style: options::Style,
        /// Selects a [options::Type] for the formatted list.  If unset, defaults to
        /// [options::Type::Conjunction].
        pub in_type: options::Type,
    }

    /// Allows the use of `listformat::Format::try_new(..., Default::default())`.
    impl Default for Options {
        /// Gets the default values of [Options] if omitted at setup.  The
        /// default values are prescribed in by the [TC39 report][tc39lf].
        ///
        ///   [tc39lf]: https://tc39.es/proposal-intl-list-format/#sec-Intl.ListFormat
        fn default() -> Self {
            Options {
                style: options::Style::Long,
                in_type: options::Type::Conjunction,
            }
        }
    }

    use std::fmt;

    /// The package workhorse: formats supplied pieces of text into an ergonomically formatted
    /// list.
    ///
    /// While ECMA 402 originally has functions under `Intl`, we probably want to
    /// obtain a separate factory from each implementor.
    ///
    /// Purposely omitted:
    ///
    /// - `supported_locales_of`.
    pub trait Format {
        /// The type of error reported, if any.
        type Error: std::error::Error;

        /// Creates a new [Format].
        ///
        /// Creation may fail, for example, if the locale-specific data is not loaded, or if
        /// the supplied options are inconsistent.
        fn try_new(l: impl crate::Locale, opts: Options) -> Result<Self, Self::Error>
        where
            Self: std::marker::Sized;

        /// Formats `list` into the supplied standard `writer` [fmt::Write].
        ///
        /// The original [ECMA 402 function][ecma402fmt] returns a string.  This is likely the only
        /// reasonably generic option in JavaScript so it is adequate.  In Rust, however, it is
        /// possible to pass in a standard formatting strategy (through `writer`).
        ///
        ///   [ecma402fmt]:
        ///   https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat/format
        ///
        /// This makes it unnecessary for [Format] to implement its own, and can
        /// completely avoid constructing any intermediary representation.  This, in turn,
        /// allows the user to provide a purpose built formatter, or a custom one if needed.
        ///
        /// A purpose built formatter could be one that formats into a fixed-size buffer; or
        /// another that knows how to format strings into a DOM.  If ECMA 402 compatibility is
        /// needed, the user can force formatting into a string by passing the appropriate
        /// formatter.
        ///
        /// > Note:
        /// > - Should there be a convenience method that prints to string specifically?
        /// > - Do we need `format_into_parts`?
        fn format<I, L, W>(self, list: L, writer: &mut W) -> fmt::Result
        where
            I: fmt::Display,
            L: IntoIterator<Item = I>,
            W: fmt::Write;
    }
}
