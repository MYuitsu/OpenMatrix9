param(
 [Parameter(Mandatory=$true)][string]$HostRoot,
 [Parameter(Mandatory=$true)][string]$PluginRoot,
 [Parameter(Mandatory=$true)][string]$Output
)
$ErrorActionPreference='Stop'
$source=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$cases=@(
 @{name='geometry';macro='gumball_geometry_smoke.FCMacro'},
 @{name='commands';macro='gumball_commands_smoke.FCMacro'},
 @{name='ownership';macro='gumball_ownership_smoke.FCMacro'},
 @{name='review';macro='gumball_review_smoke.FCMacro'},
 @{name='lifecycle';macro='gumball_lifecycle_smoke.FCMacro'},
 @{name='reentrancy';macro='gumball_reentrancy_smoke.FCMacro'},
 @{name='cancel-publication';macro='gumball_reentrancy_smoke.FCMacro';cancel='1'},
 @{name='core-dpi1';macro='gumball_core_smoke.FCMacro';dpi='1'},
 @{name='core-dpi150';macro='gumball_core_smoke.FCMacro';dpi='1.5'}
)
foreach($dpi in @('1','1.5','2')){
 $cases+=@{name="four-view-dpi$dpi";macro='gumball_four_view_smoke.FCMacro';dpi=$dpi}
 $cases+=@{name="view-lifecycle-dpi$dpi";macro='gumball_view_lifecycle_smoke.FCMacro';dpi=$dpi}
}
foreach($case in @('settings','snappy','relocate_pointer','owner_scale','owner_callback','mesh_frame','visual')){
 $cases+=@{name=$case;macro="gumball_$($case)_smoke.FCMacro"}
}
$cases+=@{name='warm';macro='gumball_persistence_smoke.FCMacro'}
$cases+=@{name='cold';macro='gumball_core_cold_smoke.FCMacro'}
$cases+=@{name='parametric';macro='gumball_parametric_smoke.FCMacro'}
$cases+=@{name='component-pointer';macro='gumball_component_pointer_smoke.FCMacro'}
$cases+=@{name='associative-warm';macro='gumball_associative_scale_smoke.FCMacro'}
$cases+=@{name='associative-cold';macro='gumball_associative_scale_smoke.FCMacro';associativeCold=$true}
$cases+=@{name='frame-objects-warm';macro='gumball_frame_objects_smoke.FCMacro'}
$cases+=@{name='frame-objects-cold';macro='gumball_frame_objects_cold_smoke.FCMacro'}
$oldDpi=$env:QT_SCALE_FACTOR;$oldWarm=$env:OM9_GUMBALL_WARM;$oldCancel=$env:OM9_GUMBALL_REENTRY_CANCEL
$oldFrameWarm=$env:OM9_GUMBALL_FRAME_WARM;$oldAssociativeCold=$env:OM9_ASSOCIATIVE_COLD
$plugin=Join-Path $PluginRoot 'bin/OpenMatrix9Gui.pyd';$hash=(Get-FileHash -LiteralPath $plugin).Hash
$runs=@()
try{
 foreach($case in $cases){
  $env:QT_SCALE_FACTOR=if($case.dpi){$case.dpi}else{'1'}
  $env:OM9_GUMBALL_WARM=Join-Path $Output 'warm';$env:OM9_GUMBALL_REENTRY_CANCEL=$case.cancel
  $env:OM9_GUMBALL_FRAME_WARM=Join-Path $Output 'frame-objects-warm'
  $env:OM9_ASSOCIATIVE_COLD=if($case.associativeCold){Join-Path $Output 'associative-warm'}else{$null}
  $caseOutput=Join-Path $Output $case.name
  $result=@(& (Join-Path $PSScriptRoot 'run-native-case.ps1') -HostRoot $HostRoot -PluginRoot $PluginRoot -Macro (Join-Path $source "tests/$($case.macro)") -Output $caseOutput)|Select-Object -Last 1
  $runs+=@{name=$case.name;macro=$case.macro;dpi=$env:QT_SCALE_FACTOR;checks=@($result.checks).Count;module=$result.module;feature_ids=$result.feature_ids;output=$caseOutput}
 }
 if((Get-FileHash -LiteralPath $plugin).Hash -ne $hash){throw 'Plugin changed during Gumball suite'}
 $summary=@{ok=$true;runs=$runs;checks=($runs|Measure-Object -Property checks -Sum).Sum;plugin_sha256=$hash.ToLowerInvariant()}
 $summary|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $Output 'suite.json') -Encoding utf8
 Write-Output "Gumball native PASS; processes=$($runs.Count); checks=$($summary.checks)"
}finally{$env:QT_SCALE_FACTOR=$oldDpi;$env:OM9_GUMBALL_WARM=$oldWarm;$env:OM9_GUMBALL_REENTRY_CANCEL=$oldCancel;$env:OM9_GUMBALL_FRAME_WARM=$oldFrameWarm;$env:OM9_ASSOCIATIVE_COLD=$oldAssociativeCold}
