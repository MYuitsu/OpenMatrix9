$ErrorActionPreference='Stop'
. 'H:/FreeCAD-src/build/om9-layer-edits/tests/rhino5_probe_registry.ps1'
$probe='H:/FreeCAD-src/build/om9-layer-edits/tests/bin/Rhino5LayerCommandProbe.rhp'
$pluginId='d2f0f5d0-4e65-4b75-b4ef-8d6ca72be839'
$facts=Get-OM9ProbeRegistryFacts ([Microsoft.Win32.Registry]::LocalMachine) ('Software\McNeel\Rhinoceros\5.0x64\Plug-ins\'+$pluginId)
if(-not(Test-OM9ProbeRegistryFacts $facts $probe)){throw 'Actual HKLM facts do not match owned diagnostic'}
$run=Assert-OM9PriorProbeOwnership 'H:/FreeCAD-src/build/matrix-layer-api-probe/efebe48877d84de3925dbaf8408ea6ca' $probe $pluginId
Write-Output "Read-only registry ownership check PASS: exact own plugin and dead completed run $run; no registry mutation."
