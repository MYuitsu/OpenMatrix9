#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-2.1-or-later
set -euo pipefail
cd "$(dirname "$0")/../.."
test "$(uname -sm)" = "Darwin arm64"
: "${CARGO_TARGET_DIR:?Set CARGO_TARGET_DIR outside the checkout}"
case "$CARGO_TARGET_DIR" in
  /*) ;;
  *) echo "CARGO_TARGET_DIR must be absolute" >&2; exit 1 ;;
esac
case "$CARGO_TARGET_DIR/" in
  "$PWD/"*) echo "CARGO_TARGET_DIR must be outside the checkout" >&2; exit 1 ;;
esac
export MACOSX_DEPLOYMENT_TARGET=15.0
export CARGO_BUILD_JOBS=2 CMAKE_BUILD_PARALLEL_LEVEL=2
export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-Wl,-headerpad_max_install_names"
root="$PWD/build/ci/macos"
deps="$root/dependencies"
app="$root/OpenMatrix9.app"
bin="$app/Contents/MacOS"
frameworks="$app/Contents/Frameworks"
resources="$app/Contents/Resources"
mkdir -p "$deps" "$bin" "$frameworks" "$resources"

# Copy and verify the same OCCT source and accuracy patch pinned for Windows.
read_pin() { python3 -c 'import json,sys; x=json.load(open("native/occt/pin.json")); print(x["sources"]["source_archive"][sys.argv[1]])' "$1"; }
archive="$deps/occt.zip"
curl --fail --location --silent --show-error --retry 3 "$(read_pin url)" -o "$archive"
printf '%s  %s\n' "$(read_pin sha256)" "$archive" | shasum -a 256 -c -
unzip -q "$archive" -d "$deps"
occt="$deps/OCCT-8_0_1"
accuracy_patch="$PWD/tools/patches/occt-8.0.1-offset-d3.patch"
patch_digest=$(python3 -c 'import json; print(json.load(open("native/occt/pin.json"))["accuracy_patch"]["sha256"])')
printf '%s  %s\n' "$patch_digest" "$accuracy_patch" | shasum -a 256 -c -
git -C "$occt" apply --ignore-space-change "$accuracy_patch"
sdk="$deps/occt-sdk"

# Build both the pinned openNURBS library and the standalone native exchange tool.
pin=eb92af3ba1806b0a34a99aba0d3bda83e3d46083
open="$deps/opennurbs"
git init -q "$open"
git -C "$open" fetch --depth 1 https://github.com/mcneel/opennurbs.git "$pin"
git -C "$open" checkout -q --detach FETCH_HEAD
json="$deps/nlohmann-json"
mkdir "$json"
for name in json.hpp LICENSE.MIT; do
  case "$name" in json.hpp) key=single_include/nlohmann/json.hpp ;; *) key=LICENSE.MIT ;; esac
  curl --fail --location --silent --show-error --retry 3 \
    "https://raw.githubusercontent.com/nlohmann/json/v3.11.3/$key" -o "$json/$name"
done
native="$deps/native-exchange"
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
cp -P "$sdk"/lib/*.dylib "$frameworks/"

export OM9_OCCT_SDK="$sdk"
export OM9_OPENNURBS_INCLUDE="$open"
export OM9_OPENNURBS_LIB="$(dirname "$(find "$native" -name libopennurbsStatic.a -print -quit)")"
export OM9_OPENNURBS_ZLIB="$(dirname "$(find "$native" -name libzlib.a -print -quit)")"

# Bundle-only load paths. Rebind dependencies to the paired dylibs in the app.
relocate() {
  local file="$1" prefix="$2" dep id=""
  lipo -verify_arch arm64 "$file"
  if [[ "$file" == *.dylib ]]; then
    id="$(otool -D "$file" | tail -n +2)"
    install_name_tool -id "@rpath/${file##*/}" "$file"
  fi
  while IFS= read -r dep; do
    [[ "$dep" != "$id" && "$dep" != "@rpath/${file##*/}" ]] || continue
    case "$dep" in
      /usr/lib/*|/System/Library/*) ;;
      *) test -f "$frameworks/${dep##*/}"
         install_name_tool -change "$dep" "$prefix/${dep##*/}" "$file" ;;
    esac
  done < <(otool -L "$file" | tail -n +2 | sed -E 's/^[[:space:]]+//; s/ \(compatibility version.*$//')
  codesign --force --sign - "$file"
}
while IFS= read -r -d '' lib; do
  relocate "$lib" '@loader_path'
