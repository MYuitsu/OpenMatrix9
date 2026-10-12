$ErrorActionPreference='Stop'
$base='H:/FreeCAD-src/build/om9-layer-edits'
$proof=Join-Path $base 'docs/validation/layer-session'
$iron='C:/Program Files/Rhinoceros 5 (64-bit)/Plug-ins/IronPython'
$tool=[IO.Path]::GetFullPath((Join-Path $proof 'Rhino5CommandLauncherChecks.exe'))
$inputScript=[IO.Path]::GetFullPath((Join-Path $base 'tests/rhino5_verify_matrix_current.py'))
& rtk proxy C:/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe /nologo /target:exe "/reference:$iron/IronPython.dll" "/reference:$iron/Microsoft.Scripting.dll" "/reference:$iron/Microsoft.Dynamic.dll" "/out:$tool" ([IO.Path]::GetFullPath((Join-Path $base 'tests/Rhino5CommandLauncherChecks.cs')))
if($LASTEXITCODE -ne 0){throw 'Launcher boundary test compile failed'}
& rtk proxy $tool $inputScript (Join-Path $proof 'command-launcher-unicode.log')
if($LASTEXITCODE -ne 0){throw 'Launcher boundary test failed'}
