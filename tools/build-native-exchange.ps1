# SPDX-License-Identifier: LGPL-2.1-or-later
# Build the native openNURBS 3DM reader/writer used by the Rust host adapter.
# The checked-in build recipe is intentionally direct: it links the already-built pinned
# openNURBS static library instead of rebuilding openNURBS through CMake.
param(
    [string]$OpenNurbsLib = "",
    [string]$OpenNurbsInclude = "",
    [string]$OpenNurbsZlib = "",
    [string]$JsonDir = "",
    [string]$OutDir = "",
    [string]$VcVars = ""
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if (-not $OpenNurbsLib) { $OpenNurbsLib = Join-Path $root "build/native-brep/out/opennurbs/Release" }
if (-not $OpenNurbsInclude) { $OpenNurbsInclude = Join-Path $root "build/standalone-deps/opennurbs" }
if (-not $OpenNurbsZlib) { $OpenNurbsZlib = Join-Path $root "build/native-brep/out/opennurbs/zlib/Release" }
if (-not $JsonDir) { $JsonDir = Join-Path $root "build/standalone-deps/nlohmann-json-3.11.3" }
if (-not $OutDir) { $OutDir = Join-Path $root "build/native-exchange/Release" }

if (-not $VcVars -or -not (Test-Path $VcVars)) {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio/Installer/vswhere.exe"
    $vs = if (Test-Path $vswhere) {
        & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    }
    if (-not $vs) { throw "MSVC x64 toolchain unavailable" }
    $VcVars = Join-Path $vs "VC/Auxiliary/Build/vcvars64.bat"
}
if (-not (Test-Path $VcVars)) { throw "vcvars64.bat not found: $VcVars" }

$library = Join-Path $OpenNurbsLib "opennurbsStatic.lib"
$zlib = Join-Path $OpenNurbsZlib "zlib.lib"
$header = Join-Path $JsonDir "json.hpp"
$license = Join-Path $JsonDir "LICENSE.MIT"
foreach ($item in @($library, $zlib, $header, $license, (Join-Path $root "native/native_exchange.cpp"))) {
    if (-not (Test-Path $item)) { throw "missing native build input: $item" }
}
$expectedHeader = "9bea4c8066ef4a1c206b2be5a36302f8926f7fdc6087af5d20b417d0cf103ea6"
$actualHeader = (Get-FileHash $header -Algorithm SHA256).Hash.ToLower()
if ($actualHeader -ne $expectedHeader) { throw "nlohmann/json header pin mismatch: $actualHeader" }
$expectedLicense = "86b998c792894ccb911a1cb7994f7a9652894e7a094c0b5e45be2f553f45cf14"
$actualLicense = (Get-FileHash $license -Algorithm SHA256).Hash.ToLower()
if ($actualLicense -ne $expectedLicense) { throw "nlohmann/json license pin mismatch: $actualLicense" }

New-Item -ItemType Directory -Force $OutDir | Out-Null
$exe = Join-Path $OutDir "om9-exchange-native.exe"
$source = Join-Path $root "native/native_exchange.cpp"
$command = 'call "' + $VcVars + '" >nul && cl /nologo /MD /std:c++20 /EHsc /utf-8 /O2 /DNDEBUG ' +
    '/DNOMINMAX /DON_CMAKE_BUILD ' +
    '/I "' + $JsonDir + '" /I "' + $OpenNurbsInclude + '" "' + $source + '" ' +
    '/Fe:"' + $exe + '" /link /SUBSYSTEM:WINDOWS /ENTRY:wmainCRTStartup ' +
    '"' + $library + '" "' + $zlib + '" ' +
    'Bcrypt.lib Shlwapi.lib Usp10.lib psapi.lib shell32.lib advapi32.lib Gdi32.lib User32.lib'
& cmd /c $command
if ($LASTEXITCODE -ne 0) { throw "native exchange compilation failed: $LASTEXITCODE" }
$fingerprint = (Get-FileHash $exe -Algorithm SHA256).Hash.ToLower()
Set-Content -LiteralPath (Join-Path $OutDir "om9-exchange-native.sha256") -Value $fingerprint -Encoding ascii
@{ executable = "om9-exchange-native.exe"; sha256 = $fingerprint;
   opennurbs_pin = "eb92af3ba1806b0a34a99aba0d3bda83e3d46083";
   nlohmann_json_sha256 = $actualHeader } |
    ConvertTo-Json | Set-Content -LiteralPath (Join-Path $OutDir "om9-exchange-native.manifest.json") -Encoding utf8
Write-Output "built $exe"
Write-Output "sha256 $fingerprint"
