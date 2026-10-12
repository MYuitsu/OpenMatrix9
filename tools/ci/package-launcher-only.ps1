param([string]$OutputRoot=(Join-Path $env:RUNNER_TEMP 'om9-launcher-package'))
$ErrorActionPreference='Stop'
$source=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
. (Join-Path $source 'launcher/Start-OM9.ps1')
[IO.Directory]::CreateDirectory($OutputRoot)|Out-Null
$metadata=Get-Content -LiteralPath (Join-Path $source 'release.json') -Raw|ConvertFrom-Json
$release=@(Get-OM9ReleaseFeed|Where-Object{$_.tag_name -ceq ('v'+$metadata.version)})
$candidate=Find-OM9Update $release '0.0.0' 0
if(-not $candidate -or $candidate.version -cne $metadata.version){throw 'No matching public native release; build/publish the native version before packaging a launcher revision'}
# Only bootstrap changes: reuse this exact public native release, never a
# local binary or a different module version. Preserve its validation record.
$accepted=Get-OM9UpdateBundle $candidate $OutputRoot
$validation=Get-Content -LiteralPath (Join-Path $accepted 'OpenMatrix9/release-validation.json') -Raw|ConvertFrom-Json
if($validation.version -cne $metadata.version -or $validation.source_commit -cnotmatch '^[a-f0-9]{40}$'){throw 'Accepted native version/source identity mismatch'}
& python (Join-Path $source 'tools/build-launcher-bundle.py') --plugin (Join-Path $accepted 'OpenMatrix9') --output (Join-Path $OutputRoot 'bundle')
if($LASTEXITCODE -ne 0){throw 'Launcher packaging failed'}
@{launcher_source_commit=$env:GITHUB_SHA;native_source_commit=$validation.source_commit;native_build_url=$validation.build_url;native_release=$candidate;native_validation_kind=$validation.validation_kind;native_tests_executed=$validation.tests_executed}|ConvertTo-Json -Depth 6|Set-Content (Join-Path $OutputRoot 'launcher-build.json') -Encoding utf8
