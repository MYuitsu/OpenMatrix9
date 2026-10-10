$ErrorActionPreference='Stop'
$build='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/build/layer-abi'
$proof='H:/FreeCAD-src/build/om9-layer-edits/docs/validation/layer-session'
& rtk proxy H:/FreeCAD-src/.pixi/envs/default/Library/bin/cmake.exe --build $build --config Release --parallel 14 *> (Join-Path $proof 'layer-collection-native-build.log')
if($LASTEXITCODE -ne 0){throw 'Native tests incremental build failed'}
& rtk proxy H:/FreeCAD-src/.pixi/envs/default/Library/bin/ctest.exe --test-dir $build -C Release --output-on-failure *> (Join-Path $proof 'layer-collection-native-tests.log')
if($LASTEXITCODE -ne 0){throw 'Native tests failed'}
Write-Output 'Native suites passed; full session/Matrix acceptance still pending.'
