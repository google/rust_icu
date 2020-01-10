#!/bin/bash
set -euo pipefail

# Generates an ICU feature coverage support report.

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
TOP_DIR="${TOP_DIR:-${DIR}/..}"
cd $TOP_DIR

C_API_HEADER_NAMES=(
  "ustring"
  "ucal"
  "udat"
  "udata"
  "uenum"
  "uloc"
  "ustring"
  "utext"
)

ICU_INCLUDE_PATH="$(icu-config --cppflags-searchpath | sed -e 's/-I//' | sed -e 's/ //g')"

for header_basename in ${C_API_HEADER_NAMES[@]}; do
  header_fullname="${ICU_INCLUDE_PATH}/unicode/${header_basename}.h"
  echo $header_basename: $header_fullname
  ctags -x --c-kinds=fp $header_fullname | sed -e 's/\s.*$//' \
    | grep -v U_DEFINE | sort | uniq \
    > "${TOP_DIR}/coverage/${header_basename}_all.txt"

  # Extracts all mentions of functions such as "utext_close" for example from
  # rust docs of the form "/// Implements `utext_close` ... ".  This is
  # simplistic but quite enough with a little bit of care.
  find . -path "*rust_icu_${header_basename}/src/*.rs" | \
    xargs grep "/// Implements \`" | sed -e 's/.*`\(.*\)`.*$/\1/' | \
    sort | uniq > "${TOP_DIR}/coverage/${header_basename}_implemented.txt"
done

# Now, write a report out. Again, simplistic, but gets the job done.

REPORT_FILE="${TOP_DIR}/coverage/report.md"
cat <<EOF > "${REPORT_FILE}"
# Implementation coverage report

| Header | Implemented |
| ------ | ----------- |
EOF
for header_basename in ${C_API_HEADER_NAMES[@]}; do
  total_functions="$(cat "${TOP_DIR}"/coverage/${header_basename}_all.txt | wc -l)"
  implemented_functions="$(cat "${TOP_DIR}"/coverage/${header_basename}_implemented.txt | wc -l)"
  echo "| \`${header_basename}.h\` | ${implemented_functions} / ${total_functions} | " >> "${REPORT_FILE}"
done

cat <<EOF >>"${REPORT_FILE}"
# Unimplemented functions per header

EOF
for header_basename in ${C_API_HEADER_NAMES[@]}; do
  cat <<EOF >>"${REPORT_FILE}"

# Header: \`${header_basename}.h\`

| Unimplemented | Implemented |
| ------------- | ----------- |
EOF
  for fun in $(cat "${TOP_DIR}/coverage/${header_basename}_implemented.txt"); do
    echo "| | \`${fun}\` |" >>"${REPORT_FILE}"
  done

  unimplemented="$(comm -23 \
    "${TOP_DIR}/coverage/${header_basename}_all.txt" \
    "${TOP_DIR}/coverage/${header_basename}_implemented.txt" | sort | uniq)"
  for fun in ${unimplemented}; do
    echo "| \`${fun}\` | |" >>"${REPORT_FILE}"
  done
done

