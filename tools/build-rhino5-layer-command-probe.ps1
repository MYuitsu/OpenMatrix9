$ErrorActionPreference='Stop'
$source=[IO.Path]::GetFullPath('H:/FreeCAD-src/build/om9-layer-edits/tests/Rhino5LayerCommandProbe.cs')
$output=[IO.Path]::GetFullPath('H:/FreeCAD-src/build/om9-layer-edits/tests/bin/Rhino5LayerCommandProbe.dll')
$compiler='C:/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe'
$sdk=[IO.Path]::GetFullPath('C:/Program Files/Rhinoceros 5 (64-bit)/System/RhinoCommon.dll')
New-Item -ItemType Directory -Path (Split-Path $output) -Force | Out-Null
& rtk proxy $compiler /nologo /target:library /platform:x64 /reference:System.Drawing.dll /reference:System.Web.Extensions.dll "/reference:$sdk" "/out:$output" $source
if($LASTEXITCODE -ne 0){throw 'Rhino5 command diagnostic compile failed'}
Copy-Item -LiteralPath $output -Destination ([IO.Path]::ChangeExtension($output,'.rhp'))
Write-Output "Diagnostic command probe built against installed Rhino5 SDK: $output; runtime still unverified."
