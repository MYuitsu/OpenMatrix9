# SPDX-License-Identifier: LGPL-2.1-or-later
# Build the contained restore worker first, then embed its image digest in OM9.
# This does not authenticate dependent native libraries or publish a release.
$ErrorActionPreference = 'Stop'
$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$targetInput = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { 'L:\om9-target' }
$targetPath = if ([System.IO.Path]::IsPathRooted($targetInput)) {
    [System.IO.Path]::GetFullPath($targetInput)
} else { [System.IO.Path]::GetFullPath((Join-Path $repoRoot $targetInput)) }
$rootPrefix = $repoRoot.TrimEnd('\') + '\'
if ($targetPath.Equals($repoRoot, [System.StringComparison]::OrdinalIgnoreCase) -or
    $targetPath.StartsWith($rootPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'CARGO_TARGET_DIR must be outside the checkout.'
}
if (-not (Test-Path -LiteralPath $targetPath -PathType Container)) {
    throw 'Reuse an existing OM9 target directory; no new build copy was created.'
}
$component = Get-Item -LiteralPath $targetPath
while ($null -ne $component) {
    if (($component.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw 'The build target must not pass through a filesystem link.'
    }
    $component = $component.Parent
}
$fingerprints = Join-Path $targetPath 'debug/.fingerprint'
if (-not (Test-Path -LiteralPath $fingerprints -PathType Container) -or
    -not (Get-ChildItem -LiteralPath $fingerprints -Filter 'om9-core-*' -Directory | Select-Object -First 1)) {
    throw 'The target directory must be an existing OM9 build target.'
}
if (Get-Process -Name 'om9' -ErrorAction SilentlyContinue) {
    throw 'OM9 is running; close it before rebuilding the worker and application together.'
}
$priorTarget = $env:CARGO_TARGET_DIR
$priorDigest = $env:OM9_BREP_WORKER_SHA256
$priorNativeDigest = $env:OM9_NATIVE_BREP_WORKER_SHA256
try {
    $env:CARGO_TARGET_DIR = $targetPath
    Push-Location -LiteralPath $repoRoot
    try {
        & rtk proxy cargo build -p om9-kernel --features isolated-brep-worker --bin om9-brep-restore-worker --bin om9-native-brep-worker -j 2
        if ($LASTEXITCODE -ne 0) { throw 'BRep worker build failed.' }
        $workerPath = Join-Path $targetPath 'debug/om9-brep-restore-worker.exe'
        $env:OM9_BREP_WORKER_SHA256 = (Get-FileHash -LiteralPath $workerPath -Algorithm SHA256).Hash.ToLowerInvariant()
        $nativeWorkerPath = Join-Path $targetPath 'debug/om9-native-brep-worker.exe'
        $env:OM9_NATIVE_BREP_WORKER_SHA256 = (Get-FileHash -LiteralPath $nativeWorkerPath -Algorithm SHA256).Hash.ToLowerInvariant()
        & rtk proxy cargo build -p om9-app --bin om9 -j 2
        if ($LASTEXITCODE -ne 0) { throw 'OM9 build failed.' }
        Write-Output "Built OM9 with BRep worker image SHA-256 $env:OM9_BREP_WORKER_SHA256 and native BRep worker image SHA-256 $env:OM9_NATIVE_BREP_WORKER_SHA256"
    } finally {
        Pop-Location
    }
} finally {
    $env:CARGO_TARGET_DIR = $priorTarget
    $env:OM9_BREP_WORKER_SHA256 = $priorDigest
    $env:OM9_NATIVE_BREP_WORKER_SHA256 = $priorNativeDigest
}
