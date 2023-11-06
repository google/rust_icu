#!/bin/bash
set -euo pipefail

# Generates an ICU feature coverage support report.

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
TOP_DIR="${TOP_DIR:-${DIR}/..}"
cd $TOP_DIR

C_API_HEADER_NAMES=(
  "ubrk"
  "ucal"
  "ucol"
  "udat"
  "udata"
  "uenum"
  "uformattable"
  "ulistformatter"
  "uloc"
  "umsg"
  "unum"
  "unumberformatter"
  "upluralrules"
  "ustring"
  "utext"
  "utrans"
  "unorm2"
)

ICU_INCLUDE_PATH="$(icu-config --cppflags-searchpath | sed -e 's/-I//' | sed -e 's/ //g')"

# Write a report out as we read the data.

REPORT_FILE="${TOP_DIR}/coverage/report.md"
REPORT_FILE_HEADER="${TOP_DIR}/coverage/report_header.md"
REPORT_FILE_DETAIL="${TOP_DIR}/coverage/report_detail.md"
cat <<EOF > "${REPORT_FILE_HEADER}"
# Implementation coverage report

| Header | Implemented |
| ------ | ----------- |
EOF

  cat <<EOF >>"${REPORT_FILE_DETAIL}"
# Unimplemented functions per header

EOF

for header_basename in ${C_API_HEADER_NAMES[@]}; do
  : > "${TOP_DIR}/coverage/${header_basename}_all.txt"
  : > "${TOP_DIR}/coverage/${header_basename}_implemented.txt"
  header_fullname="${ICU_INCLUDE_PATH}/unicode/${header_basename}.h"
  all=$(ctags -x --c-kinds=fp $header_fullname | sed -e 's/ .*$//' \
    | grep -v U_DEFINE | sort -fs | uniq)
  for fn in ${all}; do
    printf "%s\n" ${fn} >> "${TOP_DIR}/coverage/${header_basename}_all.txt"
  done

  # Extracts all mentions of functions such as "utext_close" for example from
  # rust docs of the form "/// Implements `utext_close` ... ".  This is
  # simplistic but quite enough with a little bit of care.
  files=`find . -path "*rust_icu_${header_basename}/src/*.rs"`
  impl_fns=""
  unimpl_fns=""
  for file in ${files}; do
    echo $header_basename: $header_fullname ${file}
    found_fns="$(grep "/// Implements \`" ${file} | sed -e 's/.*`\(.*\)`.*$/\1/' | sed -e 's/\(.*\)()$/\1/' | sort -fs | uniq)"
    # Match the extracted function to a function in the header being processed
    # (in case the "/// Implements `" comment is used in another context)
    for impl_fn in ${found_fns}; do
      for all_fn in ${all}; do
        if [ ${impl_fn} = ${all_fn} ]
        then
          impl_fns="${impl_fns} ${impl_fn}"
          break
        fi
      done
    done
  done
  # Sort again in case we process multiple source files
  impl_fns="$(echo ${impl_fns} | sort -fs | uniq)"
  for impl_fn in ${impl_fns}; do
    printf "%s\n" ${impl_fn} >> "${TOP_DIR}/coverage/${header_basename}_implemented.txt"
  done
  unimpl_fns=""
  for all_fn in ${all}; do
    found="false"
    for impl_fn in ${impl_fns}; do
      if [ ${impl_fn} = ${all_fn} ]
      then
        found="true"
        break
      fi
    done
    if [ "false" = ${found} ]
    then
      unimpl_fns="${unimpl_fns} ${all_fn}"
    fi
  done

  total_functions=$(echo ${all} | wc -w)
  implemented_functions=$(echo ${impl_fns} | wc -w)
  printf "| \`%s.h\` | %s / %s | \n" ${header_basename} ${implemented_functions} ${total_functions} >> "${REPORT_FILE_HEADER}"

  cat <<EOF >>"${REPORT_FILE_DETAIL}"

# Header: \`${header_basename}.h\`

| Unimplemented | Implemented |
| ------------- | ----------- |
EOF
  for impl_fn in ${impl_fns}; do
    printf "| | \`%s\` |\n" ${impl_fn} >>"${REPORT_FILE_DETAIL}"
  done

  for fun in ${unimpl_fns}; do
    printf "| \`%s\` | |\n"  ${fun} >>"${REPORT_FILE_DETAIL}"
  done

  sort -fs -o "${TOP_DIR}/coverage/${header_basename}_all.txt" "${TOP_DIR}/coverage/${header_basename}_all.txt"
  sort -fs -o "${TOP_DIR}/coverage/${header_basename}_implemented.txt" "${TOP_DIR}/coverage/${header_basename}_implemented.txt"
done

cat ${REPORT_FILE_HEADER} ${REPORT_FILE_DETAIL} > ${REPORT_FILE}
rm ${REPORT_FILE_HEADER}
rm ${REPORT_FILE_DETAIL}

