<# Preview by default. Add -Apply to delete old Cargo debug symbols.
   Keeps newest hashed PDBs per name, separately for each target and profile.
#>
[CmdletBinding()]
param([switch]$Apply, [ValidateRange(1, 1000)][int]$Keep = 5)
$ErrorActionPreference = 'Stop'
if ($Apply -and @(Get-Process -Name cargo, rustc -ErrorAction SilentlyContinue).Count) {
    throw 'Cargo or rustc is running; wait until builds finish.'
}
$count = 0
$bytes = 0
foreach ($root in @('L:\om9-target', 'L:\om9-head-target', 'L:\om9-nav-target')) {
    foreach ($profile in @('debug', 'release')) {
        $path = Join-Path $root "$profile\deps"
        if (-not (Test-Path -LiteralPath $path)) { continue }
        $ancestor = Get-Item -LiteralPath $path -Force
        while ($ancestor -and $ancestor.FullName -ne 'L:\') {
            if ($ancestor.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw "Redirected path: $($ancestor.FullName)" }
            $ancestor = $ancestor.Parent
        }
        $groups = Get-ChildItem -LiteralPath $path -File -Filter '*.pdb' -Force | Where-Object {
            -not ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) -and $_.Name -match '^.+-[0-9a-f]{16}\.pdb$'
        } | Group-Object { $_.Name -replace '-[0-9a-f]{16}\.pdb$', '' }
        foreach ($group in $groups) {
            $old = @($group.Group | Sort-Object LastWriteTimeUtc, Name -Descending | Select-Object -Skip $Keep)
            if (-not $old.Count) { continue }
            $size = ($old | Measure-Object Length -Sum).Sum
            $count += $old.Count
            $bytes += $size
            Write-Host ("{0} / {1}: {2} files, {3:N2} GiB" -f $path, $group.Name, $old.Count, ($size / 1GB))
            if ($Apply) {
                foreach ($file in $old) { Remove-Item -LiteralPath $file.FullName -Force -ErrorAction Stop }
            }
        }
    }
}
Write-Host ("Total {0} files, {1:N2} GiB logical size. {2}" -f $count, ($bytes / 1GB), $(if ($Apply) { 'Deleted.' } else { 'Preview only; add -Apply.' }))
