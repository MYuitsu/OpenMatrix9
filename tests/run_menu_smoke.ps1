param(
    [Parameter(Mandatory=$true)][string]$FreeCADExe,
    [Parameter(Mandatory=$true)][string]$DependencyPrefix,
    [double]$Scale = 1,
    [string]$Macro = 'public_icons_smoke.FCMacro',
    [string]$NativeModuleDirectory = '',
    [ValidateRange(1,600)][int]$TimeoutSeconds = 60,
    [switch]$KeepOpen
)
$ErrorActionPreference='Stop'
$moduleRoot=Split-Path $PSScriptRoot -Parent
$suite=if($Macro -eq 'menu_smoke.FCMacro'){'smoke'}else{[IO.Path]::GetFileNameWithoutExtension($Macro)}
$outputRoot=Join-Path $moduleRoot "build/$suite-$Scale/$([guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $outputRoot -Force | Out-Null
$oldVars=@{}
foreach($key in @('PATH','FREECAD_USER_HOME','OM9_SMOKE_OUTPUT','OM9_SMOKE_KEEP_OPEN','QT_SCALE_FACTOR','OM9_TEST_NATIVE_DIR')) {$oldVars[$key]=[Environment]::GetEnvironmentVariable($key,'Process')}
try {
    $env:PATH="$DependencyPrefix/bin;$(Split-Path $DependencyPrefix -Parent);"+$env:PATH
    $env:FREECAD_USER_HOME=Join-Path $outputRoot 'profile'
    $env:OM9_SMOKE_OUTPUT=$outputRoot
    $env:OM9_TEST_NATIVE_DIR=$NativeModuleDirectory
    $env:OM9_SMOKE_KEEP_OPEN=if($KeepOpen){'1'}else{'0'}
    $env:QT_SCALE_FACTOR=$Scale.ToString([System.Globalization.CultureInfo]::InvariantCulture)
    $arguments=@('-u',"`"$(Join-Path $outputRoot 'user.cfg')`"",'-s',"`"$(Join-Path $outputRoot 'system.cfg')`"","`"$(Join-Path $PSScriptRoot $Macro)`"")
    if($NativeModuleDirectory){$arguments=@('-P',"`"$NativeModuleDirectory`"")+$arguments}
    $process=Start-Process -FilePath $FreeCADExe -ArgumentList $arguments -WindowStyle Hidden -PassThru
    if(-not $KeepOpen) {
        if(-not $process.WaitForExit($TimeoutSeconds*1000)){throw "Smoke process $($process.Id) did not finish within $TimeoutSeconds seconds; inspect its logs/window."}
        $file=Join-Path $outputRoot 'results.json'
        if(-not(Test-Path -LiteralPath $file)){throw "No smoke report. Process exit: $($process.ExitCode)"}
        $result=Get-Content -LiteralPath $file -Raw | ConvertFrom-Json
        [pscustomobject]@{
            ok=$result.ok
            checks=@($result.checks).Count
            failed=@($result.checks | Where-Object { -not $_.passed } | ForEach-Object { $_.name })
            error=$result.error
        } | ConvertTo-Json -Depth 5
        Write-Output "Artifacts: $outputRoot"
        if(-not $result.ok){throw 'Runtime smoke failed'}
        if($process.ExitCode -ne 0){throw "Native process failed with exit code $($process.ExitCode); inspect startup and crash logs even if UI assertions passed."}
    }
} finally {foreach($key in $oldVars.Keys){[Environment]::SetEnvironmentVariable($key,$oldVars[$key],'Process')}}
