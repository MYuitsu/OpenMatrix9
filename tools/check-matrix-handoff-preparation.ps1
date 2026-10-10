$ErrorActionPreference='Stop'
$root='H:/FreeCAD-src/build/om9-layer-edits'
$sdk='C:/Program Files/Rhinoceros 5 (64-bit)/System/RhinoCommon.dll'
$compiler=Join-Path $root 'docs/validation/layer-session/Rhino5LayerDirectSdkCheck.exe'
& rtk proxy $compiler $sdk (Join-Path $root 'tests/rhino5_verify_matrix_om9_handoff.py') (Join-Path $root 'tests/matrix_om9_handoff_support.py') (Join-Path $root 'tests/rhino5_verify_layer_command_api.py') (Join-Path $root 'tests/rhino5_inspect_matrix_handoff.py') (Join-Path $root 'tests/rhino5_recover_matrix_handoff.py')
if($LASTEXITCODE -ne 0){throw 'Installed IronPython handoff syntax failed'}
$tokens=$null;$errors=$null
[void][Management.Automation.Language.Parser]::ParseFile((Join-Path $root 'tests/launch_matrix_om9_handoff.ps1'),[ref]$tokens,[ref]$errors)
if($errors.Count){throw ($errors | Out-String)}
Write-Output 'Two-application launcher PowerShell syntax PASS; native Matrix handoff pending.'
