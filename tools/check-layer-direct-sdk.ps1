$ErrorActionPreference='Stop'
$base='H:/FreeCAD-src/build/om9-layer-edits'
$sdk='C:/Program Files/Rhinoceros 5 (64-bit)/System/RhinoCommon.dll'
$iron='C:/Program Files/Rhinoceros 5 (64-bit)/Plug-ins/IronPython'
$proof=Join-Path $base 'docs/validation/layer-session'
$output=Join-Path $proof 'Rhino5LayerDirectSdkCheck.exe'
$compiler='C:/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe'
& rtk proxy $compiler /nologo /target:exe /platform:anycpu "/reference:$iron/IronPython.dll" "/reference:$iron/Microsoft.Scripting.dll" "/reference:$iron/Microsoft.Dynamic.dll" "/out:$output" (Join-Path $base 'tests/Rhino5LayerDirectSdkCheck.cs')
if($LASTEXITCODE -ne 0){throw 'Installed SDK/IronPython check compile failed'}
# Dependencies are test-only links next to this tool, not a cloned CAD runtime.
foreach($name in @('IronPython.dll','IronPython.Modules.dll','Microsoft.Scripting.dll','Microsoft.Dynamic.dll')) {
    $destination=Join-Path $proof $name
    if(-not(Test-Path -LiteralPath $destination)){New-Item -ItemType SymbolicLink -Path $destination -Target (Join-Path $iron $name) | Out-Null}
}
& rtk proxy $output $sdk (Join-Path $base 'tests/rhino5_verify_layer_api.py') (Join-Path $base 'tests/rhino5_layer_api_prerequisite.py') (Join-Path $base 'tests/rhino5_verify_layer_command_api.py') (Join-Path $base 'tests/rhino5_verify_matrix_current.py')
if($LASTEXITCODE -ne 0){throw 'Installed Rhino5 SDK/IronPython syntax check failed'}
