param([string]$Macro='', [switch]$Wait)
$ErrorActionPreference='Stop'
$hostRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$runtime=Join-Path $hostRoot 'Runtime'
$exe=Join-Path $hostRoot 'bin/FreeCAD.exe'
if(-not (Test-Path -LiteralPath $exe)){throw 'Place OpenMatrix9 inside FreeCAD-OM9-27.1/Mod, then run Start-OM9.cmd'}
# The existing native import adapter resolves its helper beside FreeCAD.exe.
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'bin/OM9ThreeDmImportWorker.exe') -Destination (Join-Path $hostRoot 'bin/OM9ThreeDmImportWorker.exe') -Force
$env:PATH="$hostRoot/bin;$PSScriptRoot/bin;$runtime;$runtime/DLLs;$runtime/Library/bin;"+$env:PATH
$env:PYTHONHOME=$runtime
$env:QT_PLUGIN_PATH=Join-Path $runtime 'Library/lib/qt6/plugins'
$env:QT_QPA_PLATFORM_PLUGIN_PATH=Join-Path $env:QT_PLUGIN_PATH 'platforms'
$env:FREECAD_USER_HOME=Join-Path $env:APPDATA 'OpenMatrix9/0.0.1'
New-Item -ItemType Directory -Path $env:FREECAD_USER_HOME -Force | Out-Null
$argsList=@('-M',('"'+$PSScriptRoot+'"'),'-P',('"'+(Join-Path $PSScriptRoot 'bin')+'"'),'-u',('"'+(Join-Path $env:FREECAD_USER_HOME 'user.cfg')+'"'),'-s',('"'+(Join-Path $env:FREECAD_USER_HOME 'system.cfg')+'"'))
$env:OM9_RELEASE_PLUGIN=$PSScriptRoot
if($Macro){$env:OM9_RELEASE_TEST_MACRO=[IO.Path]::GetFullPath($Macro)}else{$env:OM9_RELEASE_TEST_MACRO=''}
$argsList+=('"'+(Join-Path $PSScriptRoot 'Start-OM9.FCMacro')+'"')
$out=Join-Path $env:FREECAD_USER_HOME 'last-launch.stdout.log'
$err=Join-Path $env:FREECAD_USER_HOME 'last-launch.stderr.log'
$process=Start-Process -FilePath $exe -ArgumentList $argsList -WorkingDirectory $hostRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput $out -RedirectStandardError $err
Write-Output "OpenMatrix9 PID=$($process.Id); logs=$env:FREECAD_USER_HOME"
if($Wait){if(-not $process.WaitForExit(180000)){throw "Release test timed out; owned PID=$($process.Id)"}; if($process.ExitCode -ne 0){throw "FreeCAD exited $($process.ExitCode); see $err"}}
