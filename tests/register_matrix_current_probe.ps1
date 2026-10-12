param([Parameter(Mandatory)][ValidateSet('Prepare','Cleanup')][string]$Mode,[Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{32}$')][string]$RunId)
$ErrorActionPreference='Stop'
$ErrorView='NormalView'
if(Get-Variable -Name PSStyle -ErrorAction SilentlyContinue){$PSStyle.OutputRendering='PlainText'}
. (Join-Path $PSScriptRoot 'rhino5_probe_registry.ps1')
$root='H:/FreeCAD-src/build/matrix-layer-api-current'
$output=[IO.Path]::GetFullPath((Join-Path $root $RunId))
if(-not(Test-Path -LiteralPath $output -PathType Container)){throw 'Missing owned current-session output directory'}
$probe=[IO.Path]::GetFullPath('H:/FreeCAD-src/build/om9-layer-edits/tests/bin/current/Rhino5LayerCurrentProbe.rhp')
$pluginId='28a7966b-2c55-4fe4-8cb6-0394316dc4ae'
$path='Software\McNeel\Rhinoceros\5.0x64\Plug-ins\'+$pluginId
$recordPath=Join-Path $output 'current-probe-registration.json'
if($Mode -eq 'Prepare') {
 if(-not(Test-Path -LiteralPath $probe -PathType Leaf)){throw 'Current-session native probe missing'}
 foreach($hive in @([Microsoft.Win32.Registry]::CurrentUser,[Microsoft.Win32.Registry]::LocalMachine)){
  $key=$hive.OpenSubKey($path)
  if($null -ne $key){$key.Dispose();throw 'Current-session diagnostic GUID already registered; previous diagnostic must finish cleanup first'}
 }
 $base=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\McNeel\Rhinoceros\5.0x64\Plug-ins',$true)
 if($null -eq $base){throw 'Installed Rhino5 plugin root missing'}
 try {
  $key=$base.CreateSubKey($pluginId)
  try{$key.SetValue('Name','OM9 Matrix Current Session Probe');$key.SetValue('FileName',$probe);$key.SetValue('OM9OwnedDiagnosticRun',$RunId)}finally{$key.Dispose()}
 }finally{$base.Dispose()}
 [ordered]@{run_id=$RunId;guid=$pluginId;filename=$probe;machine_absent_before_load=$true;created=$true;removed=$false} | ConvertTo-Json | Set-Content -LiteralPath $recordPath -Encoding utf8
 Write-Output 'Current-session diagnostic GUID prepared; no Rhino process launched.'
} else {
 $record=Get-Content -LiteralPath $recordPath -Raw | ConvertFrom-Json
 if($record.run_id -ne $RunId -or $record.guid -ne $pluginId -or $record.filename -ne $probe -or -not $record.created -or -not $record.machine_absent_before_load){throw 'Current-session registration ownership mismatch'}
 $machine=Get-OM9ProbeRegistryFacts ([Microsoft.Win32.Registry]::LocalMachine) $path
 if($null -ne $machine){
  if(-not(Test-OM9ProbeRegistryFacts $machine $probe 'OM9 Matrix Current Session Probe' 'OM9LayerDirectApi' -CleanupCommand)){throw 'Generated current-session machine registration changed; refusing cleanup'}
  $machine | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $output 'machine-registration-generated.json') -Encoding utf8
  [Microsoft.Win32.Registry]::LocalMachine.DeleteSubKeyTree($path)
  $record | Add-Member -NotePropertyName machine_generated_removed -NotePropertyValue $true -Force
 }
 $key=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($path)
 try{if($null -eq $key -or $key.GetValue('OM9OwnedDiagnosticRun') -ne $RunId -or $key.GetValue('FileName') -ne $probe){throw 'Current-session HKCU ownership changed; refusing cleanup'}}finally{if($null -ne $key){$key.Dispose()}}
 [Microsoft.Win32.Registry]::CurrentUser.DeleteSubKeyTree($path)
 $record.removed=$true;$record | ConvertTo-Json | Set-Content -LiteralPath $recordPath -Encoding utf8
 Write-Output 'Only owned current-session diagnostic registration removed; Matrix process stays open.'
}
