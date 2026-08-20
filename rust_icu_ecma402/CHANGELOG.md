# Changelog

## [5.8.0](https://github.com/google/rust_icu/compare/5.7.0...5.8.0) (2026-08-20)


### Features

* corrects the public API for udata_open ([aba0cf6](https://github.com/google/rust_icu/commit/aba0cf6fc7558e641f1881b2be046ed924e5e8ab))
* release version 5.1.0 ([43ddd1f](https://github.com/google/rust_icu/commit/43ddd1f27305738b5d96529553f6a3d1c62a5da2))
* release version 5.1.0 ([416eb08](https://github.com/google/rust_icu/commit/416eb081865ab1af99e1b2d8f7c3fc5d9726b761))
* release version 5.2.0 ([571daac](https://github.com/google/rust_icu/commit/571daac87d47312db6a805ca4b1dd2dd562ad11c))
* Release version 5.3.0 ([b44c2c3](https://github.com/google/rust_icu/commit/b44c2c393e73d26c8c1744c42479110fbd20fddf))
* Release version 5.4.0 ([0f1779e](https://github.com/google/rust_icu/commit/0f1779e02ef6e2d23af3004874e3b4c49b625e0b))
* Release version 5.5.0 ([a6968ed](https://github.com/google/rust_icu/commit/a6968edb86e633078c3af12722efd7917efa176b))
* Release version 5.6.0 ([afb22bd](https://github.com/google/rust_icu/commit/afb22bde67caad9d2a2e188d2e07a4a7ca589f4f))
* Release version 5.7.0 ([1004d77](https://github.com/google/rust_icu/commit/1004d7717022b2601859a9e495ddb65147fe66c3))
* support UDataMemory::open ([2c60d22](https://github.com/google/rust_icu/commit/2c60d2283508eba298497b9a1340fc11be1194e5))
* update rust_icu to support ICU v74 ([6ef3696](https://github.com/google/rust_icu/commit/6ef3696ff92bb4a4eeb6e269d843dd83db8b1449))


### Bug Fixes

* **bazel:** make the build work on macOS hosts ([d884c35](https://github.com/google/rust_icu/commit/d884c352b6b8c81eed6117b158f0232648b2dccf))
* run `cargo fmt` at the top of the tree ([ef68d33](https://github.com/google/rust_icu/commit/ef68d331f5411786a21e7013a7ffda1443274f7b))


### Performance Improvements

* replace expect(&format!(...)) with unwrap_or_else in tests ([279b577](https://github.com/google/rust_icu/commit/279b577db30ce994617fc1294ffb9428afcf3298))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * ecma402_traits bumped from 5.7.0 to 5.8.0
    * rust_icu_common bumped from 5.7.0 to 5.8.0
    * rust_icu_udat bumped from 5.7.0 to 5.8.0
    * rust_icu_sys bumped from 5.7.0 to 5.8.0
    * rust_icu_uloc bumped from 5.7.0 to 5.8.0
    * rust_icu_ustring bumped from 5.7.0 to 5.8.0
    * rust_icu_ulistformatter bumped from 5.7.0 to 5.8.0
    * rust_icu_upluralrules bumped from 5.7.0 to 5.8.0
    * rust_icu_unum bumped from 5.7.0 to 5.8.0
    * rust_icu_unumberformatter bumped from 5.7.0 to 5.8.0
