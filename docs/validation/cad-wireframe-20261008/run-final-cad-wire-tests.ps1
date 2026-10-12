$ErrorActionPreference='Stop'
$env:TEMP='H:\FreeCAD-src\build\3dm-preservation-sdk\test-temp';$env:TMP=$env:TEMP
$taskReports=@()
foreach($taskMacro in @('cad_wireframe_smoke.FCMacro','viewport_title_smoke.FCMacro','three_dm_preservation_ring_smoke.FCMacro','three_dm_native_action_crash_smoke.FCMacro','core_workspace_lifecycle_smoke.FCMacro','cad_external_link_smoke.FCMacro','cad_geometry_density_smoke.FCMacro','cad_nested_external_link_smoke.FCMacro')){
    $taskOutput=& 'H:\FreeCAD-src\build\om9-dev\tests\run_menu_smoke.ps1' -FreeCADExe 'H:\FreeCAD-src\build\om9-cad-wire-sdk\bin\FreeCAD.exe' -DependencyPrefix 'H:\FreeCAD-src\.pixi\envs\default\Library' -Macro $taskMacro -TimeoutSeconds 300
    $taskText=$taskOutput | Out-String
    $taskText | Set-Content -LiteralPath ('H:\FreeCAD-src\build\cad-final-'+$taskMacro+'.log')
    $taskPath=[regex]::Match($taskText,'Artifacts:\s*([^\r\n]+)').Groups[1].Value.Trim()
    $taskReport=Get-Content -LiteralPath (Join-Path $taskPath 'results.json') -Raw | ConvertFrom-Json
    if(-not $taskReport.ok){throw "FAILED $taskMacro"}
    $taskReports+=@{macro=$taskMacro;path=$taskPath;checks=$taskReport.checks.Count}
    $taskReports | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath 'H:\FreeCAD-src\build\cad-final-reports.json'
    Write-Output "PASS $taskMacro $($taskReport.checks.Count) checks [$taskPath]"
}
