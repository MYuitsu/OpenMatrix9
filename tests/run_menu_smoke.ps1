param(
    [string]$FreeCADExe = 'D:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe',
    [string]$DependencyPrefix = 'D:/FreeCAD-src/.pixi/envs/default/Library',
    [double]$Scale = 1,
    [string]$Macro = 'menu_smoke.FCMacro',
    [ValidateRange(1,600)][int]$TimeoutSeconds = 60,
    [switch]$KeepOpen
)
$ErrorActionPreference='Stop'
$moduleRoot=Split-Path $PSScriptRoot -Parent
$suite=if($Macro -eq 'menu_smoke.FCMacro'){'smoke'}else{[IO.Path]::GetFileNameWithoutExtension($Macro)}
$outputRoot=Join-Path $moduleRoot "build/$suite-$Scale/$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $outputRoot -Force | Out-Null
$oldVars=@{}
foreach($key in @('PATH','FREECAD_USER_HOME','OM9_SMOKE_OUTPUT','OM9_SMOKE_KEEP_OPEN','QT_SCALE_FACTOR')) {$oldVars[$key]=[Environment]::GetEnvironmentVariable($key,'Process')}
try {
    $phase3Evidence=Join-Path $moduleRoot 'tools/phase3_evidence.py'
    $python='H:/FreeCAD-src/.pixi/envs/default/python.exe'
    if($env:OM9_PHASE3_MANIFEST){& rtk proxy $python $phase3Evidence verify $env:OM9_PHASE3_MANIFEST;if($LASTEXITCODE -ne 0){throw 'Phase3 source/runtime binding failed before launch'}}
    $phase2Evidence=Join-Path $moduleRoot 'tools/phase2_evidence.py'
    if($env:OM9_PHASE2_MANIFEST){& rtk proxy python $phase2Evidence verify $env:OM9_PHASE2_MANIFEST;if($LASTEXITCODE -ne 0){throw 'Phase2 source/runtime binding failed before launch'}}
    $env:PATH="$DependencyPrefix/bin;$(Split-Path $DependencyPrefix -Parent);"+$env:PATH
    $env:FREECAD_USER_HOME=Join-Path $outputRoot 'profile'
    $env:OM9_SMOKE_OUTPUT=$outputRoot
    $env:OM9_SMOKE_KEEP_OPEN=if($KeepOpen){'1'}else{'0'}
    $env:QT_SCALE_FACTOR=$Scale.ToString([System.Globalization.CultureInfo]::InvariantCulture)
    $arguments=@('-u',"`"$(Join-Path $outputRoot 'user.cfg')`"",'-s',"`"$(Join-Path $outputRoot 'system.cfg')`"","`"$(Join-Path $PSScriptRoot $Macro)`"")
    $process=Start-Process -FilePath $FreeCADExe -ArgumentList $arguments -WindowStyle Hidden -PassThru
    if(-not $KeepOpen) {
        if(-not $process.WaitForExit($TimeoutSeconds*1000)){throw "Smoke process $($process.Id) did not finish within $TimeoutSeconds seconds; inspect its logs/window."}
        $file=Join-Path $outputRoot 'results.json'
        if(-not(Test-Path -LiteralPath $file)){throw "No smoke report. Process exit: $($process.ExitCode)"}
        $result=Get-Content -LiteralPath $file -Raw | ConvertFrom-Json
        $result | ConvertTo-Json -Depth 5
        Write-Output "Artifacts: $outputRoot"
        if(-not $result.ok){throw 'Runtime smoke failed'}
        if($process.ExitCode -ne 0){throw "Native process failed with exit code $($process.ExitCode); inspect startup and crash logs even if UI assertions passed."}
        if($env:OM9_PHASE3_MANIFEST){& rtk proxy $python $phase3Evidence bind $env:OM9_PHASE3_MANIFEST --report $file --macro (Join-Path $PSScriptRoot $Macro) --exe $FreeCADExe --pid $process.Id --exit-code $process.ExitCode;if($LASTEXITCODE -ne 0){throw 'Phase3 evidence binding failed'}}
        if($env:OM9_PHASE2_MANIFEST){& rtk proxy python $phase2Evidence bind $env:OM9_PHASE2_MANIFEST --report $file --macro (Join-Path $PSScriptRoot $Macro) --exe $FreeCADExe --pid $process.Id --exit-code $process.ExitCode;if($LASTEXITCODE -ne 0){throw 'Phase2 evidence binding failed'}}
    }
} finally {foreach($key in $oldVars.Keys){[Environment]::SetEnvironmentVariable($key,$oldVars[$key],'Process')}}
