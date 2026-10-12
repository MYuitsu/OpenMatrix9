@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" >nul
set "PATH=H:\FreeCAD-src\.pixi\envs\default\Library\bin;H:\FreeCAD-src\.pixi\envs\default;C:\Users\nguye\.cargo\bin;%PATH%"
cd /d H:\FreeCAD-src
cmake -S build/om9-dev -B build/openmatrix9-preservation -G Ninja -DCMAKE_C_COMPILER=cl -DCMAKE_CXX_COMPILER=cl -DCMAKE_BUILD_TYPE=RelWithDebInfo -DFREECAD_SOURCE_DIR=H:/FreeCAD-src -DFREECAD_SDK_BUILD=H:/FreeCAD-src/build/3dm-preservation-sdk -DOPENMATRIX9_RUNTIME_ROOT=H:/FreeCAD-src/build/om9-ui-3dm-gil-sdk -DFREECAD_DEPENDENCY_PREFIX=H:/FreeCAD-src/.pixi/envs/default/Library -DCMAKE_PREFIX_PATH=H:/FreeCAD-src/.pixi/envs/default/Library -DFETCHCONTENT_SOURCE_DIR_OM9_OPENNURBS=H:/FreeCAD-src/build/dependencies/opennurbs
if errorlevel 1 exit /b 1
cmake --build build/openmatrix9-preservation --target OpenMatrix9Gui --parallel 20
exit /b %errorlevel%
