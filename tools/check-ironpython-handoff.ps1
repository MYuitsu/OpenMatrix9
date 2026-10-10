$ErrorActionPreference='Stop'
$root='H:/FreeCAD-src/build/om9-layer-edits'
$iron='C:/Program Files/Rhinoceros 5 (64-bit)/Plug-ins/IronPython'
$output=Join-Path $root 'docs/validation/layer-session/IronPythonHandoffChecks.exe'
& rtk proxy C:/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe /nologo /target:exe /platform:anycpu "/reference:$iron/IronPython.dll" "/reference:$iron/Microsoft.Scripting.dll" "/reference:$iron/Microsoft.Dynamic.dll" "/out:$output" (Join-Path $root 'tests/IronPythonHandoffChecks.cs')
if($LASTEXITCODE -ne 0){throw 'IronPython execution runner compile failed'}
& rtk proxy $output (Join-Path $root 'tests/ironpython_handoff_checks.py')
if($LASTEXITCODE -ne 0){throw 'Installed IronPython handoff execution failed'}
