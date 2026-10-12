param([Parameter(Mandatory)][string]$OutputDirectory)
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$runtime='H:/FreeCAD-src/build/om9-phase3-sdk'
$python='H:/FreeCAD-src/.pixi/envs/default/python.exe'
$manifest=Join-Path $runtime 'phase3-build.json'
$binder=Join-Path $root 'tools/phase3_evidence.py'
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$summary=[ordered]@{ok=$false;scope='Phase3 BRep modeling application gate';reports=@();checks=0;rhino_checks=0;logs=$OutputDirectory}
function Digest([string]$path){(Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()}
function Stage([string]$value){@{stage=$value;utc=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $OutputDirectory 'progress.json') -Encoding utf8}
function Verify(){& rtk proxy $python $binder verify $manifest;if($LASTEXITCODE -ne 0){throw 'Source/runtime verification failed'}}
function RunHost([string]$macro){
    Stage $macro
    $log=Join-Path $OutputDirectory ($macro+'.log')
    & rtk proxy pwsh -NoProfile -File (Join-Path $PSScriptRoot 'run_menu_smoke.ps1') -FreeCADExe (Join-Path $runtime 'bin/FreeCAD.exe') -DependencyPrefix 'H:/FreeCAD-src/.pixi/envs/default/Library' -Macro $macro -TimeoutSeconds 300 *> $log
    $code=$LASTEXITCODE
    $match=[regex]::Match((Get-Content -LiteralPath $log -Raw),'Artifacts: ([^\r\n]+)')
    if(-not $match.Success){throw "No exact host report: $log"}
    $path=Join-Path $match.Groups[1].Value.Trim() 'results.json'
    if($code -ne 0){throw "Host failed: $path; see $log"}
    $data=Get-Content -LiteralPath $path -Raw|ConvertFrom-Json
    if(-not $data.ok -or -not $data.checks.Count -or @($data.checks|Where-Object {-not $_.passed}).Count){throw "Invalid host checks: $path"}
    if($data.phase3_binding.owned_pid -le 0 -or $data.phase3_binding.process_exit_code -ne 0 -or $data.phase3_binding.manifest_sha256 -ne $summary.manifest_sha256){throw "Host binding mismatch: $path"}
    $summary.reports+=@{path=$path;sha256=(Digest $path);macro=$macro;checks=$data.checks.Count;pid=$data.phase3_binding.owned_pid}
    $summary.checks+=$data.checks.Count
    return @{path=$path;data=$data}
}
$old=@{}
foreach($name in @('OM9_PHASE3_MANIFEST','OM9_PHASE2_MANIFEST','OM9_PHASE3_RING_REPORT','OM9_PHASE3_RHINO_OUTPUT','OM9_PHASE3_CASES','OM9_PHASE3_RHINO_REPORT','TEMP','TMP','__COMPAT_LAYER')){$old[$name]=[Environment]::GetEnvironmentVariable($name,'Process')}
try{
    Stage 'Verify Phase3 source/runtime'
    Verify
    $summary.manifest_sha256=Digest $manifest
    $summary.module_sha256=Digest (Join-Path $runtime 'bin/OpenMatrix9Gui.pyd')
    $scripts=@{}
    foreach($folder in @($PSScriptRoot,(Join-Path $root 'tools'))){foreach($file in Get-ChildItem -LiteralPath $folder -File){if($file.Extension -in @('.py','.ps1','.FCMacro')){$scripts[$file.FullName]=Digest $file.FullName}}}
    $oracle='H:/FreeCAD-src/build/om9-phase3-native/ThreeDmModelingBrepTests.exe';$scripts[$oracle]=Digest $oracle
    $env:OM9_PHASE3_MANIFEST=$manifest;$env:OM9_PHASE2_MANIFEST=$null
    $env:TEMP=Join-Path $OutputDirectory 'temp';$env:TMP=$env:TEMP;New-Item -ItemType Directory -Path $env:TEMP -Force|Out-Null
    $brep=RunHost 'modeling_brep_smoke.FCMacro'
    $surface=RunHost 'modeling_surface_edit_smoke.FCMacro'
    $null=RunHost 'modeling_phase3_lifecycle_smoke.FCMacro'
    $null=RunHost 'modeling_phase3_review_smoke.FCMacro'
    $supplement=RunHost 'modeling_phase3_singular_smoke.FCMacro'
    $ring=RunHost 'modeling_ring_workflow_smoke.FCMacro'
    $env:OM9_PHASE3_RING_REPORT=$ring.path
    $fresh=RunHost 'modeling_phase3_saved_reopen.FCMacro'
    $cases=@()
    foreach($entry in $brep.data.fixtures){$expected=@{object_name=$entry.object_name;area=$entry.area;volume=$entry.volume;solids=[int]$entry.closed;faces=$entry.faces;bounds=$entry.bounds};$cases+=@{name=$entry.name;input=$entry.output;sha256=$entry.output_sha256;expected=@($expected)}}
    # Bind analytic fixture bounds from independent dimensions, never copy Rhino output.
    $bounds=@{'cylinder-seam'=@(-3,-3,0,3,3,7);'sphere-poles'=@(-5,-5,-5,5,5,5);'torus'=@(-12,-12,-2,12,12,2);'cavity'=@(-5,-5,-5,5,5,5);'open-cylinder-face'=@(-3,-3,0,3,3,7);'planar-hole'=@(0,0,0,10,10,0);'edited-cylinder-knots'=@(-6,-6,0,6,6,14);'open-cylinder-shell'=@(-3,-3,0,3,3,7)}
    foreach($entry in $cases){$entry.expected[0].bounds=$bounds[$entry.name]}
    $cases+=@{name='ring-modified-and-new';input=$fresh.data.output;sha256=(Digest $fresh.data.output);expected=@($fresh.data.expected)}
    $cases+=@($surface.data.exchange_cases)+@($supplement.data.exchange_cases)
    if($cases.Count -ne 15){throw 'Prepared Phase3 matrix must contain 15 named fixtures'}
    $caseFile=Join-Path $OutputDirectory 'rhino-cases.json';@{cases=$cases}|ConvertTo-Json -Depth 40|Set-Content -LiteralPath $caseFile -Encoding utf8
    Stage 'Rhino5 Open / SaveAs / Export Selected'
    $rhinoOut=Join-Path $OutputDirectory 'rhino';New-Item -ItemType Directory -Path $rhinoOut -Force|Out-Null
    $env:OM9_PHASE3_RHINO_OUTPUT=$rhinoOut;$env:OM9_PHASE3_CASES=$caseFile;$env:__COMPAT_LAYER='RunAsInvoker DPIUNAWARE'
    $exe='C:/Program Files (x86)/Rhinoceros 5/System/Rhino4.exe';$script=Join-Path $PSScriptRoot 'rhino5_phase3_roundtrip.py'
    $arguments='/nosplash /notemplate /scheme=OM9Phase3_'+[guid]::NewGuid().ToString('N')+' /runscript="_-RunPythonScript '+$script+'"'
    $process=Start-Process -FilePath $exe -ArgumentList $arguments -WindowStyle Hidden -PassThru
    $finished=$process.WaitForExit(600000)
    @{pid=$process.Id;executable=$exe;sha256=(Digest $exe);finished=$finished;exit_code=if($finished){$process.ExitCode}else{$null}}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $rhinoOut 'launch.json') -Encoding utf8
    if(-not $finished){throw "Owned Rhino timed out, PID $($process.Id)"}
    if($process.ExitCode -ne 0){throw 'Owned Rhino nonzero exit'}
    $path=Join-Path $rhinoOut 'rhino5-results.json';$rhino=Get-Content -LiteralPath $path -Raw|ConvertFrom-Json
    if(-not $rhino.ok -or $rhino.pid -ne $process.Id -or -not $rhino.rhino_version.StartsWith('5.') -or $rhino.cases.Count -ne 15){throw "Rhino gate failed: $path $($rhino.error)"}
    $summary.rhino_report=$path;$summary.rhino_report_sha256=Digest $path;$summary.rhino_checks=$rhino.checks.Count;$summary.fixtures=15
    $env:OM9_PHASE3_RHINO_REPORT=$path
    $reread=RunHost 'modeling_phase3_rhino_reimport.FCMacro'
    foreach($macro in @('modeling_curve_join_smoke.FCMacro','modeling_curve_editor_ui_smoke.FCMacro','modeling_snap_cache_smoke.FCMacro','modeling_clipboard_user_smoke.FCMacro','modeling_clipboard_failure_smoke.FCMacro','modeling_workers_smoke.FCMacro')){$null=RunHost $macro}
    foreach($name in $scripts.Keys){if((Digest $name) -ne $scripts[$name]){throw "Script/oracle changed during run: $name"}}
    foreach($entry in $summary.reports){if((Digest $entry.path) -ne $entry.sha256){throw 'Host report changed during run'}}
    Verify
    $summary.test_script_sha256=$scripts;$summary.ok=$true;Stage 'PASS'
}catch{$summary.error=($_|Out-String);Stage 'FAIL - inspect phase3-verification.json'}
finally{
    foreach($name in $old.Keys){[Environment]::SetEnvironmentVariable($name,$old[$name],'Process')}
    $summary|ConvertTo-Json -Depth 60|Set-Content -LiteralPath (Join-Path $OutputDirectory 'phase3-verification.json') -Encoding utf8
}
Write-Output (Join-Path $OutputDirectory 'phase3-verification.json')
if(-not $summary.ok){exit 1}
