param([ValidateRange(10,300)][int]$TimeoutSeconds=180,[switch]$ShowProbeUI)
$ErrorActionPreference='Stop'
$ErrorView='NormalView'
if(Get-Variable -Name PSStyle -ErrorAction SilentlyContinue){$PSStyle.OutputRendering='PlainText'}
$principal=[Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if(-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)){
 throw 'Run rhino5_verify_layer_command_api.py from Administrator Matrix. The tool token is not elevated; no Rhino process was started.'
}
. (Join-Path $PSScriptRoot 'rhino5_probe_registry.ps1')
$exe='C:/Program Files/Rhinoceros 5 (64-bit)/System/Rhino.exe'
$source='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/Mod/OpenMatrix9'
$script=Join-Path $source 'tests/rhino5_layer_api_prerequisite.py'
$probe='H:/FreeCAD-src/build/om9-layer-edits/tests/bin/Rhino5LayerCommandProbe.rhp'
$runId=[guid]::NewGuid().ToString('N')
$output=Join-Path 'H:/FreeCAD-src/build/matrix-layer-api-probe' $runId
foreach($p in @($exe,$script,$probe)){if(-not(Test-Path -LiteralPath $p -PathType Leaf)){throw "Missing probe input: $p"}}
New-Item -ItemType Directory -Path $output | Out-Null
# The native application may request elevation. A run-owned bootstrap carries
# explicit paths, so correctness does not depend on elevated environment inheritance.
$bootstrap=Join-Path $output 'run-owned-probe.py'
$idLiteral=ConvertTo-Json -InputObject $runId -Compress
$outputLiteral=ConvertTo-Json -InputObject $output -Compress
$scriptLiteral=ConvertTo-Json -InputObject $script -Compress
$bootstrapSource=@"
import os
os.environ['OM9_LAYER_API_RUN_ID'] = $idLiteral
os.environ['OM9_LAYER_API_OUTPUT'] = $outputLiteral
execfile($scriptLiteral, {'__name__': '__main__'})
"@
Set-Content -LiteralPath $bootstrap -Value $bootstrapSource -Encoding utf8
$previous=@{}
foreach($key in @('OM9_LAYER_API_RUN_ID','OM9_LAYER_API_OUTPUT')){$previous[$key]=[Environment]::GetEnvironmentVariable($key,'Process')}
$pluginId='d2f0f5d0-4e65-4b75-b4ef-8d6ca72be839'
$registryRoot='Software\McNeel\Rhinoceros\5.0x64\Plug-ins'
$registryPath=$registryRoot+'\'+$pluginId
$registrationOwned=$false;$process=$null
$registration=[ordered]@{run_id=$runId;path='HKEY_CURRENT_USER\'+$registryPath;filename=$probe;temporary=$true;created=$false;removed=$false}
$mutex=[Threading.Mutex]::new($false,'Local\OM9LayerApiProbe_d2f0f5d0')
$mutexOwned=$false
try {
 try {$mutexOwned=$mutex.WaitOne(0)}catch [Threading.AbandonedMutexException]{$mutexOwned=$true}
 if(-not $mutexOwned){throw 'Another owned diagnostic launcher is active; retry after it finishes'}
 # A normal diagnostic plugin owns its own commands. The temporary key is
 # ONLY this unused diagnostic GUID; no Matrix/default scheme settings change.
 $existing=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($registryPath)
 if($null -ne $existing){$existing.Dispose();throw 'Diagnostic GUID already registered in HKCU; refusing to replace an existing plugin'}
 $machine=Get-OM9ProbeRegistryFacts ([Microsoft.Win32.Registry]::LocalMachine) $registryPath
 if($null -ne $machine) {
  # Rhino5 elevated LoadPlugIn writes its own HKLM registration, even when
  # our bootstrap registered only HKCU. Recover only the exact known first
  # completed run, not an arbitrary preexisting installation.
  if(-not(Test-OM9ProbeRegistryFacts $machine $probe)){throw 'Existing HKLM diagnostic GUID is not our exact plugin; refusing cleanup'}
  $priorDirectory='H:/FreeCAD-src/build/matrix-layer-api-probe/efebe48877d84de3925dbaf8408ea6ca'
  $priorRun=Assert-OM9PriorProbeOwnership $priorDirectory $probe $pluginId
  $machine | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $output 'machine-registration-before.json') -Encoding utf8
  [Microsoft.Win32.Registry]::LocalMachine.DeleteSubKeyTree($registryPath)
  $registration.machine_previous_owned_run=$priorRun
  $registration.machine_previous_removed=$true
 }
 $registration.machine_absent_before_launch=$true
 $baseKey=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($registryRoot,$true)
 if($null -eq $baseKey){throw 'Installed Rhino5 per-user plugin root is missing'}
 try {
  $owned=$baseKey.CreateSubKey($pluginId)
  try {
   $owned.SetValue('Name','OM9 Layer API Command Probe')
   $owned.SetValue('FileName',[IO.Path]::GetFullPath($probe))
   $owned.SetValue('OM9OwnedDiagnosticRun',$runId)
   $registrationOwned=$true;$registration.created=$true
  } finally {$owned.Dispose()}
 } finally {$baseKey.Dispose()}
 $registration | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'registration.json') -Encoding utf8
 $env:OM9_LAYER_API_RUN_ID=$runId;$env:OM9_LAYER_API_OUTPUT=$output
 # The bootstrap queues its gate until the ScriptRunner finishes. It exits its
 # own isolated host only after the deferred terminal report is written.
 $arguments='/nosplash /notemplate /scheme=OM9LayerApi_'+$runId+' /runscript="_-RunPythonScript '+$bootstrap.Replace('\','/')+'"'
 # Interactive diagnostics may need the native plugin-load error dialog visible.
 # The user entrypoint opts in; background verifiers keep their hidden default.
 $windowStyle=if($ShowProbeUI){'Normal'}else{'Hidden'}
 $process=Start-Process -FilePath $exe -ArgumentList $arguments -WindowStyle $windowStyle -PassThru
 $launch=[ordered]@{run_id=$runId;pid=$process.Id;executable=$exe;arguments=$arguments;utc=[DateTime]::UtcNow.ToString('o');admin=$principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator);finished=$false;timed_out=$false;exit_code=$null;report_exists=$false;full_api_gate_passed=$false}
 $launch | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'launch.json') -Encoding utf8
 $deadline=[DateTime]::UtcNow.AddSeconds($TimeoutSeconds)
 while(-not $process.WaitForExit(1000)) {
  if([DateTime]::UtcNow -ge $deadline){
   $launch.timed_out=$true;$launch.report_exists=Test-Path -LiteralPath (Join-Path $output 'matrix-api-prerequisite.json')
   $launch | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'launch.json') -Encoding utf8
   throw "Owned Rhino probe timed out, PID=$($process.Id); output=$output"
  }
 }
 $launch.finished=$true;$launch.exit_code=$process.ExitCode
 $report=Join-Path $output 'matrix-api-prerequisite.json';$launch.report_exists=Test-Path -LiteralPath $report
 $launch | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'launch.json') -Encoding utf8
 if(-not $launch.report_exists){throw "Owned Rhino exited $($process.ExitCode) before probe report; output=$output"}
 $data=Get-Content -LiteralPath $report -Raw | ConvertFrom-Json
 if($data.pid -ne $process.Id -or $data.run_id -ne $runId){throw "Report does not belong to owned process: $report"}
 $launch.full_api_gate_passed=[bool]$data.full_api_gate_passed
 $launch | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'launch.json') -Encoding utf8
 if($process.ExitCode -ne 0 -or -not $data.ok -or -not $data.full_api_gate_passed){throw "Matrix API diagnostic failed: $report $($data.error)"}
 Write-Output "PASS Matrix command API diagnostic; pointer bytes=$($data.pointer_bytes); CLR=$($data.clr_version); command Undo/Redo and before-image rollback verified; report=$report; product handoff still unverified"
} finally {
 foreach($key in $previous.Keys){[Environment]::SetEnvironmentVariable($key,$previous[$key],'Process')}
 try {
 if($registrationOwned -and ($null -eq $process -or $process.HasExited)) {
  $machine=Get-OM9ProbeRegistryFacts ([Microsoft.Win32.Registry]::LocalMachine) $registryPath
  if($null -ne $machine) {
   if(-not $registration.machine_absent_before_launch -or -not(Test-OM9ProbeRegistryFacts $machine $probe)){throw 'Generated HKLM diagnostic registration changed; refusing cleanup'}
   $machine | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $output 'machine-registration-generated.json') -Encoding utf8
   [Microsoft.Win32.Registry]::LocalMachine.DeleteSubKeyTree($registryPath)
   $registration.machine_generated_removed=$true
  }
  $key=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($registryPath)
  try {
   if($null -eq $key -or $key.GetValue('OM9OwnedDiagnosticRun') -ne $runId -or $key.GetValue('FileName') -ne [IO.Path]::GetFullPath($probe)){throw 'Diagnostic registry ownership changed; refusing cleanup'}
  } finally {if($null -ne $key){$key.Dispose()}}
  # The absolute registry target is the fixed diagnostic GUID checked above;
  # never recurse through a scheme, plugin root or Matrix registration.
  [Microsoft.Win32.Registry]::CurrentUser.DeleteSubKeyTree($registryPath)
  $registration.removed=$true
 }
 } finally {
  $registration | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'registration.json') -Encoding utf8
  if($mutexOwned){$mutex.ReleaseMutex()};$mutex.Dispose()
 }
}
