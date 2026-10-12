# Test orchestration only; all geometry/editor/cache ownership stays in the bound runtime.
param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$runtime='H:/FreeCAD-src/build/om9-perf-sdk'
$manifest=Join-Path $runtime 'phase2-build.json'
$expectedManifest='5d06e5d1fd9070a14afb69800aa53f99c6001ddc1dc4bc159959f3e7476eac6d'
$binder=Join-Path $root 'tools/phase2_evidence.py'
$summaryPath=Join-Path $OutputDirectory 'phase12-verification.json'
$progressPath=Join-Path $OutputDirectory 'progress.json'
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$timingDirectory=Join-Path $OutputDirectory 'timings'
New-Item -ItemType Directory -Path $timingDirectory -Force | Out-Null
$summary=[ordered]@{ok=$false;scope='optimized-phase1-phase2-application-gates';runtime=$runtime;started_utc=[DateTime]::UtcNow.ToString('o');logs=$OutputDirectory;rhino_checks=0;host_checks=0;saved_reread_checks=0;host_report_count=0;host_reports=@();limitations=@('Scoped Phase1/2 fixtures; does not certify full openNURBS.','Warm performance gates do not measure cold index latency or prove whole-app speedup.')}
function Digest([string]$path){(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}
function Set-Stage([string]$stage){
    @{stage=$stage;utc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content -LiteralPath $progressPath -Encoding utf8
}
function Assert-Report($data,[string]$label){
    if(-not $data.ok -or $data.checks.Count -eq 0 -or @($data.checks | Where-Object {-not $_.passed}).Count){throw "$label has failed or missing checks"}
}
function Bound-Host([string]$path){
    $hostResult=Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
    Assert-Report $hostResult $path
    $binding=$hostResult.phase2_binding
    if($binding.manifest_sha256 -ne $expectedManifest -or $binding.process_exit_code -ne 0 -or $binding.owned_pid -le 0){throw "Host runtime/exit/PID binding mismatch: $path"}
    if(-not $binding.source_and_runtime_verified -or [IO.Path]::GetFullPath($binding.executable) -ne [IO.Path]::GetFullPath((Join-Path $runtime 'bin/FreeCAD.exe'))){throw "Wrong host binary: $path"}
    $moduleObserved=[bool]$hostResult.module_sha256
    if($moduleObserved){
        if($hostResult.module_sha256 -ne $summary.module_sha256){throw "Wrong loaded native module: $path"}
    }elseif([IO.Path]::GetFileName($binding.macro) -notin @('core_end_snap_smoke.FCMacro','core_point_snap_smoke.FCMacro')){
        throw "Missing native module observation: $path"
    }
    if((Digest $binding.macro) -ne $binding.macro_sha256){throw "Host macro changed: $path"}
    $summary.host_reports+=@{report=$path;report_sha256=(Digest $path);checks=$hostResult.checks.Count;macro=$binding.macro;pid=$binding.owned_pid;loaded_module_reported=$moduleObserved}
    $summary.host_checks+=$hostResult.checks.Count
    $summary.host_report_count++
    return $hostResult
}
function Run-Host([string]$macro,[string]$logName){
    $watch=[Diagnostics.Stopwatch]::StartNew()
    $log=Join-Path $OutputDirectory $logName
    & rtk proxy pwsh -NoProfile -File (Join-Path $PSScriptRoot 'run_menu_smoke.ps1') -FreeCADExe (Join-Path $runtime 'bin/FreeCAD.exe') -DependencyPrefix 'H:/FreeCAD-src/.pixi/envs/default/Library' -Macro $macro -TimeoutSeconds 300 *> $log
    $hostExit=$LASTEXITCODE
    $match=[regex]::Match((Get-Content -LiteralPath $log -Raw),'Artifacts: ([^\r\n]+)')
    if(-not $match.Success){throw "No exact host output directory: $log"}
    $path=Join-Path $match.Groups[1].Value.Trim() 'results.json'
    if($hostExit -ne 0){
        $failed=Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
        $summary.failed_host_report=$path
        $summary.failed_host_report_sha256=Digest $path
        $summary.failure_details=@{macro=$macro;exit_code=$hostExit;error=$failed.error;shortcut=$failed.shortcut;fixture_measurements=$failed.fixture_measurements}
        throw "Host process failed: $macro; see $path and $log. $($failed.error)"
    }
    $hostResult=Bound-Host $path
    $watch.Stop()
    $summary.application_session_timings+=@{macro=$macro;seconds=$watch.Elapsed.TotalSeconds;scope='startup + complete macro + validation + exit + evidence binding; not pure Copy/Paste'}
    return @{path=$path;data=$hostResult}
}
function Run-Rhino([string]$mode){
    Set-Stage "Rhino 5 ${mode}: isolated application roundtrip"
    $log=Join-Path $OutputDirectory ("rhino-$mode.log")
    $launcher=if($mode -eq 'clipboard'){Join-Path $PSScriptRoot 'run_rhino5_phase12_clipboard.ps1'}else{'H:/FreeCAD-src/build/run-rhino5-phase12-perf-phase2.ps1'}
    & rtk proxy pwsh -NoProfile -File $launcher -TimeoutSeconds 600 *> $log
    $launchCode=$LASTEXITCODE
    $match=[regex]::Match((Get-Content -LiteralPath $log -Raw),'artifacts: ([^\r\n]+)')
    if(-not $match.Success){throw "No exact Rhino output directory: $log"}
    $path=Join-Path $match.Groups[1].Value.Trim() 'rhino5-results.json'
    $rhino=Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
    if($launchCode -ne 0 -or -not $rhino.ok){
        $summary.failed_rhino_report=$path
        $summary.failed_rhino_report_sha256=Digest $path
        $summary.failure_details=@{mode=$mode;error=$rhino.error;clipboard_state=$rhino.clipboard_failure_state}
        throw "Rhino $mode failed; see $path and $log"
    }
    Assert-Report $rhino "Rhino $mode"
    $binding=$rhino.phase2_binding
    if(-not $rhino.rhino_version.StartsWith('5.') -or $binding.manifest_sha256 -ne $expectedManifest -or $binding.exit_code -ne 0 -or $binding.rhino_pid -ne $rhino.pid){throw "Rhino $mode version/runtime/PID/exit mismatch"}
    $launch=Get-Content -LiteralPath (Join-Path (Split-Path $path -Parent) 'launch.json') -Raw | ConvertFrom-Json
    if(-not $launch.finished -or $launch.exit_code -ne 0 -or $launch.pid -ne $rhino.pid){throw "Rhino $mode launch evidence mismatch"}
    $names=@($rhino.cases | ForEach-Object {$_.name} | Sort-Object)
    if($mode -eq 'phase2'){$expected=@('periodic','placed','rational')}
    else{
        $accepted=Get-Content -LiteralPath (Join-Path $root 'docs/validation/modeling-phase-1/application-evidence.json') -Raw | ConvertFrom-Json
        $expected=@(@($accepted.accepted_case_bindings | ForEach-Object {$_.name})+'native-cm' | Sort-Object)
    }
    if(($names -join ',') -ne ($expected -join ',')){throw "Rhino $mode fixture set mismatch"}
    $phase=[ordered]@{fixtures=$names;rhino_report=$path;rhino_report_sha256=(Digest $path);rhino_checks=$rhino.checks.Count;host_checks=0;rhino_pid=$rhino.pid;rhino_version=$rhino.rhino_version}
    $phase.copy_paste_timings=@()
    foreach($case in $rhino.cases){
        $hostPath=if($mode -eq 'phase2'){$case.host_report}else{$case.freecad_report}
        $child=Bound-Host $hostPath
        $phase.host_checks+=$child.checks.Count
        if($mode -eq 'clipboard'){
            $phase.copy_paste_timings+=@{fixture=$case.name;rhino_timings=@($case.timings);om9_timings=@($child.timings);objects=$child.expected.Count;payload_bytes=(Get-Item -LiteralPath $child.payload_file).Length;timing_log_errors=@($case.timing_log_errors)+@($child.timing_log_errors)}
        }
    }
    $summary.rhino_checks+=$rhino.checks.Count
    Set-Stage "Rhino $mode passed: rereading this run's SaveAs files in OM9"
    if($mode -eq 'phase2'){$env:OM9_RHINO_PHASE2_RESULT=$path;$macro='modeling_phase2_saved_reimport.FCMacro'}
    else{$env:OM9_RHINO_CLIPBOARD_REPORT=$path;$macro='modeling_clipboard_saved_reimport.FCMacro'}
    $reread=Run-Host $macro "saved-$mode.log"
    if($reread.data.cases.Count -ne $names.Count){throw "Saved $mode fixture count mismatch"}
    if($mode -eq 'phase2' -and $reread.data.rhino_report_sha256 -ne (Digest $path)){throw 'Phase2 reread consumed a different Rhino report'}
    if([IO.Path]::GetFullPath($reread.data.rhino_report) -ne [IO.Path]::GetFullPath($path)){throw "Saved $mode consumed a different run"}
    $phase.saved_reread_report=$reread.path;$phase.saved_reread_checks=$reread.data.checks.Count
    $summary.saved_reread_checks+=$reread.data.checks.Count
    return $phase
}
$oldVars=@{}
foreach($name in @('OM9_PHASE2_MANIFEST','OM9_RHINO_PHASE2_RESULT','OM9_RHINO_CLIPBOARD_REPORT','OM9_MODELING_WORKERS','OM9_PHASE12_TIMING_DIRECTORY')){$oldVars[$name]=[Environment]::GetEnvironmentVariable($name,'Process')}
try{
    Set-Stage 'Verifying optimized source and runtime hashes'
    if((Digest $manifest) -ne $expectedManifest){throw 'Runtime manifest differs from the optimized build approved by these tests'}
    & rtk proxy python $binder verify $manifest
    if($LASTEXITCODE -ne 0){throw 'Source/runtime verification failed'}
    $summary.manifest_sha256=$expectedManifest
    $summary.module_sha256=Digest (Join-Path $runtime 'bin/OpenMatrix9Gui.pyd')
    $env:OM9_PHASE2_MANIFEST=$manifest
    $env:OM9_MODELING_WORKERS=$null
    $env:OM9_PHASE12_TIMING_DIRECTORY=$timingDirectory
    $summary.application_session_timings=@()
    $scripts=@{}
    foreach($file in (Get-ChildItem -LiteralPath $PSScriptRoot -File | Where-Object {$_.Extension -in @('.py','.ps1','.FCMacro')})){$scripts[$file.FullName]=Digest $file.FullName}
    $file='H:/FreeCAD-src/build/run-rhino5-phase12-perf-phase2.ps1';$scripts[$file]=Digest $file
    $summary.phase2=Run-Rhino 'phase2'
    $summary.phase1=Run-Rhino 'clipboard'
    $groups=[ordered]@{
        'Copy/Paste rollback, cache, worker60%, curve workflow'=@('modeling_clipboard_user_smoke.FCMacro','modeling_clipboard_failure_smoke.FCMacro','modeling_snap_cache_smoke.FCMacro','modeling_snap_smoke.FCMacro','modeling_workers_smoke.FCMacro','modeling_curve_workflow_smoke.FCMacro')
        'Curve editor, topology, Rust ownership and exchange regression'=@('modeling_curve_topology_smoke.FCMacro','modeling_curve_editor_smoke.FCMacro','modeling_curve_editor_ui_smoke.FCMacro','modeling_curve_join_smoke.FCMacro','modeling_rust_ownership_smoke.FCMacro','modeling_rust_snap_ownership_smoke.FCMacro','modeling_rust_rebuild_ownership_smoke.FCMacro','modeling_rust_context_smoke.FCMacro','modeling_rust_domain_smoke.FCMacro','modeling_exchange_smoke.FCMacro','modeling_ring_roundtrip_smoke.FCMacro','core_end_snap_smoke.FCMacro','core_point_snap_smoke.FCMacro')
        'Native JSON and actual Copy/Paste alternating A/B'=@('modeling_perf_json_ab.FCMacro')
        'Warm snap:10k CAD and million-point mesh/cloud'=@('modeling_snap_performance_smoke.FCMacro','modeling_snap_heavy_smoke.FCMacro')
    }
    $summary.application_gates=@()
    foreach($group in $groups.GetEnumerator()){
        foreach($macro in $group.Value){
            Set-Stage ($group.Key+': '+$macro)
            $result=Run-Host $macro ($macro+'.log')
            $summary.application_gates+=@{macro=$macro;report=$result.path;checks=$result.data.checks.Count}
        }
    }
    if($summary.host_report_count -ne 38){throw 'Incomplete application report set'}
    if(($summary.host_reports.report | Sort-Object -Unique).Count -ne 38){throw 'Duplicate application report supplied'}
    foreach($entry in $summary.host_reports){if((Digest $entry.report) -ne $entry.report_sha256){throw 'Host report changed during verification'}}
    foreach($file in $scripts.Keys){if((Digest $file) -ne $scripts[$file]){throw "Test script changed while running: $file"}}
    if((Digest $manifest) -ne $expectedManifest){throw 'Manifest changed while verifying'}
    & rtk proxy python $binder verify $manifest
    if($LASTEXITCODE -ne 0){throw 'Final source/runtime verification failed'}
    $summary.test_script_sha256=$scripts
    $summary.ok=$true
    Set-Stage 'PASS'
}catch{
    $summary.error=($_ | Out-String)
    Set-Stage 'FAIL - inspect phase12-verification.json and stage log'
}finally{
    foreach($name in $oldVars.Keys){[Environment]::SetEnvironmentVariable($name,$oldVars[$name],'Process')}
    $summary.finished_utc=[DateTime]::UtcNow.ToString('o')
    $summary.timing_log_errors=@()
    $events=@(Get-ChildItem -LiteralPath $timingDirectory -Filter 'timing-*.jsonl' | ForEach-Object {Get-Content -LiteralPath $_.FullName | ForEach-Object {if($_){try{$_ | ConvertFrom-Json}catch{$summary.timing_log_errors+=($_ | Out-String)}}}})
    $summary.timing_summary=Join-Path $OutputDirectory 'timing-summary.json'
    @{events=$events;slowest_copy_paste=@($events | Where-Object {$_.stage -eq 'total' -and $_.operation -in @('Copy','Paste')} | Sort-Object seconds -Descending);session_timings=$summary.application_session_timings;scope='Elapsed wall time. Stages are inclusive and must not be summed; app sessions include setup/oracles and are separate from command totals.'} | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $summary.timing_summary -Encoding utf8
    $summary | ConvertTo-Json -Depth 60 | Set-Content -LiteralPath $summaryPath -Encoding utf8
}
Write-Output $summaryPath
if(-not $summary.ok){exit 1}
