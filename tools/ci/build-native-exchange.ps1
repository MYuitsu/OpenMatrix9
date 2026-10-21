# SPDX-License-Identifier: LGPL-2.1-or-later
# CI helper: build the pinned openNURBS static library and the native 3DM reader/writer.
# Local callers may point at existing build outputs with -OpenNurbsLib/-OpenNurbsZlib/-JsonDir.
param(
    [string]$OutDir = "",
    [string]$OpenNurbsLib = "",
    [string]$OpenNurbsZlib = "",
    [string]$JsonDir = ""
)
$ErrorActionPreference = "Stop"
. (Join-Path $PSScriptRoot 'msvc.ps1')
Initialize-OM9Msvc
$root = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
if (-not $OutDir) { $OutDir = Join-Path $root "build/ci/native-exchange" }
$openSource = Join-Path $root "build/standalone-deps/opennurbs"
$jsonSource = Join-Path $root "build/standalone-deps/nlohmann-json-3.11.3"
$pin = "eb92af3ba1806b0a34a99aba0d3bda83e3d46083"

if (-not $OpenNurbsLib) {
    if (-not (Test-Path (Join-Path $openSource ".git"))) {
        git clone --filter=blob:none https://github.com/mcneel/opennurbs.git $openSource
        if ($LASTEXITCODE -ne 0) { throw "openNURBS clone failed" }
    }
    git -C $openSource checkout $pin
    if ($LASTEXITCODE -ne 0) { throw "openNURBS pin checkout failed" }
    $known = Join-Path $root "build/native-brep/out/opennurbs/Release/opennurbsStatic.lib"
    $built = if (Test-Path $known) {
        Get-Item $known
    } else {
        Get-ChildItem $openSource -Recurse -Filter opennurbsStatic.lib -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match 'Release' } | Select-Object -First 1
    }
    if (-not $built) {
        $binary = Join-Path $root "build/ci/opennurbs-msvc"
        & cmake -S $openSource -B $binary -G 'Visual Studio 17 2022' -A x64 -DCMAKE_MSVC_RUNTIME_LIBRARY=MultiThreadedDLL
        if ($LASTEXITCODE -ne 0) { throw "openNURBS configure failed" }
        & cmake --build $binary --config Release --target opennurbsStatic --parallel 2
        if ($LASTEXITCODE -ne 0) { throw "openNURBS build failed" }
        $built = Get-ChildItem $binary -Recurse -Filter opennurbsStatic.lib | Select-Object -First 1
    }
    if (-not $built) { throw "opennurbsStatic.lib not found after build" }
    $OpenNurbsLib = $built.DirectoryName
    $zlib = Get-ChildItem $built.Directory.Parent.FullName -Recurse -Filter zlib.lib -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if (-not $zlib) { throw "openNURBS zlib.lib not found" }
    $OpenNurbsZlib = $zlib.DirectoryName
}
if (-not $JsonDir) { $JsonDir = $jsonSource }
if (-not (Test-Path (Join-Path $JsonDir "json.hpp"))) {
    New-Item -ItemType Directory -Force $JsonDir | Out-Null
    Invoke-WebRequest "https://raw.githubusercontent.com/nlohmann/json/v3.11.3/single_include/nlohmann/json.hpp" -OutFile (Join-Path $JsonDir "json.hpp")
    Invoke-WebRequest "https://raw.githubusercontent.com/nlohmann/json/v3.11.3/LICENSE.MIT" -OutFile (Join-Path $JsonDir "LICENSE.MIT")
}
& (Join-Path $root "tools/build-native-exchange.ps1") -OpenNurbsLib $OpenNurbsLib -OpenNurbsZlib $OpenNurbsZlib -JsonDir $JsonDir -OutDir $OutDir
if ($LASTEXITCODE -ne 0) { throw "native exchange build failed" }
Get-ChildItem $OutDir | Where-Object { $_.Name -like "om9-exchange-native*" } | Select-Object FullName,Length
