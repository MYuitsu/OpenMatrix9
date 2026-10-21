# SPDX-License-Identifier: LGPL-2.1-or-later
# Import the developer environment into this PowerShell process and its children.
function Initialize-OM9Msvc {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $vs = & $vswhere -latest -version '[17.0,18.0)' -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $vs) { throw 'MSVC x64 toolchain unavailable' }
    $vcvars = Join-Path $vs 'VC/Auxiliary/Build/vcvars64.bat'
    $compilerEnvironment = & cmd.exe /d /s /c "`"$vcvars`" >nul && set"
    if ($LASTEXITCODE -ne 0) { throw 'Cannot initialize MSVC x64 developer environment' }
    foreach ($line in $compilerEnvironment) {
        if ($line -match '^([^=]+)=(.*)$') {
            [Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process')
        }
    }
    if ($env:VSCMD_ARG_TGT_ARCH -ne 'x64' -or -not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
        throw 'Expected the MSVC x64 compiler'
    }
    $env:CC = 'cl.exe'
    $env:CXX = 'cl.exe'
    $env:CMAKE_BUILD_PARALLEL_LEVEL = '2'
}
