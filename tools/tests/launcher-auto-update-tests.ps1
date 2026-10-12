param([string]$OutputRoot='H:/FreeCAD-src/build/om9-launcher-auto-update-tests')
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot '../../launcher/Start-OM9.ps1')
$realShowUpdate=(Get-Command Show-OM9Update).ScriptBlock
$run=Join-Path $OutputRoot ([guid]::NewGuid().ToString('N'))
$cache=Join-Path $run 'profile/v1-1/OM9-launcher'
[IO.Directory]::CreateDirectory($cache)|Out-Null
[IO.File]::Copy((Join-Path $PSScriptRoot '../../launcher/Start-OM9.ps1'),(Join-Path $cache 'Start-OM9.ps1'))
$script:checks=@()
function Check($name,$value){$script:checks+=@{name=$name;passed=[bool]$value};[IO.File]::AppendAllText((Join-Path $run 'checkpoints.log'),($name+' = '+[bool]$value+[Environment]::NewLine));if(-not $value){throw $name}}
function Refuses($name,[scriptblock]$operation){$failed=$false;try{&$operation|Out-Null}catch{$failed=$true};Check $name $failed}
$release=[pscustomobject]@{tag_name='v0.0.4';draft=$false;prerelease=$true;html_url='https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.0.4';assets=@(
 [pscustomobject]@{name='OpenMatrix9-0.0.4-SetupLauncher-Windows-x64.zip';browser_download_url='https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.0.4/OpenMatrix9-0.0.4-SetupLauncher-Windows-x64.zip'},
 [pscustomobject]@{name='SHA256SUMS.txt';browser_download_url='https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.0.4/SHA256SUMS.txt'})}
$update=Find-OM9Update @($release) '0.0.3' 1
Check 'startup candidate identifies its matching official checksum asset' ($update.checksum_url -ceq 'https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.0.4/SHA256SUMS.txt' -and $update.asset_name -ceq 'OpenMatrix9-0.0.4-SetupLauncher-Windows-x64.zip')
$missing=$release.PSObject.Copy();$missing.assets=@($release.assets[0])
Check 'release without checksum cannot be installed automatically' ($null -eq (Find-OM9Update @($missing) '0.0.3' 1))
$bundle=Join-Path $run 'fixture';$payload=Join-Path $bundle 'OpenMatrix9/bin'
[IO.Directory]::CreateDirectory($payload)|Out-Null
[IO.File]::WriteAllText((Join-Path $bundle 'OpenMatrix9/InitGui.py'),'fixture bootstrap')
[IO.File]::WriteAllText((Join-Path $payload 'OpenMatrix9Gui.pyd'),'new native')
[IO.File]::WriteAllText((Join-Path $payload 'OM9ThreeDmImportWorker.exe'),'fixture worker')
$bootstrap=@('Start-OM9.ps1','Start-OM9.cmd','Start-OM9.FCMacro','OpenMatrix9.ico','README.md')
foreach($name in $bootstrap){[IO.File]::Copy((Join-Path $PSScriptRoot ('../../launcher/'+$name)),(Join-Path $bundle $name))}
$manifest=@{version='0.0.4';launcher_revision=1;freecad='1.1.4';files=@(
 @{path='InitGui.py';sha256=Get-OM9Hash (Join-Path $bundle 'OpenMatrix9/InitGui.py')},
 @{path='bin/OpenMatrix9Gui.pyd';sha256=Get-OM9Hash (Join-Path $payload 'OpenMatrix9Gui.pyd')},
 @{path='bin/OM9ThreeDmImportWorker.exe';sha256=Get-OM9Hash (Join-Path $payload 'OM9ThreeDmImportWorker.exe')});launcher_files=@($bootstrap|ForEach-Object{@{path=$_;sha256=Get-OM9Hash (Join-Path $bundle $_)}})}
