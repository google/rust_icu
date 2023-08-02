#!/usr/bin/env bash
# Copyright 2023 Google LLC
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

set -eo pipefail

export USE_ICU_CONFIG=0

function check-requirements() {
  wget --version &> /dev/null || (echo "wget is not installed"; exit 1)
  openssl version &> /dev/null || (echo "openssl is not installed"; exit 1)
}

function populate-and-run () {
  echo "Populating $ICU_DOWNLOAD_NAME"
  mkdir -p cache
  local _download_to="cache/$ICU_DOWNLOAD_NAME"
  if [[ ! -f "$_download_to" ]]; then
    echo "Downloading $ICU_TGZ_URL"
    wget -q --show-progress -N "$ICU_TGZ_URL" -O "$_download_to" || \
      (echo "failed to download: $ICU_TGZ_URL"; exit 1)
  fi

  local _hash="$(openssl dgst -sha256 "$_download_to" | awk '{ print $NF }')"
  if [[ "$ICU_SHA256" != "$_hash" ]]; then
    echo "$_download_to corrupted, removing..."
    rm "$_download_to"
    exit 1
  fi

  local _extract_to="cache/${ICU_VERSION}-extracted"
  if [[ ! -d "$_extract_to" ]]; then
    echo "Extracting $_download_to"
    tar -xf "$_download_to" -C "cache" || (echo "failed to extract $_download_to"; exit 1)
    mv cache/icu "$_extract_to"
  fi

  ICU_PREFIX="${_extract_to}/usr/local"
  ICU_PREFIX="$(readlink -f "$ICU_PREFIX")"
  [[ -d "$ICU_PREFIX" ]] || (echo "$ICU_PREFIX is not directory"; exit 1)

  env ICU_PREFIX="$ICU_PREFIX" \
      ICU_VERSION="$ICU_VERSION" \
  ./run_bindgen.sh
}

check-requirements

ICU_TGZ_URL="https://github.com/unicode-org/icu/releases/download/release-73-2/icu4c-73_2-Ubuntu22.04-x64.tgz"
ICU_DOWNLOAD_NAME="icu4c-73_2-Ubuntu22.04-x64.tgz"
ICU_SHA256="ce669c2a36d735dfc36375e8536e030b0b79c5f0bc67025a6413fc1404b07e8b"
ICU_VERSION="73.2"
populate-and-run

ICU_TGZ_URL="https://github.com/unicode-org/icu/releases/download/release-72-1/icu4c-72_1-Ubuntu22.04-x64.tgz"
ICU_DOWNLOAD_NAME="icu4c-72_1-Ubuntu22.04-x64.tgz"
ICU_SHA256="2cdcf79509b372ff8cd55af8cc22738468285513e97ea70f200d72733ba1234c"
ICU_VERSION="72.1"
populate-and-run

ICU_TGZ_URL="https://github.com/unicode-org/icu/releases/download/release-71-1/icu4c-71_1-Ubuntu20.04-x64.tgz"
ICU_DOWNLOAD_NAME="icu4c-71_1-Ubuntu20.04-x64.tgz"
ICU_SHA256="a99c51ff09666308a1d597ccef08f2916bfb710d987a5309d7fbea4f2555c17d"
ICU_VERSION="71.1"
populate-and-run

ICU_TGZ_URL="https://github.com/unicode-org/icu/releases/download/release-70-1/icu4c-70_1-Ubuntu-20.04-x64.tgz"
ICU_DOWNLOAD_NAME="icu4c-70_1-Ubuntu-20.04-x64.tgz"
ICU_SHA256="a8134e9f8a68d33600749601e143e553b5cb48c217c8941dbb9ef478fac420dd"
ICU_VERSION="70.1"
populate-and-run

ICU_TGZ_URL="https://github.com/unicode-org/icu/releases/download/release-63-1/icu4c-63_1-Ubuntu-18.04-x64.tgz"
ICU_DOWNLOAD_NAME="icu4c-63_1-Ubuntu-18.04-x64.tgz"
ICU_SHA256="5d0ae6d982ce5f187003ded4f8caac292b8c65bcfabcd2ba34f67889181e18d6"
ICU_VERSION="63.1"
populate-and-run
