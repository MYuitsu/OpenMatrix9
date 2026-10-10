# SPDX-License-Identifier: LGPL-2.1-or-later
# Windows installation/shortcut/process bootstrap only; OM9 model logic stays in Rust.
param([string]$BundleRoot=$PSScriptRoot,[string]$FreeCADPath='',
      [string]$UserHome='',[string]$DesktopPath='',[string]$StartMenuPath='',
      [switch]$Installed,[switch]$InstallOnly,[switch]$NoShortcut,[switch]$NoDialogs,
      [switch]$CheckUpdatesOnly,[switch]$SkipUpdateCheck)
$ErrorActionPreference='Stop'
if($PSVersionTable.PSVersion.Major -lt 6){$env:PSModulePath="$PSHOME\Modules;$env:SystemRoot\System32\WindowsPowerShell\v1.0\Modules"}

function Get-OM9Hash([string]$Path) {
 $stream=[IO.File]::OpenRead($Path);$sha=[Security.Cryptography.SHA256]::Create()
 try{return ([BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-','').ToLowerInvariant()}
 finally{$sha.Dispose();$stream.Dispose()}
}
function Assert-OM9PlainPath([string]$Path) {
 $current=[IO.Path]::GetFullPath($Path)
 while($current){
  if([IO.Directory]::Exists($current) -or [IO.File]::Exists($current)){
   if(([IO.File]::GetAttributes($current) -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw "Linked path is unsupported: $current"}
  }
  $current=[IO.Path]::GetDirectoryName($current)
 }
}
function Get-OM9SafeFile([string]$Root,[string]$Relative) {
 $name=$Relative.Replace('\','/')
 if(-not $name -or $name.Contains(':') -or $name.StartsWith('/') -or @($name.Split('/')|Where-Object{$_ -eq '..' -or $_ -eq '.' -or $_ -eq ''}).Count){throw "Invalid bundle path: $Relative"}
 $prefix=[IO.Path]::GetFullPath($Root).TrimEnd('\','/')+[IO.Path]::DirectorySeparatorChar
 $path=[IO.Path]::GetFullPath((Join-Path $Root $name))
 if(-not $path.StartsWith($prefix,[StringComparison]::OrdinalIgnoreCase)){throw 'Bundle path escapes module directory'}
 Assert-OM9PlainPath $path
 return $path
}
function Read-OM9Manifest([string]$Path) {
 $manifest=[IO.File]::ReadAllText($Path)|ConvertFrom-Json
 if($manifest.version -ne '0.0.2' -or $manifest.freecad -ne '1.1.4' -or @($manifest.files).Count -gt 10000){throw 'Unsupported or incomplete OM9 bundle manifest'}
 $seen=@{}
 foreach($row in $manifest.files){
  $key=([string]$row.path).Replace('\','/').ToLowerInvariant()
  if($seen.ContainsKey($key) -or $row.sha256 -notmatch '^[a-f0-9]{64}$'){throw 'Invalid or duplicate bundle file record'}
  $seen[$key]=$true
 }
 foreach($required in @('initgui.py','bin/openmatrix9gui.pyd','bin/om9threedmimportworker.exe')){if(-not $seen.ContainsKey($required)){throw "Required bundled file missing: $required"}}
 return $manifest
}
function Get-OM9PayloadHash($Manifest) {
 $rows=@($Manifest.files|ForEach-Object{([string]$_.path).Replace('\','/').ToLowerInvariant()+' '+$_.sha256}|Sort-Object)
 $sha=[Security.Cryptography.SHA256]::Create()
 try{return ([BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes(($rows -join "`n"))))).Replace('-','').ToLowerInvariant()}
 finally{$sha.Dispose()}
}
function Test-OM9HostProcess($Process,[string]$HostExecutable,[string]$NativeTarget='') {
 if(-not $HostExecutable -or -not $Process.Path){return $true}
 if([string]::Equals($Process.Path,$HostExecutable,[StringComparison]::OrdinalIgnoreCase)){return $true}
 # An alternate host can share the same plugin folder; check its loaded module too.
 if($NativeTarget){try{foreach($module in $Process.Modules){if([string]::Equals($module.FileName,$NativeTarget,[StringComparison]::OrdinalIgnoreCase)){return $true}}}catch{return $true}}
 return $false
}
function Get-OM9RunningFreeCAD([string]$HostExecutable='', [string]$NativeTarget='') {
 return @(Get-Process -Name FreeCAD -ErrorAction SilentlyContinue|Where-Object{Test-OM9HostProcess $_ $HostExecutable $NativeTarget})
}
function Invoke-OM9Install([string]$BundleRoot,[string]$UserHome,[scriptblock]$BeforePromote=$null,[string]$HostExecutable='') {
 $bundle=[IO.Path]::GetFullPath($BundleRoot);$payload=Join-Path $bundle 'OpenMatrix9'
 $manifestPath=Join-Path $bundle 'launcher-manifest.json'
 Assert-OM9PlainPath $bundle
 $manifest=Read-OM9Manifest $manifestPath;$manifestHash=Get-OM9Hash $manifestPath
 $versionRoot=Join-Path ([IO.Path]::GetFullPath($UserHome)) 'v1-1'
 $destination=Join-Path $versionRoot 'Mod/OpenMatrix9';Assert-OM9PlainPath $destination
 foreach($row in $manifest.files){
  $source=Get-OM9SafeFile $payload $row.path
  if(-not [IO.File]::Exists($source) -or (Get-OM9Hash $source) -ne $row.sha256){throw "Bundled file is missing or damaged: $($row.path)"}
 }
 $payloadHash=Get-OM9PayloadHash $manifest
 $marker=Join-Path $destination '.om9-install.json';$same=$false;$stored=$null
 if([IO.File]::Exists($marker)){
  try{$stored=[IO.File]::ReadAllText($marker)|ConvertFrom-Json;$same=($stored.payload_sha256 -eq $payloadHash -or (-not $stored.payload_sha256 -and $stored.manifest_sha256 -eq $manifestHash))}catch{$same=$false}
 }
 # Legacy install markers use the previous cached manifest as complete inventory.
 if(-not $same -and -not $stored.payload_sha256){
  try{
   $oldPath=Get-OM9SafeFile (Join-Path $versionRoot 'OM9-launcher') 'launcher-manifest.json'
   if([IO.File]::Exists($oldPath)){$same=((Get-OM9PayloadHash (Read-OM9Manifest $oldPath)) -eq $payloadHash)}
  }catch{$same=$false}
 }
 # Compare the complete file set before skipping, then verify every current byte.
 if($same){
  foreach($row in $manifest.files){$file=Get-OM9SafeFile $destination $row.path;if(-not [IO.File]::Exists($file) -or (Get-OM9Hash $file) -ne $row.sha256){$same=$false;break}}
 }
 if($same){return [pscustomobject]@{updated=$false;destination=$destination;backup=$null;manifest=$manifest;manifest_sha256=$manifestHash}}
 if([IO.Directory]::Exists($destination) -and @(Get-OM9RunningFreeCAD $HostExecutable (Join-Path $destination 'bin/OpenMatrix9Gui.pyd')).Count){throw 'Please close the FreeCAD instance using this OM9 installation before updating it. Save your work first.'}
 $id=[guid]::NewGuid().ToString('N')
 # Stages and backups are outside Mod: FreeCAD must never discover them as workbenches.
 $stage=Join-Path $versionRoot "OM9-staging/$id/OpenMatrix9"
 $backup=$null;Assert-OM9PlainPath $stage;[IO.Directory]::CreateDirectory($stage)|Out-Null
 foreach($row in $manifest.files){
  $target=Get-OM9SafeFile $stage $row.path
  [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($target))|Out-Null
  [IO.File]::Copy((Get-OM9SafeFile $payload $row.path),$target,$false)
  if((Get-OM9Hash $target) -ne $row.sha256){throw "Staged file verification failed: $($row.path)"}
 }
 [IO.File]::WriteAllText((Join-Path $stage '.om9-install.json'),(@{version=$manifest.version;manifest_sha256=$manifestHash;payload_sha256=$payloadHash}|ConvertTo-Json))
 [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($destination))|Out-Null
 try {
  if([IO.Directory]::Exists($destination)){
   $backup=Join-Path $versionRoot "OM9-backups/OpenMatrix9-$id";Assert-OM9PlainPath $backup
   [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($backup))|Out-Null
   [IO.Directory]::Move($destination,$backup)
  }
  if($BeforePromote){&$BeforePromote}
  [IO.Directory]::Move($stage,$destination)
 } catch {
  if($backup -and [IO.Directory]::Exists($backup) -and -not [IO.Directory]::Exists($destination)){[IO.Directory]::Move($backup,$destination)}
  throw
 }
 return [pscustomobject]@{updated=$true;destination=$destination;backup=$backup;manifest=$manifest;manifest_sha256=$manifestHash}
}
function Copy-OM9CacheFile([string]$Source,[string]$Target) {[IO.File]::Copy($Source,$Target,$true)}
function Invoke-OM9BundleInstall([string]$BundleRoot,[string]$om9UserRoot,[string]$Cache,[string]$HostExecutable) {
 $manifest=Read-OM9Manifest (Join-Path $BundleRoot 'launcher-manifest.json')
 $names=@('Start-OM9.ps1','Start-OM9.cmd','Start-OM9.FCMacro','OpenMatrix9.ico','README.md')
 # Preflight the whole bootstrap before replacing any installed plugin or cache file.
 foreach($name in $names){
  $rows=@($manifest.launcher_files|Where-Object{$_.path -ceq $name})
  $file=Get-OM9SafeFile $BundleRoot $name
  if($rows.Count -ne 1 -or $rows[0].sha256 -notmatch '^[a-f0-9]{64}$' -or -not [IO.File]::Exists($file) -or (Get-OM9Hash $file) -ne $rows[0].sha256){throw "Launcher bundle is incomplete or damaged: $name"}
 }
 $transaction=Join-Path $om9UserRoot ('v1-1/OM9-staging/'+[guid]::NewGuid().ToString('N')+'/launcher')
 Assert-OM9PlainPath $transaction;[IO.Directory]::CreateDirectory($transaction)|Out-Null
 $names+=@('launcher-manifest.json');$previous=@{};$changed=@();$install=$null
 foreach($name in $names){
  $file=Get-OM9SafeFile $BundleRoot $name;$staged=Join-Path $transaction $name
  [IO.File]::Copy($file,$staged,$false)
  if((Get-OM9Hash $file) -ne (Get-OM9Hash $staged)){throw "Launcher staging failed: $name"}
  $target=Get-OM9SafeFile $Cache $name
  $previous[$name]=[IO.File]::Exists($target)
  if($previous[$name]){[IO.File]::Copy($target,(Join-Path $transaction ($name+'.previous')),$false)}
 }
 try {
  $install=Invoke-OM9Install -BundleRoot $BundleRoot -UserHome $om9UserRoot -HostExecutable $HostExecutable
  foreach($name in $names){
   $changed+=@($name)
   Copy-OM9CacheFile (Join-Path $transaction $name) (Join-Path $Cache $name)
  }
 } catch {
  $failure=$_
  foreach($name in $changed){
   $target=Join-Path $Cache $name
   if($previous[$name]){[IO.File]::Copy((Join-Path $transaction ($name+'.previous')),$target,$true)}
   elseif([IO.File]::Exists($target)){[IO.File]::Delete($target)}
  }
  if($install -and $install.updated){
   [IO.Directory]::Move($install.destination,(Join-Path $transaction 'failed-plugin'))
   if($install.backup){[IO.Directory]::Move($install.backup,$install.destination)}
  }
  throw $failure
 }
 return $install
}
function Get-OM9Host([string]$Requested,[string]$om9UserRoot) {
 $exe=$Requested
 if(-not $exe){$exe=Join-Path $env:ProgramFiles 'FreeCAD 1.1/bin/FreeCAD.exe'}
 $exe=[IO.Path]::GetFullPath($exe);$python=Join-Path ([IO.Path]::GetDirectoryName($exe)) 'python.exe'
 if(-not [IO.File]::Exists($exe) -or -not [IO.File]::Exists($python)){throw 'Install official FreeCAD 1.1.4 Windows x64 first. See README.md for the download link.'}
 $info=New-Object Diagnostics.ProcessStartInfo
 $info.FileName=$python;$info.Arguments='-c "import FreeCAD as A,json;print(json.dumps(A.Version()[:3]))"'
 $info.UseShellExecute=$false;$info.CreateNoWindow=$true;$info.RedirectStandardOutput=$true;$info.RedirectStandardError=$true
 foreach($key in @('PYTHONHOME','PYTHONPATH','QT_PLUGIN_PATH','QT_QPA_PLATFORM_PLUGIN_PATH')){$info.EnvironmentVariables.Remove($key)}
 $info.EnvironmentVariables['FREECAD_USER_HOME']=Join-Path $om9UserRoot 'v1-1'
 $info.EnvironmentVariables['PATH']=[IO.Path]::GetDirectoryName($exe)+';'+$env:SystemRoot+'\System32;'+$env:SystemRoot
 $probe=[Diagnostics.Process]::Start($info)
 try {
  if(-not $probe.WaitForExit(30000)){$probe.Kill();throw 'FreeCAD version probe timed out'}
  $text=$probe.StandardOutput.ReadToEnd();$errors=$probe.StandardError.ReadToEnd()
  if($probe.ExitCode -ne 0){throw ('FreeCAD version probe failed: '+$errors)}
  $last=@($text -split '[\r\n]'|Where-Object{$_})[-1];$version=$last|ConvertFrom-Json
  if(($version -join '.') -ne '1.1.4'){throw ('This OM9 bundle requires FreeCAD 1.1.4; found '+($version -join '.'))}
 } finally{$probe.Dispose()}
 return $exe
}
function New-OM9Shortcut([string]$Cache,[string]$Desktop,[string]$HostPath,[string]$om9UserRoot,[bool]$CustomHome,[switch]$TrustedKnownFolder) {
 # The Windows Known Folder may be a OneDrive cloud reparse directory. Only a
 # single shortcut is written there; no recursive copy, move or cleanup occurs.
 if(-not $Desktop){$Desktop=[Environment]::GetFolderPath('Desktop')}
 elseif(-not $TrustedKnownFolder){Assert-OM9PlainPath $Desktop}
 [IO.Directory]::CreateDirectory($Desktop)|Out-Null
 $path=Join-Path $Desktop 'OpenMatrix9.lnk'
 if([IO.File]::Exists($path)){
  $old=Join-Path $Cache ('shortcut-backup-'+[guid]::NewGuid().ToString('N')+'.lnk')
  [IO.File]::Copy($path,$old,$false)
 }
 $shell=New-Object -ComObject WScript.Shell
 $link=$shell.CreateShortcut($path)
 $link.TargetPath=Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
 $link.Arguments='-NoLogo -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "'+(Join-Path $Cache 'Start-OM9.ps1')+'" -Installed -FreeCADPath "'+$HostPath+'"'
 if($CustomHome){$link.Arguments+=' -UserHome "'+$om9UserRoot+'"'}
 $link.WorkingDirectory=$Cache;$link.IconLocation=(Join-Path $Cache 'OpenMatrix9.ico')+',0';$link.Description='Start OpenMatrix9 in FreeCAD 1.1.4';$link.Save()
 return $path
}
function Get-OM9ProgramsFolder {return [Environment]::GetFolderPath('Programs')}
function New-OM9StartMenuShortcut([string]$Cache,[string]$Menu,[string]$HostPath,[string]$om9UserRoot,[bool]$CustomHome) {
 $knownFolder=-not $Menu
 if($knownFolder){$Menu=Join-Path (Get-OM9ProgramsFolder) 'OpenMatrix9'}
 return New-OM9Shortcut $Cache $Menu $HostPath $om9UserRoot $CustomHome -TrustedKnownFolder:$knownFolder
}
function Find-OM9Update($Releases,[string]$CurrentVersion,[int]$CurrentRevision) {
 $current=[version]$CurrentVersion;$best=$null
 foreach($release in $Releases){
  if($release.draft -or $release.tag_name -notmatch '^v([0-9]+\.[0-9]+\.[0-9]+)$'){continue}
  $number=$Matches[1];$version=[version]$number
  $page='https://github.com/MYuitsu/OpenMatrix9/releases/tag/v'+$number
  if($release.html_url -cne $page){continue}
  foreach($asset in $release.assets){
   $pattern='^OpenMatrix9-'+[regex]::Escape($number)+'-SetupLauncher(?:-r([1-9][0-9]{0,3}))?-Windows-x64\.zip$'
   if($asset.name -notmatch $pattern){continue}
   $revision=if($Matches[1]){[int]$Matches[1]}else{1}
   $download='https://github.com/MYuitsu/OpenMatrix9/releases/download/v'+$number+'/'+$asset.name
   if($asset.browser_download_url -cne $download){continue}
   if($version -lt $current -or ($version -eq $current -and $revision -le $CurrentRevision)){continue}
   if(-not $best -or $version -gt [version]$best.version -or ($version -eq [version]$best.version -and $revision -gt $best.revision)){
    $best=[pscustomobject]@{version=$number;revision=$revision;prerelease=[bool]$release.prerelease;release_url=$page;download_url=$download;key=($number+'/r'+$revision)}
   }
  }
 }
 return $best
}
function Get-OM9ReleaseFeed {
 [Net.ServicePointManager]::SecurityProtocol=[Net.SecurityProtocolType]::Tls12
 $request=[Net.HttpWebRequest]::Create('https://api.github.com/repos/MYuitsu/OpenMatrix9/releases?per_page=20')
 $request.UserAgent='OpenMatrix9-Windows-Launcher';$request.Accept='application/vnd.github+json'
 $request.Timeout=3000;$request.ReadWriteTimeout=3000;$request.AllowAutoRedirect=$false
 $response=$null;$reader=$null
 try {
  $response=$request.GetResponse();$reader=New-Object IO.StreamReader($response.GetResponseStream())
  $text=New-Object Text.StringBuilder;$buffer=New-Object char[] 4096
  while(($count=$reader.Read($buffer,0,$buffer.Length)) -gt 0){
   if($text.Length+$count -gt 1048576){throw 'Update response too large'}
   $text.Append($buffer,0,$count)|Out-Null
  }
  return ($text.ToString()|ConvertFrom-Json)
 } finally{if($reader){$reader.Dispose()};if($response){$response.Dispose()}}
}
function Show-OM9Update($Update) {
 Add-Type -AssemblyName System.Windows.Forms
 $kind=if($Update.prerelease){' (experimental)'}else{''}
 $message='OpenMatrix9 '+$Update.version+' / launcher r'+$Update.revision+$kind+' is available.'+[Environment]::NewLine+[Environment]::NewLine+'Download and extract the new package, then run Start-OM9.cmd to update. Save and close OM9 first if the plugin changes.'+[Environment]::NewLine+[Environment]::NewLine+'Open the download page?'
 $answer=[Windows.Forms.MessageBox]::Show($message,'OpenMatrix9 update',[Windows.Forms.MessageBoxButtons]::YesNo,[Windows.Forms.MessageBoxIcon]::Information,[Windows.Forms.MessageBoxDefaultButton]::Button2)
 if($answer -eq [Windows.Forms.DialogResult]::Yes){Start-Process $Update.release_url|Out-Null}
 return $true
}
function Invoke-OM9UpdateCheck([string]$Cache,[string]$CurrentVersion,[int]$CurrentRevision,[bool]$Silent) {
 $hash=Get-OM9Hash (Join-Path $Cache 'Start-OM9.ps1')
 $mutex=New-Object Threading.Mutex($false,('Local\OM9-Update-'+$hash));$locked=$false
 try {
  $locked=$mutex.WaitOne(0);if(-not $locked){return [pscustomobject]@{status='busy'}}
  $path=Join-Path $Cache 'update-check.json';$notified=$null
  if([IO.File]::Exists($path)){try{$notified=([IO.File]::ReadAllText($path)|ConvertFrom-Json).notified}catch{}}
  $result=[ordered]@{checked_at=[DateTime]::UtcNow.ToString('o');current_version=$CurrentVersion;current_revision=$CurrentRevision;status='current';notified=$notified;update=$null}
  try {
   $update=Find-OM9Update (Get-OM9ReleaseFeed) $CurrentVersion $CurrentRevision
   if($update){
    $result.status='available';$result.update=$update
    if(-not $Silent -and $notified -ne $update.key -and (Show-OM9Update $update)){$result.notified=$update.key}
   }
  } catch {$result.status='unavailable';$result.error=$_.Exception.Message}
  [IO.File]::WriteAllText($path,($result|ConvertTo-Json -Depth 5))
  return [pscustomobject]$result
 } finally{if($locked){$mutex.ReleaseMutex()};$mutex.Dispose()}
}
function Start-OM9 {
 $customHome=[bool]$UserHome
 $om9UserRoot=if($customHome){[IO.Path]::GetFullPath($UserHome)}else{Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'FreeCAD'}
 [IO.Directory]::CreateDirectory($om9UserRoot)|Out-Null
 $cache=Join-Path $om9UserRoot 'v1-1/OM9-launcher';Assert-OM9PlainPath $cache;[IO.Directory]::CreateDirectory($cache)|Out-Null
 $script:OM9Log=Join-Path $cache 'launcher.log'
 if($CheckUpdatesOnly){
  $manifest=Read-OM9Manifest (Join-Path $cache 'launcher-manifest.json')
  $revision=if($manifest.launcher_revision){[int]$manifest.launcher_revision}else{1}
  Invoke-OM9UpdateCheck $cache $manifest.version $revision ([bool]$NoDialogs)|Out-Null
  return
 }
 $lockHash=[BitConverter]::ToString([Security.Cryptography.SHA256]::Create().ComputeHash([Text.Encoding]::UTF8.GetBytes($om9UserRoot.ToLowerInvariant()))).Replace('-','')
 $mutex=New-Object Threading.Mutex($false,('Local\OM9-Launcher-'+$lockHash));$locked=$false
 try {
  $locked=$mutex.WaitOne(0);if(-not $locked){throw 'Another OM9 launcher is already running'}
  $hostExe=Get-OM9Host $FreeCADPath $om9UserRoot
  $destination=Join-Path $om9UserRoot 'v1-1/Mod/OpenMatrix9'
  if($Installed){
   $manifest=Read-OM9Manifest (Join-Path $cache 'launcher-manifest.json')
   foreach($row in $manifest.files){$file=Get-OM9SafeFile $destination $row.path;if(-not [IO.File]::Exists($file) -or (Get-OM9Hash $file) -ne $row.sha256){throw 'OM9 installation is incomplete. Run Start-OM9.cmd from the downloaded bundle again.'}}
   $status='already installed'
  } else {
   $install=Invoke-OM9BundleInstall $BundleRoot $om9UserRoot $cache $hostExe
   $status=if($install.updated){'installed/updated'}else{'up to date'}
  }
  if(-not $NoShortcut -and -not $Installed){
   $shortcut=New-OM9Shortcut $cache $DesktopPath $hostExe $om9UserRoot $customHome
   $menuShortcut=New-OM9StartMenuShortcut $cache $StartMenuPath $hostExe $om9UserRoot $customHome
   Write-Host "Desktop: $shortcut";Write-Host "Start Menu: $menuShortcut"
  }
  [IO.File]::AppendAllText($script:OM9Log,([DateTime]::Now.ToString('s')+' OM9 '+$status+'; '+$destination+[Environment]::NewLine))
  Write-Host "OM9 $status."
  if($InstallOnly){return}
  foreach($key in @('PYTHONHOME','PYTHONPATH','QT_PLUGIN_PATH','QT_QPA_PLATFORM_PLUGIN_PATH')){[Environment]::SetEnvironmentVariable($key,$null,'Process')}
  $env:PATH=[IO.Path]::GetDirectoryName($hostExe)+';'+$env:SystemRoot+'\System32;'+$env:SystemRoot
  if($customHome){$env:FREECAD_USER_HOME=Join-Path $om9UserRoot 'v1-1'}else{[Environment]::SetEnvironmentVariable('FREECAD_USER_HOME',$null,'Process')}
  $env:OM9_LAUNCH_REPORT=Join-Path $cache ('start-'+[guid]::NewGuid().ToString('N')+'.json')
  $macro=Join-Path $cache 'Start-OM9.FCMacro'
  $style=if($env:OM9_LAUNCH_TEST_HIDDEN -eq '1'){'Hidden'}else{'Normal'}
  $process=Start-Process -FilePath $hostExe -ArgumentList ('"'+$macro+'"') -WorkingDirectory ([IO.Path]::GetDirectoryName($hostExe)) -WindowStyle $style -PassThru
  Write-Host "Opening OpenMatrix9. PID=$($process.Id)"
  [IO.File]::AppendAllText($script:OM9Log,('PID='+$process.Id+'; startup report='+$env:OM9_LAUNCH_REPORT+[Environment]::NewLine))
  if(-not $SkipUpdateCheck){
   try {
    $powershell=Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
    $arguments='-NoLogo -NoProfile -ExecutionPolicy Bypass -File "'+(Join-Path $cache 'Start-OM9.ps1')+'" -CheckUpdatesOnly'
    if($customHome){$arguments+=' -UserHome "'+$om9UserRoot+'"'}
    if($NoDialogs){$arguments+=' -NoDialogs'}
    Start-Process $powershell -ArgumentList $arguments -WindowStyle Hidden|Out-Null
   } catch {[IO.File]::AppendAllText($script:OM9Log,('Update check could not start: '+$_.Exception.Message+[Environment]::NewLine))}
  }
 } finally{if($locked){$mutex.ReleaseMutex()};$mutex.Dispose()}
}
if($MyInvocation.InvocationName -ne '.'){
 try{Start-OM9}catch{
  $message=$_.Exception.Message
  if($script:OM9Log){[IO.File]::AppendAllText($script:OM9Log,([DateTime]::Now.ToString('s')+' ERROR '+$message+[Environment]::NewLine))}
  Write-Host ('OpenMatrix9 launcher: '+$message) -ForegroundColor Red
  if(-not $NoDialogs -and -not $CheckUpdatesOnly){Add-Type -AssemblyName System.Windows.Forms;[Windows.Forms.MessageBox]::Show($message,'OpenMatrix9',[Windows.Forms.MessageBoxButtons]::OK,[Windows.Forms.MessageBoxIcon]::Warning)|Out-Null}
  exit 1
 }
}
