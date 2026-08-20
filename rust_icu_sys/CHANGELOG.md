# Changelog

## [5.8.0](https://github.com/google/rust_icu/compare/5.7.0...5.8.0) (2026-08-20)


### Features

* corrects the public API for udata_open ([aba0cf6](https://github.com/google/rust_icu/commit/aba0cf6fc7558e641f1881b2be046ed924e5e8ab))
* generate static bindgens with buildenv 1.91.0 for ICU 78 ([e3dce03](https://github.com/google/rust_icu/commit/e3dce030df92bdc8e237255cf7ded3eb79786922))
* implement rust icu ucsdet ([b65c234](https://github.com/google/rust_icu/commit/b65c23456bed4d6318495be37e5823f8a14e13bd))
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
* **ures:** add rust_icu_ures safe bindings for ICU ResourceBundle C API ([26d55f1](https://github.com/google/rust_icu/commit/26d55f181945261531685753a837394030a56aa1))


### Bug Fixes

* add bindgen for ICU 78 and (upcoming) 79 ([2b60984](https://github.com/google/rust_icu/commit/2b609845dc27108c54b3794748193a30d89dd3cc))
* add bindgen for ICU 78 and (upcoming) 79 ([9b57ac8](https://github.com/google/rust_icu/commit/9b57ac8201eb38e48a6408cfaf258a9b4fc66bb4))
* add ures.h to bindgen allowlists in build.rs and run_bindgen.sh ([2f49272](https://github.com/google/rust_icu/commit/2f49272de80b0cebeba8d62bdc522de011aa2397))
* block also __gnuc_va_list to build on mingw target ([13bddec](https://github.com/google/rust_icu/commit/13bddec398f5bdd661d9ebff463549cb96f951f6))
* blocklist va_list functions and types to eliminate platform-dependent bindgen output ([c4d1fd5](https://github.com/google/rust_icu/commit/c4d1fd5696359e1139e8009dedd774207ffb9f65))
* fix icu symbol name to link on Windows ([761bc38](https://github.com/google/rust_icu/commit/761bc38e485fd2877ed8922573883fe39758a489))
* handle some bitrot items ([4143d44](https://github.com/google/rust_icu/commit/4143d440a0156eafdbe9b2d890358bb387763b95))
* patch lib_78 and lib_79 with the correct version numbers ([9069cd9](https://github.com/google/rust_icu/commit/9069cd95df0d9eae492cf8fe1b7b8612c428172f))
* pin `anyhow` to `1.0.72` ([6b22ad6](https://github.com/google/rust_icu/commit/6b22ad627c569a923f9d5379c7f7f8810a0b1deb))
* re-enable macOS in CI with native bindgen and static linking ([48c5931](https://github.com/google/rust_icu/commit/48c59312e28d7ca575b17b7db611ebaf97667ea5))
* run `cargo fmt` at the top of the tree ([ef68d33](https://github.com/google/rust_icu/commit/ef68d331f5411786a21e7013a7ffda1443274f7b))
* **rust_icu_sys:** address compiler warnings in bindgen configuration ([5f18692](https://github.com/google/rust_icu/commit/5f186928e958c40e4756870a8192a9104b0b1e02))
* **rust_icu_sys:** reinstate deref_nullptr lint ([6751d95](https://github.com/google/rust_icu/commit/6751d957d2d536ea1a56684c98b3427e95fe1c1e))
* **sys:** add __va_list_tag to bindgen blocklist ([6d555c5](https://github.com/google/rust_icu/commit/6d555c5dea31b8523d5f072b0107fda3568b7c39))
* update bindgens ([30d9278](https://github.com/google/rust_icu/commit/30d927858cb4bf2ab3a4aa271116425d0cbf85ac))


### Dependencies

* The following workspace dependencies were updated
  * build-dependencies
    * rust_icu_release bumped from 5.7.0 to 5.8.0
