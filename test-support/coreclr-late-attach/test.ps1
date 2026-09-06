[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
& (Join-Path $PSScriptRoot 'build.ps1')
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$output = Join-Path $repoRoot 'local-test/coreclr-late-attach'
$evidence = Join-Path $repoRoot 'local-test/evidence/coreclr-late-attach'
New-Item -ItemType Directory -Force $evidence | Out-Null
$dotnet = (Get-Command dotnet -ErrorAction Stop).Source
$hostAssembly = Join-Path $output 'managed/bin/SyntheticHost/release/Glyphshift.CoreClr.SyntheticHost.dll'
$runner = Join-Path $output 'managed/bin/Runner/release/Runner.dll'
$native = Join-Path $output 'glyphshift_coreclr_fixture.dll'
& $dotnet $runner $dotnet $hostAssembly $native 2>&1 | Tee-Object (Join-Path $evidence 'verification.log')
if ($LASTEXITCODE -ne 0) { throw 'CoreCLR synthetic verification failed.' }
$previousFixture = $env:GLYPHSHIFT_CORECLR_FIXTURE_DLL
try {
    $env:GLYPHSHIFT_CORECLR_FIXTURE_DLL = $native
    & cargo test --manifest-path (Join-Path $repoRoot 'Cargo.toml') --target-dir (Join-Path $output 'cargo-target') -p glyphshift-adapter-native-host --test coreclr_extension_fixture -- --ignored 2>&1 |
        Tee-Object (Join-Path $evidence 'native-abi-verification.log')
    if ($LASTEXITCODE -ne 0) { throw 'CoreCLR Native ABI interoperability failed.' }
} finally { $env:GLYPHSHIFT_CORECLR_FIXTURE_DLL = $previousFixture }
