@echo off
rem Run the OM9 shell with the GPU cluster renderer: occlusion culling and a VRAM budget on.
rem
rem   tools\run-gpu-cluster.cmd            (default: 2000 boxes, occlusion on, 512 MiB budget)
rem   tools\run-gpu-cluster.cmd 8000       (more boxes)
rem
rem What to look for: the Perspective viewport shows the boxes; right-drag orbits, middle-drag pans,
rem wheel zooms. Occlusion culls boxes hidden behind other boxes, and the budget keeps only part of
rem the cluster data in VRAM, which is the path that lets a 2 GB archive open.

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
set "OM9_GPU_CLUSTER=1"
set "OM9_GPU_OCCLUSION=1"
set "OM9_GPU_STREAM_MB=512"
set "OM9_GPU_STRESS=%~1"
if not defined OM9_GPU_STRESS set "OM9_GPU_STRESS=2000"
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=L:\om9-target"

echo [om9] building in release with the gpu-sphere feature (target dir %CARGO_TARGET_DIR%)
cargo build --release -p om9-app --features gpu-sphere --bin om9 -j 2
if errorlevel 1 exit /b 1

echo [om9] cluster path: %OM9_GPU_STRESS% boxes, occlusion on, budget %OM9_GPU_STREAM_MB% MiB
echo [om9] right-drag orbits, middle-drag pans, wheel zooms. Close the window to stop.
"%CARGO_TARGET_DIR%\release\om9.exe"
endlocal
