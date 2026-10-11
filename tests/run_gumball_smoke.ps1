param(
 [string]$PluginRuntime='H:/FreeCAD-src/build/gumball-core-runtime-20261011',
 [Parameter(Mandatory=$true)][string]$OutputDirectory,
 [ValidateSet('Core','Persistence','Regression','All')][string]$Scope='All'
)
$ErrorActionPreference='Stop'
function Get-GumballPluginHash([string]$LiteralPath){
 $algorithm=[Security.Cryptography.SHA256]::Create();$stream=[IO.File]::OpenRead($LiteralPath)
 try {return [BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace('-','')}
 finally {$stream.Dispose();$algorithm.Dispose()}
}
$hostRoot=Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$runner=Join-Path (Split-Path $hostRoot -Parent) 'build/run-om9-plugin.ps1'
$cases=@()
if($Scope -in @('Core','All')){
 $cases+=@(
  @{Name='geometry';Macro='gumball_geometry_smoke.FCMacro'},
  @{Name='commands';Macro='gumball_commands_smoke.FCMacro'},
  @{Name='ownership';Macro='gumball_ownership_smoke.FCMacro'},
  @{Name='lifecycle';Macro='gumball_lifecycle_smoke.FCMacro'},
  @{Name='reentrancy';Macro='gumball_reentrancy_smoke.FCMacro'},
  @{Name='cancel-publication';Macro='gumball_reentrancy_smoke.FCMacro';Cancel='1'},
  @{Name='core-dpi1';Macro='gumball_core_smoke.FCMacro';Dpi='1'},
  @{Name='core-dpi150';Macro='gumball_core_smoke.FCMacro';Dpi='1.5'}
 )
}
if($Scope -in @('Persistence','All')){
 $cases+=@(@{Name='warm';Macro='gumball_persistence_smoke.FCMacro'},@{Name='cold';Macro='gumball_core_cold_smoke.FCMacro'})
}
if($Scope -in @('Regression','All')){
 foreach($legacy in @('curve_smoke','core_mouse_smoke','layer_visibility_permission_smoke','history_smoke','three_dm_preservation_smoke')){
  $cases+=@{Name=$legacy;Macro='gumball_regression_smoke.FCMacro';Legacy=$legacy+'.FCMacro'}
 }
}
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$pluginFile=Join-Path $PluginRuntime 'bin/OpenMatrix9Gui.pyd'
$pluginHash=Get-GumballPluginHash $pluginFile
$summary=@();$oldDpi=$env:QT_SCALE_FACTOR;$oldWarm=$env:OM9_GUMBALL_WARM;$oldLegacy=$env:OM9_GUMBALL_REGRESSION;$oldCancel=$env:OM9_GUMBALL_REENTRY_CANCEL
try{
 foreach($case in $cases){
  $caseOutput=Join-Path $OutputDirectory $case.Name
  $env:QT_SCALE_FACTOR=if($case.Dpi){$case.Dpi}else{'1'}
  $env:OM9_GUMBALL_WARM=Join-Path $OutputDirectory 'warm'
  $env:OM9_GUMBALL_REGRESSION=$case.Legacy
  $env:OM9_GUMBALL_REENTRY_CANCEL=$case.Cancel
  if($case.Name -eq 'three_dm_preservation_smoke'){
   $temporary=Join-Path $caseOutput 'temporary';New-Item -ItemType Directory -Path $temporary -Force | Out-Null
   foreach($name in @('om9-preservation.3dm','om9-inventory-unitless.3dm')){
    Copy-Item -LiteralPath (Join-Path (Split-Path $hostRoot -Parent) "build/3dm-preserve-native/test-temp/$name") -Destination $temporary
   }
  }
  & $runner -PluginRuntime $PluginRuntime -Macro (Join-Path $PSScriptRoot $case.Macro) -OutputDirectory $caseOutput -TimeoutSeconds 180
  $result=Get-Content -LiteralPath (Join-Path $caseOutput 'results.json') -Raw | ConvertFrom-Json
  $summary+=[pscustomobject]@{name=$case.Name;checks=@($result.checks).Count;module=$result.module;dpi=$env:QT_SCALE_FACTOR;output=$caseOutput}
 }
 if((Get-GumballPluginHash $pluginFile) -ne $pluginHash){throw 'Plugin changed during native suite'}
 @{ok=$true;runs=$summary;checks=($summary | Measure-Object -Property checks -Sum).Sum;plugin_sha256=$pluginHash} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $OutputDirectory 'suite.json') -Encoding utf8
 Write-Output "Gumball $Scope PASS; processes=$($summary.Count); checks=$(($summary | Measure-Object -Property checks -Sum).Sum)"
}finally{$env:QT_SCALE_FACTOR=$oldDpi;$env:OM9_GUMBALL_WARM=$oldWarm;$env:OM9_GUMBALL_REGRESSION=$oldLegacy;$env:OM9_GUMBALL_REENTRY_CANCEL=$oldCancel}
