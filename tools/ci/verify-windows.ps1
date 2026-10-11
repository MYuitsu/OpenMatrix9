param([string]$Root=$env:OM9_CI_ROOT)
$ErrorActionPreference='Stop'
$source=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$paths=Get-Content -LiteralPath (Join-Path $Root 'build-paths.json') -Raw|ConvertFrom-Json
$evidence=Join-Path $Root 'evidence';New-Item -ItemType Directory -Path $evidence -Force|Out-Null
$python=Join-Path $source 'tools/ci/.pixi/envs/default/python.exe'
foreach($test in @('test_release_package.py','test_timestamp_release.py','test_public_source_audit.py')){
 & $python -m unittest discover -s (Join-Path $source 'tools/tests') -p $test
 if($LASTEXITCODE -ne 0){throw "Public release tooling regression failed: $test"}
}
& (Join-Path $source 'tools/tests/launcher-tests.ps1') -OutputRoot (Join-Path $evidence 'installer')
& (Join-Path $source 'tools/tests/launcher-update-tests.ps1') -OutputRoot (Join-Path $evidence 'updater')
& (Join-Path $source 'tools/tests/stock-host-tests.ps1') -OutputRoot (Join-Path $evidence 'stock-host')
& (Join-Path $source 'tools/tests/ci-gui-profile-tests.ps1') -OutputRoot (Join-Path $evidence 'gui-profile')
$env:CARGO_TARGET_DIR=Join-Path $paths.build 'Gui/rust-target'
$env:CARGO_BUILD_JOBS=[string]$paths.workers
& cargo test --locked --release --manifest-path (Join-Path $source 'rust/Cargo.toml') *> (Join-Path $evidence 'rust-tests.log')
if($LASTEXITCODE -ne 0){Get-Content -LiteralPath (Join-Path $evidence 'rust-tests.log');throw 'Rust regression failed'}
$rustTests=0
foreach($line in Get-Content -LiteralPath (Join-Path $evidence 'rust-tests.log')){if($line -match 'test result: ok\. (\d+) passed;'){$rustTests+=[int]$matches[1]}}
if($rustTests -lt 653){throw 'Expected complete Rust and Gumball dependency regression suite'}
# Reinitialize the same matching MSVC environment for the independent native oracle.
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
$vc=Join-Path $vs 'VC/Auxiliary/Build/vcvars64.bat'
$toolEnv=& cmd.exe /d /s /c "`"$vc`" >nul && set"
if($LASTEXITCODE -ne 0){throw 'Cannot initialize MSVC'}
foreach($line in $toolEnv){if($line -match '^([^=]+)=(.*)$'){[Environment]::SetEnvironmentVariable($matches[1],$matches[2],'Process')}}
& cmake --build $paths.build --target ThreeDmModelingBrepTests OM9ThreeDmHeaderOrderTest --parallel $paths.workers
if($LASTEXITCODE -ne 0){throw 'Native oracle build failed'}
$env:PATH="$($paths.host)/bin;$env:SystemRoot/System32;$env:SystemRoot"
foreach($key in @('PYTHONHOME','PYTHONPATH','QT_PLUGIN_PATH','QT_QPA_PLATFORM_PLUGIN_PATH')){Remove-Item -LiteralPath "Env:$key" -ErrorAction SilentlyContinue}
& (Join-Path $paths.build 'OM9ThreeDmHeaderOrderTest.exe')
if($LASTEXITCODE -ne 0){throw 'Stock host/kernel header order or system zlib regression failed'}
& (Join-Path $paths.build 'ThreeDmModelingBrepTests.exe') *> (Join-Path $evidence 'brep.log')
if($LASTEXITCODE -ne 0){Get-Content -LiteralPath (Join-Path $evidence 'brep.log');throw 'Eight independent analytic BRep fixtures failed'}
$reports=@()
foreach($macro in @('native-smoke.FCMacro','worker-smoke.FCMacro')){
 $output=Join-Path $evidence $macro
 $report=@(& (Join-Path $PSScriptRoot 'run-native-case.ps1') -HostRoot $paths.host -PluginRoot $paths.plugin -Macro (Join-Path $PSScriptRoot $macro) -Output $output)|Select-Object -Last 1
 $reports+=@{macro=$macro;checks=@($report.checks).Count}
}
& (Join-Path $PSScriptRoot 'run-gumball.ps1') -HostRoot $paths.host -PluginRoot $paths.plugin -Output (Join-Path $evidence 'gumball')
$gumball=Get-Content -LiteralPath (Join-Path $evidence 'gumball/suite.json') -Raw|ConvertFrom-Json
if(-not $gumball.ok -or @($gumball.runs).Count -ne 30){throw 'Complete native Gumball suite required'}
$releaseMetadataPath=Join-Path $source 'release.json'
$releaseMetadata=Get-Content -LiteralPath $releaseMetadataPath -Raw|ConvertFrom-Json
$validation=@{ok=$true;version=$releaseMetadata.version;release_metadata_sha256=(Get-FileHash -LiteralPath $releaseMetadataPath).Hash.ToLowerInvariant();host='Official FreeCAD 1.1.4 Windows x64';source_commit=$env:GITHUB_SHA;build_url="https://github.com/$env:GITHUB_REPOSITORY/actions/runs/$env:GITHUB_RUN_ID";rust_tests=$rustTests;brep_fixtures=8;native_reports=$reports;
 gumball=$gumball;
 plugin_sha256=(Get-FileHash -LiteralPath (Join-Path $paths.plugin 'bin/OpenMatrix9Gui.pyd')).Hash.ToLowerInvariant();worker_sha256=(Get-FileHash -LiteralPath (Join-Path $paths.plugin 'bin/OM9ThreeDmImportWorker.exe')).Hash.ToLowerInvariant()}
$validation|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $paths.plugin 'release-validation.json') -Encoding utf8
$validation|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $evidence 'summary.json') -Encoding utf8
Write-Output "Verified $rustTests Rust tests, eight analytic BRep fixtures, 30 native Gumball cases and official FreeCAD integration."
