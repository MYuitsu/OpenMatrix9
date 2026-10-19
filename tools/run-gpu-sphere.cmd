@echo off
rem Run the OM9 shell with the OCCT sphere viewport demo (M01-L2-01-06 manual check).
rem
rem Works from cmd.exe:  tools\run-gpu-sphere.cmd
rem The demo needs the OCCT runtime DLLs on PATH and the `gpu-sphere` cargo feature.
rem Nothing is downloaded; the SDK comes from implementation\tools\fetch_occt.py if missing.

setlocal
cd /d "%~dp0.."

set "SDK=%CD%\build\standalone-deps\occt-8.0.1-sdk"
set "TBB=%CD%\build\standalone-deps\occt-8.0.1-3rdparty\3rdparty-vc14-64\tbb-2021.13.0-x64"
set "JEMALLOC=%CD%\build\standalone-deps\occt-8.0.1-3rdparty\3rdparty-vc14-64\jemalloc-vc14-64"

if not exist "%SDK%\bin\TKG3d.dll" (
    echo [om9] OCCT SDK not materialised at %SDK%
    echo [om9] run: python implementation\tools\fetch_occt.py
    exit /b 1
)

set "PATH=%SDK%\bin;%TBB%\bin;%JEMALLOC%\bin;%PATH%"
set "OM9_OCCT_SDK=%SDK%"
set "OM9_GPU_SPHERE=1"
rem Optional first argument: number of stress objects, e.g. tools\run-gpu-sphere.cmd 2000
if not "%~1"=="" set "OM9_GPU_STRESS=%~1"
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=L:\om9-target"

echo [om9] building in release with the gpu-sphere feature (target dir %CARGO_TARGET_DIR%)
rem Release matters: a debug egui/wgpu build is several times slower and makes the camera drag stutter.
cargo build --release -p om9-app --features gpu-sphere --bin om9 -j 2
if errorlevel 1 exit /b 1

echo [om9] starting. Mouse in a viewport: right-drag orbits, right-drag+Shift or middle-drag pans,
echo [om9] wheel zooms. Close the window to stop.
"%CARGO_TARGET_DIR%\release\om9.exe"
endlocal
