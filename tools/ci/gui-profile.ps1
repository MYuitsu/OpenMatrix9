function Initialize-OM9CIGuiProfile([string]$HostRoot,[string]$Output) {
 $renderer=Join-Path $HostRoot 'bin/opengl32sw.dll'
 if(-not(Test-Path -LiteralPath $renderer -PathType Leaf)){throw 'Official host software OpenGL renderer missing'}
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
