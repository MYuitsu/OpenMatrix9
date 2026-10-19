@echo off
rem Run the built standalone OM9 shell with its OCCT/TBB/jemalloc runtime on PATH.
rem
rem The executable links the OCCT DLLs dynamically, so Explorer/double-click has no way to find
rem TKernel.dll and the rest unless they sit beside om9.exe. This script is the source-tree way to
rem start it:  tools\run-om9.cmd            (new document)
rem            tools\run-om9.cmd --ui x.om9 (open a document)
rem
rem The runtime comes from the local OCCT SDK cache; nothing is downloaded.

setlocal
cd /d "%~dp0.."

set "SDK=%CD%\build\standalone-deps\occt-8.0.1-sdk"
set "TBB=%CD%\build\standalone-deps\occt-8.0.1-3rdparty\3rdparty-vc14-64\tbb-2021.13.0-x64"
set "JEMALLOC=%CD%\build\standalone-deps\occt-8.0.1-3rdparty\3rdparty-vc14-64\jemalloc-vc14-64"

if not exist "%SDK%\bin\TKernel.dll" (
    echo [om9] OCCT runtime not found at %SDK%\bin
    exit /b 1
)

set "PATH=%SDK%\bin;%TBB%\bin;%JEMALLOC%\bin;%PATH%"
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=L:\om9-target"

if not exist "%CARGO_TARGET_DIR%\release\om9.exe" (
    echo [om9] %CARGO_TARGET_DIR%\release\om9.exe not found.
    echo [om9] build it first:  cargo build --release -p om9-app -j 2
    exit /b 1
)

"%CARGO_TARGET_DIR%\release\om9.exe" %*
endlocal
