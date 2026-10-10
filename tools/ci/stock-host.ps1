function Find-OM9StockHost([string]$Root) {
 # Official portable bundle contains a root FreeCAD.exe wrapper AND bin/freecad.exe.
 # The native host lives beside FreeCADGui.dll and uses the sibling lib/Part.pyd.
 $executables=@(Get-ChildItem -LiteralPath $Root -Filter FreeCAD.exe -File -Recurse)
 $candidates=@($executables|Where-Object{
  $_.Directory.Name -ieq 'bin' -and
  (Test-Path -LiteralPath (Join-Path $_.Directory.FullName 'FreeCADGui.dll')) -and
  (Test-Path -LiteralPath (Join-Path $_.Directory.Parent.FullName 'lib/Part.pyd'))
 })
 if($candidates.Count -ne 1){throw ("Expected one complete native FreeCAD host; found $($candidates.Count). Executables: "+(($executables.FullName) -join ', '))}
 return $candidates[0].Directory.Parent.FullName
}
