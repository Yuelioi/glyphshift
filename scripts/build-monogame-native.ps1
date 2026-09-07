[CmdletBinding()]
param([string]$OutputRoot, [switch]$Fixture)
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$localRoot = [IO.Path]::GetFullPath((Join-Path $repoRoot 'local-test'))
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    . (Join-Path $PSScriptRoot 'cargo-target.ps1')
    $OutputRoot = Join-Path (Get-GlyphshiftCargoTargetDirectory -RepoRoot $repoRoot) 'debug'
}
$output = [IO.Path]::GetFullPath($OutputRoot)
# This directory contains regenerable compiler output; Runtime evidence is staged
# separately by build-runtime-bundle. Explicit caller output paths are supported.
$deps = Join-Path $output 'deps'
New-Item -ItemType Directory -Force $deps | Out-Null
$managedProject = Join-Path $repoRoot 'crates/adapters/implementations/framework/monogame-native/managed/FontFallback.csproj'
$managedOutput = Join-Path $output 'managed-fallback'
& dotnet build $managedProject -c Release --artifacts-path $managedOutput --nologo
if ($LASTEXITCODE -ne 0) { throw 'MonoGame fallback helper build failed.' }
$helperBytes = [IO.File]::ReadAllBytes((Join-Path $managedOutput 'bin/FontFallback/release/Glyphshift.MonoGame.FontFallback.dll'))
$helperInitializer = ($helperBytes | ForEach-Object { $_.ToString() }) -join ','
[IO.File]::WriteAllText((Join-Path $deps 'embedded_fallback.h'), "#pragma once`nstatic const unsigned char fallback_assembly[] = {$helperInitializer};`n")
$headers = @(
    @('cor.h', 'src/coreclr/inc/cor.h', 'A42A94B246E62211D0956A293C22D83EDF74CCCDDC2D2083F2E81DBBFE5B67F6'),
    @('corhdr.h', 'src/coreclr/inc/corhdr.h', 'F061C03977313D58A844E14320F8BDB493D3B5D683C28F38F8466A3D78438F6F'),
    @('corprof.h', 'src/coreclr/pal/prebuilt/inc/corprof.h', '1C5B77BC945AECD31FABF6F7271C0BCA93C60916F5E0D47690EB3972789487F1'),
    @('corerror.h', 'src/coreclr/pal/prebuilt/inc/corerror.h', '44C7495965E958D1891C2D3B14CC864F9E2B8EAA4719274E82B9D4D5E33FEED5')
)
foreach ($header in $headers) {
    $destination = Join-Path $deps $header[0]
    if (-not (Test-Path $destination)) {
        Invoke-WebRequest ('https://raw.githubusercontent.com/dotnet/runtime/v6.0.36/' + $header[1]) -OutFile $destination
    }
    if ((Get-FileHash $destination -Algorithm SHA256).Hash -ne $header[2]) { throw "Header hash mismatch: $($header[0])" }
}
# Generate mechanical no-op implementations from the pinned official COM declarations.
$headerText = Get-Content (Join-Path $deps 'corprof.h') -Raw
$methods = [System.Collections.Generic.List[string]]::new()
foreach ($interface in @('ICorProfilerCallback','ICorProfilerCallback2','ICorProfilerCallback3','ICorProfilerCallback4')) {
    $definition = [regex]::Match($headerText, $interface + ' : public [^{]+\{(.*?)\n    \};', 'Singleline').Groups[1].Value
    foreach ($method in [regex]::Matches($definition, 'virtual HRESULT STDMETHODCALLTYPE (.*?) = 0;', 'Singleline')) {
        $declaration = [regex]::Replace($method.Groups[1].Value, '/\*.*?\*/', '', 'Singleline')
        $methods.Add("HRESULT STDMETHODCALLTYPE $declaration override { return S_OK; }")
    }
}
if ($methods.Count -ne 86) { throw "Unexpected callback count: $($methods.Count)" }
@('#pragma once', '#pragma warning(push)', '#pragma warning(disable:4100)', 'class CallbackBase : public ICorProfilerCallback4 { public:', ($methods -join "`n"), '};', '#pragma warning(pop)') |
    Set-Content (Join-Path $deps 'callback_base.h')
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
if (-not (Test-Path $vswhere)) { throw 'Visual Studio C++ tools discovery is unavailable.' }
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $vs) { throw 'Visual Studio x64 C++ tools are required.' }
$vcvars = Join-Path $vs 'VC/Auxiliary/Build/vcvars64.bat'
$source = Join-Path $repoRoot 'crates/adapters/implementations/framework/monogame-native/native/profiler.cpp'
$dllName = if ($Fixture) { 'glyphshift_coreclr_fixture.dll' } else { 'glyphshift_adapter_monogame_native.dll' }
$dll = Join-Path $output $dllName
$fixtureDefine = if ($Fixture) { '/DGLYPHSHIFT_SYNTHETIC_FIXTURE' } else { '' }
$buildScript = Join-Path $output 'compile.cmd'
# All interpolated values are resolved local filesystem paths, never shell fragments.
@('@echo off', "call `"$vcvars`" >nul", "cl /nologo /std:c++17 /EHsc /W4 /WX /DHOST_WINDOWS $fixtureDefine /LD /O2 /I`"$deps`" `"$source`" /link ole32.lib psapi.lib /EXPORT:DllGetClassObject,PRIVATE /EXPORT:DllCanUnloadNow,PRIVATE /OUT:`"$dll`"") |
    Set-Content $buildScript
Push-Location $output
try { & $buildScript; if ($LASTEXITCODE -ne 0) { throw 'Native CoreCLR fixture build failed.' } }
finally { Pop-Location }
Write-Output "MonoGame native component built: $dllName"
