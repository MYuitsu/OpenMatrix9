<#
Preview: powershell -NoProfile -ExecutionPolicy Bypass -File tools/clean-l-rust-build.ps1
Delete listed caches: add -Apply. Change the age with -Days 60.
Only old incremental compilation directories are removed; dependencies and executables stay.
#>
[CmdletBinding()]
param([switch]$Apply, [ValidateRange(1, 36500)][int]$Days = 30)

$ErrorActionPreference = 'Stop'
if ($Apply -and @(Get-Process -Name cargo, rustc -ErrorAction SilentlyContinue).Count) {
    throw 'Cargo or rustc is running. Wait for all Rust builds to finish before cleaning.'
}
$cutoff = (Get-Date).AddDays(-$Days)
$total = 0
foreach ($root in @('L:\om9-target', 'L:\om9-head-target', 'L:\om9-nav-target')) {
    foreach ($profile in @('debug', 'release')) {
        $path = Join-Path $root "$profile\incremental"
        if (-not (Test-Path -LiteralPath $path)) { continue }
        $ancestor = Get-Item -LiteralPath $path -Force
        while ($ancestor -and $ancestor.FullName -ne 'L:\') {
            if ($ancestor.Attributes -band [IO.FileAttributes]::ReparsePoint) {
                throw "Refusing redirected path: $($ancestor.FullName)"
            }
            $ancestor = $ancestor.Parent
        }
        foreach ($item in Get-ChildItem -LiteralPath $path -Directory -Force) {
            if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $item.LastWriteTime -ge $cutoff) { continue }
            $files = @(Get-ChildItem -LiteralPath $item.FullName -File -Recurse -Force -ErrorAction Stop)
            # A recently changed child means the cache is still in use, even if its parent is old.
            if (@($files | Where-Object { $_.LastWriteTime -ge $cutoff }).Count) { continue }
            $bytes = ($files | Measure-Object -Property Length -Sum).Sum
            $total += $bytes
            Write-Host ("{0}: {1:N2} GiB, last directory change {2:yyyy-MM-dd}" -f $item.FullName, ($bytes / 1GB), $item.LastWriteTime)
            if ($Apply) { Remove-Item -LiteralPath $item.FullName -Recurse -Force -ErrorAction Stop }
        }
    }
}
Write-Host ("Total: {0:N2} GiB. Dependencies, downloaded libraries and executables are retained." -f ($total / 1GB))
if (-not $Apply) { Write-Host "Preview only. Add -Apply to delete caches older than $Days days." }
