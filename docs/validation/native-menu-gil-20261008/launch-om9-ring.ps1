param([string]$ThreeDmPath='')
$ErrorActionPreference='Stop'
if($ThreeDmPath -and -not(Test-Path -LiteralPath $ThreeDmPath -PathType Leaf)){throw "3DM file not found: $ThreeDmPath"}
$taskRoot='H:\FreeCAD-src\build\om9-manual-ring-gil-20261008'
New-Item -ItemType Directory -Force -Path $taskRoot | Out-Null
$env:PATH='H:\FreeCAD-src\.pixi\envs\default\Library\bin;H:\FreeCAD-src\.pixi\envs\default;'+$env:PATH
$env:OM9_MANUAL_OUTPUT=$taskRoot
$env:OM9_MANUAL_3DM_FILE=$ThreeDmPath
$env:FREECAD_USER_HOME=Join-Path $taskRoot 'profile'
$env:TEMP='H:\FreeCAD-src\build\3dm-preservation-sdk\test-temp'
$env:TMP=$env:TEMP
$arguments=@('-u',('"'+(Join-Path $taskRoot 'user.cfg')+'"'),'-s',('"'+(Join-Path $taskRoot 'system.cfg')+'"'),'"H:\FreeCAD-src\build\launch-om9-ring.FCMacro"')
$process=Start-Process -FilePath 'H:\FreeCAD-src\build\om9-ui-3dm-gil-sdk\bin\FreeCAD.exe' -WorkingDirectory 'H:\FreeCAD-src\build\om9-ui-3dm-gil-sdk\bin' -ArgumentList $arguments -WindowStyle Normal -PassThru
$process.Id | Set-Content -LiteralPath (Join-Path $taskRoot 'process-id.txt')
Write-Output "Interactive OpenMatrix9 launched, PID $($process.Id)"
