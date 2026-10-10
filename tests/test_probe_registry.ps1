$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'rhino5_probe_registry.ps1')
$probe='H:/FreeCAD-src/build/om9-layer-edits/tests/bin/Rhino5LayerCommandProbe.rhp'
$commands=@{}
foreach($name in @('OM9LayerApiPrepareFixture','OM9LayerApiApplyProbe','OM9LayerApiFailLayerProbe','OM9LayerApiFailGeometryProbe','OM9LayerApiSample')){$commands[$name]='2;'+$name}
$facts=@{name='OM9 Layer API Command Probe';dotnet=1;type=16;filename=$probe;subkeys=@('CommandList','PlugIn');commands=$commands}
if(-not(Test-OM9ProbeRegistryFacts $facts $probe)){throw 'Exact diagnostic registration rejected'}
$facts.filename='C:/user/other.rhp'
if(Test-OM9ProbeRegistryFacts $facts $probe){throw 'Foreign plugin path accepted'}
$facts.filename=$probe;$facts.name='Matrix'
if(Test-OM9ProbeRegistryFacts $facts $probe){throw 'Foreign plugin name accepted'}
$facts.name='OM9 Layer API Command Probe';$facts.subkeys+=@('Unknown')
if(Test-OM9ProbeRegistryFacts $facts $probe){throw 'Augmented registry subtree accepted'}
$facts.subkeys=@('CommandList','PlugIn');$facts.commands.ForeignCommand='2;ForeignCommand'
if(Test-OM9ProbeRegistryFacts $facts $probe){throw 'Unexpected native command accepted'}
$facts.commands.Remove('ForeignCommand');$facts.commands.OM9LayerApiSample='2;WrongCommand'
if(Test-OM9ProbeRegistryFacts $facts $probe){throw 'Wrong command registration accepted'}
Write-Output 'Registry ownership guard PASS6; no registry mutation or native runtime acceptance.'

$direct=[IO.Path]::GetFullPath('H:/FreeCAD-src/build/om9-layer-edits/tests/bin/current/Rhino5LayerCurrentProbe.rhp')
$directCommands=@{}
foreach($suffix in @('PrepareFixture','ApplyProbe','FailLayerProbe','FailGeometryProbe','Sample','Cleanup')){$name='OM9LayerDirectApi'+$suffix;$directCommands[$name]='2;'+$name}
$directFacts=@{name='OM9 Matrix Current Session Probe';dotnet=1;type=16;filename=$direct;subkeys=@('CommandList','PlugIn');commands=$directCommands}
if(-not(Test-OM9ProbeRegistryFacts $directFacts $direct 'OM9 Matrix Current Session Probe' 'OM9LayerDirectApi' -CleanupCommand)){throw 'Exact current diagnostic registration rejected'}
$directFacts.commands.Remove('OM9LayerDirectApiCleanup')
if(Test-OM9ProbeRegistryFacts $directFacts $direct 'OM9 Matrix Current Session Probe' 'OM9LayerDirectApi' -CleanupCommand){throw 'Current diagnostic without cleanup accepted'}
$directFacts.commands.OM9LayerDirectApiCleanup='2;OM9LayerDirectApiCleanup';$directFacts.filename=$probe
if(Test-OM9ProbeRegistryFacts $directFacts $direct 'OM9 Matrix Current Session Probe' 'OM9LayerDirectApi' -CleanupCommand){throw 'Isolated binary accepted as current diagnostic'}
Write-Output 'Current-session registry ownership guard PASS3; no registry mutation or native runtime acceptance.'
