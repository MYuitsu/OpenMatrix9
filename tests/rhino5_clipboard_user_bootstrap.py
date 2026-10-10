# Keep Rhino5's RunPythonScript editor entry short; report top-level failures.
import os,traceback,json
output=os.environ['OM9_RHINO_CLIPBOARD_OUTPUT']
with open(os.path.join(output,'bootstrap-start.txt'),'w') as stream:stream.write('RunPythonScript entered')
try:
    with open('H:/FreeCAD-src/build/om9-perf-dev/tests/rhino5_clipboard_user.py','rb') as source_stream:code=source_stream.read()
    code=code.replace('\r\n','\n')+'\n'
    compiled=compile(code,'<clipboard-matrix>','exec')
    with open(os.path.join(output,'bootstrap-compiled.txt'),'w') as stream:stream.write(str(len(code)))
    exec(compiled,globals())
except:
    with open(os.path.join(output,'bootstrap-error.txt'),'w') as stream:stream.write(traceback.format_exc())
