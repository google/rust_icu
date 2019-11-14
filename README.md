# rust-icu: low-level rust language bindings for the ICU library

This is a library of low level native rust language bindings for the Unicode
ICU library for C (ICU4C).

See: http://icu-project.org for details about the ICU library.

NOTE: This is not an officially supported Google product.

# Structure of the repository

The repository is organized as a collection of rust crates.  Each crate
corresponds to the respective header in the ICU4C library's C API.  For
example, `rust_icu_uenum` implements the functionality that one would find in
the [uenum.h](http://www.icu-project.org/apiref/icu4c/uenum_8h.html) header
file.

At the moment, all implementations are very partial.  There is currently no
registry of the API coverage, which makes things a bit difficult to follow.

The goal is to correct this situation and have a comprehensive coverage matrix.

# Prerequisites

* ICU

  The ICU library for which the bindings are generated is version 64.2.

  You need to install the ICU library on your system, such that the binary
  `icu-config` is somewhere in your `$PATH`.  The build script will use it to
  discover the library settings and generate correct link scripts.

  Please refer to the installation instructions for the ICU library to make it
  available for binding in general.  See "ICU installation instructions"
  section way below in this document to see how to produce an example ICU
  build.

* rustfmt

  See https://github.com/rust-lang/rustfmt for instructions on how to install.

* bindgen

  [bindgen user
  guide](https://rust-lang.github.io/rust-bindgen/command-line-usage.html) for
  instructions on how to install it.

# Testing

The following tests should all build and pass.  Note that because the libraries
needed are in a custom location, we need to set `LD_LIBRARY_PATH` when running
the tests.

```bash
env LD_LIBRARY_PATH=$(icu-config --libdir) \
  cargo test
```

# Prior art

There is plenty of prior art that has been considered:

* https://github.com/servo/rust-icu
* https://github.com/open-i18n/unic
* https://github.com/fullcontact/icu-sys
* https://github.com/rust-locale
* https://github.com/unicode-rs

The current state of things is that I'd like to do a few experiments on my own
first, then see if the work can be folded into any of the above efforts.

See also:

* https://github.com/rust-lang/rfcs/issues/797
* https://unicode-rs.github.io

# Assumptions

There are a few competing approaches for ICU bindings.  However, it seems, at
least based on [information available in rust's RFC
repos](https://github.com/rust-lang/rfcs/issues/797), that the work on ICU
support in rust is still ongoing.

These are the assumptions made in the making of this library:

* We need a complete, reusable and painless ICU low-level library for rust.

  This, for example, means that we must rely on an external ICU library, and not
  lug the library itself with the binding code.  Such modularity allows the end
  user of the library to use an ICU library of their choice, and incorporate it
  in their respective systems.

* I will *not* reimplement ICU algorithms.

  An ICU reimplementation will likely take thousands of engineer years to
  complete.  For an API that is as subtle and complex as ICU, I think that it
  is probably a better return on investment to maintain a single central
  implementation.

  Also, the existence of this library doesn't prevent reimplementation. If
  someone else wants to try their hand at reimplementing ICU, that's fine too.

* This library should serve as a low-level basis for a rust implementation.

  It is obvious that the low level API is not an appropriate seam for the end
  users to interact with the ICU functionality.  A rust-ful API should be
  layered on top of these bindings.  Further,it will probably be a good idea to
  subdivide that functionality into crates, to match the expectations of rust
  developers.

  I'll gladly reuse the logical subdivision already made in some of the above
  mentioned projects.

* I'd like to explore ways to combine with existing implementations to build a
  complete ICU support for rust.

  Hopefully it will be possible to combine the good parts of all the rust
  bindings available today into a unified rust library. I am always available to
  discuss options.

  The only reason I started a separate effort instead of contributing to any of
  the projects listed in the "Prior Art" section is that I wanted to try what
  a generated library would look like in rust.

# ICU installation instructions

These instructions follow the "out-of-tree" build instructions from [the ICU
repository](https://github.com/unicode-org/icu/blob/master/icu4c/readme.html).
The instructions below are not self-contained: they assume that you have your
system set up such that you can follow the ICU build instructions.

These worked for my system.  You may be able to adapt them to work on yours.

```
mkdir -p $HOME/local
mkdir -p $HOME/tmp
cd $HOME/tmp
git clone export https://github.com/unicode-org/icu.git
mkdir icu4c-build
cd icu4c-build
../icu/icu4c/source/runConfigureICU Linux \
  --prefix=$HOME/local \
  --enable-static
make check
make install
make doc
```

If the compilation finishes with success, the directory `$HOME/local/bin` will
have the file `icu-config` which is necessary to discover the library
configuration.

