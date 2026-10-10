param(
 [ValidateRange(1,5)][int]$Phase=1,
 [Parameter(Mandatory)][string]$FreeCADExe,
 [Parameter(Mandatory)][string]$DependencyPrefix,
 [string]$ResumeSummary
)
$ErrorActionPreference='Stop'
if($Phase -gt 3){throw "Phase $Phase has no completed runtime gate yet"}
if(-not [IO.Path]::IsPathRooted($FreeCADExe) -or -not [IO.Path]::IsPathRooted($DependencyPrefix)){throw 'Use absolute executable and dependency paths'}
if($Phase -eq 3){
 if($ResumeSummary){throw 'Phase3 requires a fresh complete application gate'}
 if([IO.Path]::GetFullPath($FreeCADExe) -ne [IO.Path]::GetFullPath('H:/FreeCAD-src/build/om9-phase3-sdk/bin/FreeCAD.exe')){throw 'Phase3 runner requires its manifest-bound executable'}
 if([IO.Path]::GetFullPath($DependencyPrefix) -ne [IO.Path]::GetFullPath('H:/FreeCAD-src/.pixi/envs/default/Library')){throw 'Phase3 runner requires its manifest-bound dependency prefix'}
 $phase3Output=Join-Path (Split-Path $PSScriptRoot -Parent) ('build/phase3-gate-'+[guid]::NewGuid().ToString('N'))
 & rtk proxy pwsh -NoProfile -File (Join-Path $PSScriptRoot 'run_rhino5_phase3_verify.ps1') -OutputDirectory $phase3Output
 exit $LASTEXITCODE
}
$runtime=Split-Path (Split-Path $FreeCADExe -Parent) -Parent
$manifest=Join-Path $runtime $(if($Phase -eq 2){'phase2-build.json'}else{'modeling-build.json'})
$data=Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json
foreach($entry in $data.source_sha256.PSObject.Properties){
 $source=Join-Path $data.source_root $entry.Name
 if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.Value){throw "Build source mismatch: $($entry.Name)"}
}
foreach($entry in $data.runtime_sha256.PSObject.Properties){
 if((Get-FileHash -LiteralPath (Join-Path $runtime $entry.Name) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.Value){throw "Runtime hash mismatch: $($entry.Name)"}
}
$oldManifest=$env:OM9_MODELING_MANIFEST
$oldPhase2Manifest=$env:OM9_PHASE2_MANIFEST
$resumeBatches=@()
if($ResumeSummary){
 if($Phase -ne 2){throw 'Only Phase2 has manifest-bound resume reports'}
 $resume=Get-Content -LiteralPath $ResumeSummary -Raw | ConvertFrom-Json
 foreach($batch in $resume.batches){
  $proof=Get-Content -LiteralPath $batch.report -Raw | ConvertFrom-Json
  if(-not $proof.ok -or -not $proof.phase2_binding.source_and_runtime_verified -or $proof.phase2_binding.manifest_sha256 -ne (Get-FileHash -LiteralPath $manifest -Algorithm SHA256).Hash.ToLowerInvariant() -or $proof.phase2_binding.macro_sha256 -ne (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $batch.macro) -Algorithm SHA256).Hash.ToLowerInvariant()){throw 'Cannot resume mismatching application evidence'}
  $resumeBatches+=@{macro=$batch.macro;status=$batch.status;checks=$batch.checks;report=$batch.report;log=$batch.log}
 }
}
try {
 $env:OM9_MODELING_MANIFEST=$manifest
 if($Phase -eq 2){$env:OM9_PHASE2_MANIFEST=$manifest}
 $macros=if($Phase -eq 2){@('modeling_curve_topology_smoke.FCMacro','modeling_snap_smoke.FCMacro','modeling_snap_cache_smoke.FCMacro','modeling_snap_performance_smoke.FCMacro','modeling_snap_heavy_smoke.FCMacro','modeling_curve_workflow_smoke.FCMacro','modeling_curve_editor_smoke.FCMacro','modeling_curve_editor_ui_smoke.FCMacro','modeling_curve_join_smoke.FCMacro','core_end_snap_smoke.FCMacro','core_point_snap_smoke.FCMacro','curve_smoke.FCMacro','modeling_clipboard_smoke.FCMacro','modeling_clipboard_failure_smoke.FCMacro','modeling_workers_smoke.FCMacro','modeling_exchange_smoke.FCMacro','modeling_primitives_smoke.FCMacro','modeling_ring_roundtrip_smoke.FCMacro','three_dm_import_workers_smoke.FCMacro')}else{@('modeling_baseline_smoke.FCMacro','modeling_exchange_smoke.FCMacro','modeling_workers_smoke.FCMacro','modeling_menu_smoke.FCMacro','three_dm_exchange_menu_smoke.FCMacro','three_dm_import_workers_smoke.FCMacro','modeling_primitives_smoke.FCMacro','modeling_ring_roundtrip_smoke.FCMacro')}
 foreach($macro in $macros){
  if(@($resumeBatches | ForEach-Object {$_.macro}) -contains $macro){continue}
  if($Phase -eq 2){
   if(-not $attempt){$evidenceFolder=if($data.migration -eq 'phase2-rust-owned-core'){'modeling-phase-2-rust'}else{'modeling-phase-2'};$attempt=Join-Path (Split-Path $PSScriptRoot -Parent) ('docs/validation/'+$evidenceFolder+'/runtime-attempts/'+[guid]::NewGuid().ToString('N'));New-Item -ItemType Directory -Path $attempt -Force | Out-Null;$batches=@($resumeBatches)}
   $log=Join-Path $attempt ($macro+'.log')
   try{
    & (Join-Path $PSScriptRoot 'run_menu_smoke.ps1') -FreeCADExe $FreeCADExe -DependencyPrefix $DependencyPrefix -Macro $macro -TimeoutSeconds 600 *> $log
    $match=[regex]::Match((Get-Content -LiteralPath $log -Raw),'Artifacts: (.+)')
    if(-not $match.Success){throw 'Missing owned artifact path'}
    $report=Join-Path $match.Groups[1].Value.Trim() 'results.json';$result=Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
    $batches+=@{macro=$macro;status='passed';checks=$result.checks.Count;report=$report;log=$log}
    Write-Output "Batch $macro PASS; checks=$($result.checks.Count); prepared host batches left=$($macros.Count-$batches.Count); full openNURBS packages open=7; total future batches unknown"
    @{status='in_progress';manifest=$manifest;batches=$batches} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $attempt 'summary.json')
   }catch{
    @{status='failed';macro=$macro;error=$_.Exception.Message;manifest=$manifest;batches=$batches;log=$log} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $attempt 'summary.json')
    Get-Content -LiteralPath $log -Tail 22;throw
   }
  }else{& (Join-Path $PSScriptRoot 'run_menu_smoke.ps1') -FreeCADExe $FreeCADExe -DependencyPrefix $DependencyPrefix -Macro $macro -TimeoutSeconds 600}
 }
 if($Phase -eq 2){@{status='passed';manifest=$manifest;batches=$batches} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $attempt 'summary.json');Write-Output "Phase2 host gate summary: $attempt/summary.json"}
} finally {$env:OM9_MODELING_MANIFEST=$oldManifest;$env:OM9_PHASE2_MANIFEST=$oldPhase2Manifest}
