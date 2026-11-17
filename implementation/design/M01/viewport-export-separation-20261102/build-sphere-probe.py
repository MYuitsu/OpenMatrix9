"""Build the isolated OCCT topology probe into the existing target directory."""
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[4]
target = Path('L:/om9-target')
sdk = root / 'build/standalone-deps/occt-8.0.1-sdk'
vc = r'C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat'
source = Path(__file__).with_name('sphere-topology-probe.cpp')
libs = 'TKernel.lib TKMath.lib TKG2d.lib TKG3d.lib TKGeomBase.lib TKBRep.lib TKGeomAlgo.lib TKTopAlgo.lib TKPrim.lib TKBO.lib TKBool.lib TKShHealing.lib'
script = target / 'sphere-probe-build.cmd'
script.write_text(f'@call "{vc}" -arch=x64 >nul\n@cl /nologo /EHsc /std:c++17 /I "{sdk / "include/opencascade"}" "{source}" /Fo:"{target / "sphere-topology-probe.obj"}" /Fe:"{target / "sphere-topology-probe.exe"}" /link /LIBPATH:"{sdk / "lib"}" {libs}\n')
subprocess.run(['cmd', '/c', str(script)], cwd=target, check=True)
