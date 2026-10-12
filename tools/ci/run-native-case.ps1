param(
 [Parameter(Mandatory=$true)][string]$HostRoot,
 [Parameter(Mandatory=$true)][string]$PluginRoot,
 [Parameter(Mandatory=$true)][string]$Macro,
 [Parameter(Mandatory=$true)][string]$Output,
 [ValidateRange(1,600)][int]$TimeoutSeconds=180
)
$ErrorActionPreference='Stop'
foreach($name in @('HostRoot','PluginRoot','Macro','Output')){
 Set-Variable -Name $name -Value ([IO.Path]::GetFullPath((Get-Variable -Name $name -ValueOnly)))
}
$exe=Join-Path $HostRoot 'bin/FreeCAD.exe';$plugin=Join-Path $PluginRoot 'bin/OpenMatrix9Gui.pyd'
foreach($file in @($exe,$plugin,$Macro,(Join-Path $PluginRoot 'InitGui.py'))){
 if(-not(Test-Path -LiteralPath $file -PathType Leaf)){throw "Native CI input missing: $file"}
}
if(Test-Path -LiteralPath (Join-Path $Output 'results.json')){throw 'Native CI output already contains a report'}
New-Item -ItemType Directory -Path $Output -Force|Out-Null
$keys=@('PATH','TEMP','TMP','FREECAD_USER_HOME','OM9_SMOKE_OUTPUT','OM9_TEST_NATIVE_DIR','OM9_TEST_PLUGIN','OM9_TEST_HOST','OM9_PLUGIN_ROOT','QT_OPENGL','QT_OPENGL_DLL','PYTHONHOME','PYTHONPATH','QT_PLUGIN_PATH','QT_QPA_PLATFORM_PLUGIN_PATH')
$old=@{};foreach($key in $keys){$old[$key]=[Environment]::GetEnvironmentVariable($key,'Process')}
$before=(Get-FileHash -LiteralPath $plugin).Hash
try{
 $env:PATH="$HostRoot/bin;$PluginRoot/bin;$env:SystemRoot/System32;$env:SystemRoot"
 foreach($key in @('PYTHONHOME','PYTHONPATH','QT_PLUGIN_PATH','QT_QPA_PLATFORM_PLUGIN_PATH')){Remove-Item -LiteralPath "Env:$key" -ErrorAction SilentlyContinue}
 $env:TEMP=Join-Path $Output 'temporary';$env:TMP=$env:TEMP
 New-Item -ItemType Directory -Path $env:TEMP -Force|Out-Null
 $env:FREECAD_USER_HOME=Join-Path $Output 'profile';$env:OM9_SMOKE_OUTPUT=$Output
 $env:OM9_TEST_NATIVE_DIR=Join-Path $PluginRoot 'bin';$env:OM9_TEST_PLUGIN=$PluginRoot;$env:OM9_TEST_HOST=$HostRoot;$env:OM9_PLUGIN_ROOT=$PluginRoot
 . (Join-Path $PSScriptRoot 'gui-profile.ps1')
 Initialize-OM9CIGuiProfile $HostRoot $Output|Out-Null
 $macroLiteral=$Macro|ConvertTo-Json -Compress
 $bootstrap=@"
import os, sys
from pathlib import Path
expected = Path(os.environ['OM9_TEST_NATIVE_DIR']) / 'OpenMatrix9Gui.pyd'
loaded = sys.modules.get('OpenMatrix9Gui')
if loaded is not None and Path(loaded.__file__).resolve() != expected.resolve():
    raise RuntimeError('A different native plugin was already loaded: ' + loaded.__file__)
sys.path.insert(0, str(expected.parent))
module_root = Path(os.environ['OM9_TEST_PLUGIN'])
sys.path.insert(0, str(module_root))
import FreeCADGui as Gui
if 'OpenMatrix9Workbench' in Gui.listWorkbenches():
    Gui.removeWorkbench('OpenMatrix9Workbench')
init_gui = module_root / 'InitGui.py'
exec(compile(init_gui.read_bytes(), str(init_gui), 'exec'), dict(__name__='__main__', __file__=str(init_gui)))
# A hidden Win32 launch can leave Qt's window unexposed while isVisible is true.
# Expose the owned test window before fixtures take handle/viewport snapshots.
from PySide6 import QtTest, QtWidgets
main_window = Gui.getMainWindow()
main_window.hide()
# Fixed FBO captures reuse geometry sized from the live viewport. Give the
# owned fixture the same logical client size on local and hosted desktops;
# keep QT_SCALE_FACTOR independent so the DPI cases still exercise Qt scaling.
main_window.setFixedSize(1800, 1000)
main_window.showNormal()
if not QtTest.QTest.qWaitForWindowExposed(main_window, 5000):
    raise RuntimeError('The isolated native test window was never exposed')
QtWidgets.QApplication.processEvents()
fixture = dict(requested=[1800, 1000], actual=[main_window.width(), main_window.height()],
               device_pixel_ratio=main_window.devicePixelRatioF(), exposed=main_window.windowHandle().isExposed())
import json
(Path(os.environ['OM9_SMOKE_OUTPUT']) / 'window-fixture.json').write_text(json.dumps(fixture), encoding='utf-8')
if fixture['actual'] != fixture['requested'] or not fixture['exposed']:
    raise RuntimeError('The isolated native fixture did not reach its requested client size')
script = Path($macroLiteral)
sys.path.insert(0, str(script.parent))
exec(compile(script.read_bytes(), str(script), 'exec'), dict(__name__='__main__', __file__=str(script)))
"@
 $bootstrapPath=Join-Path $Output 'select-plugin.FCMacro'
 Set-Content -LiteralPath $bootstrapPath -Value $bootstrap -Encoding utf8
 $arguments=@('-M',('"'+$PluginRoot+'"'),'-P',('"'+(Join-Path $PluginRoot 'bin')+'"'),'-u',('"'+(Join-Path $Output 'user.cfg')+'"'),'-s',('"'+(Join-Path $Output 'system.cfg')+'"'),('"'+$bootstrapPath+'"'))
 $process=Start-Process -FilePath $exe -ArgumentList $arguments -WorkingDirectory $HostRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $Output 'stdout.log') -RedirectStandardError (Join-Path $Output 'stderr.log')
 $ownedHandle=$process.Handle
 $launch=@{host=$exe;host_sha256=(Get-FileHash -LiteralPath $exe).Hash;plugin=$plugin;plugin_sha256=$before;macro=$Macro;pid=$process.Id;exit_code=$null}
 $launch|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Output 'launch.json') -Encoding utf8
 if(-not $process.WaitForExit($TimeoutSeconds*1000)){$process.Kill();throw "Native host timeout: $Macro; see $Output"}
 $launch.exit_code=$process.ExitCode
 $launch|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Output 'launch.json') -Encoding utf8
 $report=Get-Content -LiteralPath (Join-Path $Output 'results.json') -Raw|ConvertFrom-Json
 if($process.ExitCode -ne 0 -or -not $report.ok -or @($report.checks).Count -eq 0 -or @($report.checks|Where-Object {-not $_.passed}).Count){throw "Native CI regression failed: $Macro $($report.error); see $Output"}
 if(-not $report.module -or [IO.Path]::GetFullPath($report.module) -ne $plugin){throw "Wrong native module selected: $($report.module)"}
 if((Get-FileHash -LiteralPath $plugin).Hash -ne $before){throw 'Native plugin changed during the test'}
 Write-Output "Native PASS; checks=$(@($report.checks).Count); macro=$(Split-Path $Macro -Leaf)"
 return $report
}finally{foreach($key in $old.Keys){[Environment]::SetEnvironmentVariable($key,$old[$key],'Process')}}
