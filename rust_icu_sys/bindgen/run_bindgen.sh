#! /bin/bash
# Copyright 2019 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# A script that updates the bindgen library manually.
#
# Please refer to the file README.md
#
# Requirements:
# - bash
# - awk
# - tr
# - llvm-dev package (for llvm-config that bindgen needs)

set -eo pipefail

# The directory into which the rust bindings file will be written.  If left
# unset, the current directory is the default.
OUTPUT_DIR=${OUTPUT_DIR:-.}

# The list of unicode source headers to generate bindings for. This list is
# intended to be kept in sync with the static variable by the same name in the
# build.rs file.
readonly BINDGEN_SOURCE_MODULES=(
        "ubrk"
        "ucal"
        "uclean"
        "ucnv"
        "ucol"
        "udat"
        "udatpg"
        "udata"
        "uenum"
        "ufieldpositer"
        "uformattable"
        "ulistformatter"
        "umisc"
        "umsg"
        "unum"
        "unumberformatter"
        "upluralrules"
        "uset"
        "ustring"
        "utext"
        "utrans"
        "unorm2"
        "ucptrie"
        "umutablecptrie"
)

# Types for which to generate the bindings.  Expand this list if you need more.
# The syntax is regex.  This list is intended to be kept in sync with the static
# variable by the same name in the build.rs file.
readonly BINDGEN_ALLOWLIST_TYPES=(
        "UAcceptResult"
        "UBool"
        "UBreakIterator"
        "UBreakIteratorType"
        "UCalendar.*"
        "UChar.*"
        "UCol.*"
        "UCollation.*"
        "UCollator"
        "UConverter.*"
        "UData.*"
        "UDate.*"
        "UDateTime.*"
        "UDateFormat.*"
        "UDisplayContext.*"
        "UEnumeration.*"
        "UErrorCode"
        "UField.*"
        "UFormat.*"
        "UFormattedList.*"
        "ULOC.*"
        "ULineBreakTag"
        "UListFormatter.*"
        "ULoc.*"
        "UMessageFormat"
        "UNUM.*"
        "UNumber.*",
        "UParseError"
        "UPlural.*"
        "USentenceBreakTag"
        "USet"
        "UText"
        "UTransDirection"
        "UTransPosition"
        "UTransliterator"
        "UWordBreak"
        "UNorm.*"
        "UCPTrie.*"
        "UCPTrieType"
        "UCPTRIE.*"
        "UPRV.*"
)

# Functions for which to generate the bindings.  Expand this list if you need
# more.  This list is intended to be kept in sync with the static variable by
# the same name in the build.rs file.
readonly BINDGEN_ALLOWLIST_FUNCTIONS=(
        "u_.*"
        "ubrk_.*"
        "ucal_.*"
        "ucnv_.*"
        "ucol_.*"
        "udat_.*"
        "udatpg_.*"
        "udata_.*"
        "uenum_.*"
        "ufieldpositer_.*"
        "ufmt_.*"
        "ulistfmt_.*"
        "uloc_.*"
        "umsg_.*"
        "unum_.*"
        "unumf_.*"
        "uplrules_.*"
        "utext_.*"
        "utrans_.*"
        "unorm2_.*"
        "usrc_.*"
        "umutablecp.*"
        "ucp.*"
)

function check_requirements() {
  icu-config --version &> /dev/null || \
    (echo "The generator requires icu-config to be in PATH; see README.md"; exit 1)

  bindgen --version &> /dev/null || \
    (echo "The generator requires bindgen to be in PATH; see README.md"; exit 1)

  awk --version &> /dev/null || \
    (echo "The generator requires awk to be installed; see README.md"; exit 1)

  llvm-config --version &> /dev/null || \
    (echo "The generator requires llvm-config (package llvm-dev) to be installed; see README.md"; exit 1)
}

# Generates a temporary header file to be supplied to bindgen for binding generation.
# The file is automatically removed upon exit, so if you are debugging you may want
# to remove that part.
function generate_header_file() {
  MAIN_HEADER_FILE="$(mktemp --suffix=.h)"
  echo "MAIN_HEADER_FILE=${MAIN_HEADER_FILE}"
  trap "rm ${MAIN_HEADER_FILE}" EXIT

  readonly ICU_INCLUDE_DIR="$(icu-config --prefix)/include/unicode"

  echo "// Automatically generated by run_bindgen.sh, DO NOT EDIT. " > ${MAIN_HEADER_FILE}
  for module in ${BINDGEN_SOURCE_MODULES[@]}; do
    echo "#include \"${ICU_INCLUDE_DIR}/${module}.h\"" >> "${MAIN_HEADER_FILE}"
  done
}

function main() {
  check_requirements

  generate_header_file

  # Joins all with a |, so ("a" "b") become "a|b"
  local _allowlist_types_concat="$(\
    echo ${BINDGEN_ALLOWLIST_TYPES[@]} | tr ' ' '|')"
  local _functions_concat="$(\
    echo ${BINDGEN_ALLOWLIST_FUNCTIONS[@]} | tr ' ' '|')"

  set -x 

  # Example: "67.1", "66.0.1"
  local _icu_version="$(icu-config --version)"
  # Example: "67"
  local _icu_version_major="${_icu_version%.*}"
  local _icu_version_major="${_icu_version_major%.*}"

  # Respectful code hack.
  local _allowlist="$(echo "d2hpdGVsaXN0Cg==" | base64 -d -)"


  local _output_file="${OUTPUT_DIR}/lib_${_icu_version_major}.rs"
  bindgen \
    --default-enum-style=rust \
    --no-doc-comments \
    --with-derive-default \
    --with-derive-hash \
    --with-derive-partialord \
    --with-derive-partialeq \
    --"${_allowlist}"-type="${_allowlist_types_concat}" \
    --"${_allowlist}"-function="${_functions_concat}" \
    --opaque-type="" \
    --output="${_output_file}" \
    "${MAIN_HEADER_FILE}" \
    -- \
    $(icu-config --cppflags)

  # This can fail for weird reasons, ignore.
  rustfmt --force "${_output_file}" || true
}

main
