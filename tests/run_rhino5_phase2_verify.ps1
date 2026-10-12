# User entrypoint's child process. Reuses the accepted gates without changing them.
param(
    [Parameter(Mandatory)][string]$OutputDirectory,
    [ValidateRange(1,600)][int]$RhinoTimeoutSeconds=600
)
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$runtime='H:/FreeCAD-src/build/om9-phase2-rust-sdk'
$manifest=Join-Path $runtime 'phase2-build.json'
$launcher='H:/FreeCAD-src/build/run-rhino5-phase2-rust.ps1'
$summaryPath=Join-Path $OutputDirectory 'phase2-verification.json'
$progressPath=Join-Path $OutputDirectory 'progress.json'
$summary=[ordered]@{ok=$false;scope='phase2-rust-rhino-roundtrip';started_utc=[DateTime]::UtcNow.ToString('o');runtime=$runtime;logs=$OutputDirectory}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
function Set-Stage([string]$stage) {
    @{stage=$stage;utc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content -LiteralPath $progressPath -Encoding utf8
}
function Assert-Report($data,[string]$label) {
    if(-not $data.ok -or $data.checks.Count -eq 0 -or @($data.checks | Where-Object {-not $_.passed}).Count){throw "$label contains failed or missing checks"}
}
$oldManifest=$env:OM9_PHASE2_MANIFEST
$oldReport=$env:OM9_RHINO_PHASE2_RESULT
try {
    Set-Stage 'Verifying Rust runtime and starting isolated Rhino 5'
    foreach($path in @($manifest,$launcher,(Join-Path $runtime 'bin/FreeCAD.exe'),(Join-Path $PSScriptRoot 'run_menu_smoke.ps1'))){
        if(-not (Test-Path -LiteralPath $path -PathType Leaf)){throw "Missing required file: $path"}
    }
    $manifestSha=(Get-FileHash -LiteralPath $manifest -Algorithm SHA256).Hash.ToLowerInvariant()
    $summary.manifest_sha256=$manifestSha
    $rhinoLog=Join-Path $OutputDirectory 'rhino-launch.log'
    & $launcher -TimeoutSeconds $RhinoTimeoutSeconds *> $rhinoLog
    # Select the exact run emitted by this launch; never search for the latest report.
    $match=[regex]::Match((Get-Content -LiteralPath $rhinoLog -Raw),'artifacts: ([^\r\n]+)')
    if(-not $match.Success){throw "Rhino launch did not provide its output directory; see $rhinoLog"}
    $rhinoReport=Join-Path $match.Groups[1].Value.Trim() 'rhino5-results.json'
    $summary.rhino_report=$rhinoReport
    $rhino=Get-Content -LiteralPath $rhinoReport -Raw | ConvertFrom-Json
    Assert-Report $rhino 'Rhino 5'
    if($rhino.phase2_binding.manifest_sha256 -ne $manifestSha -or $rhino.phase2_binding.exit_code -ne 0){throw 'Rhino report runtime binding or process exit mismatch'}
    $names=@($rhino.cases | ForEach-Object {$_.name} | Sort-Object)
    if(($names -join ',') -ne 'periodic,placed,rational'){throw 'Rhino fixture set differs from the Phase 2 contract'}
    $summary.fixtures=$names
    $summary.rhino_version=$rhino.rhino_version
    $summary.rhino_checks=$rhino.checks.Count
    $summary.host_checks=0
    foreach($case in $rhino.cases){
        $hostResult=Get-Content -LiteralPath $case.host_report -Raw | ConvertFrom-Json
        Assert-Report $hostResult ('OM9 '+$case.name)
        if($hostResult.phase2_binding.manifest_sha256 -ne $manifestSha){throw 'OM9 host report runtime mismatch'}
        $summary.host_checks += $hostResult.checks.Count
        $summary.module_sha256=$hostResult.module_sha256
    }
    Set-Stage 'Rhino roundtrip passed; rereading this run''s SaveAs files in OM9'
    $env:OM9_PHASE2_MANIFEST=$manifest
    $env:OM9_RHINO_PHASE2_RESULT=$rhinoReport
    $rereadLog=Join-Path $OutputDirectory 'saved-reread.log'
    & (Join-Path $PSScriptRoot 'run_menu_smoke.ps1') -FreeCADExe (Join-Path $runtime 'bin/FreeCAD.exe') -DependencyPrefix 'H:/FreeCAD-src/.pixi/envs/default/Library' -Macro modeling_phase2_saved_reimport.FCMacro -TimeoutSeconds 180 *> $rereadLog
    $match=[regex]::Match((Get-Content -LiteralPath $rereadLog -Raw),'Artifacts: ([^\r\n]+)')
    if(-not $match.Success){throw "Saved reread did not provide its output directory; see $rereadLog"}
    $rereadReport=Join-Path $match.Groups[1].Value.Trim() 'results.json'
    $summary.saved_reread_report=$rereadReport
    $reread=Get-Content -LiteralPath $rereadReport -Raw | ConvertFrom-Json
    Assert-Report $reread 'Saved reread'
    if($reread.phase2_binding.manifest_sha256 -ne $manifestSha -or $reread.module_sha256 -ne $summary.module_sha256){throw 'Saved reread runtime mismatch'}
    if($reread.rhino_report_sha256 -ne (Get-FileHash -LiteralPath $rhinoReport -Algorithm SHA256).Hash.ToLowerInvariant()){throw 'Saved reread consumed a different Rhino report'}
    if((Get-FileHash -LiteralPath $manifest -Algorithm SHA256).Hash.ToLowerInvariant() -ne $manifestSha){throw 'Runtime manifest changed while verifying'}
    $summary.saved_reread_checks=$reread.checks.Count
    $summary.freecad_version=$reread.freecad_version
    $summary.ok=$true
    Set-Stage 'PASS'
} catch {
    $summary.error=($_ | Out-String)
    Set-Stage 'FAIL - see phase2-verification.json and launch logs'
} finally {
    $env:OM9_PHASE2_MANIFEST=$oldManifest
    $env:OM9_RHINO_PHASE2_RESULT=$oldReport
    $summary.finished_utc=[DateTime]::UtcNow.ToString('o')
    $summary | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $summaryPath -Encoding utf8
}
Write-Output $summaryPath
if(-not $summary.ok){exit 1}
