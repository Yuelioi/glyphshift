[CmdletBinding()]
param([ValidateSet('x86','x64')][string]$Architecture = 'x86')
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$output = Join-Path $repoRoot "local-test/evidence/native-framework-abi/$Architecture"
New-Item -ItemType Directory -Force $output | Out-Null
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw 'MSVC tools are required.' }
$vcvars = Join-Path $vs "VC/Auxiliary/Build/$(if ($Architecture -eq 'x86') { 'vcvars32.bat' } else { 'vcvars64.bat' })"
foreach ($fixture in @(@('qt5.cpp','Qt5Gui.dll'), @('gtk3.cpp','gtk3-fixture.dll'), @('raylib.cpp','raylib.dll'), @('mono.cpp','mono-2.0-bdwgc.dll'))) {
$source = Join-Path $PSScriptRoot $fixture[0]
$build = Join-Path $output 'compile.cmd'
@('@echo off', "call `"$vcvars`" >nul", "cl /nologo /std:c++17 /EHsc /W4 /WX /Od /Ob0 /LD `"$source`" /link gdi32.lib /OUT:$($fixture[1])") | Set-Content $build
Push-Location $output
try {
    & $build
    if ($LASTEXITCODE -ne 0) { throw 'Synthetic Qt ABI fixture failed to build.' }
    Copy-Item -LiteralPath (Join-Path $output 'Qt5Gui.dll') -Destination (Join-Path $output 'Qt5Core.dll')
} finally { Pop-Location }
}
Write-Output "Synthetic framework ABI fixture ready ($Architecture)."
