[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
. (Join-Path $PSScriptRoot 'cargo-target.ps1')
$null = Get-GlyphshiftCargoTargetDirectory -RepoRoot $repoRoot
$outputRoot = Join-Path $repoRoot 'local-test/evidence/native-vgui-localize'
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw 'MSVC x86 tools are required.' }
$vcvars = Join-Path $vs 'VC/Auxiliary/Build/vcvars32.bat'
$source = Join-Path $repoRoot 'test-support/native-framework-abi/vgui_localize.cpp'
$variants = @(
    @('standard', ''),
    @('different-layout', '/DDIFFERENT_OBJECT_LAYOUT'),
    @('wrong-expression', '/DWRONG_VALUE_EXPRESSION'),
    @('wrong-abi', '/DWRONG_QUERY_ABI')
)
foreach ($variant in $variants) {
    $output = Join-Path $outputRoot $variant[0]
    New-Item -ItemType Directory -Force -Path $output | Out-Null
    $command = Join-Path $output 'compile.cmd'
    @('@echo off', "call `"$vcvars`" >nul", 'if errorlevel 1 exit /b 1',
        "cl /nologo /std:c++17 /utf-8 /EHsc /GR /O2 /Ob0 /Oy- /LD $($variant[1]) `"$source`" /link /OUT:vgui2.dll") |
        Set-Content -LiteralPath $command -Encoding utf8
    Push-Location $output
    try {
        & $command
        if ($LASTEXITCODE -ne 0) { throw "VGUI query fixture $($variant[0]) compilation failed." }
    } finally { Pop-Location }
}
$manifest = Join-Path $repoRoot 'Cargo.toml'
& cargo build --manifest-path $manifest --target i686-pc-windows-msvc -p glyphshift-adapter-vgui-localize-native
if ($LASTEXITCODE -ne 0) { throw 'Native VGUI query Adapter build failed.' }
& cargo test --manifest-path $manifest --target i686-pc-windows-msvc -p glyphshift-adapter-vgui-localize-native --lib
if ($LASTEXITCODE -ne 0) { throw 'Native VGUI control lifecycle contracts failed.' }
$previous = $env:GLYPHSHIFT_VGUI_LOCALIZE_FIXTURE_ROOT
try {
    foreach ($variant in $variants) {
        $env:GLYPHSHIFT_VGUI_LOCALIZE_FIXTURE_ROOT = Join-Path $outputRoot $variant[0]
        $test = if ($variant[0].StartsWith('wrong-')) { 'native_localize_rejects_unproved_query_abi' } else { 'native_localize_uses_runtime_publications_and_retains_old_pointers' }
        & cargo test --manifest-path $manifest --target i686-pc-windows-msvc -p glyphshift-target-runtime --test vgui_localize_contract $test -- --ignored --exact --test-threads=1
        if ($LASTEXITCODE -ne 0) { throw "Native VGUI query contract $($variant[0]) failed." }
    }
    $env:GLYPHSHIFT_VGUI_LOCALIZE_FIXTURE_ROOT = Join-Path $outputRoot 'standard'
    foreach ($test in @('native_localize_bounds_storage_across_sessions', 'native_localize_bounds_committed_text_bytes', 'native_localize_gates_reentrancy_errors_generations_and_inflight_calls')) {
        & cargo test --manifest-path $manifest --target i686-pc-windows-msvc -p glyphshift-target-runtime --test vgui_localize_contract $test -- --ignored --exact --test-threads=1
        if ($LASTEXITCODE -ne 0) { throw "Native VGUI query $test contract failed." }
    }
} finally { $env:GLYPHSHIFT_VGUI_LOCALIZE_FIXTURE_ROOT = $previous }
