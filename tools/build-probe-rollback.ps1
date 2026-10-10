$ErrorActionPreference='Stop'
$source=[IO.Path]::GetFullPath('H:/FreeCAD-src/build/om9-layer-edits/tests/Rhino5LayerCommandProbe.cs')
$output=[IO.Path]::GetFullPath('H:/FreeCAD-src/build/om9-layer-edits/docs/validation/layer-session/rollback-boundary/Rhino5LayerCommandProbe.dll')
$sdk=[IO.Path]::GetFullPath('C:/Program Files/Rhinoceros 5 (64-bit)/System/RhinoCommon.dll')
& rtk proxy 'C:/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe' /nologo /target:library /platform:x64 /reference:System.Drawing.dll /reference:System.Web.Extensions.dll "/reference:$sdk" "/out:$output" $source
if($LASTEXITCODE -ne 0){throw 'Rollback diagnostic SDK compilation failed'}
Write-Output "Rollback diagnostic compiles against installed Rhino5 SDK: $output; live probe binary not replaced."
