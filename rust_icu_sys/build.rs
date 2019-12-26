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

// See LICENSE for licensing information.
//
// This build.rs script tries to generate low-level rust bindings for the current ICU library.
// Please refer to README.md for instructions on how to build the library for
// your use.

use {std::env, std::fs::File, std::io::Write, std::path::Path, std::process::Command};

// Captures the stdout of the command or panics.
fn stdout(c: &mut Command) -> String {
    let result = c
        .output()
        .unwrap_or_else(|e| {
            panic!("Error while executing command [{:?}]:\n{:?}", c, e);
        });
    let result = String::from_utf8(result.stdout).expect("could not parse as utf8");
    result.trim().to_string()
}

fn icu_config_cmd() -> Command {
    // TODO: looks like .args() will persist on a command for some reason.
    Command::new("icu-config")
}

fn run(cmd: &mut Command, args: &[&str]) -> String {
    String::from(&stdout(cmd.args(args))).trim().to_string()
}

fn icu_config_prefix() -> String {
    run(&mut icu_config_cmd(), &["--prefix"])
}

fn lib_dir() -> String {
    run(&mut icu_config_cmd(), &["--libdir"])
}

fn ld_flags() -> String {
    // Replacements needed because of https://github.com/rust-lang/cargo/issues/7217
    run(&mut icu_config_cmd(), &["--ldflags"])
        .replace("-L", "-L ")
        .replace("-l", "-l ")
}

/// Returns the C preprocessor flags used to build ICU
fn icu_cpp_flags() -> String {
    run(&mut icu_config_cmd(), &["--cppflags"])
}

/// Returns true if the ICU library was compiled with renaming enabled.
fn has_renaming() -> bool {
    let cpp_flags = icu_cpp_flags();
    let found = cpp_flags.find("-DU_DISABLE_RENAMING=1");
    println!("flags: {}", cpp_flags);
    found.is_none()
}

fn icu_config_version() -> String {
    run(&mut icu_config_cmd(), &["--version"])
}

fn install_dir() -> String {
    run(&mut icu_config_cmd(), &["--prefix"])
}

fn rustfmt_cmd() -> Command {
    Command::new("rustfmt")
}

fn rustfmt_version() -> String {
    run(&mut rustfmt_cmd(), &["--version"])
}

fn bindgen_cmd() -> Command {
    Command::new("bindgen")
}

fn bindgen_version() -> String {
    run(&mut bindgen_cmd(), &["--version"])
}

// Returns the config major number.  For example, will return "64" for
// version "64.2"
fn icu_config_major_number() -> String {
    let version = icu_config_version();
    let components = version.split(".");
    components.take(1).last().unwrap_or("").to_string()
}


