[CmdletBinding()]
param([ValidateSet('x86', 'x64', 'Both')][string]$Architecture = 'Both')
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw 'MSVC C++ tools are required.' }
$architectures = if ($Architecture -eq 'Both') { @('x86', 'x64') } else { @($Architecture) }
foreach ($arch in $architectures) {
    $output = Join-Path $repoRoot "local-test/evidence/localize-lifecycle/$arch"
    New-Item -ItemType Directory -Force $output | Out-Null
    $vcvarsName = if ($arch -eq 'x86') { 'vcvars32.bat' } else { 'vcvars64.bat' }
    $vcvars = Join-Path $vs "VC/Auxiliary/Build/$vcvarsName"
    $source = Join-Path $repoRoot 'test-support/vgui-localize-lifecycle/lifecycle.cpp'
    $command = Join-Path $output 'compile.cmd'
    @('@echo off', 'chcp 65001 >nul', "call `"$vcvars`" >nul", 'if errorlevel 1 exit /b 1',
        "cl /nologo /std:c++17 /utf-8 /EHsc /W4 /O2 `"$source`" /Fe:lifecycle.exe") |
        Set-Content -LiteralPath $command -Encoding utf8
    Push-Location $output
    try {
        & $command
        if ($LASTEXITCODE -ne 0) { throw "Localize lifecycle $arch compilation failed." }
        & (Join-Path $output 'lifecycle.exe')
        if ($LASTEXITCODE -ne 0) { throw "Localize lifecycle $arch contract failed." }
    } finally { Pop-Location }
}
