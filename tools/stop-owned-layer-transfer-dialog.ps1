$ErrorActionPreference='Stop'
$launchPath='H:/FreeCAD-src/build/shared-host-tests/layer-transfer-final-modeling_layer_transfer_commands_smoke/shared-host-launch.json'
$launch=Get-Content -LiteralPath $launchPath -Raw|ConvertFrom-Json
if($launch.pid -ne 58080 -or [IO.Path]::GetFullPath($launch.host) -ne [IO.Path]::GetFullPath('H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe')){throw 'Unexpected owned test identity'}
$process=Get-Process -Id $launch.pid -ErrorAction SilentlyContinue
if($process){
 if($process.ProcessName -ne 'FreeCAD' -or [IO.Path]::GetFullPath($process.Path) -ne [IO.Path]::GetFullPath($launch.host)){throw 'Owned PID no longer matches FreeCAD test'}
 if($process.StartTime -gt (Get-Item -LiteralPath $launchPath).LastWriteTime.AddSeconds(5)){throw 'PID was reused after launch record'}
 $command=Get-CimInstance Win32_Process -Filter 'ProcessId=58080'
 if($command.CommandLine -notlike '*layer-transfer-final-modeling_layer_transfer_commands_smoke*'){throw 'Owned fixture command line mismatch'}
 Stop-Process -Id $process.Id -Force
 Write-Output 'Stopped only owned dialog fixture PID=58080; source reports retained.'
}else{Write-Output 'Owned fixture has already exited'}
