# SPDX-License-Identifier: LGPL-2.1-or-later
param([string]$Root = (Join-Path $PSScriptRoot '../../build/ci/standalone'))
$ErrorActionPreference = 'Stop'
$source = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$Root = [IO.Path]::GetFullPath($Root)
New-Item -ItemType Directory -Force $Root | Out-Null
. (Join-Path $PSScriptRoot 'msvc.ps1')
Initialize-OM9Msvc
if (-not $env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR = Join-Path ([IO.Path]::GetTempPath()) 'om9-target' }
$env:CARGO_TARGET_DIR = [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR)
if ($env:CARGO_TARGET_DIR.Equals($source, [StringComparison]::OrdinalIgnoreCase) -or
    $env:CARGO_TARGET_DIR.StartsWith($source + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'CARGO_TARGET_DIR must be outside the checkout'
}
$env:CARGO_BUILD_JOBS = '2'
$hostInfo = & rustc -vV
if ($LASTEXITCODE -ne 0 -or -not ($hostInfo -match '^host: x86_64-pc-windows-msvc$')) {
    throw 'The Rust toolchain must target x86_64-pc-windows-msvc'
}
if ($env:CARGO_BUILD_TARGET) { throw 'Use the native MSVC host; do not override CARGO_BUILD_TARGET' }

& python (Join-Path $source 'implementation/tools/fetch_occt.py') --fetch
if ($LASTEXITCODE -ne 0) { throw 'OCCT pin verification/materialization failed' }
$sdk = Join-Path $source 'build/standalone-deps/occt-8.0.1-sdk'
$patchRoot = Join-Path $Root 'occt-offset-d3'
& python (Join-Path $source 'tools/build_occt_offset_d3.py') --archive (Join-Path $source 'build/standalone-deps/occt-V8_0_1-source.zip') --sdk $sdk --output $patchRoot --workers 2
if ($LASTEXITCODE -ne 0) { throw 'Pinned OCCT TKG3d accuracy patch build failed' }

& (Join-Path $PSScriptRoot 'build-native-exchange.ps1') -OutDir (Join-Path $Root 'native-exchange')
if ($LASTEXITCODE -ne 0) { throw 'Native 3DM adapter build failed' }
$known = Join-Path $source 'build/native-brep/out/opennurbs/Release/opennurbsStatic.lib'
$openLibrary = if (Test-Path $known) { Get-Item $known } else {
    Get-ChildItem (Join-Path $source 'build/ci/opennurbs-msvc') -Recurse -Filter opennurbsStatic.lib | Select-Object -First 1
}
if (-not $openLibrary) { throw 'MSVC openNURBS library missing' }
$openZlib = Get-ChildItem $openLibrary.Directory.Parent.FullName -Recurse -Filter zlib.lib | Select-Object -First 1
if (-not $openZlib) {
    $openZlib = Get-ChildItem (Join-Path $source 'build/ci/opennurbs-msvc') -Recurse -Filter zlib.lib | Select-Object -First 1
}
if (-not $openLibrary -or -not $openZlib) { throw 'MSVC openNURBS/zlib libraries missing' }
$env:OM9_OCCT_SDK = $sdk
$env:OM9_OPENNURBS_INCLUDE = Join-Path $source 'build/standalone-deps/opennurbs'
$env:OM9_OPENNURBS_LIB = $openLibrary.DirectoryName
$env:OM9_OPENNURBS_ZLIB = $openZlib.DirectoryName
Push-Location $source
try {
    & cargo build --locked --release -p om9-app --bin om9 -j 2
    if ($LASTEXITCODE -ne 0) { throw 'Standalone Rust application build failed' }
} finally { Pop-Location }
$crt = Get-ChildItem (Join-Path $env:VCToolsRedistDir 'x64') -Directory -Filter 'Microsoft.VC*.CRT' | Select-Object -First 1
if (-not $crt) { throw 'MSVC x64 redistributable CRT missing' }
[ordered]@{
    target = 'om9-app/om9'; features = @('occt-brep'); workers = 2
    binary = Join-Path $env:CARGO_TARGET_DIR 'release/om9.exe'
    native = Join-Path $Root 'native-exchange'
    sdk = $sdk; patch_root = $patchRoot; crt = $crt.FullName
    opennurbs_source = $env:OM9_OPENNURBS_INCLUDE
    thirdparty_archive = Join-Path $source 'build/standalone-deps/occt-8.0.1-3rdparty-vc14-64.zip'
    rustc = @($hostInfo); cargo_lock_sha256 = (Get-FileHash (Join-Path $source 'Cargo.lock')).Hash.ToLowerInvariant()
} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $Root 'build-paths.json') -Encoding utf8
& python (Join-Path $PSScriptRoot 'package_standalone.py') --root $Root
if ($LASTEXITCODE -ne 0) { throw 'Standalone runtime packaging failed' }
if ($env:GITHUB_ENV) { "OM9_CI_ROOT=$Root" | Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8 }
