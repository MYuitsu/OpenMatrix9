param([string]$OutputRoot=(Join-Path $PSScriptRoot '../../build/stock-host-tests'))
$ErrorActionPreference='Stop'
$functionFile=Join-Path $PSScriptRoot '../ci/stock-host.ps1'
if(-not (Test-Path -LiteralPath $functionFile)){throw 'RED: portable archive host selection has not been implemented'}
. $functionFile
$run=Join-Path $OutputRoot ([guid]::NewGuid().ToString('N'))
$root=Join-Path $run 'FreeCAD_1.1.4'
New-Item -ItemType Directory -Path "$root/bin","$root/lib" -Force|Out-Null
foreach($file in @('FreeCAD.exe','bin/freecad.exe','bin/FreeCADGui.dll','lib/Part.pyd')){[IO.File]::WriteAllText((Join-Path $root $file),'fixture')}
if((Find-OM9StockHost $run) -cne [IO.Path]::GetFullPath($root)){throw 'Portable root wrapper must not be selected as the native host'}
[IO.File]::Delete((Join-Path $root 'lib/Part.pyd'))
$refused=$false;try{Find-OM9StockHost $run|Out-Null}catch{$refused=$true}
if(-not $refused){throw 'Incomplete host must be refused'}
Write-Output 'Portable stock host selection: 2 checks PASS'
