[CmdletBinding()]
param()
$ErrorActionPreference='Stop'
$repoRoot=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
& (Join-Path $repoRoot 'test-support/native-framework-abi/build-vgui.ps1')
& cargo build --manifest-path (Join-Path $repoRoot 'Cargo.toml') --target i686-pc-windows-msvc -p glyphshift-adapter-vgui-runs-native -p glyphshift-adapter-gdi-native
if($LASTEXITCODE -ne 0){throw 'VGUI/GDI native build failed.'}
$previous=$env:GLYPHSHIFT_VGUI_FIXTURE_ROOT
try{
    $env:GLYPHSHIFT_VGUI_FIXTURE_ROOT=Join-Path $repoRoot 'local-test/evidence/native-framework-abi/vgui-x86'
    & cargo test --manifest-path (Join-Path $repoRoot 'Cargo.toml') --target i686-pc-windows-msvc -p glyphshift-target-runtime --test vgui_deferred_contract --test vgui_rejection_contract -- --ignored --test-threads=1
    if($LASTEXITCODE -ne 0){throw 'VGUI deferred drawing contracts failed.'}
}finally{$env:GLYPHSHIFT_VGUI_FIXTURE_ROOT=$previous}