/// Generates an additional include file which contains the linker directives.
/// This is done because cargo does not allow the rustc link directives to be
/// anything other than `-L` and `-l`.
fn generate_linker_file(out_dir_path: &Path, lib_dir: &str, lib_names: &Vec<&str>) {
    let file_path = out_dir_path.join("link.rs");
    let mut linker_file = File::create(&file_path).unwrap();
    let mut content: Vec<String> = vec![];
    for lib in lib_names {
        let linkopt: String = format!(
            r#"#[link_args="-Wl,-rpath={}/lib{}.so"]"#, lib_dir, lib);
        content.push(linkopt);
    }
    content.push(String::from(r#"extern "C" {}"#));
    linker_file.write_all(&content.join("\n").into_bytes())
        .expect("successful write into linker file");
}

/// Generates a wrapper header that includes all headers of interest for binding.
///
/// This is the recommended way to bind complex libraries at the moment.  Returns
/// the full path of the generated wrapper header file.
fn generate_wrapper_header(
    out_dir_path: &Path, bindgen_source_modules: &Vec<&str>, include_path: &Path) -> String {

    let wrapper_path = out_dir_path.join("wrapper.h");
    let mut wrapper_file = File::create(&wrapper_path).unwrap();
    wrapper_file
        .write_all(b"/* Generated file, do not edit. */ \n")
        .unwrap();
    let includes = bindgen_source_modules
        .iter()
        .map(|f| {
            let file_path = include_path.join(format!("{}.h", f));
            let file_path_str = format!("#include \"{}\"\n", file_path.to_str().unwrap());
            println!("include-file: '{}'", file_path.to_str().unwrap());
            file_path_str
        })
        .collect::<String>();
    wrapper_file.write_all(&includes.into_bytes()).unwrap();
    String::from(wrapper_path.to_str().unwrap())
}

fn commaify(s: &Vec<&str>) -> String {
    format!("{}", s.join("|"))
}

fn run_bindgen(header_file: &str, out_dir_path: &Path) {
    let whitelist_types_regexes = commaify(&vec![
        "UBool",
        "UCalendar.*",
        "UChar.*",
        "UData.*",
        "UDate",
        "UDateFormat.*",
        "UEnumeration.*",
        "UErrorCode",
        "UText.*",
    ]);

    let whitelist_functions_regexes = commaify(&vec![
        "u_.*", "ucal_.*", "udata_*", "udat_.*", "uenum_.*", "uloc_.*", "utext_.*",
    ]);

    let opaque_types_regexes = commaify(&vec![]);

    // Common arguments for all bindgen invocations.
    let bindgen_generate_args = vec![
        "--default-enum-style=rust",
        "--no-doc-comments",
        "--with-derive-default",
        "--with-derive-hash",
        "--with-derive-partialord",
        "--with-derive-partialeq",
        "--whitelist-type",
        &whitelist_types_regexes,
        "--whitelist-function",
        &whitelist_functions_regexes,
        "--opaque-type",
        &opaque_types_regexes,
    ];

    let output_file_path = out_dir_path.join("lib.rs");
    let output_file = output_file_path.to_str().unwrap();
    let ld_flags = ld_flags();
    let cpp_flags = icu_cpp_flags();
    let mut file_args = vec![
        "-o",
        &output_file,
        &header_file,
        "--",
        &ld_flags,
        &cpp_flags,
    ];
    if !has_renaming() {
        file_args.push("-DU_DISABLE_RENAMING=1");
    }
    let all_args = [&bindgen_generate_args[..], &file_args[..]].concat();
    println!("bindgen-cmdline: {:?}", all_args);
    Command::new("bindgen")
        .args(&all_args)
        .spawn()
        .expect("bindgen finished with success");
}

// Generates the library renaming macro: this allows us to use renamed function
// names in the resulting low-level bindings library.
fn run_renamegen(out_dir_path: &Path) {
    let output_file_path = out_dir_path.join("macros.rs");
    let mut macro_file = File::create(&output_file_path).expect("file opened");
    if has_renaming() {
        println!("renaming: true");
        // The library names have been renamed, need to generate a macro that
        // converts the call of `foo()` into `foo_64()`.
        let icu_major_version = icu_config_major_number();
        let to_write = format!(
            r#"
// Macros for changing function names.

extern crate paste;

// This library was build with version renaming, so rewrite every function name
// with its name with version number appended.

// The macro below will rename a symbol `foo::bar` to `foo::bar_64` (where "64")
// may be some other number depending on the ICU library in use.
#[macro_export]
macro_rules! versioned_function {{
    ($i:ident) => {{
      paste::expr! {{
        [< $i _{0} >]
      }}
    }}
}}
"#,
            icu_major_version
        );
        macro_file
            .write_all(&to_write.into_bytes())
            .expect("successful write");
    } else {
        // The library names have not been renamed, generating an empty macro
        println!("renaming: false");
        macro_file
            .write_all(
                &r#"
// Macros for changing function names.

// There was no renaming in this one, so just short-circuit this macro.
#[macro_export]
macro_rules! versioned_function {
    ($func_name:path) => {
        $func_name
    }
}
"#
                .to_string()
                .into_bytes(),
            )
            .expect("wrote macro file without version");
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    println!("rustfmt: {}", rustfmt_version());
    println!("icu-config: {}", icu_config_version());
    println!("icu-config-cpp-flags: {}", icu_cpp_flags());
    println!("icu-config-has-renaming: {}", has_renaming());
    println!("bindgen: {}", bindgen_version());

    // The path to the directory where cargo will add the output artifacts.
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir_path = Path::new(&out_dir);

    // The path where all unicode headers can be found.
    let include_dir_path = Path::new(&icu_config_prefix())
        .join("include")
        .join("unicode");

    generate_linker_file(out_dir_path, &lib_dir(), &vec!["icui18n", "icuuc", "icudata"]);
    // The modules for which bindings will be generated.  Add more if you need
    // them.  The list should be topologicaly sorted based on the inclusion
    // relationship between the respective headers.
    // Any of these will fail if the required binaries are not present in $PATH.
    let bindgen_source_modules: Vec<&str> = vec!["ucal", "udat", "udata", "uenum", "ustring", "utext", "uclean"];
    let header_file = generate_wrapper_header(
        &out_dir_path, &bindgen_source_modules, &include_dir_path);
    run_bindgen(&header_file, out_dir_path);
    run_renamegen(out_dir_path);

    println!("cargo:install-dir={}", install_dir());
    println!("cargo:rustc-link-search=native={}", lib_dir());
    println!("cargo:rustc-flags={}", ld_flags());
    println!("done:true");
}
