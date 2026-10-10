param([int]$TimeoutSeconds=600,[string]$FixtureFilter='')
$ErrorActionPreference='Stop'
$manifest='H:/FreeCAD-src/build/om9-perf-sdk/phase2-build.json';$binder='H:/FreeCAD-src/build/om9-perf-dev/tools/phase2_evidence.py'
& rtk proxy python $binder verify $manifest
if($LASTEXITCODE -ne 0){throw 'Rhino phase2 runtime binding failed'}
$task=Join-Path 'H:/FreeCAD-src/build/rhino5-phase12-user-clipboard' ([guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $task -Force | Out-Null
$previousRuntime=$env:OM9_RHINO_CLIPBOARD_RUNTIME;$previousOutput=$env:OM9_RHINO_CLIPBOARD_OUTPUT;$previousCompat=$env:__COMPAT_LAYER;$previousFilter=$env:OM9_RHINO_CLIPBOARD_CASE_FILTER
try {
 $env:OM9_RHINO_CLIPBOARD_RUNTIME='H:/FreeCAD-src/build/om9-perf-sdk';$env:OM9_RHINO_CLIPBOARD_OUTPUT=$task;$env:__COMPAT_LAYER='RunAsInvoker DPIUNAWARE'
 $env:OM9_RHINO_CLIPBOARD_CASE_FILTER=$FixtureFilter
 $exe='C:/Program Files (x86)/Rhinoceros 5/System/Rhino4.exe'
 $scheme='OM9UserClipboardOwned_'+[IO.Path]::GetFileName($task)
 $arguments='/nosplash /notemplate /scheme='+$scheme+' /runscript="_-RunPythonScript H:/FreeCAD-src/build/om9-perf-dev/tests/rhino5_clipboard_user_bootstrap.py"'
 $process=Start-Process -FilePath $exe -ArgumentList $arguments -WindowStyle Hidden -PassThru
 [IO.File]::WriteAllText((Join-Path $task 'owned-pid.txt'),[string]$process.Id)
 Write-Output "Owned Rhino5 Phase2 PID $($process.Id); artifacts: $task"
 $finished=$process.WaitForExit($TimeoutSeconds*1000)
 @{pid=$process.Id;exe=$exe;finished=$finished;exit_code=if($finished){$process.ExitCode}else{$null};report_exists=(Test-Path -LiteralPath (Join-Path $task 'rhino5-results.json'))} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $task 'launch.json')
 if(-not $finished){throw 'Owned Rhino Phase2 gate timed out'}
 $result=Get-Content -LiteralPath (Join-Path $task 'rhino5-results.json') -Raw | ConvertFrom-Json
 Write-Output "Rhino5 Phase2: ok=$($result.ok); cases=$($result.cases.Count); checks=$($result.checks.Count); exit=$($process.ExitCode)"
 if(-not $result.ok){Write-Output $result.error;throw 'Actual Rhino Phase2 gate failed'}
 if($process.ExitCode -ne 0){throw 'Owned Rhino nonzero exit'}
 foreach($case in $result.cases){
  & rtk proxy python $binder bind $manifest --report $case.freecad_report --macro 'H:/FreeCAD-src/build/om9-perf-dev/tests/modeling_clipboard_rhino_user_roundtrip.FCMacro' --exe 'H:/FreeCAD-src/build/om9-perf-sdk/bin/FreeCAD.exe' --pid 0 --exit-code 0
  if($LASTEXITCODE -ne 0){throw 'Rhino host report binding failed'}
 }
 $result | Add-Member -NotePropertyName phase2_binding -NotePropertyValue @{manifest=$manifest;manifest_sha256=(Get-FileHash -LiteralPath $manifest -Algorithm SHA256).Hash.ToLowerInvariant();rhino_exe=$exe;rhino_exe_sha256=(Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash.ToLowerInvariant();rhino_pid=$process.Id;exit_code=$process.ExitCode;script_sha256=(Get-FileHash -LiteralPath 'H:/FreeCAD-src/build/om9-perf-dev/tests/rhino5_clipboard_user.py' -Algorithm SHA256).Hash.ToLowerInvariant()}
 $result | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath (Join-Path $task 'rhino5-results.json')
}finally{$env:OM9_RHINO_CLIPBOARD_RUNTIME=$previousRuntime;$env:OM9_RHINO_CLIPBOARD_OUTPUT=$previousOutput;$env:__COMPAT_LAYER=$previousCompat;$env:OM9_RHINO_CLIPBOARD_CASE_FILTER=$previousFilter}