$manifest|ConvertTo-Json -Depth 6|Set-Content (Join-Path $bundle 'launcher-manifest.json') -Encoding UTF8
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip=Join-Path $run $update.asset_name
[IO.Compression.ZipFile]::CreateFromDirectory($bundle,$zip)
$sum=Join-Path $run 'SHA256SUMS.txt';[IO.File]::WriteAllText($sum,((Get-OM9Hash $zip)+'  '+$update.asset_name+"`n"))
# Only the external GitHub transport is replaced: real ZIP, validation and installer run below.
function Save-OM9ReleaseAsset($Url,$Path,$Limit,$Progress){
 if($Url -ceq $update.download_url){[IO.File]::Copy($zip,$Path)}
 elseif($Url -ceq $update.checksum_url){[IO.File]::Copy($sum,$Path)}
 else{throw 'Unexpected download request'}
}
$prepared=Get-OM9UpdateBundle $update $cache
Check 'downloaded ZIP is verified and extracted without browser or manual extraction' ([IO.File]::ReadAllText((Join-Path $prepared 'OpenMatrix9/bin/OpenMatrix9Gui.pyd')) -eq 'new native')
$originalSum=[IO.File]::ReadAllText($sum)
[IO.File]::WriteAllText($sum,(('0'*64)+'  '+$update.asset_name+"`n"))
Refuses 'wrong archive checksum rejects update' {Get-OM9UpdateBundle $update $cache}
[IO.File]::WriteAllText($sum,$originalSum)
$wrongVersion=$update.PSObject.Copy();$wrongVersion.version='0.0.5'
Refuses 'selected release must match downloaded manifest' {Get-OM9UpdateBundle $wrongVersion $cache}
$badZip=Join-Path $run 'traversal.zip'
$archive=[IO.Compression.ZipFile]::Open($badZip,[IO.Compression.ZipArchiveMode]::Create)
$entry=$archive.CreateEntry('../outside.txt');$writer=New-Object IO.StreamWriter($entry.Open());$writer.Write('escape');$writer.Dispose();$archive.Dispose()
Refuses 'ZIP path traversal rejected before any extraction' {Expand-OM9UpdateArchive $badZip (Join-Path $run 'bad')}
Check 'ZIP cannot write outside staging root' (-not [IO.File]::Exists((Join-Path $run 'outside.txt')))
foreach($case in @(@{name='duplicate';paths=@('InitGui.py','initgui.py')},@{name='device';paths=@('NUL.txt')},@{name='trimmed';paths=@('InitGui.py.')})){
 $path=Join-Path $run ($case.name+'.zip');$archive=[IO.Compression.ZipFile]::Open($path,[IO.Compression.ZipArchiveMode]::Create)
 foreach($name in $case.paths){$entry=$archive.CreateEntry($name);$writer=New-Object IO.StreamWriter($entry.Open());$writer.Write('bad');$writer.Dispose()};$archive.Dispose()
 Refuses ('ambiguous Windows ZIP paths rejected: '+$case.name) {Expand-OM9UpdateArchive $path (Join-Path $run ($case.name+'-out'))}
}
Check 'insecure download redirects rejected' (-not (Test-OM9DownloadUri ([uri]'http://github.com/file')) -and -not (Test-OM9DownloadUri ([uri]'https://github.com.evil.example/file')) -and -not (Test-OM9DownloadUri ([uri]'https://user@github.com/file')))
Check 'official GitHub signed asset redirects allowed' (Test-OM9DownloadUri ([uri]'https://release-assets.githubusercontent.com/github-production-release-asset/file?sig=example'))
# An unsignalled response handle models a connection that never returns headers.
$pending=[pscustomobject]@{AsyncWaitHandle=(New-Object Threading.ManualResetEvent($false))}
$timeoutObserved=$false
try{Wait-OM9DownloadResponse $pending $null 25|Out-Null}catch{$timeoutObserved=$_.Exception -is [TimeoutException]}
finally{$pending.AsyncWaitHandle.Dispose()}
Check 'stalled asynchronous connection terminates with an explicit timeout' $timeoutObserved
$goodZip=Join-Path $run 'good.zip';[IO.File]::Copy($zip,$goodZip)
[IO.File]::WriteAllText((Join-Path $payload 'OpenMatrix9Gui.pyd'),'tampered bytes');[IO.File]::Delete($zip)
[IO.Compression.ZipFile]::CreateFromDirectory($bundle,$zip)
[IO.File]::WriteAllText($sum,((Get-OM9Hash $zip)+'  '+$update.asset_name+"`n"))
Refuses 'valid archive checksum cannot hide damaged payload manifest hashes' {Get-OM9UpdateBundle $update $cache}
[IO.File]::Delete($zip);[IO.File]::Copy($goodZip,$zip);[IO.File]::WriteAllText($sum,$originalSum)
$userRoot=Join-Path $run 'profile'
$destination=Join-Path $userRoot 'v1-1/Mod/OpenMatrix9'
[IO.Directory]::CreateDirectory((Join-Path $destination 'bin'))|Out-Null
[IO.File]::WriteAllText((Join-Path $destination 'bin/OpenMatrix9Gui.pyd'),'old native')
[IO.File]::WriteAllText((Join-Path $userRoot 'v1-1/user.cfg'),'user settings')
function Get-OM9RunningFreeCAD {return @()}
$receipt=Install-OM9UpdateBundle $prepared $update $cache 'C:\stock\FreeCAD.exe'
Check 'automatic install replaces module and persistent launcher together' ($receipt.updated -and [IO.File]::ReadAllText((Join-Path $destination 'bin/OpenMatrix9Gui.pyd')) -eq 'new native' -and (Read-OM9Manifest (Join-Path $cache 'launcher-manifest.json')).version -eq '0.0.4')
Check 'automatic install retains settings and exact previous module backup' ([IO.File]::ReadAllText((Join-Path $userRoot 'v1-1/user.cfg')) -eq 'user settings' -and [IO.File]::ReadAllText((Join-Path $receipt.backup 'bin/OpenMatrix9Gui.pyd')) -eq 'old native')
$downgrade=$update.PSObject.Copy();$downgrade.version='0.0.3';$downgrade.asset_name='OpenMatrix9-0.0.3-SetupLauncher-Windows-x64.zip';$downgrade.download_url='https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.0.3/'+$downgrade.asset_name;$downgrade.checksum_url='https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.0.3/SHA256SUMS.txt'
Refuses 'stale update cannot downgrade a newer installed version' {Install-OM9UpdateBundle $prepared $downgrade $cache 'C:\stock\FreeCAD.exe'}
function Get-OM9ReleaseFeed {return @($release)}
function Show-OM9Update {return 'Later'}
$available=Invoke-OM9UpdateCheck $cache '0.0.3' 1 $false
Check 'Later keeps update available for next startup' ($available.status -eq 'available')
$silent=Invoke-OM9UpdateCheck $cache '0.0.3' 1 $true
Check 'silent startup check never downloads or installs' ($silent.status -eq 'available')
Add-Type -AssemblyName System.Windows.Forms
Set-Item Function:Show-OM9Update $realShowUpdate
$ui=@{clicked=$false;waiting=$false;cancel=$false}
$timer=New-Object Windows.Forms.Timer;$timer.Interval=100
$timer.Add_Tick({
 foreach($form in @([Windows.Forms.Application]::OpenForms)){
  if($form.Text -eq 'OpenMatrix9 update'){
   $button=@($form.Controls|Where-Object{$_ -is [Windows.Forms.Button] -and $_.Text -eq 'Update'})[0]
   $ui.clicked=$true;$button.PerformClick()
  }
  if($form.Text -eq 'Updating OpenMatrix9'){
   $label=@($form.Controls|Where-Object{$_ -is [Windows.Forms.Label]})[0]
   if($label.Text -like 'Download verified. Save*'){
    $ui.waiting=$true
    if($ui.cancel){@($form.Controls|Where-Object{$_ -is [Windows.Forms.Button] -and $_.Text -eq 'Cancel'})[0].PerformClick()}
   }
   $form.Hide()
  }
 }
}.GetNewClosure())
$timer.Start()
try{
 $answer=Show-OM9Update $update
 Check 'real Windows Update button selects automatic installation' ($ui.clicked -and $answer -eq 'Update')
 [IO.File]::WriteAllText((Join-Path $destination 'bin/OpenMatrix9Gui.pyd'),'before cancelled update')
 function Get-OM9RunningFreeCAD {return @(123)}
 $ui.cancel=$true
 Refuses 'Cancel while waiting closes updater without installing' {Invoke-OM9Update $update $cache 'C:\stock\FreeCAD.exe'}
 Check 'waiting for close keeps current native intact on cancel' ($ui.waiting -and [IO.File]::ReadAllText((Join-Path $destination 'bin/OpenMatrix9Gui.pyd')) -eq 'before cancelled update')
 $ui.cancel=$false;$ui.waiting=$false
 $script:ownedWait=Start-Process -FilePath (Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe') -ArgumentList '-NoProfile -Command Start-Sleep -Seconds 2' -WindowStyle Hidden -PassThru
 function Get-OM9RunningFreeCAD {if(-not $script:ownedWait.HasExited){return @($script:ownedWait)};return @()}
 # Native host reopening is covered separately with the real FreeCAD bundle.
 function Start-OM9UpdatedHost {}
 $completedUpdate=Invoke-OM9Update $update $cache 'C:\stock\FreeCAD.exe'
 Check 'updater waits for owned process exit then installs automatically' ($ui.waiting -and $completedUpdate.status -eq 'installed' -and $script:ownedWait.HasExited -and [IO.File]::ReadAllText((Join-Path $destination 'bin/OpenMatrix9Gui.pyd')) -eq 'new native')
}finally{$timer.Stop();$timer.Dispose();if($script:ownedWait -and -not $script:ownedWait.HasExited){$script:ownedWait.Kill()}}
@{ok=$true;checks=$script:checks}|ConvertTo-Json -Depth 6|Set-Content (Join-Path $run 'results.json')
Write-Output "Launcher auto-update tests PASS: $($script:checks.Count); $run"
