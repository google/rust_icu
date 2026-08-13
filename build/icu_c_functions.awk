# Copyright 2020 Google LLC
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
#
# Prints the name of every function declared in an ICU public C header, one
# per line, in the order the declarations appear.
#
# This replaces "ctags -x --c-kinds=fp", which needed exuberant-ctags: the
# --c-kinds flag is specific to that implementation, so the report could not
# be generated with the ctags shipped by many systems.  awk is in POSIX, so
# nothing extra has to be installed.
#
# The extraction relies on how ICU declares its C API rather than on parsing
# C.  Every public function is introduced by a line holding U_EXPORT2, as in
#
#     U_CAPI int32_t U_EXPORT2
#     umsg_format(const UMessageFormat *fmt, ...);
#
# with the name either starting the next line, as above, or trailing on the
# same line:
#
#     U_CAPI int32_t U_EXPORT2 uloc_countAvailable(void);
#
# Keying on U_EXPORT2 rather than on U_CAPI also covers the older spellings
# U_STABLE, U_DRAFT and U_DEPRECATED, which take the same shape.

BEGIN { expect = 0 }

# Continuation lines of a doc comment, e.g. " * @see ucol_clone".  These
# mention function names but declare nothing.
/^[ \t]*\*/ { next }

{
  line = $0

  # A function pointer typedef is spelled with U_CALLCONV, not U_EXPORT2, but
  # check for "typedef" anyway so a stray one is never taken for a prototype.
  if (line ~ /U_EXPORT2/ && line !~ /typedef/) {
    rest = line
    sub(/^.*U_EXPORT2[ \t]*/, "", rest)
    if (match(rest, /^[A-Za-z_][A-Za-z0-9_]*[ \t]*\(/)) {
      name = substr(rest, RSTART, RLENGTH)
      sub(/[ \t]*\($/, "", name)
      print name
      expect = 0
      next
    }
    # The name is on a following line.
    expect = 1
    next
  }

  # ICU also ships a few C++ convenience wrappers around the C API, declared
  # as "inline <Type>" with the name on the next line.  ctags reported these,
  # so keep reporting them.
  if (line ~ /^inline[ \t]/ && line !~ /typedef/) {
    expect = 1
    next
  }

  if (expect) {
    if (line ~ /^[ \t]*$/) { next }
    if (match(line, /^[A-Za-z_][A-Za-z0-9_]*[ \t]*\(/)) {
      name = substr(line, RSTART, RLENGTH)
      sub(/[ \t]*\($/, "", name)
      print name
    }
    expect = 0
  }
}
