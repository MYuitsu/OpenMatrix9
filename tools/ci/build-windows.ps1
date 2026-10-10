param([string]$Root=(Join-Path $PSScriptRoot '../../build/ci'))
$ErrorActionPreference='Stop'
$source=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$Root=[IO.Path]::GetFullPath($Root)
New-Item -ItemType Directory -Path $Root -Force|Out-Null
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if(-not $vs){throw 'MSVC x64 toolchain unavailable'}
$vc=Join-Path $vs 'VC/Auxiliary/Build/vcvars64.bat'
$toolEnv=& cmd.exe /d /s /c "`"$vc`" >nul && set"
if($LASTEXITCODE -ne 0){throw 'Cannot initialize MSVC developer environment'}
foreach($line in $toolEnv){if($line -match '^([^=]+)=(.*)$'){[Environment]::SetEnvironmentVariable($matches[1],$matches[2],'Process')}}
$archive=Join-Path $Root 'FreeCAD.7z'
if(-not (Test-Path -LiteralPath $archive)){
 Invoke-WebRequest 'https://github.com/FreeCAD/FreeCAD/releases/download/1.1.4/FreeCAD_1.1.4-Windows-x86_64-py311.7z' -OutFile $archive
}
if((Get-FileHash -LiteralPath $archive).Hash.ToLowerInvariant() -ne '4828741fc91ee37fafcdb97a1abacb18b04ba451ac4372d9ff7a7349b36f4d6d'){throw 'Official FreeCAD archive checksum mismatch'}
$hostRoot=Join-Path $Root 'stock-host'
if(-not (Test-Path -LiteralPath $hostRoot)){
 & 7z x $archive "-o$hostRoot" -y|Out-Null
 if($LASTEXITCODE -ne 0){throw 'FreeCAD archive extraction failed'}
}
$executables=@(Get-ChildItem -LiteralPath $hostRoot -Filter FreeCAD.exe -File -Recurse)
if($executables.Count -ne 1){throw 'Expected one official FreeCAD host'}
$stock=Split-Path (Split-Path $executables[0].FullName)
$python=Join-Path $source 'tools/ci/.pixi/envs/default/python.exe'
$prefix=Join-Path $source 'tools/ci/.pixi/envs/default/Library'
& $python (Join-Path $PSScriptRoot 'prepare_sdk.py') --root $Root --host $stock
if($LASTEXITCODE -ne 0){throw 'Stock SDK preparation failed'}
$workers=[Math]::Max(1,[int][Math]::Floor([Environment]::ProcessorCount*0.6))
$env:CARGO_BUILD_JOBS=[string]$workers
Write-Output "Building standalone OpenMatrix9Gui with $workers workers (60% logical CPUs); FreeCAD is imported, never compiled."
$build=Join-Path $Root 'plugin-build';$runtime=Join-Path $Root 'runtime'
& cmake -S $source -B $build -G Ninja -DCMAKE_BUILD_TYPE=RelWithDebInfo "-DFREECAD_SOURCE_DIR=$Root/sdk-source" "-DFREECAD_SDK_BUILD=$Root/sdk" "-DFREECAD_DEPENDENCY_PREFIX=$prefix" "-DCMAKE_PREFIX_PATH=$prefix" "-DOPENMATRIX9_STOCK_HOST=$stock" "-DOPENMATRIX9_RUNTIME_ROOT=$runtime"
if($LASTEXITCODE -ne 0){throw 'Standalone plugin configure failed'}
& cmake --build $build --target OpenMatrix9Gui --parallel $workers
if($LASTEXITCODE -ne 0){throw 'Standalone plugin build failed'}
$plugin=Join-Path $runtime 'Mod/OpenMatrix9';$bin=Join-Path $plugin 'bin'
New-Item -ItemType Directory -Path $bin -Force|Out-Null
foreach($name in @('OpenMatrix9Gui.pyd','OM9ThreeDmImportWorker.exe')){Copy-Item -LiteralPath (Join-Path $runtime "bin/$name") -Destination (Join-Path $bin $name)}
foreach($name in @('LICENSE','THIRD_PARTY_NOTICES.md')){Copy-Item -LiteralPath (Join-Path $source $name) -Destination $plugin}
@{host=$stock;plugin=$plugin;build=$build;workers=$workers}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Root 'build-paths.json') -Encoding utf8
if($env:GITHUB_ENV){"OM9_CI_ROOT=$Root"|Out-File -FilePath $env:GITHUB_ENV -Append -Encoding utf8}
