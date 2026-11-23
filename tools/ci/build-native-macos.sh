#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-2.1-or-later
set -euo pipefail
cd "$(dirname "$0")/../.."
test "$(uname -sm)" = "Darwin arm64"
export MACOSX_DEPLOYMENT_TARGET=15.0
export CMAKE_BUILD_PARALLEL_LEVEL=2
deps="$PWD/build/ci/macos/dependencies"
mkdir -p "$deps"
accuracy_patch="$PWD/tools/patches/occt-8.0.1-offset-d3.patch"
occt="$deps/OCCT-8_0_1"
sdk="$deps/occt-sdk"
open="$deps/opennurbs"
json="$deps/nlohmann-json"
native="$deps/native-exchange"

# Copy and verify the same OCCT source and accuracy patch pinned for Windows.
read_pin() { python3 -c 'import json,sys; x=json.load(open("native/occt/pin.json")); print(x["sources"]["source_archive"][sys.argv[1]])' "$1"; }
archive="$deps/occt.zip"
curl --fail --location --silent --show-error --retry 3 "$(read_pin url)" -o "$archive"
printf '%s  %s\n' "$(read_pin sha256)" "$archive" | shasum -a 256 -c -
unzip -q "$archive" -d "$deps"
patch_digest=$(python3 -c 'import json; print(json.load(open("native/occt/pin.json"))["accuracy_patch"]["sha256"])')
printf '%s  %s\n' "$patch_digest" "$accuracy_patch" | shasum -a 256 -c -
git -C "$occt" apply --ignore-space-change "$accuracy_patch"

# Build both the pinned openNURBS library and the standalone native exchange tool.
pin=eb92af3ba1806b0a34a99aba0d3bda83e3d46083
git init -q "$open"
git -C "$open" fetch --depth 1 https://github.com/mcneel/opennurbs.git "$pin"
git -C "$open" checkout -q --detach FETCH_HEAD
mkdir "$json"
for name in json.hpp LICENSE.MIT; do
  case "$name" in json.hpp) key=single_include/nlohmann/json.hpp ;; *) key=LICENSE.MIT ;; esac
  curl --fail --location --silent --show-error --retry 3 \
    "https://raw.githubusercontent.com/nlohmann/json/v3.11.3/$key" -o "$json/$name"
done
cmake -S native -B "$native" -G Ninja -DCMAKE_BUILD_TYPE=Release -DCMAKE_OSX_ARCHITECTURES=arm64 \
  -DOM9_OPENNURBS_SOURCE="$open" -DOM9_JSON_SOURCE="$json"
cmake --build "$native" --target om9-exchange-native --parallel 2

cmake -S "$occt" -B "$deps/occt-build" -G Ninja \
  -DCMAKE_BUILD_TYPE=Release -DCMAKE_OSX_ARCHITECTURES=arm64 \
  -DCMAKE_INSTALL_PREFIX="$sdk" -DINSTALL_DIR_LIB=lib -DINSTALL_DIR_INCLUDE=include/opencascade \
  -DCMAKE_INSTALL_NAME_DIR=@rpath -DCMAKE_SHARED_LINKER_FLAGS=-Wl,-headerpad_max_install_names \
  -DBUILD_MODULE_FoundationClasses=OFF -DBUILD_MODULE_ModelingData=OFF \
  -DBUILD_MODULE_ModelingAlgorithms=OFF -DBUILD_MODULE_Visualization=OFF \
  -DBUILD_MODULE_ApplicationFramework=OFF -DBUILD_MODULE_DataExchange=OFF -DBUILD_MODULE_Draw=OFF \
  '-DBUILD_ADDITIONAL_TOOLKITS=TKernel;TKMath;TKG2d;TKG3d;TKGeomBase;TKBRep;TKTopAlgo;TKGeomAlgo;TKPrim;TKBO;TKBool;TKMesh;TKFillet;TKOffset;TKShHealing' \
  -DUSE_FREETYPE=OFF -DUSE_TBB=OFF -DUSE_JEMALLOC=OFF -DBUILD_TESTING=OFF
cmake --build "$deps/occt-build" --parallel 2
cmake --install "$deps/occt-build"
