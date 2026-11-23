#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-2.1-or-later
set -euo pipefail
cd "$(dirname "$0")/../.."
test "$(uname -sm)" = "Linux x86_64"
: "${CARGO_TARGET_DIR:?Set CARGO_TARGET_DIR outside the checkout}"
case "$CARGO_TARGET_DIR" in
  /*) ;;
  *) echo "CARGO_TARGET_DIR must be absolute" >&2; exit 1 ;;
esac
case "$CARGO_TARGET_DIR/" in
  "$PWD/"*) echo "CARGO_TARGET_DIR must be outside the checkout" >&2; exit 1 ;;
esac
export CARGO_BUILD_JOBS=2
root="$PWD/build/ci/linux"
deps="$root/dependencies"
native="$deps/native-exchange"
occt="$deps/OCCT-8_0_1"
app="$root/OpenMatrix9"
lib="$app/lib"
matrix="$app/addons/om9_matrix"
dist="$root/packages"
mkdir -p "$lib" "$matrix" "$dist"

export OM9_OCCT_SDK="$deps/occt-sdk"
export OM9_OPENNURBS_INCLUDE="$deps/opennurbs"
export OM9_OPENNURBS_LIB="$(dirname "$(find "$native" -name libopennurbsStatic.a -print -quit)")"
export OM9_OPENNURBS_ZLIB="$(dirname "$(find "$native" -name libzlib.a -print -quit)")"
cp -P "$OM9_OCCT_SDK"/lib/*.so* "$lib/"
# winit opens the XKB libraries at runtime, so ldd cannot discover them from om9.
for dependency in libxkbcommon-x11-0:libxkbcommon-x11.so.0 \
  libxkbcommon0:libxkbcommon.so.0 libxcb-xkb1:libxcb-xkb.so.1; do
  package="${dependency%%:*}"
  soname="${dependency#*:}"
  path="$(dpkg-query -L "$package" | awk -F/ -v name="$soname" '$NF == name')"
  test -f "$path"
  cp -L "$path" "$lib/$soname"
  cp "/usr/share/doc/$package/copyright" "$app/$package-copyright"
done
while IFS= read -r -d '' library; do
  patchelf --set-rpath '$ORIGIN' "$library"
done < <(find "$lib" -type f -print0)
cp "$native/om9-exchange-native" "$app/"
patchelf --set-rpath '$ORIGIN/lib' "$app/om9-exchange-native"
sha256sum "$app/om9-exchange-native" | cut -d ' ' -f 1 > "$app/om9-exchange-native.sha256"

cargo build --locked --release -j 2 -p om9-kernel --features isolated-brep-worker \
  --bin om9-brep-restore-worker --bin om9-native-brep-worker
release="$CARGO_TARGET_DIR/release"
for worker in om9-brep-restore-worker om9-native-brep-worker; do
  cp "$release/$worker" "$app/"
  patchelf --set-rpath '$ORIGIN/lib' "$app/$worker"
done
export OM9_BREP_WORKER_SHA256="$(sha256sum "$app/om9-brep-restore-worker" | cut -d ' ' -f 1)"
export OM9_NATIVE_BREP_WORKER_SHA256="$(sha256sum "$app/om9-native-brep-worker" | cut -d ' ' -f 1)"
cargo build --locked --release -j 2 -p om9-app --bin om9
cp "$release/om9" "$app/"
patchelf --set-rpath '$ORIGIN/lib' "$app/om9"

cargo build --locked --release -j 2 -p om9-addon-matrix --lib
cargo build --locked --release -j 2 -p om9-addons --features native --bin om9-addon
cp -a addons/matrix/addon/. "$matrix/"
sed 's/@PLATFORM@/linux-x64/g; s/@ENTRY@/libom9_matrix.so/g' \
  addons/matrix/addon/om9_addon.toml > "$matrix/om9_addon.toml"
cp "$release/libom9_matrix.so" "$matrix/"
patchelf --set-rpath '$ORIGIN/../../lib' "$matrix/libom9_matrix.so"
cp -a addons/ring_sizes "$app/addons/ring_sizes"

cp LICENSE THIRD_PARTY_NOTICES.md "$app/"
cp -a Resources/licenses "$app/licenses"
cp "$occt/LICENSE_LGPL_21.txt" "$occt/OCCT_LGPL_EXCEPTION.txt" \
  tools/patches/occt-8.0.1-offset-d3.patch \
  "$occt/src/ModelingData/TKG3d/Geom/Geom_OffsetCurveUtils.pxx" "$app/"
cp "$deps/nlohmann-json/LICENSE.MIT" "$app/nlohmann-json-LICENSE.MIT"
find "$OM9_OPENNURBS_INCLUDE" -maxdepth 1 -iname 'license*' -type f -exec cp {} "$app/" \;

version="$(python3 -c 'import json; print(json.load(open("release.json"))["version"])')"
name="OpenMatrix9-$version-All-in-One-Linux-x64.tar.gz"
tar -czf "$dist/$name" -C "$root" OpenMatrix9
(cd "$dist"; sha256sum "$name" > SHA256SUMS.txt; sha256sum --check SHA256SUMS.txt)
mkdir -p "$root/extracted"
tar -xzf "$dist/$name" -C "$root/extracted"
extracted="$root/extracted/OpenMatrix9"
python3 tools/ci/smoke_all_in_one.py --app "$extracted/om9" \
  --addon-tool "$release/om9-addon"
