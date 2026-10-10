# Diagnostic-only registry ownership checks; no product layer policy.
function Test-OM9ProbeRegistryFacts($Facts,[string]$Probe,[string]$Name='OM9 Layer API Command Probe',[string]$Prefix='OM9LayerApi',[switch]$CleanupCommand) {
 if($null -eq $Facts -or $Facts.name -cne $Name -or $Facts.dotnet -ne 1 -or $Facts.type -ne 16){return $false}
 try {if([IO.Path]::GetFullPath($Facts.filename) -ine [IO.Path]::GetFullPath($Probe)){return $false}}catch{return $false}
 $expected=@('PrepareFixture','ApplyProbe','FailLayerProbe','FailGeometryProbe','Sample') | ForEach-Object {$Prefix+$_}
 if($CleanupCommand){$expected+=@($Prefix+'Cleanup')}
 if(@($Facts.subkeys).Count -ne 2 -or 'PlugIn' -cnotin $Facts.subkeys -or 'CommandList' -cnotin $Facts.subkeys){return $false}
 if($Facts.commands.Count -ne $expected.Count){return $false}
 foreach($name in $expected){if(-not $Facts.commands.Contains($name) -or $Facts.commands[$name] -cne ('2;'+$name)){return $false}}
 return $true
}
function Get-OM9ProbeRegistryFacts($Hive,[string]$Path) {
 $key=$Hive.OpenSubKey($Path)
 if($null -eq $key){return $null}
 try {
  $commands=@{};$commandKey=$key.OpenSubKey('CommandList');$pluginKey=$key.OpenSubKey('PlugIn')
  try {
   if($null -ne $commandKey){foreach($name in $commandKey.GetValueNames()){$commands[$name]=$commandKey.GetValue($name)}}
   return @{name=$key.GetValue('Name');dotnet=$key.GetValue('IsDotNETPlugIn');type=$key.GetValue('Type');filename=if($null -ne $pluginKey){$pluginKey.GetValue('FileName')}else{$null};subkeys=$key.GetSubKeyNames();commands=$commands}
  }finally{if($null -ne $commandKey){$commandKey.Dispose()};if($null -ne $pluginKey){$pluginKey.Dispose()}}
 }finally{$key.Dispose()}
}
function Assert-OM9PriorProbeOwnership([string]$Directory,[string]$Probe,[string]$PluginId) {
 $registration=Get-Content -LiteralPath (Join-Path $Directory 'registration.json') -Raw | ConvertFrom-Json
 $launch=Get-Content -LiteralPath (Join-Path $Directory 'launch.json') -Raw | ConvertFrom-Json
 $report=Get-Content -LiteralPath (Join-Path $Directory 'matrix-api-prerequisite.json') -Raw | ConvertFrom-Json
 if(-not $registration.created -or -not $launch.finished -or $launch.timed_out -or $launch.run_id -ne $registration.run_id -or $report.run_id -ne $launch.run_id -or $report.pid -ne $launch.pid -or $report.native_command_registration.owner_id -ne $PluginId){throw 'Prior diagnostic ownership proof is incomplete; refusing registry cleanup'}
 if([IO.Path]::GetFullPath($registration.filename) -ine [IO.Path]::GetFullPath($Probe) -or @($report.command_probe_assemblies | Where-Object {[IO.Path]::GetFullPath($_.path) -ieq [IO.Path]::GetFullPath($Probe)}).Count -ne 1){throw 'Prior diagnostic plugin path does not match; refusing registry cleanup'}
 $prior=Get-Process -Id $launch.pid -ErrorAction SilentlyContinue
 if($null -ne $prior){throw "Prior diagnostic PID=$($launch.pid) is live or reused; refusing registry cleanup"}
 return $registration.run_id
}
