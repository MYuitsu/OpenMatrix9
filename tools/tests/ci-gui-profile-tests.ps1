param([string]$OutputRoot=(Join-Path $PSScriptRoot '../../build/ci-gui-profile-tests'))
$ErrorActionPreference='Stop'
$helper=Join-Path $PSScriptRoot '../ci/gui-profile.ps1'
if(-not(Test-Path -LiteralPath $helper)){throw 'RED: no isolated software OpenGL profile initializer'}
. $helper
$root=[IO.Path]::GetFullPath((Join-Path $OutputRoot ([guid]::NewGuid().ToString('N'))))
$owned=Join-Path $root 'owned'
$script:checks=0
# Binary fixture catches text-copy corruption without loading any renderer DLL.
$mesa=[byte[]](0,1,2,10,13,26,127,128,254,255)
function Assert-ProfileCheck([bool]$Condition,[string]$Message){
 if(-not $Condition){throw "RED: $Message"}
 $script:checks++
}
function New-FakeHost([string]$Path){
 New-Item -ItemType Directory -Path (Join-Path $Path 'bin') -Force|Out-Null
 [IO.File]::WriteAllBytes((Join-Path $Path 'bin/opengl32sw.dll'),$mesa)
 return $Path
}
function Same-Bytes([string]$Path,[byte[]]$Expected){
 return (Test-Path -LiteralPath $Path -PathType Leaf) -and
  [Convert]::ToBase64String([IO.File]::ReadAllBytes($Path)) -ceq [Convert]::ToBase64String($Expected)
}
function Assert-ProfileRefused([string]$HostPath,[string]$Output,[string]$Message){
 $refused=$false;try{Initialize-OM9CIGuiProfile $HostPath $Output|Out-Null}catch{$refused=$true}
 Assert-ProfileCheck ($refused -and -not(Test-Path -LiteralPath $Output)) $Message
}

$missingAlias=New-FakeHost (Join-Path $owned 'missing-alias')
Assert-ProfileRefused $missingAlias "$root/missing-alias-profile" 'A missing matching native Mesa alias must be refused before profile creation'

$hostPath=New-FakeHost (Join-Path $owned 'host')
$alias=Join-Path $hostPath 'bin/opengl32.dll'
Initialize-OM9CISoftwareRenderer $hostPath $owned|Out-Null
Assert-ProfileCheck ((Same-Bytes $alias $mesa) -and -not((Get-Item -LiteralPath $alias).Attributes -band [IO.FileAttributes]::ReparsePoint)) 'Native opengl32.dll must be a byte-identical extracted Mesa copy'
$timestamp=[datetime]::new(2020,1,2,3,4,5,[DateTimeKind]::Utc)
[IO.File]::SetLastWriteTimeUtc($alias,$timestamp)
Initialize-OM9CISoftwareRenderer $hostPath $owned|Out-Null
Assert-ProfileCheck ((Same-Bytes $alias $mesa) -and [IO.File]::GetLastWriteTimeUtc($alias) -eq $timestamp) 'A matching existing alias must be preserved without rewriting'

Initialize-OM9CIGuiProfile $hostPath "$root/test"|Out-Null
[xml]$xml=Get-Content -LiteralPath "$root/test/user.cfg" -Raw
$setting=$xml.SelectSingleNode('//FCParamGroup[@Name="OpenGL"]/FCBool[@Name="UseSoftwareOpenGL"]')
Assert-ProfileCheck ($setting -and $setting.Value -ceq '1') 'FreeCAD must request software OpenGL before QApplication startup'
$capture=$xml.SelectSingleNode('/FCParameters/FCParamGroup[@Name="Root"]/FCParamGroup[@Name="BaseApp"]/FCParamGroup[@Name="Preferences"]/FCParamGroup[@Name="View"]/FCText[@Name="SavePicture"]')
Assert-ProfileCheck ($capture -and $capture.InnerText -ceq 'FramebufferObject') 'Isolated View/SavePicture must render through the native FramebufferObject path'
Assert-ProfileCheck ($env:QT_OPENGL -ceq 'software') 'Qt must use the same software implementation'
Assert-ProfileCheck ([IO.Path]::GetFullPath($env:QT_OPENGL_DLL) -ieq [IO.Path]::GetFullPath($alias)) 'Qt and native Coin must load the same owned opengl32.dll alias'
Assert-ProfileCheck (Test-Path -LiteralPath "$root/test/profile" -PathType Container) 'FreeCAD ignores a nonexistent FREECAD_USER_HOME'

