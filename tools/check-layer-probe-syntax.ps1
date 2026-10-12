$ErrorActionPreference='Stop'
$taskTokens=$null;$taskErrors=$null
$path='H:/FreeCAD-src/build/om9-layer-edits/tests/run_rhino5_layer_api_prerequisite.ps1'
$null=[System.Management.Automation.Language.Parser]::ParseFile($path,[ref]$taskTokens,[ref]$taskErrors)
if($taskErrors.Count){$taskErrors | Format-List | Out-String | Write-Error;throw 'Probe PowerShell parse failed'}
Write-Output 'Probe PowerShell syntax PASS; actual Administrator Matrix run still required.'
