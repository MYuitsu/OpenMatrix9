import os,traceback
out=os.environ['OM9_RHINO_PHASE2_OUTPUT']
try:
    with open('H:/FreeCAD-src/build/om9-perf-dev/tests/rhino5_phase2_manual.py','rb') as stream:code=stream.read().replace('\r\n','\n')+'\n'
    exec(compile(code,'<phase2-matrix>','exec'),globals())
except:
    with open(os.path.join(out,'bootstrap-error.txt'),'w') as stream:stream.write(traceback.format_exc())
