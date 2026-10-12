$ErrorActionPreference='Stop'
$root='H:/FreeCAD-src/build/om9-layer-edits'
$source=[IO.Path]::GetFullPath((Join-Path $root 'tests/Rhino5LayerCommandProbe.cs'))
$output=[IO.Path]::GetFullPath((Join-Path $root 'tests/bin/current/Rhino5LayerCurrentProbe.dll'))
$sdk=[IO.Path]::GetFullPath('C:/Program Files/Rhinoceros 5 (64-bit)/System/RhinoCommon.dll')
New-Item -ItemType Directory -Path (Split-Path $output) -Force | Out-Null
& rtk proxy 'C:/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe' /nologo /target:library /platform:x64 /define:OM9_CURRENT_SESSION_PROBE /reference:System.Drawing.dll /reference:System.Web.Extensions.dll "/reference:$sdk" "/out:$output" $source
if($LASTEXITCODE -ne 0){throw 'Current Matrix diagnostic SDK compilation failed'}
Copy-Item -LiteralPath $output -Destination ([IO.Path]::ChangeExtension($output,'.rhp'))
Write-Output "Current Matrix native diagnostic built: $output; actual current-session execution pending."
