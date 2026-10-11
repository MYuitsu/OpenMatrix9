param([string]$OutputRoot=(Join-Path $PSScriptRoot '../../build/ci-gui-profile-tests'))
$ErrorActionPreference='Stop'
$helper=Join-Path $PSScriptRoot '../ci/gui-profile.ps1'
if(-not(Test-Path -LiteralPath $helper)){throw 'RED: no isolated software OpenGL profile initializer'}
. $helper
$root=Join-Path $OutputRoot ([guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path "$root/host/bin" -Force|Out-Null
[IO.File]::WriteAllText("$root/host/bin/opengl32sw.dll",'official host fixture')
Initialize-OM9CIGuiProfile "$root/host" "$root/test"|Out-Null
[xml]$xml=Get-Content -LiteralPath "$root/test/user.cfg" -Raw
$setting=$xml.SelectSingleNode('//FCParamGroup[@Name="OpenGL"]/FCBool[@Name="UseSoftwareOpenGL"]')
if($setting.Value -cne '1'){throw 'FreeCAD must request software OpenGL before QApplication startup'}
if($env:QT_OPENGL -cne 'software'){throw 'Qt must use the same software implementation'}
if(-not(Test-Path "$root/test/profile" -PathType Container)){throw 'FreeCAD ignores a nonexistent FREECAD_USER_HOME'}
[IO.File]::Delete("$root/host/bin/opengl32sw.dll")
$refused=$false;try{Initialize-OM9CIGuiProfile "$root/host" "$root/missing"|Out-Null}catch{$refused=$true}
if(-not $refused -or(Test-Path "$root/missing")){throw 'Missing official software renderer must be refused before profile creation'}
Write-Output 'Isolated CI GUI profile: 4 checks PASS'