done < <(find "$frameworks" -type f -name '*.dylib' -print0)
cp "$native/om9-exchange-native" "$bin/"
relocate "$bin/om9-exchange-native" '@loader_path/../Frameworks'
shasum -a 256 "$bin/om9-exchange-native" | cut -d ' ' -f 1 > "$bin/om9-exchange-native.sha256"
cargo build --locked --release -j 2 -p om9-kernel --features isolated-brep-worker \
  --bin om9-brep-restore-worker --bin om9-native-brep-worker
release="$CARGO_TARGET_DIR/release"
for worker in om9-brep-restore-worker om9-native-brep-worker; do
  cp "$release/$worker" "$bin/"
  relocate "$bin/$worker" '@loader_path/../Frameworks'
done
export OM9_BREP_WORKER_SHA256="$(shasum -a 256 "$bin/om9-brep-restore-worker" | cut -d ' ' -f 1)"
export OM9_NATIVE_BREP_WORKER_SHA256="$(shasum -a 256 "$bin/om9-native-brep-worker" | cut -d ' ' -f 1)"
cargo build --locked --release -j 2 -p om9-app --bin om9
cp "$release/om9" "$bin/"
relocate "$bin/om9" '@loader_path/../Frameworks'

cargo build --locked --release -j 2 -p om9-addon-matrix --lib
cargo build --locked --release -j 2 -p om9-addons --features native --bin om9-addon
addons="$bin/addons"
matrix="$addons/om9_matrix"
mkdir -p "$matrix"
ditto addons/matrix/addon "$matrix"
sed 's/@PLATFORM@/macos-arm64/g; s/@ENTRY@/libom9_matrix.dylib/g' \
  addons/matrix/addon/om9_addon.toml > "$matrix/om9_addon.toml"
cp "$release/libom9_matrix.dylib" "$matrix/"
relocate "$matrix/libom9_matrix.dylib" '@loader_path/../../../Frameworks'
ditto addons/ring_sizes "$addons/ring_sizes"

cp LICENSE THIRD_PARTY_NOTICES.md "$resources/"
ditto Resources/licenses "$resources/licenses"
cp "$occt/LICENSE_LGPL_21.txt" "$occt/OCCT_LGPL_EXCEPTION.txt" "$accuracy_patch" "$resources/"
cp "$occt/src/ModelingData/TKG3d/Geom/Geom_OffsetCurveUtils.pxx" "$resources/"
cp "$json/LICENSE.MIT" "$resources/nlohmann-json-LICENSE.MIT"
find "$open" -maxdepth 1 -iname 'license*' -type f -exec cp {} "$resources/" \;
version="$(python3 -c 'import json; print(json.load(open("release.json"))["version"])')"
python3 - "$app" "$version" <<'PY'
import pathlib, plistlib, sys
app, version = pathlib.Path(sys.argv[1]), sys.argv[2]
with (app / "Contents/Info.plist").open("wb") as out:
    plistlib.dump(dict(CFBundleExecutable="om9", CFBundleIdentifier="org.openmatrix9.app",
                      CFBundleName="OpenMatrix9", CFBundlePackageType="APPL",
                      CFBundleShortVersionString=version.split("-")[0],
                      CFBundleVersion=version.split("-")[0],
                      LSMinimumSystemVersion="15.0", NSHighResolutionCapable=True), out)
PY
codesign --force --sign - "$app"
codesign --verify --deep --strict "$app"
python3 tools/ci/smoke_all_in_one.py --app "$bin/om9" --addon-tool "$release/om9-addon"
dist="$root/packages"
mkdir "$dist"
ditto -c -k --keepParent "$app" "$dist/OpenMatrix9-$version-All-in-One-macOS-arm64.zip"
(cd "$dist"; shasum -a 256 ./*.zip > SHA256SUMS.txt; shasum -a 256 -c SHA256SUMS.txt)
