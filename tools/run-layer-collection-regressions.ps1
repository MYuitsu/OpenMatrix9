$ErrorActionPreference='Stop'
$module='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/Mod/OpenMatrix9'
$runtime='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/build/om9-plugin-runtime'
foreach($name in @('modeling_layer_storage_payload_smoke','modeling_layer_document_smoke','modeling_layer_export_smoke','modeling_layer_receive_smoke','modeling_layer_native_creation_smoke','modeling_layer_legacy_notes_smoke')) {
 Write-Output "Collection regression: $name"
 & rtk proxy pwsh -NoProfile -File H:/FreeCAD-src/build/run-om9-plugin.ps1 -PluginRuntime $runtime -Macro (Join-Path $module ('tests/'+$name+'.FCMacro')) -OutputDirectory ('H:/FreeCAD-src/build/shared-host-tests/layer-collection-final-'+$name) -TimeoutSeconds 240
 if($LASTEXITCODE -ne 0){throw "Collection regression failed: $name"}
}
Write-Output 'Scoped collection regressions passed; complete Session/Matrix acceptance pending.'
