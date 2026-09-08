[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
. (Join-Path $PSScriptRoot 'cargo-target.ps1')
$target = Get-GlyphshiftCargoTargetDirectory -RepoRoot $repoRoot
& cargo test --manifest-path (Join-Path $repoRoot 'Cargo.toml') -p glyphshift-adapter-catsystem2-native --target i686-pc-windows-msvc --lib
if ($LASTEXITCODE -ne 0) { throw 'CatSystem2 unit tests failed.' }
& cargo build --manifest-path (Join-Path $repoRoot 'Cargo.toml') -p glyphshift-adapter-catsystem2-native --target i686-pc-windows-msvc
if ($LASTEXITCODE -ne 0) { throw 'CatSystem2 native build failed.' }
$adapter = Join-Path $target 'i686-pc-windows-msvc/debug/glyphshift_adapter_catsystem2_native.dll'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw 'MSVC x86 tools are required.' }
$vcvars = Join-Path $vs 'VC/Auxiliary/Build/vcvars32.bat'
$source = Join-Path $repoRoot 'test-support/catsystem2-native/fixture.cpp'
foreach ($variant in @(@('standard', ''), @('different-layout', '/DOBJECT_PADDING=17'), @('unrelated', '/DWRONG_TYPE'), @('wrong-buffer', '/DWRONG_MEMBER'), @('wrong-display', '/DWRONG_DISPLAY'))) {
    $output = Join-Path $repoRoot "local-test/evidence/catsystem2-contract/$($variant[0])"
    New-Item -ItemType Directory -Force $output | Out-Null
    $command = Join-Path $output 'compile.cmd'
    @('@echo off', "call `"$vcvars`" >nul", 'if errorlevel 1 exit /b 1',
        "cl /nologo /std:c++17 /utf-8 /EHsc /O2 /Ob0 /Oy- /Gy- $($variant[1]) `"$source`" /Fe:fixture.exe /link /OPT:NOICF") |
        Set-Content -LiteralPath $command -Encoding utf8
    Push-Location $output
    try {
        & $command
        if ($LASTEXITCODE -ne 0) { throw "CatSystem2 fixture compilation failed: $($variant[0])" }
        & ./fixture.exe $adapter
        if ($LASTEXITCODE -ne 0) { throw "CatSystem2 contract failed: $($variant[0])" }
    } finally { Pop-Location }
}
