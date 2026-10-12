$ErrorActionPreference='Stop'
$module='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/Mod/OpenMatrix9'
$runtime='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/build/om9-plugin-runtime'
foreach($name in @('modeling_clipboard_failure_smoke','modeling_ring_workflow_smoke','modeling_curve_workflow_smoke')) {
 & rtk proxy pwsh -NoProfile -File H:/FreeCAD-src/build/run-om9-plugin.ps1 -PluginRuntime $runtime -Macro (Join-Path $module ('tests/'+$name+'.FCMacro')) -OutputDirectory ('H:/FreeCAD-src/build/shared-host-tests/layer-transfer-remaining-'+$name) -TimeoutSeconds 240
 if($LASTEXITCODE -ne 0){throw "Transfer regression failed: $name"}
}
$build='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/build/layer-abi'
$proof='H:/FreeCAD-src/build/om9-layer-edits/docs/validation/layer-session'
& rtk proxy H:/FreeCAD-src/.pixi/envs/default/Library/bin/cmake.exe --build $build --config Release --parallel 14 *> (Join-Path $proof 'layer-transfer-native-build.log')
if($LASTEXITCODE -ne 0){throw 'Native tests incremental build failed'}
& rtk proxy H:/FreeCAD-src/.pixi/envs/default/Library/bin/ctest.exe --test-dir $build -C Release --output-on-failure *> (Join-Path $proof 'layer-transfer-native-tests.log')
if($LASTEXITCODE -ne 0){throw 'Native tests failed'}
Write-Output 'Scoped transfer regressions and native suites passed; full retained/Matrix acceptance pending.'
