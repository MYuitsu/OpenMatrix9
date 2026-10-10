$ErrorActionPreference='Stop'
$root='H:/FreeCAD-src/build/om9-layer-edits'
$manifest=Join-Path $root 'bridge/rhino5/rust-core/Cargo.toml'
$runtime=Join-Path $root 'bridge/rhino5/runtime'
$workers=[Math]::Max(1,[Math]::Floor([Environment]::ProcessorCount*0.60))
New-Item -ItemType Directory -Path $runtime -Force | Out-Null
& rtk proxy cargo build --offline --locked --release --jobs $workers --manifest-path $manifest
if($LASTEXITCODE -ne 0){throw 'Matrix Rust bridge build failed'}
$rustBinary=Join-Path $root 'bridge/rhino5/rust-core/target/release/om9_rhino_layer.dll'
$rustHash=(Get-FileHash -LiteralPath $rustBinary -Algorithm SHA256).Hash.ToLowerInvariant()
$rustName='om9_rhino_layer_'+$rustHash.Substring(0,12)+'.dll'
$rustDestination=Join-Path $runtime $rustName
if(Test-Path -LiteralPath $rustDestination){
    if((Get-FileHash -LiteralPath $rustDestination -Algorithm SHA256).Hash.ToLowerInvariant() -ne $rustHash){throw 'Existing content-addressed Rust DLL differs; refusing replacement'}
}else{Copy-Item -LiteralPath $rustBinary -Destination $rustDestination}
$identity=Join-Path $runtime 'NativeIdentity.cs'
[IO.File]::WriteAllText($identity,('namespace OM9LayerTransfer { public static partial class NativeLayerApi { const string Dll="'+$rustName+'"; } }'))
$compiler='C:/Windows/Microsoft.NET/Framework64/v4.0.30319/csc.exe'
$sdk='C:/Program Files/Rhinoceros 5 (64-bit)/System/RhinoCommon.dll'
$sources=Get-ChildItem -LiteralPath (Join-Path $root 'bridge/rhino5/OM9LayerTransfer') -Filter '*.cs' -File
$sourceText=($sources | Sort-Object Name | ForEach-Object {[IO.File]::ReadAllText($_.FullName)}) -join "`n"
$sha=[Security.Cryptography.SHA256]::Create()
$sourceHash=([BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($sourceText+$rustHash)))).Replace('-','').ToLowerInvariant()
$adapterName='OM9LayerTransfer_'+$sourceHash.Substring(0,12)+'.dll'
& rtk proxy $compiler /nologo /target:library /platform:x64 /reference:System.Drawing.dll /reference:System.Web.Extensions.dll "/reference:$sdk" "/out:$runtime/$adapterName" $sources.FullName $identity
if($LASTEXITCODE -ne 0){throw 'Matrix RhinoCommon adapter build failed'}
$record=@{version=1;runtime=$runtime;adapter=$adapterName;adapter_sha256=(Get-FileHash -LiteralPath (Join-Path $runtime $adapterName) -Algorithm SHA256).Hash.ToLowerInvariant();rust=$rustName;rust_sha256=$rustHash;source_sha256=$sourceHash;workers=$workers;application_accepted=$false}
[IO.File]::WriteAllText((Join-Path $runtime 'manifest.json'),($record | ConvertTo-Json -Depth 6),[Text.UTF8Encoding]::new($false))
Write-Output "Matrix adapter + shared Rust policy built separately; workers=$workers. No FreeCAD/OM9 native rebuild. Application handoff pending."
