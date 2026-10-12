param([string]$OutputRoot=(Join-Path $env:RUNNER_TEMP 'om9-launcher-package'))
$ErrorActionPreference='Stop'
$source=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../..'))
. (Join-Path $source 'launcher/Start-OM9.ps1')
[IO.Directory]::CreateDirectory($OutputRoot)|Out-Null
$archive=Join-Path $OutputRoot 'accepted-native.zip'
# Native code is unchanged: reuse the immutable, publicly built v0.0.3 payload.
# This is a launcher revision package, not a newly built native release.
Save-OM9ReleaseAsset 'https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.0.3/OpenMatrix9-0.0.3-SetupLauncher-Windows-x64.zip' $archive 256MB
if((Get-OM9Hash $archive) -cne 'c30ece976e08d2678a4ba77f3e6fead7607cb18e5fc1d43773f6883b53ebd466'){throw 'Accepted public native package checksum mismatch'}
$accepted=Join-Path $OutputRoot 'accepted';Expand-OM9UpdateArchive $archive $accepted
$validation=Get-Content -LiteralPath (Join-Path $accepted 'OpenMatrix9/release-validation.json') -Raw|ConvertFrom-Json
if($validation.source_commit -cne '2330e20af4aa5b050eaa928f7728766c02a3924d'){throw 'Accepted native source commit mismatch'}
& python (Join-Path $source 'tools/build-launcher-bundle.py') --plugin (Join-Path $accepted 'OpenMatrix9') --output (Join-Path $OutputRoot 'bundle')
if($LASTEXITCODE -ne 0){throw 'Launcher packaging failed'}
@{launcher_source_commit=$env:GITHUB_SHA;native_source_commit=$validation.source_commit;native_build_url=$validation.build_url;native_archive_sha256=(Get-OM9Hash $archive)}|ConvertTo-Json|Set-Content (Join-Path $OutputRoot 'launcher-build.json') -Encoding utf8
