# Exercise the actual report-reader functions without launching applications.
$ErrorActionPreference='Stop'
$root=Split-Path $PSScriptRoot -Parent
$runtime='H:/FreeCAD-src/build/om9-perf-sdk'
$expectedManifest='5d06e5d1fd9070a14afb69800aa53f99c6001ddc1dc4bc159959f3e7476eac6d'
$tokens=$null;$errors=$null
$ast=[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot 'run_rhino5_phase12_verify.ps1'),[ref]$tokens,[ref]$errors)
if($errors.Count){throw 'Report helper does not parse'}
foreach($function in $ast.FindAll({param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst]},$false)){
    Invoke-Expression $function.Extent.Text
}
$accepted=Get-Content -LiteralPath (Join-Path $root 'docs/validation/phase12-performance/summary.json') -Raw | ConvertFrom-Json
$geometry=$accepted.host_reports | Where-Object {[IO.Path]::GetFileName($_.macro) -eq 'modeling_perf_json_ab.FCMacro'} | Select-Object -First 1
$task=Join-Path "$root/build/phase12-report-reader" ([guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $task -Force | Out-Null
$results=@()
function Case([string]$name,[bool]$mustPass,[scriptblock]$change,[string]$source=$geometry.report){
    $data=Get-Content -LiteralPath $source -Raw | ConvertFrom-Json
    & $change $data
    $path=Join-Path $task ($name+'.json')
    $data | ConvertTo-Json -Depth 100 | Set-Content -LiteralPath $path -Encoding utf8
    $script:summary=@{module_sha256=$accepted.module_sha256;host_reports=@();host_checks=0;host_report_count=0}
    $succeeded=$false;$errorText=$null
    try{$null=Bound-Host $path;$succeeded=$true}catch{$errorText=$_ | Out-String}
    if($succeeded -ne $mustPass){throw "Unexpected report-reader outcome: $name ($errorText)"}
    if(-not $mustPass -and $script:summary.host_report_count -ne 0){throw "Rejected report was counted: $name"}
    $script:results+=@{name=$name;passed=$true;accepted=$succeeded;error=$errorText}
}
Case 'valid-current-geometry' $true {}
Case 'failed-check' $false {param($d) $d.checks[0].passed=$false}
Case 'empty-checks' $false {param($d) $d.checks=@()}
Case 'wrong-manifest' $false {param($d) $d.phase2_binding.manifest_sha256='wrong'}
Case 'wrong-module' $false {param($d) $d.module_sha256='wrong'}
Case 'missing-geometry-module-observation' $false {param($d) $d.PSObject.Properties.Remove('module_sha256')}
Case 'nonzero-process-exit' $false {param($d) $d.phase2_binding.process_exit_code=1}
Case 'wrong-executable' $false {param($d) $d.phase2_binding.executable='H:/FreeCAD-src/build/om9-phase2-rust-sdk/bin/FreeCAD.exe'}
Case 'invalid-owned-pid' $false {param($d) $d.phase2_binding.owned_pid=0}
Case 'changed-macro' $false {param($d) $d.phase2_binding.macro_sha256='wrong'}
foreach($name in @('core_end_snap_smoke.FCMacro','core_point_snap_smoke.FCMacro')){
    $row=$accepted.host_reports | Where-Object {[IO.Path]::GetFileName($_.macro) -eq $name} | Select-Object -First 1
    Case ($name+'-scoped-binding') $true {} $row.report
    if($script:summary.host_reports[0].loaded_module_reported){throw 'UI-only report was upgraded to loaded-module observation'}
}
@{ok=$true;checks=$results} | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $task 'results.json') -Encoding utf8
Write-Output "Report-reader PASS: $($results.Count) checks; $task/results.json"
