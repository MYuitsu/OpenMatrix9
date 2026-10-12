param([string]$OutputRoot='H:/FreeCAD-src/build/om9-launcher-live-update-tests')
# Explicit network + installed FreeCAD test; never invoked on a user's normal profile.
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot '../../launcher/Start-OM9.ps1')
$run=Join-Path $OutputRoot ([guid]::NewGuid().ToString('N'))
$profile=Join-Path $run 'profile';$cache=Join-Path $profile 'v1-1/OM9-launcher'
[IO.Directory]::CreateDirectory($cache)|Out-Null
[IO.File]::Copy((Join-Path $PSScriptRoot '../../launcher/Start-OM9.ps1'),(Join-Path $cache 'Start-OM9.ps1'))
$checks=@();$script:liveOwnedHost=$null
function Check($name,$value){$script:checks+=@{name=$name;passed=[bool]$value};if(-not $value){throw $name}}
$hostExe=Get-OM9Host '' $profile
$update=Find-OM9Update (Get-OM9ReleaseFeed) '0.0.2' 2
Check 'GitHub startup query finds an official newer release with checksum' ($update -and $update.checksum_url)
$env:OM9_LAUNCH_TEST_HIDDEN='1';$env:OM9_LAUNCH_TEST_CLOSE='1'
Add-Type -AssemblyName System.Windows.Forms
$ui=@{clicked=$false}
$timer=New-Object Windows.Forms.Timer;$timer.Interval=100
$timer.Add_Tick({foreach($form in @([Windows.Forms.Application]::OpenForms)){
 if($form.Text -eq 'OpenMatrix9 update'){
  $ui.clicked=$true;@($form.Controls|Where-Object{$_ -is [Windows.Forms.Button] -and $_.Text -eq 'Update'})[0].PerformClick()
 }
 if($form.Text -eq 'Updating OpenMatrix9'){$form.Hide()}
}}.GetNewClosure())
$timer.Start()
try{
 Check 'actual Update button offers automatic download' ((Show-OM9Update $update) -eq 'Update' -and $ui.clicked)
 $receipt=Invoke-OM9Update $update $cache $hostExe
 Check 'public HTTPS archive downloads verifies and installs automatically' ($receipt.status -eq 'installed' -and (Read-OM9Manifest (Join-Path $cache 'launcher-manifest.json')).version -eq $update.version)
 $deadline=[DateTime]::UtcNow.AddSeconds(90)
 do{
  $reports=@(Get-ChildItem -LiteralPath $cache -Filter 'start-*.json')
  if($reports.Count){break};Start-Sleep -Milliseconds 300
 }while([DateTime]::UtcNow -lt $deadline)
 Check 'automatic updater starts real FreeCAD after installing' ($reports.Count -gt 0)
 $startup=Get-Content -LiteralPath $reports[0].FullName -Raw|ConvertFrom-Json
 Check 'reopened application activates OM9 from the isolated updated profile' ($startup.ok -and $startup.workbench -eq 'OpenMatrix9Workbench' -and $startup.module.StartsWith($profile,[StringComparison]::OrdinalIgnoreCase))
 Check 'reopened host uses official FreeCAD 1.1.4' (($startup.version[0..2] -join '.') -eq '1.1.4')
 @{ok=$true;checks=$checks;release=$update;receipt=$receipt;startup=$startup}|ConvertTo-Json -Depth 7|Set-Content (Join-Path $run 'results.json')
 Write-Output "Live Update + FreeCAD reopen PASS: $($checks.Count); $run"
}finally{$timer.Stop();$timer.Dispose()}
