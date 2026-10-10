"""Host test glue: precise native OCCT mass oracle, separately hashed executable."""
from pathlib import Path
import json, subprocess, os, hashlib
EXECUTABLE=Path('H:/FreeCAD-src/build/om9-phase3-native/ThreeDmModelingBrepTests.exe')
def measure(shape,path):
    path=Path(path);shape.exportBrep(str(path))
    environment=os.environ.copy()
    environment['PATH']='H:/FreeCAD-src/.pixi/envs/default/Library/bin;'+environment.get('PATH','')
    result=subprocess.run([str(EXECUTABLE),str(path)],capture_output=True,text=True,timeout=30,env=environment,creationflags=subprocess.CREATE_NO_WINDOW)
    if result.returncode:raise RuntimeError('Native mass oracle: '+result.stdout+result.stderr)
    values=json.loads(result.stdout)
    # OCCT's adaptive mass on extrusion surfaces misses underlying rational
    # curve knot spans. Exact NURBS conversion exposes spans without resampling.
    values['area_raw_adaptive']=values['area'];values['volume_raw_adaptive']=values['volume']
    values['area']=values['area_nurbs'];values['volume']=values['volume_nurbs']
    values.update(brep_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),oracle_sha256=hashlib.sha256(EXECUTABLE.read_bytes()).hexdigest())
    return values
