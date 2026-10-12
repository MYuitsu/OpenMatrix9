$ErrorActionPreference = 'Stop'
$taskLibrary = 'C:/Program Files/Rhinoceros 5 (64-bit)/Plug-ins/IronPython'
foreach ($taskName in @('Microsoft.Scripting.dll','Microsoft.Dynamic.dll','IronPython.dll','IronPython.Modules.dll')) {
    [void][Reflection.Assembly]::LoadFrom((Join-Path $taskLibrary $taskName))
}
$taskEngine = [IronPython.Hosting.Python]::CreateEngine()
$taskPaths = $taskEngine.GetSearchPaths()
$taskPaths.Add((Join-Path $taskLibrary 'Lib'))
$taskEngine.SetSearchPaths($taskPaths)
$taskScope = $taskEngine.CreateScope()
$taskSource = [IO.File]::ReadAllText('H:/FreeCAD-src/build/om9-dev/tests/rhino5_gui_profiles_probe.py')
$taskFirst = $taskSource.IndexOf('def finite_number(')
$taskLast = $taskSource.IndexOf('def document_measurements(')
$taskHelpers = $taskSource.Substring($taskFirst,$taskLast-$taskFirst)
$taskTests = @'
import math,json,sys
unset=-1.23432101234321e308
assert sample_deviation([[1e200,0,0]],[[0,0,0]])==1e200
assert sample_deviation([[unset,0,0]],[[0,0,0]])==abs(unset)
assert sample_deviation([[unset,unset,unset]],[[0,0,0]]) is None
assert sample_deviation([[float('inf'),0,0]],[[0,0,0]]) is None
assert sample_deviation([[float('nan'),0,0]],[[0,0,0]]) is None
assert sample_deviation([[0.7e-9,0.7e-9,0.7e-9]],[[0,0,0]])>1e-9
payload={'unset':report_number(unset),'nan':report_number(float('nan')),'inf':report_number(float('inf'))}
encoded=json.dumps(payload,ensure_ascii=True,allow_nan=False)
assert json.loads(encoded)['unset']==unset
print('Installed Rhino5 IronPython numeric engine PASS: scaled deviation, Unset/nonfinite diagnostics and strict JSON. No Rhino document/API execution.')
print(sys.version)
'@
[void]$taskEngine.Execute("import math`n"+$taskHelpers+"`n"+$taskTests,$taskScope)
