param([string]$Root=$env:OM9_CI_ROOT)
$ErrorActionPreference='Stop'
$source=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
$paths=Get-Content -LiteralPath (Join-Path $Root 'build-paths.json') -Raw|ConvertFrom-Json
$evidence=Join-Path $Root 'evidence';New-Item -ItemType Directory -Path $evidence -Force|Out-Null
$python=Join-Path $source 'tools/ci/.pixi/envs/default/python.exe'
foreach($test in @('test_release_package.py','test_public_source_audit.py')){
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
if($rustTests -lt 262){throw 'Expected complete Rust regression suite'}
# Reinitialize the same matching MSVC environment for the independent native oracle.
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
$vc=Join-Path $vs 'VC/Auxiliary/Build/vcvars64.bat'
$toolEnv=& cmd.exe /d /s /c "`"$vc`" >nul && set"
if($LASTEXITCODE -ne 0){throw 'Cannot initialize MSVC'}
foreach($line in $toolEnv){if($line -match '^([^=]+)=(.*)$'){[Environment]::SetEnvironmentVariable($matches[1],$matches[2],'Process')}}
& cmake --build $paths.build --target ThreeDmModelingBrepTests --parallel $paths.workers
if($LASTEXITCODE -ne 0){throw 'Native oracle build failed'}
$env:PATH="$($paths.host)/bin;$env:SystemRoot/System32;$env:SystemRoot"
foreach($key in @('PYTHONHOME','PYTHONPATH','QT_PLUGIN_PATH','QT_QPA_PLATFORM_PLUGIN_PATH')){Remove-Item -LiteralPath "Env:$key" -ErrorAction SilentlyContinue}
& (Join-Path $paths.build 'ThreeDmModelingBrepTests.exe') *> (Join-Path $evidence 'brep.log')
if($LASTEXITCODE -ne 0){Get-Content -LiteralPath (Join-Path $evidence 'brep.log');throw 'Eight independent analytic BRep fixtures failed'}
$reports=@()
. (Join-Path $PSScriptRoot 'gui-profile.ps1')
foreach($macro in @('native-smoke.FCMacro','worker-smoke.FCMacro')){
 $output=Join-Path $evidence $macro;New-Item -ItemType Directory -Path $output -Force|Out-Null
 $env:FREECAD_USER_HOME=Join-Path $output 'profile';$env:OM9_SMOKE_OUTPUT=$output;$env:OM9_TEST_PLUGIN=$paths.plugin;$env:OM9_TEST_HOST=$paths.host
 Initialize-OM9CIGuiProfile $paths.host $output|Out-Null
 Write-Output "Starting $macro with the official host software OpenGL renderer."
 $arguments=@('-M',('"'+$paths.plugin+'"'),'-u',('"'+(Join-Path $output 'user.cfg')+'"'),'-s',('"'+(Join-Path $output 'system.cfg')+'"'),('"'+(Join-Path $PSScriptRoot $macro)+'"'))
 $process=Start-Process -FilePath (Join-Path $paths.host 'bin/FreeCAD.exe') -ArgumentList $arguments -WorkingDirectory $paths.host -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $output 'stdout.log') -RedirectStandardError (Join-Path $output 'stderr.log')
 if(-not $process.WaitForExit(180000)){$process.Kill();throw "Owned CI host timeout: $macro; see $output"}
 if($process.ExitCode -ne 0){throw "CI host exit $($process.ExitCode): $macro; see $output"}
 $report=Get-Content -LiteralPath (Join-Path $output 'results.json') -Raw|ConvertFrom-Json
 if(-not $report.ok){throw "Native host regression failed: $macro $($report.error)"}
 $reports+=@{macro=$macro;checks=@($report.checks).Count}
}
$validation=@{ok=$true;host='Official FreeCAD 1.1.4 Windows x64';source_commit=$env:GITHUB_SHA;build_url="https://github.com/$env:GITHUB_REPOSITORY/actions/runs/$env:GITHUB_RUN_ID";rust_tests=$rustTests;brep_fixtures=8;native_reports=$reports;
 plugin_sha256=(Get-FileHash -LiteralPath (Join-Path $paths.plugin 'bin/OpenMatrix9Gui.pyd')).Hash.ToLowerInvariant();worker_sha256=(Get-FileHash -LiteralPath (Join-Path $paths.plugin 'bin/OM9ThreeDmImportWorker.exe')).Hash.ToLowerInvariant()}
$validation|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $paths.plugin 'release-validation.json') -Encoding utf8
$validation|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $evidence 'summary.json') -Encoding utf8
Write-Output "Verified $rustTests Rust tests, eight analytic BRep fixtures and official FreeCAD native integration."