$missingBundle=Join-Path $owned 'missing-bundle'
New-Item -ItemType Directory -Path "$missingBundle/bin" -Force|Out-Null
$refused=$false;try{Initialize-OM9CISoftwareRenderer $missingBundle $owned|Out-Null}catch{$refused=$true}
Assert-ProfileCheck ($refused -and -not(Test-Path -LiteralPath "$missingBundle/bin/opengl32.dll")) 'A missing bundled renderer must be refused without creating an alias'
Assert-ProfileRefused $missingBundle "$root/missing-bundle-profile" 'A missing bundled renderer must be refused before profile creation'

$conflict=New-FakeHost (Join-Path $owned 'conflict')
$otherRenderer=[byte[]](255,128,0,7)
[IO.File]::WriteAllBytes("$conflict/bin/opengl32.dll",$otherRenderer)
$refused=$false;try{Initialize-OM9CISoftwareRenderer $conflict $owned|Out-Null}catch{$refused=$true}
Assert-ProfileCheck ($refused -and (Same-Bytes "$conflict/bin/opengl32.dll" $otherRenderer)) 'A conflicting existing alias must be refused and left byte-identical'
Assert-ProfileRefused $conflict "$root/conflict-profile" 'A conflicting renderer must be refused before profile creation'

# Prefix-sharing sibling catches an unsafe string-prefix ownership test.
$outside=New-FakeHost (Join-Path $root 'owned-sibling/host')
$refused=$false;try{Initialize-OM9CISoftwareRenderer $outside $owned|Out-Null}catch{$refused=$true}
Assert-ProfileCheck ($refused -and -not(Test-Path -LiteralPath "$outside/bin/opengl32.dll")) 'Deployment must refuse a host outside the owned extraction root'
[IO.File]::WriteAllBytes("$outside/bin/opengl32.dll",$mesa)
$refused=$false;try{Initialize-OM9CISoftwareRenderer $outside $owned|Out-Null}catch{$refused=$true}
Assert-ProfileCheck ($refused -and (Same-Bytes "$outside/bin/opengl32.dll" $mesa)) 'A matching existing alias must not bypass the ownership boundary'

# Directory junctions exercise reparse refusal without symlink privileges.
# All junction targets and sentinel files remain under this private test root.
$linked=New-FakeHost (Join-Path $owned 'linked-alias')
$linkTarget=Join-Path $root 'alias-target'
New-Item -ItemType Directory -Path $linkTarget -Force|Out-Null
[IO.File]::WriteAllBytes("$linkTarget/sentinel.bin",$otherRenderer)
New-Item -ItemType Junction -Path "$linked/bin/opengl32.dll" -Target $linkTarget|Out-Null
$refused=$false;try{Initialize-OM9CISoftwareRenderer $linked $owned|Out-Null}catch{$refused=$true}
Assert-ProfileCheck ($refused -and ((Get-Item -LiteralPath "$linked/bin/opengl32.dll").Attributes -band [IO.FileAttributes]::ReparsePoint) -and (Same-Bytes "$linkTarget/sentinel.bin" $otherRenderer) -and @(Get-ChildItem -LiteralPath $linkTarget).Count -eq 1) 'An existing reparse alias and its target must never be replaced or modified'
Assert-ProfileRefused $linked "$root/reparse-profile" 'A reparse directory alias must be refused before profile creation'

$linkedBin=Join-Path $owned 'linked-bin'
$binTarget=Join-Path $root 'external-bin'
New-Item -ItemType Directory -Path $linkedBin,$binTarget -Force|Out-Null
[IO.File]::WriteAllBytes("$binTarget/opengl32sw.dll",$mesa)
New-Item -ItemType Junction -Path "$linkedBin/bin" -Target $binTarget|Out-Null
$refused=$false;try{Initialize-OM9CISoftwareRenderer $linkedBin $owned|Out-Null}catch{$refused=$true}
Assert-ProfileCheck ($refused -and -not(Test-Path -LiteralPath "$binTarget/opengl32.dll") -and (Same-Bytes "$binTarget/opengl32sw.dll" $mesa)) 'Deployment must refuse a host path that traverses a reparse directory'
Write-Output "Isolated CI GUI profile and renderer: $script:checks checks PASS"
