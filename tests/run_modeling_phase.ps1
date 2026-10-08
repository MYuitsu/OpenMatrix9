param(
 [ValidateRange(1,5)][int]$Phase=1,
 [Parameter(Mandatory)][string]$FreeCADExe,
 [Parameter(Mandatory)][string]$DependencyPrefix,
 [Parameter(Mandatory)][string]$NativeTestDirectory
)
$ErrorActionPreference='Stop'
if($Phase -ne 1){throw "Phase $Phase has no completed runtime gate yet"}
if(-not [IO.Path]::IsPathRooted($FreeCADExe) -or -not [IO.Path]::IsPathRooted($DependencyPrefix) -or -not [IO.Path]::IsPathRooted($NativeTestDirectory)){throw 'Use absolute executable, dependency and native-test paths'}
$runtime=Split-Path (Split-Path $FreeCADExe -Parent) -Parent
$manifest=Join-Path $runtime 'modeling-build.json'
$data=Get-Content -LiteralPath $manifest -Raw | ConvertFrom-Json
foreach($entry in $data.source_sha256.PSObject.Properties){
 $source=Join-Path $data.source_root $entry.Name
 if((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.Value){throw "Build source mismatch: $($entry.Name)"}
}
foreach($entry in $data.runtime_sha256.PSObject.Properties){
 if((Get-FileHash -LiteralPath (Join-Path $runtime $entry.Name) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $entry.Value){throw "Runtime hash mismatch: $($entry.Name)"}
}
$oldManifest=$env:OM9_MODELING_MANIFEST
$oldFixtures=$env:OM9_MODELING_FIXTURES
$oldFailureWorker=$env:OM9_MODELING_FAILURE_WORKER
$oldWorkerFixture=$env:OM9_IMPORT_WORKER_FIXTURE
try {
 $env:OM9_MODELING_MANIFEST=$manifest
 $env:OM9_MODELING_FIXTURES=Join-Path $NativeTestDirectory 'modeling-fixtures'
 $env:OM9_MODELING_FAILURE_WORKER=Join-Path $NativeTestDirectory 'ThreeDmImportFailureWorker.exe'
 $env:OM9_IMPORT_WORKER_FIXTURE=Join-Path $NativeTestDirectory 'import-worker-fixture.3dm'
 foreach($macro in @('modeling_baseline_smoke.FCMacro','modeling_exchange_smoke.FCMacro','modeling_workers_smoke.FCMacro','modeling_menu_smoke.FCMacro','modeling_container_smoke.FCMacro','three_dm_exchange_menu_smoke.FCMacro','three_dm_import_workers_smoke.FCMacro')){
  & (Join-Path $PSScriptRoot 'run_menu_smoke.ps1') -FreeCADExe $FreeCADExe -DependencyPrefix $DependencyPrefix -Macro $macro -TimeoutSeconds 180
 }
} finally {$env:OM9_MODELING_MANIFEST=$oldManifest;$env:OM9_MODELING_FIXTURES=$oldFixtures;$env:OM9_MODELING_FAILURE_WORKER=$oldFailureWorker;$env:OM9_IMPORT_WORKER_FIXTURE=$oldWorkerFixture}
