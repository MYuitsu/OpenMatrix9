function Initialize-OM9CISoftwareRenderer([string]$HostRoot,[string]$OwnedHostRoot) {
 $hostPath=[IO.Path]::GetFullPath($HostRoot)
 $ownedPath=[IO.Path]::GetFullPath($OwnedHostRoot).TrimEnd([IO.Path]::DirectorySeparatorChar,[IO.Path]::AltDirectorySeparatorChar)
 if(-not $hostPath.StartsWith($ownedPath+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){
  throw 'Software renderer deployment requires a host inside the disposable CI extraction root'
 }
 # Only the downloaded CI host is writable. Refuse links into another host.
 $directory=Get-Item -LiteralPath (Join-Path $hostPath 'bin')
 while($directory){
  if($directory.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'CI renderer deployment must not traverse a host link'}
  if($directory.FullName -ieq $ownedPath){break}
  $directory=$directory.Parent
 }
 if(-not $directory){throw 'Disposable CI extraction root not found'}
 $bundled=Join-Path $hostPath 'bin/opengl32sw.dll';$alias=Join-Path $hostPath 'bin/opengl32.dll'
 if(-not(Test-Path -LiteralPath $bundled -PathType Leaf)){throw 'Official host software OpenGL renderer missing'}
 if((Get-Item -LiteralPath $bundled).Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Bundled CI renderer must be an extracted file'}
 $existing=@(Get-ChildItem -LiteralPath (Join-Path $hostPath 'bin') -Force|Where-Object {$_.Name -ieq 'opengl32.dll'})
 if($existing.Count){
  if($existing[0].Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Do not replace a renderer link'}
  if((Get-FileHash -LiteralPath $alias).Hash -ne (Get-FileHash -LiteralPath $bundled).Hash){throw 'Existing OpenGL renderer conflicts with the official bundled Mesa'}
 }else{Copy-Item -LiteralPath $bundled -Destination $alias}
 return $alias
}

function Initialize-OM9CIGuiProfile([string]$HostRoot,[string]$Output) {
 $bundled=Join-Path $HostRoot 'bin/opengl32sw.dll';$renderer=Join-Path $HostRoot 'bin/opengl32.dll'
 foreach($file in @($bundled,$renderer)){
  if(-not(Test-Path -LiteralPath $file -PathType Leaf)){throw 'Official host software OpenGL renderer missing'}
 }
 if((Get-FileHash -LiteralPath $renderer).Hash -ne (Get-FileHash -LiteralPath $bundled).Hash){throw 'Qt and native Coin must use the same official Mesa renderer'}
 New-Item -ItemType Directory -Path (Join-Path $Output 'profile') -Force|Out-Null
 # StartupProcess::setupApplication reads this before constructing QApplication.
 # Hosted Windows runners expose OpenGL 1.1; use the official host's bundled Mesa.
 $configuration=@'
<?xml version="1.0" encoding="UTF-8"?>
<FCParameters>
 <FCParamGroup Name="Root">
  <FCParamGroup Name="BaseApp">
   <FCParamGroup Name="Preferences">
    <FCParamGroup Name="OpenGL"><FCBool Name="UseSoftwareOpenGL" Value="1"/></FCParamGroup>
    <FCParamGroup Name="View"><FCText Name="SavePicture">FramebufferObject</FCText></FCParamGroup>
   </FCParamGroup>
  </FCParamGroup>
 </FCParamGroup>
</FCParameters>
'@
 $configuration|Set-Content -LiteralPath (Join-Path $Output 'user.cfg') -Encoding utf8
 $env:QT_OPENGL='software'
 $env:QT_OPENGL_DLL=$renderer
 return (Join-Path $Output 'user.cfg')
}
