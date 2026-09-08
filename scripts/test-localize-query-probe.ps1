[CmdletBinding()]
param([switch]$Replacement)
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$output = Join-Path $repoRoot 'local-test/evidence/localize-query-fixture'
New-Item -ItemType Directory -Force $output | Out-Null
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw 'MSVC x86 tools are required.' }
$vcvars = Join-Path $vs 'VC/Auxiliary/Build/vcvars32.bat'
$source = Join-Path $repoRoot 'test-support/vgui-localize-probe/fixture.cpp'
$command = Join-Path $output 'compile.cmd'
@('@echo off', 'chcp 65001 >nul', "call `"$vcvars`" >nul", 'if errorlevel 1 exit /b 1',
    "cl /nologo /std:c++17 /utf-8 /EHsc /W4 /O2 /Ob0 `"$source`" /Fe:query-fixture.exe") |
    Set-Content -LiteralPath $command -Encoding utf8
Push-Location $output
try {
    & $command
    if ($LASTEXITCODE -ne 0) { throw 'Query fixture compilation failed.' }
    $extra = @()
    if ($Replacement) { $extra += '--allow-replacement' }
    & python (Join-Path $repoRoot 'test-support/vgui-localize-probe/run.py') `
        --fixture (Join-Path $output 'query-fixture.exe') --output (Join-Path $output 'queries.jsonl') `
        @extra
    if ($LASTEXITCODE -ne 0) { throw 'Query probe contract failed.' }
} finally { Pop-Location }
