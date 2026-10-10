param([Parameter(Mandatory)][ValidatePattern('^[0-9a-f]{32}$')][string]$RunId)
$ErrorActionPreference='Stop'
$ErrorView='NormalView'
if(Get-Variable PSStyle -ErrorAction SilentlyContinue){$PSStyle.OutputRendering='PlainText'}
$output=[IO.Path]::GetFullPath(('H:/FreeCAD-src/build/matrix-om9-handoff/'+$RunId))
$session=Get-Content -LiteralPath (Join-Path $output 'session.json') -Raw | ConvertFrom-Json
$runtime='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/build/om9-plugin-runtime'
$macro='C:/Users/nguye/.codex/worktrees/layer-session-handoff/FreeCAD-src/Mod/OpenMatrix9/tests/modeling_matrix_handoff_service.FCMacro'
$hostExe='H:/FreeCAD-src/build/relWithDebInfo/bin/FreeCAD.exe'
$plugin=[IO.Path]::GetFullPath((Join-Path $runtime 'bin/OpenMatrix9Gui.pyd'))
if($session.version -ne 1 -or $session.run_id -ne $RunId -or [IO.Path]::GetFullPath($session.output) -ne $output -or [IO.Path]::GetFullPath($session.plugin) -ne $plugin){throw 'Handoff companion ownership mismatch'}
if((Get-FileHash -LiteralPath $hostExe).Hash -ine $session.host_sha256 -or (Get-FileHash -LiteralPath $plugin).Hash -ine $session.plugin_sha256){throw 'Selected shared host/plugin hashes changed before launch'}
# Only the shared FreeCAD host is started, with its own profile/document.
& 'H:/FreeCAD-src/build/run-om9-plugin.ps1' -PluginRuntime $runtime -Macro $macro -OutputDirectory $output -KeepOpen
