$ErrorActionPreference='Stop'
$module='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/Mod/OpenMatrix9'
$runtime='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/build/om9-plugin-runtime'
foreach($name in @('modeling_layer_transfer_commands_smoke','modeling_layer_collection_smoke','modeling_layer_clipboard_smoke','modeling_layer_receive_smoke','modeling_layer_export_smoke','modeling_layer_document_smoke','modeling_layer_native_creation_smoke','modeling_clipboard_user_smoke','modeling_clipboard_failure_smoke','modeling_ring_workflow_smoke','modeling_curve_workflow_smoke')) {
 Write-Output "Transfer regression: $name"
 & rtk proxy pwsh -NoProfile -File H:/FreeCAD-src/build/run-om9-plugin.ps1 -PluginRuntime $runtime -Macro (Join-Path $module ('tests/'+$name+'.FCMacro')) -OutputDirectory ('H:/FreeCAD-src/build/shared-host-tests/layer-transfer-verified-'+$name) -TimeoutSeconds 240
 if($LASTEXITCODE -ne 0){throw "Transfer regression failed: $name"}
}
Write-Output 'Scoped public transfer regressions passed; full retained/Matrix acceptance pending.'
