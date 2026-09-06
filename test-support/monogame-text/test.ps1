[CmdletBinding()]
param([switch]$Fixture, [switch]$NativeOnly)
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$output = Join-Path $repoRoot 'local-test/monogame-text'
$evidence = Join-Path $repoRoot 'local-test/evidence/monogame-text'
New-Item -ItemType Directory -Force $evidence | Out-Null
& (Join-Path $PSScriptRoot '../coreclr-late-attach/build.ps1')
if (-not $Fixture) { & (Join-Path $repoRoot 'scripts/build-monogame-native.ps1') }
foreach ($project in @('Host', 'Attach')) {
    & dotnet build (Join-Path $PSScriptRoot "$project/$project.csproj") -c Release --artifacts-path (Join-Path $output 'managed') --nologo
    if ($LASTEXITCODE -ne 0) { throw "MonoGame test build failed: $project" }
}
& dotnet build (Join-Path $PSScriptRoot 'TextureCodec/TextureCodec.csproj') -c Release --artifacts-path (Join-Path $output 'codec') --nologo
if ($LASTEXITCODE -ne 0) { throw 'Texture codec contract build failed.' }
& dotnet (Join-Path $output 'codec/bin/TextureCodec/release/TextureCodec.dll')
if ($LASTEXITCODE -ne 0) { throw 'Texture codec contract failed.' }
$previousDotnet = $env:GLYPHSHIFT_TEST_DOTNET
$previousProduction = $env:GLYPHSHIFT_MONOGAME_PRODUCTION
$previousRuntime = $env:GLYPHSHIFT_TEST_RUNTIME
$previousDeployments = $env:GLYPHSHIFT_TEST_DEPLOYMENTS
$previousReference = $env:GLYPHSHIFT_FALLBACK_REFERENCE
try {
    $env:GLYPHSHIFT_TEST_DOTNET = (Get-Command dotnet -ErrorAction Stop).Source
    $env:GLYPHSHIFT_FALLBACK_REFERENCE = Join-Path $repoRoot 'local-test/monogame-native/managed-fallback/bin/FontFallback/release/Glyphshift.MonoGame.FontFallback.dll'
    $env:GLYPHSHIFT_MONOGAME_PRODUCTION = if ($Fixture) { '0' } else { '1' }
    $env:GLYPHSHIFT_TEST_RUNTIME = $null
    $env:GLYPHSHIFT_TEST_DEPLOYMENTS = $null
    if (-not $Fixture -and -not $NativeOnly) {
        $native = Join-Path $repoRoot 'local-test/monogame-native/glyphshift_adapter_monogame_native.dll'
        $runtimeBuild = Join-Path $repoRoot 'local-test/monogame-runtime-build'
        $deploymentRoot = Join-Path $repoRoot 'local-test/evidence/monogame-runtime'
        & cargo run --manifest-path (Join-Path $repoRoot 'Cargo.toml') --target-dir $runtimeBuild -p glyphshift-target-runtime --example monogame_deployment -- $native $deploymentRoot
        if ($LASTEXITCODE -ne 0) { throw 'Runtime deployment fixture failed.' }
        & cargo build --manifest-path (Join-Path $repoRoot 'Cargo.toml') --target-dir $runtimeBuild -p glyphshift-target-runtime
        if ($LASTEXITCODE -ne 0) { throw 'Runtime build failed.' }
        $env:GLYPHSHIFT_TEST_RUNTIME = Join-Path $runtimeBuild 'debug/glyphshift_target_runtime.dll'
        $env:GLYPHSHIFT_TEST_DEPLOYMENTS = $deploymentRoot
    }
    & node (Join-Path $repoRoot 'apps/glyphshift-desktop/node_modules/@playwright/test/cli.js') test --config (Join-Path $PSScriptRoot 'playwright.config.cjs') 2>&1 |
        Tee-Object (Join-Path $evidence 'verification.log')
    if ($LASTEXITCODE -ne 0) { throw 'MonoGame rendering verification failed.' }
} finally {
    $env:GLYPHSHIFT_TEST_DOTNET = $previousDotnet
    $env:GLYPHSHIFT_MONOGAME_PRODUCTION = $previousProduction
    $env:GLYPHSHIFT_TEST_RUNTIME = $previousRuntime
    $env:GLYPHSHIFT_TEST_DEPLOYMENTS = $previousDeployments
    $env:GLYPHSHIFT_FALLBACK_REFERENCE = $previousReference
}
