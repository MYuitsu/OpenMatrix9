param([string]$OutputRoot='H:/FreeCAD-src/build/om9-launcher-tests')
$ErrorActionPreference='Stop'
$source=Join-Path $PSScriptRoot '../../launcher/Start-OM9.ps1'
if(-not (Test-Path -LiteralPath $source)){throw 'RED: approved installer/launcher has not been implemented'}
. $source
$run=Join-Path $OutputRoot ([guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($run)|Out-Null
$bundle=Join-Path $run 'bundle';$om9UserRoot=Join-Path $run 'profile'
[IO.Directory]::CreateDirectory((Join-Path $bundle 'OpenMatrix9/bin'))|Out-Null
$payload=Join-Path $bundle 'OpenMatrix9'
[IO.File]::WriteAllText((Join-Path $payload 'InitGui.py'),'test bootstrap')
[IO.File]::WriteAllText((Join-Path $payload 'bin/OpenMatrix9Gui.pyd'),'test native version 1')
[IO.File]::WriteAllText((Join-Path $payload 'bin/OM9ThreeDmImportWorker.exe'),'test worker')
[IO.File]::WriteAllText((Join-Path $payload 'bin/obsolete-helper.dll'),'test removed helper')
function Write-Manifest {
 $files=@(Get-ChildItem -LiteralPath $payload -File -Recurse|ForEach-Object{[ordered]@{path=$_.FullName.Substring($payload.Length+1).Replace('\','/');sha256=Get-OM9Hash $_.FullName}})
 [ordered]@{version='0.0.2';freecad='1.1.4';files=$files}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $bundle 'launcher-manifest.json') -Encoding UTF8
}
$script:checks=@()
function Check($name,$value){$script:checks+=@{name=$name;passed=[bool]$value};if(-not $value){throw $name}}
function Refuses($name,[scriptblock]$operation){$failed=$false;try{&$operation|Out-Null}catch{$failed=$true};Check $name $failed}
Write-Manifest
Check 'different host without this module does not block installation' (-not (Test-OM9HostProcess ([pscustomobject]@{Path='H:\dev\FreeCAD.exe';Modules=@()}) 'C:\stock\FreeCAD.exe' 'C:\profile\OpenMatrix9Gui.pyd'))
Check 'target host process blocks update' (Test-OM9HostProcess ([pscustomobject]@{Path='C:\stock\FreeCAD.exe';Modules=@()}) 'C:\stock\FreeCAD.exe')
function Get-OM9RunningFreeCAD {return @(123)}
$first=Invoke-OM9Install -BundleRoot $bundle -UserHome $om9UserRoot
Check 'first run installs a complete bundle' ($first.updated -and (Test-Path (Join-Path $first.destination 'bin/OM9ThreeDmImportWorker.exe')))
Check 'new isolated installation is allowed while another host is open' $first.updated
function Get-OM9RunningFreeCAD {return @()}
$config=Join-Path $om9UserRoot 'v1-1/user.cfg';[IO.File]::WriteAllText($config,'user settings must remain')
$second=Invoke-OM9Install -BundleRoot $bundle -UserHome $om9UserRoot
Check 'unchanged bundle avoids copy and backup' (-not $second.updated -and -not $second.backup)
[IO.File]::WriteAllText((Join-Path $payload 'bin/OpenMatrix9Gui.pyd'),'test native version 2');Write-Manifest
$third=Invoke-OM9Install -BundleRoot $bundle -UserHome $om9UserRoot
Check 'update installs new native and preserves prior module' ($third.updated -and [IO.File]::ReadAllText((Join-Path $third.backup 'bin/OpenMatrix9Gui.pyd')) -eq 'test native version 1')
Check 'user settings survive installation and update' ([IO.File]::ReadAllText($config) -eq 'user settings must remain')
$before=Get-OM9Hash (Join-Path $third.destination 'bin/OpenMatrix9Gui.pyd')
$metadata=Read-OM9Manifest (Join-Path $bundle 'launcher-manifest.json');$metadata|Add-Member launcher_revision 2
$metadata|ConvertTo-Json -Depth 5|Set-Content (Join-Path $bundle 'launcher-manifest.json') -Encoding UTF8
function Get-OM9RunningFreeCAD {return @(123)}
$metadataOnly=Invoke-OM9Install -BundleRoot $bundle -UserHome $om9UserRoot
Check 'launcher-only update does not replace loaded native module' (-not $metadataOnly.updated -and -not $metadataOnly.backup)
Check 'launcher metadata change preserves installed native hash' ((Get-OM9Hash (Join-Path $third.destination 'bin/OpenMatrix9Gui.pyd')) -eq $before)
function Get-OM9RunningFreeCAD {return @()}
[IO.File]::Delete((Join-Path $payload 'bin/obsolete-helper.dll'));Write-Manifest
function Get-OM9RunningFreeCAD {return @(123)}
Refuses 'removed bundled helper is a real payload update and respects host guard' {Invoke-OM9Install -BundleRoot $bundle -UserHome $om9UserRoot}
Check 'host refusal retains previous helper and native' ([IO.File]::Exists((Join-Path $third.destination 'bin/obsolete-helper.dll')) -and (Get-OM9Hash (Join-Path $third.destination 'bin/OpenMatrix9Gui.pyd')) -eq $before)
function Get-OM9RunningFreeCAD {return @()}
$removed=Invoke-OM9Install -BundleRoot $bundle -UserHome $om9UserRoot
Check 'complete payload replacement removes obsolete bundled helper' ($removed.updated -and -not [IO.File]::Exists((Join-Path $third.destination 'bin/obsolete-helper.dll')))
[IO.File]::WriteAllText((Join-Path $payload 'bin/OpenMatrix9Gui.pyd'),'truncated invalid bytes')
Refuses 'corrupt bundle is rejected before destination mutation' {Invoke-OM9Install -BundleRoot $bundle -UserHome $om9UserRoot}
Check 'corrupt source leaves installed native intact' ((Get-OM9Hash (Join-Path $third.destination 'bin/OpenMatrix9Gui.pyd')) -eq $before)
Write-Manifest
Refuses 'failed promotion rolls back complete previous module' {Invoke-OM9Install -BundleRoot $bundle -UserHome $om9UserRoot -BeforePromote {throw 'injected failure'}}
Check 'rollback restores exact native and settings' (((Get-OM9Hash (Join-Path $third.destination 'bin/OpenMatrix9Gui.pyd')) -eq $before) -and [IO.File]::ReadAllText($config) -eq 'user settings must remain')
function Get-OM9RunningFreeCAD {return @(123)}
Refuses 'update refuses while FreeCAD is open' {Invoke-OM9Install -BundleRoot $bundle -UserHome $om9UserRoot}
Check 'running host rejection leaves installed module intact' ((Get-OM9Hash (Join-Path $third.destination 'bin/OpenMatrix9Gui.pyd')) -eq $before)
function Get-OM9RunningFreeCAD {return @()}
function Get-OM9Host {return 'C:\stock\FreeCAD.exe'}
$BundleRoot=$bundle;$UserHome=$om9UserRoot;$InstallOnly=$true;$NoShortcut=$true
Refuses 'entry rejects incomplete launcher bundle' {Start-OM9}
Check 'incomplete bootstrap cannot replace installed native' ((Get-OM9Hash (Join-Path $third.destination 'bin/OpenMatrix9Gui.pyd')) -eq $before)
$cache=Join-Path $om9UserRoot 'v1-1/OM9-launcher'
$bootstrap=@('Start-OM9.ps1','Start-OM9.cmd','Start-OM9.FCMacro','OpenMatrix9.ico','README.md')
foreach($name in $bootstrap){[IO.File]::Copy((Join-Path (Split-Path $source) $name),(Join-Path $bundle $name))}
$manifest=Read-OM9Manifest (Join-Path $bundle 'launcher-manifest.json')
$manifest|Add-Member launcher_files @($bootstrap|ForEach-Object{@{path=$_;sha256=Get-OM9Hash (Join-Path $bundle $_)}})
$manifest|ConvertTo-Json -Depth 5|Set-Content (Join-Path $bundle 'launcher-manifest.json') -Encoding UTF8
foreach($name in ($bootstrap+@('launcher-manifest.json'))){[IO.File]::WriteAllText((Join-Path $cache $name),('old cache '+$name))}
$DesktopPath=Join-Path $run 'Desktop';[IO.Directory]::CreateDirectory($DesktopPath)|Out-Null
$shortcut=Join-Path $DesktopPath 'OpenMatrix9.lnk';[IO.File]::WriteAllText($shortcut,'existing shortcut')
function Copy-OM9CacheFile($Source,$Target){if($Target.EndsWith('Start-OM9.FCMacro')){throw 'injected cache write failure'};[IO.File]::Copy($Source,$Target,$true)}
Refuses 'cache promotion failure rejects complete entry update' {Start-OM9}
Check 'cache failure restores installed native and every old cache file' (((Get-OM9Hash (Join-Path $third.destination 'bin/OpenMatrix9Gui.pyd')) -eq $before) -and -not @($bootstrap+@('launcher-manifest.json')|Where-Object{[IO.File]::ReadAllText((Join-Path $cache $_)) -ne ('old cache '+$_)}).Count)
Check 'failed entry leaves existing shortcut unchanged' ([IO.File]::ReadAllText($shortcut) -eq 'existing shortcut')
function Copy-OM9CacheFile($Source,$Target){[IO.File]::Copy($Source,$Target,$true)}
Start-OM9
Check 'complete entry updates plugin and persistent bootstrap together' ((Get-OM9Hash (Join-Path $third.destination 'bin/OpenMatrix9Gui.pyd')) -eq (Get-OM9Hash (Join-Path $payload 'bin/OpenMatrix9Gui.pyd')) -and (Get-OM9Hash (Join-Path $cache 'launcher-manifest.json')) -eq (Get-OM9Hash (Join-Path $bundle 'launcher-manifest.json')))
$bad=@{version='0.0.2';freecad='1.1.4';files=@(@{path='../escape';sha256=('0'*64)})}
$bad|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $bundle 'launcher-manifest.json')
Refuses 'manifest path traversal is rejected' {Invoke-OM9Install -BundleRoot $bundle -UserHome $om9UserRoot}
Check 'backups and failed stages stay outside Mod' (-not @(Get-ChildItem (Join-Path $om9UserRoot 'v1-1/Mod') -Directory|Where-Object Name -ne 'OpenMatrix9').Count)
@{ok=$true;checks=$script:checks}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $run 'results.json')
Write-Output "Launcher tests PASS: $($script:checks.Count); $run"
