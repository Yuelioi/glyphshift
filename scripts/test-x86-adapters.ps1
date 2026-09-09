[CmdletBinding()]
param([ValidateSet('x86','x64')][string]$Architecture = 'x86')
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
. (Join-Path $PSScriptRoot 'cargo-target.ps1')
$null = Get-GlyphshiftCargoTargetDirectory -RepoRoot $repoRoot
$targetArgs = if ($Architecture -eq 'x86') { @('--target','i686-pc-windows-msvc') } else { @() }
$packages = @('glyphshift-adapter-gdi-native','glyphshift-adapter-gdiplus-native','glyphshift-adapter-directwrite-native',
    'glyphshift-adapter-qt-painter-native','glyphshift-adapter-gtk3-pango-native',
    'glyphshift-adapter-raylib-native','glyphshift-adapter-unity-mono-standard-ui-native')
$buildArgs = @('build','--manifest-path',(Join-Path $repoRoot 'Cargo.toml')) + $targetArgs
foreach ($package in $packages) { $buildArgs += @('-p',$package) }
& cargo @buildArgs
if ($LASTEXITCODE -ne 0) { throw 'Native adapters did not build.' }
& (Join-Path $repoRoot 'test-support/native-framework-abi/build.ps1') -Architecture $Architecture
$previousRoot = $env:GLYPHSHIFT_FRAMEWORK_ABI_ROOT
try {
    $env:GLYPHSHIFT_FRAMEWORK_ABI_ROOT = Join-Path $repoRoot "local-test/evidence/native-framework-abi/$Architecture"
    $common = @('test','--manifest-path',(Join-Path $repoRoot 'Cargo.toml')) + $targetArgs + @('-p','glyphshift-adapter-native-host')
    & cargo @common --test native_gdiplus_activation_contract --test native_directwrite_activation_contract -- --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'System drawing contracts failed.' }
    & cargo @common --test native_qt_x86_abi_contract --test native_c_framework_abi_contract --test native_unity_mono_abi_contract -- --ignored --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'Framework ABI contracts failed.' }
    $runtimeArgs = @('test','--manifest-path',(Join-Path $repoRoot 'Cargo.toml')) + $targetArgs + @('-p','glyphshift-target-runtime')
    & cargo @runtimeArgs --test text_scope_contract -- --ignored --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw 'Complete-text and subordinate GDI capture contract failed.' }
} finally { $env:GLYPHSHIFT_FRAMEWORK_ABI_ROOT = $previousRoot }
