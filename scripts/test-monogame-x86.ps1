[CmdletBinding()]
param([string]$DotnetPath = $env:GLYPHSHIFT_TEST_DOTNET)
$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrWhiteSpace($DotnetPath) -or -not (Test-Path -LiteralPath $DotnetPath -PathType Leaf)) {
    throw 'Supply a 32-bit .NET 6 host through -DotnetPath or GLYPHSHIFT_TEST_DOTNET.'
}
$DotnetPath = (Resolve-Path -LiteralPath $DotnetPath).Path
$image = [IO.File]::ReadAllBytes($DotnetPath)
$pe = if ($image.Length -ge 64) { [BitConverter]::ToUInt32($image,60) } else { 0 }
if ($pe -lt 64 -or $pe -gt $image.Length - 6 -or
    [BitConverter]::ToUInt32($image,$pe) -ne 0x4550 -or
    [BitConverter]::ToUInt16($image,$pe+4) -ne 0x14c) {
    throw 'The supplied .NET host must be an x86 PE executable.'
}
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
. (Join-Path $PSScriptRoot 'cargo-target.ps1')
$targetRoot = Get-GlyphshiftCargoTargetDirectory -RepoRoot $repoRoot
$nativeProfileRoot = Join-Path $targetRoot 'i686-pc-windows-msvc/debug'
$evidence = Join-Path $repoRoot 'local-test/evidence/monogame-x86'
$managed = Join-Path $evidence 'managed'
& (Join-Path $PSScriptRoot 'build-monogame-native.ps1') -Architecture x86 -OutputRoot $nativeProfileRoot
& dotnet build (Join-Path $repoRoot 'test-support/monogame-text/Host/Host.csproj') -c Release --artifacts-path $managed --nologo
if ($LASTEXITCODE -ne 0) { throw 'MonoGame synthetic host build failed.' }
$values = @{
    GLYPHSHIFT_TEST_DOTNET = $DotnetPath
    GLYPHSHIFT_MONOGAME_NATIVE = (Join-Path $nativeProfileRoot 'glyphshift_adapter_monogame_native.dll')
    GLYPHSHIFT_MONOGAME_MANAGED = (Join-Path $managed 'bin')
    GLYPHSHIFT_MONOGAME_EVIDENCE = (Join-Path $evidence 'pixels')
    GLYPHSHIFT_FALLBACK_REFERENCE = (Join-Path $nativeProfileRoot 'managed-fallback/bin/FontFallback/release/Glyphshift.MonoGame.FontFallback.dll')
    GLYPHSHIFT_MONOGAME_PRODUCTION = '1'
    GLYPHSHIFT_TEST_RUNTIME = $null
    GLYPHSHIFT_TEST_DEPLOYMENTS = $null
}
$previous = @{}
foreach ($name in $values.Keys) { $previous[$name] = [Environment]::GetEnvironmentVariable($name) }
try {
    foreach ($name in $values.Keys) {
        if ($null -eq $values[$name]) { Remove-Item -LiteralPath "Env:$name" -ErrorAction SilentlyContinue }
        else { [Environment]::SetEnvironmentVariable($name, $values[$name]) }
    }
    & node (Join-Path $repoRoot 'apps/glyphshift-desktop/node_modules/@playwright/test/cli.js') test --config (Join-Path $repoRoot 'test-support/monogame-text/playwright.config.cjs')
    if ($LASTEXITCODE -ne 0) { throw 'MonoGame x86 pixel contracts failed.' }
} finally {
    foreach ($name in $previous.Keys) {
        if ($null -eq $previous[$name]) { Remove-Item -LiteralPath "Env:$name" -ErrorAction SilentlyContinue }
        else { [Environment]::SetEnvironmentVariable($name, $previous[$name]) }
    }
}
