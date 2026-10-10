$ErrorActionPreference='Stop'
foreach($path in @('H:/FreeCAD-src/build/om9-layer-edits/tests/run_rhino5_layer_api_prerequisite.ps1','H:/FreeCAD-src/build/om9-layer-edits/tools/build-rhino5-layer-command-probe.ps1','H:/FreeCAD-src/build/om9-layer-edits/tests/register_matrix_current_probe.ps1','H:/FreeCAD-src/build/om9-layer-edits/tools/build-matrix-current-probe.ps1')) {
 $taskTokens=$null;$taskErrors=$null
 [void][System.Management.Automation.Language.Parser]::ParseFile($path,[ref]$taskTokens,[ref]$taskErrors)
 if($taskErrors.Count){throw ($taskErrors | Out-String)}
 Write-Output "PowerShell syntax PASS: $path"
}
