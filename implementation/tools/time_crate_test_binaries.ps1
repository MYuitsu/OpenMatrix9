# Time every test binary of one crate, so "why is the pass slow" has a measured answer
# instead of a guess. Build/link time is excluded: `--no-run` builds first, then each
# compiled test executable is started once and timed.
#
#   pwsh -File implementation/tools/time_crate_test_binaries.ps1 -Crate om9-app
#   pwsh -File implementation/tools/time_crate_test_binaries.ps1 -Crate om9-archive -Top 8
#
# Cargo runs test *binaries* one at a time, so this sum is the floor for
# `cargo test -p <crate>`; -TestThreads only affects tests inside one binary.
param(
    [Parameter(Mandatory = $true)][string]$Crate,
    [int]$Top = 10,
    [int]$TestThreads = 0,
    [string]$TargetDir = 'L:\om9-target',
    # This host's 8 GiB paging file cannot hold default parallelism (rustc: os error 1455).
    [int]$Jobs = 2
)
$ErrorActionPreference = 'Stop'
if ($TargetDir) { $env:CARGO_TARGET_DIR = $TargetDir }
$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$workspace = $root   # crates live at the repository root now
Push-Location $workspace
try {
    $json = & cargo test -p $Crate --no-run -j $Jobs --message-format=json 2>$null
    $exes = $json | ForEach-Object {
        if ($_ -match '"executable":"([^"]+)"') { $Matches[1] }
    } | Where-Object { $_ -and $_ -ne 'null' } | Select-Object -Unique
    if (-not $exes) { throw "no test binaries for $Crate" }
    Write-Host "$Crate : $($exes.Count) test binaries"
    $rows = foreach ($exe in $exes) {
        $arguments = @()
        if ($TestThreads -gt 0) { $arguments += @('--test-threads', $TestThreads) }
        $arguments += @('-q')
        $watch = [System.Diagnostics.Stopwatch]::StartNew()
        & $exe @arguments *> $null
        $watch.Stop()
        [pscustomobject]@{
            ms   = [int]$watch.ElapsedMilliseconds
            name = (Split-Path $exe -Leaf) -replace '-[0-9a-f]{16}\.exe$', ''
        }
    }
    $rows | Sort-Object ms -Descending | Select-Object -First $Top | Format-Table -AutoSize
    $total = ($rows | Measure-Object ms -Sum).Sum
    Write-Host ("total test-binary time: {0:N1}s over {1} binaries" -f ($total / 1000), $rows.Count)
}
finally { Pop-Location }
