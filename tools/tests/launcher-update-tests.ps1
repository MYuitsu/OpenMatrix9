param([string]$OutputRoot='H:/FreeCAD-src/build/om9-launcher-update-tests')
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot '../../launcher/Start-OM9.ps1')
$run=Join-Path $OutputRoot ([guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($run)|Out-Null
[IO.File]::Copy((Join-Path $PSScriptRoot '../../launcher/Start-OM9.ps1'),(Join-Path $run 'Start-OM9.ps1'))
$script:checks=@()
function Check($name,$value){$script:checks+=@{name=$name;passed=[bool]$value};if(-not $value){throw $name}}
function Release($version,$revision=1,$draft=$false){
 $name="OpenMatrix9-$version-SetupLauncher";if($revision -gt 1){$name+="-r$revision"};$name+='-Windows-x64.zip'
 return [pscustomobject]@{tag_name="v$version";draft=$draft;prerelease=$true;html_url="https://github.com/MYuitsu/OpenMatrix9/releases/tag/v$version";assets=@([pscustomobject]@{name=$name;browser_download_url="https://github.com/MYuitsu/OpenMatrix9/releases/download/v$version/$name"},[pscustomobject]@{name='SHA256SUMS.txt';browser_download_url="https://github.com/MYuitsu/OpenMatrix9/releases/download/v$version/SHA256SUMS.txt"})}
}
$same=Release '0.0.2' 2
Check 'installed release has no update' ($null -eq (Find-OM9Update @($same) '0.0.2' 2))
Check 'older development release cannot trigger downgrade' ($null -eq (Find-OM9Update @((Release '0.0.1')) '0.0.2' 2))
$new=Find-OM9Update @((Release '0.0.3')) '0.0.2' 2
Check 'new application version is offered' ($new.version -eq '0.0.3' -and $new.prerelease)
Check 'launcher revision update is offered' ((Find-OM9Update @((Release '0.0.2' 3)) '0.0.2' 2).revision -eq 3)
Check 'draft release is excluded' ($null -eq (Find-OM9Update @((Release '0.0.3' 1 $true)) '0.0.2' 2))
$wrong=Release '0.0.4';$wrong.assets[0].browser_download_url='https://example.com/foreign.zip'
Check 'foreign download endpoint is excluded' ($null -eq (Find-OM9Update @($wrong) '0.0.2' 2))
$wrong=Release '0.0.4';$wrong.assets[0].name='OpenMatrix9-0.0.4-Linux.zip'
Check 'unsupported platform asset is excluded' ($null -eq (Find-OM9Update @($wrong) '0.0.2' 2))
Check 'versions compare numerically' ((Find-OM9Update @((Release '0.0.9'),(Release '0.0.10')) '0.0.2' 2).version -eq '0.0.10')
function Get-OM9ReleaseFeed {throw 'offline test'}
$offline=Invoke-OM9UpdateCheck $run '0.0.2' 2 $true
Check 'offline check returns status without breaking launch' ($offline.status -eq 'unavailable')
Check 'offline diagnostic is recorded' (([IO.File]::ReadAllText((Join-Path $run 'update-check.json'))|ConvertFrom-Json).status -eq 'unavailable')
function Get-OM9ReleaseFeed {return @((Release '0.0.3'))}
$script:notifications=0
function Show-OM9Update($Update){$script:notifications++;return 'Later'}
$available=Invoke-OM9UpdateCheck $run '0.0.2' 2 $true
Check 'silent check records available update without UI' ($available.status -eq 'available' -and $script:notifications -eq 0)
Invoke-OM9UpdateCheck $run '0.0.2' 2 $false|Out-Null
Check 'new release notification appears at startup' ($script:notifications -eq 1)
Invoke-OM9UpdateCheck $run '0.0.2' 2 $false|Out-Null
Check 'declined update remains available on next startup' ($script:notifications -eq 2)
$desktop=Join-Path $run 'Desktop';$menu=Join-Path $run 'Programs/OpenMatrix9'
$first=New-OM9Shortcut $run $desktop 'C:\stock\FreeCAD.exe' $run $true
$second=New-OM9StartMenuShortcut $run $menu 'C:\stock\FreeCAD.exe' $run $true
$shell=New-Object -ComObject WScript.Shell;$a=$shell.CreateShortcut($first);$b=$shell.CreateShortcut($second)
Check 'Desktop and Start Menu shortcuts both exist' ([IO.File]::Exists($first) -and [IO.File]::Exists($second))
Check 'both shortcuts use the same persistent launcher and icon' ($a.Arguments -eq $b.Arguments -and $a.IconLocation -eq $b.IconLocation -and $a.Arguments -match '\-Installed')
$redirectTarget=Join-Path $run 'ProgramsTarget';[IO.Directory]::CreateDirectory($redirectTarget)|Out-Null
$redirect=Join-Path $run 'RedirectedPrograms';New-Item -ItemType Junction -Path $redirect -Target $redirectTarget|Out-Null
function Get-OM9ProgramsFolder {return $redirect}
try {
 $redirected=New-OM9StartMenuShortcut $run '' 'C:\stock\FreeCAD.exe' $run $true
 Check 'redirected Windows Programs Known Folder creates a working shortcut' ([IO.File]::Exists((Join-Path $redirectTarget 'OpenMatrix9/OpenMatrix9.lnk')))
 $c=$shell.CreateShortcut($redirected)
 Check 'redirected Start Menu still uses the persistent launcher' ($c.Arguments -eq $a.Arguments)
} finally{[IO.Directory]::Delete($redirect)}
@{ok=$true;checks=$script:checks}|ConvertTo-Json -Depth 5|Set-Content (Join-Path $run 'results.json')
Write-Output "Launcher update tests PASS: $($script:checks.Count); $run"
