[CmdletBinding()]
param()
$ErrorActionPreference='Stop'
$repoRoot=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$output=Join-Path $repoRoot 'local-test/evidence/native-framework-abi/vgui-x86'
New-Item -ItemType Directory -Force $output | Out-Null
$vswhere=Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs=& $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if(-not $vs){throw 'MSVC x86 tools are required.'}
$vcvars=Join-Path $vs 'VC/Auxiliary/Build/vcvars32.bat'
foreach($fixture in @(@('vgui_surface.cpp','vguimatsurface.dll',''),@('vgui_image.cpp','caption-a.dll',''),@('vgui_image.cpp','caption-b.dll','/DDIFFERENT_OBJECT_LAYOUT'))){
    $source=Join-Path $PSScriptRoot $fixture[0]
    $command=Join-Path $output 'compile.cmd'
    @('@echo off',"call `"$vcvars`" >nul","cl /nologo /std:c++17 /EHsc /GR /O2 /Ob0 /Oy- /LD $($fixture[2]) `"$source`" /link gdi32.lib /OUT:$($fixture[1])")|Set-Content $command
    Push-Location $output
    try{& $command;if($LASTEXITCODE -ne 0){throw 'VGUI synthetic ABI fixture build failed.'}}finally{Pop-Location}
}
